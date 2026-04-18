# oxo-call 满血版 - GitHub Issues 清单

**目标**: 基于全面审计和测试，生成可执行的优化 Issue
**评估版本**: v0.11.0
**生成时间**: 2026-04-18

---

## P0 - 极速 (性能关键路径)

### Issue #1: [性能] 缓存系统 O(n) → O(1) 优化

**标签**: `P0`, `performance`, `good first issue`

**问题描述**:
`src/cache.rs` 使用 JSONL 线性扫描，每次查找都读取整个文件。假设 1000 条缓存 ≈ 200KB，每次查找 5-20ms。

**代码位置**:
```rust
// src/cache.rs:115-135
for line in content.lines() {
    if let Ok(entry) = serde_json::from_str::<CacheEntry>(line)
        && entry.hash == hash { ... }
}
```

**优化方案**:
1. 启动时加载全部缓存到 `DashMap<String, CacheEntry>`（O(1) 查找）
2. 后台异步 WAL 持久化（写前日志，避免每次修改都全量写回）
3. LRU 淘汰策略，max_entries 可配置（默认 10,000）

**预期收益**:
- 缓存查找: 5ms → 0.05ms (100x 提升)
- 支持高频调用场景（批处理、循环脚本）

**测试标准**:
```bash
cargo bench -- cache_lookup
# 基准: 1000 次查找 < 50ms
cargo test --lib cache
```

**复杂度**: 中等  
**预估工时**: 2-3 天

---

### Issue #2: [性能] 批处理同步 I/O → 异步化

**标签**: `P0`, `performance`, `async`

**问题描述**:
`src/runner/batch.rs` 使用 `spawn_blocking` + `std::process::Command`，线程池可能成为瓶颈。

**代码位置**:
```rust
// src/runner/batch.rs:97-110
tokio::task::spawn_blocking(move || {
    std::process::Command::new("sh")  // 阻塞线程
        .arg("-c").arg(&cmd).status()
}).await?
```

**优化方案**:
1. 使用 `tokio::process::Command` 实现真正的异步执行
2. 工作窃取调度 (`tokio::task::spawn`)
3. 保持信号量并发控制，避免资源耗尽

**预期收益**:
- 批处理吞吐量: +50-100%
- 降低线程上下文切换开销

**测试标准**:
```bash
# 100 个 samtools sort 任务，8 并行
oxo-call run samtools "sort input.bam" --input-list samples.txt -j 8
# 监控 CPU 利用率和完成时间
```

**复杂度**: 中等  
**预估工时**: 1-2 天

---

### Issue #3: [性能] HTTP 客户端连接池调优

**标签**: `P0`, `performance`, `http`

**问题描述**:
`src/llm/provider.rs` 使用 `reqwest::Client::new()` 默认配置，缺少连接池调优。

**代码位置**:
```rust
// src/llm/provider.rs:41-46
pub fn new(config: Config) -> Self {
    LlmClient {
        config,
        client: reqwest::Client::new(),  // 默认配置
    }
}
```

**优化方案**:
```rust
client: reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .connect_timeout(Duration::from_secs(10))
    .pool_max_idle_per_host(16)
    .pool_idle_timeout(Duration::from_secs(300))
    .tcp_keepalive(Duration::from_secs(60))
    .build()?
```

**预期收益**:
- LLM 调用延迟: -30%
- 连接复用率提升，减少 TCP 握手开销

**复杂度**: 低  
**预估工时**: 0.5 天

---

### Issue #4: [性能] 文档内存缓存

**标签**: `P0`, `performance`, `memory`

**问题描述**:
同一工具的多次调用重复从磁盘读取文档，无内存缓存。

**代码位置**:
```rust
// src/docs.rs
if !skip_cache && let Ok(cached) = self.load_cache(tool) {
    docs.cached_docs = Some(cached);  // 每次都读磁盘
}
```

**优化方案**:
1. 使用 `Arc<String>` 共享文档内容（零拷贝）
2. LRU 内存缓存，避免内存膨胀
3. 可选的内存限制配置（默认 100MB）

**预期收益**:
- 重复工具调用: 零 I/O
- 适合批处理同类型任务

**复杂度**: 低  
**预估工时**: 1 天

---

## P0 - 准确 (LLM 准确性)

### Issue #5: [准确] Flag 硬验证层

**标签**: `P0`, `accuracy`, `llm`, `critical`

**问题描述**:
当前依赖 LLM 自律不发明 flag，但小模型（≤3B）仍有幻觉风险。

**优化方案**:
1. 生成后解析 `ARGS`，提取所有 flag（`--*` 和 `-?` 模式）
2. 验证所有 flag 是否来自：
   - Skill 文件中的示例
   - 文档提取的 flag 目录（`StructuredDoc.flag_catalog`）
3. 发现未知 flag 时：
   - 自动降级重试（仅使用已知 flag 子集重新生成）
   - 或提示用户确认

**代码位置**:
```rust
// src/llm/response.rs 新增
pub fn validate_flags(args: &str, valid_flags: &[String]) -> Result<(), InvalidFlag>;
```

**预期收益**:
- Flag 幻觉率: -80%
- 用户信任度显著提升

**测试标准**:
```bash
# 使用 0.5B 模型测试 100 个 samtools 命令
oxo-call dry-run samtools "xxx" --model tinyllama
# 统计 flag 准确率
```

**复杂度**: 中等  
**预估工时**: 2-3 天

---

### Issue #6: [准确] 子命令硬验证层

**标签**: `P0`, `accuracy`, `llm`, `critical`

**问题描述**:
samtools/bcftools 等工具要求子命令第一，但 LLM 可能生成错误顺序。

**优化方案**:
1. 从文档 `USAGE` 段提取合法子命令列表
2. 验证 `ARGS` 第一个 token 是否为合法子命令
3. 不合法时自动修正或重试

**测试标准**:
```bash
# 故意描述模糊的任务，验证子命令正确性
oxo-call dry-run samtools "process the BAM file"
# 期望: 第一个 token 必须是 view/sort/index/flagstat 等之一
```

**复杂度**: 低  
**预估工时**: 1-2 天

---

## P1 - 准确 (架构升级)

### Issue #7: [架构] LlmProvider trait 真正实现

**标签**: `P1`, `architecture`, `breaking-change`

**问题描述**:
`LlmProvider` trait 已定义但未使用，所有 provider 逻辑硬编码在 `LlmClient`。

**代码位置**:
```rust
// src/llm/types.rs:46-50 (已定义但未使用)
pub trait LlmProvider {
    async fn chat_completion(...) -> Result<String>;
    fn name(&self) -> &str;
}
```

**优化方案**:
```rust
pub struct LlmClient {
    provider: Box<dyn LlmProvider>,
    config: Config,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let provider: Box<dyn LlmProvider> = match config.provider.as_str() {
            "openai" => Box::new(OpenAiProvider::new(&config)),
            "anthropic" => Box::new(AnthropicProvider::new(&config)),
            "github-copilot" => Box::new(CopilotProvider::new(&config)),
            "ollama" => Box::new(OllamaProvider::new(&config)),
            _ => panic!("Unknown provider"),
        };
        Self { provider, config }
    }
}
```

**预期收益**:
- 运行时 provider 切换
- provider 链式 fallback
- 更容易添加新 provider

**复杂度**: 中等  
**预估工时**: 3-5 天

---

### Issue #8: [架构] Orchestrator 完全集成

**标签**: `P1`, `architecture`, `multi-agent`

**问题描述**:
`orchestrator/` 模块已实现 Supervisor/Planner/Executor/Validator，但 `main.rs` 仍主要使用 `runner::Runner`。

**优化方案**:
1. 将 `supervisor.decide()` 作为默认执行入口
2. 实现 `planner` 的完整任务分解（pipeline 检测 + 步骤规划）
3. 实现 `validator` 的自动修复建议
4. 添加 `--verbose` 可视化 orchestration 决策过程

**预期收益**:
- 复杂任务准确率: +20%
- 支持多步 pipeline 自动化
- 失败时自动诊断和修复建议

**测试标准**:
```bash
# 复杂多步任务
oxo-call run --scenario full samtools "sort BAM, then index it, then get flagstat"
# 验证自动分解为 3 个步骤并正确执行
```

**复杂度**: 高  
**预估工时**: 5-7 天

---

### Issue #9: [准确] 小模型运行时自适应

**标签**: `P1`, `llm`, `small-models`

**问题描述**:
当前 Prompt tier 只基于模型大小静态选择，缺少基于实际输出质量的动态调整。

**优化方案**:
1. 首次生成后评估质量（格式合规性、flag 有效性）
2. 质量不达标时自动降级提示词（Full → Medium → Compact）
3. 动态温度微调（空输出 → 降低 temperature）
4. ≤3B 模型增加 "thinking" 预推理步骤

**测试标准**:
```bash
# 使用 0.5B-3B 模型测试
oxo-call dry-run samtools "sort input.bam" --model qwen2.5:0.5b
# 验证准确率提升 15-25%
```

**复杂度**: 中等  
**预估工时**: 3-4 天

---

## P1 - 可靠

### Issue #10: [可靠] 流式响应支持

**标签**: `P1`, `reliability`, `streaming`

**问题描述**:
当前 LLM 调用阻塞等待完整响应，无流式处理。

**优化方案**:
1. 实现 SSE 流式解析（OpenAI/Anthropic 原生支持）
2. 实时显示生成进度（逐字输出）
3. 首 token 延迟 < 100ms

**测试标准**:
```bash
oxo-call dry-run samtools "sort input.bam" --stream
# 观察实时输出
```

**复杂度**: 中等  
**预估工时**: 2-3 天

---

### Issue #11: [可靠] 错误反馈闭环

**标签**: `P1`, `reliability`, `feedback`

**问题描述**:
执行失败时记录到 error_db，但缺少成功修复后的反馈学习。

**优化方案**:
1. 用户修复命令并成功执行后，记录"修复对"
2. 自动更新 mini-skill cache
3. 同类错误二次发生时优先使用历史修复方案

**预期收益**:
- 同类错误二次成功率: +40%

**复杂度**: 低  
**预估工时**: 1-2 天

---

### Issue #12: [可靠] 危险命令检测

**标签**: `P1`, `safety`, `security`

**问题描述**:
缺少对危险命令的自动检测和警告。

**优化方案**:
1. 危险模式检测：`--rm`, `DROP`, `mkfs`, `dd if=/dev/zero` 等
2. 文件系统破坏性操作确认
3. dry-run 差异对比（预期 vs 实际）

**测试标准**:
```bash
oxo-call run rm "delete all files in /data" --dry-run
# 期望: 警告提示 + 确认要求
```

**复杂度**: 低  
**预估工时**: 1 天

---

## P2 - 智慧

### Issue #13: [智慧] 上下文感知优化

**标签**: `P2`, `ux`, `personalization`

**问题描述**:
每次调用都是独立的，不记住用户偏好。

**优化方案**:
1. 记住用户常用参数（默认线程数、默认路径等）
2. 学习特定数据集的处理模式
3. 个性化命令推荐

**测试标准**:
```bash
# 多次使用 -@ 16 后，自动生成时默认使用 16 线程
oxo-call run samtools "sort input.bam"  # 自动使用 -@ 16
```

**复杂度**: 中等  
**预估工时**: 3-5 天

---

### Issue #14: [智慧] 交互式调试

**标签**: `P2`, `ux`, `debugging`

**问题描述**:
执行失败时无交互式诊断。

**优化方案**:
1. 失败时提供交互式诊断模式
2. LLM 逐步分析 stderr
3. 建议修复方案并一键应用

**测试标准**:
```bash
oxo-call run samtools "sort input.bam" --interactive
# 失败时进入交互模式
```

**复杂度**: 中等  
**预估工时**: 3-4 天

---

### Issue #15: [智慧] Chat 模式完善

**标签**: `P2`, `ux`, `chat`

**问题描述**:
`chat` 子命令已存在，但功能较简单。

**优化方案**:
1. 多轮对话上下文管理
2. 对话历史回溯与复用
3. 基于对话历史的命令优化

**测试标准**:
```bash
oxo-call chat
# > 我想分析 RNA-seq 数据
# > 用什么工具比较好？
# > 具体怎么跑？
```

**复杂度**: 中等  
**预估工时**: 2-3 天

---

## 不纳入计划 (专注核心目标)

根据维护者指示，以下功能**不纳入满血版计划**：

- ❌ 其他组学 workflow (空间转录组、蛋白质组学、Hi-C 等)
- ❌ 外部工作流平台集成 (Galaxy、Dockstore)
- ❌ 云平台深度集成 (AWS/GCP/Azure 专有功能)

专注投入: oxo-call 本身核心能力的极致优化

---

## 执行建议

### Sprint 1 (2 周): P0 极速
- Issue #1: 缓存 O(1) 化
- Issue #2: 批处理异步化
- Issue #3: HTTP 客户端调优

### Sprint 2 (2 周): P0 准确
- Issue #5: Flag 硬验证
- Issue #6: 子命令验证
- Issue #4: 文档内存缓存

### Sprint 3 (3 周): P1 架构
- Issue #7: Provider trait 重构
- Issue #8: Orchestrator 集成
- Issue #9: 小模型自适应

### Sprint 4 (2 周): P1 可靠
- Issue #10: 流式响应
- Issue #11: 错误反馈闭环
- Issue #12: 危险命令检测

---

## 测试矩阵

每个 Issue 完成后需通过：

| 测试项 | 标准 |
|--------|------|
| `cargo test --lib` | 100% 通过 |
| `cargo clippy` | 无警告 |
| `cargo fmt --check` | 通过 |
| 功能测试 | 对应功能正常工作 |
| 性能基准 | 达到预期收益 |

---

*Generated by oxo-call comprehensive audit, 2026-04-18*
