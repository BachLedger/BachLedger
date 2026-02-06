//! Type definitions for ethereum/tests JSON format

use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// Hex-encoded bytes
#[derive(Debug, Clone, Default)]
pub struct HexBytes(pub Vec<u8>);

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        if s.is_empty() {
            return Ok(HexBytes(Vec::new()));
        }
        hex::decode(s)
            .map(HexBytes)
            .map_err(serde::de::Error::custom)
    }
}

/// Hex-encoded U256
#[derive(Debug, Clone, Default)]
pub struct HexU256(pub [u8; 32]);

impl<'de> Deserialize<'de> for HexU256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        if s.is_empty() {
            return Ok(HexU256([0u8; 32]));
        }

        // Pad with leading zero if odd length
        let padded = if s.len() % 2 == 1 {
            format!("0{}", s)
        } else {
            s.to_string()
        };

        let bytes = hex::decode(&padded).map_err(serde::de::Error::custom)?;
        let mut result = [0u8; 32];
        let offset = 32 - bytes.len().min(32);
        result[offset..].copy_from_slice(&bytes[bytes.len().saturating_sub(32)..]);
        Ok(HexU256(result))
    }
}

/// Hex-encoded u64
#[derive(Debug, Clone, Default)]
pub struct HexU64(pub u64);

impl<'de> Deserialize<'de> for HexU64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        if s.is_empty() {
            return Ok(HexU64(0));
        }
        u64::from_str_radix(s, 16)
            .map(HexU64)
            .map_err(serde::de::Error::custom)
    }
}

/// Hex-encoded address (20 bytes)
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct HexAddress(pub [u8; 20]);

impl<'de> Deserialize<'de> for HexAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 20 {
            return Err(serde::de::Error::custom(format!(
                "invalid address length: {}",
                bytes.len()
            )));
        }
        let mut result = [0u8; 20];
        result.copy_from_slice(&bytes);
        Ok(HexAddress(result))
    }
}

/// Hex-encoded H256 (32 bytes)
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct HexH256(pub [u8; 32]);

impl<'de> Deserialize<'de> for HexH256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        if s.is_empty() {
            return Ok(HexH256([0u8; 32]));
        }
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        let mut result = [0u8; 32];
        if bytes.len() == 32 {
            result.copy_from_slice(&bytes);
        } else if bytes.len() < 32 {
            let offset = 32 - bytes.len();
            result[offset..].copy_from_slice(&bytes);
        }
        Ok(HexH256(result))
    }
}

// =============================================================================
// VM Test Types
// =============================================================================

/// VM test file structure (map of test name -> test case)
pub type VmTestFile = HashMap<String, VmTestCase>;

/// Single VM test case
#[derive(Debug, Deserialize)]
pub struct VmTestCase {
    /// Environment info
    pub env: VmEnv,
    /// Execution parameters
    pub exec: VmExec,
    /// Expected gas remaining (None if test should fail)
    pub gas: Option<HexU64>,
    /// Expected logs hash
    pub logs: Option<HexH256>,
    /// Expected output
    pub out: Option<HexBytes>,
    /// Pre-execution state
    pub pre: HashMap<String, AccountState>,
    /// Post-execution state (None if test should fail)
    pub post: Option<HashMap<String, AccountState>>,
}

/// VM test environment
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmEnv {
    /// Current coinbase
    pub current_coinbase: HexAddress,
    /// Current difficulty
    pub current_difficulty: HexU256,
    /// Current gas limit
    pub current_gas_limit: HexU64,
    /// Current block number
    pub current_number: HexU64,
    /// Current timestamp
    pub current_timestamp: HexU64,
    /// Current base fee (optional, EIP-1559)
    pub current_base_fee: Option<HexU64>,
    /// Current random (optional, post-merge)
    pub current_random: Option<HexH256>,
}

/// VM test execution parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmExec {
    /// Address of the executing account
    pub address: HexAddress,
    /// Caller address
    pub caller: HexAddress,
    /// Code to execute
    pub code: HexBytes,
    /// Input data
    pub data: HexBytes,
    /// Gas provided
    pub gas: HexU64,
    /// Gas price
    pub gas_price: HexU64,
    /// Origin address
    pub origin: HexAddress,
    /// Value transferred
    pub value: HexU256,
}

// =============================================================================
// State Test Types
// =============================================================================

/// State test file structure
pub type StateTestFile = HashMap<String, StateTestCase>;

/// Single state test case
#[derive(Debug, Deserialize)]
pub struct StateTestCase {
    /// Environment info
    pub env: StateEnv,
    /// Pre-execution state
    pub pre: HashMap<String, AccountState>,
    /// Transaction parameters
    pub transaction: StateTransaction,
    /// Post-execution state expectations per fork
    pub post: HashMap<String, Vec<PostStateResult>>,
}

/// State test environment
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateEnv {
    /// Current coinbase
    pub current_coinbase: HexAddress,
    /// Current difficulty
    pub current_difficulty: HexU256,
    /// Current gas limit
    pub current_gas_limit: HexU64,
    /// Current block number
    pub current_number: HexU64,
    /// Current timestamp
    pub current_timestamp: HexU64,
    /// Previous hash
    pub previous_hash: HexH256,
    /// Current base fee (optional)
    pub current_base_fee: Option<HexU64>,
    /// Current random (optional)
    pub current_random: Option<HexH256>,
}

/// State test transaction
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTransaction {
    /// Data options
    pub data: Vec<HexBytes>,
    /// Gas limit options
    pub gas_limit: Vec<HexU64>,
    /// Gas price
    pub gas_price: Option<HexU64>,
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<HexU64>,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<HexU64>,
    /// Nonce
    pub nonce: HexU64,
    /// Secret key
    pub secret_key: HexH256,
    /// To address (None for contract creation)
    pub to: Option<String>,
    /// Value options
    pub value: Vec<HexU256>,
    /// Access list (optional)
    pub access_lists: Option<Vec<Option<Vec<AccessListEntry>>>>,
}

/// Access list entry
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListEntry {
    /// Address
    pub address: HexAddress,
    /// Storage keys
    pub storage_keys: Vec<HexH256>,
}

/// Post-state result for a specific index combination
#[derive(Debug, Deserialize)]
pub struct PostStateResult {
    /// Hash of post-state
    pub hash: HexH256,
    /// Index selectors
    pub indexes: IndexSelector,
    /// Expected logs hash
    pub logs: HexH256,
    /// Transaction bytes (optional)
    pub txbytes: Option<HexBytes>,
    /// Expected exception (optional)
    pub expect_exception: Option<String>,
}

/// Index selector for transaction variations
#[derive(Debug, Deserialize)]
pub struct IndexSelector {
    /// Data index
    pub data: usize,
    /// Gas index
    pub gas: usize,
    /// Value index
    pub value: usize,
}

// =============================================================================
// Common Types
// =============================================================================

/// Account state
#[derive(Debug, Deserialize)]
pub struct AccountState {
    /// Balance
    pub balance: HexU256,
    /// Code
    pub code: HexBytes,
    /// Nonce
    pub nonce: HexU64,
    /// Storage
    pub storage: HashMap<String, HexU256>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_bytes_deserialize() {
        let json = r#""0x1234""#;
        let bytes: HexBytes = serde_json::from_str(json).unwrap();
        assert_eq!(bytes.0, vec![0x12, 0x34]);
    }

    #[test]
    fn test_hex_u256_deserialize() {
        let json = r#""0x1""#;
        let value: HexU256 = serde_json::from_str(json).unwrap();
        let mut expected = [0u8; 32];
        expected[31] = 1;
        assert_eq!(value.0, expected);
    }

    #[test]
    fn test_hex_address_deserialize() {
        let json = r#""0x1234567890123456789012345678901234567890""#;
        let addr: HexAddress = serde_json::from_str(json).unwrap();
        assert_eq!(
            addr.0,
            [
                0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78,
                0x90, 0x12, 0x34, 0x56, 0x78, 0x90
            ]
        );
    }

    #[test]
    fn test_hex_u64_deserialize() {
        let json = r#""0x100""#;
        let value: HexU64 = serde_json::from_str(json).unwrap();
        assert_eq!(value.0, 256);
    }

    #[test]
    fn test_empty_hex_bytes() {
        let json = r#""0x""#;
        let bytes: HexBytes = serde_json::from_str(json).unwrap();
        assert!(bytes.0.is_empty());
    }
}
