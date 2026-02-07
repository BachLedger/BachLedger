//! Request handler and method dispatcher

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;

use bach_network::NetworkService;
use bach_storage::{BlockDb, StateDb};
use bach_txpool::TxPool;

use crate::error::JsonRpcError;
use crate::methods::{eth, net, web3};
use crate::types::{JsonRpcRequest, JsonRpcResponse};

/// Type alias for async method handler
pub type MethodFn = Box<
    dyn Fn(Arc<RpcContext>, Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, JsonRpcError>> + Send>>
        + Send
        + Sync,
>;

/// Shared context for RPC handlers
pub struct RpcContext {
    /// State database access
    pub state_db: Arc<StateDb>,
    /// Block database access
    pub block_db: Arc<BlockDb>,
    /// Transaction pool
    pub txpool: Arc<TxPool>,
    /// Chain ID
    pub chain_id: u64,
    /// Current gas price (in wei)
    pub gas_price: std::sync::atomic::AtomicU64,
    /// Optional network service for transaction broadcasting
    pub network: Option<Arc<NetworkService>>,
}

impl RpcContext {
    /// Create a new RPC context
    pub fn new(
        state_db: Arc<StateDb>,
        block_db: Arc<BlockDb>,
        txpool: Arc<TxPool>,
        chain_id: u64,
    ) -> Self {
        Self {
            state_db,
            block_db,
            txpool,
            chain_id,
            gas_price: std::sync::atomic::AtomicU64::new(1_000_000_000),
            network: None,
        }
    }

    /// Create a new RPC context with network service
    pub fn with_network(
        state_db: Arc<StateDb>,
        block_db: Arc<BlockDb>,
        txpool: Arc<TxPool>,
        chain_id: u64,
        network: Arc<NetworkService>,
    ) -> Self {
        Self {
            state_db,
            block_db,
            txpool,
            chain_id,
            gas_price: std::sync::atomic::AtomicU64::new(1_000_000_000),
            network: Some(network),
        }
    }

    /// Get current gas price
    pub fn get_gas_price(&self) -> u64 {
        self.gas_price.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Set gas price
    pub fn set_gas_price(&self, price: u64) {
        self.gas_price
            .store(price, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Method registry for dispatching RPC calls
pub struct MethodRegistry {
    methods: HashMap<String, MethodFn>,
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodRegistry {
    /// Create a new method registry with all methods registered
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
        };

        // Register eth_* methods
        registry.register("eth_chainId", eth::eth_chain_id);
        registry.register("eth_blockNumber", eth::eth_block_number);
        registry.register("eth_gasPrice", eth::eth_gas_price);
        registry.register("eth_getBalance", eth::eth_get_balance);
        registry.register("eth_getTransactionCount", eth::eth_get_transaction_count);
        registry.register("eth_getCode", eth::eth_get_code);
        registry.register("eth_getStorageAt", eth::eth_get_storage_at);
        registry.register("eth_call", eth::eth_call);
        registry.register("eth_estimateGas", eth::eth_estimate_gas);
        registry.register("eth_sendRawTransaction", eth::eth_send_raw_transaction);
        registry.register("eth_getBlockByNumber", eth::eth_get_block_by_number);
        registry.register("eth_getBlockByHash", eth::eth_get_block_by_hash);
        registry.register("eth_getTransactionByHash", eth::eth_get_transaction_by_hash);
        registry.register("eth_getTransactionReceipt", eth::eth_get_transaction_receipt);

        // Register net_* methods
        registry.register("net_version", net::net_version);
        registry.register("net_listening", net::net_listening);
        registry.register("net_peerCount", net::net_peer_count);

        // Register web3_* methods
        registry.register("web3_clientVersion", web3::web3_client_version);
        registry.register("web3_sha3", web3::web3_sha3);

        registry
    }

    /// Register a method handler
    pub fn register<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn(Arc<RpcContext>, Vec<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, JsonRpcError>> + Send + 'static,
    {
        self.methods.insert(
            name.to_string(),
            Box::new(move |ctx, params| Box::pin(handler(ctx, params))),
        );
    }

    /// Dispatch a method call
    pub async fn dispatch(
        &self,
        ctx: Arc<RpcContext>,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, JsonRpcError> {
        match self.methods.get(method) {
            Some(handler) => handler(ctx, params).await,
            None => Err(JsonRpcError::method_not_found(method)),
        }
    }

    /// Check if a method is registered
    pub fn has_method(&self, name: &str) -> bool {
        self.methods.contains_key(name)
    }

    /// Get list of registered methods
    pub fn method_names(&self) -> Vec<&str> {
        self.methods.keys().map(|s| s.as_str()).collect()
    }
}

/// RPC request handler
pub struct RpcHandler {
    ctx: Arc<RpcContext>,
    registry: MethodRegistry,
}

impl RpcHandler {
    /// Create a new RPC handler
    pub fn new(ctx: Arc<RpcContext>) -> Self {
        Self {
            ctx,
            registry: MethodRegistry::new(),
        }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse::error(
                request.id,
                JsonRpcError::invalid_request("invalid JSON-RPC version"),
            );
        }

        // Dispatch the method
        match self
            .registry
            .dispatch(self.ctx.clone(), &request.method, request.params)
            .await
        {
            Ok(result) => JsonRpcResponse::success(request.id, result),
            Err(error) => JsonRpcResponse::error(request.id, error),
        }
    }

    /// Get the RPC context
    pub fn context(&self) -> &Arc<RpcContext> {
        &self.ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== MethodRegistry Tests =====

    #[test]
    fn test_method_registry_default_methods() {
        let registry = MethodRegistry::new();

        assert!(registry.has_method("eth_chainId"));
        assert!(registry.has_method("eth_blockNumber"));
        assert!(registry.has_method("eth_gasPrice"));
        assert!(registry.has_method("eth_getBalance"));
        assert!(registry.has_method("net_version"));
        assert!(registry.has_method("web3_clientVersion"));
        assert!(!registry.has_method("unknown_method"));
    }

    #[test]
    fn test_method_registry_names() {
        let registry = MethodRegistry::new();
        let names = registry.method_names();

        assert!(names.contains(&"eth_chainId"));
        assert!(names.contains(&"net_version"));
        assert!(names.contains(&"web3_sha3"));
    }

    #[test]
    fn test_method_registry_all_eth_methods() {
        let registry = MethodRegistry::new();

        // All eth_* methods should be registered
        let eth_methods = [
            "eth_chainId",
            "eth_blockNumber",
            "eth_gasPrice",
            "eth_getBalance",
            "eth_getTransactionCount",
            "eth_getCode",
            "eth_getStorageAt",
            "eth_call",
            "eth_estimateGas",
            "eth_sendRawTransaction",
            "eth_getBlockByNumber",
            "eth_getBlockByHash",
            "eth_getTransactionByHash",
            "eth_getTransactionReceipt",
        ];

        for method in eth_methods {
            assert!(registry.has_method(method), "Missing method: {}", method);
        }
    }

    #[test]
    fn test_method_registry_all_net_methods() {
        let registry = MethodRegistry::new();

        let net_methods = ["net_version", "net_listening", "net_peerCount"];

        for method in net_methods {
            assert!(registry.has_method(method), "Missing method: {}", method);
        }
    }

    #[test]
    fn test_method_registry_all_web3_methods() {
        let registry = MethodRegistry::new();

        let web3_methods = ["web3_clientVersion", "web3_sha3"];

        for method in web3_methods {
            assert!(registry.has_method(method), "Missing method: {}", method);
        }
    }

    #[test]
    fn test_method_registry_default() {
        let registry1 = MethodRegistry::new();
        let registry2 = MethodRegistry::default();

        // Both should have the same methods
        assert_eq!(registry1.method_names().len(), registry2.method_names().len());
    }

    #[test]
    fn test_method_registry_custom_handler() {
        let mut registry = MethodRegistry::new();

        async fn custom_handler(
            _ctx: Arc<RpcContext>,
            _params: Vec<Value>,
        ) -> Result<Value, JsonRpcError> {
            Ok(Value::String("custom".to_string()))
        }

        registry.register("custom_method", custom_handler);

        assert!(registry.has_method("custom_method"));
    }

    #[test]
    fn test_method_count() {
        let registry = MethodRegistry::new();
        let names = registry.method_names();

        // Should have all methods: 14 eth + 3 net + 2 web3 = 19
        assert_eq!(names.len(), 19);
    }
}
