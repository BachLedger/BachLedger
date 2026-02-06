# BachLedger Git Workflow

## Branch Strategy

```
main                    # 稳定版本，所有测试必须通过
├── develop             # 开发主分支，集成测试通过即可合并
│   ├── feature/xxx     # 功能分支
│   ├── fix/xxx         # 修复分支
│   └── refactor/xxx    # 重构分支
└── release/v0.x.x      # 发布分支
```

## 分支命名规范

- `feature/module-name` - 新功能，如 `feature/scheduler-seamless`
- `fix/issue-description` - Bug 修复，如 `fix/gas-price-underflow`
- `refactor/module-name` - 重构，如 `refactor/transaction-types`
- `release/v0.1.0` - 发布分支

## 团队并行开发流程

### 1. Coder 工作流
```bash
# 创建功能分支
git checkout develop
git pull origin develop
git checkout -b feature/module-name

# 开发并提交（细粒度 commit）
git add specific-files
git commit -m "feat(module): description"

# 完成后推送
git push -u origin feature/module-name
```

### 2. Tester 工作流
```bash
# 在 coder 分支上添加测试
git checkout feature/module-name
git pull origin feature/module-name

# 添加测试
git add tests/
git commit -m "test(module): add unit tests"
git push
```

### 3. Reviewer 工作流
```bash
# Review 完成后，合并到 develop
git checkout develop
git merge --no-ff feature/module-name -m "Merge feature/module-name (#PR)"
git push origin develop
```

## Commit Message 规范

格式：`<type>(<scope>): <description>`

Types:
- `feat` - 新功能
- `fix` - Bug 修复
- `refactor` - 重构（不改变行为）
- `test` - 测试
- `docs` - 文档
- `chore` - 构建/工具

示例：
```
feat(scheduler): implement seamless scheduling algorithm
fix(crypto): prevent EIP-2 signature malleability
test(types): add transaction edge case tests
```

## 版本回退

### 回退单个 commit
```bash
git revert <commit-hash>
```

### 回退到特定版本
```bash
git checkout <commit-hash> -- path/to/file
```

### 重置分支（谨慎使用）
```bash
git reset --hard <commit-hash>  # 丢弃所有更改
git reset --soft <commit-hash>  # 保留更改在暂存区
```

## Tag 管理

### 里程碑打 Tag
```bash
git tag -a v0.1.0-alpha -m "Phase 1 complete: primitives, crypto, types"
git push origin v0.1.0-alpha
```

### 当前里程碑
- `v0.1.0-alpha` - 基础类型和密码学完成
- `v0.2.0-alpha` - Scheduler 和 RLP 完成（进行中）
- `v0.3.0-alpha` - EVM 基础完成
- `v0.4.0-alpha` - 共识和网络完成
- `v1.0.0-beta` - 完整功能，待集成测试
