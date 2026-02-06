//! HTTP server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

use crate::error::{RpcError, RpcResult};
use crate::handler::RpcHandler;
use crate::types::{JsonRpcRequest, JsonRpcResponse};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Listen address
    pub listen_addr: SocketAddr,
    /// Maximum request body size (default: 10MB)
    pub max_body_size: usize,
    /// Request timeout (default: 30s)
    pub request_timeout: Duration,
    /// Enable CORS (default: true)
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8545".parse().unwrap(),
            max_body_size: 10 * 1024 * 1024,
            request_timeout: Duration::from_secs(30),
            enable_cors: true,
        }
    }
}

impl ServerConfig {
    /// Create a new server config with the given address
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            ..Default::default()
        }
    }
}

/// RPC server state
pub struct ServerState {
    /// RPC handler for processing requests
    pub handler: RpcHandler,
}

impl ServerState {
    /// Create new server state
    pub fn new(handler: RpcHandler) -> Self {
        Self { handler }
    }
}

/// RPC HTTP server
pub struct RpcServer {
    config: ServerConfig,
    state: Arc<ServerState>,
}

impl RpcServer {
    /// Create a new RPC server
    pub fn new(config: ServerConfig, handler: RpcHandler) -> Self {
        Self {
            config,
            state: Arc::new(ServerState::new(handler)),
        }
    }

    /// Build the router
    fn build_router(&self) -> Router {
        let mut router = Router::new()
            .route("/", post(handle_rpc))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(RequestBodyLimitLayer::new(self.config.max_body_size)),
            );

        if self.config.enable_cors {
            router = router.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );
        }

        router.with_state(self.state.clone())
    }

    /// Run the server
    pub async fn run(self) -> RpcResult<()> {
        let app = self.build_router();

        let listener = TcpListener::bind(self.config.listen_addr).await?;
        tracing::info!("RPC server listening on {}", self.config.listen_addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| RpcError::Bind(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    /// Get the server listen address
    pub fn listen_addr(&self) -> SocketAddr {
        self.config.listen_addr
    }
}

/// Handle JSON-RPC requests
async fn handle_rpc(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let response = state.handler.handle_request(request).await;
    Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.listen_addr.port(), 8545);
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert!(config.enable_cors);
    }

    #[test]
    fn test_server_config_new() {
        let addr: SocketAddr = "127.0.0.1:9545".parse().unwrap();
        let config = ServerConfig::new(addr);
        assert_eq!(config.listen_addr.port(), 9545);
    }
}
