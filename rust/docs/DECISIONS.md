# BachLedger Rust - 技术决策记录

> 最后更新: 2026-02-06
> 状态: 已确认

---

## 决策汇总

| # | 领域 | 决策项 | 选择 | 确认日期 |
|---|------|--------|------|----------|
| D01 | 密码学 | 签名算法 | k256 (纯 Rust) | 2026-02-06 |
| D02 | 密码学 | 哈希函数 | sha3 crate | 2026-02-06 |
| D03 | 数据类型 | 大整数 U256 | primitive-types | 2026-02-06 |
| D04 | 存储 | KV 存储 | RocksDB | 2026-02-06 |
| D05 | 可观测性 | 指标导出 | JSON + CLI 仪表盘 | 2026-02-06 |
| D06 | 并发 | 异步运行时 | tokio | 2026-02-06 |
| D07 | 序列化 | 编码格式 | RLP (以太坊兼容) | 2026-02-06 |
| D08 | 交易 | 交易格式 | 以太坊兼容 (EIP-155) | 2026-02-06 |
| D09 | 日志 | 日志框架 | tracing | 2026-02-06 |
| D10 | 错误 | 错误处理 | thiserror + anyhow | 2026-02-06 |
| D11 | EVM | 预编译合约 | MVP: 0x01-0x04 | 2026-02-06 |
| D12 | EVM | EVM 版本 | Shanghai | 2026-02-06 |

---

## 依赖清单

```toml
[workspace.dependencies]
# 密码学
k256 = "0.13"
sha3 = "0.10"
sha2 = "0.10"          # SHA256 预编译

# 数据类型
primitive-types = "0.12"

# 存储
rocksdb = "0.22"

# 序列化
rlp = "0.5"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 错误处理
thiserror = "1"
anyhow = "1"

# 开发/测试
proptest = "1"
criterion = "0.5"
```

---

## 架构决策

### OEV 流水线
```
TxPool → Ordering(TBFT) → Execution(Seamless) → Validation → Storage
```

### 模块结构
```
crates/
├── bach-primitives/   # 基础类型
├── bach-crypto/       # 密码学
├── bach-types/        # 交易、区块结构
├── bach-rlp/          # RLP 编解码
├── bach-storage/      # RocksDB 封装
├── bach-evm/          # EVM 执行引擎
├── bach-scheduler/    # Seamless Scheduling
├── bach-consensus/    # TBFT 共识
├── bach-network/      # P2P 网络
├── bach-txpool/       # 交易池
├── bach-metrics/      # 可观测性
├── bach-core/         # 核心流程
└── bach-node/         # 节点入口
```

---

## 性能目标

| 指标 | 目标值 |
|------|--------|
| TPS (64线程) | > 5000 |
| 区块延迟 | < 500ms |
| 线程空闲率 | < 10% |

---

## 变更历史

| 日期 | 变更 | 原因 |
|------|------|------|
| 2026-02-06 | 初始决策 | 项目启动 |
