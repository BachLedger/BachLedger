# BachLedger Rust 重新实现计划

## 1. 项目概述

### 1.1 背景

BachLedger 是一个高性能区块链系统，其核心创新在于 **Seamless Scheduling（无缝调度）** 算法，通过动态依赖检测和跨块交易调度，充分利用并行计算资源，解决传统区块链系统中块间同步导致的线程空闲时间问题。

原有实现基于长安链 (ChainMaker) 进行二次开发，受限于原有架构的设计约束。本项目旨在使用 Rust 从零开始重新实现 BachLedger 的核心理念，原生采用 **OEV (Ordering-Execution-Validation)** 架构，不做任何妥协。

### 1.2 设计目标

| 目标 | 描述 |
|------|------|
| **原生 OEV 架构** | 从第一性原理出发，原生实现 Ordering-Execution-Validation 流水线 |
| **Seamless Scheduling** | 核心算法原生实现，无缝跨块交易调度 |
| **最小外部依赖** | 尽可能使用 Rust 标准库和自研组件 |
| **EVM 兼容** | 完整支持 EVM/Solidity 智能合约执行 |
| **高性能** | 充分利用 Rust 的零成本抽象和内存安全特性 |
| **模块化** | Monorepo 架构，各组件可独立开发、测试、复用 |

### 1.3 核心创新点回顾

根据论文，BachLedger 的核心创新包括：

1. **Seamless Scheduling**: 利用块内同步等待期间的空闲线程，提前执行后续块的交易
2. **Ownership Table**: 维护存储键的当前所有者，用于高效冲突检测
3. **Priority Code**: 语义前缀序列号，包含释放状态位 + 区块高度 + 哈希派生值

---

## 2. 架构设计

### 2.1 OEV 流水线架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BachLedger OEV Pipeline                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐         │
│   │ TX Pool  │───▶│ Ordering │───▶│Execution │───▶│Validation│───▶ Storage
│   │          │    │ (TBFT)   │    │(Seamless)│    │          │         │
│   └──────────┘    └──────────┘    └──────────┘    └──────────┘         │
│        │                               │                                │
│        │         Pipeline Parallelism  │                                │
│        │    ◄─────────────────────────►│                                │
│        │                               │                                │
│   Block N    Block N+1    Block N+2    │                                │
│   ════════   ════════     ════════     │                                │
│   [tx,tx]    [tx,tx]      [tx,tx]  ───►│ Unified Scheduling Queue       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Seamless Scheduling 核心流程

```
┌─────────────────────────────────────────────────────────────────┐
│                    Seamless Scheduling Flow                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Optimistic Execution (并行)                                  │
│     ┌────────────────────────────────────────────┐              │
│     │  foreach tx in block (parallel):           │              │
│     │    priority = (0, height, hash(tx, block)) │              │
│     │    (rset, wset) = execute(tx, snapshot)    │              │
│     │    foreach w in wset:                      │              │
│     │      ownership_table[w].try_set(priority)  │              │
│     └────────────────────────────────────────────┘              │
│                           │                                      │
│                           ▼                                      │
│  2. Conflict Detection (并行)                                    │
│     ┌────────────────────────────────────────────┐              │
│     │  foreach tx in executed_queue (parallel):  │              │
│     │    if !check_ownership(tx.rset, tx.wset):  │              │
│     │      abort_queue.push(tx)                  │              │
│     │    else:                                   │              │
│     │      success_queue.push(tx)                │              │
│     └────────────────────────────────────────────┘              │
│                           │                                      │
│                           ▼                                      │
│  3. Re-execution (循环直到 abort_queue 为空)                     │
│     ┌────────────────────────────────────────────┐              │
│     │  while abort_queue.not_empty():            │              │
│     │    re_execute_parallel(abort_queue)        │              │
│     │    detect_conflict()                       │              │
│     └────────────────────────────────────────────┘              │
│                           │                                      │
│                           ▼                                      │
│  4. Release Ownership & Commit                                   │
│     ┌────────────────────────────────────────────┐              │
│     │  foreach tx in success_queue:              │              │
│     │    foreach key in tx.wset:                 │              │
│     │      ownership_table[key].release()        │              │
│     │  commit(block)                             │              │
│     └────────────────────────────────────────────┘              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Ownership Table 数据结构

```rust
/// Priority Code: 用于确定交易执行优先级
/// 格式: [release_bit (1 bit)] [block_height (63 bits)] [hash_value (256 bits)]
struct PriorityCode {
    release_bit: bool,      // 是否已释放所有权
    block_height: u64,      // 区块高度
    hash_value: [u8; 32],   // hash(tx, block_txs)
}

/// Ownership Entry: 单个存储键的所有权记录
struct OwnershipEntry {
    owner: PriorityCode,    // 当前所有者的优先级码
    lock: RwLock<()>,       // 细粒度读写锁
}

/// Ownership Table: 全局所有权表
struct OwnershipTable {
    entries: HashMap<StorageKey, OwnershipEntry>,
}
```

---

## 3. Monorepo 模块划分

```
rust/
├── Cargo.toml                 # Workspace 配置
├── docs/
│   └── PLAN.md               # 本文档
│
├── crates/
│   ├── bach-primitives/      # 基础类型与密码学原语
│   ├── bach-crypto/          # 密码学实现 (哈希、签名、默克尔树)
│   ├── bach-network/         # P2P 网络层
│   ├── bach-consensus/       # 共识算法 (TBFT)
│   ├── bach-txpool/          # 交易池
│   ├── bach-scheduler/       # Seamless Scheduling 核心
│   ├── bach-storage/         # 状态存储 (MPT, 快照)
│   ├── bach-evm/             # EVM 执行引擎
│   ├── bach-runtime/         # 合约运行时抽象
│   ├── bach-core/            # 核心流程编排
│   └── bach-node/            # 节点程序入口
│
└── tests/                    # 集成测试
```

### 3.1 模块依赖关系

```
                              bach-node
                                  │
                                  ▼
                              bach-core
                    ┌─────────────┼─────────────┐
                    │             │             │
                    ▼             ▼             ▼
             bach-consensus  bach-scheduler  bach-storage
                    │             │             │
                    ▼             ▼             ▼
              bach-network    bach-evm     bach-crypto
                    │             │             │
                    └─────────────┼─────────────┘
                                  ▼
                           bach-primitives
```

---

## 4. 模块详细设计

### 4.1 `bach-primitives` - 基础类型

**职责**: 定义系统中所有基础数据类型

```rust
// 核心类型
pub type Address = [u8; 20];
pub type Hash = [u8; 32];
pub type BlockHeight = u64;
pub type Nonce = u64;

// 交易结构
pub struct Transaction {
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub data: Vec<u8>,
    pub nonce: Nonce,
    pub gas_limit: u64,
    pub gas_price: U256,
    pub signature: Signature,
}

// 区块结构
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

pub struct BlockHeader {
    pub height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub tx_root: Hash,
    pub receipts_root: Hash,
    pub timestamp: u64,
    pub proposer: Address,
}

// 读写集
pub struct ReadSet(pub Vec<(StorageKey, Option<StorageValue>)>);
pub struct WriteSet(pub Vec<(StorageKey, StorageValue)>);
```

**外部依赖**: 无 (仅 std)

---

### 4.2 `bach-crypto` - 密码学

**职责**: 提供所有密码学原语

```rust
// 哈希函数 (自研 Keccak-256)
pub fn keccak256(data: &[u8]) -> Hash;

// 签名算法 (secp256k1 ECDSA)
pub fn sign(message: &Hash, private_key: &PrivateKey) -> Signature;
pub fn recover(message: &Hash, signature: &Signature) -> Result<PublicKey>;
pub fn verify(message: &Hash, signature: &Signature, public_key: &PublicKey) -> bool;

// Merkle Patricia Trie
pub struct MerklePatriciaTrie { ... }
impl MerklePatriciaTrie {
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    pub fn insert(&mut self, key: &[u8], value: &[u8]);
    pub fn root_hash(&self) -> Hash;
}
```

**外部依赖**: 
- 考虑自研 Keccak-256 (约 500 行代码)
- secp256k1: 可考虑使用 `k256` (纯 Rust) 或自研简化版本

---

### 4.3 `bach-network` - P2P 网络

**职责**: 节点发现、消息广播、点对点通信

```rust
pub trait NetworkService {
    // 广播消息到所有对等节点
    fn broadcast(&self, message: NetworkMessage);
    
    // 发送消息到特定节点
    fn send_to(&self, peer: PeerId, message: NetworkMessage);
    
    // 订阅特定类型消息
    fn subscribe(&self, msg_type: MessageType) -> Receiver<NetworkMessage>;
}

pub enum NetworkMessage {
    Transaction(Transaction),
    Block(Block),
    ConsensusMessage(ConsensusMessage),
    SyncRequest(SyncRequest),
    SyncResponse(SyncResponse),
}
```

**外部依赖**: 
- TCP/UDP: 使用 `std::net`
- 可选: 自研简化 libp2p 协议或使用 `libp2p` crate

---

### 4.4 `bach-consensus` - TBFT 共识

**职责**: 实现 Tendermint BFT 共识算法

```rust
pub struct TBFTConsensus {
    height: BlockHeight,
    round: u32,
    step: ConsensusStep,
    validators: ValidatorSet,
}

pub enum ConsensusStep {
    Propose,
    Prevote,
    Precommit,
    Commit,
}

impl TBFTConsensus {
    pub fn on_proposal(&mut self, proposal: Proposal) -> Vec<ConsensusAction>;
    pub fn on_prevote(&mut self, vote: Vote) -> Vec<ConsensusAction>;
    pub fn on_precommit(&mut self, vote: Vote) -> Vec<ConsensusAction>;
    pub fn on_timeout(&mut self) -> Vec<ConsensusAction>;
}
```

**外部依赖**: 无

---

### 4.5 `bach-txpool` - 交易池

**职责**: 交易收集、验证、排序

```rust
pub struct TransactionPool {
    pending: BTreeMap<Address, BTreeMap<Nonce, Transaction>>,
    queued: HashMap<Hash, Transaction>,
}

impl TransactionPool {
    pub fn add(&mut self, tx: Transaction) -> Result<()>;
    pub fn remove(&mut self, tx_hash: &Hash);
    pub fn pending_transactions(&self, limit: usize) -> Vec<Transaction>;
    pub fn notify_block(&mut self, block: &Block);
}
```

**外部依赖**: 无

---

### 4.6 `bach-scheduler` - Seamless Scheduling 核心 ⭐

**职责**: 实现论文核心算法

```rust
/// Priority Code 实现
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PriorityCode {
    bytes: [u8; 41], // 1 + 8 + 32
}

impl PriorityCode {
    pub fn new(block_height: u64, tx: &Transaction, block_txs_hash: &Hash) -> Self;
    pub fn release(&mut self);
    pub fn is_released(&self) -> bool;
}

/// Ownership Entry
pub struct OwnershipEntry {
    owner: AtomicU64,  // 压缩的 priority code 比较值
    owner_full: RwLock<PriorityCode>,
}

impl OwnershipEntry {
    pub fn try_set_owner(&self, who: &PriorityCode) -> bool;
    pub fn check_ownership(&self, who: &PriorityCode) -> bool;
    pub fn release_ownership(&self);
}

/// Ownership Table
pub struct OwnershipTable {
    entries: DashMap<StorageKey, OwnershipEntry>,
}

/// Seamless Scheduler
pub struct SeamlessScheduler {
    ownership_table: OwnershipTable,
    executor_pool: ExecutorPool,
    block_mutex: Mutex<()>,  // 块间有序互斥
}

impl SeamlessScheduler {
    /// 调度一个区块的所有交易
    pub fn schedule(&self, block: Block, snapshot: &StateSnapshot) -> ExecutedBlock;
    
    /// 乐观执行
    fn optimistic_execute(&self, txs: &[Transaction], snapshot: &StateSnapshot) 
        -> Vec<ExecutedTx>;
    
    /// 冲突检测
    fn detect_conflicts(&self, executed: &[ExecutedTx]) 
        -> (Vec<ExecutedTx>, Vec<ExecutedTx>);  // (success, abort)
    
    /// 重新执行
    fn re_execute(&self, aborted: Vec<ExecutedTx>, snapshot: &StateSnapshot) 
        -> Vec<ExecutedTx>;
}
```

**外部依赖**: 
- `dashmap`: 并发 HashMap (或自研)
- 线程池: 自研或使用 `rayon`

---

### 4.7 `bach-storage` - 状态存储

**职责**: 世界状态管理、快照、持久化

```rust
/// 状态数据库抽象
pub trait StateDB {
    fn get(&self, key: &StorageKey) -> Option<StorageValue>;
    fn set(&mut self, key: StorageKey, value: StorageValue);
    fn delete(&mut self, key: &StorageKey);
    fn commit(&mut self) -> Hash;  // 返回 state root
}

/// 快照 (用于执行时读取)
pub struct StateSnapshot {
    trie: MerklePatriciaTrie,
    cache: HashMap<StorageKey, Option<StorageValue>>,
}

/// 持久化存储
pub struct PersistentStorage {
    db: RocksDB,  // 或自研 LSM-Tree
}
```

**外部依赖**: 
- 考虑 `rocksdb` 或自研简化 KV 存储

---

### 4.8 `bach-evm` - EVM 执行引擎 ⭐

**职责**: 完整 EVM 实现，支持 Solidity 合约

```rust
/// EVM 解释器
pub struct EVM {
    state: Box<dyn StateDB>,
    context: ExecutionContext,
}

/// 执行上下文
pub struct ExecutionContext {
    pub origin: Address,
    pub gas_price: U256,
    pub block_number: u64,
    pub timestamp: u64,
    pub coinbase: Address,
    pub gas_limit: u64,
    pub chain_id: u64,
}

/// EVM 栈
pub struct Stack {
    data: Vec<U256>,
}

/// EVM 内存
pub struct Memory {
    data: Vec<u8>,
}

impl EVM {
    pub fn execute(&mut self, tx: &Transaction) -> ExecutionResult;
    
    // 操作码实现
    fn execute_opcode(&mut self, opcode: Opcode) -> Result<()>;
}

/// 所有 EVM 操作码
pub enum Opcode {
    // Stack operations
    PUSH1, PUSH2, ..., PUSH32,
    POP, DUP1, ..., DUP16, SWAP1, ..., SWAP16,
    
    // Arithmetic
    ADD, MUL, SUB, DIV, MOD, EXP, ...
    
    // Comparison & Bitwise
    LT, GT, EQ, ISZERO, AND, OR, XOR, NOT, ...
    
    // Memory & Storage
    MLOAD, MSTORE, SLOAD, SSTORE,
    
    // Control flow
    JUMP, JUMPI, PC, JUMPDEST,
    
    // System
    CALL, DELEGATECALL, STATICCALL, CREATE, CREATE2,
    RETURN, REVERT, SELFDESTRUCT,
    
    // Environment
    ADDRESS, BALANCE, ORIGIN, CALLER, CALLVALUE, ...
}
```

**外部依赖**: 
- U256 大整数: 自研或使用 `primitive-types`
- 预编译合约: 自研或使用成熟库

---

### 4.9 `bach-runtime` - 合约运行时

**职责**: 抽象合约执行接口，支持多种 VM

```rust
pub trait ContractRuntime {
    fn execute(
        &self,
        tx: &Transaction,
        state: &mut dyn StateDB,
        context: &ExecutionContext,
    ) -> ExecutionResult;
}

pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub output: Vec<u8>,
    pub logs: Vec<Log>,
    pub read_set: ReadSet,
    pub write_set: WriteSet,
}
```

---

### 4.10 `bach-core` - 核心流程编排

**职责**: 整合各模块，实现完整区块链流程

```rust
pub struct BachLedger {
    network: Box<dyn NetworkService>,
    consensus: TBFTConsensus,
    txpool: TransactionPool,
    scheduler: SeamlessScheduler,
    storage: PersistentStorage,
    evm: EVM,
}

impl BachLedger {
    pub fn new(config: NodeConfig) -> Self;
    pub fn start(&mut self);
    
    // OEV 流水线
    fn ordering_phase(&mut self) -> Block;
    fn execution_phase(&mut self, block: Block) -> ExecutedBlock;
    fn validation_phase(&mut self, block: ExecutedBlock) -> ValidatedBlock;
    fn storage_phase(&mut self, block: ValidatedBlock);
}
```

---

### 4.11 `bach-node` - 节点入口

**职责**: 命令行接口、配置加载、节点启动

```rust
fn main() {
    let config = load_config();
    let mut node = BachLedger::new(config);
    node.start();
}
```

---

## 5. 外部依赖策略

### 5.1 最小依赖原则

| 功能 | 自研方案 | 备选依赖 | 优先级 |
|------|----------|----------|--------|
| 哈希 (Keccak-256) | 自研 (~500 LOC) | `sha3` | 自研 |
| 椭圆曲线 (secp256k1) | 复杂，建议使用库 | `k256` | 使用库 |
| 大整数 (U256) | 自研 (~1000 LOC) | `primitive-types` | 可自研 |
| 并发 HashMap | 自研 | `dashmap` | 可自研 |
| 线程池 | `std::thread` | `rayon` | 自研 |
| 序列化 | 自研 RLP | `rlp` | 自研 |
| KV 存储 | 自研简化版 | `rocksdb` | 视需求 |
| 网络 | `std::net` | `libp2p` | 自研基础版 |

### 5.2 推荐最终依赖列表

```toml
[workspace.dependencies]
# 密码学 (难以自研，安全敏感)
k256 = "0.13"           # secp256k1 ECDSA

# 可选: 如果需要快速原型
# rocksdb = "0.21"      # 持久化存储
# rayon = "1.8"         # 并行迭代器
```

---

## 6. 实现路线图

### Phase 1: 基础设施 (Week 1-2)

- [ ] 初始化 Cargo workspace
- [ ] `bach-primitives`: 所有基础类型定义
- [ ] `bach-crypto`: Keccak-256, 基础签名验证
- [ ] 单元测试框架搭建

### Phase 2: 核心算法 (Week 3-4)

- [ ] `bach-scheduler`: 完整 Seamless Scheduling 实现
  - [ ] PriorityCode
  - [ ] OwnershipEntry
  - [ ] OwnershipTable
  - [ ] SeamlessScheduler
- [ ] 算法正确性测试
- [ ] 性能基准测试

### Phase 3: EVM 执行引擎 (Week 5-7)

- [ ] `bach-evm`: EVM 解释器
  - [ ] 栈、内存、存储操作
  - [ ] 所有操作码实现
  - [ ] Gas 计算
  - [ ] 预编译合约
- [ ] 使用 Ethereum 测试用例验证
- [ ] `bach-storage`: MPT 实现、状态快照

### Phase 4: 共识与网络 (Week 8-9)

- [ ] `bach-consensus`: TBFT 实现
- [ ] `bach-network`: 基础 P2P 网络
- [ ] `bach-txpool`: 交易池

### Phase 5: 集成与测试 (Week 10-11)

- [ ] `bach-core`: 流程编排
- [ ] `bach-node`: 节点程序
- [ ] 端到端测试
- [ ] 多节点网络测试

### Phase 6: 优化与完善 (Week 12+)

- [ ] 性能优化
- [ ] 文档完善
- [ ] 安全审计
- [ ] 基准测试对比 (vs ChainMaker, FISCO-BCOS)

---

## 7. 性能目标

基于论文实验结果，新实现应达到：

| 指标 | 目标值 | 说明 |
|------|--------|------|
| TPS (64 线程) | > 5000 | ERC-20 转账场景 |
| 延迟 | < 500ms | 交易确认时间 |
| 资源利用率 | > 80% | CPU 利用率 |
| 空闲时间占比 | < 10% | Seamless Scheduling 核心指标 |

---

## 8. 测试策略

### 8.1 单元测试

每个 crate 独立测试：
- `bach-crypto`: 哈希、签名、MPT 正确性
- `bach-scheduler`: 调度算法正确性、并发安全
- `bach-evm`: 操作码正确性 (使用 Ethereum 测试向量)

### 8.2 集成测试

- 单节点完整流程
- 多节点共识
- 高并发压力测试
- 冲突交易场景测试

### 8.3 基准测试

- 与原 Go 实现对比
- 与 ChainMaker、FISCO-BCOS 对比
- 不同线程数下的扩展性测试

---

## 9. 文档要求

- [ ] README.md: 项目介绍、快速开始
- [ ] ARCHITECTURE.md: 架构设计详解
- [ ] API.md: 公开 API 文档
- [ ] CONTRIBUTING.md: 贡献指南
- [ ] 每个 crate 的 rustdoc 注释

---

## 10. 开发规范

### 10.1 代码风格

- 使用 `rustfmt` 格式化
- 使用 `clippy` 静态检查
- 公开 API 必须有文档注释
- 错误处理使用 `Result<T, E>`

### 10.2 Git 规范

- 主分支: `main`
- 功能分支: `feat/xxx`
- 修复分支: `fix/xxx`
- Commit message: Conventional Commits

### 10.3 CI/CD

- 所有 PR 必须通过测试
- 代码覆盖率 > 80%
- 无 clippy 警告

---

## 附录 A: 参考资料

1. BachLedger 论文 (IEEE ICPADS 2024)
2. Ethereum Yellow Paper
3. Tendermint BFT 论文
4. Block-STM 论文
5. Rust Async Book

## 附录 B: 名词解释

| 术语 | 解释 |
|------|------|
| OEV | Ordering-Execution-Validation，先排序后执行的架构 |
| EOV | Execution-Ordering-Validation，先执行后排序的架构 |
| Seamless Scheduling | 无缝调度，BachLedger 核心算法 |
| Ownership Table | 所有权表，记录存储键的当前所有者 |
| Priority Code | 优先级码，确定交易执行顺序 |
| TBFT | Tendermint BFT，拜占庭容错共识算法 |
| MPT | Merkle Patricia Trie，以太坊状态树 |
| RLP | Recursive Length Prefix，以太坊序列化编码 |
