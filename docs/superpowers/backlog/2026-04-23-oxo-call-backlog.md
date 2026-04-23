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