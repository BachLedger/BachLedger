# 使用 AI 复现论文系统：BachLedger 实践指南

本文档介绍如何使用 Claude Code 根据学术论文复现完整的区块链系统，重点介绍 **trial-2** 中基于 ICDD Skill 的多 Agent 协作方法。

---

## 1. 项目背景

### 1.1 论文来源

**BachLedger: Orchestrating Parallel Execution with Dynamic Dependency Detection and Seamless Scheduling**

### 1.2 复现目标

基于论文描述，用 Rust 从零实现一个完整的医疗区块链系统：

| 模块 | 功能 |
|------|------|
| bach-primitives | 基础类型 (Address, H256, U256) |
| bach-crypto | 密码学操作 (keccak256, secp256k1) |
| bach-types | 区块链类型 (Block, Transaction) |
| bach-state | 状态管理 |
| bach-scheduler | **Seamless Scheduling 核心算法** |
| bach-evm | EVM 解释器 |
| bach-consensus | TBFT 共识 (n > 3f+1) |
| bach-network | P2P 网络层 |
| bach-storage | 持久化存储 |
| bach-rpc | JSON-RPC 接口 |
| bach-node | 全节点二进制 |
| bach-contracts | 医疗合约模板 |

---

## 2. 两次尝试对比

### 2.1 Trial-1: 传统对话式开发

**方法**: 直接与 Claude 对话，逐模块实现

**问题**:
- 上下文窗口限制导致前后不一致
- 接口定义随实现漂移
- 测试覆盖不完整
- 代码质量依赖人工审查

### 2.2 Trial-2: ICDD Skill 驱动的多 Agent 协作

**方法**: 定义标准化的 Skill，通过角色隔离和流程约束确保质量

**优势**:
- 接口先行，锁定后不可修改
- 测试驱动开发 (TDD)
- 多角色交叉审查
- 自动化验证脚本
- 知识库持久化

---

## 3. ICDD Skill 设计

### 3.1 核心理念

**Interface-Contract-Driven Development (接口契约驱动开发)**

```
需求 → 接口定义 → 测试编写 → 代码实现 → 多角度审查 → 渗透测试
```

### 3.2 Agent 角色定义

| Agent | 职责 | 可见范围 | 约束 |
|-------|------|----------|------|
| **Architect** | 需求分析、接口设计 | 全部 | 接口锁定后不可修改 |
| **Tester** | 编写测试 (TDD Red) | 仅接口 | **禁止查看实现** |
| **Coder** | 实现代码 (TDD Green) | 接口 + 测试 | **禁止修改测试** |
| **Reviewer-Logic** | 审查实现逻辑 | 仅实现 | 无法看到测试 |
| **Reviewer-Test** | 审查测试质量 | 仅测试 | 无法看到实现 |
| **Reviewer-Integration** | 跨模块检查 | 全部 | - |
| **Attacker** | 渗透测试 | 全部 + 运行时 | 必须提供 PoC |
| **Documenter** | 知识库管理 | 全部 | - |

### 3.3 隔离规则 (关键!)

```
┌─────────────────────────────────────────────────────────┐
│  Tester 看不到 Implementation  →  测试基于契约而非实现    │
│  Coder 不能改 Tests           →  代码适配测试而非反过来  │
│  Reviewer 交叉盲审            →  避免确认偏误            │
│  接口锁定后不可变             →  防止实现反推接口        │
└─────────────────────────────────────────────────────────┘
```

### 3.4 工作流程

```
User: "实现 bach-crypto 模块"
         │
         ▼
┌────────────────────────────────────────┐
│ Step 1: Derive Requirements            │
│ Architect 分析论文，输出 requirements.md│
└────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────┐
│ Step 2: Lock Interfaces                │
│ Architect 定义 trait/API，锁定不可变    │
└────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────┐
│ Step 3a: TDD Red                       │
│ Tester 只看接口，编写失败的测试         │
│ → check_test_quality.sh 验证           │
└────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────┐
│ Step 3b: TDD Green                     │
│ Coder 实现代码使测试通过                │
│ → check_stub_detection.sh 检测占位实现 │
│ → check_interface_drift.sh 检测接口漂移│
└────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────┐
│ Step 3c-e: Parallel Reviews            │
│ Logic / Test / Integration 三路审查    │
└────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────┐
│ Step 4: Attack & Review                │
│ Attacker 渗透测试 → Reviewer 验证攻击   │
└────────────────────────────────────────┘
         │
         ▼
    模块完成 ✓
```

---

## 4. Skill 文件结构

```
.claude/skills/icdd/
├── SKILL.md                    # Skill 入口和总览
├── assets/                     # 模板文件 (填充后提交)
│   ├── requirements.md         #   需求图 + 风险登记
│   ├── interface-contract.md   #   接口契约定义
│   ├── review-checklist.md     #   审查清单
│   ├── attack-report.md        #   渗透测试报告
│   └── agent-handoff.md        #   Agent 交接文档
├── references/                 # Agent 提示词
│   ├── step1-derive-requirements.md
│   ├── step2-lock-interfaces.md
│   ├── step3-tester.md
│   ├── step3-coder.md
│   ├── step3-reviewer-*.md
│   ├── step4-attacker.md
│   └── documenter.md
├── scripts/
│   ├── validators/             # 自动化验证脚本
│   │   ├── check_test_quality.sh
│   │   ├── check_stub_detection.sh
│   │   ├── check_interface_drift.sh
│   │   └── check_trait_compliance.sh
│   ├── knowledge/              # 知识库管理
│   │   ├── init_kb.sh
│   │   └── trigger_documenter.sh
│   └── worktree/               # 并行开发支持
│       ├── create_worktrees.sh
│       └── merge_worktrees.sh
```

---

## 5. 使用示例

### 5.1 初始化知识库

```bash
cd .claude/skills/icdd
./scripts/knowledge/init_kb.sh
```

### 5.2 启动模块开发

```bash
# 在 Claude Code 中调用 ICDD skill
/icdd

# 或直接加载特定步骤的提示词
# 提供: references/step1-derive-requirements.md + 论文相关章节
```

### 5.3 验证器使用

```bash
# 检测测试是否为假测试 (assert true, 空 body 等)
./scripts/validators/check_test_quality.sh rust/bach-crypto/tests/

# 检测占位实现 (todo!, unimplemented!, panic!)
./scripts/validators/check_stub_detection.sh rust/bach-crypto/src/

# 检测接口漂移
./scripts/validators/check_interface_drift.sh rust/bach-crypto/src/lib.rs
```

---

## 6. 实践效果

### 6.1 代码规模

| 指标 | 数值 |
|------|------|
| Rust 模块 | 12 个 |
| 代码行数 | ~15,000 行 |
| 测试用例 | 800+ 个 |
| 测试覆盖率 | ~85% |

### 6.2 功能验证

- ✅ 单节点本地运行
- ✅ 4 节点 Docker 网络 (TBFT 共识)
- ✅ JSON-RPC 接口 (Ethereum 兼容)
- ✅ Solidity 合约编译、部署、调用
- ✅ 状态变更验证

### 6.3 Docker 镜像

```bash
docker pull youngyee/bachledger-node:latest

# 启动 4 节点网络
cd deployment
./setup.sh
docker compose up -d
```

---

## 7. 关键经验

### 7.1 为什么需要角色隔离？

| 问题 | 没有隔离时 | 有隔离后 |
|------|-----------|---------|
| 测试质量 | 测试可能为实现"量身定做" | 测试基于契约，真正验证行为 |
| 接口稳定性 | 实现困难时修改接口 | 接口锁定，实现必须适配 |
| 代码审查 | 审查者已知实现细节，产生偏见 | 盲审发现更多问题 |

### 7.2 验证脚本的作用

- **check_stub_detection.sh**: 防止 `todo!()` 占位符进入主分支
- **check_interface_drift.sh**: 检测 trait 签名是否被悄悄修改
- **check_test_quality.sh**: 检测 `assert!(true)` 等假测试

### 7.3 知识库的价值

- 跨模块的设计决策记录
- Agent 之间的上下文传递
- 问题追踪和解决方案积累

---

## 8. 快速复现步骤

```bash
# 1. 克隆仓库
git clone https://github.com/BachLedger/BachLedger.git
cd BachLedger

# 2. 切换到 trial-2 分支 (包含 ICDD skill)
git checkout trial-2

# 3. 查看 ICDD skill
cat .claude/skills/icdd/SKILL.md

# 4. 编译运行
cd rust
cargo build --release
cargo test

# 5. 启动节点
./target/release/bach-node --data-dir ./data --rpc run
```

---

## 9. 总结

Trial-2 的核心创新是将多 Agent 协作规则 **形式化为可复用的 Skill**：

1. **角色隔离** - 每个 Agent 只能看到其职责范围内的内容
2. **流程约束** - 通过验证脚本强制执行质量门禁
3. **接口先行** - 锁定接口后实现，防止实现反推接口
4. **知识传承** - 知识库记录决策，支持长周期开发

这种方法论不仅适用于论文复现，也适用于任何需要高质量、可维护代码的项目。

---

*文档版本: 1.0 | 更新日期: 2026-02-09*
