# BachLedger 可观测性设计

> 报告编号: REPORT-002
> 日期: 2026-02-06
> 状态: 待确认

---

## 1. 可观测性目标

| 目标 | 说明 |
|------|------|
| **性能瓶颈定位** | 精确识别哪个阶段/模块是瓶颈 |
| **优化效果验证** | 量化每次优化的收益 |
| **异常检测** | 快速发现性能退化 |
| **容量规划** | 预测系统扩展需求 |

---

## 2. OEV 流水线指标设计

### 2.1 流水线时序图 + 测量点

```
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                        OEV Pipeline Metrics                              │
    ├─────────────────────────────────────────────────────────────────────────┤
    │                                                                          │
    │  TX Input        Ordering          Execution         Validation   Storage│
    │     │               │                  │                 │           │   │
    │     │◄──── M1 ────►│◄───── M2 ───────►│◄───── M3 ──────►│◄── M4 ──►│   │
    │     │               │                  │                 │           │   │
    │  ┌──▼──┐        ┌───▼───┐         ┌────▼────┐       ┌────▼────┐ ┌───▼───┐
    │  │TxPool│───────│ TBFT  │─────────│Scheduler│───────│ Verify  │─│ Commit│
    │  └──┬──┘        └───┬───┘         └────┬────┘       └────┬────┘ └───┬───┘
    │     │               │                  │                 │           │   │
    │     │◄─ M1.1 ─►│◄─ M2.1 ─►│      ◄─ M3.1 ─►│       ◄─ M4.1 ─►│        │
    │     │          │          │      │         │       │         │        │
    │   收集TX     提案生成   投票轮次  乐观执行   冲突检测  签名验证   状态写入  │
    │                                  重执行                                  │
    │                                                                          │
    └─────────────────────────────────────────────────────────────────────────┘

    M1: TxPool 阶段耗时
    M2: Ordering (TBFT) 阶段耗时
    M3: Execution (Seamless) 阶段耗时
    M4: Validation + Storage 阶段耗时
```

### 2.2 核心指标定义

#### 流水线级别指标

| 指标名 | 类型 | 单位 | 说明 |
|--------|------|------|------|
| `pipeline.block.latency` | Histogram | ms | 区块从开始到提交的总延迟 |
| `pipeline.block.tps` | Gauge | tx/s | 实时 TPS |
| `pipeline.stage.ordering.duration` | Histogram | ms | Ordering 阶段耗时 |
| `pipeline.stage.execution.duration` | Histogram | ms | Execution 阶段耗时 |
| `pipeline.stage.validation.duration` | Histogram | ms | Validation 阶段耗时 |
| `pipeline.stage.storage.duration` | Histogram | ms | Storage 阶段耗时 |

#### Seamless Scheduling 专项指标

| 指标名 | 类型 | 单位 | 说明 |
|--------|------|------|------|
| `scheduler.optimistic_exec.duration` | Histogram | μs | 乐观执行耗时 |
| `scheduler.conflict_detect.duration` | Histogram | μs | 冲突检测耗时 |
| `scheduler.re_exec.count` | Counter | 次 | 重执行次数 |
| `scheduler.re_exec.rounds` | Histogram | 轮 | 每块重执行轮数 |
| `scheduler.ownership.contention` | Gauge | % | 所有权争用率 |
| `scheduler.thread.idle_ratio` | Gauge | % | 线程空闲率（核心指标！）|
| `scheduler.thread.utilization` | Gauge | % | 线程利用率 |

#### EVM 执行指标

| 指标名 | 类型 | 单位 | 说明 |
|--------|------|------|------|
| `evm.tx.duration` | Histogram | μs | 单笔交易执行耗时 |
| `evm.tx.gas_used` | Histogram | gas | 单笔交易 gas 消耗 |
| `evm.opcode.count` | Counter | 次 | 各操作码执行次数 |
| `evm.storage.read` | Counter | 次 | SLOAD 次数 |
| `evm.storage.write` | Counter | 次 | SSTORE 次数 |
| `evm.call.depth` | Histogram | 层 | 调用深度 |

#### TBFT 共识指标

| 指标名 | 类型 | 单位 | 说明 |
|--------|------|------|------|
| `consensus.round.duration` | Histogram | ms | 单轮共识耗时 |
| `consensus.round.count` | Histogram | 轮 | 达成共识所需轮数 |
| `consensus.proposal.latency` | Histogram | ms | 提案延迟 |
| `consensus.vote.latency` | Histogram | ms | 投票延迟 |
| `consensus.timeout.count` | Counter | 次 | 超时次数 |

#### 密码学操作指标

| 指标名 | 类型 | 单位 | 说明 |
|--------|------|------|------|
| `crypto.sign.duration` | Histogram | μs | 签名耗时 |
| `crypto.verify.duration` | Histogram | μs | 验签耗时 |
| `crypto.hash.duration` | Histogram | μs | 哈希耗时 |
| `crypto.recover.duration` | Histogram | μs | 公钥恢复耗时 |

---

## 3. 实现方案

### 3.1 Metrics Trait 设计

```rust
/// 指标收集器 trait
pub trait Metrics: Send + Sync {
    /// 记录直方图数据点
    fn histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// 增加计数器
    fn counter(&self, name: &str, delta: u64, labels: &[(&str, &str)]);

    /// 设置仪表盘值
    fn gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

/// 计时器辅助宏
#[macro_export]
macro_rules! timed {
    ($metrics:expr, $name:expr, $labels:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        $metrics.histogram($name, start.elapsed().as_micros() as f64, $labels);
        result
    }};
}

// 使用示例
fn execute_block(&self, block: &Block) -> ExecutedBlock {
    timed!(self.metrics, "pipeline.stage.execution.duration", &[], {
        // 执行逻辑
    })
}
```

### 3.2 存储后端选项

| 选项 | 优点 | 缺点 | 建议 |
|------|------|------|------|
| **A: 内存 + JSON 导出** | 简单，无依赖 | 无持久化 | MVP 阶段 |
| B: Prometheus 格式 | 标准，可对接生态 | 需要额外服务 | 生产环境 |
| C: 自研时序存储 | 完全控制 | 工作量大 | 不推荐 |

### 3.3 数据结构设计

```rust
/// 直方图实现（无外部依赖）
pub struct Histogram {
    /// 分桶边界 (μs): [10, 50, 100, 500, 1000, 5000, 10000, ...]
    buckets: Vec<f64>,
    /// 每个桶的计数
    counts: Vec<AtomicU64>,
    /// 总和（用于计算平均值）
    sum: AtomicU64,
    /// 总数
    count: AtomicU64,
}

impl Histogram {
    pub fn observe(&self, value: f64) { ... }

    pub fn percentile(&self, p: f64) -> f64 { ... }  // p50, p95, p99

    pub fn mean(&self) -> f64 { ... }
}

/// 指标快照（用于导出）
#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub histograms: HashMap<String, HistogramSnapshot>,
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
}
```

---

## 4. 可视化报告设计

### 4.1 实时 CLI 仪表盘

```
┌─────────────────── BachLedger Metrics ───────────────────┐
│                                                          │
│  Block Height: 12,345      TPS: 4,823     Time: 14:32:05 │
│                                                          │
│  ─────────────── Pipeline Latency (ms) ──────────────    │
│                                                          │
│  Ordering   ████████░░░░░░░░░░░░░░░░░░░░░░░  45ms (18%)  │
│  Execution  ████████████████████░░░░░░░░░░░ 156ms (62%)  │
│  Validation ████░░░░░░░░░░░░░░░░░░░░░░░░░░░  32ms (13%)  │
│  Storage    ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  18ms  (7%)  │
│                                                          │
│  ─────────────── Thread Utilization ─────────────────    │
│                                                          │
│  [████████████████████████████░░░░] 87.5% (56/64 cores)  │
│  Idle Ratio: 12.5%  (Target: <10%)                       │
│                                                          │
│  ─────────────── Scheduler Stats ────────────────────    │
│                                                          │
│  Re-exec Rounds:  avg=1.2  p95=3  p99=5                  │
│  Conflict Rate:   8.3%                                   │
│  Ownership Wait:  avg=12μs  p95=45μs                     │
│                                                          │
│  ─────────────── EVM Stats ──────────────────────────    │
│                                                          │
│  TX Exec Time:    avg=89μs  p95=234μs  p99=567μs         │
│  SLOAD/block:     1,234     SSTORE/block: 456            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### 4.2 性能报告（JSON 格式）

```json
{
  "report_time": "2026-02-06T14:32:05Z",
  "duration_secs": 60,
  "summary": {
    "blocks_processed": 48,
    "transactions_processed": 231456,
    "avg_tps": 3857,
    "avg_block_latency_ms": 251
  },
  "pipeline_breakdown": {
    "ordering": { "avg_ms": 45, "p95_ms": 78, "p99_ms": 112, "pct": 18 },
    "execution": { "avg_ms": 156, "p95_ms": 234, "p99_ms": 312, "pct": 62 },
    "validation": { "avg_ms": 32, "p95_ms": 56, "p99_ms": 89, "pct": 13 },
    "storage": { "avg_ms": 18, "p95_ms": 34, "p99_ms": 67, "pct": 7 }
  },
  "scheduler": {
    "thread_utilization_pct": 87.5,
    "idle_ratio_pct": 12.5,
    "avg_reexec_rounds": 1.2,
    "conflict_rate_pct": 8.3
  },
  "bottleneck_analysis": {
    "primary_bottleneck": "execution",
    "sub_bottleneck": "evm.storage.read",
    "recommendation": "Consider state caching optimization"
  }
}
```

### 4.3 对比报告（优化前后）

```
┌────────────────── Performance Comparison ──────────────────┐
│                                                            │
│  Baseline: commit abc123    Current: commit def456         │
│                                                            │
│  Metric                    Baseline    Current    Change   │
│  ──────────────────────────────────────────────────────    │
│  TPS                         3,200      4,823    +50.7% ✓  │
│  Block Latency (avg)         312ms      251ms    -19.6% ✓  │
│  Thread Utilization          72.3%      87.5%    +15.2% ✓  │
│  Idle Ratio                  27.7%      12.5%    -15.2% ✓  │
│  Re-exec Rounds (avg)          2.1        1.2    -42.9% ✓  │
│                                                            │
│  Crypto (verify, avg)         65μs       64μs     -1.5%    │
│  EVM (tx exec, avg)          112μs       89μs    -20.5% ✓  │
│                                                            │
│  Summary: +50.7% TPS improvement, execution optimized      │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## 5. 采样策略

### 5.1 分层采样

| 层级 | 采样率 | 说明 |
|------|--------|------|
| **Block** | 100% | 每个区块必须记录 |
| **Transaction** | 10% | 高负载时降采样 |
| **Opcode** | 1% | 仅调试时全采样 |

### 5.2 动态采样

```rust
pub struct AdaptiveSampler {
    base_rate: f64,
    current_rate: f64,
    load_threshold: f64,
}

impl AdaptiveSampler {
    pub fn should_sample(&self, current_tps: f64) -> bool {
        if current_tps > self.load_threshold {
            // 高负载时降低采样率
            rand::random::<f64>() < self.current_rate * 0.1
        } else {
            rand::random::<f64>() < self.current_rate
        }
    }
}
```

---

## 6. 架构集成

### 6.1 模块集成点

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   ┌───────────┐     ┌───────────┐     ┌───────────┐        │
│   │  TxPool   │────►│ Consensus │────►│ Scheduler │        │
│   └─────┬─────┘     └─────┬─────┘     └─────┬─────┘        │
│         │                 │                 │               │
│         ▼                 ▼                 ▼               │
│   ┌─────────────────────────────────────────────────┐      │
│   │              MetricsCollector                    │      │
│   │                                                  │      │
│   │   collect() ──► aggregate() ──► export()        │      │
│   │                                                  │      │
│   └─────────────────────────┬───────────────────────┘      │
│                             │                               │
│              ┌──────────────┼──────────────┐               │
│              ▼              ▼              ▼               │
│         ┌────────┐    ┌────────┐    ┌────────┐            │
│         │  CLI   │    │  JSON  │    │Prometheus│           │
│         │Dashboard│    │ Export │    │ Endpoint │           │
│         └────────┘    └────────┘    └────────┘            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 新增 Crate

```
crates/
├── bach-metrics/          # 新增：可观测性基础设施
│   ├── src/
│   │   ├── lib.rs
│   │   ├── histogram.rs   # 直方图实现
│   │   ├── collector.rs   # 指标收集器
│   │   ├── exporter.rs    # 导出器 (JSON/CLI)
│   │   └── sampler.rs     # 采样器
│   └── Cargo.toml
```

---

## 7. 待确认问题

1. **CLI 仪表盘**：是否需要实时 CLI 仪表盘？（推荐: 是）

2. **指标粒度**：是否需要 Opcode 级别的细粒度指标？（推荐: 可选，调试时开启）

3. **导出格式**：除了 JSON，是否需要 Prometheus 格式？（推荐: MVP 阶段仅 JSON）

4. **存储**：指标数据是否需要持久化？（推荐: MVP 阶段仅内存）

---

## 8. 实现优先级

| 阶段 | 指标 | 优先级 |
|------|------|--------|
| Phase 1 | 基础 Metrics trait + 时间宏 | P0 |
| Phase 2 | Pipeline 阶段耗时 | P0 |
| Phase 3 | Scheduler 专项指标（空闲率） | P0 |
| Phase 4 | EVM 执行指标 | P1 |
| Phase 5 | CLI 仪表盘 | P1 |
| Phase 6 | 对比报告生成 | P2 |

---

*报告结束 - 等待确认*
