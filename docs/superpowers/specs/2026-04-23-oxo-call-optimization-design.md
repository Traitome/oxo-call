# oxo-call Systematic Optimization Design

**Date:** 2026-04-23
**Project:** oxo-call v0.12.1
**Goal:** Comprehensive performance-focused optimization and enhancement
**Output:** Prioritized backlog with rationale, ready for implementation planning

---

## Executive Summary

This design defines a systematic approach to review, optimize, and enhance oxo-call — a Rust-based bioinformatics CLI assistant with 70+ source files. The review prioritizes performance optimization across all metrics (LLM latency, local processing, memory, startup time) while also addressing reliability and architecture.

**Approach:** Hybrid methodology combining static analysis for quick wins, targeted profiling for verification, and architecture review for long-term improvements.

**Deliverable:** Prioritized backlog (P0–P3) with detailed findings, recommended fixes, and measurement plans.

---

## 1. Review Methodology

### Phase 1: Static Analysis (Quick Wins)

Scan all source files with focus on:

| Category | Detection Focus |
|----------|-----------------|
| **String handling** | Unnecessary `.clone()`, `.to_string()`, string allocations in hot loops |
| **Async patterns** | Blocking calls in async context, missing `.await` optimization, unnecessary `tokio::spawn` |
| **Collection operations** | Iteration patterns, O(n²) where O(n) possible, early termination opportunities |
| **Caching** | Missing LRU cache hits, redundant doc/skill fetching, stale data not reused |
| **Error handling** | `.unwrap()` and `.expect()` causing panic risk, error propagation overhead |

### Phase 2: Targeted Profiling (Verification)

Measure actual performance:

| Metric | Method |
|--------|--------|
| LLM accuracy | Use existing `oxo-bench` crate in `crates/oxo-bench` |
| Doc parsing | Add microbenchmarks for parsing latency |
| Skill matching | Benchmark lookup speed |
| Memory | Track heap allocations during typical command flow |
| Startup | Measure from invocation to first usable response |

### Phase 3: Architecture Review (Long-term)

Assess module boundaries:

| Area | Focus |
|------|-------|
| Coupling analysis | Modules that depend on too many others |
| Data flow clarity | Trace request → LLM → response path, identify unnecessary hops |
| Extensibility | Where would new providers or skill sources cause churn |

---

## 2. Target Files & Modules

### Phase 1 Priority Files

| Module | Files | Priority Reason |
|--------|-------|-----------------|
| **LLM Core** | `src/llm/mod.rs`, `src/llm/provider.rs`, `src/llm/prompt.rs`, `src/llm/streaming.rs` | Every command hits this — latency critical |
| **Orchestrator** | `src/orchestrator/mod.rs`, `src/orchestrator/planner.rs`, `src/orchestrator/executor.rs` | Decision logic, command routing — efficiency matters |
| **Runner** | `src/runner/mod.rs`, `src/runner/core.rs`, `src/runner/validation.rs` | Execution path — affects all operations |
| **Docs/Skills** | `src/docs.rs`, `src/skill.rs`, `src/cache.rs` | I/O heavy — caching and parsing efficiency |
| **Engine** | `src/engine.rs`, `src/generator.rs` | Core dispatch — request-to-command pipeline |

### Phase 2 Measurement Points

| Metric | Where to Measure |
|--------|------------------|
| LLM latency | `src/llm/provider.rs` — API call timing |
| Doc parsing | `src/doc_processor.rs`, `src/docs.rs` — parsing benchmarks |
| Skill matching | `src/skill.rs`, `src/index.rs` — lookup speed |
| Memory | `src/cache.rs` — LRU cache effectiveness |
| Startup | `src/main.rs`, `src/cli.rs` — initialization overhead |

### Phase 3 Coupling Analysis

| Area | Files to Review |
|------|-----------------|
| Config system | `src/config.rs`, `src/cli.rs` — how settings propagate |
| Knowledge system | `src/knowledge/mod.rs`, `src/knowledge/tool_knowledge.rs` — skill integration |
| Workflow engine | `src/workflow.rs`, `src/workflow_graph.rs` — pipeline complexity |
| MCP integration | `src/mcp.rs` — external protocol coupling |

---

## 3. Performance Pattern Detection Taxonomy

### String & Allocation Patterns

| Pattern | Detection | Impact |
|---------|-----------|--------|
| Unnecessary `.clone()` | String/struct cloned when reference suffices | Memory + CPU overhead |
| `.to_string()` on literals | `"text".to_string()` instead of `&str` | Heap allocation per call |
| String concatenation in loops | `s += x` repeated vs `String::push_str` or `format!` | Reallocs, O(n²) worst case |
| Owned types in function signatures | `String` param when `&str` works | Forces caller to allocate |

### Async & I/O Patterns

| Pattern | Detection | Impact |
|---------|-----------|--------|
| Blocking in async | `std::fs` in async context instead of `tokio::fs` | Blocks executor thread |
| Unnecessary `tokio::spawn` | Single task spawned when direct `.await` suffices | Scheduler overhead |
| Missing concurrent execution | Sequential `.await` when `futures::join!` possible | Latency aggregation |
| Repeated network calls | Same API/doc fetched multiple times per request | Network latency × N |

### Collection & Iteration Patterns

| Pattern | Detection | Impact |
|---------|-----------|--------|
| O(n²) lookups | Nested loops on same collection | Scales poorly for large inputs |
| Unbounded collection growth | Collection grows without limits | Memory exhaustion risk |
| `.collect()` on intermediate | Iterator chain with unnecessary collect | Allocation + iteration overhead |
| Missing early termination | Full iteration when first match suffices | Wasted cycles |

### Caching & Memory Patterns

| Pattern | Detection | Impact |
|---------|-----------|--------|
| Missing cache hit check | Cache checked but data still fetched | Redundant work |
| Cache key inefficiency | Complex key construction (hashing overhead) | Lookup latency |
| Large structs in cache | Entire objects cached vs needed subset | Memory waste |
| No cache invalidation strategy | Stale data served indefinitely | Accuracy degradation |

---

## 4. Backlog Output Format

### Finding Entry Template

```markdown
### [PRIORITY] Finding Title

**Location:** `src/path/to/file.rs:line_range`
**Category:** Performance / Reliability / Architecture
**Impact:** High / Medium / Low (with rationale)

**Problem:**
Concise description of the inefficiency or issue.

**Current Code:**
```rust
// Example of problematic pattern
```

**Recommended Fix:**
```rust
// Example of improved implementation
```

**Effort:** Hours estimated | Quick win / Requires design
**Dependencies:** Other findings that must be fixed first (if any)
**Verification:** How to measure improvement (benchmark, profiling)
```

### Priority Ranking Criteria

| Priority | Criteria |
|----------|----------|
| **P0 (Critical)** | Crashes, security vulnerabilities, data loss risk — must fix immediately |
| **P1 (High)** | >10% performance impact, affects every command, user-visible latency |
| **P2 (Medium)** | 1-10% performance impact, affects specific modules, maintainability concern |
| **P3 (Low)** | Minor optimization, code clarity improvement, future-proofing |

### Backlog Sections Structure

1. **P0–P1 Performance Findings** — actionable optimizations with clear impact
2. **P0–P1 Reliability/Security Findings** — unwraps, panics, error handling gaps
3. **P2–P3 Findings** — smaller improvements, deferred items
4. **Architecture Recommendations** — refactoring opportunities for maintainability
5. **Measurement Plan** — how to verify improvements after implementation

---

## 5. Review Process Workflow

```
Phase 1: Static Analysis (Primary effort)
├── Read each priority file (LLM → Orchestrator → Runner → Docs/Skills → Engine)
├── Apply pattern detection taxonomy
├── Document findings in backlog format
├── Rank by priority (P0–P3)
└── Output: Prioritized findings list

Phase 2: Targeted Profiling (Verification, optional)
├── Identify top 5 P0/P1 findings from Phase 1
├── Create microbenchmarks for affected code paths
├── Measure baseline performance
├── Validate impact estimates from static analysis
└── Output: Benchmark data confirming/adjusting priority

Phase 3: Architecture Review (Long-term)
├── Trace data flow: request → LLM → command → execution
├── Map module dependencies (imports analysis)
├── Identify tight coupling, unclear boundaries
├── Document refactoring recommendations
└── Output: Architecture improvement proposals
```

### Session Deliverables

1. **This design document** — `docs/superpowers/specs/2026-04-23-oxo-call-optimization-design.md`
2. **Prioritized backlog** — Embedded in design doc or separate file

### Next Steps

After user approval:
- Invoke `writing-plans` skill to create detailed implementation plan
- Plan converts backlog items into actionable tasks with dependencies
- User reviews plan → Implementation begins

---

## 6. Success Criteria

| Outcome | Measurement |
|---------|-------------|
| Comprehensive coverage | All 70+ source files reviewed for patterns |
| Prioritized backlog | Findings ranked P0–P3 with rationale |
| Actionable fixes | Each finding has recommended code change |
| Verification plan | Benchmarks or profiling methods specified |
| Architecture clarity | Module boundaries and data flow documented |

---

## Appendix: Project Context Summary

**Project:** oxo-call — AI-powered CLI assistant for bioinformatics
**Version:** 0.12.1 (Rust 2024 edition)
**Source Files:** 70+ across modules: `llm`, `orchestrator`, `runner`, `knowledge`, `execution`
**Skills:** 150+ bioinformatics tool skills (samtools, bwa, gatk, etc.)
**LLM Backends:** GitHub Copilot, OpenAI, Anthropic, Ollama
**Recent Commits:** Prompt engineering accuracy, Ollama API fixes, chat command, security audit (v0.12 release)