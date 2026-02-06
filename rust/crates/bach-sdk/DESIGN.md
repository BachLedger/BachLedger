# Bach SDK Design Document

## Overview

`bach-sdk` is the Rust client SDK for interacting with BachLedger blockchain nodes. It provides:
- RPC client for node communication
- Transaction building and signing
- Account/wallet management
- Contract interaction helpers (ABI encoding/decoding)

## Architecture

```
bach-sdk/
  src/
    lib.rs           # Public API exports
    client.rs        # BachClient - main RPC client
    transport.rs     # Transport layer (HTTP/WebSocket)
    wallet.rs        # Wallet and account management
    signer.rs        # Transaction signing
    tx_builder.rs    # Transaction builder
    contract.rs      # Contract interaction helpers
    abi/
      mod.rs         # ABI module
      encode.rs      # ABI encoding
      decode.rs      # ABI decoding
      types.rs       # Solidity type mappings
    types.rs         # SDK-specific types
    error.rs         # Error types
```

## Core Components

### 1. BachClient

The main entry point for SDK users. Manages RPC connection and provides high-level APIs.

```rust
pub struct BachClient {
    transport: Box<dyn Transport>,
    chain_id: u64,
}

impl BachClient {
    // Constructor
    pub async fn connect(url: &str) -> Result<Self, SdkError>;
    pub fn new_mock() -> Self;  // For testing

    // Chain info
    pub async fn chain_id(&self) -> Result<u64, SdkError>;
    pub async fn gas_price(&self) -> Result<u128, SdkError>;
    pub async fn block_number(&self) -> Result<u64, SdkError>;

    // Account queries
    pub async fn get_balance(&self, address: &Address) -> Result<U256, SdkError>;
    pub async fn get_nonce(&self, address: &Address) -> Result<u64, SdkError>;
    pub async fn get_code(&self, address: &Address) -> Result<Bytes, SdkError>;

    // Block queries
    pub async fn get_block(&self, number: BlockId) -> Result<Option<Block>, SdkError>;
    pub async fn get_block_by_hash(&self, hash: &H256) -> Result<Option<Block>, SdkError>;

    // Transaction queries
    pub async fn get_transaction(&self, hash: &H256) -> Result<Option<SignedTransaction>, SdkError>;
    pub async fn get_receipt(&self, hash: &H256) -> Result<Option<Receipt>, SdkError>;

    // Transaction submission
    pub async fn send_raw_transaction(&self, tx: &[u8]) -> Result<H256, SdkError>;
    pub async fn send_transaction(&self, tx: &SignedTransaction) -> Result<H256, SdkError>;

    // Call (read-only execution)
    pub async fn call(&self, call: &CallRequest, block: BlockId) -> Result<Bytes, SdkError>;

    // Gas estimation
    pub async fn estimate_gas(&self, call: &CallRequest) -> Result<u64, SdkError>;
}
```

### 2. Transport Layer

Abstraction for RPC communication. Supports HTTP and WebSocket.

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &[serde_json::Value],
    ) -> Result<T, SdkError>;
}

pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
}

pub struct MockTransport {
    responses: HashMap<String, serde_json::Value>,
}
```

### 3. Wallet

Manages private keys and addresses. Supports in-memory and keystore-based wallets.

```rust
pub struct Wallet {
    private_key: PrivateKey,
    public_key: PublicKey,
    address: Address,
}

impl Wallet {
    // Creation
    pub fn new_random() -> Self;
    pub fn from_private_key(key: &[u8; 32]) -> Result<Self, SdkError>;
    pub fn from_private_key_hex(hex: &str) -> Result<Self, SdkError>;

    // Properties
    pub fn address(&self) -> &Address;
    pub fn public_key(&self) -> &PublicKey;

    // Signing
    pub fn sign_message(&self, message: &[u8]) -> Result<Signature, SdkError>;
    pub fn sign_transaction(&self, tx: &TransactionRequest) -> Result<SignedTransaction, SdkError>;

    // Keystore (optional, for persistence)
    pub fn to_keystore(&self, password: &str) -> Result<String, SdkError>;
    pub fn from_keystore(json: &str, password: &str) -> Result<Self, SdkError>;
}
```

### 4. Transaction Builder

Fluent API for constructing transactions.

```rust
pub struct TxBuilder {
    chain_id: u64,
    nonce: Option<u64>,
    gas_limit: Option<u64>,
    gas_price: Option<u128>,
    max_fee_per_gas: Option<u128>,
    max_priority_fee_per_gas: Option<u128>,
    to: Option<Address>,
    value: u128,
    data: Bytes,
}

impl TxBuilder {
    pub fn new(chain_id: u64) -> Self;

    // Setters (chainable)
    pub fn nonce(self, nonce: u64) -> Self;
    pub fn gas_limit(self, limit: u64) -> Self;
    pub fn gas_price(self, price: u128) -> Self;
    pub fn max_fee_per_gas(self, fee: u128) -> Self;
    pub fn max_priority_fee_per_gas(self, fee: u128) -> Self;
    pub fn to(self, address: Address) -> Self;
    pub fn value(self, value: u128) -> Self;
    pub fn data(self, data: Bytes) -> Self;

    // Build
    pub fn build_legacy(self) -> Result<LegacyTx, SdkError>;
    pub fn build_eip1559(self) -> Result<DynamicFeeTx, SdkError>;

    // Sign and build
    pub fn sign(self, wallet: &Wallet) -> Result<SignedTransaction, SdkError>;
}
```

### 5. Contract Interaction

Helpers for encoding/decoding contract calls.

```rust
pub struct Contract {
    address: Address,
    abi: Abi,
}

impl Contract {
    pub fn new(address: Address, abi_json: &str) -> Result<Self, SdkError>;

    // Encode function call
    pub fn encode_call(&self, function: &str, args: &[Token]) -> Result<Bytes, SdkError>;

    // Decode function return
    pub fn decode_output(&self, function: &str, data: &[u8]) -> Result<Vec<Token>, SdkError>;

    // Decode event log
    pub fn decode_log(&self, log: &Log) -> Result<DecodedLog, SdkError>;
}
```

### 6. ABI Encoding/Decoding

Solidity ABI encoding following the specification.

```rust
/// Solidity type tokens
pub enum Token {
    Address(Address),
    Uint(U256, usize),      // value, bits (8-256)
    Int(I256, usize),       // value, bits (8-256)
    Bool(bool),
    Bytes(Vec<u8>),
    FixedBytes(Vec<u8>, usize), // value, size (1-32)
    String(String),
    Array(Vec<Token>),
    FixedArray(Vec<Token>, usize),
    Tuple(Vec<Token>),
}

pub fn encode(tokens: &[Token]) -> Vec<u8>;
pub fn decode(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, AbiError>;
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("Transport error: {0}")]
    Transport(String),

    #[error("RPC error: {code} - {message}")]
    Rpc { code: i64, message: String },

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("ABI encoding error: {0}")]
    AbiEncode(String),

    #[error("ABI decoding error: {0}")]
    AbiDecode(String),

    #[error("Transaction build error: {0}")]
    TxBuild(String),

    #[error("Invalid hex: {0}")]
    InvalidHex(String),
}
```

## Usage Examples

### Basic Transfer

```rust
use bach_sdk::{BachClient, Wallet, TxBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to node
    let client = BachClient::connect("http://localhost:8545").await?;

    // Load wallet
    let wallet = Wallet::from_private_key_hex("0x...")?;

    // Get nonce and gas price
    let nonce = client.get_nonce(wallet.address()).await?;
    let gas_price = client.gas_price().await?;

    // Build and sign transaction
    let tx = TxBuilder::new(client.chain_id().await?)
        .nonce(nonce)
        .gas_limit(21000)
        .gas_price(gas_price)
        .to("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d".parse()?)
        .value(1_000_000_000_000_000_000) // 1 ETH
        .sign(&wallet)?;

    // Send transaction
    let tx_hash = client.send_transaction(&tx).await?;
    println!("Transaction sent: {}", tx_hash.to_hex());

    Ok(())
}
```

### Contract Interaction

```rust
use bach_sdk::{BachClient, Contract, Token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BachClient::connect("http://localhost:8545").await?;

    // Load contract
    let abi = include_str!("erc20.json");
    let contract = Contract::new(
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?,
        abi,
    )?;

    // Encode balanceOf call
    let call_data = contract.encode_call(
        "balanceOf",
        &[Token::Address("0x...".parse()?)],
    )?;

    // Execute call
    let result = client.call(&CallRequest {
        to: Some(contract.address()),
        data: Some(call_data),
        ..Default::default()
    }, BlockId::Latest).await?;

    // Decode result
    let tokens = contract.decode_output("balanceOf", &result)?;
    println!("Balance: {:?}", tokens[0]);

    Ok(())
}
```

## Dependencies

```toml
[dependencies]
bach-primitives = { path = "../bach-primitives" }
bach-crypto = { path = "../bach-crypto" }
bach-types = { path = "../bach-types" }

tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
async-trait = "0.1"
hex = "0.4"
bytes = "1"
```

## Testing Strategy

1. **Unit Tests**: Test individual components (encoding, signing, building)
2. **Mock Tests**: Use `MockTransport` to test client logic without network
3. **Integration Tests**: Test against local devnet (optional, in bach-e2e)

## Implementation Order

1. `error.rs` - Error types
2. `types.rs` - SDK types (CallRequest, BlockId, etc.)
3. `transport.rs` - Transport trait and MockTransport
4. `wallet.rs` - Wallet implementation
5. `signer.rs` - Transaction signing logic
6. `tx_builder.rs` - Transaction builder
7. `client.rs` - BachClient with mock support
8. `abi/` - ABI encoding/decoding
9. `contract.rs` - Contract helpers

## Future Enhancements

- WebSocket transport with subscription support
- Hardware wallet integration (Ledger, Trezor)
- ENS resolution
- Multicall batching
- Gas estimation strategies
- Transaction receipt polling/waiting
