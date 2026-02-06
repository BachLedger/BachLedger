//! SDK types

use bach_primitives::{Address, H256, U256};
use bytes::Bytes;
use serde::Serialize;

/// Block identifier for RPC queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockId {
    /// Block number
    Number(u64),
    /// Latest block
    #[default]
    Latest,
    /// Pending block (includes pending transactions)
    Pending,
    /// Earliest block (genesis)
    Earliest,
    /// Safe block (finalized by consensus)
    Safe,
    /// Finalized block
    Finalized,
}

impl Serialize for BlockId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BlockId::Number(n) => serializer.serialize_str(&format!("0x{:x}", n)),
            BlockId::Latest => serializer.serialize_str("latest"),
            BlockId::Pending => serializer.serialize_str("pending"),
            BlockId::Earliest => serializer.serialize_str("earliest"),
            BlockId::Safe => serializer.serialize_str("safe"),
            BlockId::Finalized => serializer.serialize_str("finalized"),
        }
    }
}

/// Call request for eth_call and eth_estimateGas
#[derive(Debug, Clone, Default)]
pub struct CallRequest {
    /// Sender address
    pub from: Option<Address>,
    /// Recipient address
    pub to: Option<Address>,
    /// Gas limit
    pub gas: Option<u64>,
    /// Gas price (legacy)
    pub gas_price: Option<u128>,
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<u128>,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<u128>,
    /// Value to transfer
    pub value: Option<U256>,
    /// Input data
    pub data: Option<Bytes>,
}

impl Serialize for CallRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        // Count non-None fields
        let mut count = 0;
        if self.from.is_some() { count += 1; }
        if self.to.is_some() { count += 1; }
        if self.gas.is_some() { count += 1; }
        if self.gas_price.is_some() { count += 1; }
        if self.max_fee_per_gas.is_some() { count += 1; }
        if self.max_priority_fee_per_gas.is_some() { count += 1; }
        if self.value.is_some() { count += 1; }
        if self.data.is_some() { count += 1; }

        let mut map = serializer.serialize_map(Some(count))?;

        if let Some(from) = &self.from {
            map.serialize_entry("from", &from.to_hex())?;
        }
        if let Some(to) = &self.to {
            map.serialize_entry("to", &to.to_hex())?;
        }
        if let Some(gas) = &self.gas {
            map.serialize_entry("gas", &format!("0x{:x}", gas))?;
        }
        if let Some(gas_price) = &self.gas_price {
            map.serialize_entry("gasPrice", &format!("0x{:x}", gas_price))?;
        }
        if let Some(max_fee) = &self.max_fee_per_gas {
            map.serialize_entry("maxFeePerGas", &format!("0x{:x}", max_fee))?;
        }
        if let Some(max_priority) = &self.max_priority_fee_per_gas {
            map.serialize_entry("maxPriorityFeePerGas", &format!("0x{:x}", max_priority))?;
        }
        if let Some(value) = &self.value {
            // Format U256 as hex
            let mut bytes = [0u8; 32];
            value.to_big_endian(&mut bytes);
            let hex = format!("0x{}", hex::encode(bytes).trim_start_matches('0'));
            let hex = if hex == "0x" { "0x0".to_string() } else { hex };
            map.serialize_entry("value", &hex)?;
        }
        if let Some(data) = &self.data {
            map.serialize_entry("data", &format!("0x{}", hex::encode(data)))?;
        }

        map.end()
    }
}

/// Transaction request for building transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionRequest {
    /// Chain ID
    pub chain_id: u64,
    /// Sender nonce
    pub nonce: Option<u64>,
    /// Gas limit
    pub gas_limit: Option<u64>,
    /// Gas price (legacy transactions)
    pub gas_price: Option<u128>,
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<u128>,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<u128>,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Value to transfer
    pub value: u128,
    /// Input data
    pub data: Bytes,
}

/// Pending transaction handle
#[derive(Debug, Clone)]
pub struct PendingTransaction {
    /// Transaction hash
    pub hash: H256,
}

impl PendingTransaction {
    /// Create a new pending transaction
    pub fn new(hash: H256) -> Self {
        Self { hash }
    }

    /// Get the transaction hash
    pub fn hash(&self) -> &H256 {
        &self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id_serialize() {
        assert_eq!(
            serde_json::to_string(&BlockId::Latest).unwrap(),
            "\"latest\""
        );
        assert_eq!(
            serde_json::to_string(&BlockId::Number(100)).unwrap(),
            "\"0x64\""
        );
        assert_eq!(
            serde_json::to_string(&BlockId::Pending).unwrap(),
            "\"pending\""
        );
    }

    #[test]
    fn test_call_request_serialize() {
        let req = CallRequest {
            to: Some(Address::ZERO),
            data: Some(Bytes::from(vec![0x01, 0x02])),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("to").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("from").is_none()); // None fields skipped
    }

    #[test]
    fn test_call_request_serialize_with_value() {
        let req = CallRequest {
            to: Some(Address::ZERO),
            value: Some(U256::from(1000)),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("value"));
    }
}
