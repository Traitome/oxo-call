# Chat Mode Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace verbose chat mode system prompts with concise, focused prompts that enforce <100 word output and COMMAND: + NOTE: format.

**Architecture:** Single file change to `src/chat.rs` - replace two method implementations (`build_system_prompt` and `build_general_system_prompt`).

**Tech Stack:** Rust, existing test infrastructure

---

### Task 1: Update build_system_prompt method

**Files:**
- Modify: `src/chat.rs:507-517`

- [ ] **Step 1: Replace the method implementation**

Edit `src/chat.rs` line 507-517, replace the current prompt with:

```rust
    fn build_system_prompt(&self) -> String {
        match self.scenario {
            ChatScenario::Bare => String::new(),
            ChatScenario::Prompt | ChatScenario::Skill | ChatScenario::Doc | ChatScenario::Full => {
                "You are a bioinformatics CLI assistant. Answer questions about tools directly and accurately.\n\
                 \n\
                 RULES:\n\
                 1. Answer ONLY what was asked — no installation guides, no prerequisites, no step-by-step tutorials.\n\
                 2. Maximum 100 words. Fit in one CLI screen.\n\
                 3. For \"how to\" questions: use format:\n\
                    COMMAND: <exact CLI args, NO tool name>\n\
                    NOTE: <one sentence about key flags/behavior>\n\
                 4. For concept questions: give 1-2 sentence direct explanation.\n\
                 5. Respond in the same language as the question."
                    .to_string()
            }
        }
    }
```

- [ ] **Step 2: Verify existing tests pass**

Run: `cargo test --lib chat::tests::test_build_system_prompt`
Expected: All tests pass (prompt still returns non-empty string for non-bare scenarios)

- [ ] **Step 3: Commit**

```bash
git add src/chat.rs
git commit -m "feat(chat): concise system prompt for tool-specific questions"
```

---

### Task 2: Update build_general_system_prompt method

**Files:**
- Modify: `src/chat.rs:520-534`

- [ ] **Step 1: Replace the method implementation**

Edit `src/chat.rs` line 520-534, replace the current prompt with:

```rust
    /// System prompt used when there is no specific tool context (general conversation mode).
    fn build_general_system_prompt(&self) -> String {
        match self.scenario {
            ChatScenario::Bare => String::new(),
            _ => "You are a versatile assistant with expertise in bioinformatics, shell scripting, and CLI workflows.\n\
                 \n\
                 RULES:\n\
                 1. Answer directly — no tutorials, no step-by-step guides unless explicitly requested.\n\
                 2. Maximum 100 words.\n\
                 3. For command questions: COMMAND: <args> + NOTE: <brief explanation>\n\
                 4. Respond in the same language as the question."
                .to_string(),
        }
    }
```

- [ ] **Step 2: Verify existing tests pass**

Run: `cargo test --lib chat::tests::test_build_general_system_prompt`
Expected: All tests pass (prompt still returns non-empty string for non-bare scenarios)

- [ ] **Step 3: Commit**

```bash
git add src/chat.rs
git commit -m "feat(chat): concise system prompt for general questions"
```

---

### Task 3: Full test suite verification

- [ ] **Step 1: Run all chat module tests**

Run: `cargo test --lib chat`
Expected: All tests pass

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 3: Final commit if any adjustments needed**

If any fixes were required:
```bash
git add src/chat.rs
git commit -m "fix(chat): adjust prompt tests for new concise format"
```

---

### Task 4: Manual verification

- [ ] **Step 1: Build the project**

Run: `cargo build --release`
Expected: Build succeeds

- [ ] **Step 2: Test chat mode manually**

Run: `./target/release/oxo-call chat samtools "Sort BAM file by coordinate"`
Expected: Output is <100 words, contains COMMAND: + NOTE: format, no installation steps

---

## Spec Coverage Check

| Spec Requirement | Task |
|------------------|------|
| Under 100 words | Task 1, 2 (prompt rules) |
| No irrelevant content | Task 1, 2 (prompt rules) |
| COMMAND: + NOTE: format | Task 1, 2 (prompt rules) |
| Tool-specific prompt | Task 1 |
| General prompt | Task 2 |
| Existing tests pass | Task 3 |
| Manual verification | Task 4 |

## Placeholder Check

- No "TBD", "TODO", "implement later"
- All code shown explicitly
- All commands specified
- All expected outputs specified