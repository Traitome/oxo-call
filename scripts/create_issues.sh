#!/bin/bash
# 创建 GitHub Issues 脚本
# 使用前确保已配置 gh CLI: gh auth login

cd /root/.openclaw/workspace/oxo-call-main

# Issue #1: Cache O(1) Optimization
echo "Creating Issue #1..."
gh issue create \
  --title "[P0][Performance] Cache system O(n) → O(1) optimization" \
  --label "P0,performance,good first issue" \
  --body "## Problem
\`src/cache.rs\` uses JSONL linear scan, loading entire file on each lookup. With 1000 entries (~200KB), each lookup takes 5-20ms.

## Code Location
\`\`\`rust
// src/cache.rs:115-135
for line in content.lines() {
    if let Ok(entry) = serde_json::from_str::<CacheEntry>(line)
        && entry.hash == hash { ... }
}
\`\`\`

## Solution
1. Load all cache into \`DashMap<String, CacheEntry>\` at startup (O(1) lookup)
2. Async WAL persistence (write-ahead log, avoid full rewrite on each update)
3. LRU eviction with configurable max_entries (default: 10,000)

## Expected Improvement
- Cache lookup: 5ms → 0.05ms (100x faster)
- Better for batch processing and loop scripts

## Testing
\`\`\`bash
cargo bench -- cache_lookup
# Benchmark: 1000 lookups < 50ms
cargo test --lib cache
\`\`\`

**Complexity**: Medium  
**Est. Time**: 2-3 days

## Design Principles
- ⚡ **Efficiency**: Optimized for small models and high-frequency calls
- 🔧 **Modularity**: Cache module remains independent and replaceable"

# Issue #2: Batch Async
echo "Creating Issue #2..."
gh issue create \
  --title "[P0][Performance] Batch processing sync I/O → async" \
  --label "P0,performance,async" \
  --body "## Problem
\`src/runner/batch.rs\` uses \`spawn_blocking\` + \`std::process::Command\`, causing thread pool bottlenecks.

## Solution
1. Use \`tokio::process::Command\` for true async execution
2. Work-stealing scheduling with \`tokio::task::spawn\`
3. Keep semaphore concurrency control

## Expected Improvement
- Batch throughput: +50-100%
- Reduced thread context switching overhead

**Complexity**: Medium  
**Est. Time**: 1-2 days"

# Issue #3: HTTP Tuning
echo "Creating Issue #3..."
gh issue create \
  --title "[P0][Performance] HTTP client connection pool tuning" \
  --label "P0,performance,http" \
  --body "## Problem
\`src/llm/provider.rs\` uses \`reqwest::Client::new()\` default config without connection pool tuning.

## Solution
\`\`\`rust
client: reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .connect_timeout(Duration::from_secs(10))
    .pool_max_idle_per_host(16)
    .pool_idle_timeout(Duration::from_secs(300))
    .tcp_keepalive(Duration::from_secs(60))
    .build()?
\`\`\`

## Expected Improvement
- LLM call latency: -30%
- Better connection reuse, less TCP handshake overhead

**Complexity**: Low  
**Est. Time**: 0.5 day"

# Issue #4: Flag Validation
echo "Creating Issue #4..."
gh issue create \
  --title "[P0][Accuracy] Hard flag validation layer" \
  --label "P0,accuracy,llm,critical" \
  --body "## Problem
Current system relies on LLM self-discipline to not hallucinate flags. Small models (≤3B) still have hallucination risks.

## Solution
1. Parse \`ARGS\` after generation, extract all flags (\`--*\` and \`-?\` patterns)
2. Validate all flags against:
   - Skill file examples
   - Document-extracted flag catalog (\`StructuredDoc.flag_catalog\`)
3. On unknown flag: auto-retry with known flag subset only, or prompt user

## Expected Improvement
- Flag hallucination rate: -80%
- Significant user trust improvement

**Complexity**: Medium  
**Est. Time**: 2-3 days

## Design Principles
- 🎯 **Accuracy**: Multi-stage validation prevents hallucination
- 🔧 **Reliability**: Hard validation before execution"

# Issue #5: Orchestrator Integration
echo "Creating Issue #5..."
gh issue create \
  --title "[P1][Architecture] Orchestrator full integration" \
  --label "P1,architecture,multi-agent" \
  --body "## Problem
\`orchestrator/\` module has Supervisor/Planner/Executor/Validator implemented, but \`main.rs\` still mainly uses \`runner::Runner\`.

## Solution
1. Make \`supervisor.decide()\` the default execution entry
2. Implement full task decomposition in \`planner\` (pipeline detection + step planning)
3. Implement auto-repair suggestions in \`validator\`
4. Add \`--verbose\` for orchestration decision visualization

## Expected Improvement
- Complex task accuracy: +20%
- Multi-step pipeline automation
- Auto-diagnosis and repair suggestions on failure

**Complexity**: High  
**Est. Time**: 5-7 days

## Design Principles
- 🧠 **Intelligence**: Multi-agent coordination for complex tasks
- 🔍 **Transparency**: Full traceability of decision-making"

echo "Done! Created 5 core issues."
echo "Check them at: https://github.com/Traitome/oxo-call/issues"
