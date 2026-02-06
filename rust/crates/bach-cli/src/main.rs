//! # bach-cli
//!
//! Command-line interface for BachLedger blockchain.
//!
//! ## Usage
//!
//! ```bash
//! # Account commands
//! bach account create
//! bach account list
//! bach account balance 0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d
//!
//! # Transaction commands
//! bach tx send --to 0x... --amount 1.0 --key 0x...
//! bach tx deploy --bytecode 0x... --key 0x...
//! bach tx call --contract 0x... --data 0x... --key 0x...
//!
//! # Query commands
//! bach query block latest
//! bach query tx 0x...
//! bach query chain-id
//! bach query gas-price
//! ```

use clap::{Parser, Subcommand};

mod commands;
mod config;
mod error;
mod output;

pub use config::Config;
pub use error::CliError;
pub use output::Output;

/// BachLedger CLI
#[derive(Parser, Debug)]
#[command(name = "bach")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// RPC endpoint URL
    #[arg(long, global = true)]
    rpc_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

/// CLI commands
#[derive(Debug, Subcommand)]
enum Commands {
    /// Account management
    #[command(subcommand)]
    Account(commands::account::AccountCommand),
    /// Transaction operations
    #[command(subcommand)]
    Tx(commands::tx::TxCommand),
    /// Query blockchain state
    #[command(subcommand)]
    Query(commands::query::QueryCommand),
    /// Show or edit configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        /// Set RPC URL
        #[arg(long)]
        set_rpc: Option<String>,
        /// Set chain ID
        #[arg(long)]
        set_chain_id: Option<u64>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Load config
    let mut config = Config::load();

    // Override RPC URL if provided
    if let Some(rpc_url) = cli.rpc_url {
        config.rpc_url = rpc_url;
    }

    let result = match cli.command {
        Commands::Account(cmd) => cmd.execute(&config, cli.json).await,
        Commands::Tx(cmd) => cmd.execute(&config, cli.json).await,
        Commands::Query(cmd) => cmd.execute(&config, cli.json).await,
        Commands::Config {
            show,
            set_rpc,
            set_chain_id,
        } => handle_config(&mut config, show, set_rpc, set_chain_id, cli.json),
    };

    if let Err(e) = result {
        if cli.json {
            println!(
                "{}",
                serde_json::json!({
                    "error": e.to_string(),
                    "success": false
                })
            );
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}

fn handle_config(
    config: &mut Config,
    show: bool,
    set_rpc: Option<String>,
    set_chain_id: Option<u64>,
    json: bool,
) -> Result<(), CliError> {
    let mut modified = false;

    if let Some(rpc) = set_rpc {
        config.rpc_url = rpc;
        modified = true;
    }

    if let Some(chain_id) = set_chain_id {
        config.chain_id = chain_id;
        modified = true;
    }

    if modified {
        config.save()?;
        Output::new(json)
            .field("status", "saved")
            .message("Configuration saved")
            .print();
    } else if show {
        Output::new(json)
            .field("rpc_url", &config.rpc_url)
            .field_u64("chain_id", config.chain_id)
            .field_u64("gas_limit", config.gas_limit)
            .message(&format!(
                "RPC URL: {}\nChain ID: {}\nGas Limit: {}",
                config.rpc_url, config.chain_id, config.gas_limit
            ))
            .print();
    } else {
        Output::new(json)
            .message("Use --show to display config, or --set-rpc/--set-chain-id to modify")
            .print();
    }

    Ok(())
}
