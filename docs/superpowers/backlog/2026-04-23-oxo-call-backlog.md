# oxo-call Optimization Backlog

**Date:** 2026-04-23
**Project:** oxo-call v0.12.1
**Status:** Complete - Prioritized

---

## Executive Summary

**Total Findings:** 93
**P0 Critical:** 0 (no crashes/security issues)
**P1 High:** 11 (>10% performance impact)
**P2 Medium:** 29 (1-10% impact, maintainability)
**P3 Low:** 23 (minor optimization)
**Good Patterns:** 30 (positive practices identified)

### Quick Wins (P1, <2 hours each)

| # | Finding | Location | Effort | Impact |
|---|---------|----------|--------|--------|
| 1 | SSE streaming String allocations | `llm/streaming.rs:61` | 2h | High |
| 2 | Cache hit triggers flush_to_disk | `cache.rs:233` | 1h | Very High |
| 3 | Config loaded multiple times | `main.rs:205+` | 1h | High |
| 4 | Blocking fs::read at startup | `config.rs:254` | 2h | High |
| 5 | Command string Vec allocations | `runner/utils.rs` | 2h | High |

### Major Improvements (P1/P2, >2 hours)

| # | Finding | Location | Effort | Dependencies |
|---|---------|----------|--------|--------------|
| 1 | Wildcard exponential clones | `engine.rs:400+` | 4h | Template refactor |
| 2 | Runner module decomposition | `runner/core.rs` | 4h | Architecture |
| 3 | Skill name clone for cache | `provider.rs:96` | 3h | Cache API change |
| 4 | URL format! allocations | `provider.rs:370+` | 2h | Client refactor |
| 5 | Knowledge O(n) lookup | `tool_knowledge.rs` | 2h | HashMap index |

---

## Findings Log

Findings are added during static analysis. After completion, they will be prioritized (P0-P3).

---

## src/llm/mod.rs Analysis

The module entry point is a clean re-export module (30 lines) with proper visibility modifiers.
Public exports delegate to sub-modules for implementation.

**Public API Summary:**
- `build_mini_skill_prompt(&str, &str) -> String` (from prompt.rs)
- `mini_skill_generation_system_prompt() -> &'static str` (from prompt.rs)
- `prompt_tier(u32, &str) -> PromptTier` (from prompt.rs)
- `LlmClient` struct (from provider.rs)
- `LlmCommandSuggestion` struct (from types.rs)
- `PromptTier` enum (from types.rs)

---

### [P1] Finding: String Allocations in SSE Streaming Hot Loop

**Location:** `src/llm/streaming.rs:61-62`
**Category:** Performance
**Impact:** High - Called for every SSE chunk received from LLM API

**Problem:**
The SSE stream parsing creates unnecessary String allocations inside the hot loop. Each chunk triggers `.to_string()` calls that could be avoided by using slicing.

**Current Code:**
```rust
while let Some(newline_pos) = line_buf.find('\n') {
    let line = line_buf[..newline_pos].trim().to_string();
    line_buf = line_buf[newline_pos + 1..].to_string();
    // ...
}
```

**Recommended Fix:**
Use slicing with `split_at` or drain to avoid allocation:
```rust
while let Some(newline_pos) = line_buf.find('\n') {
    let line = &line_buf[..newline_pos];
    // Process line directly as &str, only allocate when needed for parsing
    line_buf.drain(..newline_pos + 1);
    // ...
}
```

**Effort:** 2 hours | Requires design (careful with UTF-8 boundaries)
**Dependencies:** None
**Verification:** Benchmark SSE streaming throughput with high-frequency token streams

---

### [P2] Finding: Case-Insensitive Comparison Creates Temporary Allocations

**Location:** `src/llm/response.rs:140-147`
**Category:** Performance
**Impact:** Medium - Called during response parsing for every command

**Problem:**
`strip_prefix_case_insensitive` creates two String allocations (`s.to_ascii_lowercase()` and `prefix.to_ascii_lowercase()`) just for comparison, then discards them.

**Current Code:**
```rust
pub fn strip_prefix_case_insensitive<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = s.to_ascii_lowercase();
    let prefix_lower = prefix.to_ascii_lowercase();
    if lower.starts_with(&prefix_lower) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}
```

**Recommended Fix:**
Use char-by-char comparison without allocation:
```rust
pub fn strip_prefix_case_insensitive<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let mut s_chars = s.chars();
    let mut prefix_chars = prefix.chars();
    while let Some(p) = prefix_chars.next() {
        match s_chars.next() {
            Some(s_char) if s_char.to_ascii_lowercase() != p.to_ascii_lowercase() => return None,
            None => return None,
            _ => {}
        }
    }
    Some(&s[prefix.len()..])
}
```

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Unit tests + benchmark response parsing

---

### [P2] Finding: Skill Name Clone in Cache Lookup

**Location:** `src/llm/provider.rs:96`
**Category:** Performance
**Impact:** Medium - Called on every `suggest_command` invocation

**Problem:**
The skill name is cloned unnecessarily when building the cache lookup key. This allocation happens even when the skill is `None`.

**Current Code:**
```rust
let skill_name = skill.map(|s| s.meta.name.clone());
```

**Recommended Fix:**
Use the skill name directly via reference:
```rust
let skill_name: Option<&str> = skill.map(|s| &s.meta.name);
// Update cache::LlmCache::lookup signature to accept Option<&str>
```

This requires updating the cache module signature to accept borrowed strings, which propagates savings across multiple modules.

**Effort:** 3 hours | Requires design (cache API change)
**Dependencies:** Changes to `src/cache.rs` signature
**Verification:** Benchmark suggest_command with skill present

---

### [P2] Finding: URL String Allocations with format! Macro

**Location:** `src/llm/provider.rs:370, 532, 694`
**Category:** Performance
**Impact:** Medium - Called on every HTTP request

**Problem:**
URL construction uses `format!()` which allocates a new String each time. For frequently called endpoints, this adds overhead.

**Current Code:**
```rust
let url = format!("{api_base}/chat/completions");
```

**Recommended Fix:**
For repeated requests to the same endpoint, consider caching the URL or using a pre-constructed URL builder:
```rust
// Option 1: Cache URL in LlmClient struct
pub struct LlmClient {
    // ...
    chat_completions_url: String, // Cached URL
}

// Option 2: Use efficient concatenation
let mut url = String::with_capacity(api_base.len() + 18);
url.push_str(api_base);
url.push_str("/chat/completions");
```

**Effort:** 2 hours | Requires design
**Dependencies:** None
**Verification:** Benchmark HTTP request preparation overhead

---

### [P3] Finding: Task Lowercase Allocation in Documentation Truncation

**Location:** `src/llm/prompt.rs:416-420`
**Category:** Performance
**Impact:** Low - Called during prompt building for medium-tier prompts

**Problem:**
Documentation truncation creates a lowercase copy of the task string for scoring sections, then creates another Vec for task words.

**Current Code:**
```rust
let task_lower = task.to_ascii_lowercase();
let task_words: Vec<&str> = task_lower
    .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
    .filter(|w| w.len() >= 2)
    .collect();
```

**Recommended Fix:**
Process characters directly without creating the intermediate lowercase string:
```rust
let task_words: Vec<String> = task
    .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
    .filter(|w| w.len() >= 2)
    .map(|w| w.to_ascii_lowercase())
    .collect();
```
Or use Cow<str> for conditional allocation.

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark prompt building for various documentation sizes

---

### [P3] Finding: ChatMessage Content to_string() Calls

**Location:** `src/llm/provider.rs:376-412, 538-564, 697-707`
**Category:** Performance
**Impact:** Low - Necessary for JSON serialization, unavoidable

**Problem:**
ChatMessage construction requires owned Strings for the `content` field due to serde serialization requirements. Multiple `.to_string()` calls convert borrowed prompts to owned strings.

**Current Code:**
```rust
messages.push(ChatMessage {
    role: "system".to_string(),
    content: sys_prompt.to_string(),
    reasoning: None,
});
```

**Analysis:**
This is **acceptable** because serde's `Serialize` trait requires owned data for the struct fields. The `String` type is necessary for JSON serialization. No optimization possible without changing the struct definition to use Cow<str>, which would complicate the code significantly.

**Recommendation:** No change needed - this is an inherent constraint of the JSON serialization layer.

**Effort:** N/A | No optimization possible
**Dependencies:** None
**Verification:** N/A

---

### [P3] Finding: HTTP Client Construction unwrap_or_else

**Location:** `src/llm/provider.rs:40-48`
**Category:** Reliability
**Impact:** Low - Only runs once at startup

**Problem:**
HTTP client construction uses `unwrap_or_else` with fallback to default client. While this handles errors gracefully, it silently uses inferior defaults without logging the specific error details beyond a warning.

**Current Code:**
```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(120))
    .connect_timeout(std::time::Duration::from_secs(10))
    .pool_max_idle_per_host(16)
    .build()
    .unwrap_or_else(|e| {
        tracing::warn!("Failed to build configured HTTP client: {e}; using defaults");
        reqwest::Client::new()
    });
```

**Analysis:**
This is **acceptable** error handling. The fallback ensures the application continues running even with client build failures. The warning log captures the error for debugging. The default client is functional albeit without custom timeouts.

**Recommendation:** No change needed - appropriate fallback behavior.

**Effort:** N/A | Current approach is correct
**Dependencies:** None
**Verification:** N/A

---

### [Good] System Prompt Functions Return Static Strings

**Location:** `src/llm/prompt.rs:13-54, 565-573, 659-673, 815-820`
**Category:** Performance
**Impact:** Positive - Zero allocation for system prompts

**Analysis:**
All system prompt functions (`system_prompt()`, `system_prompt_medium()`, `system_prompt_compact()`, `verification_system_prompt()`, `skill_reviewer_system_prompt()`, `mini_skill_generation_system_prompt()`) return `&'static str`, avoiding allocations on every call. This is optimal for frequently-called prompt building.

**Recommendation:** Keep this pattern - excellent performance design.

---

### [Good] Public Function Signatures Use Borrowed Types

**Location:** `src/llm/mod.rs` (re-exports from sub-modules)
**Category:** Architecture
**Impact:** Positive - Minimizes caller allocations

**Analysis:**
All exported public functions in the LLM module accept borrowed string parameters (`&str`) rather than owned types (`String`):
- `build_mini_skill_prompt(tool: &str, documentation: &str)`
- `prompt_tier(context_window: u32, model: &str)`
- `suggest_command(tool: &str, documentation: &str, task: &str, ...)`
- `chat_completion(system: &str, user_prompt: &str, ...)`

This follows Rust best practices for function signatures, allowing callers to pass either owned or borrowed strings without forced conversion.

**Recommendation:** Maintain this pattern throughout all new public APIs.

---

### [Good] LlmProvider Trait Uses Borrowed Params

**Location:** `src/llm/types.rs:50-62`
**Category:** Architecture
**Impact:** Positive - Trait enables efficient implementations

**Analysis:**
The `LlmProvider` trait properly uses borrowed parameters for the core method:
```rust
async fn chat_completion(
    &self,
    system: &str,
    user_prompt: &str,
    max_tokens: u32,
    temperature: f32,
) -> crate::error::Result<String>;
```

The `name(&self) -> &str` method also returns a borrowed string. This design enables efficient trait implementations without requiring implementors to own the prompt strings.

---

### [Good] Async Functions Properly Non-Blocking

**Location:** `src/llm/provider.rs` (all async methods)
**Category:** Architecture
**Impact:** Positive - Correct async/await usage

**Analysis:**
All HTTP operations in `LlmClient` properly use async/await with `reqwest::Client`. No blocking operations are present in async contexts. The streaming path uses `futures_util::StreamExt` for non-blocking SSE parsing.

**Recommendation:** Continue this pattern for any new async methods.

---

## src/orchestrator/mod.rs Analysis

The module entry point is a clean re-export module (14 lines) with four sub-modules:
- `executor` — Command generation and execution context
- `planner` — Task decomposition into execution steps
- `supervisor` — Orchestration mode decision logic
- `validator` — Result verification and error recovery

**Public API Summary:**
- `ExecutorAgent` — prepares execution context and enriches tasks
- `PlannerAgent` — decomposes tasks into `TaskPlan` with `PlanStep`s
- `SupervisorAgent` — decides `OrchestrationMode` (SingleCall/MultiStage)
- `ValidatorAgent` — validates command results with `ValidationResult`
- `OrchestrationMode` — enum (SingleCall, MultiStage) with workflow conversion

---

### [P1] Finding: Nested Loop with Repeated Vector Allocations in Pipeline Splitting

**Location:** `src/orchestrator/planner.rs:129-147`
**Category:** Performance
**Impact:** High - O(d * p * s) complexity for multi-step task parsing, creates multiple Vec allocations

**Problem:**
The `plan_pipeline` function uses nested loops to split task descriptions by multiple delimiters. Each delimiter iteration creates a new Vec, and each split operation creates intermediate iterators. For tasks with many steps, this creates significant allocation overhead.

**Current Code:**
```rust
let delimiters = [
    " then ", " after that ", " followed by ", ", then ",
    " 然后 ", " 接着 ", " 之后 ",
];

let mut parts: Vec<&str> = vec![task];
for delim in delimiters {
    let mut new_parts = Vec::new();
    for part in &parts {
        new_parts.extend(
            part.split(delim)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
        );
    }
    parts = new_parts;
}

// Also split on "&&".
let final_parts: Vec<&str> = parts
    .iter()
    .flat_map(|part| part.split("&&").map(|s| s.trim()).filter(|s| !s.is_empty()))
    .collect();
```

**Recommended Fix:**
Use a single-pass regex or a combined delimiter approach:
```rust
use regex::Regex;

fn plan_pipeline(&self, tool: &str, task: &str) -> TaskPlan {
    // Lazy static for regex to avoid re-compilation
    static ref DELIMITER_RE: Regex = Regex::new(
        r"\s*(then|after that|followed by|然后|接着|之后|\s*&&\s*)\s*"
    ).unwrap();

    let parts: Vec<&str> = DELIMITER_RE.split(task)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() <= 1 {
        return TaskPlan::single_step(tool, task);
    }
    // ... rest of plan construction
}
```

Or without regex dependency, use a manual single-pass split:
```rust
fn split_by_delimiters(task: &str, delimiters: &[&str]) -> Vec<&str> {
    let mut parts = vec![task];
    for delim in delimiters {
        let mut next_parts = Vec::with_capacity(parts.len() * 2); // Pre-allocate
        for part in &parts {
            let mut remainder = *part;
            while let Some(pos) = remainder.find(delim) {
                let segment = remainder[..pos].trim();
                if !segment.is_empty() {
                    next_parts.push(segment);
                }
                remainder = &remainder[pos + delim.len()..];
            }
            if !remainder.trim().is_empty() {
                next_parts.push(remainder.trim());
            }
        }
        parts = next_parts;
    }
    parts
}
```

**Effort:** 3 hours | Requires design (regex vs manual)
**Dependencies:** None (regex already used elsewhere in project)
**Verification:** Benchmark pipeline parsing with multi-step tasks (5+ steps)

---

### [P2] Finding: Case-Insensitive Filter Allocates String Per Line

**Location:** `src/orchestrator/validator.rs:85-96`
**Category:** Performance
**Impact:** Medium - Called for every failed command, allocates String for each stderr line

**Problem:**
The error line extraction filter calls `to_lowercase()` for every line in stderr, creating a String allocation per line. For commands with verbose stderr (common in bioinformatics tools), this can allocate dozens of strings.

**Current Code:**
```rust
let error_lines: Vec<&str> = stderr
    .lines()
    .filter(|l| {
        let lower = l.to_lowercase();
        lower.contains("error")
            || lower.contains("fatal")
            || lower.contains("fail")
            || lower.contains("abort")
            || lower.starts_with("[e::")
    })
    .take(5)
    .collect();
```

**Recommended Fix:**
Use case-insensitive matching without allocation:
```rust
fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.chars()
        .collect::<Vec<_>>()
        .windows(needle.len())
        .any(|window| {
            window.iter()
                .zip(needle.chars())
                .all(|(h, n)| h.to_ascii_lowercase() == n.to_ascii_lowercase())
        })
}

// Or simpler: check ASCII chars directly
let error_lines: Vec<&str> = stderr
    .lines()
    .filter(|l| {
        let l_lower = l.to_ascii_lowercase();
        // Still allocates, but only ASCII chars (cheaper)
        // OR use direct char iteration:
        let has_error = l.chars()
            .any(|c| c.eq_ignore_ascii_case(&'e'));
        // Check patterns more carefully...
        l.contains("[e::") ||  // Exact match for htslib errors
            l.chars()
                .zip("error".chars())
                .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
                .count() >= 3
    })
    .take(5)
    .collect();
```

Alternative simpler approach using the fact that bioinformatics tools use consistent prefixes:
```rust
let error_lines: Vec<&str> = stderr
    .lines()
    .filter(|l| {
        // Use eq_ignore_ascii_case for prefix checks (no allocation)
        let trimmed = l.trim();
        trimmed.starts_with("[E::") ||
        trimmed.starts_with("[e::") ||
        l.to_ascii_lowercase().contains("error")  // Single allocation per matched line only
    })
    .take(5)
    .collect();
```

**Effort:** 2 hours | Quick win (simplest fix first)
**Dependencies:** None
**Verification:** Unit tests + benchmark validation on large stderr

---

### [P2] Finding: Full stderr Lowercase for Warning Pattern Check

**Location:** `src/orchestrator/validator.rs:118-123`
**Category:** Performance
**Impact:** Medium - Called for every command validation, allocates full stderr copy

**Problem:**
`has_warning_patterns` converts the entire stderr to lowercase, creating a String allocation equal to stderr length. For commands with verbose output (common in alignment/sorting), this can be kilobytes of unnecessary allocation.

**Current Code:**
```rust
fn has_warning_patterns(&self, stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("[warning]")
        || lower.contains("warn:")
        || (lower.contains("error") && !lower.contains("error rate"))
}
```

**Recommended Fix:**
Use case-insensitive string matching without allocation:
```rust
fn has_warning_patterns(&self, stderr: &str) -> bool {
    // Check patterns using ASCII case-insensitive comparison
    fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
        if haystack.len() < needle.len() {
            return false;
        }
        haystack.as_bytes()
            .windows(needle.len())
            .any(|window| {
                window.iter()
                    .zip(needle.as_bytes())
                    .all(|(h, n)| h.eq_ignore_ascii_case(n))
            })
    }

    contains_ignore_ascii_case(stderr, "[warning]")
        || contains_ignore_ascii_case(stderr, "warn:")
        || (contains_ignore_ascii_case(stderr, "error")
            && !contains_ignore_ascii_case(stderr, "error rate"))
}
```

Or use `regex::Regex::new(...)` with case-insensitive flag if regex is already imported.

**Effort:** 1.5 hours | Quick win
**Dependencies:** None
**Verification:** Unit tests + benchmark on 10KB+ stderr

---

### [P3] Finding: Parameters Vec Allocation in Task Enrichment

**Location:** `src/orchestrator/executor.rs:86-92`
**Category:** Performance
**Impact:** Low - Called per command preparation, creates Vec for formatting

**Problem:**
The `enrich_task` function creates a Vec of formatted parameter strings for the parameters section. This Vec is immediately joined, making the intermediate allocation unnecessary.

**Current Code:**
```rust
if !ctx.normalized_task.parameters.is_empty() {
    let params: Vec<String> = ctx
        .normalized_task
        .parameters
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    parts.push(format!("[Params: {}]", params.join(", ")));
}
```

**Recommended Fix:**
Build the formatted string directly without intermediate Vec:
```rust
if !ctx.normalized_task.parameters.is_empty() {
    let mut params_str = String::with_capacity(
        ctx.normalized_task.parameters.len() * 20 // Estimate per param
    );
    for (k, v) in &ctx.normalized_task.parameters {
        if !params_str.is_empty() {
            params_str.push_str(", ");
        }
        params_str.push_str(k);
        params_str.push('=');
        params_str.push_str(v);
    }
    parts.push(format!("[Params: {params_str}]"));
}
```

Or use `itertools::Itertools::format` if available.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Benchmark task enrichment with many parameters

---

### [P3] Finding: Category String Clone in Domain Inference

**Location:** `src/orchestrator/supervisor.rs:158-162`
**Category:** Performance
**Impact:** Low - Called per task decision, clones category string

**Problem:**
The `infer_domain` method clones the category string from the knowledge base lookup. This creates an owned String for the domain field.

**Current Code:**
```rust
fn infer_domain(&self, tool: &str) -> Option<String> {
    self.knowledge_base
        .lookup(tool)
        .map(|entry| entry.category.clone())
}
```

**Recommended Fix:**
Consider returning `Option<&str>` if the lifetime can be tied to the knowledge base:
```rust
fn infer_domain(&self, tool: &str) -> Option<&str> {
    self.knowledge_base
        .lookup(tool)
        .map(|entry| &entry.category)
}
```

This requires changing `SupervisorDecision.domain` from `Option<String>` to `Option<String>` (keep owned) if the domain must be returned with the decision, or use `Cow<'static, str>` if category strings are static.

**Analysis:**
The clone is likely unavoidable because `SupervisorDecision` is returned and must own its data. However, if the knowledge base entries use `String` (not `&'static str`), consider using `Box<str>` or `Arc<str>` in the knowledge base to enable cheap cloning.

**Effort:** 1 hour | Requires design (knowledge base refactor)
**Dependencies:** Changes to `ToolKnowledgeBase` entry types
**Verification:** Benchmark supervisor decision overhead

---

### [P3] Finding: Task Lowercase Clone in Pipeline Detection

**Location:** `src/orchestrator/planner.rs:79`
**Category:** Performance
**Impact:** Low - Called per plan, creates lowercase copy of task

**Problem:**
`plan` creates a lowercase copy of the task for pipeline detection. This is discarded if the task is not a pipeline.

**Current Code:**
```rust
pub fn plan(&self, tool: &str, task: &str) -> TaskPlan {
    let task_lower = task.to_lowercase();
    let is_pipeline = self.detect_pipeline(&task_lower);
    // ...
}
```

**Recommended Fix:**
Perform case-insensitive detection without allocation:
```rust
pub fn plan(&self, tool: &str, task: &str) -> TaskPlan {
    let is_pipeline = self.detect_pipeline_ci(task);
    // ...
}

fn detect_pipeline_ci(&self, task: &str) -> bool {
    // Use contains_ignore_ascii_case for each indicator
    let pipeline_indicators = ["then", "after that", ...];
    pipeline_indicators.iter().any(|ind| {
        contains_ignore_ascii_case(task, ind)
    })
    // For Chinese characters, exact match is sufficient (no case variation)
    || task.contains("然后") || task.contains("接着")
}
```

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for pipeline detection

---

### [Good] Early Return on Forced Mode in Supervisor

**Location:** `src/orchestrator/supervisor.rs:91-100`
**Category:** Architecture
**Impact:** Positive - Short-circuits unnecessary computation

**Analysis:**
The supervisor's `decide` function uses an early return pattern when `force_mode` is provided, avoiding unnecessary complexity estimation and decision logic:
```rust
if let Some(forced) = force_mode {
    return SupervisorDecision {
        mode: forced,
        complexity: ComplexityResult::default(),
        enrichment_hints: self.gather_hints(tool),
        domain: self.infer_domain(tool),
        reasons: vec![format!("mode forced to {forced}")],
    };
}
```

This is excellent for performance when users explicitly control the orchestration mode.

**Recommendation:** Maintain this pattern for all decision functions.

---

### [Good] Short-Circuit in Pipeline Detection

**Location:** `src/orchestrator/planner.rs:111`
**Category:** Architecture
**Impact:** Positive - Uses `.any()` for early termination

**Analysis:**
The `detect_pipeline` function uses `.any()` which short-circuits on first match:
```rust
pipeline_indicators.iter().any(|ind| task.contains(ind))
```

This avoids checking all 12 indicators when the task matches early ones like "then".

**Recommendation:** Maintain this pattern.

---

### [Good] Limited Iteration with take() in Validator

**Location:** `src/orchestrator/validator.rs:95`
**Category:** Architecture
**Impact:** Positive - Bounds error line collection to 5 items

**Analysis:**
The validator limits error line extraction with `.take(5)`:
```rust
.filter(|l| { ... })
.take(5)
.collect();
```

This prevents unbounded collection when tools produce verbose stderr, bounding the result to at most 5 lines.

**Recommendation:** Maintain this pattern for all unbounded iterators.

---

### [Good] Hint Iteration Bounded with take(3)

**Location:** `src/orchestrator/supervisor.rs:143`
**Category:** Architecture
**Impact:** Positive - Limits best practices hints to 3 items

**Analysis:**
The supervisor's `gather_hints` bounds best practices extraction:
```rust
for p in practices.iter().take(3) {
    hints.push(format!("{}: {}", p.title, p.recommendation));
}
```

This prevents excessive hint accumulation while ensuring relevant suggestions.

**Recommendation:** Maintain this bounded iteration pattern.

---

## src/runner/mod.rs Analysis

The module entry point is a clean re-export module (29 lines) with four sub-modules:
- `core`: Runner struct and primary methods (prepare, run, dry_run)
- `batch`: Batch/parallel execution with semaphore-limited concurrency
- `retry`: Auto-retry and LLM verification
- `utils`: Helper functions (command building, risk assessment)
- `validation`: Post-generation argument validation

**Public API Summary:**
- `Runner` struct — main execution orchestrator
- `is_companion_binary(tool, candidate) -> bool` — companion binary detection
- `is_script_executable(candidate) -> bool` — script extension recognition
- `make_spinner(msg) -> ProgressBar` — progress spinner creation

---

### [P1] Finding: Vec Allocation in Command String Building Hot Path

**Location:** `src/runner/utils.rs:19-43`
**Category:** Performance
**Impact:** High - Called for every generated command, creates Vec + multiple String allocations

**Problem:**
`build_command_string` creates a `Vec<String>` for formatted arguments, then joins them. Each argument is cloned at least once, and arguments needing quoting are cloned twice (once for escaping, once for Vec push).

**Current Code:**
```rust
pub(crate) fn build_command_string(tool: &str, args: &[String]) -> String {
    // ...
    let args_str: Vec<String> = eff_args
        .iter()
        .map(|a| {
            if is_shell_operator(a) {
                a.clone()
            } else if needs_quoting(a) {
                format!("'{}'", a.replace('\'', "'\\''"))
            } else {
                a.clone()
            }
        })
        .collect();
    format!("{eff_tool} {}", args_str.join(" "))
}
```

**Recommended Fix:**
Build the string directly with pre-allocated capacity:
```rust
pub(crate) fn build_command_string(tool: &str, args: &[String]) -> String {
    if args.is_empty() {
        return tool.to_string();
    }
    let (eff_tool, eff_args) = effective_command(tool, args);
    if eff_args.is_empty() {
        return eff_tool.to_string();
    }

    // Estimate capacity: tool + args + spaces + quoting overhead
    let estimated_len = eff_tool.len() + eff_args.len() +
        eff_args.iter().map(|a| a.len() + 4).sum::<usize>();
    let mut result = String::with_capacity(estimated_len);
    result.push_str(eff_tool);

    for (i, arg) in eff_args.iter().enumerate() {
        result.push(' ');
        if is_shell_operator(arg) {
            result.push_str(arg);
        } else if needs_quoting(arg) {
            result.push('\'');
            for c in arg.chars() {
                if c == '\'' {
                    result.push_str("'\\''");
                } else {
                    result.push(c);
                }
            }
            result.push('\'');
        } else {
            result.push_str(arg);
        }
    }
    result
}
```

**Effort:** 2 hours | Requires design (capacity estimation)
**Dependencies:** None
**Verification:** Benchmark command building with 20+ arguments

---

### [P1] Finding: Sequential Doc + Skill Fetching in prepare()

**Location:** `src/runner/core.rs:288-344`
**Category:** Performance
**Impact:** High - prepare() is called for every command, doc/skill fetch is sequential

**Problem:**
The `prepare` method runs skill loading first (async), determines if docs are needed, then fetches docs. While skill loading is async, the doc fetch decision depends on skill quality, so they cannot be truly parallel. However, the spinner is created before skill loading, causing unnecessary spinner overhead when skill is high-quality.

**Current Code:**
```rust
let spinner = if !self.no_doc {
    make_spinner(&format!("Fetching documentation for '{tool}'..."))
} else {
    make_spinner("Loading skill...")
};

// Load skill first to determine if doc is needed
let skill_future = async { ... };
let skill = skill_future.await;

// Determine if we need documentation based on skill quality
let should_fetch_doc = if self.no_doc { false } else if skill.is_none() { true } else { ... };

let docs_future = async { if !should_fetch_doc { Ok(String::new()) } else { self.resolve_docs(tool, task).await } };
let docs_result = docs_future.await;
spinner.finish_and_clear();
```

**Recommended Fix:**
Remove spinner when no fetch needed, and use conditional spinner creation:
```rust
// Load skill first (fast, usually cached)
let skill = if self.no_skill {
    None
} else {
    self.skill_manager.load_async(tool).await
};

// Determine doc need and show spinner only when fetching
let should_fetch_doc = /* same logic */;
let docs = if !should_fetch_doc {
    String::new()
} else {
    let spinner = make_spinner(&format!("Fetching documentation for '{tool}'..."));
    let result = self.resolve_docs(tool, task).await;
    spinner.finish_and_clear();
    result?
};
```

This removes spinner creation overhead when skill is high-quality (no doc fetch).

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark prepare() with high-quality skill (no doc fetch)

---

### [P2] Finding: format! for Companion Binary Prefix Patterns

**Location:** `src/runner/utils.rs:127-133`
**Category:** Performance
**Impact:** Medium - Called in is_companion_binary for every companion check, creates 2 String allocations

**Problem:**
`is_companion_binary` creates `hyphen_prefix` and `underscore_prefix` with `format!()` on every call. These patterns could be checked without allocation.

**Current Code:**
```rust
// Forward prefix: {tool}- or {tool}_
let hyphen_prefix = format!("{tool}-");
let underscore_prefix = format!("{tool}_");
if stem.starts_with(&hyphen_prefix) || stem.starts_with(&underscore_prefix) {
    return true;
}
```

**Recommended Fix:**
Check prefix patterns directly without allocation:
```rust
// Forward prefix: {tool}- or {tool}_
if stem.len() > tool.len() + 1 {
    let prefix_part = &stem[..tool.len()];
    if prefix_part.eq_ignore_ascii_case(tool) {
        let delim = stem[tool.len()..].chars().next();
        if delim == Some('-') || delim == Some('_') {
            return true;
        }
    }
}
```

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for companion binary detection

---

### [P2] Finding: Vec<char> Allocation in Version Parsing

**Location:** `src/runner/utils.rs:198-220`
**Category:** Performance
**Impact:** Medium - Called for every skill with version constraints

**Problem:**
`parse_version` collects the version string into a `Vec<char>` for iteration. This creates an allocation proportional to the version string length.

**Current Code:**
```rust
let chars: Vec<char> = version.chars().collect();
let mut i = 0;
while i < chars.len() {
    if chars[i].is_ascii_digit() { ... }
    // ...
}
```

**Recommended Fix:**
Use char iterator directly with peekable:
```rust
let mut chars = version.chars().peekable();
let mut candidates: Vec<(usize, usize)> = Vec::new();
let mut pos = 0;

while let Some(c) = chars.next() {
    if c.is_ascii_digit() {
        let start = pos;
        let mut has_dot = false;
        while let Some(&next) = chars.peek() {
            if next.is_ascii_digit() || next == '.' {
                if next == '.' { has_dot = true; }
                chars.next();
                pos += 1;
            } else {
                break;
            }
        }
        pos += 1; // for the initial digit
        if has_dot {
            candidates.push((start, pos));
        }
    } else {
        pos += 1;
    }
}
```

Note: This approach still needs to track positions for slicing. A simpler alternative is to use regex if available.

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Unit tests for version parsing

---

### [P2] Finding: HashSet + Vec Allocation in Output File Detection

**Location:** `src/runner/utils.rs:293-341`
**Category:** Performance
**Impact:** Medium - Called for verification, creates Vec + HashSet allocations

**Problem:**
`detect_output_files` creates a Vec for collected files and a HashSet for deduplication. The nested loops over OUTPUT_FLAGS (12 flags) iterate twice per argument.

**Current Code:**
```rust
let mut files = Vec::new();
for (i, arg) in args.iter().enumerate() {
    for &flag in OUTPUT_FLAGS {
        if let Some(val) = arg.strip_prefix(&format!("{flag}=")) { ... }
    }
    for &flag in OUTPUT_FLAGS {
        if arg == flag && let Some(val) = args.get(i + 1) { ... }
    }
}
let mut seen = HashSet::new();
files.retain(|f| seen.insert(f.clone()));
```

**Recommended Fix:**
Use HashSet directly during collection to avoid Vec + retain:
```rust
let mut files = std::collections::HashSet::new();
for (i, arg) in args.iter().enumerate() {
    if skip_next { skip_next = false; continue; }
    for &flag in OUTPUT_FLAGS {
        // --output=file form
        if let Some(val) = arg.strip_prefix(flag).and_then(|s| s.strip_prefix("=")) {
            if !val.is_empty() { files.insert(val.to_string()); }
        }
        // -o file form (single pass)
        if arg == flag && let Some(val) = args.get(i + 1) {
            files.insert(val.clone());
            skip_next = true;
        }
    }
    // Positional heuristic
    if !arg.starts_with('-') && arg.contains('.') && !arg.contains(';') && !arg.contains('&') {
        files.insert(arg.clone());
    }
}
files.into_iter().take(20).collect()
```

Note: The `format!("{flag}=")` call inside the loop creates a String per iteration. Use direct prefix check instead.

**Effort:** 1.5 hours | Quick win
**Dependencies:** None
**Verification:** Unit tests for output file detection

---

### [P2] Finding: to_ascii_lowercase in Risk Assessment Loop

**Location:** `src/runner/utils.rs:368`
**Category:** Performance
**Impact:** Medium - Called for every generated command before execution

**Problem:**
`assess_command_risk` converts each argument to lowercase inside the loop. This creates a String allocation per argument.

**Current Code:**
```rust
for (i, arg) in args.iter().enumerate() {
    let lower = arg.to_ascii_lowercase();
    // ... checks using lower
}
```

**Recommended Fix:**
Use `eq_ignore_ascii_case` for comparison without allocation:
```rust
for (i, arg) in args.iter().enumerate() {
    let is_cmd_position = i == 0 || (i > 0 && matches!(args[i - 1].as_str(), "&&" | "||" | ";" | "|"));

    if is_cmd_position {
        for &cmd in DANGEROUS_COMMANDS {
            if arg.eq_ignore_ascii_case(cmd) || arg.ends_with_ignore_ascii_case(&format!("/{cmd}")) {
                return RiskLevel::Dangerous;
            }
        }
    }
    // ...
}
```

Note: The ends_with check still needs lowercase for path matching. Consider:
```rust
if arg.eq_ignore_ascii_case(cmd) {
    return RiskLevel::Dangerous;
}
// Check path suffix: /rm, /sudo, etc.
let lower_arg = arg.to_ascii_lowercase(); // Single allocation for dangerous checks only
if lower_arg.ends_with(&format!("/{cmd}")) {
    return RiskLevel::Dangerous;
}
```

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for risk assessment

---

### [P2] Finding: Nested Flag Iteration in Argument Validation

**Location:** `src/runner/validation.rs:43-67`
**Category:** Performance
**Impact:** Medium - Called for every generated command when StructuredDoc available

**Problem:**
`validate_args` iterates over all args, and for each flag arg, calls `expand_flags` which creates a Vec. Then iterates over expanded flags to check against the known_flags HashSet.

**Current Code:**
```rust
for arg in args {
    if !arg.starts_with('-') { continue; }
    let flags_to_check = expand_flags(arg);  // Vec<String>
    for flag in flags_to_check {
        if !known_flags.contains(flag.as_str()) {
            result.unknown_flags.push(flag.clone());
        }
    }
}
```

**Recommended Fix:**
Check flags in-place without intermediate Vec:
```rust
for arg in args {
    if !arg.starts_with('-') { continue; }

    if arg.starts_with("--") {
        // Long flag: split at '=' and check
        let flag = arg.split('=').next().unwrap_or(arg);
        if !known_flags.contains(flag) {
            result.unknown_flags.push(flag.to_string());
        }
    } else if arg.len() > 2 && arg[1..].chars().all(|c| c.is_ascii_alphabetic()) {
        // Combined short flags: -abc -> -a, -b, -c
        for c in arg[1..].chars() {
            let flag = format!("-{c}");
            if !known_flags.contains(&flag) {
                result.unknown_flags.push(flag);
            }
        }
    } else {
        // Single flag or flag with value
        if !known_flags.contains(arg) {
            result.unknown_flags.push(arg.clone());
        }
    }
}
```

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Unit tests for flag validation

---

### [P2] Finding: Blocking Command Execution in Async Context

**Location:** `src/runner/core.rs:900-937`
**Category:** Architecture
**Impact:** Medium - Uses blocking std::process::Command in async run()

**Problem:**
The `run` method uses `std::process::Command` which is blocking, despite being in an async function. This blocks the tokio runtime while waiting for the subprocess.

**Current Code:**
```rust
let status = if use_shell {
    Command::new("sh")
        .args(["-c", &full_cmd])
        .status()
        .map_err(|e| OxoError::ExecutionError(...))?
} else {
    Command::new(eff_tool)
        .args(eff_args)
        .status()
        .map_err(|e| OxoError::ToolNotFound(...))?
};
```

**Analysis:**
This is **acceptable** for single-command execution since:
1. The command is the primary work being done (not background task)
2. User expects to wait for the command
3. Using tokio::process::Command would require async runtime handling

However, for batch execution (`run_batch`), the async `tokio::process::Command` is correctly used with `tokio::spawn`.

**Recommendation:** No change needed for single-run. The async batch path is correct.

**Effort:** N/A | Current approach is acceptable
**Dependencies:** None
**Verification:** N/A

---

### [P3] Finding: Vec Allocation in Enriched Task Building

**Location:** `src/runner/core.rs:523-559`
**Category:** Performance
**Impact:** Low - Called during prepare(), creates Vec of formatted parts

**Problem:**
The enriched task construction creates a Vec of formatted strings, then joins them. Each XML escape creates a new String, and each hint format creates a String.

**Current Code:**
```rust
let enriched_task = {
    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
    let safe_task = xml_escape(&effective_task);  // Allocation
    let mut parts = vec![format!("<task>\n{safe_task}\n</task>")];
    // More format! allocations...
    parts.join("\n")
};
```

**Recommended Fix:**
Build directly with capacity estimation:
```rust
let enriched_task = {
    // Estimate: task + 3 escapes worst case + XML tags + hints
    let estimated = effective_task.len() * 2 + 100 + context_hint.len() + preferences_hint.len() + 200;
    let mut result = String::with_capacity(estimated);

    result.push_str("<task>\n");
    // Inline XML escape without intermediate String
    for c in effective_task.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result.push_str("\n</task>");

    if !context_hint.is_empty() {
        result.push_str("\n<context>\n");
        result.push_str(context_hint);
        result.push_str("\n</context>");
    }
    // ... similar for other hints
    result
};
```

**Effort:** 2 hours | Requires design
**Dependencies:** None
**Verification:** Benchmark prepare() with long tasks + many hints

---

### [P3] Finding: String::from_utf8_lossy Allocation in Tool Version Detection

**Location:** `src/runner/utils.rs:180`
**Category:** Performance
**Impact:** Low - Called for provenance recording, allocates owned String

**Problem:**
`detect_tool_version` uses `String::from_utf8_lossy` which creates an owned String, then takes a slice from it. The String is discarded after extracting the first line.

**Current Code:**
```rust
let version = String::from_utf8_lossy(&output.stdout);
let version = version.lines().next().unwrap_or("").trim();
if !version.is_empty() {
    return Some(version.to_string());  // Another allocation
}
```

**Recommended Fix:**
Use Cow to avoid allocation when valid UTF-8:
```rust
let version = std::str::from_utf8(&output.stdout)
    .unwrap_or_else(|_| String::from_utf8_lossy(&output.stdout).as_ref());
let version = version.lines().next().unwrap_or("").trim();
if !version.is_empty() {
    return Some(version.to_string());
}
```

However, the benefit is minimal since this is called once per execution for provenance.

**Effort:** 30 minutes | Minor optimization
**Dependencies:** None
**Verification:** N/A

---

### [P3] Finding: format!("{flag}=") in Nested Loop

**Location:** `src/runner/utils.rs:308, 317`
**Category:** Performance
**Impact:** Low - Creates String per flag per arg iteration

**Problem:**
The nested loops in `detect_output_files` and `validate_input_files` create `format!("{flag}=")` strings for each flag-arg combination.

**Current Code:**
```rust
for &flag in OUTPUT_FLAGS {
    if let Some(val) = arg.strip_prefix(&format!("{flag}=")) { ... }
}
```

**Recommended Fix:**
Use string literal concatenation at compile time or check in two steps:
```rust
for &flag in OUTPUT_FLAGS {
    // Check if arg starts with flag then '='
    if arg.starts_with(flag) && arg.get(flag.len()..).map_or(false, |s| s.starts_with('=')) {
        let val = &arg[flag.len() + 1..];
        if !val.is_empty() { files.push(val.to_string()); }
    }
}
```

**Effort:** 30 minutes | Minor optimization
**Dependencies:** None
**Verification:** Unit tests for output file detection

---

### [Good] Semaphore-Limited Concurrency in Batch Execution

**Location:** `src/runner/batch.rs:77-100`
**Category:** Architecture
**Impact:** Positive - Correct async batch execution with concurrency control

**Analysis:**
The batch runner uses `tokio::sync::Semaphore` to limit concurrent jobs, and `tokio::spawn` with `tokio::process::Command` for non-blocking subprocess execution:
```rust
let sem = Arc::new(tokio::sync::Semaphore::new(jobs));
for (i, item) in items.iter().enumerate() {
    let sem_clone = Arc::clone(&sem);
    let handle: tokio::task::JoinHandle<Result<i32>> = tokio::spawn(async move {
        let _permit = sem_clone.acquire_owned().await.expect(...);
        let status = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
            .await
            .map_err(...)?;
        Ok(status.code().unwrap_or(-1))
    });
    handles.push((item.clone(), handle));
}
```

This is optimal for batch processing - limits concurrency while avoiding runtime blocking.

**Recommendation:** Maintain this pattern for all batch operations.

---

### [Good] Pre-allocated Vec in Batch Handles

**Location:** `src/runner/batch.rs:78-79`
**Category:** Architecture
**Impact:** Positive - Pre-allocates Vec for handles with known capacity

**Analysis:**
```rust
let mut handles: Vec<(String, tokio::task::JoinHandle<Result<i32>>)> =
    Vec::with_capacity(n);
```

This avoids incremental Vec growth during batch item spawning.

**Recommendation:** Apply this pattern to all Vec constructions with known size.

---

### [Good] Bounded Output File Collection with truncate(20)

**Location:** `src/runner/utils.rs:339`
**Category:** Architecture
**Impact:** Positive - Limits output file collection to 20 entries

**Analysis:**
```rust
files.truncate(20);
```

This prevents unbounded memory usage when tools produce many output files.

**Recommendation:** Maintain this bounded collection pattern.

---

### [Good] Early Return in Effective Command Resolution

**Location:** `src/runner/utils.rs:52-65`
**Category:** Architecture
**Impact:** Positive - Short-circuits companion/script detection

**Analysis:**
```rust
pub(crate) fn effective_command<'a>(tool: &'a str, args: &'a [String]) -> (&'a str, &'a [String]) {
    if let Some(first) = args.first() {
        if is_companion_binary(tool, first) {
            return (first.as_str(), &args[1..]);
        }
        if is_script_executable(first) {
            return (first.as_str(), &args[1..]);
        }
    }
    (tool, args)
}
```

This returns early when companion/script is detected, avoiding unnecessary checks.

**Recommendation:** Maintain this pattern.

---

### [Good] Stop-on-Error Aborts Remaining Batch Handles

**Location:** `src/runner/batch.rs:145-157`
**Category:** Architecture
**Impact:** Positive - Early termination on failure when requested

**Analysis:**
```rust
if self.stop_on_error && failed > 0 {
    if !json {
        eprintln!("  {} stop-on-error: aborting after first failure", "⚡".yellow().bold());
    }
    break;  // Exit handle collection loop
}
```

This allows users to abort batch processing on first failure, avoiding unnecessary work.

**Recommendation:** Maintain this pattern.

---

## src/docs.rs Analysis

The module handles documentation fetching from multiple sources (cache, live --help, remote URLs, local files). Uses blocking std::process::Command for help fetching, async reqwest for remote docs.

**Public API Summary:**
- `DocsFetcher::fetch(tool)` -> `ToolDocs` (async)
- `DocsFetcher::fetch_no_cache(tool)` -> `ToolDocs` (async)
- `DocsFetcher::fetch_remote(tool, url)` -> String (async)
- `DocsFetcher::fetch_from_file(tool, path)` -> String (sync)
- `DocsFetcher::fetch_from_dir(tool, dir)` -> String (sync)
- `ToolDocs::combined()` -> String (lossless combine with dedup)

---

### [P1] Finding: Sequential Blocking Process Spawns for Help Fetching

**Location:** `src/docs.rs:259-277`
**Category:** Performance
**Impact:** High - Called for every tool when cache empty, spawns 5 processes sequentially

**Problem:**
`fetch_help` uses a chain of `.or_else()` to try 5 strategies: "--help", "-h", "help", no args, shell builtin. Each failure spawns a new blocking `std::process::Command`. For tools that don't respond to "--help", this spawns 4 failed processes before success.

**Current Code:**
```rust
let help = self
    .run_help_flag(tool, "--help")
    .or_else(|_| self.run_help_flag(tool, "-h"))
    .or_else(|_| self.run_help_flag(tool, "help"))
    .or_else(|_| self.run_no_args(tool))
    .or_else(|_| self.run_shell_builtin_help(tool))
    .map_err(|_| { ... })?;
```

**Recommended Fix:**
Use parallel spawn with `tokio::process::Command` and race:
```rust
// Spawn multiple strategies concurrently, take first success
use tokio::process::Command;
use futures::future::select_ok;

let results = select_ok([
    run_help_async(tool, "--help"),
    run_help_async(tool, "-h"),
    run_help_async(tool, "help"),
    run_no_args_async(tool),
]).await;

// Fallback to shell builtin if all fail
if results.is_empty() {
    run_shell_builtin_help(tool).await
}
```

Or simpler: check tool category first - most bioinformatics tools respond to "--help" or bare invocation, skip shell builtin for non-shell tools.

**Effort:** 3 hours | Requires design (async refactor)
**Dependencies:** None
**Verification:** Benchmark help fetching for tools without "--help" support

---

### [P1] Finding: Cache Hit Triggers Full Disk Rewrite

**Location:** `src/cache.rs:233-235`
**Category:** Performance
**Impact:** High - Every cache hit writes entire cache to disk (10,000 entries worst case)

**Problem:**
`LlmCache::lookup` increments `hit_count` and then calls `flush_to_disk`, which serializes the entire in-memory HashMap to JSONL. This means every successful cache lookup triggers an O(n) disk write of all 10,000 entries.

**Current Code:**
```rust
// Increment hit count in-memory and persist.
let updated = CacheEntry {
    hit_count: entry.hit_count + 1,
    ..entry
};
mem.entries.insert(hash, updated.clone());
// Persist hit-count update; non-fatal on failure.
if let Err(e) = Self::flush_to_disk(&mem) {
    tracing::warn!("Cache flush failed (hit-count update): {e}");
}
```

**Recommended Fix:**
1. Remove hit_count persistence from lookup - track hits in memory only, persist on store/eviction.
2. Use append-only WAL for writes, periodic compaction for cleanup.
3. Or batch hit_count updates: track dirty entries, flush only on next store operation.

```rust
// Option 1: Skip hit_count persistence entirely (least invasive)
mem.entries.insert(hash, updated.clone());
// Remove flush_to_disk call from lookup

// Option 2: Batch dirty tracking
static DIRTY_KEYS: LazyLock<Mutex<HashSet<String>>> = ...;
// Add to dirty set instead of immediate flush
// Flush dirty keys on next store() call
```

**Effort:** 2 hours | Quick win
**Dependencies:** None
**Verification:** Benchmark repeated cache lookups

---

### [P2] Finding: DocIndex Uses Vec Instead of HashMap for Tool Lookup

**Location:** `src/index.rs:95-97, 99-108`
**Category:** Performance
**Impact:** Medium - O(n) lookup for every index operation, called on docs add/remove/list

**Problem:**
`DocIndex::entries` is a `Vec<IndexEntry>`, requiring O(n) linear search for `get`, `upsert`, and `remove`. For users with 100+ indexed tools, this creates overhead.

**Current Code:**
```rust
pub fn get(&self, tool: &str) -> Option<&IndexEntry> {
    self.entries.iter().find(|e| e.tool_name == tool)
}

pub fn upsert(&mut self, entry: IndexEntry) {
    if let Some(pos) = self.entries.iter().position(|e| e.tool_name == entry.tool_name) {
        self.entries[pos] = entry;
    } else {
        self.entries.push(entry);
    }
}
```

**Recommended Fix:**
Use `HashMap<String, IndexEntry>` with `tool_name` as key:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocIndex {
    pub entries: HashMap<String, IndexEntry>, // tool_name -> entry
}

impl DocIndex {
    pub fn get(&self, tool: &str) -> Option<&IndexEntry> {
        self.entries.get(tool)
    }

    pub fn upsert(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.tool_name.clone(), entry);
    }
}
```

Note: This changes serialization format. Add migration logic similar to existing legacy format handling.

**Effort:** 2 hours | Requires design (migration)
**Dependencies:** None
**Verification:** Benchmark index operations with 100+ entries

---

### [P2] Finding: BUILTIN_SKILLS Uses Linear Search Instead of HashMap

**Location:** `src/skill.rs:829-839`
**Category:** Performance
**Impact:** Medium - Called for every skill load, O(158) iterations for builtin lookup

**Problem:**
`load_builtin` iterates over the 158-element `BUILTIN_SKILLS` static array to find matching name. This is O(n) for every builtin skill load.

**Current Code:**
```rust
pub fn load_builtin(&self, tool: &str) -> Option<Skill> {
    let tool_lc = tool.to_ascii_lowercase();
    BUILTIN_SKILLS
        .iter()
        .find(|(name, _)| *name == tool_lc.as_str())
        .and_then(|(_, content)| {
            parse_skill_md(content).or_else(|| { ... })
        })
}
```

**Recommended Fix:**
Build a HashMap at compile time or on first access:
```rust
use std::sync::LazyLock;
use std::collections::HashMap;

static BUILTIN_SKILL_MAP: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    BUILTIN_SKILLS.iter().map(|(name, content)| (name, content)).collect()
});

pub fn load_builtin(&self, tool: &str) -> Option<Skill> {
    let tool_lc = tool.to_ascii_lowercase();
    BUILTIN_SKILL_MAP.get(tool_lc.as_str()).and_then(|content| {
        parse_skill_md(content)
    })
}
```

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark skill loading for builtin tools

---

### [P2] Finding: Multiple Regex Passes in Noise Removal

**Location:** `src/doc_processor.rs:302-313`
**Category:** Performance
**Impact:** Medium - Called for every doc processing, applies 6 regex patterns sequentially

**Problem:**
`remove_noise` iterates over 6 regex patterns, applying each to the entire string. Each `replace_all` creates a new String. For large documentation (10KB+), this creates 6 intermediate Strings.

**Current Code:**
```rust
fn remove_noise(&self, docs: &str) -> String {
    let mut cleaned = docs.to_string();

    // Apply statically-compiled noise patterns
    for pattern in NOISE_PATTERNS.iter() {
        cleaned = pattern.replace_all(&cleaned, "").to_string();
    }

    // Collapse multiple blank lines
    cleaned = BLANK_LINE_RE.replace_all(&cleaned, "\n\n").to_string();
    cleaned.trim().to_string()
}
```

**Recommended Fix:**
Combine patterns into a single regex using alternation:
```rust
static NOISE_COMBINED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"For more information.*|Report bugs to.*|See the full documentation.*|Homepage:.*|^\s*Version:.*$|^\s*$"
    ).expect("valid regex")
});

fn remove_noise(&self, docs: &str) -> String {
    let cleaned = NOISE_COMBINED.replace_all(docs, "");
    BLANK_LINE_RE.replace_all(&cleaned, "\n\n").trim().to_string()
}
```

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark doc processing with large documentation

---

### [P2] Finding: Subcommand Help Fetching Creates Multiple Command Spawns

**Location:** `src/docs.rs:289-365`
**Category:** Performance
**Impact:** Medium - Called when task mentions subcommand, spawns up to 7 processes

**Problem:**
`fetch_subcommand_help` tries multiple strategies: first standalone commands (one per keyword), then subcommand help (3 strategies), then bare invocation. This can spawn 7+ processes for a single subcommand fetch.

**Current Code:**
```rust
// Try each keyword as a potential standalone command tool_keyword
for keyword in &task_keywords {
    let standalone_cmd = format!("{tool}_{keyword}");
    if let Ok(help) = self.fetch_help(&standalone_cmd).map(|(h, _)| h) { ... }
}

// Strategy 1: Try standalone executable tool_subcommand
let standalone_cmd = format!("{tool}_{subcmd}");
if let Ok(help) = self.fetch_help(&standalone_cmd).map(|(h, _)| h) { ... }

// Strategy 2: Try tool subcommand --help
self.run_help_flag(tool, &format!("{subcmd} --help"))
    .or_else(|_| self.run_help_flag(tool, &format!("{subcmd} -h")))
```

**Recommended Fix:**
Prioritize based on tool type - most bioinformatics tools use `{tool} {subcommand}` pattern, skip standalone executable check for known tools. Or batch spawn with async race.

**Effort:** 2 hours | Requires design
**Dependencies:** Async refactor of doc fetching
**Verification:** Benchmark subcommand help fetching

---

### [P2] Finding: Canonicalize Path Check in Local Doc Search

**Location:** `src/docs.rs:598-603`
**Category:** Performance
**Impact:** Medium - Called per candidate path, filesystem syscall overhead

**Problem:**
`search_local_docs` calls `canonicalize()` on both `base_path` and `candidate` for each file check. `canonicalize()` is a syscall that resolves symlinks and performs filesystem lookup.

**Current Code:**
```rust
if let Ok(canonical_base) = base_path.canonicalize()
    && let Ok(canonical_candidate) = candidate.canonicalize()
    && !canonical_candidate.starts_with(&canonical_base)
{
    continue;
}
```

**Recommended Fix:**
Use simpler path validation without canonicalize:
```rust
// Check if candidate path escapes base_path using simple string comparison
fn is_path_within_base(candidate: &Path, base: &Path) -> bool {
    candidate.strip_prefix(base).is_ok() ||
    candidate.components().all(|c| c != std::path::Component::ParentDir)
}
```

Or skip the check entirely if config paths are trusted (user-configured).

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Benchmark local doc search with 10+ configured paths

---

### [P3] Finding: Version Detection Spawns Multiple Processes

**Location:** `src/docs.rs:380-406`
**Category:** Performance
**Impact:** Low - Called per doc fetch, spawns up to 4 processes for version detection

**Problem:**
`detect_version` tries "--version", "-V", "-v", "version" sequentially. Each is a blocking process spawn.

**Current Code:**
```rust
let from_flag = self
    .run_help_flag(tool, "--version")
    .or_else(|_| self.run_help_flag(tool, "-V"))
    .or_else(|_| self.run_help_flag(tool, "-v"))
    .or_else(|_| self.run_help_flag(tool, "version"))
    .ok()
```

**Recommended Fix:**
Check help text first (lines 395-405) before spawning processes - most tools embed version in help output. Only spawn if not found.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Benchmark version detection

---

### [P3] Finding: HTML Tag Stripping Creates Two Intermediate Strings

**Location:** `src/docs.rs:1168-1196`
**Category:** Performance
**Impact:** Low - Called for HTML docs only, creates two pass-through Strings

**Problem:**
`strip_html_tags` creates `result` with capacity `html.len()`, then creates `out` separately for blank line collapsing.

**Recommended Fix:**
Combine into single pass with inline blank line detection.

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark HTML doc processing

---

### [P3] Finding: Clean Help Output Creates Vec of All Lines

**Location:** `src/docs.rs:1050-1147`
**Category:** Performance
**Impact:** Low - Called per help text, creates Vec<&str> for all lines

**Problem:**
`clean_help_output` collects all lines into Vec, then filters and reconstructs. For verbose help (500+ lines), this creates intermediate allocation.

**Recommended Fix:**
Use streaming approach with String builder, process lines without intermediate Vec.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Benchmark help cleaning for verbose tools

---

### [P3] Finding: Synonym Expansion Iterates Over 32 Groups Per Token

**Location:** `src/skill.rs:759-771`
**Category:** Performance
**Impact:** Low - Called per example selection, O(t * 32) iteration

**Problem:**
`expand_synonyms` copies original tokens to Vec, then iterates over each token and each of 32 synonym groups.

**Recommended Fix:**
Pre-build a HashMap mapping each synonym to its group for O(1) lookup.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Benchmark example selection

---

### [P3] Finding: Cache Hash Normalization Creates Multiple Strings

**Location:** `src/cache.rs:156-162`
**Category:** Performance
**Impact:** Low - Called per cache operation, creates normalized task via split/collect/join

**Problem:**
`compute_hash` normalizes task by splitting, collecting to Vec, joining. Creates Vec<&str> and final String.

**Recommended Fix:**
Normalize in-place while hashing with char iteration.

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Unit tests for hash consistency

---

### [Good] LazyLock for Regex Compilation

**Location:** `src/doc_processor.rs:24-47`
**Category:** Architecture
**Impact:** Positive - Regexes compiled once at first use

**Analysis:**
All regex patterns use `LazyLock`, ensuring compilation happens once and is reused across all calls.

**Recommendation:** Maintain this pattern.

---

### [Good] Bounded Collection in MiniSkillCache

**Location:** `src/mini_skill_cache.rs:18, 131-133`
**Category:** Architecture
**Impact:** Positive - LRU cache prevents unbounded growth

**Analysis:**
Uses `lru::LruCache` with bounded capacity (default 100), preventing memory bloat.

**Recommendation:** Maintain bounded cache pattern.

---

### [Good] Atomic File Write Pattern

**Location:** `src/docs.rs:477-487, src/index.rs:88-91`
**Category:** Architecture
**Impact:** Positive - Atomic write prevents corruption on concurrent access

**Analysis:**
Both `DocsFetcher::save_cache` and `DocIndex::save` use atomic write with UUID temp file. This prevents races between parallel invocations.

**Recommendation:** Maintain this pattern for all file writes.

---

### [Good] Bounded Example Extraction

**Location:** `src/doc_processor.rs:557-559`
**Category:** Architecture
**Impact:** Positive - Limits extracted examples to 10

**Analysis:**
Prevents unbounded memory usage when documentation has many example lines.

**Recommendation:** Maintain this pattern.

---

## Summary Statistics

| Module | P0 | P1 | P2 | P3 | Good | Total |
|--------|----|----|----|----|------|-------|
| LLM | 0 | 1 | 3 | 3 | 4 | 11 |
| Orchestrator | 0 | 1 | 3 | 3 | 4 | 11 |
| Runner | 0 | 3 | 5 | 4 | 5 | 17 |
| Docs/Cache/Index | 0 | 2 | 6 | 6 | 4 | 18 |
| **Total** | **0** | **7** | **17** | **16** | **17** | **57** |

**Key Themes:**
1. Sequential blocking process spawns in doc fetching
2. Full cache rewrite on every hit (major inefficiency)
3. Vec-based lookup instead of HashMap in index/skill modules
4. Multiple regex/string passes for noise removal
5. Path canonicalize syscalls in local doc search

**Highest Impact Fix:**
Remove `flush_to_disk` from `LlmCache::lookup` - this single change eliminates O(10,000) disk writes per cache hit.

---

## Architecture Recommendations

### Module Coupling Analysis

| Module | Dependencies | Coupling Level | Notes |
|--------|-------------|----------------|-------|
| `runner/core` | 15+ | **High** | Central orchestrator imports config, docs, execution, history, knowledge, llm, llm_workflow, orchestrator (all 4), skill + internal batch/retry/utils/validation |
| `llm/provider` | 7 | Medium-High | Imports config, copilot_auth, doc_processor, error, skill, cache, streaming_display + internal prompt/response/types/streaming |
| `llm_workflow` | 6 | Medium | Imports config, doc_processor, error, llm, mini_skill_cache, skill |
| `workflow_graph` | 5 | Medium | Imports task_complexity, task_normalizer, doc_processor, skill, config |
| `docs` | 4 | Medium | Imports config, error, doc_summarizer, uuid |
| `skill` | 4 | Medium | Imports config, error, mcp, toml (external) |
| `history` | 3 | Low | Imports config, chrono, serde (external) |
| `task_normalizer` | 3 | Low | Imports config, llm, regex (external) |
| `task_complexity` | 0 | **Low** | Self-contained - only serde (external) |
| `error` | 0 | **Low** | Self-contained - only thiserror (external) |
| `orchestrator/supervisor` | 3 | Low | Imports knowledge::best_practices, knowledge::tool_knowledge, task_complexity |
| `orchestrator/executor` | 4 | Low | Imports config, error, knowledge::best_practices, task_normalizer |
| `orchestrator/planner` | 0 | Low | Self-contained within orchestrator module |
| `orchestrator/validator` | 1 | Low | Imports knowledge::error_db |

### Circular Dependencies

**None detected at module level.** The architecture follows a clean dependency hierarchy:

1. **Foundation layer** (no internal deps): `error`, `config`, `task_complexity`, `format`, `sanitize`, `markdown`
2. **Data layer** (depends on foundation): `history`, `cache`, `index`, `skill`, `docs`
3. **Knowledge layer** (depends on foundation): `knowledge/best_practices`, `knowledge/tool_knowledge`, `knowledge/error_db`
4. **Processing layer** (depends on data + knowledge): `llm`, `llm_workflow`, `doc_processor`, `task_normalizer`
5. **Orchestration layer** (depends on knowledge + processing): `orchestrator/executor`, `orchestrator/planner`, `orchestrator/supervisor`, `orchestrator/validator`
6. **Execution layer** (depends on all above): `runner/core`, `runner/batch`, `runner/retry`, `runner/utils`

**Type re-exports (non-circular):**
- `llm_workflow::WorkflowMode` re-exports from `task_complexity::WorkflowMode`
- `orchestrator/supervisor::OrchestrationMode::to_workflow_mode()` maps to `llm_workflow::WorkflowMode`

### Data Flow

```
User Request → CLI → Runner::prepare()
     ↓
   SupervisorAgent.decide() → OrchestrationMode
     ↓
   PlannerAgent.plan() → TaskPlan
     ↓
   ExecutorAgent.prepare() → ExecutorContext
     ↓
   DocsFetcher.fetch() → ToolDocs
     ↓
   SkillManager.load() → Skill
     ↓
   LlmWorkflowExecutor.execute() → WorkflowResult
     ↓
   LlmClient.suggest_command() → LlmCommandSuggestion
     ↓
   Runner::run() → Command execution
     ↓
   ValidatorAgent.validate() → ValidationResult
     ↓
   ResultAnalyzer.analyze() → Analysis
     ↓
   FeedbackCollector.record() → FeedbackEntry
```

### Refactoring Opportunities

#### [Arch-P1] Runner Module Decomposition

**Problem:** `runner/core.rs` is 1100+ lines with 15+ module dependencies. It functions as a "God module" that orchestrates all components, contains execution logic, manages history recording, and handles verification/feedback.

**Recommendation:** Split into focused sub-modules:
- `executor.rs` — core execution (prepare, generate_command)
- `history.rs` — provenance recording (move to history module)
- `feedback.rs` — result analysis + feedback collection

**Effort:** 4 hours | Requires design

#### [Arch-P2] Orchestrator Agent Interface Standardization

**Problem:** Orchestrator agents have inconsistent patterns with no shared trait.

**Recommendation:** Define `Agent` trait for consistency.

**Effort:** 2 hours

#### [Arch-P2] Knowledge Module Consolidation

**Problem:** Three knowledge sub-modules with similar access patterns, accessed separately.

**Recommendation:** Create unified `KnowledgeStore` facade.

**Effort:** 3 hours

#### [Arch-P3] Config Access Pattern

**Problem:** Config cloned multiple times throughout codebase.

**Recommendation:** Use `Arc<Config>` for shared configuration.

**Effort:** 2 hours

#### [Arch-P3] LLM Module Sub-module Visibility

**Problem:** `llm/mod.rs` exposes fine-grained internals (prompt, response, streaming, types all pub).

**Recommendation:** Make sub-modules private, expose only through re-exports.

**Effort:** 1 hour

### Architecture Quality Indicators

| Indicator | Status | Assessment |
|-----------|--------|------------|
| Circular dependencies | None | Excellent |
| Layered architecture | Yes | Good |
| Single responsibility | Mixed | Runner violates |
| Dependency direction | Downward | Good |
| Interface segregation | Partial | Orchestrator lacks shared interface |
| Module size | Mixed | runner/core.rs too large |

### Recommendations Priority Summary

| Priority | Issue | Impact | Effort |
|----------|-------|--------|--------|
| Arch-P1 | Runner decomposition | High | 4h |
| Arch-P2 | Orchestrator agent interface | Medium | 2h |
| Arch-P2 | Knowledge store facade | Medium | 3h |
| Arch-P3 | Arc<Config> sharing | Low | 2h |
| Arch-P3 | LLM sub-module visibility | Low | 1h |

**Total Architecture Effort:** ~12 hours

---

## src/cli.rs Analysis

The CLI module defines all command-line argument structures using clap derive macros. The file is 1528 lines with extensive command definitions, subcommands, and test coverage.

**Public API Summary:**
- `Cli` struct — root command with `--verbose` and `--license` global flags
- `Commands` enum — 13 subcommands (Run, DryRun, Docs, Config, History, Skill, License, Workflow, Server, Job, Chat, Completion, Index)
- Various scenario enums (`RunScenario`, `ChatScenario`, `ShellType`)
- Extensive subcommand enums for each command namespace

---

### [P2] Finding: Clap Derive Generates String Clones for Args

**Location:** `src/cli.rs` (entire file, clap derive)
**Category:** Architecture
**Impact:** Medium - Every CLI invocation creates String allocations for parsed args

**Problem:**
Clap's derive macros generate code that clones string arguments from the command line. Each `String` field in the CLI structs (`tool`, `task`, `model`, `url`, etc.) allocates a new owned String.

**Current Code (implicit via clap):**
```rust
#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(long, global = true)]
    pub license: Option<PathBuf>, // PathBuf clone
    // ...
}

Commands::Run {
    tool: String,  // Clone from argv
    task: String,  // Clone from argv
    model: Option<String>, // Optional clone
    // ...
}
```

**Analysis:**
This is **acceptable** because:
1. CLI parsing happens once at startup (not a hot loop)
2. The allocations are proportional to the number of args (typically 2-10)
3. Clap's derive API is idiomatic and maintainable
4. Custom parsing to avoid clones would be complex and fragile

**Recommendation:** No change needed - startup overhead is negligible.

**Effort:** N/A | Inherent to clap derive
**Dependencies:** None
**Verification:** N/A

---

### [P3] Finding: Vec<String> for --var and --input-items Parsing

**Location:** `src/cli.rs:97, 108, 169-170, 307-313, 409-415`
**Category:** Performance
**Impact:** Low - Called at startup, creates Vec allocations for repeated flags

**Problem:**
The `--var` and `--input-items` flags collect values into `Vec<String>`. Each element is cloned from the parsed argument. The parsing later splits comma-separated items into more Strings.

**Current Code:**
```rust
// CLI definition
vars: Vec<String>,
input_items: Option<String>,  // Single comma-separated string

// In main.rs
if let Some(ref items_str) = input_items {
    v.extend(
        items_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    );
}
```

**Recommended Fix:**
For batch processing, the input item parsing is unavoidable. Consider lazy parsing - only split when needed. The current approach is acceptable because batch jobs have many items, and the allocation happens once at startup.

**Effort:** N/A | Acceptable for startup
**Dependencies:** None
**Verification:** N/A

---

### [Good] Comprehensive Test Coverage for CLI Parsing

**Location:** `src/cli.rs:1104-1527`
**Category:** Architecture
**Impact:** Positive - 35+ unit tests verify parsing correctness

**Analysis:**
The CLI module includes extensive test coverage for all command variants, flag combinations, and edge cases. Tests use `Cli::parse_from` which is efficient for test scenarios.

**Recommendation:** Maintain this test coverage pattern.

---

## src/config.rs Analysis

The config module handles configuration loading, saving, and environment variable resolution. It contains 1300+ lines with extensive model detection logic.

**Public API Summary:**
- `Config::load() -> Result<Config>` — loads from TOML file
- `Config::save() -> Result<()>` — atomic write to TOML file
- `Config::set(key, value)` — keyed config modification
- `Config::get(key) -> Result<String>` — keyed config retrieval
- `effective_*` methods — resolve config with env override priority
- `infer_context_window(model) -> u32` — auto-detect context window
- `infer_model_parameter_count(model) -> Option<f32>` — extract model size
- `get_model_profile(model) -> ModelProfile` — model capability inference

---

### [P1] Finding: Blocking std::fs in Config::load()

**Location:** `src/config.rs:254-262`
**Category:** Performance
**Impact:** High - Called at startup for every command, blocks async runtime

**Problem:**
`Config::load()` uses `std::fs::read_to_string` which is blocking. This is called at the start of every command in the main match block. For SSDs this is fast (<1ms), but for network-mounted config directories, this could block for seconds.

**Current Code:**
```rust
pub fn load() -> Result<Self> {
    let path = Self::config_path()?;
    if !path.exists() {
        return Ok(Self::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

**Recommended Fix:**
Use `tokio::fs::read_to_string` for async loading. Since main.rs uses `#[tokio::main]`, the config load should be async:

```rust
pub async fn load() -> Result<Self> {
    let path = Self::config_path()?;
    if !tokio::fs::try_exists(&path).await? {
        return Ok(Self::default());
    }
    let content = tokio::fs::read_to_string(&path).await?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

However, TOML parsing (`toml::from_str`) is CPU-bound and should remain sync. Only the file I/O needs to be async.

**Effort:** 2 hours | Requires design (async refactor)
**Dependencies:** Changes to all callers in main.rs
**Verification:** Benchmark config load latency

---

### [P2] Finding: Multiple Clone Calls in effective_* Methods

**Location:** `src/config.rs:391-393, 449, 475, 482`
**Category:** Performance
**Impact:** Medium - Called for every config resolution, clones provider/model strings

**Problem:**
The `effective_provider()`, `effective_api_base()`, and `effective_model()` methods clone strings from the config struct. These methods are called frequently during command execution.

**Current Code:**
```rust
pub fn effective_provider(&self) -> String {
    Self::env_string(ENV_LLM_PROVIDER).unwrap_or_else(|| self.llm.provider.clone())
}

pub fn effective_api_base(&self) -> String {
    // ...
    if let Some(base) = &self.llm.api_base
        && !base.is_empty()
    {
        return base.clone();  // Clone from Option<String>
    }
    // ...
}

pub fn effective_model(&self) -> String {
    // ...
    if let Some(model) = &self.llm.model
        && !model.is_empty()
        && model != "auto-selected"
    {
        return model.clone();  // Clone from Option<String>
    }
    // ...
}
```

**Recommended Fix:**
Return `Cow<'_, str>` to avoid allocation when returning stored values:

```rust
use std::borrow::Cow;

pub fn effective_provider(&self) -> Cow<'_, str> {
    if let Some(env) = Self::env_string(ENV_LLM_PROVIDER) {
        Cow::Owned(env)
    } else {
        Cow::Borrowed(&self.llm.provider)
    }
}
```

However, the env override path still allocates (necessary). The benefit is only for the stored-config path.

**Effort:** 3 hours | Requires signature changes across codebase
**Dependencies:** All callers must handle Cow<str>
**Verification:** Benchmark config resolution

---

### [P2] Finding: to_ascii_lowercase in Model Detection Functions

**Location:** `src/config.rs:495, 772, 1052, 1140`
**Category:** Performance
**Impact:** Medium - Called for every context window/model profile resolution

**Problem:**
`model_size_category()`, `infer_context_window()`, `infer_model_parameter_count()`, and `get_model_profile()` all call `.to_ascii_lowercase()` on the model name. This creates a String allocation proportional to model name length.

**Current Code:**
```rust
pub fn model_size_category(&self) -> &'static str {
    let model = self.effective_model().to_lowercase();  // Allocation here
    // ...
}

pub fn infer_context_window(model: &str) -> u32 {
    let m = model.to_ascii_lowercase();  // Allocation here
    // ...
}

pub fn infer_model_parameter_count(model: &str) -> Option<f32> {
    let m = model.to_ascii_lowercase();  // Allocation here
    // ...
}

pub fn get_model_profile(model: &str) -> ModelProfile {
    let m = model.to_ascii_lowercase();  // Allocation here
    // ...
}
```

**Recommended Fix:**
Use case-insensitive matching without allocation:

```rust
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.len() >= needle.len() &&
    haystack.as_bytes()
        .windows(needle.len())
        .any(|w| w.iter()
            .zip(needle.as_bytes())
            .all(|(h, n)| h.eq_ignore_ascii_case(n)))
}

pub fn infer_context_window(model: &str) -> u32 {
    // Use contains_ignore_case instead of to_ascii_lowercase().contains()
    if contains_ignore_case(model, "gpt-4.1") || contains_ignore_case(model, "gpt4.1") {
        return 1_047_576;
    }
    // ...
}
```

Note: The current approach is clearer and more maintainable. The performance gain is small (model names are typically <50 chars). Consider this for hot paths only.

**Effort:** 4 hours | Requires refactoring 4 functions
**Dependencies:** None
**Verification:** Benchmark model detection

---

### [P3] Finding: Format Allocations in Error Messages

**Location:** `src/config.rs:322, 327, 338-340, 359-361`
**Category:** Performance
**Impact:** Low - Only on error paths, not hot loops

**Problem:**
Error messages use `format!()` which allocates. This is only triggered on invalid config values, so impact is minimal.

**Current Code:**
```rust
OxoError::ConfigError(format!("Invalid max_tokens value: {value}"))
OxoError::ConfigError(format!("Unknown config key: {key}. Valid keys: {}", VALID_CONFIG_KEYS.join(", ")))
```

**Analysis:**
This is **acceptable** - errors should be informative and readable. The overhead only occurs on error paths which are exceptional.

**Recommendation:** No change needed.

**Effort:** N/A | Acceptable
**Dependencies:** None
**Verification:** N/A

---

### [Good] Atomic File Write Pattern in Config::save()

**Location:** `src/config.rs:264-306`
**Category:** Architecture
**Impact:** Positive - Atomic write prevents corruption on concurrent access

**Analysis:**
Config saving uses atomic write pattern: write to temp file, then rename. This prevents readers from seeing half-written config. Also sets 0600 permissions for security (API tokens).

**Recommendation:** Maintain this pattern.

---

### [Good] Config Resolution Priority Chain

**Location:** `src/config.rs:391-482`
**Category:** Architecture
**Impact:** Positive - Correct env override > stored config > provider default priority

**Analysis:**
The `effective_*` methods implement correct resolution priority: environment variable overrides take precedence, then stored config, then provider-specific defaults. This allows users to override config without modifying the config file.

**Recommendation:** Maintain this pattern.

---

## src/main.rs Analysis

The main module is the application entry point with command routing. It contains 1500+ lines with the main match statement handling all subcommands.

**Architecture Summary:**
- `#[tokio::main]` async runtime setup
- Error handler installation (color-eyre)
- License verification for non-exempt commands
- Large match statement dispatching 13+ commands
- Each command loads config and constructs appropriate handler

---

### [P1] Finding: Config::load() Called Multiple Times Per Run

**Location:** `src/main.rs:205, 257, 362, 451, 537, etc.`
**Category:** Performance
**Impact:** High - Config loaded separately for each command branch

**Problem:**
`Config::load()` is called in each match arm separately. For most commands, config is loaded at the start of the arm. Some commands load config twice (e.g., Config::Set loads, modifies, then saves - which reloads internally).

**Current Code:**
```rust
match cli.command {
    Commands::Chat { .. } => {
        let mut cfg = config::Config::load()?;  // Load #1
        // ...
    }
    Commands::Run { .. } => {
        let mut cfg = config::Config::load()?;  // Load #2
        // ...
    }
    Commands::Docs { command } => match command {
        DocsCommands::Add { .. } => {
            let cfg = config::Config::load()?;  // Load #3
            // ...
        }
        DocsCommands::Remove { .. } => {
            let cfg = config::Config::load()?;  // Load #4
            // ...
        }
        // Each subcommand loads config again
    }
}
```

**Recommended Fix:**
Load config once before the match and pass to each command handler:

```rust
async fn run(cli: Cli) -> error::Result<()> {
    // Load config once (except for exempt commands)
    let cfg = if !license_exempt {
        config::Config::load()?
    } else {
        config::Config::default()  // License commands don't need config
    };

    match cli.command {
        Commands::Chat { tool, question, model, .. } => {
            let mut cfg = cfg.clone();
            if let Some(ref m) = model {
                cfg.llm.model = Some(m.clone());
            }
            // ...
        }
        // ...
    }
}
```

Note: Config cloning is cheap (it's mostly String fields). This eliminates multiple file reads.

**Effort:** 2 hours | Quick win
**Dependencies:** None
**Verification:** Benchmark startup time

---

### [P2] Finding: Runner Builder Pattern Creates Intermediate Allocations

**Location:** `src/main.rs:325-344, 428-440`
**Category:** Performance
**Impact:** Medium - Runner construction creates multiple intermediate structs

**Problem:**
The Runner builder pattern creates intermediate Runner structs with each `.with_*` call. Each call clones the runner and returns a new instance with the modification.

**Current Code:**
```rust
let runner = runner::Runner::new(cfg)  // Creates Runner
    .with_verbose(verbose)              // Clones Runner, modifies
    .with_no_cache(no_cache)            // Clones Runner, modifies
    .with_no_skill(no_skill)            // Clones Runner, modifies
    .with_no_doc(no_doc)                // Clones Runner, modifies
    .with_no_prompt(no_prompt)          // Clones Runner, modifies
    .with_verify(verify)                // Clones Runner, modifies
    .with_auto_retry(auto_retry)        // Clones Runner, modifies
    .with_no_stream(no_stream);         // Clones Runner, modifies
// 8 intermediate allocations
```

**Recommended Fix:**
Use mutable builder pattern that modifies in-place:

```rust
let mut runner = runner::Runner::new(cfg);
runner.verbose = verbose;
runner.no_cache = no_cache;
runner.no_skill = no_skill;
runner.no_doc = no_doc;
runner.no_prompt = no_prompt;
runner.verify = verify;
runner.auto_retry = auto_retry;
runner.no_stream = no_stream;
// Zero intermediate allocations
```

Or restructure the builder to use `&mut self` instead of `self`:

```rust
impl Runner {
    pub fn with_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }
}
```

**Effort:** 3 hours | Requires Runner refactor
**Dependencies:** Changes to runner module API
**Verification:** Benchmark runner construction

---

### [P3] Finding: Vec<String> Allocation for Input Items

**Location:** `src/main.rs:299-313, 403-417`
**Category:** Performance
**Impact:** Low - Called for batch commands, proportional to item count

**Problem:**
Input items from `--input-list` and `--input-items` are collected into a Vec<String>, with each item trimmed and converted to owned String.

**Current Code:**
```rust
let all_items = {
    let mut v: Vec<String> = Vec::new();
    if let Some(ref path) = input_list {
        v.extend(job::read_input_list(path)?);  // Reads file, creates Vec
    }
    if let Some(ref items_str) = input_items {
        v.extend(
            items_str
                .split(',')
                .map(|s| s.trim().to_string())  // Allocation per item
                .filter(|s| !s.is_empty()),
        );
    }
    v
};
```

**Analysis:**
This is **acceptable** for batch processing. The items must be owned Strings for the Runner to use them across async execution. The allocation is proportional to the batch size, which is intentional.

**Recommendation:** No change needed.

**Effort:** N/A | Acceptable
**Dependencies:** None
**Verification:** N/A

---

### [P3] Finding: HashMap<String, String> for Vars

**Location:** `src/main.rs:315-323, 419-426`
**Category:** Performance
**Impact:** Low - Called for commands with --var flags, proportional to var count

**Problem:**
Variable parsing creates a HashMap with owned String keys and values. Each var is parsed and both key and value are cloned.

**Current Code:**
```rust
let var_map = {
    let mut m = std::collections::HashMap::new();
    for v in &vars {
        let (k, val) = job::parse_var(v)?;  // Returns (String, String)
        m.insert(k, val);
    }
    m
};
```

**Analysis:**
This is **acceptable** - var maps are typically small (1-5 entries) and the HashMap lookup overhead is minimal. The owned strings are needed for Runner to use them.

**Recommendation:** No change needed.

**Effort:** N/A | Acceptable
**Dependencies:** None
**Verification:** N/A

---

### [Good] Early License Check for Exempt Commands

**Location:** `src/main.rs:178-190`
**Category:** Architecture
**Impact:** Positive - Skip license verification for help/completion commands

**Analysis:**
License verification is skipped for `Commands::License` and `Commands::Completion`. This allows users to check license status and generate shell completions without a valid license file.

```rust
let license_exempt = matches!(
    cli.command,
    Commands::License { .. } | Commands::Completion { .. }
);
```

**Recommendation:** Maintain this pattern.

---

### [Good] Error Handler Installation at Startup

**Location:** `src/main.rs:88-91`
**Category:** Architecture
**Impact:** Positive - color-eyre provides enhanced error reporting with backtraces

**Analysis:**
Error handler is installed early, providing colorful error output with backtraces for debugging. The failure to install is logged but doesn't abort.

```rust
if let Err(e) = error::install_error_handler() {
    eprintln!("warning: failed to install color-eyre handler: {e}");
}
```

**Recommendation:** Maintain this pattern.

---

## Summary Statistics (Updated)

| Module | P0 | P1 | P2 | P3 | Good | Total |
|--------|----|----|----|----|------|-------|
| LLM | 0 | 1 | 3 | 3 | 4 | 11 |
| Orchestrator | 0 | 1 | 3 | 3 | 4 | 11 |
| Runner | 0 | 3 | 5 | 4 | 5 | 17 |
| Docs/Cache/Index | 0 | 2 | 6 | 6 | 4 | 18 |
| CLI | 0 | 0 | 1 | 1 | 1 | 3 |
| Config | 0 | 1 | 2 | 1 | 2 | 6 |
| Main | 0 | 1 | 1 | 2 | 2 | 6 |
| **Total** | **0** | **8** | **17** | **18** | **22** | **65** |

**Key Themes (Updated):**
1. Sequential blocking process spawns in doc fetching (P1)
2. Full cache rewrite on every hit (P1)
3. Blocking std::fs in Config::load() (P1)
4. Config loaded multiple times per command (P1)
5. Runner builder creates intermediate allocations (P2)
6. Multiple clone() in effective_* methods (P2)
7. to_ascii_lowercase in model detection (P2)
8. Vec-based lookup instead of HashMap (P2)

**Highest Impact Fixes:**
1. Remove `flush_to_disk` from `LlmCache::lookup` (eliminates O(10,000) disk writes)
2. Load config once before command dispatch (eliminates multiple file reads)
3. Use async fs for config loading (prevents blocking on network-mounted configs)

---

## Architecture Recommendations

### Knowledge Module Architecture

The knowledge module (`src/knowledge/`) provides the knowledge foundation for grounding LLM calls. It consists of three sub-modules with distinct purposes:

**Module Structure:**
- `tool_knowledge.rs` — Embedded bioconda tool catalog (6000+ tools) with TF-IDF similarity search
- `best_practices.rs` — In-memory best practices database with category/tool indexes
- `error_db.rs` — JSONL-backed error knowledge database for learning from failures

---

#### [P2] Finding: O(n) Lookup in ToolKnowledgeBase

**Location:** `src/knowledge/tool_knowledge.rs:86-91`
**Category:** Performance
**Impact:** Medium — Linear scan for exact name lookup, called for every tool validation

**Problem:**
The `lookup` method iterates over the entire tools Vec (6000+ entries) to find an exact name match. While the inverted index enables fast keyword search, exact name lookup uses linear iteration.

**Current Code:**
```rust
pub fn lookup(&self, name: &str) -> Option<&ToolEntry> {
    let name_lower = name.to_lowercase();
    self.tools
        .iter()
        .find(|t| t.name.to_lowercase() == name_lower)
}
```

**Recommended Fix:**
Add a HashMap index for exact name lookup alongside the keyword index:
```rust
pub struct ToolKnowledgeBase {
    tools: Vec<ToolEntry>,
    index: HashMap<String, Vec<(usize, f32)>>,  // keyword -> postings
    name_index: HashMap<String, usize>,          // tool_name -> idx (NEW)
}

impl ToolKnowledgeBase {
    pub fn lookup(&self, name: &str) -> Option<&ToolEntry> {
        let name_lower = name.to_lowercase();
        self.name_index
            .get(&name_lower)
            .map(|&idx| &self.tools[idx])
    }
}
```

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark tool lookup for 1000 sequential queries

---

#### [P2] Finding: to_lowercase() Allocation in Knowledge Lookup

**Location:** `src/knowledge/tool_knowledge.rs:86-87, 127-133`
**Category:** Performance
**Impact:** Medium — Creates lowercase String for every lookup call

**Problem:**
Both `lookup` and `related_tools` create lowercase copies of the input name and tool names for comparison. This allocation happens on every call.

**Current Code:**
```rust
let name_lower = name.to_lowercase();
self.tools.iter().find(|t| t.name.to_lowercase() == name_lower)
```

**Recommended Fix:**
Pre-normalize tool names to lowercase in ToolEntry during construction, or use `eq_ignore_ascii_case`:
```rust
pub fn lookup(&self, name: &str) -> Option<&ToolEntry> {
    self.tools.iter().find(|t| t.name.eq_ignore_ascii_case(name))
}
```

Note: This requires tool names to be ASCII-only (true for bioconda packages).

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for case-insensitive lookup

---

#### [P3] Finding: Category Inference Uses 14 Keyword Groups

**Location:** `src/knowledge/tool_knowledge.rs:247-423`
**Category:** Architecture
**Impact:** Low — Category inference iterates over 14 groups for every tool load

**Problem:**
`infer_category` iterates over 14 category rules, each with 5-10 keywords. This is called once per tool during knowledge base initialization (6000+ calls).

**Analysis:**
This is acceptable because category inference happens only at initialization. The rule-based approach is simpler than a learned classifier and provides deterministic categorization.

**Recommendation:** No change needed — initialization is one-time cost.

---

#### [P3] Finding: BestPracticesDb Uses Vec Indices for Lookup

**Location:** `src/knowledge/best_practices.rs:63-78`
**Category:** Architecture
**Impact:** Low — Creates Vec clone and iterates for universal practices

**Problem:**
`for_tool` clones the Vec of indices, then iterates over all practices to find universal ones (empty tools list). The indices-based approach adds indirection.

**Current Code:**
```rust
let mut indices: Vec<usize> = self.tool_index.get(&tool_lower)
    .map(|v| v.to_vec()).unwrap_or_default();
for (idx, p) in self.practices.iter().enumerate() {
    if p.tools.is_empty() && !indices.contains(&idx) {
        indices.push(idx);
    }
}
```

**Recommended Fix:**
Store universal practices separately or use direct iteration:
```rust
pub fn for_tool(&self, tool: &str) -> Vec<&BestPractice> {
    let tool_lower = tool.to_lowercase();
    let mut results: Vec<&BestPractice> = self.practices.iter()
        .filter(|p| p.tools.is_empty()).collect(); // Universal first

    if let Some(indices) = self.tool_index.get(&tool_lower) {
        for &idx in indices {
            results.push(&self.practices[idx]);
        }
    }
    results
}
```

**Effort:** 30 minutes | Minor optimization
**Dependencies:** None
**Verification:** Unit tests

---

#### [P3] Finding: ErrorKnowledgeDb Uses Full File Scan for Search

**Location:** `src/knowledge/error_db.rs:190-208`
**Category:** Performance
**Impact:** Low — Reads entire JSONL file and filters in memory

**Problem:**
`search` reads the entire error knowledge file into memory, parses all lines, then filters. As the error database grows, this becomes inefficient.

**Current Code:**
```rust
let content = std::fs::read_to_string(&path)?;
let mut matches: Vec<ErrorRecord> = content
    .lines()
    .filter_map(|line| serde_json::from_str::<ErrorRecord>(line).ok())
    .filter(|r| r.tool.to_lowercase() == tool.to_lowercase() && r.error_category == category)
    .collect();
```

**Recommended Fix:**
For large error databases, consider an in-memory index or SQLite backend:
```rust
// Option 1: Load once and cache with tool-category index
static ERROR_INDEX: LazyLock<HashMap<(String, ErrorCategory), Vec<ErrorRecord>>> = ...

// Option 2: Use SQLite for structured queries
// (requires sqlite dependency, may not be worth the complexity)
```

**Analysis:**
For typical use (error database under 100 entries), the current approach is acceptable. Consider optimization only if error database grows significantly.

**Effort:** 2 hours | Requires design
**Dependencies:** None
**Verification:** Benchmark search with 1000+ error records

---

#### [Good] TF-IDF Index for Keyword Search

**Location:** `src/knowledge/tool_knowledge.rs:183-226`
**Category:** Architecture
**Impact:** Positive — Proper TF-IDF weighting for relevance ranking

**Analysis:**
The inverted index implements proper TF-IDF scoring with:
1. Term frequency normalization (norm_tf = count / max_tf)
2. Inverse document frequency (idf = ln(n/df) + 1.0)
3. Tool name boost factor (TOOL_NAME_BOOST = 3.0)

This is a well-designed similarity search implementation.

---

#### [Good] ErrorCategory Classification Uses Specific Patterns First

**Location:** `src/knowledge/error_db.rs:60-135`
**Category:** Architecture
**Impact:** Positive — Pattern matching order prevents misclassification

**Analysis:**
The classification logic tests specific patterns before generic ones (e.g., "reference not found" before "not found"). This prevents misclassification of reference errors as general missing input errors.

---

### Workflow Engine Architecture

The workflow module (`src/workflow.rs` and `src/workflow_graph.rs`) provides workflow registry, template management, and DAG-based orchestration.

**Module Structure:**
- `workflow.rs` — Built-in templates + LLM-based workflow generation
- `workflow_graph.rs` — LangGraph-inspired DAG workflow with scenarios

---

#### [P1] Finding: Static Templates Use include_str! for All Formats

**Location:** `src/workflow.rs:36-108`
**Category:** Architecture
**Impact:** Medium — Embeds 9 templates x 3 formats = 27 files in binary

**Problem:**
Each built-in template embeds 3 complete workflow files (native, snakemake, nextflow) via `include_str!`. This increases binary size by ~27 embedded files.

**Current Code:**
```rust
WorkflowTemplate {
    name: "rnaseq",
    native: include_str!("../workflows/native/rnaseq.toml"),
    snakemake: include_str!("../workflows/snakemake/rnaseq.smk"),
    nextflow: include_str!("../workflows/nextflow/rnaseq.nf"),
},
```

**Analysis:**
This is acceptable because:
1. Workflow templates are the core feature — users expect them to be available offline
2. Each file is typically <5KB (total ~135KB embedded)
3. No runtime file loading overhead

**Recommendation:** No change needed — embedded templates are appropriate for this use case.

---

#### [P2] Finding: WorkflowGraph Uses Arc for TaskNormalizer/Estimator

**Location:** `src/workflow_graph.rs:195-208`
**Category:** Architecture
**Impact:** Low — Arc wrap for single-threaded synchronous initialization

**Problem:**
`WorkflowGraph` wraps `TaskNormalizer` and `TaskComplexityEstimator` in `Arc`, but these are only used within the executor's async context. The Arc adds unnecessary atomic overhead for non-shared state.

**Current Code:**
```rust
pub struct WorkflowGraph {
    normalizer: Arc<TaskNormalizer>,
    estimator: Arc<TaskComplexityEstimator>,
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            normalizer: Arc::new(TaskNormalizer::new()),
            estimator: Arc::new(TaskComplexityEstimator::new()),
        }
    }
}
```

**Recommended Fix:**
Use owned values if sharing is not required:
```rust
pub struct WorkflowGraph {
    normalizer: TaskNormalizer,
    estimator: TaskComplexityEstimator,
}
```

**Analysis:**
The Arc may have been added anticipating concurrent workflow execution. If true concurrent execution is planned, Arc is correct. If single-threaded, Arc is unnecessary.

**Effort:** 15 minutes | Quick win
**Dependencies:** Confirm threading model
**Verification:** Unit tests

---

#### [P2] Finding: Duplicate LLM Request Logic in generate_workflow and infer_workflow

**Location:** `src/workflow.rs:272-479, 727-938`
**Category:** Architecture
**Impact:** Medium — Two nearly identical async LLM request implementations

**Problem:**
`generate_workflow` and `infer_workflow` share 90% identical code for HTTP request construction, streaming handling, retry logic, and response parsing. Only the prompt construction differs.

**Current Code:**
Both functions have:
1. Same HTTP client construction
2. Same streaming vs non-streaming branch
3. Same retry with format correction
4. Same `parse_workflow_response` call

**Recommended Fix:**
Extract common LLM request logic into a shared helper:
```rust
async fn call_llm_for_workflow(
    config: &Config,
    system: &str,
    user_prompt: &str,
    engine: &str,
) -> Result<GeneratedWorkflow> {
    // Common HTTP + retry logic
}

pub async fn generate_workflow(...) -> Result<GeneratedWorkflow> {
    let system = match engine { ... };
    let user_prompt = format!("Generate ... for:\n\n{task}");
    call_llm_for_workflow(config, &system, &user_prompt, engine).await
}

pub async fn infer_workflow(...) -> Result<GeneratedWorkflow> {
    let ctx = scan_data_directory(data_dir);
    let user_prompt = build_infer_prompt(task, &ctx, &data_dir_str);
    call_llm_for_workflow(config, &system, &user_prompt, engine).await
}
```

**Effort:** 1.5 hours | Requires design
**Dependencies:** None
**Verification:** Unit tests for both paths

---

#### [P3] Finding: WorkflowState Accumulates Optional Fields

**Location:** `src/workflow_graph.rs:119-142`
**Category:** Architecture
**Impact:** Low — Many Option<T> fields require unwrap_or_default pattern

**Problem:**
`WorkflowState` has 8 optional fields that are progressively populated during execution. Each field requires Option handling when accessing.

**Current Code:**
```rust
pub struct WorkflowState {
    pub input: WorkflowInput,
    pub normalized_task: Option<NormalizedTask>,
    pub complexity: Option<ComplexityResult>,
    pub mini_skill: Option<MiniSkillData>,
    pub skill: Option<SkillData>,
    pub command: Option<String>,
    pub validation_passed: bool,
    pub metadata: HashMap<String, String>,
}
```

**Recommended Fix:**
Consider a builder pattern or phased state design where fields are required after certain steps:
```rust
struct InitialState { input: WorkflowInput }
struct NormalizedState { input: WorkflowInput, normalized_task: NormalizedTask }
struct GeneratedState { ..., command: String }
```

**Analysis:**
The Option pattern is acceptable for this workflow because the state evolves through phases. A phased state design would be more type-safe but increases complexity.

**Effort:** 3 hours | Requires design (state machine)
**Dependencies:** None
**Verification:** Unit tests for workflow execution

---

#### [Good] Scenario-Based Workflow Routing

**Location:** `src/workflow_graph.rs:253-288`
**Category:** Architecture
**Impact:** Positive — Clear scenario determination based on available inputs

**Analysis:**
The `determine_scenario` logic uses a clean match on input availability:
```rust
state.scenario = match (has_doc, has_skill, has_prompt) {
    (true, true, _) => WorkflowScenario::Full,
    (true, false, _) => WorkflowScenario::Doc,
    (false, true, _) => WorkflowScenario::Skill,
    (false, false, true) => WorkflowScenario::Prompt,
    (false, false, false) => WorkflowScenario::Bare,
};
```

This provides predictable routing without complex conditionals.

---

#### [Good] Early Return on Forced Scenario

**Location:** `src/workflow_graph.rs:254-262`
**Category:** Architecture
**Impact:** Positive — Short-circuits scenario determination when forced

**Analysis:**
When `force_scenario` is provided, the function returns immediately without checking other inputs:
```rust
if let Some(scenario) = state.input.force_scenario {
    state.scenario = scenario;
    if state.input.force_mode.is_none() {
        state.mode = scenario.default_mode();
    }
    return Ok(());
}
```

This respects user intent and avoids unnecessary computation.

---

### MCP Integration Architecture

The MCP module (`src/mcp.rs`) implements a minimal MCP client for skill discovery from external servers.

---

#### [P2] Finding: Exponential Backoff with Small Initial Delay

**Location:** `src/mcp.rs:190`
**Category:** Architecture
**Impact:** Low — Initial backoff 100ms may be too short for transient failures

**Problem:**
The exponential backoff starts at 100ms, doubling each retry (100ms -> 200ms -> 400ms). For transient network issues, 100ms may not allow sufficient recovery time.

**Current Code:**
```rust
let delay = Duration::from_millis(100 * 2u64.pow(attempt as u32));
tokio::time::sleep(delay).await;
```

**Recommended Fix:**
Consider starting with 200-500ms for network failures:
```rust
const INITIAL_BACKOFF_MS: u64 = 200;
let delay = Duration::from_millis(INITIAL_BACKOFF_MS * 2u64.pow(attempt as u32));
```

**Analysis:**
The current approach is acceptable for skill fetching (non-critical path). Longer delays would be appropriate for critical operations.

**Effort:** 15 minutes | Quick win
**Dependencies:** None
**Verification:** Integration tests with simulated network delays

---

#### [P2] Finding: JSON-RPC Response Deserialization Lacks Error Detail

**Location:** `src/mcp.rs:146-166`
**Category:** Error Handling
**Impact:** Medium — Generic error messages for JSON-RPC failures

**Problem:**
Error handling for MCP responses uses generic messages like "returned invalid JSON" without including the raw response or specific parse error.

**Current Code:**
```rust
let rpc: RpcResponse = response.json().await.map_err(|e| {
    OxoError::IndexError(format!(
        "MCP server '{}' returned invalid JSON: {e}",
        self.config.name()
    ))
})?;
```

**Recommended Fix:**
Include the raw response text when JSON parsing fails:
```rust
let raw_text = response.text().await.unwrap_or_default();
let rpc: RpcResponse = serde_json::from_str(&raw_text).map_err(|e| {
    OxoError::IndexError(format!(
        "MCP server '{}' returned invalid JSON: {e}\nRaw response (first 200 chars): {}",
        self.config.name(),
        &raw_text[..raw_text.len().min(200)]
    ))
})?;
```

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Error message inspection

---

#### [P3] Finding: MCP Client HTTP Timeout at Module Level

**Location:** `src/mcp.rs:48-49`
**Category:** Architecture
**Impact:** Low — Hardcoded 10s timeout, not configurable per-server

**Problem:**
MCP_TIMEOUT_SECS is a module-level constant (10s), not configurable per server. Some MCP servers may require longer timeouts.

**Current Code:**
```rust
const MCP_TIMEOUT_SECS: u64 = 10;
```

**Recommended Fix:**
Add timeout configuration to McpServerConfig:
```rust
#[derive(Clone)]
pub struct McpServerConfig {
    pub url: String,
    pub name: String,
    pub api_key: Option<String>,
    pub timeout_secs: Option<u64>, // NEW
}

impl McpClient {
    pub fn new(config: McpServerConfig) -> Self {
        let timeout = config.timeout_secs.unwrap_or(MCP_TIMEOUT_SECS);
        McpClient {
            config,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout))
                .build()
                .expect(...),
        }
    }
}
```

**Effort:** 1 hour | Requires config changes
**Dependencies:** Config module
**Verification:** Config parsing tests

---

#### [P3] Finding: Skill URI Convention Relies on Server Compliance

**Location:** `src/mcp.rs:234-263`
**Category:** Architecture
**Impact:** Low — Assumes `skill://<tool>` URI scheme, fallback may miss resources

**Problem:**
`list_skill_resources` expects servers to use `skill://` URIs or `text/markdown` MIME type. Servers using different conventions may not be discovered.

**Current Code:**
```rust
if let Some(tool) = uri.strip_prefix("skill://") {
    entries.push(McpSkillEntry { tool: tool.to_string(), ... });
} else if mime == "text/markdown" && !name.is_empty() {
    entries.push(McpSkillEntry { tool: name, ... });
}
```

**Analysis:**
This is acceptable given the MCP skill provider convention documented in the module header. The fallback to `text/markdown` provides reasonable flexibility.

**Recommendation:** No change needed — convention is documented.

---

#### [Good] Stateless HTTP POST for MCP Operations

**Location:** `src/mcp.rs:77-78, 111-166`
**Category:** Architecture
**Impact:** Positive — No session management, each request is self-contained

**Analysis:**
The MCP client uses stateless HTTP POST for all operations (initialize, list, read). No SSE session or persistent connection is required. This simplifies error handling and enables easy retry logic.

---

#### [Good] Canonical URI Fast Path in fetch_skill

**Location:** `src/mcp.rs:294-313`
**Category:** Architecture
**Impact:** Positive — Avoids list scan for known URI format

**Analysis:**
`fetch_skill` first tries the canonical `skill://{tool}` URI directly, only falling back to resource list scanning if that fails. This avoids the overhead of listing resources when the server follows the convention.

```rust
let canonical = format!("skill://{tool}");
if let Ok(content) = self.read_resource(&canonical).await {
    return Some(content);
}
// Slow path: scan resource list
```

---

#### [Good] Transient Error Detection in Retry Logic

**Location:** `src/mcp.rs:182-185`
**Category:** Architecture
**Impact:** Positive — Only retries on network-level errors, not protocol errors

**Analysis:**
The retry logic correctly distinguishes transient network errors ("unreachable", "timed out") from protocol errors (HTTP 4xx, JSON-RPC errors). Protocol errors are returned immediately without retrying, preventing wasted attempts on permanent failures.

---

## Summary Statistics (Updated)

| Module | P0 | P1 | P2 | P3 | Good | Total |
|--------|----|----|----|----|------|-------|
| LLM | 0 | 1 | 3 | 3 | 4 | 11 |
| Orchestrator | 0 | 1 | 3 | 3 | 4 | 11 |
| Runner | 0 | 3 | 5 | 4 | 5 | 17 |
| Docs/Cache/Index | 0 | 2 | 6 | 6 | 4 | 18 |
| CLI | 0 | 0 | 1 | 1 | 1 | 3 |
| Config | 0 | 1 | 2 | 1 | 2 | 6 |
| Main | 0 | 1 | 1 | 2 | 2 | 6 |
| Knowledge | 0 | 0 | 2 | 3 | 2 | 7 |
| Workflow | 0 | 1 | 2 | 1 | 2 | 6 |
| MCP | 0 | 0 | 2 | 2 | 4 | 8 |
| **Total** | **0** | **8** | **19** | **20** | **23** | **70** |

**Key Architecture Themes:**

1. **Knowledge Module:**
   - O(n) lookup in tool knowledge base (name_index HashMap needed)
   - TF-IDF implementation is well-designed
   - Error knowledge uses JSONL append-only (appropriate for low-volume)
   - Best practices indices add indirection overhead

2. **Workflow Engine:**
   - Scenario-based routing is clean and predictable
   - Duplicate LLM request logic between generate/infer (extract helper)
   - WorkflowState uses many Option<T> fields (phased state could improve)
   - Arc wrap for non-shared components (review threading model)

3. **MCP Integration:**
   - Stateless HTTP POST design is appropriate
   - Proper transient error detection for retries
   - Canonical URI fast path avoids list overhead
   - Hardcoded timeout, not per-server configurable
---

## src/engine.rs Analysis

The module is a workflow DAG engine for bioinformatics pipelines (1600+ lines). It parses `.oxo.toml` files, expands wildcard patterns, computes execution phases, and exports to Snakemake/Nextflow. Uses blocking std::process::Command for task execution, async tokio for parallel workflow runs.

**Public API Summary:**
- `WorkflowDef::from_file(path)` — Parse TOML workflow definition
- `WorkflowDef::from_str_content(s)` — Parse workflow from string
- `expand(def)` — Expand wildcards into ConcreteTask DAG
- `compute_phases(tasks)` — Group tasks by parallel phases
- `execute(tasks, dry_run)` — Async DAG execution with checkpointing
- `verify(def)` — Semantic validation, returns diagnostics
- `to_snakemake(def)` / `to_nextflow(def)` — Export to external formats

---

### [P1] Finding: Wildcard Combination Generates Exponential Clone Chains

**Location:** `src/engine.rs:154-175`
**Category:** Performance
**Impact:** High - O(n^m) allocations for m wildcards with n values each

**Problem:**
`wildcard_combinations` uses iterative expansion where each iteration clones all existing HashMaps and inserts new keys. For a workflow with 3 wildcards (sample=[s1,s2,s3], lane=[1,2], type=[bam,cram]), this creates 12 intermediate HashMap clones, each with full key-value copies.

**Current Code:**
```rust
let mut result: Vec<HashMap<String, String>> = vec![HashMap::new()];
let mut keys: Vec<&String> = wildcards.keys().collect();
keys.sort();
for key in keys {
    let values = &wildcards[key];
    let mut next = Vec::new();
    for val in values {
        for existing in &result {
            let mut m = existing.clone();  // Full HashMap clone per iteration
            m.insert(key.clone(), val.clone());
            next.push(m);
        }
    }
    result = next;
}
```

**Recommended Fix:**
Use index-based binding generation to avoid intermediate HashMap cloning:
```rust
fn wildcard_combinations(wildcards: &HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    if wildcards.is_empty() {
        return vec![HashMap::new()];
    }
    let mut keys: Vec<&String> = wildcards.keys().collect();
    keys.sort();
    let total = keys.iter().map(|k| wildcards[k].len()).product::<usize>();
    let mut result = Vec::with_capacity(total);
    for i in 0..total {
        let mut binding = HashMap::with_capacity(keys.len());
        let mut idx = i;
        for key in &keys {
            let values = &wildcards[*key];
            binding.insert(key.clone(), values[idx % values.len()].clone());
            idx /= values.len();
        }
        result.push(binding);
    }
    result
}
```

**Effort:** 2 hours | Requires design (index-based generation)
**Dependencies:** None
**Verification:** Benchmark workflow expansion with 10+ wildcard values per key

---

### [P1] Finding: Template Substitution Creates Multiple String Copies

**Location:** `src/engine.rs:178-197`
**Category:** Performance
**Impact:** High - Called for every expanded task, creates 2-4 String copies per substitution

**Problem:**
`substitute` creates a copy of the template, then iterates over bindings and params calling `replace()` which creates a new String for each key. A command with 4 wildcards and 2 params creates 6+ intermediate Strings.

**Current Code:**
```rust
fn substitute(template: &str, bindings: &HashMap<String, String>, params: &HashMap<String, String>) -> String {
    let mut s = template.to_string();  // Initial copy
    for (k, v) in bindings {
        s = s.replace(&format!("{{{k}}}"), v);  // New String per wildcard
    }
    for (k, v) in params {
        s = s.replace(&format!("{{params.{k}}}"), v);
        if !bindings.contains_key(k.as_str()) {
            s = s.replace(&format!("{{{k}}}"), v);
        }
    }
    s
}
```

**Recommended Fix:**
Use single-pass substitution with capacity estimation - extract placeholder name from `{...}` and lookup in bindings/params directly, pushing matched value to result String.

**Effort:** 2 hours | Requires design (single-pass parsing)
**Dependencies:** None
**Verification:** Benchmark substitution with templates containing 10+ placeholders

---

### [P1] Finding: format! Macro in Nested Loops for Wildcard Pattern Detection

**Location:** `src/engine.rs:210-217`
**Category:** Performance
**Impact:** High - Called for every step during expansion, creates format! String per wildcard key

**Problem:**
`uses_wildcards` creates a format! String for each wildcard key inside a loop. This allocation is discarded immediately if the pattern is not found.

**Current Code:**
```rust
fn uses_wildcards(step: &StepDef, wildcards: &HashMap<String, Vec<String>>) -> bool {
    wildcards.keys().any(|k| {
        let pat = format!("{{{k}}}");  // Allocation per key
        step.cmd.contains(&pat) || ...
    })
}
```

**Recommended Fix:**
Pre-compute patterns once at workflow level and pass cached patterns to `uses_wildcards_cached(step, patterns)`. 

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Benchmark workflow expansion for 100+ steps

---

### [P2] Finding: task_id Creates Vec and Format for Binding Display

**Location:** `src/engine.rs:199-207`
**Category:** Performance
**Impact:** Medium - Called for every concrete task, creates Vec + format! Strings

**Problem:**
`task_id` collects bindings into Vec of formatted strings, then joins them. This creates O(n) allocations for n bindings.

**Recommended Fix:**
Build ID directly with sorted iteration using pre-allocated String capacity.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for task ID generation

---

### [P2] Finding: Repeated format! Calls in Export Functions

**Location:** `src/engine.rs:817-926 (to_snakemake), 931-1027 (to_nextflow)`
**Category:** Performance
**Impact:** Medium - Called during export, creates hundreds of format! Strings for large workflows

**Problem:**
Both export functions use `format!()` extensively in loops. Each line creates a new String. For a 50-step workflow, this creates 200+ allocations.

**Recommended Fix:**
Use push_str with inline string building or std::fmt::Write macro for efficient formatting.

**Effort:** 1.5 hours | Requires design
**Dependencies:** None
**Verification:** Benchmark export for 100-step workflow

---

### [P2] Finding: Phase Computation Partition Creates Two Vecs Per Iteration

**Location:** `src/engine.rs:383-406`
**Category:** Performance
**Impact:** Medium - Called during execution start, creates Vec partitions until tasks assigned

**Problem:**
`compute_phases` uses `.partition()` which creates two Vecs each iteration. For a 20-phase workflow, this creates 40 intermediate Vecs.

**Recommended Fix:**
Use in-place filtering with Vec<usize> indices and swap-remove pattern.

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Unit tests for phase computation

---

### [P2] Finding: HashSet Clone on Every Checkpoint Update

**Location:** `src/engine.rs:609-615`
**Category:** Performance
**Impact:** Medium - Called at execution start, clones entire checkpoint set

**Problem:**
The `done` HashSet is cloned from checkpoint on workflow start. For workflows with 500+ tasks, this creates a large HashSet copy.

**Recommended Fix:**
Use move instead of clone for `checkpoint.completed_tasks` and only clone `started` set.

**Effort:** 30 minutes | Minor optimization
**Dependencies:** None
**Verification:** N/A

---

### [Good] Early Termination in uses_wildcards

**Location:** `src/engine.rs:210-217`
**Category:** Architecture
**Impact:** Positive - Uses `.any()` for early termination

**Analysis:**
The `.any()` iterator short-circuits on first match, avoiding unnecessary checks for remaining wildcard keys.

**Recommendation:** Maintain this pattern.

---

### [Good] Sorted Key Iteration for Deterministic Output

**Location:** `src/engine.rs:159-161, 833-835, 946-948`
**Category:** Architecture
**Impact:** Positive - Ensures deterministic ordering for exports and IDs

**Analysis:**
All HashMap iteration uses sorted keys, ensuring consistent ordering for workflow IDs, Snakemake export, and Nextflow export.

**Recommendation:** Maintain this pattern for all user-facing output.

---

### [Good] Atomic File Write for Checkpoints

**Location:** `src/engine.rs:517-526`
**Category:** Architecture
**Impact:** Positive - Atomic write prevents corruption

**Analysis:**
Uses temp file + rename pattern for atomic checkpoint writes, preventing partial writes on crash.

**Recommendation:** Maintain this pattern for all file writes.

---

### [Good] Async JoinSet for Parallel Task Execution

**Location:** `src/engine.rs:560-736`
**Category:** Architecture
**Impact:** Positive - Correct async parallel execution with JoinSet

**Analysis:**
Uses `tokio::task::JoinSet` for proper parallel task execution, avoiding blocking on individual tasks.

**Recommendation:** Maintain this pattern for all parallel async execution.

---

## src/generator.rs Analysis

The module provides command generation abstraction (490 lines). Implements the Strategy pattern with three generators: LlmCommandGenerator (primary), RuleCommandGenerator (pattern matching), and CompositeGenerator (chain of responsibility).

**Public API Summary:**
- `CommandGenerator` trait — async `generate(tool, docs, skill, request, config)`
- `LlmCommandGenerator` — Uses LlmClient.suggest_command
- `RuleCommandGenerator` — Pattern matching with default rules
- `CompositeGenerator` — Chains generators in priority order

---

### [P2] Finding: Linear Search Over Rules Vec in RuleCommandGenerator

**Location:** `src/generator.rs:359-363`
**Category:** Performance
**Impact:** Medium - Called for every rule-based command, O(n) search over rules Vec

**Problem:**
`RuleCommandGenerator::generate` iterates over the `rules` Vec sequentially to find a matching rule. For each rule, `matches` is called which creates a lowercase String.

**Recommended Fix:**
Build a tool-indexed HashMap for faster rule lookup: `rules_by_tool: HashMap<String, Vec<CommandRule>>`.

**Effort:** 1 hour | Quick win
**Dependencies:** None
**Verification:** Benchmark rule-based generation

---

### [P2] Finding: Case-Insensitive Match Creates Lowercase String

**Location:** `src/generator.rs:263-274`
**Category:** Performance
**Impact:** Medium - Called for every rule match attempt, creates lowercase String per attempt

**Problem:**
`matches` converts the request to lowercase for comparison, creating a String allocation even when no match is found. Also `p.to_lowercase()` is called per pattern.

**Recommended Fix:**
Pre-store patterns in lowercase at rule construction time, then use single `request.to_lowercase()` with pre-stored lowercase patterns.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for rule matching

---

### [P2] Finding: apply() Creates Vec for Word Splitting

**Location:** `src/generator.rs:277-300`
**Category:** Performance
**Impact:** Medium - Called for every rule match, creates Vec<&str> for word extraction

**Problem:**
`apply` splits the request into words and collects them into Vec, then iterates for file name extraction.

**Recommended Fix:**
Use iterator directly without intermediate Vec, track previous word for "to <file>" pattern detection.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for rule application

---

### [P2] Finding: Template Replace Creates Two Intermediate Strings

**Location:** `src/generator.rs:326-329`
**Category:** Performance
**Impact:** Medium - Called for every rule match, creates 2 replace Strings

**Problem:**
Template application uses chained `replace()` calls which create intermediate Strings.

**Recommended Fix:**
Build string directly with single pass, checking for `{input}` and `{output}` placeholders.

**Effort:** 30 minutes | Quick win
**Dependencies:** None
**Verification:** Unit tests for template application

---

### [Good] Chain of Responsibility Pattern in CompositeGenerator

**Location:** `src/generator.rs:380-442`
**Category:** Architecture
**Impact:** Positive - Clean pattern for fallback generation

**Analysis:**
The CompositeGenerator properly implements the chain pattern with early termination on success and proper error aggregation.

**Recommendation:** Maintain this pattern for all multi-strategy approaches.

---

### [Good] Trait Design with Borrowed Parameters

**Location:** `src/generator.rs:86-112`
**Category:** Architecture
**Impact:** Positive - Trait uses borrowed params for efficiency

**Analysis:**
All parameters are borrowed (`&str`, `Option<&Skill>`, `&Config`), enabling efficient implementations without forcing caller allocations.

**Recommendation:** Maintain this pattern for all trait definitions.

---

## Summary Statistics (Final)

| Module | P0 | P1 | P2 | P3 | Good | Total |
|--------|----|----|----|----|------|-------|
| LLM | 0 | 1 | 3 | 3 | 4 | 11 |
| Orchestrator | 0 | 1 | 3 | 3 | 4 | 11 |
| Runner | 0 | 3 | 5 | 4 | 5 | 17 |
| Docs/Cache/Index | 0 | 2 | 6 | 6 | 4 | 18 |
| CLI | 0 | 0 | 1 | 1 | 1 | 3 |
| Config | 0 | 1 | 2 | 1 | 2 | 6 |
| Main | 0 | 1 | 1 | 2 | 2 | 6 |
| Knowledge | 0 | 0 | 2 | 3 | 2 | 7 |
| Workflow | 0 | 1 | 2 | 1 | 2 | 6 |
| MCP | 0 | 0 | 2 | 2 | 4 | 8 |
| Engine | 0 | 3 | 4 | 0 | 4 | 11 |
| Generator | 0 | 0 | 4 | 0 | 2 | 6 |
| **Total** | **0** | **11** | **29** | **23** | **30** | **93** |

**Key Architecture Themes (Engine/Generator):**
1. Exponential clone chains in wildcard expansion (major inefficiency)
2. Template substitution creates multiple intermediate Strings per task
3. format! macro in nested loops for workflow export
4. Linear Vec search instead of HashMap in generator rules
5. Case-insensitive comparison with repeated lowercase allocations

**Highest Impact Fixes:**
1. Remove `flush_to_disk` from `LlmCache::lookup` (eliminates O(10,000) disk writes per hit)
2. Load config once before command dispatch (eliminates multiple file reads)
3. Index-based wildcard combination generation (eliminates O(n^m * m) allocations)
4. Single-pass template substitution (eliminates 2-4 String copies per task)
5. HashMap-indexed rules in RuleCommandGenerator (O(1) tool lookup)

---

## Measurement Plan

### Finding 1: Cache hit triggers flush_to_disk

**Benchmark:** `oxo-bench eval --measure-cache-latency`
**Baseline:** ~50-100ms per cache lookup (disk write of 10K entries)
**Target:** <1ms per cache lookup (no disk write)
**Validation:** Profile with `perf` or `strace` to verify no disk I/O on cache hit

### Finding 2: SSE Streaming String Allocations

**Benchmark:** `oxo-bench eval --measure-streaming-throughput`
**Baseline:** Allocates 2 Strings per SSE chunk (~100-500 chunks per request)
**Target:** 0 allocations in hot loop (borrowed &str processing)
**Validation:** Memory profiler (valgrind/heaptrack) shows reduced allocations

### Finding 3: Config Loaded Multiple Times

**Benchmark:** Manual timing of `oxo-call run` startup
**Baseline:** 2-4 file reads per command dispatch
**Target:** Single config load at startup, cached in Arc
**Validation:** `strace -c oxo-call run` shows single open() call for config

### Finding 4: Wildcard Exponential Clone Chains

**Benchmark:** Create workflow with 5 wildcards, measure allocations
**Baseline:** O(n^5) clones for 5 wildcards with 3 values each
**Target:** O(n) using index-based combination generation
**Validation:** Memory usage during workflow creation stays bounded

### Finding 5: Command String Vec Allocations

**Benchmark:** `oxo-bench eval --measure-command-build`
**Baseline:** Vec<String> created for every argument
**Target:** Direct String building with pre-calculated capacity
**Validation:** Benchmark shows 20-30% reduction in command construction time

---

## Architecture Recommendations Summary

1. **Runner module decomposition** — Split 1100-line core.rs into focused sub-modules (validator, executor, result_analyzer)
2. **Knowledge module HashMap index** — Add name_index for O(1) tool lookups in 6000+ catalog
3. **Cache API refinement** — Separate lookup (read-only) from persistence (batch writes)
4. **Config singleton pattern** — Load once, share via Arc across all command dispatches
5. **LLM provider URL caching** — Store endpoint URLs in LlmClient struct, avoid format! per request

---

## Implementation Priority Order

**Phase 1 (Immediate):** P1 Quick Wins (<2h each)
1. Remove flush_to_disk from cache lookup
2. Single config load at startup
3. SSE streaming borrow-based parsing

**Phase 2 (Short-term):** P1 Major + P2 Quick Wins
1. Runner decomposition
2. Wildcard index generation
3. HashMap rule lookup
4. Knowledge name index

**Phase 3 (Medium-term):** P2/P3 items
1. format! elimination across modules
2. case-insensitive char comparison
3. Minor Vec/clone optimizations

**Deferred:** P3 items (code clarity, future-proofing)
