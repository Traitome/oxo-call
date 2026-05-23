# oxo-call v0.13 — 3-Stage Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor oxo-call to a 3-stage pipeline (EXTRACT→GENERATE→VALIDATE) achieving 100% structural correctness with a single LLM call.

**Architecture:** Clean pipeline: Extract parses --help+skill into schema, Generate produces command via single LLM call with cross-domain few-shot + CoT, Validate deterministically strips/adds/enforces. No template bypass, no rule engine fallback, no LLM-within-postprocess.

**Tech Stack:** Rust 2024 edition, clap, serde, tokio, reqwest, ollama/openai providers.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/llm/prompt.rs` | Single unified prompt builder + cross-domain few-shot |
| `src/llm/provider.rs` | Clean LLM client, simplified suggest_command |
| `src/llm/postprocess.rs` | 5-step validate pipeline (STRIP→ENFORCE→ADD→REPLACE→SAFETY) |
| `src/llm/response.rs` | ARGS: + think: line parsing |
| `src/llm/types.rs` | Simplified types, no PromptTier |
| `src/config.rs` | Simplified config, no PromptTierConfig |
| `src/runner/core.rs` | Integrate validate stage, safety check |
| `src/runner/validation.rs` | Command safety check (already done) |
| `crates/oxo-bench/src/bench/compare.rs` | ROUGE-L scoring, hallucination metric |
| `crates/oxo-bench/src/bench/runner.rs` | 5-mode benchmark support |
| `src/llm/tests.rs` | Updated prompt tests |

---

### Task 1: Simplify types — remove PromptTier

**Files:**
- Modify: `src/llm/types.rs:102-111`
- Modify: `src/config.rs:100-134, 571-587, 1028-1035`

- [ ] **Step 1: Remove PromptTier from types.rs**

Read `src/llm/types.rs` around line 102. Delete the `PromptTier` enum entirely (lines 102-111). We no longer need tier selection — all models use the unified prompt.

- [ ] **Step 2: Remove PromptTierConfig from config.rs**

Read `src/config.rs`. Delete `PromptTierConfig` enum (lines ~100-134). Delete `from_str_loose`. Delete `config_tier_to_llm_tier` function (~line 1028-1035). Update `LlmConfig` struct to remove `prompt_tier` field. Remove `"llm.prompt_tier"` from `set`, `effective_value`, and `source` match arms.

- [ ] **Step 3: Update effective_prompt_tier callers**

Search for `effective_prompt_tier()` calls. Replace with a simple function that always returns `"unified"`. Or remove the function entirely and update callers. Key callers: `src/runner/core.rs:855`, `src/llm/provider.rs:785`.

- [ ] **Step 4: Update prompt_tier() in prompt.rs**

Read `src/llm/prompt.rs` around line 238. Simplify `prompt_tier()` to always return a single value (we'll rename it or remove it later). For now, make it return `PromptTier::Slim` unconditionally.

- [ ] **Step 5: Build and verify**

Run: `cargo build 2>&1 | grep "^error" | wc -l`
Expected: 0 errors

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "refactor: remove PromptTier, use unified prompt for all models"
```

---

### Task 2: Rewrite prompt.rs — unified builder + cross-domain few-shot + CoT

**Files:**
- Modify: `src/llm/prompt.rs` (complete rewrite)
- Modify: `src/llm/mod.rs` (update exports)

- [ ] **Step 1: Read current prompt.rs to understand structure**

Read the file. Note what functions are needed: `build_prompt`, `system_prompt`, `system_prompt_slim`, `find_best_subcommand_for_task`. Note what to remove: `build_prompt_full`, `build_prompt_medium`, `build_prompt_compact`, `build_prompt_compact`, `TOOL_DEFAULT_FEW_SHOT`, 50+ tool-specific if blocks, XML tag sections.

- [ ] **Step 2: Delete old prompt builders**

Remove:
- `system_prompt()` (14-rule version)
- `system_prompt_medium()`
- `system_prompt_compact()`  
- `build_prompt_full()` (~500 lines)
- `build_prompt_medium()` (~170 lines)
- `build_prompt_compact()` (~175 lines)
- `TOOL_DEFAULT_FEW_SHOT` (229 entries)
- `build_example_driven_prompt()`
- `build_retry_prompt()` (simplify)
- `truncate_documentation_for_task()`
- `split_into_sections()`
- All tool-specific `if tool == "star"` blocks (~350 lines)
- `estimate_tokens()`, `prompt_tier()`
- `score_flag_for_task()`, `get_template_example()`, `task_matches_keyword()`
- `is_commonly_used_flag()`, `build_subcommand_selection_prompt()`
- `synonym_match_for_subcmd()`, `generate_synthetic_examples()`
- `extract_task_values()` import helper

- [ ] **Step 3: Add cross-domain few-shot examples**

After system prompt, add:

```rust
/// Cross-domain few-shot examples — teach general CLI structure.
const CROSS_DOMAIN_EXAMPLES: &[(&str, &str)] = &[
    // File operations
    ("copy directory to backup", "cp -r source_dir/ /backup/"),
    ("move file to new location", "mv file.txt /new/path/"),
    // Text processing  
    ("search for ERROR in all .log files", "grep -rn ERROR *.log"),
    ("sort a CSV by second column numerically", "sort -t',' -k2,2 -n data.csv"),
    // Version control
    ("clone a repository", "git clone https://github.com/user/repo.git"),
    ("commit changes with message", "git commit -m 'fix: resolve bug'"),
    // System
    ("list processes sorted by memory", "ps aux --sort=-%mem"),
    ("show disk usage of directories", "du -sh */"),
    // Compression
    ("create tar.gz archive", "tar -czf archive.tar.gz /path/to/dir/"),
    ("compress a file with gzip", "gzip -k input.fastq"),
    // Network
    ("download a file", "curl -L -o output.tar.gz https://example.com/file.tar.gz"),
    // Bioinformatics (minimal, structural)
    ("align reads to reference", "bwa mem -t 4 reference.fa reads.fq > out.sam"),
    ("sort a BAM file", "samtools sort -@ 4 -o sorted.bam input.bam"),
];
```

- [ ] **Step 4: Write unified system prompt**

```rust
pub fn system_prompt_unified() -> &'static str {
    "You are a CLI expert. Convert tasks to exact command-line arguments.\n\
     Output format:\n\
     think: <brief reasoning>\n\
     ARGS: <arguments without tool name>"
}
```

- [ ] **Step 5: Write unified prompt builder**

```rust
pub fn build_unified_prompt(
    tool: &str,
    task: &str,
    sdoc: &StructuredDoc,
    skill: Option<&Skill>,
) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!("Tool: {tool}\n"));
    prompt.push_str(&format!("Task: {task}\n"));

    // Subcommand hint
    if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
        let best = find_best_subcommand_for_task(task, sdoc);
        if let Some(ref sub) = best {
            prompt.push_str(&format!("Subcommand: {sub}\n"));
        } else if sdoc.subcommands.len() <= 5 {
            prompt.push_str(&format!("Subcommand: {}\n", sdoc.subcommands.join(", ")));
        }
    } else if !sdoc.has_subcommands {
        prompt.push_str("(no subcommand)\n");
    }

    // Companion binary
    if !sdoc.companion_binaries.is_empty() {
        prompt.push_str(&format!("Binary: {}\n", sdoc.companion_binaries[0]));
    }

    // Flag list (max 5, pre-assigned values)
    let task_values = super::task_values::extract_task_values(task);
    let mut shown = 0usize;
    let required: Vec<_> = sdoc.flag_catalog.iter().filter(|e| e.required).take(3).collect();
    if !required.is_empty() {
        prompt.push_str("\nFlags:\n");
        for f in &required {
            let val = assign_value_to_flag(f, &task_values);
            prompt.push_str(&format!("  {}{}\n", f.flag, val));
            shown += 1;
        }
    }
    if shown < 5 {
        let task_lower = task.to_ascii_lowercase();
        let task_kw: Vec<&str> = task_lower.split_whitespace()
            .filter(|w| w.len() >= 2 && !w.contains('.')).collect();
        let mut opt: Vec<_> = sdoc.flag_catalog.iter()
            .filter(|e| !e.required).collect();
        opt.sort_by(|a, b| {
            let sa = flag_score(a, &task_kw, &task_lower);
            let sb = flag_score(b, &task_kw, &task_lower);
            sb.cmp(&sa)
        });
        if shown == 0 { prompt.push_str("\nFlags:\n"); }
        for f in opt.iter().take(5 - shown) {
            let val = assign_value_to_flag(f, &task_values);
            prompt.push_str(&format!("  {}{}\n", f.flag, val));
        }
    }

    // Cross-domain example (different tool category)
    let cross_example = pick_cross_domain_example(tool, task);
    if let Some((cross_task, cross_args)) = cross_example {
        prompt.push_str(&format!("\nExample:\n  Task: {cross_task}\n  ARGS: {cross_args}\n"));
    }

    // Tool-specific example (from skill or doc)
    let tool_example = skill
        .and_then(|s| s.select_examples(1, Some(task)).first().map(|e| e.args.as_str()))
        .or_else(|| sdoc.extracted_examples.first().map(|s| s.as_str()));
    if let Some(args) = tool_example {
        prompt.push_str(&format!("\n{}-specific:\n  ARGS: {args}\n", tool));
    }

    prompt.push_str("\nThink step by step, then output:\nthink: <reasoning>\nARGS:");
    prompt
}
```

- [ ] **Step 6: Write helper functions**

```rust
fn flag_score(entry: &FlagEntry, kw: &[&str], task_lower: &str) -> i32 {
    let dl = entry.description.to_ascii_lowercase();
    let fl = entry.flag.to_ascii_lowercase();
    let mut s = 0i32;
    for k in kw {
        if dl.contains(k) { s += 2; }
        if fl.contains(k) { s += 1; }
    }
    if dl.contains("output") && (task_lower.contains("output") || task_lower.contains("save")) { s += 3; }
    if (dl.contains("thread") || dl.contains("cpu")) && task_lower.contains("thread") { s += 3; }
    s
}

fn pick_cross_domain_example(tool: &str, task: &str) -> Option<(&'static str, &'static str)> {
    let task_lower = task.to_ascii_lowercase();
    // Pick a structurally different example to teach general CLI patterns
    if task_lower.contains("copy") || task_lower.contains("move") {
        Some(("search for ERROR in all .log files", "grep -rn ERROR *.log"))
    } else if task_lower.contains("search") || task_lower.contains("find") {
        Some(("copy directory to backup", "cp -r source_dir/ /backup/"))
    } else {
        // Default: show a simple file operation
        CROSS_DOMAIN_EXAMPLES.first().copied()
    }
}
```

- [ ] **Step 7: Keep find_best_subcommand_for_task**

This function (already built and tested) remains unchanged.

- [ ] **Step 8: Update prompt module exports**

In `src/llm/mod.rs`, update:
```rust
pub use prompt::{build_unified_prompt, system_prompt_unified, find_best_subcommand_for_task};
```

- [ ] **Step 9: Update build_prompt callers**

Search for `build_prompt(` calls. Update all to use `build_unified_prompt(tool, task, sdoc, skill)`. Key locations: `src/llm/provider.rs:1116,1127`, `src/llm/tests.rs`.

- [ ] **Step 10: Build and fix errors**

Run: `cargo build 2>&1 | grep "^error" | head -20`
Fix any compilation errors from removed functions.

- [ ] **Step 11: Commit**

```bash
git add -A && git commit -m "refactor: unified prompt builder with cross-domain few-shot and CoT"
```

---

### Task 3: Simplify response.rs — ARGS: + think: parsing

**Files:**
- Modify: `src/llm/response.rs`

- [ ] **Step 1: Add think: line parsing**

Read `src/llm/response.rs`. Find `parse_response`. Add logic to extract and discard `think:` lines:

```rust
pub fn parse_args_line(raw: &str) -> Option<Vec<String>> {
    // Find ARGS: line, extract arguments
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(args_str) = trimmed.strip_prefix("ARGS:").or_else(|| trimmed.strip_prefix("args:")) {
            let args = parse_shell_args(args_str.trim());
            if !args.is_empty() {
                return Some(args);
            }
        }
    }
    // Fallback: try to extract command from full text
    let cleaned = strip_markdown_fences(raw);
    // Try JSON as last resort
    if let Ok(suggestion) = try_parse_json_response(&cleaned) {
        return Some(suggestion.args);
    }
    None
}
```

- [ ] **Step 2: Simplify parse_response**

Remove complex JSON parsing, keep `ARGS:/EXPLANATION:` format and `parse_shell_args`. The unified prompt outputs `think:` + `ARGS:`, so we only need `ARGS:` parsing.

- [ ] **Step 3: Build and test**

Run: `cargo test 2>&1 | grep "test result"`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "refactor: simplify response parsing for think:+ARGS: format"
```

---

### Task 4: Rewrite postprocess.rs — 5-step validate pipeline

**Files:**
- Modify: `src/llm/postprocess.rs` (major rewrite)

- [ ] **Step 1: Delete old post-processing functions**

Remove all functions except `extract_task_values` (used elsewhere). Remove: `apply_corrections_to_args`, `apply_template_corrections`, `validate_flags_against_catalog` (old version), `add_missing_required_flags` (old version), `add_task_implied_flags`, `limit_flag_count`, `fill_missing_flag_values`, `replace_generic_values` (old version), `fix_output_extensions`, `fix_generic_output_bam`, `apply_tool_specific_corrections`, `filter_irrelevant_flags_for_small_model`, `enforce_mandatory_positional_args`, `clean_help_text_in_args`, `enforce_semantic_requirements` (old version), all helper functions.

- [ ] **Step 2: Write 5-step validate pipeline**

```rust
use crate::doc_processor::{FlagEntry, StructuredDoc};

/// 5-step deterministic validation. Never calls LLM.
pub fn validate_command(
    args: &[String],
    tool: &str,
    task: &str,
    sdoc: &StructuredDoc,
    best_subcommand: Option<&str>,
) -> Vec<String> {
    let mut result = args.to_vec();
    
    // Step 1: STRIP — remove flags not in unified schema
    result = strip_unknown_flags(&result, sdoc);
    
    // Step 2: ENFORCE — force correct subcommand
    result = enforce_subcommand(&result, sdoc, best_subcommand);
    
    // Step 3: ADD — insert missing required flags
    result = add_required_flags(&result, sdoc, task);
    
    // Step 4: REPLACE — substitute generic filenames
    result = replace_generic_values(&result, task);
    
    // Step 5: SAFETY — no destructive commands
    // (handled in runner/core.rs before execution)
    
    result
}
```

- [ ] **Step 3: Implement Step 1 — STRIP**

```rust
fn strip_unknown_flags(args: &[String], sdoc: &StructuredDoc) -> Vec<String> {
    let known: std::collections::HashSet<String> = sdoc.flag_catalog.iter()
        .flat_map(|e| {
            let mut flags = vec![e.flag.clone()];
            if let Some(ref alt) = e.alt_form { flags.push(alt.clone()); }
            flags
        })
        .chain(UNIVERSAL_FLAGS.iter().map(|s| s.to_string()))
        .map(|f| f.trim_end_matches('=').to_ascii_lowercase())
        .collect();
    
    if known.is_empty() { return args.to_vec(); }
    
    let mut result = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            let flag_key = arg.split('=').next().unwrap_or(arg).to_ascii_lowercase();
            if known.contains(&flag_key) || known.contains(&arg.to_ascii_lowercase()) {
                result.push(arg.clone());
                if i + 1 < args.len() && !args[i+1].starts_with('-') && !arg.contains('=') {
                    result.push(args[i+1].clone());
                    i += 1;
                }
            }
            // else: skip this unknown flag (and its value if present)
            else if i + 1 < args.len() && !args[i+1].starts_with('-') && !arg.contains('=') {
                i += 1; // skip value too
            }
        } else {
            result.push(arg.clone());
        }
        i += 1;
    }
    result
}

const UNIVERSAL_FLAGS: &[&str] = &[
    "-h", "--help", "-v", "--version",
    "-o", "--output", "--outdir", "--out", "-O",
    "-t", "--threads", "-@", "-p",
    "-i", "--input", "-f", "-1", "-2",
    "-r", "-R", "--reference", "--ref", "-x",
    "-q", "--quality", "-l", "--length",
    "-d", "--dir", "-m", "--memory",
    "-s", "-n", "-a", "-b", "-g", "-e", "-w", "-k", "-c",
];
```

- [ ] **Step 4: Implement Step 2 — ENFORCE**

```rust
fn enforce_subcommand(
    args: &[String],
    sdoc: &StructuredDoc,
    best_subcommand: Option<&str>,
) -> Vec<String> {
    let mut result = args.to_vec();
    if !sdoc.has_subcommands || sdoc.subcommands.is_empty() || result.is_empty() {
        return result;
    }
    
    let first = &result[0];
    let first_is_flag = first.starts_with('-');
    let first_in_subs = sdoc.subcommands.iter()
        .any(|s| s.eq_ignore_ascii_case(first));
    
    if let Some(forced) = best_subcommand {
        if first_is_flag {
            result.insert(0, forced.to_string());
        } else if !first_in_subs || !forced.eq_ignore_ascii_case(first) {
            result[0] = forced.to_string();
        }
    } else if first_is_flag {
        if let Some(fallback) = sdoc.subcommands.first() {
            result.insert(0, fallback.clone());
        }
    }
    
    result
}
```

- [ ] **Step 5: Implement Step 3 — ADD**

```rust
fn add_required_flags(args: &[String], sdoc: &StructuredDoc, task: &str) -> Vec<String> {
    let mut result = args.to_vec();
    let task_values = super::task_values::extract_task_values(task);
    let task_lower = task.to_ascii_lowercase();
    let args_lower = result.join(" ").to_ascii_lowercase();
    
    for entry in &sdoc.flag_catalog {
        if !entry.required { continue; }
        let flag_key = entry.flag.split('=').next().unwrap_or(&entry.flag)
            .to_ascii_lowercase();
        if args_lower.contains(&flag_key) { continue; }
        
        // Insert required flag with best-guess value
        let value = assign_flag_value(entry, &task_values);
        let insert_pos = result.iter().position(|a| !a.starts_with('-'))
            .unwrap_or(result.len());
        if let Some(val) = value {
            result.insert(insert_pos, val);
        }
        result.insert(insert_pos, entry.flag.clone());
    }
    
    // Always add output flag when task implies output
    let has_output = args_lower.contains(" -o ") || args_lower.contains("--out");
    if !has_output && (task_lower.contains("save") || task_lower.contains("output")
        || task_lower.contains("write") || !task_values.output_files.is_empty())
    {
        if let Some(out_file) = task_values.output_files.first() {
            result.push("-o".to_string());
            result.push(out_file.clone());
        }
    }
    
    // Always add thread flag when task mentions threads
    let has_thread = args_lower.contains(" -@ ") || args_lower.contains(" -t ")
        || args_lower.contains("--thread");
    if !has_thread && (task_lower.contains("thread") || task_lower.contains("cpu")
        || task_lower.contains("core"))
    {
        for n in &task_values.numbers {
            if let Ok(v) = n.parse::<u32>() {
                if v > 0 && v <= 128 {
                    result.push("-@".to_string());
                    result.push(n.clone());
                    break;
                }
            }
        }
    }
    
    result
}
```

- [ ] **Step 6: Implement Step 4 — REPLACE**

```rust
fn replace_generic_values(args: &[String], task: &str) -> Vec<String> {
    let task_values = super::task_values::extract_task_values(task);
    let generics: &[&str] = &[
        "output.bam", "output.vcf", "output.fastq", "output.sam", "output.bed",
        "output.txt", "output_dir", "output_dir/", "out.bam", "out.sam",
        "input.bam", "input.fastq", "input.fa", "input.fasta",
        "reference.fa", "reference.fasta", "genome.fa",
        "reads.fq", "reads.fastq", "reads_1.fq", "reads_2.fq",
        "annotation.gtf", "annotation.gff", "database", "metrics.txt",
        "sorted.bam", "filtered.bam", "merged.bam", "aligned.bam",
    ];
    
    args.iter().map(|arg| {
        let al = arg.to_ascii_lowercase();
        if generics.contains(&al.as_str()) {
            // Try to find a matching replacement from task values
            if al.contains("output") || al.contains("out.") || al.contains("sorted")
                || al.contains("filtered") || al.contains("merged") || al.contains("aligned")
            {
                if let Some(repl) = task_values.output_files.first() {
                    return repl.clone();
                }
            }
            if al.contains("input") || al.contains("read") {
                if let Some(repl) = task_values.input_files.first() {
                    return repl.clone();
                }
            }
            if al.contains("reference") || al.contains("genome") {
                if let Some(repl) = task_values.reference_files.first() {
                    return repl.clone();
                }
                for f in &task_values.input_files {
                    let fl = f.to_ascii_lowercase();
                    if fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna") {
                        return f.clone();
                    }
                }
            }
            if al.contains("annotation") || al.contains(".gtf") || al.contains(".gff") {
                if let Some(repl) = task_values.annotation_files.first() {
                    return repl.clone();
                }
            }
            if al.contains("database") || al.contains("db") {
                if let Some(repl) = task_values.database_files.first() {
                    return repl.clone();
                }
            }
        }
        arg.clone()
    }).collect()
}
```

- [ ] **Step 7: Build and test**

Run: `cargo build 2>&1 | grep "^error" | wc -l`
Expected: 0 errors
Run: `cargo test 2>&1 | grep "test result"`
Expected: all pass (some postprocess tests may need updating)

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "refactor: 5-step validate pipeline replacing old post-processing"
```

---

### Task 5: Simplify provider.rs — clean suggest_command

**Files:**
- Modify: `src/llm/provider.rs`

- [ ] **Step 1: Remove old paths**

Delete: `suggest_command_two_step` (entire function), Phase A pre-detection variables (rule_subcmd, llm_subcmd, template_subcmd, selected_subcommand), Phase B bypass logic (~200 lines), Phase F5 sub-LLM flag correction, FEW-SHOT multi-turn dispatch in `call_api`.

- [ ] **Step 2: Simplify suggest_command to ~80 lines**

```rust
pub async fn suggest_command(
    &self,
    tool: &str,
    documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    no_prompt: bool,
    structured_doc: Option<&StructuredDoc>,
) -> Result<LlmCommandSuggestion> {
    let sdoc = match structured_doc {
        Some(s) => s,
        None => return Err(OxoError::LlmError("No structured doc available".into())),
    };

    let model = self.config.effective_model();
    let temperature = Some(0.0);
    
    // Build unified prompt
    let prompt = build_unified_prompt(tool, task, sdoc, skill);
    
    // Single LLM call
    let sys = if no_prompt { "" } else { system_prompt_unified() };
    let api_start = std::time::Instant::now();
    let raw = self.request_with_system(sys, &prompt, None, temperature).await?;
    let inference_ms = api_start.elapsed().as_secs_f64() * 1000.0;
    
    // Parse response
    let args = match parse_args_line(&raw) {
        Some(a) => a,
        None => parse_shell_args(&raw),
    };
    
    // Validate
    let best_sub = find_best_subcommand_for_task(task, sdoc);
    let validated = validate_command(&args, tool, task, sdoc, best_sub.as_deref());
    
    Ok(LlmCommandSuggestion {
        args: validated,
        explanation: String::new(),
        inference_ms,
    })
}
```

- [ ] **Step 3: Simplify call_api**

Remove `tier: PromptTier` parameter. Remove few-shot dispatch. Single system prompt for all models.

- [ ] **Step 4: Remove unused imports**

Clean up imports in provider.rs. Remove `system_prompt_compact`, `system_prompt_medium`, `generate_from_template_with_score`, `merge_llm_into_template`, `merge_template_with_llm`, `find_best_template`, `fill_template`, `rule_based_subcommand_match`, `detect_subcommand_for_tool`.

- [ ] **Step 5: Build and test**

Run: `cargo build 2>&1 | grep "^error" | wc -l`
Run: `cargo test 2>&1 | grep "test result"`

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "refactor: simplify suggest_command to clean 3-stage pipeline"
```

---

### Task 6: Update runner/core.rs — integrate validate + safety

**Files:**
- Modify: `src/runner/core.rs`

- [ ] **Step 1: Remove old tier/model-size logic**

Remove `model_size_category()` call, `use_two_step` logic, verbose tier logging.

- [ ] **Step 2: Ensure safety check runs**

The safety check was already added in a previous commit. Verify it's called in both `run()` and `dry_run()`.

- [ ] **Step 3: Build and test**

Run: `cargo test 2>&1 | grep "test result"`

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "refactor: clean up runner for 3-stage pipeline"
```

---

### Task 7: Update tests for new pipeline

**Files:**
- Modify: `src/llm/tests.rs`

- [ ] **Step 1: Update prompt tests**

Remove tests for deleted functions (`test_build_prompt_compact_uses_tool_defaults`, `test_build_prompt_medium_with_structured_doc`, `test_doc_accuracy_*`). Update remaining tests to use `build_unified_prompt`.

- [ ] **Step 2: Update provider tests**

Remove tests for `suggest_command_two_step`. Update tests for new `suggest_command` signature.

- [ ] **Step 3: Run all tests**

Run: `cargo test 2>&1 | grep "test result"`
Expected: 978 + 1025 + 208 = 2211 passed

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "test: update tests for 3-stage pipeline"
```

---

### Task 8: Add ROUGE-L and hallucination metrics to benchmark

**Files:**
- Modify: `crates/oxo-bench/src/bench/compare.rs`
- Modify: `crates/oxo-bench/src/bench/runner.rs`

- [ ] **Step 1: Add ROUGE-L function to compare.rs**

```rust
/// ROUGE-L: longest common subsequence similarity.
pub fn rouge_l(a: &str, b: &str) -> f64 {
    let a_tokens: Vec<&str> = a.split_whitespace().collect();
    let b_tokens: Vec<&str> = b.split_whitespace().collect();
    let n = a_tokens.len();
    let m = b_tokens.len();
    if n == 0 || m == 0 { return if n == m { 1.0 } else { 0.0 }; }
    
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if a_tokens[i-1] == b_tokens[j-1] {
                dp[i][j] = dp[i-1][j-1] + 1;
            } else {
                dp[i][j] = dp[i-1][j].max(dp[i][j-1]);
            }
        }
    }
    let lcs = dp[n][m] as f64;
    let recall = lcs / n as f64;
    let precision = lcs / m as f64;
    if recall + precision == 0.0 { 0.0 } else { 2.0 * recall * precision / (recall + precision) }
}
```

- [ ] **Step 2: Add hallucination rate to CompareResult**

```rust
pub struct CompareResult {
    // ... existing fields ...
    pub hallucination_rate: f64,  // flags_not_in_catalog / total_flags
    pub rouge_l_score: f64,
}
```

- [ ] **Step 3: Update aggregate_results in runner.rs**

Add `avg_hallucination_rate` and `avg_rouge_l` to model summary output.

- [ ] **Step 4: Build and test**

Run: `cargo test -p oxo-bench 2>&1 | grep "test result"`

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add ROUGE-L and hallucination rate to benchmark metrics"
```

---

### Task 9: 5-mode benchmark support

**Files:**
- Modify: `crates/oxo-bench/src/main.rs`
- Modify: `crates/oxo-bench/src/bench/runner.rs`

- [ ] **Step 1: Ensure 5 scenarios work**

The benchmark already supports `scenarios = ["bare", "prompt", "doc", "skill", "full"]`. Verify that each scenario correctly toggles the `--no-doc`, `--no-skill`, `--no-prompt` flags on `oxo-call dry-run`.

- [ ] **Step 2: Create benchmark configs for all 5 modes**

```toml
# bench_config_all.toml
scenarios = ["bare", "prompt", "doc", "skill", "full"]
```

- [ ] **Step 3: Build and verify**

Run: `cargo build -p oxo-bench`

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: support 5-mode benchmark evaluation"
```

---

### Task 10: Final cleanup — make ci, dead code, documentation

**Files:**
- Modify: Multiple files (dead code removal)
- Modify: `README.md` (architecture update)

- [ ] **Step 1: Remove dead code**

Search for and remove:
- `template_engine.rs`: remove bypass functions, keep `fill_template` only
- `rule_engine.rs`: remove bypass-related exports
- `config.rs`: remove `model_size_category`, `infer_model_parameter_count`
- `diagnostic.rs`: remove unused trace methods

- [ ] **Step 2: Run make ci**

```bash
make ci
```
Expected: fmt ✓, clippy ✓, build ✓, test ✓

- [ ] **Step 3: Update README architecture section**

Document the 3-stage pipeline architecture.

- [ ] **Step 4: Final commit**

```bash
git add -A && git commit -m "chore: remove dead code, update docs for v0.13 3-stage pipeline"
```

---

### Task 11: Comprehensive benchmark and iterate

**Files:**
- No code changes, only benchmark runs

- [ ] **Step 1: Run 7B doc benchmark**

```bash
OXO_CALL_LLM_PROVIDER=ollama OXO_CALL_LLM_MODEL=qwen2.5-coder:7b \
  cargo run -p oxo-bench -- eval --config bench_config.toml \
  --data-dir ../oxo-call-test/bench/test-bench/bench_data \
  --output /tmp/bench-v13-7b --oxo-call target/release/oxo-call
```

- [ ] **Step 2: Run DeepSeek V4 doc benchmark**

```bash
OXO_CALL_LLM_PROVIDER=openai OXO_CALL_LLM_MODEL=DeepSeek-V4-Flash \
  OXO_CALL_LLM_API_BASE=https://api.chat.csu.edu.cn/v1 \
  OXO_CALL_LLM_API_TOKEN=<token> \
  cargo run -p oxo-bench -- eval --config bench_config_dsv4.toml \
  --data-dir ../oxo-call-test/bench/test-bench/bench_data \
  --output /tmp/bench-v13-dsv4 --oxo-call target/release/oxo-call
```

- [ ] **Step 3: Analyze results**

Compare subcommand_match_rate, flag_recall, format_valid_rate, hallucination_rate against v0.12 baseline (41%).

- [ ] **Step 4: Iterate on prompt/validate**

Based on results, tune: flag list size, example selection, subcommand matching threshold, value replacement patterns.

- [ ] **Step 5: Commit iterations**

Each iteration that improves metrics gets its own commit.
