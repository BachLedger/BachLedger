# BachLedger E2E Testing Strategy

## Executive Summary

This document defines the end-to-end testing strategy for BachLedger. The goal is **NOT** to re-test what unit tests already cover, but to verify that components work together correctly.

## 1. Testing Philosophy

### What E2E Tests Are For

E2E tests verify **integration boundaries** - the places where components meet. They answer: "When I connect A to B, does the data flow correctly?"

### What E2E Tests Are NOT For

- **Unit behavior**: Already covered by 900+ unit tests
- **EVM opcodes**: `bach-evm-tests` handles this comprehensively
- **Edge cases**: Unit tests should cover these
- **Performance**: Use benchmarks (`criterion`) instead

### Core Principles

1. **Integration-focused**: Test component boundaries, not component internals
2. **Declarative**: Tests describe WHAT should happen, not HOW
3. **Fast**: All E2E tests complete in < 30 seconds total
4. **Simple**: No complex infrastructure - just `cargo test -p bach-e2e`
5. **Deterministic**: Same inputs always produce same outputs

## 2. What to Test (and What NOT to Test)

### MUST Test (Integration Boundaries)

| Boundary | Why It's Critical | Example |
|----------|-------------------|---------|
| **Transaction -> Executor** | Entry point for all state changes | Signed tx executes and updates state |
| **Executor -> Storage** | State persistence correctness | Executed state survives restart |
| **Executor -> EVM** | Contract execution pipeline | Contract deployment and calls work |
| **Scheduler -> Executor** | Parallel execution correctness | Parallel batches produce same result as serial |
| **TxPool -> Block Builder** | Transaction ordering | Transactions sorted by gas price |

### SHOULD Test (Complex Workflows)

| Workflow | Scenarios |
|----------|-----------|
| **Contract Lifecycle** | Deploy, call, self-destruct |
| **Multi-block State** | State accumulates across blocks |
| **Transaction Chains** | Nonce sequences, dependent transactions |
| **ERC20 Pattern** | Transfer, approve, transferFrom |

### DO NOT Test (Covered Elsewhere)

| Component | Why Skip | Covered By |
|-----------|----------|------------|
| Individual opcodes | 256 opcodes already tested | `bach-evm-tests` |
| RLP encoding | Deterministic serialization | `bach-rlp` unit tests |
| Crypto primitives | Signature/hash operations | `bach-crypto` unit tests |
| Network protocol | Async/network complexity | `bach-network` integration tests |
| Consensus state machine | Protocol correctness | `bach-consensus` tests |

## 3. Test Harness API Design

### Design Goals

1. **Builder pattern** for readable test setup
2. **Immutable inputs, observable outputs** - no hidden mutation
3. **Zero external dependencies** - no network, no persistent disk

### Core Types

```rust
/// Test harness - entry point for all E2E tests
pub struct TestHarness {
    executor: BlockExecutor,
    accounts: HashMap<String, TestAccount>,
    chain_id: u64,
    block_number: u64,
}

/// A named test account with keypair
pub struct TestAccount {
    pub name: String,
    pub address: Address,
    pub private_key: [u8; 32],
    pub balance: u128,
    pub nonce: u64,
}

/// Builder for transactions
pub struct TxBuilder {
    from: String,
    to: Option<Address>,
    value: u128,
    data: Vec<u8>,
    gas_limit: u64,
    nonce: Option<u64>,
}

/// Execution result for verification
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub logs: Vec<Log>,
    pub contract_address: Option<Address>,
    pub error: Option<String>,
}
```

### API Examples

```rust
// Simple value transfer
#[test]
fn test_value_transfer() {
    let mut harness = TestHarness::new()
        .with_account("alice", 10.ether())
        .with_account("bob", 0);

    let result = harness.execute(
        TxBuilder::transfer("alice", "bob", 1.ether())
    );

    assert!(result.success);
    assert_eq!(harness.balance("alice"), 9.ether() - result.gas_cost());
    assert_eq!(harness.balance("bob"), 1.ether());
}

// Contract deployment and call
#[test]
fn test_contract_lifecycle() {
    let mut harness = TestHarness::new()
        .with_account("deployer", 10.ether());

    // Deploy
    let deploy_result = harness.execute(
        TxBuilder::deploy("deployer", COUNTER_BYTECODE)
    );
    assert!(deploy_result.success);
    let contract = deploy_result.contract_address.unwrap();

    // Call increment
    let call_result = harness.execute(
        TxBuilder::call("deployer", contract)
            .data(encode_call("increment", &[]))
    );
    assert!(call_result.success);

    // Verify state
    let count = harness.storage(contract, H256::ZERO);
    assert_eq!(count, H256::from_low_u64_be(1));
}

// Parallel execution verification
#[test]
fn test_parallel_execution_correctness() {
    let mut harness = TestHarness::new()
        .with_accounts(100, 10.ether());  // 100 funded accounts

    // Independent transfers (should parallelize)
    let txs: Vec<_> = (0..50).map(|i| {
        TxBuilder::transfer(
            &format!("account_{}", i * 2),
            &format!("account_{}", i * 2 + 1),
            1.ether()
        )
    }).collect();

    let results = harness.execute_parallel(txs);

    // Verify all succeeded
    assert!(results.iter().all(|r| r.success));

    // Verify same result as serial execution
    let serial_results = harness.execute_serial(txs);
    assert_eq!(results.final_state_root(), serial_results.final_state_root());
}
```

### Utility Traits

```rust
/// Ether denomination helper
pub trait EtherDenom {
    fn wei(self) -> u128;
    fn gwei(self) -> u128;
    fn ether(self) -> u128;
}

impl EtherDenom for u64 {
    fn wei(self) -> u128 { self as u128 }
    fn gwei(self) -> u128 { self as u128 * 1_000_000_000 }
    fn ether(self) -> u128 { self as u128 * 1_000_000_000_000_000_000 }
}
```

## 4. Key Test Scenarios (Prioritized)

### Priority 1: Critical Path (Must Pass for MVP)

#### 1.1 Basic Transaction Flow
```rust
#[test]
fn test_simple_transfer() {
    // Fund alice, transfer to bob, verify balances
}

#[test]
fn test_transfer_insufficient_balance() {
    // Attempt transfer with insufficient funds, expect failure
}

#[test]
fn test_nonce_sequence() {
    // Execute 5 transactions from same account, verify nonces
}
```

#### 1.2 Contract Deployment
```rust
#[test]
fn test_contract_deploy() {
    // Deploy simple contract, verify code stored
}

#[test]
fn test_contract_deploy_with_constructor() {
    // Deploy with constructor args, verify initialized state
}
```

#### 1.3 Contract Calls
```rust
#[test]
fn test_contract_call_pure() {
    // Call pure function, verify return value
}

#[test]
fn test_contract_call_state_change() {
    // Call function that modifies storage, verify storage changed
}
```

### Priority 2: Scheduler Integration

#### 2.1 Parallel Execution Correctness
```rust
#[test]
fn test_independent_transactions_parallelize() {
    // 10 independent transfers execute in one batch
}

#[test]
fn test_dependent_transactions_serialize() {
    // Transactions accessing same state execute serially
}

#[test]
fn test_parallel_equals_serial_result() {
    // Same transactions, parallel vs serial, same final state
}
```

#### 2.2 Conflict Detection
```rust
#[test]
fn test_read_write_conflict() {
    // tx1 writes slot X, tx2 reads slot X -> dependency detected
}

#[test]
fn test_write_write_conflict() {
    // tx1 writes slot X, tx2 writes slot X -> serialized
}
```

### Priority 3: Multi-Block Scenarios

#### 3.1 State Persistence
```rust
#[test]
fn test_state_persists_across_blocks() {
    // Block 1: deploy contract
    // Block 2: call contract
    // Verify state from block 1 visible in block 2
}
```

#### 3.2 Block Building
```rust
#[test]
fn test_block_respects_gas_limit() {
    // Submit transactions exceeding block gas limit
    // Verify correct subset included
}
```

### Priority 4: Real-World Patterns

#### 4.1 ERC20 Token
```rust
#[test]
fn test_erc20_transfer() {
    // Deploy ERC20, transfer tokens
}

#[test]
fn test_erc20_approve_transferfrom() {
    // Approve spender, spender calls transferFrom
}
```

#### 4.2 Multi-Sig Pattern
```rust
#[test]
fn test_multisig_execution() {
    // Deploy multisig, collect signatures, execute
}
```

## 5. Test Runner Approach

### Running Tests

```bash
# Run all E2E tests
cargo test -p bach-e2e

# Run with verbose output
cargo test -p bach-e2e -- --nocapture

# Run specific test
cargo test -p bach-e2e test_simple_transfer

# Run parallel tests only
cargo test -p bach-e2e parallel
```

### Test Organization

```
crates/bach-e2e/
├── Cargo.toml
├── DESIGN.md           # This document
├── src/
│   ├── lib.rs          # Public API
│   ├── harness.rs      # TestHarness implementation
│   ├── account.rs      # TestAccount, key management
│   ├── builder.rs      # TxBuilder
│   ├── contracts/      # Pre-compiled test contracts
│   │   ├── mod.rs
│   │   ├── counter.rs  # Simple counter contract bytecode
│   │   └── erc20.rs    # Minimal ERC20 bytecode
│   └── utils.rs        # EtherDenom, encoding helpers
└── tests/
    ├── basic.rs        # Priority 1 tests
    ├── scheduler.rs    # Priority 2 tests
    ├── multiblock.rs   # Priority 3 tests
    └── patterns.rs     # Priority 4 tests
```

### Test Fixtures

Pre-compiled contract bytecode stored as constants:

```rust
// src/contracts/counter.rs
/// Simple counter contract
/// function increment() public { count++; }
/// function get() public view returns (uint256) { return count; }
pub const COUNTER_BYTECODE: &[u8] = &[
    0x60, 0x80, 0x60, 0x40, /* ... */
];

pub const COUNTER_ABI: &str = r#"[
    {"name":"increment","inputs":[],"outputs":[]},
    {"name":"get","inputs":[],"outputs":[{"type":"uint256"}]}
]"#;
```

## 6. Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [x] Define testing strategy (this document)
- [ ] Implement `TestHarness` core
- [ ] Implement `TestAccount` with key generation
- [ ] Implement `TxBuilder` with signing
- [ ] Basic value transfer test passing

### Phase 2: Contract Support (Week 2)
- [ ] Add contract deployment support
- [ ] Add contract call support
- [ ] Pre-compile test contracts (Counter, ERC20)
- [ ] Contract lifecycle tests passing

### Phase 3: Scheduler Integration (Week 3)
- [ ] Integrate `bach-scheduler` with harness
- [ ] Add parallel execution mode
- [ ] Add serial vs parallel comparison
- [ ] Scheduler tests passing

### Phase 4: Polish (Week 4)
- [ ] Multi-block scenarios
- [ ] Error message improvements
- [ ] Documentation and examples
- [ ] CI integration

## 7. Success Criteria

### Quantitative
- [ ] All Priority 1 tests pass
- [ ] All Priority 2 tests pass
- [ ] Full test suite runs in < 30 seconds
- [ ] Zero flaky tests

### Qualitative
- [ ] New developer can write a test in < 5 minutes
- [ ] Test failures clearly indicate what went wrong
- [ ] No external dependencies (network, persistent storage)

## 8. Non-Goals (Explicitly Out of Scope)

1. **Fuzzing**: Use `proptest` in unit tests instead
2. **Performance benchmarks**: Use `criterion` benches
3. **Network simulation**: Too complex, test `bach-network` separately
4. **Consensus simulation**: Test `bach-consensus` with its own harness
5. **State sync**: Requires network, out of scope
6. **JSON-RPC compatibility**: Separate testing effort

## Appendix A: Test Contract Source

### Counter.sol
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Counter {
    uint256 public count;

    function increment() public {
        count++;
    }

    function get() public view returns (uint256) {
        return count;
    }
}
```

### MinimalERC20.sol
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract MinimalERC20 {
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    constructor(uint256 initialSupply) {
        balanceOf[msg.sender] = initialSupply;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        return true;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}
```

## Appendix B: Harness Implementation Skeleton

```rust
// src/harness.rs
use bach_core::{BlockExecutor, ExecutionState};
use bach_crypto::{sign_message, SecretKey};
use bach_primitives::{Address, H256};
use bach_types::{Block, BlockBody, BlockHeader, SignedTransaction};
use std::collections::HashMap;

pub struct TestHarness {
    executor: BlockExecutor,
    accounts: HashMap<String, TestAccount>,
    chain_id: u64,
    block_number: u64,
    base_fee: u128,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            executor: BlockExecutor::new(1), // chain_id = 1
            accounts: HashMap::new(),
            chain_id: 1,
            block_number: 0,
            base_fee: 1_000_000_000, // 1 gwei
        }
    }

    pub fn with_account(mut self, name: &str, balance: u128) -> Self {
        let account = TestAccount::generate(name, balance);
        // Fund account in executor state
        self.executor.state_mut().set_account(
            account.address,
            bach_storage::Account {
                balance,
                nonce: 0,
                ..Default::default()
            }
        );
        self.accounts.insert(name.to_string(), account);
        self
    }

    pub fn execute(&mut self, builder: TxBuilder) -> ExecutionResult {
        let tx = self.build_and_sign(builder);
        let block = self.create_block(vec![tx]);

        match self.executor.execute_block(&block) {
            Ok(result) => {
                self.block_number += 1;
                ExecutionResult::from_receipt(&result.receipts[0])
            }
            Err(e) => ExecutionResult::error(e.to_string())
        }
    }

    pub fn balance(&self, name: &str) -> u128 {
        let addr = &self.accounts[name].address;
        self.executor.state()
            .get_account(addr)
            .map(|a| a.balance)
            .unwrap_or(0)
    }

    pub fn storage(&self, address: Address, slot: H256) -> H256 {
        self.executor.state().get_storage(&address, &slot)
    }

    // Private helpers...
}
```
