# bach-rpc Design Document

## Overview

`bach-rpc` provides a JSON-RPC 2.0 server implementation compatible with Ethereum RPC standards. It serves as the network interface for BachLedger nodes, enabling `bach-sdk` clients to interact with the blockchain.

## Architecture

```
+--------------------+
|    bach-sdk        |  <- Client (HttpTransport)
|  (BachClient)      |
+--------------------+
         |
         | HTTP POST (JSON-RPC 2.0)
         v
+--------------------+
|    bach-rpc        |  <- Server (Port 8545)
|   (RpcServer)      |
+--------------------+
         |
    +----+----+
    |         |
    v         v
+-------+  +--------+
| State |  | TxPool |
+-------+  +--------+
    |         |
    v         v
+--------------------+
|    bach-core       |  <- BlockExecutor
+--------------------+
         |
         v
+--------------------+
|   bach-storage     |  <- RocksDB
+--------------------+
```

## Module Structure

```
bach-rpc/
  src/
    lib.rs           # Public API
    server.rs        # HTTP server (axum/tower)
    handler.rs       # Request routing
    methods/
      mod.rs         # Method registry
      eth.rs         # eth_* methods
      net.rs         # net_* methods
      web3.rs        # web3_* methods
    types.rs         # RPC-specific types
    error.rs         # RPC error codes
```

## Dependencies

```toml
[dependencies]
bach-primitives = { path = "../bach-primitives" }
bach-types = { path = "../bach-types" }
bach-storage = { path = "../bach-storage" }
bach-txpool = { path = "../bach-txpool" }
bach-core = { path = "../bach-core" }
bach-crypto = { path = "../bach-crypto" }

# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP server
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# JSON-RPC
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utilities
tracing = "0.1"
hex = "0.4"
bytes = "1"
```

## Core Types

### JSON-RPC Request/Response

```rust
/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub method: String,
    #[serde(default)]
    pub params: Vec<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC Error
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard error codes
pub mod error_code {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;

    // Ethereum-specific
    pub const EXECUTION_ERROR: i64 = 3;
    pub const TRANSACTION_REJECTED: i64 = -32003;
}
```

### RPC Context

```rust
/// Shared state for RPC handlers
pub struct RpcContext {
    /// State database access
    pub state_db: Arc<StateDb>,
    /// Block database access
    pub block_db: Arc<BlockDb>,
    /// Transaction pool
    pub txpool: Arc<TxPool>,
    /// Block executor
    pub executor: Arc<RwLock<BlockExecutor>>,
    /// Chain configuration
    pub chain_id: u64,
    /// Current gas price
    pub gas_price: AtomicU128,
}

impl RpcContext {
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
            executor: Arc::new(RwLock::new(BlockExecutor::new(chain_id))),
            chain_id,
            gas_price: AtomicU128::new(1_000_000_000), // 1 gwei default
        }
    }
}
```

## RPC Methods Implementation

### eth_* Methods (Priority Order)

| Method | Description | Implementation Notes |
|--------|-------------|---------------------|
| `eth_chainId` | Return chain ID | Constant from config |
| `eth_blockNumber` | Latest block number | Read from BlockDb |
| `eth_gasPrice` | Current gas price | From RpcContext |
| `eth_getBalance` | Account balance | Read from StateDb |
| `eth_getTransactionCount` | Account nonce | Read from StateDb |
| `eth_getCode` | Contract bytecode | Read from StateDb |
| `eth_call` | Execute read-only call | BlockExecutor simulation |
| `eth_estimateGas` | Estimate gas usage | BlockExecutor simulation |
| `eth_sendRawTransaction` | Submit transaction | TxPool.add() |
| `eth_getTransactionReceipt` | Get receipt | Read from BlockDb |
| `eth_getBlockByNumber` | Get block by number | Read from BlockDb |
| `eth_getBlockByHash` | Get block by hash | Read from BlockDb |
| `eth_getTransactionByHash` | Get transaction | TxPool or BlockDb |
| `eth_getStorageAt` | Read storage slot | Read from StateDb |
| `eth_getLogs` | Query event logs | Scan BlockDb receipts |

### Method Implementation Example

```rust
/// eth_getBalance implementation
pub async fn eth_get_balance(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    // Parse parameters
    if params.len() < 2 {
        return Err(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "missing required parameters".into(),
            data: None,
        });
    }

    let address = parse_address(&params[0])?;
    let block_id = parse_block_id(&params[1])?;

    // Get state at specified block
    let balance = match block_id {
        BlockId::Latest | BlockId::Pending => {
            ctx.state_db.get_balance(&address)?
        }
        BlockId::Number(n) => {
            // Historical state query (if supported)
            ctx.state_db.get_balance_at(&address, n)?
        }
        _ => ctx.state_db.get_balance(&address)?,
    };

    Ok(Value::String(format!("0x{:x}", balance)))
}

/// eth_sendRawTransaction implementation
pub async fn eth_send_raw_transaction(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    let raw_tx = parse_hex_bytes(&params[0])?;

    // Decode and validate transaction
    let signed_tx = decode_transaction(&raw_tx)?;
    let sender = recover_sender(&signed_tx)?;
    let hash = compute_tx_hash(&raw_tx);

    // Add to transaction pool
    ctx.txpool.add(signed_tx, sender, hash)
        .map_err(|e| JsonRpcError {
            code: error_code::TRANSACTION_REJECTED,
            message: e.to_string(),
            data: None,
        })?;

    Ok(Value::String(hash.to_hex()))
}

/// eth_call implementation
pub async fn eth_call(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    let call_request: CallRequest = serde_json::from_value(params[0].clone())?;
    let _block_id = parse_block_id(&params[1])?;

    // Build execution environment
    let executor = ctx.executor.read().await;

    // Execute call (read-only, no state changes)
    let result = executor.simulate_call(
        call_request.from.unwrap_or(Address::ZERO),
        call_request.to.unwrap_or(Address::ZERO),
        call_request.data.unwrap_or_default().to_vec(),
        call_request.gas.unwrap_or(30_000_000),
        call_request.value.unwrap_or(U256::ZERO).as_u128(),
    )?;

    Ok(Value::String(format!("0x{}", hex::encode(result))))
}
```

### net_* Methods

| Method | Description |
|--------|-------------|
| `net_version` | Network ID (same as chain ID) |
| `net_listening` | Always returns true |
| `net_peerCount` | Returns "0x0" (single node) |

### web3_* Methods

| Method | Description |
|--------|-------------|
| `web3_clientVersion` | "BachLedger/0.1.0" |
| `web3_sha3` | Keccak256 hash |

## HTTP Server

### Server Configuration

```rust
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
```

### Server Implementation

```rust
pub struct RpcServer {
    config: ServerConfig,
    context: Arc<RpcContext>,
}

impl RpcServer {
    pub fn new(config: ServerConfig, context: Arc<RpcContext>) -> Self {
        Self { config, context }
    }

    pub async fn run(self) -> Result<(), RpcError> {
        let app = Router::new()
            .route("/", post(handle_rpc))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
                    .layer(TimeoutLayer::new(self.config.request_timeout))
                    .layer(DefaultBodyLimit::max(self.config.max_body_size))
            )
            .with_state(self.context);

        let listener = TcpListener::bind(self.config.listen_addr).await?;
        tracing::info!("RPC server listening on {}", self.config.listen_addr);

        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_rpc(
    State(ctx): State<Arc<RpcContext>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let result = dispatch_method(&ctx, &request.method, request.params).await;

    Json(match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id,
            result: None,
            error: Some(error),
        },
    })
}
```

### Method Dispatcher

```rust
pub type MethodHandler = Box<dyn Fn(Arc<RpcContext>, Vec<Value>) -> BoxFuture<'static, Result<Value, JsonRpcError>> + Send + Sync>;

pub struct MethodRegistry {
    methods: HashMap<String, MethodHandler>,
}

impl MethodRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
        };

        // Register eth_* methods
        registry.register("eth_chainId", eth_chain_id);
        registry.register("eth_blockNumber", eth_block_number);
        registry.register("eth_gasPrice", eth_gas_price);
        registry.register("eth_getBalance", eth_get_balance);
        registry.register("eth_getTransactionCount", eth_get_transaction_count);
        registry.register("eth_getCode", eth_get_code);
        registry.register("eth_call", eth_call);
        registry.register("eth_estimateGas", eth_estimate_gas);
        registry.register("eth_sendRawTransaction", eth_send_raw_transaction);
        registry.register("eth_getTransactionReceipt", eth_get_transaction_receipt);
        registry.register("eth_getBlockByNumber", eth_get_block_by_number);
        registry.register("eth_getBlockByHash", eth_get_block_by_hash);
        registry.register("eth_getTransactionByHash", eth_get_transaction_by_hash);
        registry.register("eth_getStorageAt", eth_get_storage_at);

        // Register net_* methods
        registry.register("net_version", net_version);
        registry.register("net_listening", net_listening);
        registry.register("net_peerCount", net_peer_count);

        // Register web3_* methods
        registry.register("web3_clientVersion", web3_client_version);
        registry.register("web3_sha3", web3_sha3);

        registry
    }

    pub async fn dispatch(
        &self,
        ctx: Arc<RpcContext>,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, JsonRpcError> {
        match self.methods.get(method) {
            Some(handler) => handler(ctx, params).await,
            None => Err(JsonRpcError {
                code: error_code::METHOD_NOT_FOUND,
                message: format!("method not found: {}", method),
                data: None,
            }),
        }
    }
}
```

---

# bach-node Design

## Overview

`bach-node` is the main entry point for running a BachLedger node. It initializes all components, processes the genesis block, and runs the main event loop.

## Architecture

```
+-------------------+
|    CLI Parser     |  <- clap
+-------------------+
         |
         v
+-------------------+
|  Configuration    |  <- Load from file/env
+-------------------+
         |
         v
+-------------------+
|   Initialization  |
| - Open Database   |
| - Genesis Block   |
| - TxPool          |
| - BlockExecutor   |
+-------------------+
         |
    +----+----+
    |         |
    v         v
+-------+  +--------+
|  RPC  |  |  Main  |
|Server |  |  Loop  |
+-------+  +--------+
```

## Module Structure

```
bach-node/
  src/
    main.rs          # Entry point
    cli.rs           # CLI argument parsing
    config.rs        # Configuration types
    genesis.rs       # Genesis block handling
    node.rs          # Node orchestration
```

## CLI Arguments

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "bach-node")]
#[command(about = "BachLedger blockchain node")]
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

    /// Enable RPC server
    #[arg(long, default_value = "true")]
    pub rpc: bool,

    /// Genesis file path (optional)
    #[arg(long)]
    pub genesis: Option<PathBuf>,

    /// Block gas limit
    #[arg(long, default_value = "30000000")]
    pub gas_limit: u64,

    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
}
```

## Configuration

```rust
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
}

#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub enabled: bool,
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
}

#[derive(Debug, Clone)]
pub struct GenesisConfig {
    /// Initial account allocations
    pub alloc: HashMap<Address, GenesisAccount>,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Genesis extra data
    pub extra_data: Bytes,
    /// Initial difficulty
    pub difficulty: u64,
    /// Initial gas limit
    pub gas_limit: u64,
}

#[derive(Debug, Clone)]
pub struct GenesisAccount {
    pub balance: U256,
    pub nonce: u64,
    pub code: Option<Bytes>,
    pub storage: HashMap<H256, H256>,
}

#[derive(Debug, Clone)]
pub struct BlockConfig {
    pub gas_limit: u64,
    pub block_time: Duration,
}
```

## Genesis Block Handling

```rust
pub struct GenesisBuilder {
    config: GenesisConfig,
    chain_id: u64,
}

impl GenesisBuilder {
    pub fn new(config: GenesisConfig, chain_id: u64) -> Self {
        Self { config, chain_id }
    }

    /// Initialize genesis state in database
    pub fn init_genesis(
        &self,
        state_db: &mut StateDb,
        block_db: &BlockDb,
    ) -> Result<Block, GenesisError> {
        // Check if genesis already exists
        if block_db.get_latest_block()?.is_some() {
            return Err(GenesisError::AlreadyInitialized);
        }

        // Apply initial allocations
        for (address, account) in &self.config.alloc {
            let state_account = Account {
                nonce: account.nonce,
                balance: account.balance.as_u128(),
                code_hash: if let Some(code) = &account.code {
                    let hash = keccak256(code);
                    state_db.set_code(hash, code.to_vec())?;
                    hash
                } else {
                    EMPTY_CODE_HASH
                },
                storage_root: EMPTY_STORAGE_ROOT,
            };

            state_db.set_account(*address, state_account)?;

            // Set initial storage
            for (key, value) in &account.storage {
                state_db.set_storage(*address, *key, *value)?;
            }
        }

        // Build genesis block
        let genesis_block = Block {
            header: BlockHeader {
                parent_hash: H256::ZERO,
                ommers_hash: EMPTY_OMMERS_HASH,
                beneficiary: Address::ZERO,
                state_root: state_db.compute_root()?,
                transactions_root: EMPTY_TX_ROOT,
                receipts_root: EMPTY_RECEIPTS_ROOT,
                logs_bloom: Bloom::default(),
                difficulty: self.config.difficulty,
                number: 0,
                gas_limit: self.config.gas_limit,
                gas_used: 0,
                timestamp: self.config.timestamp,
                extra_data: self.config.extra_data.clone(),
                mix_hash: H256::ZERO,
                nonce: 0,
                base_fee_per_gas: Some(1_000_000_000), // 1 gwei
            },
            body: BlockBody {
                transactions: vec![],
            },
        };

        // Store genesis block
        let hash = genesis_block.hash();
        block_db.put_header(&hash, &encode_header(&genesis_block.header))?;
        block_db.put_body(&hash, &encode_body(&genesis_block.body))?;
        block_db.put_hash_by_number(0, &hash)?;
        block_db.set_latest_block(0)?;
        block_db.set_finalized_block(0)?;

        tracing::info!("Genesis block initialized: {}", hash.to_hex());

        Ok(genesis_block)
    }
}
```

## Node Implementation

```rust
pub struct Node {
    config: NodeConfig,
    state_db: Arc<StateDb>,
    block_db: Arc<BlockDb>,
    txpool: Arc<TxPool>,
    executor: Arc<RwLock<BlockExecutor>>,
    rpc_server: Option<RpcServer>,
}

impl Node {
    pub async fn new(config: NodeConfig) -> Result<Self, NodeError> {
        // Open database
        let db = Database::new(&config.datadir.join("db"));
        db.open()?;

        let state_db = Arc::new(StateDb::new(db.clone()));
        let block_db = Arc::new(BlockDb::new(db));

        // Initialize genesis if needed
        let genesis_builder = GenesisBuilder::new(
            config.genesis.clone(),
            config.chain_id,
        );

        if block_db.get_latest_block()?.is_none() {
            genesis_builder.init_genesis(&mut state_db.clone(), &block_db)?;
        }

        // Create transaction pool
        let txpool = Arc::new(TxPool::with_defaults());

        // Create block executor
        let executor = Arc::new(RwLock::new(
            BlockExecutor::new(config.chain_id)
        ));

        // Create RPC server if enabled
        let rpc_server = if config.rpc.enabled {
            let rpc_context = Arc::new(RpcContext::new(
                state_db.clone(),
                block_db.clone(),
                txpool.clone(),
                config.chain_id,
            ));

            Some(RpcServer::new(
                ServerConfig {
                    listen_addr: config.rpc.listen_addr,
                    ..Default::default()
                },
                rpc_context,
            ))
        } else {
            None
        };

        Ok(Self {
            config,
            state_db,
            block_db,
            txpool,
            executor,
            rpc_server,
        })
    }

    pub async fn run(self) -> Result<(), NodeError> {
        tracing::info!("Starting BachLedger node...");
        tracing::info!("Chain ID: {}", self.config.chain_id);
        tracing::info!("Data directory: {:?}", self.config.datadir);

        // Start RPC server in background
        if let Some(rpc_server) = self.rpc_server {
            tokio::spawn(async move {
                if let Err(e) = rpc_server.run().await {
                    tracing::error!("RPC server error: {}", e);
                }
            });
        }

        // Main event loop
        self.main_loop().await
    }

    async fn main_loop(&self) -> Result<(), NodeError> {
        let mut interval = tokio::time::interval(
            self.config.block.block_time
        );

        loop {
            interval.tick().await;

            // Check for pending transactions
            let pending = self.txpool.get_pending(100);

            if !pending.is_empty() {
                tracing::debug!("Processing {} pending transactions", pending.len());

                // Build and execute block
                if let Err(e) = self.produce_block(pending).await {
                    tracing::error!("Block production error: {}", e);
                }
            }
        }
    }

    async fn produce_block(
        &self,
        transactions: Vec<PooledTransaction>,
    ) -> Result<(), NodeError> {
        let mut executor = self.executor.write().await;

        // Get latest block info
        let latest_number = self.block_db.get_latest_block()?.unwrap_or(0);
        let latest_hash = self.block_db.get_hash_by_number(latest_number)?
            .unwrap_or(H256::ZERO);

        // Build block
        let block = Block {
            header: BlockHeader {
                parent_hash: latest_hash,
                number: latest_number + 1,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                gas_limit: self.config.block.gas_limit,
                beneficiary: Address::ZERO, // TODO: Configure coinbase
                base_fee_per_gas: Some(1_000_000_000),
                ..Default::default()
            },
            body: BlockBody {
                transactions: transactions.iter().map(|pt| pt.tx.clone()).collect(),
            },
        };

        // Execute block
        let result = executor.execute_block(&block)?;

        // Store block and receipts
        let hash = block.hash();
        self.block_db.put_header(&hash, &encode_header(&block.header))?;
        self.block_db.put_body(&hash, &encode_body(&block.body))?;
        self.block_db.put_receipts(&hash, &encode_receipts(&result.receipts))?;
        self.block_db.put_hash_by_number(block.header.number, &hash)?;
        self.block_db.set_latest_block(block.header.number)?;

        // Update transaction pool nonces
        for tx in &transactions {
            self.txpool.set_nonce(&tx.sender, tx.nonce() + 1);
        }

        tracing::info!(
            "Block {} produced: {} txs, {} gas used",
            block.header.number,
            transactions.len(),
            result.gas_used
        );

        Ok(())
    }
}
```

## Main Entry Point

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_str(&cli.log_level).unwrap_or_default())
        .init();

    // Build configuration
    let genesis_config = if let Some(genesis_path) = &cli.genesis {
        load_genesis_file(genesis_path)?
    } else {
        default_genesis_config()
    };

    let config = NodeConfig {
        datadir: cli.datadir,
        chain_id: cli.chain_id,
        rpc: RpcConfig {
            enabled: cli.rpc,
            listen_addr: cli.rpc_addr,
            max_connections: 100,
        },
        genesis: genesis_config,
        block: BlockConfig {
            gas_limit: cli.gas_limit,
            block_time: Duration::from_secs(1),
        },
    };

    // Create and run node
    let node = Node::new(config).await?;
    node.run().await?;

    Ok(())
}

fn default_genesis_config() -> GenesisConfig {
    let mut alloc = HashMap::new();

    // Pre-fund test accounts
    let test_accounts = [
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", // Hardhat account #0
    ];

    for addr_str in test_accounts {
        let addr = Address::from_hex(addr_str).unwrap();
        alloc.insert(addr, GenesisAccount {
            balance: U256::from(10_000_000_000_000_000_000_000u128), // 10000 ETH
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        });
    }

    GenesisConfig {
        alloc,
        timestamp: 0,
        extra_data: Bytes::new(),
        difficulty: 1,
        gas_limit: 30_000_000,
    }
}
```

---

## Implementation Phases

### Phase 1: Core RPC (Priority: High)

1. Set up bach-rpc crate structure
2. Implement JSON-RPC types and error handling
3. Implement basic HTTP server with axum
4. Implement core eth_* methods:
   - `eth_chainId`
   - `eth_blockNumber`
   - `eth_gasPrice`
   - `eth_getBalance`
   - `eth_getTransactionCount`
   - `eth_sendRawTransaction`

### Phase 2: Extended RPC (Priority: Medium)

1. Implement remaining eth_* methods:
   - `eth_call`
   - `eth_estimateGas`
   - `eth_getCode`
   - `eth_getStorageAt`
   - `eth_getBlockByNumber`
   - `eth_getBlockByHash`
   - `eth_getTransactionReceipt`
   - `eth_getTransactionByHash`

2. Implement net_* and web3_* methods

### Phase 3: Node Implementation (Priority: High)

1. Implement CLI argument parsing
2. Implement configuration loading
3. Implement genesis block handling
4. Implement main event loop
5. Implement block production

### Phase 4: Integration & Testing

1. Integration tests with bach-sdk
2. End-to-end transaction flow tests
3. Performance benchmarking

---

## Compatibility Notes

### bach-sdk Integration

The RPC server is designed to be fully compatible with `bach-sdk`'s `HttpTransport`:

```rust
// Client usage (bach-sdk)
let client = BachClient::connect("http://localhost:8545").await?;
let balance = client.get_balance(&address, BlockId::Latest).await?;

// Server (bach-rpc) receives:
// POST / HTTP/1.1
// {"jsonrpc":"2.0","id":1,"method":"eth_getBalance","params":["0x...", "latest"]}
```

### Response Format

All responses follow the JSON-RPC 2.0 format expected by `bach-sdk`:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0xde0b6b3a7640000"
}
```

Or for errors:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found: eth_unknownMethod"
  }
}
```
