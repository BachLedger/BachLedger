//! Configuration types for bach-node

use bach_primitives::{Address, H256, U256};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Node configuration
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Data directory
    pub datadir: PathBuf,
    /// Chain ID
    pub chain_id: u64,
    /// RPC configuration
    pub rpc: RpcConfig,
    /// Genesis configuration
    pub genesis: GenesisConfig,
    /// Block configuration
    pub block: BlockConfig,
    /// P2P network configuration
    pub p2p: P2pConfig,
    /// Consensus configuration
    pub consensus: ConsensusConfig,
}

/// RPC server configuration
#[derive(Debug, Clone)]
pub struct RpcConfig {
    /// Whether RPC is enabled
    pub enabled: bool,
    /// Listen address
    pub listen_addr: SocketAddr,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addr: "0.0.0.0:8545".parse().unwrap(),
        }
    }
}

/// Genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Initial account allocations
    #[serde(default)]
    pub alloc: HashMap<String, GenesisAccount>,
    /// Genesis timestamp
    #[serde(default)]
    pub timestamp: u64,
    /// Genesis extra data
    #[serde(default)]
    pub extra_data: String,
    /// Initial difficulty
    #[serde(default = "default_difficulty")]
    pub difficulty: u64,
    /// Initial gas limit
    #[serde(default = "default_gas_limit")]
    pub gas_limit: u64,
    /// Chain ID
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

fn default_difficulty() -> u64 {
    1
}

fn default_gas_limit() -> u64 {
    30_000_000
}

fn default_chain_id() -> u64 {
    1337
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            alloc: HashMap::new(),
            timestamp: 0,
            extra_data: String::new(),
            difficulty: default_difficulty(),
            gas_limit: default_gas_limit(),
            chain_id: default_chain_id(),
        }
    }
}

/// Genesis account allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAccount {
    /// Account balance (as hex string for JSON compatibility)
    #[serde(default)]
    pub balance: String,
    /// Account nonce
    #[serde(default)]
    pub nonce: u64,
    /// Contract code (hex string)
    #[serde(default)]
    pub code: Option<String>,
    /// Storage (slot -> value mapping)
    #[serde(default)]
    pub storage: HashMap<String, String>,
}

impl GenesisAccount {
    /// Parse balance from hex or decimal string
    pub fn parse_balance(&self) -> U256 {
        let s = self.balance.trim();
        if s.is_empty() {
            return U256::zero();
        }
        if s.starts_with("0x") || s.starts_with("0X") {
            U256::from_str_radix(&s[2..], 16).unwrap_or(U256::zero())
        } else {
            U256::from_dec_str(s).unwrap_or(U256::zero())
        }
    }

    /// Parse code from hex string
    pub fn parse_code(&self) -> Option<Bytes> {
        self.code.as_ref().and_then(|c| {
            let c = c.trim();
            let c = c.strip_prefix("0x").unwrap_or(c);
            hex::decode(c).ok().map(Bytes::from)
        })
    }

    /// Parse storage entries
    pub fn parse_storage(&self) -> HashMap<H256, H256> {
        let mut result = HashMap::new();
        for (key, value) in &self.storage {
            let key = key.trim();
            let key = key.strip_prefix("0x").unwrap_or(key);
            let value = value.trim();
            let value = value.strip_prefix("0x").unwrap_or(value);

            if let (Ok(k_bytes), Ok(v_bytes)) = (hex::decode(key), hex::decode(value)) {
                if k_bytes.len() == 32 && v_bytes.len() == 32 {
                    if let (Ok(k), Ok(v)) = (H256::from_slice(&k_bytes), H256::from_slice(&v_bytes)) {
                        result.insert(k, v);
                    }
                }
            }
        }
        result
    }
}

/// Block production configuration
#[derive(Debug, Clone)]
pub struct BlockConfig {
    /// Block gas limit
    pub gas_limit: u64,
    /// Block time (interval between blocks)
    pub block_time: Duration,
    /// Coinbase/beneficiary address for block rewards
    pub coinbase: Option<Address>,
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self {
            gas_limit: 30_000_000,
            block_time: Duration::from_secs(1),
            coinbase: None,
        }
    }
}

/// P2P network configuration
#[derive(Debug, Clone)]
pub struct P2pConfig {
    /// P2P listen address
    pub listen_addr: SocketAddr,
    /// Bootstrap peer addresses
    pub bootnodes: Vec<SocketAddr>,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:30303".parse().unwrap(),
            bootnodes: Vec::new(),
        }
    }
}

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Validator private key bytes (32 bytes). Empty means non-validator mode.
    pub validator_key: Vec<u8>,
    /// Validator addresses (defines the validator set)
    pub validator_addrs: Vec<Address>,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            validator_key: Vec::new(),
            validator_addrs: Vec::new(),
        }
    }
}

/// Default genesis with pre-funded test accounts
pub fn default_genesis_config() -> GenesisConfig {
    let mut alloc = HashMap::new();

    // Pre-fund Hardhat/Anvil default test accounts (10000 ETH each)
    let test_accounts = [
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", // Account #0
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", // Account #1
        "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // Account #2
        "0x90F79bf6EB2c4f870365E785982E1f101E93b906", // Account #3
        "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", // Account #4
    ];

    // 10000 ETH in wei
    let balance = "0x21e19e0c9bab2400000";

    for addr in test_accounts {
        alloc.insert(
            addr.to_string(),
            GenesisAccount {
                balance: balance.to_string(),
                nonce: 0,
                code: None,
                storage: HashMap::new(),
            },
        );
    }

    GenesisConfig {
        alloc,
        timestamp: 0,
        extra_data: String::new(),
        difficulty: 1,
        gas_limit: 30_000_000,
        chain_id: 1337,
    }
}

/// Parse address from hex string
pub fn parse_address(s: &str) -> Option<Address> {
    let s = s.trim();
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s)
        .ok()
        .and_then(|bytes| Address::from_slice(&bytes).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_account_parse_balance_hex() {
        let account = GenesisAccount {
            balance: "0x21e19e0c9bab2400000".to_string(),
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        };
        let balance = account.parse_balance();
        // 10000 ETH = 10000 * 10^18 wei
        assert!(balance > U256::zero());
    }

    #[test]
    fn test_genesis_account_parse_balance_decimal() {
        let account = GenesisAccount {
            balance: "1000000000000000000".to_string(), // 1 ETH
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        };
        let balance = account.parse_balance();
        assert_eq!(balance, U256::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_genesis_account_parse_empty_balance() {
        let account = GenesisAccount {
            balance: "".to_string(),
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        };
        let balance = account.parse_balance();
        assert_eq!(balance, U256::zero());
    }

    #[test]
    fn test_genesis_account_parse_code() {
        let account = GenesisAccount {
            balance: "0".to_string(),
            nonce: 0,
            code: Some("0x6000600000".to_string()),
            storage: HashMap::new(),
        };
        let code = account.parse_code().unwrap();
        assert_eq!(code.as_ref(), &[0x60, 0x00, 0x60, 0x00, 0x00]);
    }

    #[test]
    fn test_genesis_account_parse_storage() {
        let mut storage = HashMap::new();
        storage.insert(
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        );
        let account = GenesisAccount {
            balance: "0".to_string(),
            nonce: 0,
            code: None,
            storage,
        };
        let parsed = account.parse_storage();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.get(&H256::ZERO), Some(&H256::from_bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01])));
    }

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        // Note: to_hex() returns with 0x prefix
        let hex = addr.to_hex();
        assert!(hex.contains("f39fd6e51aad88f6f4ce6ab8827279cfffb92266") ||
                hex.contains("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
    }

    #[test]
    fn test_default_genesis_config() {
        let genesis = default_genesis_config();
        assert_eq!(genesis.alloc.len(), 5);
        assert_eq!(genesis.chain_id, 1337);
        assert_eq!(genesis.gas_limit, 30_000_000);
    }

    #[test]
    fn test_genesis_config_serde() {
        let json = r#"{
            "alloc": {
                "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266": {
                    "balance": "0x21e19e0c9bab2400000"
                }
            },
            "timestamp": 1234567890,
            "difficulty": 1,
            "gas_limit": 30000000,
            "chain_id": 1337
        }"#;
        let genesis: GenesisConfig = serde_json::from_str(json).unwrap();
        assert_eq!(genesis.alloc.len(), 1);
        assert_eq!(genesis.timestamp, 1234567890);
        assert_eq!(genesis.chain_id, 1337);
    }
}
