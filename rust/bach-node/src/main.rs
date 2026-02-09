//! BachLedger Medical Blockchain Node
//!
//! Command-line interface for running a BachLedger node.

use bach_node::{BachNode, NodeConfig, NodeError};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

/// BachLedger Medical Blockchain Node
#[derive(Parser)]
#[command(name = "bach-node")]
#[command(author = "BachLedger Team")]
#[command(version = "0.1.0")]
#[command(about = "Secure blockchain for medical data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Data directory for blockchain storage
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,

    /// P2P network listen address
    #[arg(long, default_value = "0.0.0.0:30303")]
    listen_addr: String,

    /// Bootstrap peers (comma-separated)
    #[arg(long)]
    bootnodes: Option<String>,

    /// Validator private key file
    #[arg(long)]
    validator_key: Option<PathBuf>,

    /// Chain ID
    #[arg(long, default_value = "31337")]
    chain_id: u64,

    /// Block time in milliseconds
    #[arg(long, default_value = "3000")]
    block_time: u64,

    /// Enable JSON-RPC server
    #[arg(long, default_value = "true")]
    rpc: bool,

    /// JSON-RPC HTTP listen address
    #[arg(long, default_value = "0.0.0.0:8545")]
    rpc_addr: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the node
    Run,

    /// Initialize a new data directory
    Init {
        /// Genesis file path
        #[arg(long)]
        genesis: Option<PathBuf>,
    },

    /// Show node information
    Info,

    /// Generate a new validator key
    GenKey {
        /// Output file path
        #[arg(long, default_value = "validator.key")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), NodeError> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level)),
        )
        .init();

    // Load config from file if specified, otherwise use CLI args
    let config = if let Some(config_path) = cli.config {
        NodeConfig::from_file(&config_path)?
    } else {
        build_config_from_cli(&cli)?
    };

    match cli.command {
        Some(Commands::Init { genesis }) => {
            init_node(&config, genesis.as_deref()).await?;
        }
        Some(Commands::Info) => {
            show_info(&config).await?;
        }
        Some(Commands::GenKey { output }) => {
            generate_key(&output)?;
        }
        Some(Commands::Run) | None => {
            run_node(config).await?;
        }
    }

    Ok(())
}

fn build_config_from_cli(cli: &Cli) -> Result<NodeConfig, NodeError> {
    let listen_addr: SocketAddr = cli.listen_addr.parse().map_err(|e| {
        NodeError::ConfigError(format!("Invalid listen address: {}", e))
    })?;

    let rpc_addr: SocketAddr = cli.rpc_addr.parse().map_err(|e| {
        NodeError::ConfigError(format!("Invalid RPC address: {}", e))
    })?;

    let bootstrap_peers: Vec<SocketAddr> = cli
        .bootnodes
        .as_ref()
        .map(|s| {
            s.split(',')
                .filter_map(|addr| addr.trim().parse().ok())
                .collect()
        })
        .unwrap_or_default();

    let validator_key = if let Some(ref key_path) = cli.validator_key {
        let key_hex = std::fs::read_to_string(key_path).map_err(|e| {
            NodeError::ConfigError(format!("Failed to read validator key: {}", e))
        })?;
        let key_bytes = hex::decode(key_hex.trim()).map_err(|e| {
            NodeError::ConfigError(format!("Invalid key format: {}", e))
        })?;
        if key_bytes.len() != 32 {
            return Err(NodeError::ConfigError("Key must be 32 bytes".to_string()));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Some(key)
    } else {
        None
    };

    let mut config = NodeConfig::new(cli.data_dir.clone())
        .with_listen_addr(listen_addr)
        .with_bootstrap_peers(bootstrap_peers)
        .with_chain_id(cli.chain_id);

    if let Some(key) = validator_key {
        config = config.with_validator_key(key);
    }

    if cli.rpc {
        config = config.with_rpc(rpc_addr);
    }

    config.block_time_ms = cli.block_time;

    Ok(config)
}

async fn run_node(config: NodeConfig) -> Result<(), NodeError> {
    tracing::info!("Starting BachLedger node");
    tracing::info!("Chain ID: {}", config.chain_id);
    tracing::info!("Data directory: {:?}", config.data_dir);
    tracing::info!("P2P address: {}", config.listen_addr);

    if config.rpc_enabled {
        tracing::info!("RPC address: {:?}", config.rpc_addr);
    }

    let mut node = BachNode::new(config);
    node.start().await?;

    tracing::info!("Node started successfully");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");

    tracing::info!("Shutdown signal received");
    node.stop().await?;

    tracing::info!("Node stopped");
    Ok(())
}

async fn init_node(config: &NodeConfig, _genesis: Option<&std::path::Path>) -> Result<(), NodeError> {
    tracing::info!("Initializing new node at {:?}", config.data_dir);

    // Create data directory
    std::fs::create_dir_all(&config.data_dir)?;

    // Initialize storage
    let mut node = BachNode::new(config.clone());
    node.init()?;

    tracing::info!("Node initialized successfully");
    Ok(())
}

async fn show_info(config: &NodeConfig) -> Result<(), NodeError> {
    println!("BachLedger Node Configuration");
    println!("==============================");
    println!("Data directory: {:?}", config.data_dir);
    println!("Chain ID: {}", config.chain_id);
    println!("P2P address: {}", config.listen_addr);
    println!("Bootstrap peers: {:?}", config.bootstrap_peers);
    println!("RPC enabled: {}", config.rpc_enabled);
    if let Some(ref addr) = config.rpc_addr {
        println!("RPC address: {}", addr);
    }
    println!("Block time: {}ms", config.block_time_ms);
    println!("Max transactions per block: {}", config.max_txs_per_block);
    println!("Validator: {}", config.validator_key.is_some());

    Ok(())
}

fn generate_key(output: &PathBuf) -> Result<(), NodeError> {
    use bach_crypto::PrivateKey;

    tracing::info!("Generating new validator key");

    let key = PrivateKey::random();
    let key_bytes = key.to_bytes();
    let key_hex = hex::encode(&key_bytes);

    std::fs::write(output, &key_hex)?;

    let address = key.public_key().to_address();
    let pubkey = key.public_key().to_bytes();

    println!("Validator key generated successfully");
    println!("Private key saved to: {:?}", output);
    println!("Address: 0x{}", hex::encode(address.as_bytes()));
    println!("Public key: 0x04{}", hex::encode(&pubkey));

    Ok(())
}
