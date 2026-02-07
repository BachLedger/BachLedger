//! BachLedger node binary
//!
//! This is the main entry point for running a BachLedger node.

mod cli;
mod config;
mod genesis;
mod node;

use anyhow::Result;
use bach_rpc::{RpcHandler, RpcServer, ServerConfig};
use cli::Cli;
use config::{default_genesis_config, BlockConfig, GenesisConfig, NodeConfig, RpcConfig};
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
    };

    // Create and run node
    let node = Arc::new(Node::new(config.clone()).await?);

    // Handle Ctrl+C for graceful shutdown
    let node_clone = Arc::clone(&node);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutdown signal received");
        node_clone.stop().await;
    });

    // Start RPC server if enabled
    if config.rpc.enabled {
        let rpc_ctx = Arc::new(node.create_rpc_context());
        let rpc_handler = RpcHandler::new(rpc_ctx);
        let listen_addr = node.rpc_config().listen_addr;
        let rpc_server = RpcServer::new(ServerConfig::new(listen_addr), rpc_handler);

        // Spawn RPC server in background
        tokio::spawn(async move {
            if let Err(e) = rpc_server.run().await {
                tracing::error!("RPC server error: {}", e);
            }
        });

        tracing::info!("RPC server started on {}", listen_addr);
    }

    // Run the node
    node.run().await?;

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
