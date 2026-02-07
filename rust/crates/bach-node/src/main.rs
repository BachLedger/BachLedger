//! BachLedger node binary
//!
//! This is the main entry point for running a BachLedger node.

mod cli;
mod config;
mod consensus_driver;
mod genesis;
mod node;

use anyhow::Result;
use bach_consensus::{TbftConfig, TbftConsensus, Validator, ValidatorSet};
use bach_crypto::public_key_to_address;
use bach_network::{NetworkConfig, NetworkService, PeerId};
use bach_rpc::{RpcHandler, RpcServer, ServerConfig};
use cli::Cli;
use config::{
    default_genesis_config, BlockConfig, ConsensusConfig, GenesisConfig, NodeConfig, P2pConfig,
    RpcConfig,
};
use consensus_driver::ConsensusDriver;
use k256::ecdsa::SigningKey;
use node::Node;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    tracing::info!("BachLedger node starting...");

    // Load genesis configuration
    let genesis_config = if let Some(genesis_path) = &cli.genesis {
        load_genesis_file(genesis_path)?
    } else {
        default_genesis_config()
    };

    // Parse bootnodes
    let bootnodes: Vec<std::net::SocketAddr> = cli
        .bootnodes
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Parse validator key
    let validator_key = if !cli.validator_key.is_empty() {
        let key_hex = cli.validator_key.strip_prefix("0x").unwrap_or(&cli.validator_key);
        hex::decode(key_hex).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Parse validator addresses
    let validator_addrs: Vec<bach_primitives::Address> = cli
        .validators
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| config::parse_address(s.trim()))
        .collect();

    // Build node configuration
    let config = NodeConfig {
        datadir: cli.datadir,
        chain_id: cli.chain_id,
        rpc: RpcConfig {
            enabled: cli.rpc,
            listen_addr: cli.rpc_addr,
        },
        genesis: genesis_config,
        block: BlockConfig {
            gas_limit: cli.gas_limit,
            block_time: Duration::from_secs(cli.block_time),
            coinbase: None,
        },
        p2p: P2pConfig {
            listen_addr: cli.p2p_addr,
            bootnodes,
        },
        consensus: ConsensusConfig {
            validator_key,
            validator_addrs,
        },
    };

    // Create the node
    let node = Arc::new(Node::new(config.clone()).await?);

    // Determine whether to run in consensus mode or solo mode
    let has_validators = !config.consensus.validator_addrs.is_empty()
        && !config.consensus.validator_key.is_empty();

    if has_validators {
        // ── Consensus mode: start network + TBFT ───────────────────
        tracing::info!("Starting in consensus mode with {} validators", config.consensus.validator_addrs.len());

        // Load signing key
        let signing_key = SigningKey::from_slice(&config.consensus.validator_key)
            .map_err(|e| anyhow::anyhow!("Invalid validator key: {}", e))?;
        let our_address = public_key_to_address(signing_key.verifying_key());
        tracing::info!("Validator address: {}", our_address);

        // Build validator set
        let validators: Vec<Validator> = config
            .consensus
            .validator_addrs
            .iter()
            .map(|addr| Validator::new(*addr, 100))
            .collect();
        let validator_set = ValidatorSet::from_validators(validators);

        // Build and start network service
        let net_config = NetworkConfig {
            listen_addr: config.p2p.listen_addr,
            bootstrap_peers: config.p2p.bootnodes.clone(),
            max_peers: 50,
            protocol_version: 1,
            chain_id: config.chain_id,
            genesis_hash: node.genesis_hash(),
            peer_id: PeerId::random(),
        };
        let mut network = NetworkService::new(net_config);
        let net_events = network.take_events().unwrap();
        let network = Arc::new(network);

        network.start().await.map_err(|e| anyhow::anyhow!("Network start failed: {}", e))?;
        tracing::info!("P2P network started on {}", config.p2p.listen_addr);

        // Start RPC server with network reference
        if config.rpc.enabled {
            let rpc_ctx = Arc::new(bach_rpc::RpcContext::with_network(
                node.state_db().clone(),
                node.block_db().clone(),
                node.txpool().clone(),
                node.chain_id(),
                network.clone(),
            ));
            let rpc_handler = RpcHandler::new(rpc_ctx);
            let listen_addr = node.rpc_config().listen_addr;
            let rpc_server = RpcServer::new(ServerConfig::new(listen_addr), rpc_handler);
            tokio::spawn(async move {
                if let Err(e) = rpc_server.run().await {
                    tracing::error!("RPC server error: {}", e);
                }
            });
            tracing::info!("RPC server started on {}", listen_addr);
        }

        // Build consensus engine
        let tbft_config = TbftConfig {
            address: our_address,
            ..Default::default()
        };
        let consensus = TbftConsensus::new(tbft_config, validator_set);

        // Shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let node_clone = Arc::clone(&node);
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Shutdown signal received");
            node_clone.stop().await;
            let _ = shutdown_tx.send(true);
        });

        let driver = ConsensusDriver::new(
            consensus,
            network,
            node.txpool().clone(),
            node.executor().clone(),
            node.state_db().clone(),
            node.block_db().clone(),
            node.chain_id(),
            config.block.gas_limit,
            Some(signing_key),
        );

        driver.run(net_events, shutdown_rx).await;
    } else {
        // ── Solo mode: simple timer-based block production ─────────
        tracing::info!("Starting in solo mode (no validators configured)");

        // Start RPC server without network
        if config.rpc.enabled {
            let rpc_ctx = Arc::new(node.create_rpc_context());
            let rpc_handler = RpcHandler::new(rpc_ctx);
            let listen_addr = node.rpc_config().listen_addr;
            let rpc_server = RpcServer::new(ServerConfig::new(listen_addr), rpc_handler);
            tokio::spawn(async move {
                if let Err(e) = rpc_server.run().await {
                    tracing::error!("RPC server error: {}", e);
                }
            });
            tracing::info!("RPC server started on {}", listen_addr);
        }

        node.run().await?;
    }

    tracing::info!("BachLedger node stopped");
    Ok(())
}

/// Load genesis configuration from file
fn load_genesis_file(path: &std::path::Path) -> Result<GenesisConfig> {
    tracing::info!("Loading genesis from {:?}", path);
    let content = std::fs::read_to_string(path)?;
    let genesis: GenesisConfig = serde_json::from_str(&content)?;
    Ok(genesis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_genesis_file() {
        let mut file = NamedTempFile::new().unwrap();
        let genesis_json = r#"{
            "alloc": {
                "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266": {
                    "balance": "0x21e19e0c9bab2400000"
                }
            },
            "timestamp": 0,
            "difficulty": 1,
            "gas_limit": 30000000,
            "chain_id": 1337
        }"#;
        file.write_all(genesis_json.as_bytes()).unwrap();

        let genesis = load_genesis_file(file.path()).unwrap();
        assert_eq!(genesis.alloc.len(), 1);
        assert_eq!(genesis.chain_id, 1337);
    }
}
