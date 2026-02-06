# BachLedger 代码索引 (MEMO)

本文档作为代码库的索引，供团队成员快速定位代码位置和理解项目结构。

## 项目结构

```
rust/
├── crates/                      # 核心 crate
│   ├── bach-primitives/         # 基础类型
│   ├── bach-crypto/             # 密码学
│   ├── bach-types/              # 交易/区块类型
│   ├── bach-rlp/                # RLP 编码
│   ├── bach-storage/            # 存储层
│   ├── bach-evm/                # EVM 执行
│   ├── bach-scheduler/          # 调度器
│   ├── bach-consensus/          # 共识
│   ├── bach-network/            # 网络
│   ├── bach-txpool/             # 交易池
│   ├── bach-metrics/            # 指标
│   ├── bach-core/               # 核心集成
│   ├── bach-rpc/                # JSON-RPC
│   ├── bach-node/               # 节点
│   ├── bach-sdk/                # SDK
│   ├── bach-cli/                # CLI
│   └── bach-e2e/                # E2E 测试
├── contracts/                    # Solidity 合约 (新建)
├── examples/                     # 示例代码
└── scripts/                      # 脚本
```

## 关键文件索引

### 密码学相关
- `crates/bach-crypto/src/signature.rs` - ECDSA 签名、地址派生
- `crates/bach-crypto/src/hash.rs` - keccak256 哈希
- `crates/bach-cli/src/commands/keygen.rs` - 密钥生成工具

### 交易相关
- `crates/bach-types/src/transaction.rs` - 交易类型定义 (LegacyTx, SignedTransaction)
- `crates/bach-rlp/src/lib.rs` - RLP 编码/解码

### RPC 相关
- `crates/bach-rpc/src/methods/eth.rs` - eth_* RPC 方法
- `crates/bach-rpc/src/server.rs` - RPC 服务器
- `crates/bach-rpc/DESIGN.md` - RPC 设计文档

### SDK 相关
- `crates/bach-sdk/src/client.rs` - 客户端实现
- `crates/bach-sdk/src/wallet.rs` - 钱包实现

### CLI 相关
- `crates/bach-cli/src/commands/tx.rs` - 交易命令
- `crates/bach-cli/src/commands/query.rs` - 查询命令
- `crates/bach-cli/src/commands/keygen.rs` - 密钥生成

### 测试相关
- `examples/manual_test.rs` - 手动 E2E 测试
- `scripts/run_manual_test.sh` - 测试运行脚本

## 测试账户

| 账户 | 地址 | 私钥 |
|------|------|------|
| #0 | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | `0xac0974bec...` |
| #1 | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `0x59c6995e9...` |

## 当前任务

### 资产交易合约 (进行中)
- 状态: 设计阶段
- 位置: `contracts/` (待创建)
- 功能: mint, burn, transfer
- 设计文档: `contracts/DESIGN.md` (待创建)

## 更新日志

- 2024-02-06: 创建 MEMO 文档
- 2024-02-06: 完成 keygen 密钥生成工具
- 2024-02-06: 完成 RPC 服务器和 Node 实现
