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