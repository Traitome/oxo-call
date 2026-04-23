# oxo-call Optimization Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Systematically review oxo-call codebase to produce a prioritized backlog of performance, reliability, and architecture findings.

**Architecture:** Three-phase approach: Phase 1 (Static Analysis) reads priority files and detects patterns → Phase 2 documents findings → Phase 3 prioritizes and outputs backlog. Each phase produces checkpoint deliverables.

**Tech Stack:** Rust 2024 edition, existing `oxo-bench` for validation

---

## File Structure

**Output Files:**
- `docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md` — Prioritized findings document

**Input Files (to be read, not modified):**
- Phase 1 Priority: `src/llm/*.rs`, `src/orchestrator/*.rs`, `src/runner/*.rs`, `src/docs.rs`, `src/skill.rs`, `src/cache.rs`, `src/engine.rs`, `src/generator.rs`
- Phase 1 Extended: `src/doc_processor.rs`, `src/index.rs`, `src/config.rs`, `src/cli.rs`, `src/main.rs`
- Phase 3: `src/knowledge/*.rs`, `src/workflow*.rs`, `src/mcp.rs`

---

## Phase 1: Static Analysis — LLM Core Module

### Task 1: Analyze LLM Module Entry Point

**Files:**
- Read: `src/llm/mod.rs`

- [ ] **Step 1: Read the LLM module entry file**

Read file and identify:
- Public exports and their signatures
- String types in function params (String vs &str)
- Async function patterns (blocking vs async)
- Any `.clone()` or `.to_string()` in exports
- Error handling patterns (unwrap/expect vs Result)

- [ ] **Step 2: Document findings for `src/llm/mod.rs`**

Create findings in backlog format:
```markdown
### [PRIORITY] Finding: [Title based on detected issue]

**Location:** `src/llm/mod.rs:[line_range]`
**Category:** Performance / Reliability / Architecture
**Impact:** High / Medium / Low

**Problem:**
[Specific issue found]

**Current Code:**
[Code snippet showing issue]

**Recommended Fix:**
[Code snippet showing fix]

**Effort:** [estimate] | Quick win / Requires design
**Dependencies:** None
**Verification:** [how to measure]
```

- [ ] **Step 3: Commit finding checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add LLM mod.rs findings checkpoint"
```

---

### Task 2: Analyze LLM Provider Module

**Files:**
- Read: `src/llm/provider.rs`

- [ ] **Step 1: Read LLM provider file (latency-critical path)**

Focus on:
- API call functions (network latency patterns)
- `.clone()` on request/response structs
- Blocking I/O in async functions (std::fs vs tokio::fs)
- Repeated network calls without caching
- Error handling (unwrap/expect in async)
- Timeout and retry patterns

- [ ] **Step 2: Document findings for `src/llm/provider.rs`**

For each detected issue, add finding to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add LLM provider.rs findings checkpoint"
```

---

### Task 3: Analyze LLM Prompt Module

**Files:**
- Read: `src/llm/prompt.rs`

- [ ] **Step 1: Read LLM prompt construction file**

Focus on:
- String building patterns (format! vs push_str)
- Large string allocations for prompts
- `.clone()` on prompt templates
- Unnecessary `.to_string()` on literals
- Template reuse patterns (cache vs rebuild)

- [ ] **Step 2: Document findings for `src/llm/prompt.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add LLM prompt.rs findings checkpoint"
```

---

### Task 4: Analyze LLM Streaming Module

**Files:**
- Read: `src/llm/streaming.rs`

- [ ] **Step 1: Read LLM streaming response handler**

Focus on:
- Stream processing efficiency
- Buffer management (grow vs fixed size)
- String parsing in streaming context
- Async iteration patterns
- Memory accumulation during stream

- [ ] **Step 2: Document findings for `src/llm/streaming.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add LLM streaming.rs findings checkpoint"
```

---

## Phase 1: Static Analysis — Orchestrator Module

### Task 5: Analyze Orchestrator Module Entry

**Files:**
- Read: `src/orchestrator/mod.rs`

- [ ] **Step 1: Read orchestrator module entry**

Focus on:
- Decision routing logic
- String matching for tool selection
- Collection iteration (nested loops)
- Early termination opportunities
- Error propagation patterns

- [ ] **Step 2: Document findings for `src/orchestrator/mod.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add orchestrator mod.rs findings checkpoint"
```

---

### Task 6: Analyze Orchestrator Planner

**Files:**
- Read: `src/orchestrator/planner.rs`

- [ ] **Step 1: Read planning/decision logic**

Focus on:
- Plan construction patterns
- Collection building (Vec growth)
- Clone on plan structs
- String operations in plan descriptions
- Iteration over plan steps

- [ ] **Step 2: Document findings for `src/orchestrator/planner.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add orchestrator planner.rs findings checkpoint"
```

---

### Task 7: Analyze Orchestrator Executor

**Files:**
- Read: `src/orchestrator/executor.rs`

- [ ] **Step 1: Read execution dispatch logic**

Focus on:
- Command execution patterns
- Async execution (sequential vs parallel)
- Error handling in execution context
- Result collection patterns
- State tracking efficiency

- [ ] **Step 2: Document findings for `src/orchestrator/executor.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add orchestrator executor.rs findings checkpoint"
```

---

## Phase 1: Static Analysis — Runner Module

### Task 8: Analyze Runner Module Entry

**Files:**
- Read: `src/runner/mod.rs`

- [ ] **Step 1: Read runner module entry**

Focus on:
- Run function signatures (String vs &str)
- Execution lifecycle patterns
- Result handling patterns
- Async execution wrapper patterns

- [ ] **Step 2: Document findings for `src/runner/mod.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add runner mod.rs findings checkpoint"
```

---

### Task 9: Analyze Runner Core

**Files:**
- Read: `src/runner/core.rs`

- [ ] **Step 1: Read core execution logic**

Focus on:
- Command construction patterns
- String allocation for commands
- subprocess spawn patterns
- Output parsing efficiency
- Buffer handling

- [ ] **Step 2: Document findings for `src/runner/core.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add runner core.rs findings checkpoint"
```

---

### Task 10: Analyze Runner Validation

**Files:**
- Read: `src/runner/validation.rs`

- [ ] **Step 1: Read validation logic**

Focus on:
- Validation iteration patterns
- String matching for validation
- Error collection patterns
- Early termination in validation loops

- [ ] **Step 2: Document findings for `src/runner/validation.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add runner validation.rs findings checkpoint"
```

---

## Phase 1: Static Analysis — Docs/Skills/Cache

### Task 11: Analyze Docs Module

**Files:**
- Read: `src/docs.rs`

- [ ] **Step 1: Read documentation fetching/processing**

Focus on:
- Doc fetching patterns (cache vs fetch)
- String parsing efficiency
- Large doc handling (memory)
- `.clone()` on doc structs
- Error handling in I/O

- [ ] **Step 2: Document findings for `src/docs.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add docs.rs findings checkpoint"
```

---

### Task 12: Analyze Skill Module

**Files:**
- Read: `src/skill.rs`

- [ ] **Step 1: Read skill matching/loading logic**

Focus on:
- Skill lookup patterns (O(n) vs O(n²))
- Skill struct cloning
- String matching for skill names
- Skill caching effectiveness
- Collection iteration patterns

- [ ] **Step 2: Document findings for `src/skill.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add skill.rs findings checkpoint"
```

---

### Task 13: Analyze Cache Module

**Files:**
- Read: `src/cache.rs`

- [ ] **Step 1: Read LRU cache implementation**

Focus on:
- Cache key construction efficiency
- Cache value sizes
- Cache hit/miss patterns
- Eviction strategy
- Memory bounds

- [ ] **Step 2: Document findings for `src/cache.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add cache.rs findings checkpoint"
```

---

### Task 14: Analyze Doc Processor

**Files:**
- Read: `src/doc_processor.rs`

- [ ] **Step 1: Read doc parsing logic**

Focus on:
- Parsing efficiency patterns
- String allocation during parsing
- Large text handling
- Regex vs manual parsing
- Memory during parse

- [ ] **Step 2: Document findings for `src/doc_processor.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add doc_processor.rs findings checkpoint"
```

---

### Task 15: Analyze Index Module

**Files:**
- Read: `src/index.rs`

- [ ] **Step 1: Read skill/tool index logic**

Focus on:
- Index lookup efficiency
- Index update patterns
- Collection growth patterns
- Search algorithm efficiency

- [ ] **Step 2: Document findings for `src/index.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add index.rs findings checkpoint"
```

---

## Phase 1: Static Analysis — Engine & Generator

### Task 16: Analyze Engine Module

**Files:**
- Read: `src/engine.rs`

- [ ] **Step 1: Read core dispatch engine**

Focus on:
- Request routing patterns
- Dispatch efficiency
- String matching for routing
- Collection iteration for matching
- Error handling in dispatch

- [ ] **Step 2: Document findings for `src/engine.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add engine.rs findings checkpoint"
```

---

### Task 17: Analyze Generator Module

**Files:**
- Read: `src/generator.rs`

- [ ] **Step 1: Read command generation logic**

Focus on:
- Command string building
- Template application efficiency
- String allocation patterns
- Clone on command structs
- Output formatting patterns

- [ ] **Step 2: Document findings for `src/generator.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add generator.rs findings checkpoint"
```

---

## Phase 1: Static Analysis — Config & CLI

### Task 18: Analyze Config Module

**Files:**
- Read: `src/config.rs`

- [ ] **Step 1: Read configuration handling**

Focus on:
- Config loading patterns (I/O blocking vs async)
- Config struct cloning
- String handling in config
- Validation iteration patterns
- Error handling patterns

- [ ] **Step 2: Document findings for `src/config.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add config.rs findings checkpoint"
```

---

### Task 19: Analyze CLI Module

**Files:**
- Read: `src/cli.rs`

- [ ] **Step 1: Read CLI argument handling**

Focus on:
- Argument parsing efficiency
- String allocation for args
- Validation patterns
- Command routing patterns

- [ ] **Step 2: Document findings for `src/cli.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add cli.rs findings checkpoint"
```

---

### Task 20: Analyze Main Entry Point

**Files:**
- Read: `src/main.rs`

- [ ] **Step 1: Read application entry point**

Focus on:
- Startup initialization patterns
- Lazy vs eager loading
- Initialization sequence
- Error handling at startup
- Memory allocation at startup

- [ ] **Step 2: Document findings for `src/main.rs`**

Add findings to backlog.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add main.rs findings checkpoint"
```

---

## Phase 2: Prioritization

### Task 21: Rank Findings by Priority

**Files:**
- Modify: `docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md`

- [ ] **Step 1: Review all findings collected**

Read through backlog and categorize each finding:

| Priority | Criteria |
|----------|----------|
| **P0 (Critical)** | Crashes, security vulnerabilities, data loss risk |
| **P1 (High)** | >10% performance impact, every command, user-visible latency |
| **P2 (Medium)** | 1-10% performance impact, specific modules, maintainability |
| **P3 (Low)** | Minor optimization, code clarity, future-proofing |

- [ ] **Step 2: Reorganize backlog by priority**

Reorder findings into sections:
1. P0–P1 Performance Findings
2. P0–P1 Reliability/Security Findings
3. P2–P3 Findings
4. Architecture Recommendations

- [ ] **Step 3: Add verification plans for top findings**

For P0/P1 findings, specify:
- Benchmark command to measure baseline
- Expected improvement metric
- Validation method

- [ ] **Step 4: Commit prioritized backlog**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: prioritize oxo-call optimization backlog"
```

---

## Phase 3: Architecture Review

### Task 22: Analyze Module Coupling

**Files:**
- Read: `src/lib.rs` and trace imports across modules

- [ ] **Step 1: Map module dependencies**

For each module, identify:
- Imports from other modules
- Circular dependencies
- Modules with excessive imports (>5)
- Tight coupling indicators

- [ ] **Step 2: Document architecture findings**

Add to backlog as Architecture Recommendations section:
- Coupling analysis summary
- Data flow diagram (text format)
- Refactoring opportunities

- [ ] **Step 3: Commit architecture checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add architecture analysis findings"
```

---

### Task 23: Analyze Knowledge Module

**Files:**
- Read: `src/knowledge/mod.rs`, `src/knowledge/tool_knowledge.rs`

- [ ] **Step 1: Read knowledge/skill integration**

Focus on:
- Knowledge lookup patterns
- Struct coupling with skill system
- Data flow between knowledge and other modules

- [ ] **Step 2: Document architecture findings**

Add to Architecture Recommendations.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add knowledge module architecture findings"
```

---

### Task 24: Analyze Workflow Engine

**Files:**
- Read: `src/workflow.rs`, `src/workflow_graph.rs`

- [ ] **Step 1: Read workflow pipeline logic**

Focus on:
- Pipeline complexity
- Module boundaries
- Extensibility patterns
- Data flow through workflow

- [ ] **Step 2: Document architecture findings**

Add to Architecture Recommendations.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add workflow engine architecture findings"
```

---

### Task 25: Analyze MCP Integration

**Files:**
- Read: `src/mcp.rs`

- [ ] **Step 1: Read MCP protocol integration**

Focus on:
- External protocol coupling
- Error handling for external calls
- Async patterns with external services
- Memory for protocol buffers

- [ ] **Step 2: Document architecture findings**

Add to Architecture Recommendations.

- [ ] **Step 3: Commit checkpoint**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: add MCP integration architecture findings"
```

---

## Final Deliverable

### Task 26: Create Backlog Summary

**Files:**
- Modify: `docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md`

- [ ] **Step 1: Add executive summary to backlog**

Add header section:
```markdown
# oxo-call Optimization Backlog

**Date:** 2026-04-23
**Total Findings:** [count]
**P0 Critical:** [count]
**P1 High:** [count]
**P2 Medium:** [count]
**P3 Low:** [count]
**Architecture Recommendations:** [count]

## Quick Wins (P1, <2 hours each)
[List top 5 P1 quick wins]

## Major Improvements (P0/P1, >2 hours)
[List major items requiring design]

## Deferred Items (P2/P3)
[Summary of deferred items]
```

- [ ] **Step 2: Add measurement plan**

For top 5 findings, specify:
```markdown
## Measurement Plan

### Finding X: [Title]
**Benchmark:** `oxo-bench eval --metric X`
**Baseline:** [expected current value]
**Target:** [improvement goal]
**Validation:** [how to confirm fix worked]
```

- [ ] **Step 3: Final commit**

```bash
git add docs/superpowers/backlog/2026-04-23-oxo-call-backlog.md
git commit -m "docs: complete oxo-call optimization backlog"

# Push to remote
git push origin main
```

---

## Self-Review Checklist

After completing the plan, verify:

- [ ] **Spec coverage:** All modules from spec have corresponding tasks
- [ ] **No placeholders:** No TBD/TODO/similar patterns
- [ ] **Type consistency:** Finding format consistent across all tasks
- [ ] **Deliverable clear:** Final output is prioritized backlog document