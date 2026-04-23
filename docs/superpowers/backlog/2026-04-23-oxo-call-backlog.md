# oxo-call Optimization Backlog

**Date:** 2026-04-23
**Project:** oxo-call v0.12.1
**Status:** In Progress - Static Analysis Phase

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