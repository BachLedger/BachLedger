# BachLedger

A high-performance Ethereum-compatible blockchain implementation in Rust.

## Project Structure

```
rust/
├── crates/
│   ├── bach-primitives/   # 基础类型 (Address, H256, U256)
│   ├── bach-crypto/       # 密码学 (keccak256, secp256k1 签名)
│   ├── bach-types/        # 交易和区块类型
│   ├── bach-rlp/          # RLP 编码/解码
│   ├── bach-storage/      # RocksDB 存储层
│   ├── bach-evm/          # EVM 执行引擎
│   ├── bach-scheduler/    # 交易调度器
│   ├── bach-consensus/    # 共识模块
│   ├── bach-network/      # P2P 网络
│   ├── bach-txpool/       # 交易池
│   ├── bach-metrics/      # 指标收集
│   ├── bach-core/         # 核心组件集成
│   ├── bach-rpc/          # JSON-RPC 服务器
│   ├── bach-node/         # 完整节点实现
│   ├── bach-sdk/          # 客户端 SDK
│   ├── bach-cli/          # 命令行工具
│   ├── bach-e2e/          # 端到端测试
│   └── bach-evm-tests/    # EVM 测试套件
├── examples/
│   └── manual_test.rs     # 手动测试脚本
└── scripts/
    └── run_manual_test.sh # 自动化测试脚本
```

## Quick Start

### Build

```bash
cargo build --release
```

### Run Tests

```bash
# 运行所有测试
cargo test

# 运行特定 crate 测试
cargo test -p bach-rpc
cargo test -p bach-node
cargo test -p bach-cli
```

### Start Node

```bash
cargo run -p bach-node --release -- \
    --datadir ./testdata \
    --chain-id 1337 \
    --rpc-addr 0.0.0.0:8545 \
    --log-level info
```

## Manual Testing Guide

### Method 1: Automated Script (Recommended)

```bash
./scripts/run_manual_test.sh
```

This script automatically:
1. Cleans old test data
2. Compiles and starts the node
3. Waits for node readiness
4. Runs complete E2E tests
5. Shuts down the node

### Method 2: Step-by-Step Manual Testing

#### 1. Generate Cryptographic Materials

```bash
# Generate a new account (shows all derived values)
./target/release/bach keygen new --show-private-key

# Generate a node identity key for P2P
./target/release/bach keygen node-key -o node.key

# Generate encrypted keystore
./target/release/bach keygen new -o my-account.json -p "my-password"

# Batch generate test accounts
./target/release/bach keygen batch -c 5 -o ./test-keys

# Derive address from private key
./target/release/bach keygen derive -k 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

#### 2. Start the Node

```bash
# In terminal 1: Start the node
cargo run -p bach-node --release -- \
    --datadir ./testdata \
    --chain-id 1337 \
    --rpc-addr 0.0.0.0:8545 \
    --log-level info
```

#### 3. Query Chain State (CLI)

```bash
# Query chain ID
./target/release/bach query chain-id

# Query latest block
./target/release/bach query block latest

# Query gas price
./target/release/bach query gas-price

# Query account balance
./target/release/bach query balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

#### 4. Send Transactions (CLI)

```bash
# Send ETH transfer
./target/release/bach tx send \
    --to 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
    --amount 1.0 \
    --key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Deploy contract
./target/release/bach tx deploy \
    --bytecode 0x608060405234801561001057600080fd5b50... \
    --key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Call contract
./target/release/bach tx call \
    --contract 0x5FbDB2315678afecb367f032d93F642f64180aa3 \
    --data 0x60fe47b10000000000000000000000000000000000000000000000000000000000000042 \
    --key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

#### 5. Direct RPC Calls (curl)

```bash
# Get Chain ID
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Get Block Number
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get Balance
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","latest"],"id":1}'

# Get Transaction Count (nonce)
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getTransactionCount","params":["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","latest"],"id":1}'

# Get Gas Price
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_gasPrice","params":[],"id":1}'

# Send Raw Transaction
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0xf86c..."],"id":1}'

# Call Contract (read-only)
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_call","params":[{"to":"0x5FbDB2315678afecb367f032d93F642f64180aa3","data":"0x6d4ce63c"},"latest"],"id":1}'

# Get Contract Code
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getCode","params":["0x5FbDB2315678afecb367f032d93F642f64180aa3","latest"],"id":1}'

# Get Transaction Receipt
curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0x...txhash..."],"id":1}'
```

#### 6. Run Complete E2E Test

```bash
# Make sure node is running on localhost:8545
cargo run --example manual_test --release
```

### Test Coverage

| Step | Content |
|------|---------|
| 1. Connection Test | Verify Chain ID, block height, gas price |
| 2. Account Query | Query test account balance and nonce |
| 3. ETH Transfer | Send 1 ETH to another account |
| 4. Contract Deployment | Deploy SimpleStorage contract |
| 5. Contract Interaction | Call set() and get() methods |
| 6. State Verification | Verify final block height and balance |

### Pre-configured Test Accounts

These are standard Hardhat/Foundry test accounts, pre-funded in genesis:

| Account | Address | Private Key |
|---------|---------|-------------|
| #0 | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80` |
| #1 | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d` |
| #2 | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | `0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a` |

Each account is pre-funded with **10,000 ETH** for testing.

> **Warning**: These private keys are publicly known. Never use them on mainnet or with real funds.

## CLI Reference

### Account Commands

```bash
bach account create              # Create new account
bach account create -n myaccount # Create with name
bach account list                # List accounts in keystore
bach account balance <address>   # Get balance
bach account import -k <key>     # Import from private key
```

### Keygen Commands

```bash
bach keygen new                        # Generate new keypair
bach keygen new --show-private-key     # Show private key
bach keygen new -o key.json            # Save to file
bach keygen new -o key.json -p pass    # Save encrypted keystore
bach keygen node-key                   # Generate node identity
bach keygen batch -c 10 -o ./keys      # Batch generate
bach keygen inspect <keystore.json>    # Inspect keystore
bach keygen decrypt <file> -p pass     # Decrypt keystore
bach keygen derive -k <private_key>    # Derive address from key
```

### Transaction Commands

```bash
bach tx send --to <addr> --amount <eth> --key <privkey>
bach tx deploy --bytecode <hex> --key <privkey>
bach tx call --contract <addr> --data <hex> --key <privkey>
```

### Query Commands

```bash
bach query chain-id
bach query block latest
bach query block <number>
bach query tx <hash>
bach query gas-price
bach query balance <address>
```

### Global Options

```bash
--json          # Output in JSON format
--rpc-url <url> # Override RPC endpoint (default: http://localhost:8545)
```

## Supported RPC Methods

### eth_* Methods

| Method | Description |
|--------|-------------|
| `eth_chainId` | Get chain ID |
| `eth_blockNumber` | Get latest block number |
| `eth_gasPrice` | Get current gas price |
| `eth_getBalance` | Get account balance |
| `eth_getTransactionCount` | Get account nonce |
| `eth_getCode` | Get contract code |
| `eth_getStorageAt` | Get storage value |
| `eth_call` | Execute read-only call |
| `eth_estimateGas` | Estimate gas for transaction |
| `eth_sendRawTransaction` | Submit signed transaction |
| `eth_getTransactionByHash` | Get transaction by hash |
| `eth_getTransactionReceipt` | Get transaction receipt |
| `eth_getBlockByNumber` | Get block by number |
| `eth_getBlockByHash` | Get block by hash |
| `eth_getLogs` | Get event logs |
| `eth_syncing` | Get sync status |
| `eth_accounts` | Get accounts (empty) |
| `eth_coinbase` | Get coinbase address |
| `eth_mining` | Get mining status |
| `eth_hashrate` | Get hashrate |

### net_* Methods

| Method | Description |
|--------|-------------|
| `net_version` | Get network ID |
| `net_listening` | Get listening status |
| `net_peerCount` | Get peer count |

### web3_* Methods

| Method | Description |
|--------|-------------|
| `web3_clientVersion` | Get client version |
| `web3_sha3` | Keccak256 hash |

## Configuration

### Node Configuration

```bash
cargo run -p bach-node --release -- \
    --datadir ./data           # Data directory
    --chain-id 1337            # Chain ID
    --rpc-addr 0.0.0.0:8545    # RPC listen address
    --log-level info           # Log level (trace/debug/info/warn/error)
```

### Genesis Configuration

The node automatically creates a genesis block with:
- Chain ID: configurable (default 1337)
- Pre-funded test accounts
- Initial gas limit: 30,000,000

## Development

### Running Specific Test Suites

```bash
# Unit tests
cargo test -p bach-primitives
cargo test -p bach-crypto
cargo test -p bach-types
cargo test -p bach-rlp

# Integration tests
cargo test -p bach-rpc
cargo test -p bach-node
cargo test -p bach-cli
cargo test -p bach-sdk

# E2E tests (requires running node)
cargo test -p bach-e2e
```

### Benchmarks

```bash
cargo bench -p bach-crypto
cargo bench -p bach-evm
```

## License

MIT OR Apache-2.0
