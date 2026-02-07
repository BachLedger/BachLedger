//! CLI argument parsing for bach-node

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

/// BachLedger blockchain node
#[derive(Parser, Debug, Clone)]
#[command(name = "bachledger")]
#[command(about = "BachLedger blockchain node")]
#[command(version)]
pub struct Cli {
    /// Data directory for blockchain storage
    #[arg(long, default_value = "./data")]
    pub datadir: PathBuf,

    /// Chain ID
    #[arg(long, default_value = "1337")]
    pub chain_id: u64,

    /// RPC server listen address
    #[arg(long, default_value = "0.0.0.0:8545")]
    pub rpc_addr: SocketAddr,

    /// Enable RPC server (use --no-rpc to disable)
    #[arg(long, default_value_t = true)]
    pub rpc: bool,

    /// Genesis file path (optional, uses default if not specified)
    #[arg(long)]
    pub genesis: Option<PathBuf>,

    /// Block gas limit
    #[arg(long, default_value = "30000000")]
    pub gas_limit: u64,

    /// Block time in seconds
    #[arg(long, default_value = "1")]
    pub block_time: u64,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// P2P listen address
    #[arg(long, default_value = "0.0.0.0:30303")]
    pub p2p_addr: SocketAddr,

    /// Bootstrap peer addresses (comma-separated, e.g. "1.2.3.4:30303,5.6.7.8:30303")
    #[arg(long, default_value = "")]
    pub bootnodes: String,

    /// Validator private key (hex, without 0x prefix)
    #[arg(long, default_value = "")]
    pub validator_key: String,

    /// Validator addresses (comma-separated hex addresses for the validator set)
    #[arg(long, default_value = "")]
    pub validators: String,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["bachledger"]);
        assert_eq!(cli.datadir, PathBuf::from("./data"));
        assert_eq!(cli.chain_id, 1337);
        assert_eq!(cli.rpc_addr.to_string(), "0.0.0.0:8545");
        assert!(cli.rpc);
        assert!(cli.genesis.is_none());
        assert_eq!(cli.gas_limit, 30_000_000);
        assert_eq!(cli.block_time, 1);
        assert_eq!(cli.log_level, "info");
    }

    #[test]
    fn test_cli_custom_values() {
        let cli = Cli::parse_from([
            "bachledger",
            "--datadir", "/tmp/bachledger",
            "--chain-id", "42",
            "--rpc-addr", "127.0.0.1:9545",
            "--genesis", "/path/to/genesis.json",
            "--gas-limit", "50000000",
            "--block-time", "5",
            "--log-level", "debug",
        ]);
        assert_eq!(cli.datadir, PathBuf::from("/tmp/bachledger"));
        assert_eq!(cli.chain_id, 42);
        assert_eq!(cli.rpc_addr.to_string(), "127.0.0.1:9545");
        assert!(cli.rpc); // Default is true
        assert_eq!(cli.genesis, Some(PathBuf::from("/path/to/genesis.json")));
        assert_eq!(cli.gas_limit, 50_000_000);
        assert_eq!(cli.block_time, 5);
        assert_eq!(cli.log_level, "debug");
    }
}
