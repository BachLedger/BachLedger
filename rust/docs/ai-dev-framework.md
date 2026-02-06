# AI 辅助大规模开发框架设计

> 基于 Kimi Agent Golang SDK 实现的「组长-员工」协作模式

## 1. 核心问题与设计理念

### 1.1 我们要解决什么问题？

Anthropic 的 C 编译器项目证明了 agent teams 的可行性，但其方案有以下局限：

| Anthropic 方案 | 局限性 | 我们的改进方向 |
|---------------|--------|---------------|
| 简单的文件锁机制 | 无法表达复杂的任务依赖 | **项目流程 DAG** |
| Claude 自行决定下一步 | 缺乏全局视角，容易偏离 | **人类作为组长把控方向** |
| 测试驱动验证 | 仅靠测试无法保证需求完整性 | **Coder/Tester/Critic 各自独立、批判** |
| 无通信机制 | Agent 间缺乏共享上下文 | **Memo 公共大脑（非直接通信）** |
| 乐观并行 | 频繁的 merge conflict | **尽量避免冲突 + 冲突解决作为 fallback** |

### 1.2 核心设计理念

```
┌─────────────────────────────────────────────────────────────────────┐
│                      人类作为「组长」的职责                           │
├─────────────────────────────────────────────────────────────────────┤
│  ✓ 需求的提出与澄清                                                  │
│  ✓ 关键决断（技术选型、架构决策、风险评估）                            │
│  ✓ 资源授权（密钥、服务器、账号）                                      │
│  ✓ 验收标准的定义                                                    │
│  ✗ 不需要：事无巨细地指导每一步                                       │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                      Agent 作为「员工」的职责                         │
├─────────────────────────────────────────────────────────────────────┤
│  ✓ 需求分析与细化（生成待确认的问题清单）                              │
│  ✓ 任务分解与 DAG 构建                                               │
│  ✓ 代码实现与自测                                                    │
│  ✓ 主动暴露不确定性（而非自圆其说）                                    │
│  ✓ 结果的可验证报告                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.3 解决「自圆其说」的核心策略

你最担心的问题：**智能体仅仅为了自圆其说报告任务成功，没有机制保证需求确实被完整实现。**

解决方案：**每个角色都独立、批判，不为其他角色的工作「圆场」。**

```
┌─────────────────────────────────────────────────────────────────────┐
│                      独立批判原则                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Coder:  实现代码，但也要批判性地审视需求是否合理              │
│           发现问题要主动暴露，而非糊弄过去                       │
│                                                                      │
│   Tester: 编写测试，目标是「找到 bug」而非「证明没 bug」           │
│           不要为了让测试通过而降低覆盖率或跳过边界情况           │
│                                                                      │
│   Critic: 审查代码，目标是「发现问题」而非「帮助通过」             │
│           不要为了让任务完成而降低标准                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**验证流程：**

```
Coder 完成 → Tester 测试 → Critic 审查 → 都通过才完成
                                    ↓
                            任何一个拒绝 → 返回修复
```

**为什么这样设计？**
- 每个角色都有批判性思维，不是只有 Critic 一个“看门人”
- 避免「走过场」式的测试和审查
- 人类在关键节点介入即可，不需要层层审批

### 1.4 资源与决策前置原则

**核心理念：在规划阶段就识别并确认所有资源需求和关键决策，而非执行时才发现。**

这不仅是避免犯错，更是 **用户体验** 问题：

```
❌ 糟糕的体验：
   用户: "帮我实现 X 功能"
   Agent: [执行 30 分钟]
   Agent: "我需要数据库密码"
   用户: [提供]
   Agent: [执行 20 分钟]
   Agent: "我需要决定用 MySQL 还是 PostgreSQL"
   用户: [崩溃]

✅ 好的体验：
   用户: "帮我实现 X 功能"
   Agent: [分析 2 分钟]
   Agent: "我分析了这个任务，需要确认：
          1. 数据库选型：MySQL 还是 PostgreSQL？（建议 PostgreSQL，因为...）
          2. 需要以下资源：数据库密码、S3 bucket 访问权限
          3. 预计耗时 2 小时，分解为 5 个子任务
          确认后我开始执行。"
   用户: [一次性确认]
   Agent: [执行，中间不再打扰]
```

---

## 2. 系统架构

### 2.1 核心理念：简单就是好

**不需要复杂的多组件架构，本质上就是：**
- 一个精心调整的系统提示词
- 一个 CLI 交互界面
- 一个问答循环

### 2.2 整体流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI 交互流程                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   用户输入需求                                                       │
│        │                                                             │
│        ▼                                                             │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  Planner（精心调整的系统提示词）                              │   │
│   │                                                              │   │
│   │  • 分析需求，识别模糊点                                       │   │
│   │  • 识别所需资源（密钥、权限等）                               │   │
│   │  • 识别关键决策点                                             │   │
│   │  • 分析潜在风险                                               │   │
│   │  • 生成任务拆解                                               │   │
│   └──────────────────────────┬──────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│                    ┌──────────────────┐                             │
│                    │  有问题要确认？   │                             │
│                    └────────┬─────────┘                             │
│                             │                                        │
│              ┌──────────────┴──────────────┐                        │
│              │ Yes                         │ No                     │
│              ▼                             ▼                        │
│   ┌─────────────────────┐      ┌─────────────────────┐             │
│   │  向用户提问          │      │  开始执行            │             │
│   │  等待用户回复        │      │  (Coder/Tester/     │             │
│   └──────────┬──────────┘      │   Critic 循环)      │             │
│              │                  └──────────┬──────────┘             │
│              │                             │                        │
│              └──────────────┬──────────────┘                        │
│                             │                                        │
│                             ▼                                        │
│                    ┌──────────────────┐                             │
│                    │  执行中有新问题？ │                             │
│                    └────────┬─────────┘                             │
│                             │                                        │
│              ┌──────────────┴──────────────┐                        │
│              │ Yes                         │ No                     │
│              ▼                             ▼                        │
│   ┌─────────────────────┐      ┌─────────────────────┐             │
│   │  暂停，向用户提问   │      │  任务完成 ✓         │             │
│   └─────────────────────┘      └─────────────────────┘             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.3 CLI 交互示例

```bash
$ aidev "实现用户认证模块"

📋 分析中...

我分析了这个需求，有以下问题需要确认：

1️⃣ 数据库选型
   - MySQL 还是 PostgreSQL？
   - 建议：PostgreSQL（支持 JSON 字段，扩展性更好）

2️⃣ 认证方式
   - Session 还是 JWT？
   - 建议：JWT（无状态，适合分布式）

3️⃣ 需要以下资源
   - 数据库连接密码
   - JWT 签名密钥

4️⃣ 潜在风险
   - 密码存储需要使用 bcrypt，不能明文
   - 需要考虑 token 刷新机制

请回复你的意见（直接回车表示同意建议）：
> 用 MySQL，其他同意

✅ 收到，开始执行...

[执行中] 创建数据库 schema...
[执行中] 实现用户注册 API...
[执行中] 实现登录 API...

⚠️  执行中发现问题：
   现有代码中 config.go 已经有数据库配置，
   应该复用还是新建？

> 复用现有的

✅ 继续执行...

[测试中] 运行单元测试...
[审查中] Critic 审查代码...

✅ 任务完成！
   - 新增文件: auth/handler.go, auth/jwt.go, auth/model.go
   - 修改文件: config.go
   - 测试覆盖率: 85%
```

### 2.4 系统组成

实际上就这几个部分：

| 组件 | 实现 | 说明 |
|-----|------|------|
| **CLI** | Go cobra/readline | 简单的命令行交互 |
| **Planner Prompt** | 精心调整的系统提示词 | 需求分析、风险识别、任务拆解 |
| **Coder Prompt** | 代码实现提示词 | 批判性地实现代码 |
| **Tester Prompt** | 测试提示词 | 目标是找 bug |
| **Critic Prompt** | 审查提示词 | 目标是发现问题 |
| **Memo** | 现有项目 | 共享上下文 |
| **Git** | 标准 Git | 代码管理 |

### 2.5 Planner 与 Coder/Tester/Critic 的交互逻辑

**核心思路：Planner 只负责开头，之后是执行循环。**

```
┌─────────────────────────────────────────────────────────────────────┐
│                        阶段划分                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐                                                   │
│  │   规划阶段    │  Planner 独立完成，与用户交互                     │
│  └──────┬───────┘                                                   │
│         │                                                            │
│         │ 用户确认后                                                 │
│         ▼                                                            │
│  ┌──────────────┐                                                   │
│  │   执行阶段    │  Coder/Tester/Critic 循环，Planner 不参与        │
│  └──────────────┘                                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**执行阶段的循环：**

```
┌─────────────────────────────────────────────────────────────────────┐
│                     执行循环（单个任务）                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                    ┌─────────┐                                      │
│         ┌────────▶│  Coder  │                                      │
│         │          └────┬────┘                                      │
│         │               │ 代码完成                                   │
│         │               ▼                                            │
│         │          ┌─────────┐                                      │
│         │          │  Test   │ (自动化测试)                         │
│         │          └────┬────┘                                      │
│         │               │                                            │
│         │    ┌──────────┴──────────┐                                │
│         │    │ 测试失败            │ 测试通过                       │
│         │    ▼                     ▼                                │
│         │  返回 Coder          ┌─────────┐                          │
│         │  (附带失败信息)      │ Critic  │                          │
│         │                      └────┬────┘                          │
│         │                           │                                │
│         │               ┌───────────┴───────────┐                   │
│         │               │ 审查不通过            │ 审查通过          │
│         │               ▼                       ▼                   │
│         └─────────  返回 Coder             任务完成 ✓               │
│                    (附带问题列表)                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**关键设计点：**

1. **Planner 不参与执行阶段**
   - 规划完成后，Planner 的工作就结束了
   - 执行阶段是 Coder → Test → Critic 的独立循环
   - 避免角色混乱

2. **信息传递方式**
   ```
   Planner 输出:
   ├── 任务列表 (tasks.json)
   ├── 已确认的决策 (decisions.json)  
   └── 已授权的资源 (resources.json)
        ↓
   写入 Memo，所有 Agent 可读取
        ↓
   Coder/Tester/Critic 从 Memo 获取上下文
   ```

3. **执行中需要用户决策时**
   ```
   Coder 发现问题 → 暂停 → CLI 提问用户 → 用户回复 → 继续执行
   
   注意：这里是 Coder 直接与用户交互，不需要回到 Planner
   ```

4. **重试机制**
   ```go
   maxRetries := 3
   
   for i := 0; i < maxRetries; i++ {
       code := coder.Execute(task)
       
       testResult := tester.Test(code)
       if !testResult.Pass {
           coder.Feedback(testResult.Errors)
           continue
       }
       
       reviewResult := critic.Review(code)
       if !reviewResult.Approved {
           coder.Feedback(reviewResult.Issues)
           continue
       }
       
       return Success
   }
   
   // 重试耗尽，通知用户
   return NeedHumanHelp
   ```

5. **多任务时的调度**
   ```
   任务 DAG:
       A ──┬──▶ B ──┬──▶ D
           │       │
           └──▶ C ─┘
   
   执行顺序：
   1. A 完成（Coder→Test→Critic 循环）
   2. B 和 C 可以并行（如果有多个 Coder）
   3. B 和 C 都完成后，执行 D
   
   每个任务内部都是独立的 Coder→Test→Critic 循环
   ```


---

## 3. 项目流程 DAG

### 3.1 从 Todo List 到 DAG

Plan Mode 的 todo list 是线性的，无法表达复杂依赖。我们需要 **项目流程 DAG**：

```
                              ┌─────────────────┐
                              │  需求分析       │
                              │  (requirement)  │
                              └────────┬────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    ▼                  ▼                  ▼
             ┌──────────┐       ┌──────────┐       ┌──────────┐
             │ 模块设计  │       │ 接口设计  │       │ 测试设计  │
             │ (design) │       │ (api)    │       │ (test)   │
             └────┬─────┘       └────┬─────┘       └────┬─────┘
                  │                  │                  │
                  │      ┌───────────┴───────────┐     │
                  │      │                       │     │
                  ▼      ▼                       ▼     ▼
            ┌──────────────┐              ┌──────────────┐
            │ 模块A实现     │              │ 模块B实现     │
            │ (impl-a)     │              │ (impl-b)     │
            └──────┬───────┘              └──────┬───────┘
                   │                             │
                   │    ┌────────────────────────┘
                   │    │
                   ▼    ▼
            ┌──────────────┐
            │   集成测试    │
            │ (integration)│
            └──────┬───────┘
                   │
                   ▼
            ┌──────────────┐
            │   验收测试    │ ← 需要人类确认
            │ (acceptance) │
            └──────────────┘
```

### 3.2 任务节点定义

```go
type TaskStatus string

const (
    TaskPending    TaskStatus = "pending"     // 等待依赖完成
    TaskReady      TaskStatus = "ready"       // 依赖满足，可执行
    TaskRunning    TaskStatus = "running"     // 执行中
    TaskReview     TaskStatus = "review"      // 等待审查
    TaskBlocked    TaskStatus = "blocked"     // 等待人类决策
    TaskCompleted  TaskStatus = "completed"   // 已完成
    TaskFailed     TaskStatus = "failed"      // 失败
)

type Task struct {
    ID           string            `json:"id"`
    Type         TaskType          `json:"type"`          // code/test/design/review/decision
    Title        string            `json:"title"`
    Description  string            `json:"description"`
    
    // DAG 结构
    Dependencies []string          `json:"dependencies"`  // 前置任务 ID
    Dependents   []string          `json:"dependents"`    // 后续任务 ID
    
    // 执行信息
    Status       TaskStatus        `json:"status"`
    AssignedTo   string            `json:"assigned_to"`   // Agent ID 或 "human"
    
    // 验证要求
    Verification VerificationSpec  `json:"verification"`
    
    // 结果
    Result       *TaskResult       `json:"result,omitempty"`
    
    // 元数据
    CreatedAt    time.Time         `json:"created_at"`
    UpdatedAt    time.Time         `json:"updated_at"`
    
    // 资源需求（需要人类授权）
    Resources    []ResourceRequest `json:"resources,omitempty"`
}

type VerificationSpec struct {
    RequireTests       bool     `json:"require_tests"`        // 必须有测试
    RequireCoverage    float64  `json:"require_coverage"`     // 最低覆盖率
    RequireReview      bool     `json:"require_review"`       // 需要 Critic 审查
    RequireHumanReview bool     `json:"require_human_review"` // 需要人类审查
    AcceptanceCriteria []string `json:"acceptance_criteria"`  // 验收标准
}

type ResourceRequest struct {
    Type        string `json:"type"`        // secret/server/api_key/...
    Name        string `json:"name"`        // 资源名称
    Reason      string `json:"reason"`      // 为什么需要
    Approved    bool   `json:"approved"`    // 是否已授权
    ApprovedBy  string `json:"approved_by"` // 授权人
    ApprovedAt  *time.Time `json:"approved_at,omitempty"`
}
```

### 3.3 DAG 调度算法

```go
// 调度器核心逻辑
func (s *Scheduler) Schedule() {
    for {
        // 1. 获取所有 ready 状态的任务
        readyTasks := s.dag.GetReadyTasks()
        
        // 2. 检查资源冲突（两个任务修改同一文件）
        nonConflicting := s.conflictDetector.Filter(readyTasks)
        
        // 3. 按优先级排序
        sorted := s.prioritize(nonConflicting)
        
        // 4. 分配给空闲 Agent
        for _, task := range sorted {
            if agent := s.agentPool.GetIdle(task.Type); agent != nil {
                s.dispatch(agent, task)
            }
        }
        
        // 5. 检查阻塞任务，通知人类
        blocked := s.dag.GetBlockedTasks()
        for _, task := range blocked {
            s.notifyHuman(task)
        }
        
        time.Sleep(s.interval)
    }
}
```

---

## 4. Agent 角色设计

### 4.1 角色分类

**核心原则：每个角色都独立、批判，不为其他角色的工作「圆场」。**

| 角色 | 职责 | 批判性要求 | 数量 |
|-----|------|----------|------|
| **Planner** | 需求分析、DAG 构建 | 质疑需求合理性，识别风险点 | 1 |
| **Coder** | 代码实现 | 发现问题主动暴露，不糊弄过去 | N (可并行) |
| **Tester** | 测试编写执行 | 目标是「找到 bug」而非「证明没 bug」 | 1 |
| **Critic** | 代码审查 | 目标是「发现问题」而非「帮助通过」 | 1 |

### 4.2 各角色的批判性要求

**Critic Agent** 的核心职责是 **独立验证**，而非帮助执行者通过：

```go
type ReviewResult struct {
    Approved    bool     `json:"approved"`
    Issues      []Issue  `json:"issues"`
    MustFix     []string `json:"must_fix"`      // 必须修复才能通过
    Suggestions []string `json:"suggestions"`   // 建议但不强制
}

type Issue struct {
    Severity    string `json:"severity"`    // critical/major/minor
    Location    string `json:"location"`
    Description string `json:"description"`
}
```

**Critic Prompt 核心指令：**

```
你是一个严格的代码审查者。你的目标是发现问题，而不是帮助代码通过审查。

你必须回答：
1. 这段代码是否完整实现了需求？
2. 是否有 bug 或边界情况未处理？
3. 测试是否充分？

发现 critical/major 问题必须拒绝通过。
不要为了让任务通过而降低标准。
```

### 4.3 为什么不需要 Agent 间通信？

**人类组织的沟通成本问题不需要在 Agent 上复现。**

- 人类需要沟通是因为记忆有限、上下文不共享
- Agent 可以通过 **共享文档**（Memo）获得相同的上下文
- 直接通信引入复杂度：协议设计、消息队列、同步问题

**替代方案：Memo 作为公共大脑（黑板模式）**

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Coder 1 │     │ Coder 2 │     │  Critic │
└────┬────┘     └────┬────┘     └────┬────┘
     │              │              │
     │  读/写       │  读/写       │  读
     ▼              ▼              ▼
┌─────────────────────────────────────────────┐
│               Memo (公共大脑)                 │
│                                              │
│  .memo/index/                                │
│  ├── arch.json       # 架构和模块              │
│  ├── interface.json  # 接口定义                │
│  ├── decisions.json  # 设计决策              │
│  ├── progress.json   # 任务进度              │
│  └── issues.json     # 问题和待办              │
│                                              │
└─────────────────────────────────────────────┘
```

每个 Agent 启动时读取 Memo，获得：
- 项目架构和设计决策
- 当前任务进度
- 其他 Agent 的工作成果（通过 Git 提交）

---

## 5. 人机交互设计

### 5.1 前置确认流程

**核心原则：所有资源需求和关键决策在规划阶段一次性确认。**

```
用户输入需求
     │
     ▼
┌─────────────────────────────────────────────┐
│              Planner 分析                    │
│                                              │
│  1. 解析需求，生成任务 DAG                   │
│  2. 识别所有需要的资源                       │
│  3. 识别所有需要的决策点                     │
│  4. 估算工作量和时间                       │
│                                              │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│              确认请求 (一次性)                 │
│                                              │
│  📝 任务概要:                                 │
│     实现 XX 功能，分解为 5 个子任务              │
│     预计耗时: 2 小时                           │
│                                              │
│  🛠️ 需要的资源:                              │
│     1. 数据库连接密码                          │
│     2. S3 bucket 访问权限                     │
│                                              │
│  ❓ 需要确认的决策:                            │
│     1. 数据库选型: MySQL vs PostgreSQL?        │
│        建议: PostgreSQL，因为...              │
│     2. 缓存策略: Redis vs 内存?             │
│        建议: Redis，因为...                  │
│                                              │
│  [确认] [修改] [取消]                         │
│                                              │
└─────────────────────────────────────────────┘
                       │
                       ▼
                人类确认后开始执行
                (中间不再打扰)
```

### 5.2 确认请求数据结构

```go
type PlanConfirmation struct {
    // 任务概要
    Summary        string   `json:"summary"`
    TaskCount      int      `json:"task_count"`
    EstimatedTime  string   `json:"estimated_time"`
    
    // 资源需求（全部列出）
    Resources      []ResourceRequest `json:"resources"`
    
    // 决策点（全部列出）
    Decisions      []DecisionPoint   `json:"decisions"`
    
    // DAG 预览
    TaskDAG        []TaskPreview     `json:"task_dag"`
}

type ResourceRequest struct {
    Type   string `json:"type"`    // secret/server/api_key
    Name   string `json:"name"`
    Reason string `json:"reason"`  // 为什么需要
}

type DecisionPoint struct {
    Question       string   `json:"question"`
    Options        []string `json:"options"`
    Recommendation string   `json:"recommendation"`
    Reason         string   `json:"reason"`
}

type TaskPreview struct {
    ID          string   `json:"id"`
    Title       string   `json:"title"`
    DependsOn   []string `json:"depends_on"`
    CanParallel bool     `json:"can_parallel"`  // 是否可并行
}
```

---

## 6. 并行策略

### 6.1 尽量避免冲突 + 冲突解决作为 Fallback

**原则：通过 DAG 依赖和任务划分尽量避免冲突，但不为了完全避免而牺牲并行度。**

```
┌─────────────────────────────────────────────┐
│               冲突处理策略                    │
├─────────────────────────────────────────────┤
│                                              │
│  第一防线：主动避免                           │
│     • DAG 依赖分析，有依赖的任务顺序执行     │
│     • 任务划分时尽量让不同任务操作不同文件   │
│     • 模块化设计，减少交叉依赖               │
│                                              │
│  第二防线：Fallback 机制                      │
│     • 如果冲突发生，Agent 自行解决            │
│     • 不为了完全避免而阻塞并行               │
│                                              │
└─────────────────────────────────────────────┘
```

Anthropic 的经验（作为 Fallback 的依据）：
> "Merge conflicts are frequent, but Claude is smart enough to figure that out."

### 6.2 什么时候引入多 Agent 并行？

**原则：仅在任务可以独立拆解时才引入 multi-agent。**

| 场景 | 是否并行 | 说明 |
|------|---------|------|
| 3 个独立模块开发 | ✅ 是 | 每个模块一个 Coder |
| 测试 100 个独立 case | ✅ 是 | Anthropic 的方式 |
| 一个复杂的大模块 | ❌ 否 | 单 Agent 顺序执行 |
| 有严格顺序依赖的任务 | ❌ 否 | DAG 顺序执行 |

### 6.3 Git 工作流

```
main ─────●─────●─────●─────●─────▶
          \         \         /
           \         \       /
task/001 ───●──●──●────╯       /  ← Coder1
                    merge    /
                            /
task/002 ──────●──●──●────╯      ← Coder2 (可能有 conflict)

规则:
1. 每个任务一个分支
2. 完成后 PR 到 main
3. 冲突时 Agent 自行解决
4. Critic review PR
```

---

## 7. Git 协作模式

### 7.1 改进 Anthropic 的简单锁机制

Anthropic 用文件锁 + git 同步，问题：
- 频繁 merge conflict
- 无法表达文件级依赖

我们的改进：

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Git Workflow                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│    main ─────●─────●─────●─────●─────●─────●─────▶                  │
│               \           \           \                              │
│                \           \           \                             │
│    task/001 ────●──●──●────╳           \                            │
│                     merge──┘            \                            │
│                                          \                           │
│    task/002 ─────────●──●──●──●──────────╳                          │
│                           merge──────────┘                           │
│                                                                      │
│   规则：                                                              │
│   1. 每个任务一个分支 (task/{task_id})                               │
│   2. 只有通过验证的任务才能合并到 main                                │
│   3. 调度器确保有依赖的任务顺序执行                                   │
│   4. 无依赖的任务可以并行，合并时解决冲突                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 文件级锁定

```go
type FileLock struct {
    Path     string    `json:"path"`       // 文件路径
    TaskID   string    `json:"task_id"`    // 持有锁的任务
    AgentID  string    `json:"agent_id"`   // 持有锁的 Agent
    LockType LockType  `json:"lock_type"`  // read/write
    AcquiredAt time.Time `json:"acquired_at"`
}

type LockType string
const (
    LockRead  LockType = "read"   // 允许多个读
    LockWrite LockType = "write"  // 独占写
)

// 调度器在分配任务前检查文件冲突
func (s *Scheduler) checkFileConflict(task *Task) bool {
    affectedFiles := s.analyzer.GetAffectedFiles(task)
    for _, file := range affectedFiles {
        if lock := s.lockManager.GetLock(file); lock != nil {
            if lock.LockType == LockWrite || task.NeedsWrite(file) {
                return true // 有冲突
            }
        }
    }
    return false
}
```

---

## 8. 实现路线图

### Phase 1: 单 Agent MVP (Week 1-2)

- [ ] 项目脚手架 (Go module)
- [ ] Kimi Agent SDK 集成
- [ ] Planner: 需求分析、资源/决策前置收集
- [ ] Coder: 单 Agent 执行
- [ ] CLI 界面: 确认流程

### Phase 2: Memo 集成 (Week 3-4)

- [ ] 改造 Memo: 新增 decisions/progress/resources
- [ ] Agent 读取 Memo 上下文
- [ ] Agent 更新进度到 Memo

### Phase 3: 验证层 (Week 5-6)

- [ ] Test Runner 集成
- [ ] Critic Agent
- [ ] 验证流程: Test + Critic 都通过才完成

### Phase 4: 多 Agent 并行 (Week 7-8)

- [ ] 任务 DAG
- [ ] 多 Coder 并行执行
- [ ] Git 分支管理
- [ ] Merge conflict 处理

### Phase 5: 优化与生产化 (Week 9-10)

- [ ] 错误恢复机制
- [ ] 日志与监控
- [ ] 端到端测试

---

## 9. 关键技术决策

### 9.1 为什么用 Go？

| 考量 | Go 的优势 |
|-----|----------|
| Kimi SDK | 官方提供 Golang SDK |
| 并发模型 | goroutine 天然适合多 Agent 调度 |
| 部署简单 | 单二进制，易于分发 |
| 生态 | Git 操作、测试框架都有成熟库 |

### 9.2 状态存储选择

```go
// 选项 1: SQLite (单机简单)
// 选项 2: PostgreSQL (分布式)
// 选项 3: 纯文件 (Git-native)

// 建议：Phase 1 用文件 + Git，Phase 2 迁移到 SQLite
```

### 9.3 Agent 上下文管理

借鉴 Anthropic 的经验：
- 避免上下文污染（输出精简、分页）
- 维护 README/进度文件
- 日志写文件而非 stdout

---

## 10. 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|-----|-------|------|---------|
| Agent 陷入循环 | 高 | 中 | 设置最大重试次数、超时、人类干预 |
| merge conflict 频繁 | 中 | 中 | 文件级锁、任务粒度细化 |
| 需求理解偏差 | 高 | 高 | 需求确认流程、中间检查点 |
| 测试不充分 | 中 | 高 | Critic 独立审查、覆盖率门槛 |
| 成本失控 | 中 | 中 | Token 预算、任务超时 |

---

## 附录: 与现有方案对比

| 特性 | Plan Mode | Anthropic Agent Teams | 本框架 |
|-----|-----------|----------------------|--------|
| 任务结构 | 线性 Todo | 文件锁 | DAG |
| 并行能力 | 无 | 乐观并行 | 乐观并行 (保留) |
| 验证机制 | 人工审查 | 测试驱动 | Test + Critic |
| 人机协作 | 每步确认 | 几乎无 | 前置确认 |
| Agent 通信 | 无 | 无 | 无 (用 Memo 替代) |
| 共享上下文 | 无 | 无 | Memo 公共大脑 |
| 资源授权 | 无 | 无 | 前置收集 |

### 2.6 代码设计原则：极简 + Prompt 外置

**核心原则：代码是胶水，Prompt 是灵魂**

```
代码职责：
├── 读取用户输入
├── 调用 Kimi API
├── 解析响应
├── 执行工具调用
└── 循环直到完成

Prompt 职责：
├── 定义角色行为
├── 规定输出格式
├── 约束决策边界
└── 指导问题处理
```

**项目结构：**

```
aidev/
├── main.go              # 入口，< 50 行
├── agent.go             # Agent 封装，< 100 行
├── tools.go             # 工具定义，< 150 行
├── loop.go              # 主循环，< 100 行
│
├── prompts/             # Prompt 文件（编译时嵌入）
│   ├── planner.md       # Planner 系统提示词
│   ├── coder.md         # Coder 系统提示词
│   ├── tester.md        # Tester 系统提示词
│   └── critic.md        # Critic 系统提示词
│
└── go.mod
```

**Go Embed 使用：**

```go
package main

import (
    "embed"
)

//go:embed prompts/*.md
var promptFS embed.FS

func loadPrompt(name string) string {
    data, _ := promptFS.ReadFile("prompts/" + name + ".md")
    return string(data)
}

// 使用
plannerPrompt := loadPrompt("planner")
coderPrompt := loadPrompt("coder")
```

**代码简洁原则：**

| 原则 | 说明 |
|------|------|
| **单文件可读** | 每个 .go 文件独立可理解，无需跳转 |
| **无抽象层** | 不搞 interface 套 interface，直接写 |
| **无配置文件** | 配置直接写代码里或用环境变量 |
| **无 ORM** | 直接 SQL 或文件操作 |
| **无框架** | 标准库 + Kimi SDK，够了 |

**主循环伪代码（完整逻辑 < 50 行）：**

```go
func main() {
    task := os.Args[1]
    
    // 1. 规划阶段
    plan := runAgent("planner", task)
    questions := extractQuestions(plan)
    
    if len(questions) > 0 {
        answers := askUser(questions)
        plan = runAgent("planner", task + "\n用户回复：" + answers)
    }
    
    // 2. 执行阶段
    for _, subtask := range plan.Tasks {
        for retry := 0; retry < 3; retry++ {
            code := runAgent("coder", subtask)
            
            testResult := runTests()
            if !testResult.Pass {
                subtask = subtask + "\n测试失败：" + testResult.Error
                continue
            }
            
            review := runAgent("critic", code)
            if !review.Approved {
                subtask = subtask + "\n审查意见：" + review.Issues
                continue
            }
            
            break // 成功
        }
    }
    
    fmt.Println("✅ Done")
}
```

**Prompt 文件示例 (prompts/planner.md)：**

```markdown
# Planner

你是一个开发任务规划者。

## 职责

1. 分析用户需求
2. 拆解为可执行的子任务
3. 识别需要用户确认的决策点
4. 识别需要的外部资源

## 输出格式

必须输出 JSON：

\`\`\`json
{
  "questions": [
    {"id": 1, "question": "...", "suggestion": "..."}
  ],
  "tasks": [
    {"id": "T1", "description": "...", "depends_on": []}
  ],
  "resources_needed": ["数据库密码", "API Key"]
}
\`\`\`

## 约束

- 每个任务必须是独立可测试的
- 任务粒度：1个任务 = 1个文件或1个函数
- 有依赖关系时必须声明 depends_on
```

**为什么这样设计：**

1. **Prompt 是核心资产**
   - 代码逻辑简单固定，不常改
   - Prompt 需要反复调优，独立文件方便迭代
   - Markdown 格式，版本控制友好

2. **编译时嵌入的好处**
   - 单二进制部署，无外部依赖
   - 启动快，无文件 IO
   - 不会出现"找不到配置文件"的问题

3. **代码简单的好处**
   - 一个人能完全理解
   - 出问题容易排查
   - 修改成本低
