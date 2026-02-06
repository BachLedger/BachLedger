//! Web3 namespace RPC methods (web3_*)

use std::sync::Arc;

use bach_crypto::keccak256;
use serde_json::Value;

use crate::error::JsonRpcError;
use crate::handler::RpcContext;
use crate::types::parse_hex_bytes;

/// Client version string
const CLIENT_VERSION: &str = "BachLedger/0.1.0";

/// web3_clientVersion - Returns the client version
pub async fn web3_client_version(
    _ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    Ok(Value::String(CLIENT_VERSION.to_string()))
}

/// web3_sha3 - Returns Keccak-256 hash of the given data
pub async fn web3_sha3(
    _ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing data parameter"));
    }

    let data = parse_hex_bytes(&params[0])?;
    let hash = keccak256(&data);

    Ok(Value::String(hash.to_hex()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_version() {
        assert!(CLIENT_VERSION.starts_with("BachLedger"));
        assert!(CLIENT_VERSION.contains("/"));
    }

    #[test]
    fn test_client_version_format() {
        // Format should be "BachLedger/X.Y.Z"
        let parts: Vec<&str> = CLIENT_VERSION.split('/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "BachLedger");

        // Version should have major.minor.patch format
        let version_parts: Vec<&str> = parts[1].split('.').collect();
        assert_eq!(version_parts.len(), 3);

        // All parts should be numeric
        for part in version_parts {
            assert!(part.parse::<u32>().is_ok(), "Version part '{}' is not numeric", part);
        }
    }

    #[test]
    fn test_keccak256_empty_input() {
        let hash = keccak256(&[]);
        // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        assert_eq!(
            hash.to_hex(),
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }

    #[test]
    fn test_keccak256_hello() {
        // keccak256("hello") known test vector
        let hash = keccak256(b"hello");
        assert_eq!(
            hash.to_hex(),
            "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    #[test]
    fn test_parse_hex_bytes_for_sha3() {
        // Test that parse_hex_bytes works correctly for web3_sha3 inputs
        let result = parse_hex_bytes(&Value::String("0x68656c6c6f".to_string())); // "hello" in hex
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"hello".to_vec());
    }
}
