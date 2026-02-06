//! Network namespace RPC methods (net_*)

use std::sync::Arc;

use serde_json::Value;

use crate::error::JsonRpcError;
use crate::handler::RpcContext;
use crate::types::format_u64;

/// net_version - Returns the current network ID
pub async fn net_version(
    ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    // Network ID is same as chain ID for Ethereum-compatible networks
    Ok(Value::String(ctx.chain_id.to_string()))
}

/// net_listening - Returns true if client is actively listening
pub async fn net_listening(
    _ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    // Always return true for now
    Ok(Value::Bool(true))
}

/// net_peerCount - Returns number of peers
pub async fn net_peer_count(
    _ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    // Single node - no peers
    Ok(Value::String(format_u64(0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_u64_peer_count() {
        // net_peerCount returns hex-formatted count
        assert_eq!(format_u64(0), "0x0");
        assert_eq!(format_u64(10), "0xa");
        assert_eq!(format_u64(255), "0xff");
    }
}
