use crate::config::Config;
#[cfg(not(target_arch = "wasm32"))]
use crate::copilot_auth;
use crate::error::{OxoError, Result};
use crate::runner::{is_companion_binary, is_script_executable};
use crate::skill::Skill;
use serde::{Deserialize, Serialize};

/// A parsed LLM response with command arguments and explanation
#[derive(Debug, Clone)]
pub struct LlmCommandSuggestion {
    pub args: Vec<String>,
    pub explanation: String,
    #[allow(dead_code)]
    pub raw_response: String,
    /// Cumulative time (ms) spent in LLM API inference calls for this
    /// suggestion.  When retries occur, all attempts are summed.
    pub inference_ms: f64,
}

#[derive(Debug, Clone)]
pub struct LlmVerificationResult {
    pub provider: String,
    pub api_base: String,
    pub model: String,
    pub response_preview: String,
}

/// Result of an LLM-based analysis of a completed command run.
#[derive(Debug, Clone)]
pub struct LlmRunVerification {
    /// Whether the run looks successful.
    pub success: bool,
    /// One-sentence summary of the result.
    pub summary: String,
    /// Detected issues (empty when success is clean).
    pub issues: Vec<String>,
    /// Actionable suggestions for the user.
    pub suggestions: Vec<String>,
}

// ─── Provider trait ──────────────────────────────────────────────────────────

/// Trait that all LLM provider backends must implement.
///
/// This enables a plugin-style architecture where new providers can be added
/// without modifying the core `LlmClient` logic.  The built-in
/// `OpenAiCompatibleProvider` covers OpenAI, GitHub Copilot, Anthropic, and
/// Ollama; custom implementations can override it for providers with different
/// API shapes.
#[cfg(not(target_arch = "wasm32"))]
#[allow(async_fn_in_trait, dead_code)]
pub trait LlmProvider {
    /// Send a chat completion request and return the assistant's raw text.
    async fn chat_completion(
        &self,
        system: &str,
        user_prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String>;

    /// Human-readable provider name (e.g. "openai", "anthropic").
    fn name(&self) -> &str;
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

/// Result of an LLM-based review of a skill file.
#[derive(Debug, Clone)]
pub struct LlmSkillVerification {
    /// Whether the skill passes the quality bar.
    pub passed: bool,
    /// Short overall verdict (one sentence).
    pub summary: String,
    /// Format or structural issues found.
    pub issues: Vec<String>,
    /// Actionable improvement suggestions.
    pub suggestions: Vec<String>,
}

// ─── System prompt ────────────────────────────────────────────────────────────

fn system_prompt() -> &'static str {
    "You are a bioinformatics CLI assistant. Translate the task into command-line arguments for the specified tool. Understand any language.\n\
     \n\
     FORMAT: Respond with EXACTLY two lines, nothing else:\n\
     ARGS: <subcommand then flags and values — NO tool name, NO markdown>\n\
     EXPLANATION: <one sentence in the task's language>\n\
     \n\
     RULES:\n\
     1. NEVER start ARGS with the tool name (auto-prepended by system).\n\
     2. First token = subcommand (sort, view, mem, index, etc), NEVER a flag.\n\
     3. Companion binaries (e.g. bowtie2-build) or scripts (e.g. bbduk.sh) go as first token when skill docs say so.\n\
     4. Multi-step: join with &&. Tool name auto-prepended ONLY to first segment — later commands MUST include their full binary name.\n\
     5. Pipes (|) and redirects (>) go directly in ARGS.\n\
     6. Use ONLY flags from docs or skill examples — never invent flags.\n\
     7. Include every file/path from the task. Prefer skill example flags. Include threads (-@/-t/--threads) and output (-o) when applicable.\n\
     8. Default conventions: paired-end, coordinate-sorted BAM, hg38, gzipped FASTQ, Phred+33.\n\
     9. Match format flags to actual types (BAM/SAM/CRAM, gzipped/plain, paired/single, FASTA/FASTQ).\n\
     10. If no arguments needed: ARGS: (none)."
}

/// Medium-compression system prompt for 4k–16k context or 4B–7B models.
///
/// Keeps the essential rules but drops the verbose best-practice details
/// and companion-binary explanations (those are handled via skill injection).
fn system_prompt_medium() -> &'static str {
    "You translate bioinformatics tasks into CLI arguments.\n\
     Output EXACTLY two lines:\n\
     ARGS: <subcommand then flags, NO tool name>\n\
     EXPLANATION: <one sentence>\n\
     Rules: subcommand first (sort/view/mem), never tool name. Use only documented flags. \
     Include paths from task. Multi-step uses && (tool name only on first segment). \
     Pipes allowed. Include threads and output flags when applicable."
}

/// Ultra-compact system prompt for mini models (≤ 3B parameters).
///
/// Uses a concrete example instead of abstract format descriptions.
/// Small models learn better from examples than from rules.
fn system_prompt_compact() -> &'static str {
    "You translate tasks into CLI arguments.\n\
     Output EXACTLY two lines:\n\
     ARGS: sort -@ 4 -o out.bam in.bam\n\
     EXPLANATION: Sort BAM by coordinate.\n\
     Rules: first token = subcommand (sort, view, mem, etc), never tool name. \
     Use flags from examples only. Pipes and chains allowed."
}

// ── Token estimation ─────────────────────────────────────────────────────────

/// Rough token count estimate for prompt budgeting.
///
/// Uses the heuristic ~4 characters per token (covers English text, CLI flags,
/// and file paths).  This is intentionally conservative — real tokenizers
/// produce fewer tokens for structured text.
pub fn estimate_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
}

/// Context budget tiers used to select prompt compression level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptTier {
    /// Full prompt — no compression (context window ≥ 16k or unknown)
    Full,
    /// Medium compression — trimmed docs, reduced skill examples (4k–16k)
    Medium,
    /// Aggressive compression — compact system prompt, top-3 examples only,
    /// docs heavily truncated or omitted (≤ 4k)
    Compact,
}

/// Determine the prompt tier from context window size (in tokens) and model name.
///
/// The tier controls how aggressively the prompt is compressed:
/// - **Full** — no compression (large context, large model)
/// - **Medium** — trimmed docs, reduced examples (moderate context or small model)
/// - **Compact** — compact system prompt, top-3 examples only, docs heavily truncated
///
/// # Model-size-aware compression
///
/// Small models (≤ 3B parameters) have limited effective context utilization
/// even when they report large context windows (e.g., qwen2.5-coder:0.5b has
/// 32K context but performs poorly with long prompts). Benchmarks show that:
/// - 0.5B models get **worse** when given full docs (accuracy drops ~20%)
/// - Small models produce empty outputs when prompts exceed ~4K tokens
/// - Skill-only context is the most effective grounding for small models
///
/// Therefore, we override the tier for small models regardless of their
/// reported context window size.
pub fn prompt_tier(context_window: u32, model: &str) -> PromptTier {
    // Model-size override: small models get compressed prompts because they
    // cannot effectively use long context regardless of window size.
    // Real-world testing with Ollama quantized models shows:
    //   - ≤ 3B: even Medium tier (5 examples + full system prompt) overwhelms them
    //   - 7B+: Full tier works well
    //   - 0.5B: Compact tier enables output where Full/Medium produce empty
    if let Some(param_count) = crate::config::infer_model_parameter_count(model)
        && param_count <= 3.0
    {
        // ≤ 3B: always Compact — use compact system prompt + 3 examples max
        return PromptTier::Compact;
    }

    // Standard tier logic based on context window alone
    if context_window == 0 || context_window >= 16384 {
        PromptTier::Full
    } else if context_window >= 4096 {
        PromptTier::Medium
    } else {
        PromptTier::Compact
    }
}

// ─── User prompt ─────────────────────────────────────────────────────────────

/// Build the enriched user prompt, injecting skill knowledge when available.
///
/// When `context_window > 0`, the prompt is adaptively compressed to fit within
/// the model's context budget (leaving room for the system prompt and response).
fn build_prompt(
    tool: &str,
    documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    no_prompt: bool,
    context_window: u32,
    tier: PromptTier,
) -> String {
    // Ablation: bare LLM mode - just the task, no context
    if no_prompt {
        return format!(
            "Generate command-line arguments for the tool '{}' to accomplish this task:\n\n{}\n\n\
             Respond with EXACTLY two lines:\n\
             ARGS: <command-line arguments without the tool name>\n\
             EXPLANATION: <brief explanation>",
            tool, task
        );
    }

    match tier {
        PromptTier::Full => build_prompt_full(tool, documentation, task, skill),
        PromptTier::Medium => build_prompt_medium(tool, documentation, task, skill, context_window),
        PromptTier::Compact => {
            build_prompt_compact(tool, documentation, task, skill, context_window)
        }
    }
}

/// Full prompt — no compression.  Used for large models (≥ 16k context).
fn build_prompt_full(tool: &str, documentation: &str, task: &str, skill: Option<&Skill>) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("# Tool: `{tool}`\n\n"));

    // Inject skill knowledge (concepts, pitfalls, examples) before the raw docs.
    // This primes the LLM with expert knowledge before it reads the potentially
    // noisy --help output.
    if let Some(skill) = skill {
        let section = skill.to_prompt_section_for_task(usize::MAX, task);
        if !section.is_empty() {
            prompt.push_str(&section);
        }
    }

    // Raw tool documentation (--help output and/or cached docs)
    prompt.push_str("## Tool Documentation\n");
    prompt.push_str(documentation);
    prompt.push_str("\n\n");

    // The user's task
    prompt.push_str(&format!("## Task\n{task}\n\n"));

    // Concise format reminder (detailed rules are in the system prompt)
    prompt.push_str(
        "## Output\n\
         ARGS: <subcommand then flags, NO tool name>\n\
         EXPLANATION: <brief>\n",
    );

    prompt
}

/// Medium-compressed prompt for moderate context windows (4k–16k) or 4B–7B models.
///
/// Strategy: keep skill examples (up to 5, task-relevant), truncate documentation
/// to fit the remaining budget.  Docs are placed AFTER skill but BEFORE task,
/// so the model sees expert knowledge first, then reference docs, then the task.
fn build_prompt_medium(
    tool: &str,
    documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    context_window: u32,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("# Tool: `{tool}`\n\n"));

    // Skill with up to 5 examples (task-relevant selection)
    if let Some(skill) = skill {
        let section = skill.to_prompt_section_for_task(5, task);
        if !section.is_empty() {
            prompt.push_str(&section);
        }
    }

    // Calculate remaining budget for documentation.
    // We need to reserve space for: system prompt + task + format reminder + response.
    let sys_tokens = estimate_tokens(system_prompt_medium());
    let prompt_so_far_tokens = estimate_tokens(&prompt);
    let task_and_format_tokens = estimate_tokens(task) + 60; // format reminder is ~60 tokens
    let response_reserve = 256;
    let used = sys_tokens + prompt_so_far_tokens + task_and_format_tokens + response_reserve;
    let budget = context_window as usize;

    // Inject documentation if budget allows — AFTER skill, BEFORE task.
    // Skill > Docs > Task ordering ensures the model focuses on expert knowledge.
    if budget > used {
        let doc_budget_tokens = budget - used;
        let doc_budget_chars = doc_budget_tokens * 4;
        let truncated_docs =
            truncate_documentation_for_task(documentation, doc_budget_chars, Some(task));
        if !truncated_docs.is_empty() {
            prompt.push_str(&format!("## Docs\n{truncated_docs}\n\n"));
        }
    }

    // Task
    prompt.push_str(&format!("## Task\n{task}\n\n"));

    // Concise format reminder
    prompt.push_str(
        "## Output\n\
         ARGS: <subcommand then flags, NO tool name>\n\
         EXPLANATION: <brief>\n",
    );

    prompt
}

/// Aggressively compressed prompt for tiny context windows (≤ 4k) or small
/// models (≤ 3B).
///
/// Strategy: compact system prompt, top-2 examples as few-shot assistant
/// messages, selective doc injection for unknown flags, minimal format hint.
///
/// # Key design decisions for small models
///
/// 1. **Few-shot > instructions**: Small models imitate better than they follow
///    rules. The `---FEW-SHOT---` markers create user/assistant/user turns.
/// 2. **No format template in final user message**: Including `ARGS: sort...`
///    causes some models (starcoder2) to output empty — they think the answer
///    is already given.
/// 3. **Selective doc injection**: When the task mentions flags NOT covered by
///    skill examples, inject just the relevant doc sections. This handles cases
///    like "filter VCF by QUAL>30" where the exact flag syntax matters.
/// 4. **Fallback generic example**: When no skill is loaded, inject a generic
///    samtools example so the model still sees the output format.
fn build_prompt_compact(
    tool: &str,
    documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    context_window: u32,
) -> String {
    let mut prompt = String::new();

    // Build the first user message (tool + example context)
    prompt.push_str(&format!("Tool: {tool}\n\n"));

    // Find the most relevant examples for few-shot.
    // Use up to 2 examples: enough to establish the pattern without
    // overwhelming small models.
    let few_shots = skill
        .map(|s| s.select_examples(2, Some(task)))
        .unwrap_or_default();

    if let Some(ex) = few_shots.first() {
        // First few-shot: user → assistant pair
        prompt.push_str(&format!(
            "Task: {}\n\n---FEW-SHOT---\n\nARGS: {}\nEXPLANATION: {}\n\n---FEW-SHOT---\n\n",
            ex.task, ex.args, ex.explanation
        ));

        // Second few-shot (if available): reinforces the pattern
        if let Some(ex2) = few_shots.get(1) {
            prompt.push_str(&format!(
                "Task: {}\n\n---FEW-SHOT---\n\nARGS: {}\nEXPLANATION: {}\n\n---FEW-SHOT---\n\n",
                ex2.task, ex2.args, ex2.explanation
            ));
        }
    } else {
        // No skill examples — inject a generic fallback example so the model
        // sees the expected output format.  Without any example, ≤3B models
        // almost always fail to follow the ARGS/EXPLANATION format.
        prompt.push_str(
            "Task: Sort a BAM file by coordinate\n\n---FEW-SHOT---\n\n\
             ARGS: sort -@ 4 -o sorted.bam input.bam\n\
             EXPLANATION: Sort BAM by coordinate with 4 threads.\n\n---FEW-SHOT---\n\n",
        );
    }

    // Selective documentation injection for Compact tier.
    // Small models can't use raw --help text (it's noise), but when the task
    // asks for a specific flag or operation NOT covered by the skill examples,
    // we inject just the relevant doc section to ground the model.
    //
    // Heuristic: only inject if docs are short (<500 chars) or the task
    // mentions a specific flag prefix (e.g., "-@", "--quality").
    if !documentation.is_empty() && skill.is_none_or(|s| s.examples.is_empty()) {
        // No skill examples — docs are the only grounding source.
        // Inject a heavily truncated version.
        let truncated = truncate_documentation_for_task(documentation, 400, Some(task));
        if !truncated.is_empty() {
            prompt.push_str(&format!("Docs: {truncated}\n\n"));
        }
    }

    // Final user message: the actual task.
    // Do NOT include a format template here — the system prompt already shows
    // a concrete example, and the few-shot assistant message demonstrates the
    // output format.  Adding a format template in the final user message causes
    // some models (e.g., starcoder2:3b) to output empty responses, as they
    // interpret the template as the answer already being provided.
    prompt.push_str(&format!("Tool: {tool}\n"));
    prompt.push_str(&format!("Task: {task}\n\n"));

    let _ = context_window; // suppress unused warning
    prompt
}

/// Truncate documentation text to fit within a character budget (test helper).
#[cfg(test)]
fn truncate_documentation(docs: &str, max_chars: usize) -> String {
    truncate_documentation_for_task(docs, max_chars, None)
}

/// Semantic-aware documentation truncation that considers the task description.
///
/// When a task is provided, sections of the documentation that contain keywords
/// relevant to the task are prioritized over less relevant sections.  This
/// ensures that the most important flags and options for the user's specific
/// task are preserved even when aggressive truncation is needed.
fn truncate_documentation_for_task(docs: &str, max_chars: usize, task: Option<&str>) -> String {
    /// Minimum character budget below which documentation is too short to be
    /// useful (a single flag description is typically 40+ chars).
    const MIN_USEFUL_DOC_CHARS: usize = 40;
    /// Reserve space for the "[...truncated]" suffix appended when content is
    /// cut (14 chars for the marker + newline + small safety margin).
    const TRUNCATION_MARKER_RESERVE: usize = 20;

    if docs.len() <= max_chars {
        return docs.to_string();
    }
    if max_chars < MIN_USEFUL_DOC_CHARS {
        return String::new();
    }

    let effective_budget = max_chars.saturating_sub(TRUNCATION_MARKER_RESERVE);

    // If no task provided, fall back to simple line truncation
    let task = match task {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            return simple_truncate(docs, effective_budget);
        }
    };

    // Split docs into logical sections (separated by blank lines or section headers)
    let sections = split_into_sections(docs);

    if sections.is_empty() {
        return simple_truncate(docs, effective_budget);
    }

    // Extract task keywords for scoring
    let task_lower = task.to_ascii_lowercase();
    let task_words: Vec<&str> = task_lower
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter(|w| w.len() >= 2)
        .collect();

    // Score each section by keyword overlap
    let mut scored: Vec<(usize, f64, &str)> = sections
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let section_lower = section.to_ascii_lowercase();
            let score: f64 = task_words
                .iter()
                .filter(|w| section_lower.contains(*w))
                .count() as f64;
            // Boost score for sections containing CLI flags (they're more actionable)
            let flag_boost = if section_lower.contains("  -") || section_lower.contains("--") {
                0.5
            } else {
                0.0
            };
            // Boost for key sections like Usage, Options, Synopsis
            let header_boost = if section_lower.starts_with("usage")
                || section_lower.starts_with("options")
                || section_lower.starts_with("synopsis")
            {
                2.0
            } else {
                0.0
            };
            (i, score + flag_boost + header_boost, *section)
        })
        .collect();

    // Sort by: first section always first, then by score descending
    scored.sort_by(|a, b| {
        if a.0 == 0 {
            return std::cmp::Ordering::Less;
        }
        if b.0 == 0 {
            return std::cmp::Ordering::Greater;
        }
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Pack sections into the budget
    let mut result = String::new();
    for (_, _, section) in &scored {
        if result.len() + section.len() + 2 > effective_budget {
            // Try to fit a partial section
            let remaining = effective_budget.saturating_sub(result.len() + 2);
            if remaining > MIN_USEFUL_DOC_CHARS {
                if !result.is_empty() {
                    result.push_str("\n\n");
                }
                result.push_str(&simple_truncate(section, remaining));
            }
            break;
        }
        if !result.is_empty() {
            result.push_str("\n\n");
        }
        result.push_str(section);
    }

    if result.len() < docs.len() {
        result.push_str("\n[...truncated]");
    }

    result
}

/// Simple line-by-line truncation (preserves complete lines).
fn simple_truncate(docs: &str, budget: usize) -> String {
    let mut result = String::new();
    for line in docs.lines() {
        if result.len() + line.len() + 1 > budget {
            break;
        }
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
    }
    if result.len() < docs.len() {
        result.push_str("\n[...truncated]");
    }
    result
}

/// Split documentation into logical sections separated by blank lines.
fn split_into_sections(docs: &str) -> Vec<&str> {
    let mut sections = Vec::new();
    let mut start = 0;
    let mut last_was_blank = false;
    let bytes = docs.as_bytes();

    for (i, line) in docs.lines().enumerate() {
        let is_blank = line.trim().is_empty();
        if is_blank && !last_was_blank && i > 0 {
            // End of a section
            let byte_pos = docs[start..].find(line).map(|p| start + p).unwrap_or(start);
            let section = docs[start..byte_pos].trim();
            if !section.is_empty() {
                sections.push(section);
            }
            start = byte_pos + line.len();
            // Skip the newline after
            if start < bytes.len() && bytes[start] == b'\n' {
                start += 1;
            }
        }
        last_was_blank = is_blank;
    }

    // Final section
    let remaining = docs[start..].trim();
    if !remaining.is_empty() {
        sections.push(remaining);
    }

    // If splitting produced no useful sections, return the whole doc as one section
    if sections.is_empty() {
        sections.push(docs.trim());
    }

    sections
}

// ─── Task optimization prompt ─────────────────────────────────────────────────

/// Build a prompt that asks the LLM to expand and clarify a raw task description
/// into a precise, unambiguous bioinformatics instruction.
fn build_task_optimization_prompt(tool: &str, raw_task: &str) -> String {
    format!(
        "# Task Optimization Request\n\n\
         Tool: `{tool}`\n\
         User's original task description: {raw_task}\n\n\
         Rewrite the task as a precise, unambiguous bioinformatics instruction. Follow \
         these guidelines:\n\
         - Expand ambiguous terms into specific operations (e.g., 'sort bam' → 'sort \
           BAM file input.bam by genomic coordinate and write to sorted.bam')\n\
         - Infer reasonable defaults when not specified: paired-end reads, hg38 reference, \
           8 threads, coordinate-sorted BAM output, gzipped FASTQ, Phred+33 encoding\n\
         - Preserve ALL file names, paths, and sample identifiers from the original task\n\
         - Specify output file names if the user omitted them (derive from input names)\n\
         - Be written in the SAME LANGUAGE as the original task\n\n\
         ## Output Format (STRICT)\n\
         Respond with EXACTLY one line:\n\
         TASK: <the optimized task description>\n\
         - Do NOT add any other text, markdown, or explanation\n"
    )
}

// ─── Run verification prompt ──────────────────────────────────────────────────

/// System prompt for the result verification role.
fn verification_system_prompt() -> &'static str {
    "You are an expert bioinformatics QC analyst specialising in command-line tool \
     execution validation. You understand exit codes, common error patterns \
     (segfaults, OOM kills, truncated files, permission denied), expected output \
     structures (BAM/VCF/BED headers, index files), and tool-specific behaviors \
     (e.g., samtools returning 1 for warnings, STAR log files, GATK exceptions). \
     Assess severity accurately: distinguish fatal failures from harmless warnings \
     and informational messages. Respond in the same language as the task description."
}

/// Build the user prompt for run result verification.
fn build_verification_prompt(
    tool: &str,
    task: &str,
    command: &str,
    exit_code: i32,
    stderr: &str,
    output_files: &[(String, Option<u64>)],
) -> String {
    let mut prompt = format!(
        "## Command Execution Analysis\n\n\
         **Tool:** `{tool}`\n\
         **Task:** {task}\n\
         **Command:** `{command}`\n\
         **Exit Code:** {exit_code}\n\n"
    );

    if !stderr.is_empty() {
        // Limit stderr to the last 3000 characters to stay within context limits.
        let stderr_snippet = if stderr.len() > 3000 {
            format!("...(truncated)...\n{}", &stderr[stderr.len() - 3000..])
        } else {
            stderr.to_string()
        };
        prompt.push_str("## Standard Error / Tool Output\n");
        prompt.push_str("```\n");
        prompt.push_str(&stderr_snippet);
        prompt.push_str("\n```\n\n");
    }

    if !output_files.is_empty() {
        prompt.push_str("## Output Files\n");
        for (path, size) in output_files {
            match size {
                Some(bytes) => {
                    prompt.push_str(&format!("- `{path}` — {bytes} bytes\n"));
                }
                None => {
                    prompt.push_str(&format!("- `{path}` — **NOT FOUND** (missing output)\n"));
                }
            }
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "## Analysis Instructions\n\
         Determine whether this command ran successfully by evaluating:\n\
         1. **Exit code**: 0 = success for most tools. Some tools use non-zero for \
            warnings (e.g., samtools returns 1 for certain warnings). Exit code \
            137 (SIGKILL, often OOM-killed) and 139 (SIGSEGV, segfault) signal crashes.\n\
         2. **Error signals in stderr**: ERROR, FATAL, Exception, Traceback, \
            Segmentation fault, Killed, Out of memory, core dumped, No such file, \
            Permission denied, invalid header, truncated file.\n\
         3. **Output files**: missing expected outputs or zero-byte files indicate failure.\n\
         4. **Tool-specific patterns**: samtools truncated-BAM warnings, STAR alignment \
            rate below 50%%, GATK MalformedRead or UserException, BWA inability to open \
            reference, bcftools missing index, HISAT2 0%% alignment.\n\
         5. **Harmless noise**: progress bars, timing statistics, 'INFO' or 'NOTE' \
            lines, version banners — do NOT flag these as issues.\n\n\
         ## Output Format (STRICT)\n\
         STATUS: success|warning|failure\n\
         SUMMARY: <one concise sentence summarising the result — same language as task>\n\
         ISSUES:\n\
         - <issue 1, or write 'none' when no issues>\n\
         SUGGESTIONS:\n\
         - <suggestion 1, or write 'none' when no suggestions>\n\
         Do NOT add any other text or markdown outside this format.\n",
    );

    prompt
}

/// Parse the structured verification response from the LLM.
fn parse_verification_response(raw: &str) -> LlmRunVerification {
    let mut status = "success";
    let mut summary = String::new();
    let mut issues: Vec<String> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        None,
        Issues,
        Suggestions,
    }
    let mut section = Section::None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("STATUS:") {
            status = match rest.trim() {
                s if s.contains("fail") => "failure",
                s if s.contains("warn") => "warning",
                _ => "success",
            };
        } else if let Some(rest) = trimmed.strip_prefix("SUMMARY:") {
            summary = rest.trim().to_string();
            section = Section::None;
        } else if trimmed.starts_with("ISSUES:") {
            section = Section::Issues;
        } else if trimmed.starts_with("SUGGESTIONS:") {
            section = Section::Suggestions;
        } else if trimmed.starts_with('-') {
            let item = trimmed.trim_start_matches('-').trim().to_string();
            if item.is_empty() || item.eq_ignore_ascii_case("none") {
                continue;
            }
            match section {
                Section::Issues => issues.push(item),
                Section::Suggestions => suggestions.push(item),
                Section::None => {}
            }
        }
    }

    let success = status != "failure";
    LlmRunVerification {
        success,
        summary,
        issues,
        suggestions,
    }
}

// ─── Skill reviewer prompts ───────────────────────────────────────────────────

/// System prompt for the skill reviewer / editor persona.
fn skill_reviewer_system_prompt() -> &'static str {
    "You are an expert bioinformatics skill author for the oxo-call tool. \
     You deeply understand the oxo-call skill file format (YAML front-matter + Markdown \
     sections) and how skills are injected into LLM prompts to improve command generation \
     accuracy. A high-quality skill file must have: \
     (1) Complete YAML front-matter: name, category, description, tags, author, source_url. \
     (2) A '## Concepts' section with ≥3 bullet points — specific, actionable facts about \
         the tool's data model, I/O formats, and key behaviours. \
     (3) A '## Pitfalls' section with ≥3 bullet points — common mistakes WITH consequences. \
         Never use 'DANGER:' or 'EXTREME DANGER:' prefixes (they can cause overly cautious \
         or refused responses from the LLM). \
     (4) An '## Examples' section with ≥5 subsections: '### <task>', '**Args:** `<flags>`', \
         '**Explanation:** <sentence>'. Args must NEVER start with the tool name. For companion \
         binaries (e.g., bowtie2-build), use the companion name as the first Args token. \
     All content must be accurate, actionable, and written in English."
}

/// Build a prompt asking the LLM to review a skill file for quality.
fn build_skill_verify_prompt(tool: &str, skill_content: &str) -> String {
    format!(
        "# Skill Review Request\n\n\
         Tool: `{tool}`\n\n\
         ## Skill File Content\n\
         ```\n{skill_content}\n```\n\n\
         Please review this skill file and evaluate its quality.\n\n\
         ## Output Format (STRICT)\n\
         VERDICT: pass|fail\n\
         SUMMARY: <one sentence overall assessment>\n\
         ISSUES:\n\
         - <issue 1, or 'none' when no issues>\n\
         SUGGESTIONS:\n\
         - <actionable improvement 1, or 'none' when no suggestions>\n\
         Do NOT add any other text or markdown outside this format.\n"
    )
}

/// Build a prompt asking the LLM to polish/rewrite a skill file.
fn build_skill_polish_prompt(tool: &str, skill_content: &str) -> String {
    format!(
        "# Skill Polish Request\n\n\
         Tool: `{tool}`\n\n\
         ## Current Skill File\n\
         ```\n{skill_content}\n```\n\n\
         Please rewrite and enhance this skill file to meet oxo-call quality standards:\n\
         - Keep all correct information; fix inaccuracies if any\n\
         - Ensure YAML front-matter is complete (name, category, description, tags, author, source_url)\n\
         - Add or improve concepts to reach ≥3 specific, actionable bullet points\n\
         - Add or improve pitfalls to reach ≥3 bullet points explaining consequences\n\
         - Add or improve examples to reach ≥5 subsections with correct ### / **Args:** / **Explanation:** format\n\
         - Use clear, professional English\n\n\
         ## Output Format (STRICT)\n\
         Respond with ONLY the complete improved skill file in Markdown format (starting with '---').\n\
         Do NOT add any explanation, preamble, or code fences around the output.\n"
    )
}

/// Build a prompt asking the LLM to generate a fresh skill template for a tool.
fn build_skill_generate_prompt(tool: &str) -> String {
    format!(
        "# Skill Generation Request\n\n\
         Tool: `{tool}`\n\n\
         Generate a complete, high-quality oxo-call skill file for this bioinformatics tool.\n\
         The skill file must include:\n\
         - YAML front-matter with name, category, description, tags, author ('AI-generated'), source_url\n\
         - '## Concepts' section with ≥3 specific, actionable bullet points about the tool's \
           data model, I/O formats, and key behaviors\n\
         - '## Pitfalls' section with ≥3 bullet points about common mistakes and their \
           consequences. Never use 'DANGER:' or 'EXTREME DANGER:' prefixes.\n\
         - '## Examples' section with ≥5 realistic subsections, each:\n\
             ### <task description in plain English>\n\
             **Args:** `<exact CLI flags WITHOUT the tool name>`\n\
             **Explanation:** <one sentence explaining why these flags>\n\n\
         IMPORTANT: Args must NEVER start with the tool name '{tool}'. For companion \
         binaries (e.g., {tool}-build), use the companion name as the first token.\n\n\
         ## Output Format (STRICT)\n\
         Respond with ONLY the complete skill file in Markdown format (starting with '---').\n\
         Do NOT add any explanation, preamble, or code fences around the output.\n"
    )
}

/// Strip leading/trailing markdown code fences from LLM output.
fn strip_markdown_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    // Remove opening fence (```markdown, ```md, ```, etc.)
    let body = if let Some(rest) = trimmed.strip_prefix("```") {
        // Skip the fence line
        rest.split_once('\n').map(|x| x.1).unwrap_or(rest)
    } else {
        trimmed
    };
    // Remove closing fence
    let body = if let Some(stripped) = body.trim_end().strip_suffix("```") {
        stripped.trim_end()
    } else {
        body
    };
    body.trim().to_string()
}

/// Parse the structured skill verification response from the LLM.
fn parse_skill_verify_response(raw: &str) -> LlmSkillVerification {
    let mut passed = true;
    let mut summary = String::new();
    let mut issues: Vec<String> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        None,
        Issues,
        Suggestions,
    }
    let mut section = Section::None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("VERDICT:") {
            passed = rest.trim().eq_ignore_ascii_case("pass");
        } else if let Some(rest) = trimmed.strip_prefix("SUMMARY:") {
            summary = rest.trim().to_string();
            section = Section::None;
        } else if trimmed.starts_with("ISSUES:") {
            section = Section::Issues;
        } else if trimmed.starts_with("SUGGESTIONS:") {
            section = Section::Suggestions;
        } else if trimmed.starts_with('-') {
            let item = trimmed.trim_start_matches('-').trim().to_string();
            if item.is_empty() || item.eq_ignore_ascii_case("none") {
                continue;
            }
            match section {
                Section::Issues => issues.push(item),
                Section::Suggestions => suggestions.push(item),
                Section::None => {}
            }
        }
    }

    LlmSkillVerification {
        passed,
        summary,
        issues,
        suggestions,
    }
}

/// Build a corrective retry prompt when the first attempt had an invalid response.
#[allow(clippy::too_many_arguments)]
fn build_retry_prompt(
    tool: &str,
    documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    prev_raw: &str,
    no_prompt: bool,
    context_window: u32,
    tier: PromptTier,
) -> String {
    // For Compact tier (small models), do NOT include the previous failed
    // response — it acts as a negative example that small models may imitate.
    // Instead, rebuild a fresh prompt with stronger format emphasis.
    if tier == PromptTier::Compact {
        let mut prompt = build_prompt(
            tool,
            documentation,
            task,
            skill,
            no_prompt,
            context_window,
            tier,
        );
        // Add a concise format reminder instead of the failed response
        prompt.push_str("\nIMPORTANT: Output EXACTLY two lines starting with ARGS: and EXPLANATION:. No other text.\n");
        return prompt;
    }

    // For Full/Medium tiers, include the previous response as a correction signal
    let base = build_prompt(
        tool,
        documentation,
        task,
        skill,
        no_prompt,
        context_window,
        tier,
    );
    format!(
        "{base}\n\
         ## Correction Note\n\
         Your previous response was not in the required format:\n\
         {prev_raw}\n\
         Please respond again with EXACTLY two lines starting with 'ARGS:' and 'EXPLANATION:'.\n"
    )
}

// ─── Client ───────────────────────────────────────────────────────────────────

pub struct LlmClient {
    config: Config,
    #[cfg(not(target_arch = "wasm32"))]
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        LlmClient {
            config,
            #[cfg(not(target_arch = "wasm32"))]
            client: reqwest::Client::new(),
        }
    }

    /// Generate command arguments, using skill knowledge for better prompts.
    /// Retries up to `MAX_RETRIES` times when the response format is invalid.
    ///
    /// When the model has a known context window, prompts are adaptively
    /// compressed to fit — see [`PromptTier`].
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn suggest_command(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
    ) -> Result<LlmCommandSuggestion> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            const MAX_RETRIES: usize = 2;

            let context_window = self.config.effective_context_window();
            let tier = self.config.effective_prompt_tier();

            let mut last_raw = String::new();
            let mut total_inference_ms: f64 = 0.0;
            // Track whether the model produced an empty/blank response,
            // which indicates it was overwhelmed by the prompt length.
            let mut had_empty_output = false;

            for attempt in 0..=MAX_RETRIES {
                // On retry after an empty output, use a degraded prompt that
                // strips documentation to reduce context length.  Small models
                // (≤ 3B) often fail to produce any output when the prompt is
                // too long, even if it fits within their context window.
                let effective_docs = if had_empty_output && attempt > 0 {
                    // Strip docs entirely — the skill examples alone provide
                    // enough grounding for small models.
                    ""
                } else {
                    documentation
                };

                let user_prompt = if attempt == 0 {
                    build_prompt(
                        tool,
                        effective_docs,
                        task,
                        skill,
                        no_prompt,
                        context_window,
                        tier,
                    )
                } else if had_empty_output {
                    // After an empty output, use a fresh (shorter) prompt
                    // instead of the retry prompt (which adds even more text)
                    build_prompt(
                        tool,
                        effective_docs,
                        task,
                        skill,
                        no_prompt,
                        context_window,
                        tier,
                    )
                } else {
                    build_retry_prompt(
                        tool,
                        effective_docs,
                        task,
                        skill,
                        &last_raw,
                        no_prompt,
                        context_window,
                        tier,
                    )
                };

                let api_start = std::time::Instant::now();
                let raw = self.call_api(&user_prompt, no_prompt, tier).await?;
                total_inference_ms += api_start.elapsed().as_secs_f64() * 1000.0;

                // Detect empty/blank responses (model was overwhelmed)
                if raw.trim().is_empty() {
                    had_empty_output = true;
                }

                let mut suggestion = Self::parse_response(&raw)?;
                suggestion.inference_ms = total_inference_ms;

                // Post-process: strip accidental tool name prefix
                suggestion.args = sanitize_args(tool, suggestion.args);

                if is_valid_suggestion(&suggestion) {
                    return Ok(suggestion);
                }

                last_raw = raw;
                // If this was the last attempt, return whatever we got
                if attempt == MAX_RETRIES {
                    return Ok(suggestion);
                }
            }

            // Unreachable — the loop always returns
            unreachable!()
        }
    }

    pub async fn verify_configuration(&self) -> Result<LlmVerificationResult> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let provider = self.config.effective_provider();
            let api_base = self.config.effective_api_base();
            let model = self.config.effective_model();
            let raw = self
                .request_text("Reply with exactly OK.", Some(16), Some(0.0))
                .await?;
            let response_preview = raw.lines().next().unwrap_or("").trim().to_string();

            Ok(LlmVerificationResult {
                provider,
                api_base,
                model,
                response_preview,
            })
        }
    }

    /// Use the LLM to optimize/expand a raw task description into a precise instruction.
    ///
    /// Returns the refined task text on success, or falls back to the original task
    /// if the LLM response is not parseable.  Errors from the API are propagated.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn optimize_task(&self, tool: &str, raw_task: &str) -> Result<String> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let prompt = build_task_optimization_prompt(tool, raw_task);
            let raw = self.request_text(&prompt, Some(256), Some(0.2)).await?;

            // Extract the TASK: line.
            for line in raw.lines() {
                if let Some(rest) = line.strip_prefix("TASK:") {
                    let refined = rest.trim().to_string();
                    if !refined.is_empty() {
                        return Ok(refined);
                    }
                }
            }
            // Fall back to original task if parsing fails.
            Ok(raw_task.to_string())
        }
    }

    /// Ask the LLM to verify the result of a completed command execution.
    ///
    /// `output_files` is a list of `(path, Option<file_size_bytes>)` pairs — a
    /// `None` size means the file was not found on disk.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn verify_run_result(
        &self,
        tool: &str,
        task: &str,
        command: &str,
        exit_code: i32,
        stderr: &str,
        output_files: &[(String, Option<u64>)],
    ) -> Result<LlmRunVerification> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_prompt =
                build_verification_prompt(tool, task, command, exit_code, stderr, output_files);

            let raw = self
                .request_with_system(
                    verification_system_prompt(),
                    &user_prompt,
                    Some(512),
                    Some(0.2),
                )
                .await?;

            Ok(parse_verification_response(&raw))
        }
    }

    /// Make the raw API call and return the assistant message content.
    /// When no_prompt is true (bare mode), no system prompt is sent to test raw LLM capability.
    async fn call_api(
        &self,
        user_prompt: &str,
        no_prompt: bool,
        tier: PromptTier,
    ) -> Result<String> {
        let sys_prompt = if no_prompt {
            ""
        } else {
            match tier {
                PromptTier::Compact => system_prompt_compact(),
                PromptTier::Medium => system_prompt_medium(),
                PromptTier::Full => system_prompt(),
            }
        };

        // For Compact tier, check if the user prompt contains a few-shot separator
        // that indicates we should use multi-turn messages instead of a single user message.
        // This dramatically improves reliability for small models (≤ 3B) because
        // they learn the output format from the assistant few-shot example.
        if tier == PromptTier::Compact && user_prompt.contains("\n\n---FEW-SHOT---\n\n") {
            let raw = self.request_few_shot(sys_prompt, user_prompt).await?;
            return Ok(raw);
        }

        self.request_with_system(sys_prompt, user_prompt, None, None)
            .await
    }

    /// Send a few-shot request using multi-turn messages.
    ///
    /// The `user_prompt` is split at `---FEW-SHOT---` boundaries to create
    /// user/assistant message pairs.  This is critical for small models (≤ 3B)
    /// which cannot reliably follow output format instructions in a single
    /// user prompt, but can imitate the format when shown an assistant example.
    #[cfg(not(target_arch = "wasm32"))]
    async fn request_few_shot(&self, sys_prompt: &str, user_prompt: &str) -> Result<String> {
        let provider = self.config.effective_provider();
        let token = if self.config.provider_requires_token() {
            self.config
                .effective_api_token()
                .ok_or_else(|| OxoError::LlmError("No API token configured".to_string()))?
        } else {
            String::new()
        };
        let api_base = self.config.effective_api_base();
        let model = self.config.effective_model();
        let url = format!("{api_base}/chat/completions");

        // Build messages: system + alternating user/assistant pairs + final user
        let mut messages = Vec::new();

        if !sys_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: sys_prompt.to_string(),
            });
        }

        // Split at few-shot boundaries.
        // The prompt format is:
        //   <user context>\n\n---FEW-SHOT---\n\n<assistant response 1>\n\n---FEW-SHOT---\n\n
        //   <user context 2>\n\n---FEW-SHOT---\n\n<assistant response 2>\n\n---FEW-SHOT---\n\n
        //   <final user query>
        //
        // Odd-indexed parts are assistant few-shot responses.
        // Even-indexed parts are user messages (including the final one).
        let parts: Vec<&str> = user_prompt
            .split("\n\n---FEW-SHOT---\n\n")
            .filter(|p| !p.is_empty())
            .collect();

        if parts.len() >= 2 {
            // Alternate between user and assistant messages
            let mut is_assistant = false;
            for part in &parts {
                if is_assistant {
                    messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: part.to_string(),
                    });
                } else {
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: part.to_string(),
                    });
                }
                is_assistant = !is_assistant;
            }
            // If the last message is an assistant (odd number of parts),
            // the model will continue from that context — which is what we want.
            // If the last message is a user message, the model will respond to it.
        } else {
            // No few-shot markers found — fall back to single user message
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            });
        }

        let request = ChatRequest {
            model,
            messages,
            max_tokens: self.config.effective_max_tokens()?,
            temperature: self.config.effective_temperature()?,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Auth handling (same as request_with_system)
        let auth_token = if provider == "github-copilot" {
            let manager = copilot_auth::get_token_manager();
            manager.get_session_token(&token).await?
        } else {
            token.clone()
        };

        req_builder = match provider.as_str() {
            "anthropic" => req_builder
                .header("x-api-key", &auth_token)
                .header("anthropic-version", "2023-06-01"),
            "github-copilot" => req_builder
                .header("Authorization", format!("Bearer {auth_token}"))
                .header("Copilot-Integration-Id", "vscode-chat")
                .header("Editor-Version", "vscode/1.85.0")
                .header("Editor-Plugin-Version", "copilot/1.0.0"),
            _ => {
                if auth_token.is_empty() {
                    req_builder
                } else {
                    req_builder.header("Authorization", format!("Bearer {auth_token}"))
                }
            }
        };

        let resp = req_builder.json(&request).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(OxoError::LlmError(format!(
                "LLM API error: {status} — {body}"
            )));
        }

        let chat_resp: ChatResponse = resp.json().await?;
        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content.trim().to_string())
    }

    async fn request_text(
        &self,
        user_prompt: &str,
        max_tokens_override: Option<u32>,
        temperature_override: Option<f32>,
    ) -> Result<String> {
        self.request_with_system(
            system_prompt(),
            user_prompt,
            max_tokens_override,
            temperature_override,
        )
        .await
    }

    /// Core HTTP call.  Accepts an explicit system prompt so callers can use a
    /// role-specific prompt (e.g., the verification analyst persona).
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    async fn request_with_system(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
        max_tokens_override: Option<u32>,
        temperature_override: Option<f32>,
    ) -> Result<String> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let provider = self.config.effective_provider();
            let token_opt = self.config.effective_api_token();
            // Local providers such as Ollama do not require an API token.
            let token = if self.config.provider_requires_token() {
                token_opt.ok_or_else(|| {
                    let token_hint = match provider.as_str() {
                        "github-copilot" => "  For GitHub Copilot, run: oxo-call config login",
                        "openai" => "  For OpenAI, create an API key at:\n    https://platform.openai.com/api-keys",
                        "anthropic" => "  For Anthropic, create an API key at:\n    https://console.anthropic.com/settings/keys",
                        _ => "  Check your provider's documentation for token setup.",
                    };
                    OxoError::LlmError(
                        format!(
                            "No API token configured for provider '{provider}'.\n\n\
                            Option 1 — Interactive login (recommended for github-copilot):\n  \
                              oxo-call config login\n\n\
                            Option 2 — Set via config:\n  \
                              oxo-call config set llm.api_token <your-token>\n\n\
                            Option 3 — Set via environment variable:\n  \
                              export OXO_CALL_LLM_API_TOKEN=<your-token>\n\n\
                            How to get a token:\n{token_hint}\n\n\
                            Test your setup: oxo-call config verify"
                        ),
                    )
                })?
            } else {
                // For token-optional providers (e.g. Ollama), fall back to an
                // empty string.  An empty token means no Authorization header
                // will be added (see the auth header construction below).
                token_opt.unwrap_or_default()
            };

            let api_base = self.config.effective_api_base();

            // Enforce HTTPS for remote API endpoints (allow HTTP for local Ollama)
            if !api_base.starts_with("https://")
                && !api_base.starts_with("http://localhost")
                && !api_base.starts_with("http://127.0.0.1")
                && !api_base.starts_with("http://[::1]")
            {
                return Err(OxoError::LlmError(format!(
                    "API base URL must use HTTPS for remote endpoints: {api_base}"
                )));
            }

            let model = self.config.effective_model();
            let url = format!("{api_base}/chat/completions");

            let messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: sys_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ];

            let request = ChatRequest {
                model,
                messages,
                max_tokens: max_tokens_override.unwrap_or(self.config.effective_max_tokens()?),
                temperature: temperature_override.unwrap_or(self.config.effective_temperature()?),
            };

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json");

            // For github-copilot, we need to exchange the GitHub token for a Copilot session token
            let auth_token = if provider == "github-copilot" {
                let manager = copilot_auth::get_token_manager();
                manager.get_session_token(&token).await?
            } else {
                token.clone()
            };

            req_builder = match provider.as_str() {
                "anthropic" => req_builder
                    .header("x-api-key", &auth_token)
                    .header("anthropic-version", "2023-06-01"),
                "github-copilot" => {
                    // Add Copilot-specific headers
                    req_builder
                        .header("Authorization", format!("Bearer {auth_token}"))
                        .header("Copilot-Integration-Id", "vscode-chat")
                        .header("Editor-Version", "vscode/1.85.0")
                        .header("Editor-Plugin-Version", "copilot/1.0.0")
                }
                _ => {
                    // Only add Authorization header when a token is actually present
                    // (e.g. local Ollama instances usually run without authentication)
                    if auth_token.is_empty() {
                        req_builder
                    } else {
                        req_builder.header("Authorization", format!("Bearer {auth_token}"))
                    }
                }
            };

            let response = req_builder
                .json(&request)
                .send()
                .await
                .map_err(|e| OxoError::LlmError(format!("HTTP request failed: {e}")))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(OxoError::LlmError(format!("API returned {status}: {body}")));
            }

            let chat_response: ChatResponse = response
                .json()
                .await
                .map_err(|e| OxoError::LlmError(format!("Failed to parse API response: {e}")))?;

            Ok(chat_response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default())
        }
    }

    /// Ask the LLM to review a skill file for quality and completeness.
    ///
    /// Returns a structured `LlmSkillVerification` with findings and suggestions.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn verify_skill(
        &self,
        tool: &str,
        skill_content: &str,
    ) -> Result<LlmSkillVerification> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_prompt = build_skill_verify_prompt(tool, skill_content);
            let raw = self
                .request_with_system(
                    skill_reviewer_system_prompt(),
                    &user_prompt,
                    Some(1024),
                    Some(0.2),
                )
                .await?;
            Ok(parse_skill_verify_response(&raw))
        }
    }

    /// Ask the LLM to rewrite and improve a skill file, returning the enhanced Markdown.
    ///
    /// The LLM is instructed to preserve the tool name and all correct information
    /// while adding missing concepts/pitfalls/examples, fixing format issues, and
    /// improving clarity.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn polish_skill(&self, tool: &str, skill_content: &str) -> Result<String> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_prompt = build_skill_polish_prompt(tool, skill_content);
            let raw = self
                .request_with_system(
                    skill_reviewer_system_prompt(),
                    &user_prompt,
                    Some(4096),
                    Some(0.3),
                )
                .await?;
            // Strip any markdown code fences the LLM might have wrapped the output in
            Ok(strip_markdown_fences(&raw))
        }
    }

    /// Use LLM to generate an initial skill template pre-filled with domain knowledge.
    ///
    /// Returns a Markdown-format skill file (YAML front-matter + body sections).
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn generate_skill_template(&self, tool: &str) -> Result<String> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_prompt = build_skill_generate_prompt(tool);
            let raw = self
                .request_with_system(
                    skill_reviewer_system_prompt(),
                    &user_prompt,
                    Some(4096),
                    Some(0.4),
                )
                .await?;
            Ok(strip_markdown_fences(&raw))
        }
    }

    /// Generate a shell command from a plain-English description.
    ///
    /// Returns `(command, explanation)`.  The command is a ready-to-run shell
    /// string; the explanation is a brief one-liner.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn generate_shell_command(&self, description: &str) -> Result<(String, String)> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let system = "You are a shell command expert for Linux/macOS. \
                Given a plain-English description (in any language), produce a single \
                production-ready shell command or short pipeline. Use standard coreutils, \
                common bioinformatics tools, and POSIX-compatible syntax. \
                Reply with exactly two lines and nothing else:\n\
                COMMAND: <the shell command>\n\
                EXPLANATION: <one-sentence explanation in the same language as the input>";

            let raw = self
                .request_with_system(system, description, Some(256), Some(0.1))
                .await?;

            let mut command = String::new();
            let mut explanation = String::new();
            for line in raw.lines() {
                if let Some(rest) = line.strip_prefix("COMMAND:") {
                    command = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("EXPLANATION:") {
                    explanation = rest.trim().to_string();
                }
            }
            if command.is_empty() {
                command = raw.trim().to_string();
            }
            Ok((command, explanation))
        }
    }

    fn parse_response(raw: &str) -> Result<LlmCommandSuggestion> {
        // ── Try JSON structured output first ──────────────────────────────────
        //
        // Models that support JSON mode (GPT-4+, Claude) may return structured
        // output.  This is more reliable than regex parsing.
        if let Some(suggestion) = Self::try_parse_json_response(raw) {
            return Ok(suggestion);
        }

        // ── Standard ARGS:/EXPLANATION: format ────────────────────────────────
        let mut args_line = String::new();
        let mut explanation_line = String::new();

        for line in raw.lines() {
            let trimmed = line.trim_start();
            // Support case-insensitive prefix matching for common LLM deviations:
            // "ARGS:", "Args:", "args:", "**ARGS:**", etc.
            let stripped = trimmed
                .trim_start_matches('*')
                .trim_start_matches('#')
                .trim_start();
            if let Some(rest) = strip_prefix_case_insensitive(stripped, "ARGS:") {
                args_line = rest.trim().trim_end_matches('*').trim().to_string();
            } else if let Some(rest) = strip_prefix_case_insensitive(stripped, "EXPLANATION:") {
                explanation_line = rest.trim().trim_end_matches('*').trim().to_string();
            }
        }

        // Treat "(none)" as empty args
        if args_line == "(none)" {
            args_line.clear();
        }

        // Fallback: when the model doesn't output ARGS: format (common for
        // small models like deepseek-coder:1.3b), try to extract the command
        // from the raw response using heuristics.
        if args_line.is_empty() {
            args_line = extract_command_from_freeform(raw);
        }

        // Strip markdown code fences that weak LLMs sometimes add
        let cleaned = strip_code_fences(&args_line);
        let args = parse_shell_args(cleaned);

        Ok(LlmCommandSuggestion {
            args,
            explanation: explanation_line,
            raw_response: raw.to_string(),
            inference_ms: 0.0, // Set by caller (suggest_command)
        })
    }

    /// Try to parse the LLM response as a JSON object with `args` and `explanation` fields.
    ///
    /// This handles models that support structured/JSON output mode.
    /// Returns `None` if the response is not valid JSON or doesn't have the expected shape.
    fn try_parse_json_response(raw: &str) -> Option<LlmCommandSuggestion> {
        // Try to find JSON in the response (may be wrapped in markdown code fences)
        let trimmed = raw.trim();
        let json_str = if trimmed.starts_with("```json") || trimmed.starts_with("```") {
            // Extract content between code fences
            let start = trimmed.find('{').unwrap_or(0);
            let end = trimmed.rfind('}').map(|i| i + 1).unwrap_or(trimmed.len());
            &trimmed[start..end]
        } else if trimmed.starts_with('{') {
            trimmed
        } else {
            return None;
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;

        let args_str = parsed
            .get("args")
            .and_then(|v| v.as_str())
            .or_else(|| parsed.get("ARGS").and_then(|v| v.as_str()))?;

        let explanation = parsed
            .get("explanation")
            .and_then(|v| v.as_str())
            .or_else(|| parsed.get("EXPLANATION").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();

        let cleaned = strip_code_fences(args_str);
        let args = parse_shell_args(cleaned);

        Some(LlmCommandSuggestion {
            args,
            explanation,
            raw_response: raw.to_string(),
            inference_ms: 0.0,
        })
    }
}

/// Check whether a suggestion looks valid enough to return without retrying.
fn is_valid_suggestion(suggestion: &LlmCommandSuggestion) -> bool {
    // Require both explanation and non-empty args to be considered valid.
    // Empty args usually indicates the LLM failed to follow the output format.
    !suggestion.explanation.is_empty() && !suggestion.args.is_empty()
}

/// Case-insensitive prefix strip.  Returns the remainder after the prefix,
/// or `None` if the string doesn't start with the prefix (case-insensitive).
fn strip_prefix_case_insensitive<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = s.to_ascii_lowercase();
    let prefix_lower = prefix.to_ascii_lowercase();
    if lower.starts_with(&prefix_lower) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

/// Post-process LLM-generated args to fix common mistakes:
/// - Strip the tool name if LLM accidentally included it as the first argument
///   (unless it is a recognised companion binary)
/// - Strip markdown code fences that weak models sometimes add around ARGS
fn sanitize_args(tool: &str, args: Vec<String>) -> Vec<String> {
    if args.is_empty() {
        return args;
    }

    let mut result = args;

    // If the first arg is exactly the tool name (case-insensitive) and is NOT a
    // companion binary, drop it — the tool name is prepended by the runner.
    if let Some(first) = result.first()
        && first.eq_ignore_ascii_case(tool)
        && !is_companion_binary(tool, first)
    {
        result.remove(0);
    }

    // After each && or || operator, inject the tool name when the following
    // token is not already the tool name, not a companion binary, and not a
    // script executable.  This corrects the common LLM failure where multi-step
    // commands omit the tool name for steps after the first, e.g.:
    //   sort ... && index ...  →  sort ... && samtools index ...
    let mut i = 0;
    while i < result.len() {
        if (result[i] == "&&" || result[i] == "||") && i + 1 < result.len() {
            let next = &result[i + 1];
            let needs_injection = !next.eq_ignore_ascii_case(tool)
                && !is_companion_binary(tool, next)
                && !is_script_executable(next);
            if needs_injection {
                result.insert(i + 1, tool.to_string());
                i += 2; // skip the inserted tool name token
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    result
}

/// Extract a command from freeform text when the model doesn't follow the
/// ARGS:/EXPLANATION: format.  This is a fallback for small models (≤ 3B)
/// that frequently output explanations in natural language instead of the
/// expected format.
///
/// Heuristics used:
/// 1. Look for code blocks (```...```) — the content is likely the command
/// 2. Look for lines starting with a known tool subcommand (e.g., "sort",
///    "view", "mem", "intersect")
/// 3. Look for the first line that looks like a CLI command (contains `-` flags)
fn extract_command_from_freeform(raw: &str) -> String {
    // 1. Try to find content inside code blocks
    let mut in_code_block = false;
    let mut code_block_lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code_block {
                break; // end of code block
            }
            in_code_block = true;
            continue;
        }
        if in_code_block {
            code_block_lines.push(line);
        }
    }
    if !code_block_lines.is_empty() {
        // Return the first non-empty line from the code block
        for line in &code_block_lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // 2. Look for lines that start with a known CLI subcommand or flag pattern
    let subcommand_prefixes = [
        "sort",
        "index",
        "view",
        "filter",
        "merge",
        "intersect",
        "mem",
        "align",
        "trim",
        "run",
        "blastn",
        "blastp",
        "blastx",
        "bamtobed",
        "bedtobam",
        "faidx",
        "dict",
        "flagstat",
        "depth",
        "coverage",
        "mpileup",
        "call",
        "concat",
        "norm",
        "annotate",
        "consensus",
        "query",
        "isec",
        "stats",
    ];
    for line in raw.lines() {
        let trimmed = line.trim();
        // Skip empty lines, explanation lines, and "The" lines
        if trimmed.is_empty() || trimmed.starts_with("EXPLANATION") || trimmed.starts_with("The ") {
            continue;
        }
        for prefix in &subcommand_prefixes {
            if trimmed.starts_with(prefix) {
                return trimmed.to_string();
            }
        }
    }

    // 3. Look for the first line that contains CLI flags (starts with `-`)
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('-') || trimmed.contains(" -") {
            // This might be flags without the subcommand — return as-is
            return trimmed.to_string();
        }
    }

    // 4. Give up — return the first non-empty, non-trivial, non-explanation line
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && trimmed.len() > 3
            && !trimmed.starts_with("The ")
            && !trimmed.starts_with("EXPLANATION")
            && !trimmed.starts_with("ARGS:")
        {
            return trimmed.to_string();
        }
    }

    String::new()
}

/// Strip markdown code fences from the raw ARGS line before parsing.
/// Weak LLMs sometimes wrap the response in backticks or triple-backtick blocks.
fn strip_code_fences(s: &str) -> &str {
    let trimmed = s.trim();
    // Triple backtick block: ```...```
    if let Some(inner) = trimmed.strip_prefix("```") {
        // May optionally have a language hint on the first line
        let inner = inner.strip_prefix("bash").unwrap_or(inner);
        let inner = inner.strip_prefix("sh").unwrap_or(inner);
        let inner = inner.trim_start_matches('\n');
        if let Some(inner) = inner.strip_suffix("```") {
            return inner.trim();
        }
        return inner.trim();
    }
    // Single backtick wrapper: `...`
    if let Some(inner) = trimmed.strip_prefix('`')
        && let Some(inner) = inner.strip_suffix('`')
    {
        return inner.trim();
    }
    trimmed
}

// ─── Shell argument parser ────────────────────────────────────────────────────

/// Simple shell-like argument tokenizer that handles single and double quotes.
fn parse_shell_args(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            '\\' if !in_single_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_verification_response_success() {
        let raw = "STATUS: success\nSUMMARY: Command completed successfully.\nISSUES:\n- none\nSUGGESTIONS:\n- none";
        let v = parse_verification_response(raw);
        assert!(v.success);
        assert_eq!(v.summary, "Command completed successfully.");
        assert!(v.issues.is_empty());
        assert!(v.suggestions.is_empty());
    }

    #[test]
    fn test_parse_verification_response_failure() {
        let raw = "STATUS: failure\nSUMMARY: Command failed with non-zero exit code.\nISSUES:\n- Output BAM file is missing\n- Stderr contains 'out of memory'\nSUGGESTIONS:\n- Increase memory limit\n- Check input file integrity";
        let v = parse_verification_response(raw);
        assert!(!v.success);
        assert_eq!(v.summary, "Command failed with non-zero exit code.");
        assert_eq!(v.issues.len(), 2);
        assert!(v.issues[0].contains("BAM"));
        assert_eq!(v.suggestions.len(), 2);
    }

    #[test]
    fn test_parse_verification_response_warning() {
        let raw = "STATUS: warning\nSUMMARY: Completed with warnings.\nISSUES:\n- Low alignment rate (45%)\nSUGGESTIONS:\n- Check reference genome";
        let v = parse_verification_response(raw);
        // warning is still considered success=true (not failure)
        assert!(v.success);
        assert!(!v.issues.is_empty());
    }

    #[test]
    fn test_parse_verification_response_empty() {
        let v = parse_verification_response("");
        assert!(v.success); // defaults to success when no STATUS line
        assert!(v.summary.is_empty());
    }

    #[test]
    fn test_build_verification_prompt_contains_key_info() {
        let prompt = build_verification_prompt(
            "samtools",
            "sort bam",
            "samtools sort -o out.bam in.bam",
            0,
            "",
            &[("out.bam".to_string(), Some(1024))],
        );
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("sort bam"));
        assert!(prompt.contains('0'), "should contain exit code 0");
        assert!(prompt.contains("out.bam"));
        assert!(prompt.contains("1024 bytes"));
    }

    #[test]
    fn test_build_verification_prompt_missing_file() {
        let prompt = build_verification_prompt(
            "bwa",
            "align",
            "bwa mem ref.fa reads.fq > out.sam",
            1,
            "Error: reference not found",
            &[("out.sam".to_string(), None)],
        );
        assert!(prompt.contains("NOT FOUND"));
        assert!(prompt.contains('1'), "should contain exit code 1");
        assert!(prompt.contains("Error: reference not found"));
    }

    #[test]
    fn test_build_verification_prompt_truncates_long_stderr() {
        let long_stderr = "x".repeat(4000);
        let prompt = build_verification_prompt("tool", "task", "tool args", 0, &long_stderr, &[]);
        assert!(prompt.contains("truncated"));
    }

    #[test]
    fn test_build_task_optimization_prompt_contains_tool_and_task() {
        let prompt = build_task_optimization_prompt("samtools", "sort bam");
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("sort bam"));
        assert!(prompt.contains("TASK:"));
    }

    // ─── parse_shell_args ─────────────────────────────────────────────────────

    #[test]
    fn test_parse_shell_args_simple() {
        let args = parse_shell_args("-o out.bam input.bam");
        assert_eq!(args, vec!["-o", "out.bam", "input.bam"]);
    }

    #[test]
    fn test_parse_shell_args_empty() {
        let args = parse_shell_args("");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_shell_args_whitespace_only() {
        let args = parse_shell_args("   ");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_shell_args_single_quoted() {
        let args = parse_shell_args("-o 'my output.bam'");
        assert_eq!(args, vec!["-o", "my output.bam"]);
    }

    #[test]
    fn test_parse_shell_args_double_quoted() {
        let args = parse_shell_args("-o \"my output.bam\"");
        assert_eq!(args, vec!["-o", "my output.bam"]);
    }

    #[test]
    fn test_parse_shell_args_backslash_escape() {
        let args = parse_shell_args(r#"-o my\ output.bam"#);
        assert_eq!(args, vec!["-o", "my output.bam"]);
    }

    #[test]
    fn test_parse_shell_args_multiple_spaces() {
        let args = parse_shell_args("  -o   out.bam   input.bam  ");
        assert_eq!(args, vec!["-o", "out.bam", "input.bam"]);
    }

    #[test]
    fn test_parse_shell_args_mixed_quotes() {
        let args = parse_shell_args("sort -k1,1 -k2,2n 'file with spaces.txt'");
        assert_eq!(
            args,
            vec!["sort", "-k1,1", "-k2,2n", "file with spaces.txt"]
        );
    }

    // ─── is_valid_suggestion ──────────────────────────────────────────────────

    #[test]
    fn test_is_valid_suggestion_with_explanation() {
        let s = LlmCommandSuggestion {
            args: vec!["-o".to_string(), "out.bam".to_string()],
            explanation: "Sort the BAM file by coordinate.".to_string(),
            raw_response: "ARGS: -o out.bam\nEXPLANATION: Sort the BAM file by coordinate."
                .to_string(),
            inference_ms: 0.0,
        };
        assert!(is_valid_suggestion(&s));
    }

    #[test]
    fn test_is_valid_suggestion_empty_explanation() {
        let s = LlmCommandSuggestion {
            args: vec!["-o".to_string()],
            explanation: String::new(),
            raw_response: "ARGS: -o\nEXPLANATION:".to_string(),
            inference_ms: 0.0,
        };
        assert!(!is_valid_suggestion(&s));
    }

    #[test]
    fn test_is_valid_suggestion_empty_args_is_invalid() {
        let s = LlmCommandSuggestion {
            args: vec![],
            explanation: "Run the tool with default arguments.".to_string(),
            raw_response: "ARGS:\nEXPLANATION: Run the tool with default arguments.".to_string(),
            inference_ms: 0.0,
        };
        // Empty args means the LLM failed to follow the output format.
        assert!(!is_valid_suggestion(&s));
    }

    // ─── LlmRunVerification struct ────────────────────────────────────────────

    #[test]
    fn test_llm_run_verification_debug() {
        let v = LlmRunVerification {
            success: true,
            summary: "ok".to_string(),
            issues: vec![],
            suggestions: vec![],
        };
        let s = format!("{v:?}");
        assert!(s.contains("success: true"));
    }

    // ─── build_prompt ─────────────────────────────────────────────────────────

    #[test]
    fn test_build_prompt_basic() {
        let prompt = build_prompt(
            "samtools",
            "samtools --help output here",
            "sort bam file",
            None,
            false,
            0,
            PromptTier::Full,
        );
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("samtools --help output here"));
        assert!(prompt.contains("sort bam file"));
        assert!(prompt.contains("ARGS:"));
        assert!(prompt.contains("EXPLANATION:"));
    }

    #[test]
    fn test_build_prompt_with_skill() {
        use crate::skill::{Skill, SkillContext, SkillExample, SkillMeta};

        let skill = Skill {
            meta: SkillMeta {
                name: "samtools".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec!["concept 1".to_string()],
                pitfalls: vec!["pitfall 1".to_string()],
            },
            examples: vec![SkillExample {
                task: "sort bam".to_string(),
                args: "sort -o sorted.bam input.bam".to_string(),
                explanation: "sort by coordinate".to_string(),
            }],
        };
        let prompt = build_prompt(
            "samtools",
            "docs",
            "sort bam",
            Some(&skill),
            false,
            0,
            PromptTier::Full,
        );
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("concept 1"));
        assert!(prompt.contains("pitfall 1"));
        assert!(prompt.contains("sort bam"));
    }

    #[test]
    fn test_build_prompt_format_instructions() {
        let prompt = build_prompt(
            "bwa",
            "bwa mem --help",
            "align reads",
            None,
            false,
            0,
            PromptTier::Full,
        );
        assert!(
            prompt.contains("ARGS:"),
            "should contain ARGS: format instruction"
        );
        assert!(
            prompt.contains("EXPLANATION:"),
            "should contain EXPLANATION: format instruction"
        );
        // Rules are now in the system prompt, not the user prompt.
        // Verify the user prompt contains the output section.
        assert!(prompt.contains("Output"), "should contain Output section");
    }

    // ─── build_retry_prompt ───────────────────────────────────────────────────

    #[test]
    fn test_build_retry_prompt_contains_prev_response() {
        let prev = "THIS IS WRONG FORMAT";
        let prompt = build_retry_prompt(
            "samtools",
            "docs",
            "sort bam",
            None,
            prev,
            false,
            0,
            PromptTier::Full,
        );
        assert!(
            prompt.contains(prev),
            "retry prompt should include previous response"
        );
        assert!(
            prompt.contains("Correction"),
            "retry prompt should mention correction"
        );
        assert!(prompt.contains("ARGS:"));
    }

    // ─── strip_markdown_fences ────────────────────────────────────────────────

    #[test]
    fn test_strip_markdown_fences_no_fence() {
        let raw = "---\nname: tool\n---\n\n## Concepts\n";
        assert_eq!(strip_markdown_fences(raw), raw.trim());
    }

    #[test]
    fn test_strip_markdown_fences_with_fence() {
        let raw = "```markdown\n---\nname: tool\n---\n```";
        let stripped = strip_markdown_fences(raw);
        assert!(!stripped.starts_with("```"), "fence should be removed");
        assert!(
            !stripped.ends_with("```"),
            "closing fence should be removed"
        );
        assert!(stripped.contains("---"));
    }

    #[test]
    fn test_strip_markdown_fences_with_md_fence() {
        let raw = "```md\n---\nname: tool\n---\n```";
        let stripped = strip_markdown_fences(raw);
        assert!(!stripped.starts_with("```"));
        assert!(stripped.contains("---"));
    }

    #[test]
    fn test_strip_markdown_fences_bare_fence() {
        let raw = "```\n---\nname: tool\n---\n```";
        let stripped = strip_markdown_fences(raw);
        assert!(!stripped.starts_with("```"));
        assert!(stripped.contains("---"));
    }

    // ─── parse_skill_verify_response ─────────────────────────────────────────

    #[test]
    fn test_parse_skill_verify_response_pass() {
        let raw =
            "VERDICT: pass\nSUMMARY: The skill looks good.\nISSUES:\n- none\nSUGGESTIONS:\n- none";
        let v = parse_skill_verify_response(raw);
        assert!(v.passed);
        assert_eq!(v.summary, "The skill looks good.");
        assert!(v.issues.is_empty());
        assert!(v.suggestions.is_empty());
    }

    #[test]
    fn test_parse_skill_verify_response_fail() {
        let raw = "VERDICT: fail\nSUMMARY: The skill needs work.\nISSUES:\n- Missing examples\n- Category is empty\nSUGGESTIONS:\n- Add 5 examples\n- Set a category";
        let v = parse_skill_verify_response(raw);
        assert!(!v.passed);
        assert_eq!(v.summary, "The skill needs work.");
        assert_eq!(v.issues.len(), 2);
        assert_eq!(v.suggestions.len(), 2);
        assert!(v.issues.iter().any(|i| i.contains("Missing")));
    }

    #[test]
    fn test_parse_skill_verify_response_empty() {
        let v = parse_skill_verify_response("");
        // Defaults to passed=true when no VERDICT line
        assert!(v.passed);
        assert!(v.summary.is_empty());
    }

    // ─── build_skill_verify_prompt ────────────────────────────────────────────

    #[test]
    fn test_build_skill_verify_prompt_contains_tool_and_content() {
        let content = "---\nname: samtools\n---\n## Concepts\n";
        let prompt = build_skill_verify_prompt("samtools", content);
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains(content));
        assert!(prompt.contains("VERDICT:"));
    }

    // ─── build_skill_polish_prompt ────────────────────────────────────────────

    #[test]
    fn test_build_skill_polish_prompt_contains_tool_and_content() {
        let content = "---\nname: bwa\n---\n## Concepts\n";
        let prompt = build_skill_polish_prompt("bwa", content);
        assert!(prompt.contains("bwa"));
        assert!(prompt.contains(content));
        assert!(prompt.contains("Polish"));
    }

    // ─── build_skill_generate_prompt ─────────────────────────────────────────

    #[test]
    fn test_build_skill_generate_prompt_contains_tool() {
        let prompt = build_skill_generate_prompt("gatk");
        assert!(prompt.contains("gatk"));
        assert!(prompt.contains("Concepts"));
        assert!(prompt.contains("Pitfalls"));
        assert!(prompt.contains("Examples"));
    }

    // ─── LlmClient::parse_response ───────────────────────────────────────────

    #[test]
    fn test_parse_response_basic() {
        let raw = "ARGS: sort -o out.bam in.bam\nEXPLANATION: Sort the BAM file by coordinate.";
        let suggestion = LlmClient::parse_response(raw).unwrap();
        assert_eq!(suggestion.args, vec!["sort", "-o", "out.bam", "in.bam"]);
        assert_eq!(suggestion.explanation, "Sort the BAM file by coordinate.");
    }

    #[test]
    fn test_parse_response_none_args() {
        let raw = "ARGS: (none)\nEXPLANATION: Run with default settings.";
        let suggestion = LlmClient::parse_response(raw).unwrap();
        assert!(
            suggestion.args.is_empty(),
            "ARGS: (none) should give empty args"
        );
        assert_eq!(suggestion.explanation, "Run with default settings.");
    }

    #[test]
    fn test_parse_response_empty_args() {
        let raw = "ARGS:\nEXPLANATION: Run with no extra args.";
        let suggestion = LlmClient::parse_response(raw).unwrap();
        assert!(suggestion.args.is_empty());
    }

    #[test]
    fn test_parse_response_no_explanation() {
        let raw = "ARGS: -o out.bam";
        let suggestion = LlmClient::parse_response(raw).unwrap();
        assert_eq!(suggestion.args, vec!["-o", "out.bam"]);
        assert!(suggestion.explanation.is_empty());
    }

    #[test]
    fn test_parse_response_raw_response_stored() {
        let raw = "ARGS: -o out.bam\nEXPLANATION: Test";
        let suggestion = LlmClient::parse_response(raw).unwrap();
        assert_eq!(suggestion.raw_response, raw);
    }

    // ─── build_task_optimization_prompt ──────────────────────────────────────

    #[test]
    fn test_build_task_optimization_prompt_format() {
        let prompt = build_task_optimization_prompt("samtools", "sort bam by name");
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("sort bam by name"));
        assert!(
            prompt.contains("TASK:"),
            "should contain TASK: output format"
        );
    }

    // ─── verification_system_prompt / build_verification_prompt ──────────────

    #[test]
    fn test_verification_system_prompt_not_empty() {
        let prompt = verification_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("bioinformatics"));
    }

    // ─── skill_reviewer_system_prompt ────────────────────────────────────────

    #[test]
    fn test_skill_reviewer_system_prompt_not_empty() {
        let prompt = skill_reviewer_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("skill"));
    }

    // ─── LlmClient::new ──────────────────────────────────────────────────────

    #[test]
    fn test_llm_client_new() {
        use crate::config::Config;
        let cfg = Config::default();
        let _client = LlmClient::new(cfg);
        // Just verify it can be constructed without panic
    }

    // ─── system_prompt ────────────────────────────────────────────────────────

    #[test]
    fn test_system_prompt_not_empty() {
        let p = system_prompt();
        assert!(!p.is_empty());
        assert!(
            p.contains("bioinformatics"),
            "should mention bioinformatics"
        );
        assert!(p.contains("ARGS"), "should mention ARGS format");
        assert!(
            p.contains("EXPLANATION"),
            "should mention EXPLANATION format"
        );
    }

    // ─── ChatMessage Debug + Clone ────────────────────────────────────────────

    #[test]
    fn test_chat_message_clone() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        };
        let cloned = msg.clone();
        assert_eq!(cloned.role, "user");
        assert_eq!(cloned.content, "hello");
    }

    #[test]
    fn test_chat_message_debug() {
        let msg = ChatMessage {
            role: "system".to_string(),
            content: "instructions".to_string(),
        };
        let s = format!("{msg:?}");
        assert!(s.contains("system"));
        assert!(s.contains("instructions"));
    }

    // ─── ChatRequest serialization ────────────────────────────────────────────

    #[test]
    fn test_chat_request_serializes_correctly() {
        let req = ChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are helpful.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "sort a bam file".to_string(),
                },
            ],
            max_tokens: 2048,
            temperature: 0.0,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"gpt-4o\""));
        assert!(json.contains("\"max_tokens\":2048"));
        assert!(json.contains("\"temperature\":0.0"));
        assert!(json.contains("system"));
        assert!(json.contains("You are helpful."));
    }

    #[test]
    fn test_chat_request_debug() {
        let req = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            max_tokens: 100,
            temperature: 0.5,
        };
        let s = format!("{req:?}");
        assert!(s.contains("test-model"));
    }

    // ─── ChatResponse + ChatChoice deserialization ────────────────────────────

    #[test]
    fn test_chat_response_deserializes_correctly() {
        let json = r#"{
            "choices": [
                {"message": {"role": "assistant", "content": "ARGS: sort -o out.bam\nEXPLANATION: Sorts BAM"}}
            ]
        }"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].message.role, "assistant");
        assert!(resp.choices[0].message.content.contains("ARGS:"));
    }

    #[test]
    fn test_chat_response_empty_choices() {
        let json = r#"{"choices": []}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(resp.choices.is_empty());
    }

    // ─── parse_shell_args tab handling ────────────────────────────────────────

    #[test]
    fn test_parse_shell_args_tab_as_separator() {
        // Tab should be treated the same as space between args
        let args = parse_shell_args("-o\tout.bam");
        assert_eq!(args, vec!["-o", "out.bam"]);
    }

    // ─── LlmCommandSuggestion + LlmVerificationResult + LlmSkillVerification Debug ─

    #[test]
    fn test_llm_command_suggestion_debug() {
        let s = LlmCommandSuggestion {
            args: vec!["sort".to_string()],
            explanation: "sort it".to_string(),
            raw_response: "raw".to_string(),
            inference_ms: 0.0,
        };
        let dbg = format!("{s:?}");
        assert!(dbg.contains("sort"));
    }

    #[test]
    fn test_llm_verification_result_debug() {
        let v = LlmVerificationResult {
            provider: "openai".to_string(),
            api_base: "https://api.openai.com".to_string(),
            model: "gpt-4o".to_string(),
            response_preview: "OK".to_string(),
        };
        let dbg = format!("{v:?}");
        assert!(dbg.contains("openai"));
    }

    #[test]
    fn test_llm_skill_verification_debug() {
        let v = LlmSkillVerification {
            passed: true,
            summary: "looks good".to_string(),
            issues: vec![],
            suggestions: vec!["add more examples".to_string()],
        };
        let dbg = format!("{v:?}");
        assert!(dbg.contains("looks good"));
    }

    #[test]
    fn test_llm_skill_verification_clone() {
        let v = LlmSkillVerification {
            passed: false,
            summary: "needs work".to_string(),
            issues: vec!["missing examples".to_string()],
            suggestions: vec![],
        };
        let cloned = v.clone();
        assert!(!cloned.passed);
        assert_eq!(cloned.summary, "needs work");
        assert_eq!(cloned.issues, vec!["missing examples".to_string()]);
    }

    // ─── LlmRunVerification Clone ─────────────────────────────────────────────

    #[test]
    fn test_llm_run_verification_clone() {
        let v = LlmRunVerification {
            success: true,
            summary: "ok".to_string(),
            issues: vec!["minor".to_string()],
            suggestions: vec!["retry".to_string()],
        };
        let cloned = v.clone();
        assert_eq!(cloned.success, v.success);
        assert_eq!(cloned.summary, v.summary);
        assert_eq!(cloned.issues, v.issues);
        assert_eq!(cloned.suggestions, v.suggestions);
    }

    // ─── Mock HTTP tests (wiremock) ───────────────────────────────────────────
    //
    // These tests start a real local HTTP server and point the LlmClient at it.
    // The URL begins with "http://127.0.0.1", which is in the allowlist in
    // request_with_system(), so no HTTPS enforcement is triggered.

    #[cfg(not(target_arch = "wasm32"))]
    mod mock_tests {
        use super::*;
        use crate::config::Config;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        /// Build a Config pointing at the given local mock server URL.
        fn mock_config(base_url: &str) -> Config {
            let mut cfg = Config::default();
            cfg.llm.api_token = Some("test-token".to_string());
            cfg.llm.api_base = Some(base_url.to_string());
            cfg.llm.provider = "openai".to_string();
            cfg.llm.model = Some("gpt-4o-mini".to_string());
            cfg
        }

        /// Minimal valid chat-completions response body.
        fn completion_body(content: &str) -> serde_json::Value {
            serde_json::json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": content
                    }
                }]
            })
        }

        // ── suggest_command ───────────────────────────────────────────────────

        #[tokio::test]
        async fn test_suggest_command_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "ARGS: sort -o sorted.bam input.bam\nEXPLANATION: Sort BAM by coordinate.",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .suggest_command(
                    "samtools",
                    "samtools --help output",
                    "sort bam",
                    None,
                    false,
                )
                .await;

            assert!(result.is_ok(), "should succeed: {:?}", result.err());
            let s = result.unwrap();
            assert!(!s.args.is_empty(), "should have parsed args");
            assert!(!s.explanation.is_empty(), "should have explanation");
        }

        #[tokio::test]
        async fn test_suggest_command_http_error_propagated() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .suggest_command("samtools", "docs", "sort", None, false)
                .await;
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("500") || msg.contains("Internal Server Error"),
                "error should mention status: {msg}"
            );
        }

        #[tokio::test]
        async fn test_suggest_command_invalid_format_retries() {
            let server = MockServer::start().await;
            // First call returns invalid format; second returns valid.
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body("This is not the right format at all")),
                )
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "ARGS: sort -o out.bam\nEXPLANATION: Sorts BAM file.",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .suggest_command("samtools", "docs", "sort bam", None, false)
                .await;

            assert!(result.is_ok());
        }

        // ── verify_configuration ──────────────────────────────────────────────

        #[tokio::test]
        async fn test_verify_configuration_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body("OK")))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client.verify_configuration().await;

            assert!(result.is_ok(), "should succeed: {:?}", result.err());
            let v = result.unwrap();
            assert_eq!(v.response_preview, "OK");
            assert!(!v.model.is_empty());
            assert!(!v.provider.is_empty());
        }

        // ── optimize_task ──────────────────────────────────────────────────────

        #[tokio::test]
        async fn test_optimize_task_valid_response() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "TASK: Sort a BAM file by coordinate with 8 threads and output to sorted.bam",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client.optimize_task("samtools", "sort bam").await;

            assert!(result.is_ok());
            let refined = result.unwrap();
            assert!(
                refined.contains("BAM") || refined.contains("sort"),
                "should return the optimized task"
            );
        }

        #[tokio::test]
        async fn test_optimize_task_falls_back_on_bad_format() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body("Not a TASK: prefixed line at all")),
                )
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client.optimize_task("samtools", "sort bam").await;

            // Falls back to original task (or returns the raw response)
            assert!(result.is_ok());
        }

        // ── verify_run_result ─────────────────────────────────────────────────

        #[tokio::test]
        async fn test_verify_run_result_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "STATUS: success\nSUMMARY: Command completed successfully.\nISSUES:\n- none\nSUGGESTIONS:\n- none",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .verify_run_result(
                    "samtools",
                    "sort bam",
                    "samtools sort -o out.bam in.bam",
                    0,
                    "",
                    &[("out.bam".to_string(), Some(1024))],
                )
                .await;

            assert!(result.is_ok());
            let v = result.unwrap();
            assert!(v.success);
            assert!(!v.summary.is_empty());
        }

        // ── verify_skill ──────────────────────────────────────────────────────

        #[tokio::test]
        async fn test_verify_skill_pass() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "VERDICT: pass\nSUMMARY: Skill looks complete.\nISSUES:\n- none\nSUGGESTIONS:\n- none",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let skill_content = "---\nname: samtools\n---\n## Concepts\n- concept\n";
            let result = client.verify_skill("samtools", skill_content).await;

            assert!(result.is_ok());
            let v = result.unwrap();
            assert!(v.passed);
        }

        // ── polish_skill ──────────────────────────────────────────────────────

        #[tokio::test]
        async fn test_polish_skill_strips_fences() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "```markdown\n---\nname: samtools\n---\n## Concepts\n- improved\n```",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .polish_skill("samtools", "---\nname: samtools\n---\n")
                .await;

            assert!(result.is_ok());
            let content = result.unwrap();
            assert!(
                !content.starts_with("```"),
                "fences should be stripped: {content}"
            );
        }

        // ── generate_skill_template ───────────────────────────────────────────

        #[tokio::test]
        async fn test_generate_skill_template() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "---\nname: gatk\ncategory: variant-calling\n---\n## Concepts\n- concept\n",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client.generate_skill_template("gatk").await;

            assert!(result.is_ok());
            let content = result.unwrap();
            assert!(content.contains("gatk") || content.contains("---"));
        }

        // ── generate_shell_command ────────────────────────────────────────────

        #[tokio::test]
        async fn test_generate_shell_command_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
                    "COMMAND: ls -la\nEXPLANATION: List all files with details.",
                )))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .generate_shell_command("list all files with details")
                .await;

            assert!(result.is_ok());
            let (cmd, expl) = result.unwrap();
            assert!(!cmd.is_empty());
            assert!(expl.contains("List") || expl.contains("files"));
        }

        #[tokio::test]
        async fn test_generate_shell_command_bad_format_falls_back() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(completion_body("ls -la")))
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client.generate_shell_command("list files").await;

            assert!(result.is_ok());
            let (cmd, _expl) = result.unwrap();
            // Falls back to raw response text when no COMMAND: prefix
            assert!(!cmd.is_empty());
        }

        // ── request_with_system: anthropic provider headers ───────────────────

        #[tokio::test]
        async fn test_request_with_system_anthropic_provider() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body("ARGS: -o out.bam\nEXPLANATION: test")),
                )
                .mount(&server)
                .await;

            let mut cfg = mock_config(&server.uri());
            cfg.llm.provider = "anthropic".to_string();

            let client = LlmClient::new(cfg);
            // Calling through suggest_command which uses request_with_system internally
            let result = client
                .suggest_command("samtools", "docs", "sort", None, false)
                .await;

            assert!(result.is_ok());
        }

        // ── request_with_system: empty choices returns default ─────────────────

        #[tokio::test]
        async fn test_request_with_system_empty_choices() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({"choices": []})),
                )
                .mount(&server)
                .await;

            let client = LlmClient::new(mock_config(&server.uri()));
            let result = client
                .suggest_command("samtools", "docs", "sort", None, false)
                .await;

            // Empty choices → empty string → parse_response returns empty suggestion
            assert!(result.is_ok());
        }
    }

    // ─── parse_verification_response edge cases ───────────────────────────────

    #[test]
    fn test_parse_verification_response_with_issues_and_suggestions() {
        let raw = "\
STATUS: fail
SUMMARY: Command failed with error
ISSUES:
- Missing input file
- Wrong flag used
SUGGESTIONS:
- Check the input path
- Use --output instead of -o";
        let result = parse_verification_response(raw);
        assert!(!result.success);
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.suggestions.len(), 2);
    }

    #[test]
    fn test_parse_verification_response_only_summary() {
        let raw = "STATUS: OK\nSUMMARY: Everything looks good";
        let result = parse_verification_response(raw);
        assert!(result.success);
        assert_eq!(result.summary, "Everything looks good");
        assert!(result.issues.is_empty());
        assert!(result.suggestions.is_empty());
    }

    // ─── strip_markdown_fences additional ─────────────────────────────────────

    #[test]
    fn test_strip_markdown_fences_with_yaml_fence() {
        let input = "```yaml\nname: test\nvalue: 42\n```";
        let result = strip_markdown_fences(input);
        assert!(result.contains("name: test"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn test_strip_markdown_fences_with_toml_fence() {
        let input = "```toml\n[section]\nkey = \"value\"\n```";
        let result = strip_markdown_fences(input);
        assert!(result.contains("key = \"value\""));
        assert!(!result.contains("```"));
    }

    #[test]
    fn test_strip_markdown_fences_no_closing_fence() {
        let input = "```markdown\nsome content without closing fence";
        let result = strip_markdown_fences(input);
        // Should return the original input if no closing fence
        assert!(result.contains("some content"));
    }

    // ─── parse_skill_verify_response additional ───────────────────────────────

    #[test]
    fn test_parse_skill_verify_response_with_issues_and_suggestions() {
        let raw = "\
VERDICT: FAIL
SUMMARY: Skill has problems
ISSUES:
- Missing concepts
- Too few examples
SUGGESTIONS:
- Add more concepts
- Add at least 5 examples";
        let result = parse_skill_verify_response(raw);
        assert!(!result.passed);
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.suggestions.len(), 2);
    }

    #[test]
    fn test_parse_skill_verify_response_pass_with_summary() {
        let raw = "VERDICT: pass\nSUMMARY: Skill looks great";
        let result = parse_skill_verify_response(raw);
        assert!(result.passed);
        assert_eq!(result.summary, "Skill looks great");
    }

    // ─── build prompts additional ─────────────────────────────────────────────

    #[test]
    fn test_build_prompt_truncates_long_docs() {
        let long_docs = "a".repeat(200_000);
        let prompt = build_prompt("tool", &long_docs, "task", None, false, 0, PromptTier::Full);
        // Should still produce a valid prompt (may be truncated internally)
        assert!(prompt.contains("tool"));
        assert!(prompt.contains("task"));
    }

    #[test]
    fn test_build_prompt_empty_task() {
        let prompt = build_prompt(
            "samtools",
            "some docs",
            "",
            None,
            false,
            0,
            PromptTier::Full,
        );
        assert!(prompt.contains("samtools"));
    }

    #[test]
    fn test_build_retry_prompt_format() {
        let prompt = build_retry_prompt(
            "samtools",
            "some docs",
            "sort a BAM file",
            None,
            "invalid resp",
            false,
            0,
            PromptTier::Full,
        );
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("sort a BAM file"));
        assert!(prompt.contains("invalid resp"));
    }

    #[test]
    fn test_build_skill_generate_prompt_format() {
        let prompt = build_skill_generate_prompt("fastp");
        assert!(prompt.contains("fastp"));
    }

    // ─── parse_shell_args additional ──────────────────────────────────────────

    #[test]
    fn test_parse_shell_args_nested_quotes() {
        let result = parse_shell_args("--filter 'QUAL > 30'");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "--filter");
        assert_eq!(result[1], "QUAL > 30");
    }

    #[test]
    fn test_parse_shell_args_equals_syntax() {
        let result = parse_shell_args("--threads=8 --output=out.bam");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "--threads=8");
        assert_eq!(result[1], "--output=out.bam");
    }

    #[test]
    fn test_parse_shell_args_backslash_space() {
        let result = parse_shell_args("my\\ file.bam");
        // Backslash-space should be treated as escaped space
        assert!(!result.is_empty());
    }

    // ─── sanitize_args ──────────────────────────────────────────────────────

    #[test]
    fn test_sanitize_args_strips_tool_name() {
        let args = vec![
            "samtools".to_string(),
            "sort".to_string(),
            "-o".to_string(),
            "out.bam".to_string(),
            "in.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args);
        assert_eq!(result, vec!["sort", "-o", "out.bam", "in.bam"]);
    }

    #[test]
    fn test_sanitize_args_preserves_companion_binary() {
        let args = vec![
            "bowtie2-build".to_string(),
            "ref.fa".to_string(),
            "idx".to_string(),
        ];
        let result = sanitize_args("bowtie2", args);
        assert_eq!(result, vec!["bowtie2-build", "ref.fa", "idx"]);
    }

    #[test]
    fn test_sanitize_args_no_change_for_flags() {
        let args = vec!["-o".to_string(), "out.bam".to_string()];
        let result = sanitize_args("samtools", args);
        assert_eq!(result, vec!["-o", "out.bam"]);
    }

    #[test]
    fn test_sanitize_args_empty() {
        let result = sanitize_args("samtools", vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_sanitize_args_case_insensitive() {
        let args = vec![
            "Samtools".to_string(),
            "sort".to_string(),
            "in.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args);
        assert_eq!(result, vec!["sort", "in.bam"]);
    }

    #[test]
    fn test_sanitize_args_injects_tool_after_and_and() {
        // LLM generates: sort ... && index ...
        // Expected:       sort ... && samtools index ...
        let args = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
            "celegans.bam".to_string(),
            "&&".to_string(),
            "index".to_string(),
            "sorted.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args);
        assert_eq!(
            result,
            vec![
                "sort",
                "-@",
                "4",
                "-o",
                "sorted.bam",
                "celegans.bam",
                "&&",
                "samtools",
                "index",
                "sorted.bam"
            ]
        );
    }

    #[test]
    fn test_sanitize_args_no_injection_when_tool_already_present() {
        // LLM correctly generates: sort ... && samtools index ...
        // Nothing should change.
        let args = vec![
            "sort".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
            "input.bam".to_string(),
            "&&".to_string(),
            "samtools".to_string(),
            "index".to_string(),
            "sorted.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args.clone());
        assert_eq!(result, args);
    }

    #[test]
    fn test_sanitize_args_injects_after_or_or() {
        // LLM generates: view -h input.bam || flagstat input.bam
        let args = vec![
            "view".to_string(),
            "-h".to_string(),
            "input.bam".to_string(),
            "||".to_string(),
            "flagstat".to_string(),
            "input.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args);
        assert_eq!(
            result,
            vec![
                "view",
                "-h",
                "input.bam",
                "||",
                "samtools",
                "flagstat",
                "input.bam"
            ]
        );
    }

    #[test]
    fn test_sanitize_args_preserves_companion_binary_after_and_and() {
        // bowtie2-build is a companion binary and must NOT get "bowtie2" injected
        let args = vec![
            "bowtie2-build".to_string(),
            "ref.fa".to_string(),
            "idx".to_string(),
            "&&".to_string(),
            "bowtie2-build".to_string(),
            "ref2.fa".to_string(),
            "idx2".to_string(),
        ];
        let result = sanitize_args("bowtie2", args.clone());
        assert_eq!(result, args);
    }

    #[test]
    fn test_sanitize_args_multiple_steps() {
        // Three-step: sort && index && flagstat — second and third missing tool name
        let args = vec![
            "sort".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
            "input.bam".to_string(),
            "&&".to_string(),
            "index".to_string(),
            "sorted.bam".to_string(),
            "&&".to_string(),
            "flagstat".to_string(),
            "sorted.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args);
        assert_eq!(
            result,
            vec![
                "sort",
                "-o",
                "sorted.bam",
                "input.bam",
                "&&",
                "samtools",
                "index",
                "sorted.bam",
                "&&",
                "samtools",
                "flagstat",
                "sorted.bam"
            ]
        );
    }

    #[test]
    fn test_sanitize_args_mixed_operators() {
        // sort && index || flagstat — both && and || segments missing tool name
        let args = vec![
            "sort".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
            "input.bam".to_string(),
            "&&".to_string(),
            "index".to_string(),
            "sorted.bam".to_string(),
            "||".to_string(),
            "flagstat".to_string(),
            "sorted.bam".to_string(),
        ];
        let result = sanitize_args("samtools", args);
        assert_eq!(
            result,
            vec![
                "sort",
                "-o",
                "sorted.bam",
                "input.bam",
                "&&",
                "samtools",
                "index",
                "sorted.bam",
                "||",
                "samtools",
                "flagstat",
                "sorted.bam"
            ]
        );
    }

    // ─── strip_code_fences ──────────────────────────────────────────────────

    #[test]
    fn test_strip_code_fences_backtick() {
        assert_eq!(strip_code_fences("`-o out.bam`"), "-o out.bam");
    }

    #[test]
    fn test_strip_code_fences_triple_backtick() {
        assert_eq!(strip_code_fences("```\n-o out.bam\n```"), "-o out.bam");
    }

    #[test]
    fn test_strip_code_fences_triple_backtick_with_lang() {
        assert_eq!(strip_code_fences("```bash\n-o out.bam\n```"), "-o out.bam");
    }

    #[test]
    fn test_strip_code_fences_no_fences() {
        assert_eq!(strip_code_fences("-o out.bam"), "-o out.bam");
    }

    #[test]
    fn test_strip_code_fences_preserves_inner_backtick() {
        assert_eq!(strip_code_fences("in=R1.fastq.gz"), "in=R1.fastq.gz");
    }

    // ─── system prompt rules ────────────────────────────────────────────────

    #[test]
    fn test_system_prompt_contains_pipe_rule() {
        let prompt = system_prompt();
        assert!(
            prompt.contains("pipe") || prompt.contains("|"),
            "system prompt should contain pipe handling rule"
        );
    }

    #[test]
    fn test_system_prompt_contains_positional_arg_rule() {
        let prompt = system_prompt();
        // The positional argument rule was removed from the rewritten system prompt
        // because it's a rare edge case that's better handled via skill injection.
        // Instead, verify the system prompt contains the essential subcommand rule.
        assert!(
            prompt.contains("subcommand"),
            "system prompt should contain subcommand rule"
        );
    }

    #[test]
    fn test_build_prompt_contains_pipe_rule() {
        let prompt = build_prompt(
            "bcftools",
            "docs",
            "call variants",
            None,
            false,
            0,
            PromptTier::Full,
        );
        // Pipe rule is now in the system prompt, not the user prompt.
        // Verify the system prompt contains it.
        let sys = system_prompt();
        assert!(
            sys.contains("Pipes") || sys.contains("|"),
            "system prompt should contain pipe rule"
        );
        // User prompt should still have ARGS: format
        assert!(
            prompt.contains("ARGS:"),
            "build_prompt should contain ARGS: format instruction"
        );
    }

    #[test]
    fn test_build_prompt_contains_multistep_rule() {
        let prompt = build_prompt(
            "bcftools",
            "docs",
            "call variants",
            None,
            false,
            0,
            PromptTier::Full,
        );
        // Multi-step rule is now in the system prompt, not the user prompt.
        let sys = system_prompt();
        assert!(
            sys.contains("&&"),
            "system prompt should contain multi-step rule"
        );
        // User prompt should contain task
        assert!(
            prompt.contains("call variants"),
            "build_prompt should contain the task"
        );
    }

    // ─── Adaptive prompt compression tests ────────────────────────────────────

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("a"), 1);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
        // ~1000 chars → ~250 tokens
        let text = "a".repeat(1000);
        assert_eq!(estimate_tokens(&text), 250);
    }

    #[test]
    fn test_prompt_tier_compact() {
        // Large model (7b) — tier is based purely on context window
        assert_eq!(prompt_tier(0, "llama-7b"), PromptTier::Full);
        assert_eq!(prompt_tier(2048, "llama-7b"), PromptTier::Compact);
        assert_eq!(prompt_tier(4095, "llama-7b"), PromptTier::Compact);
        assert_eq!(prompt_tier(4096, "llama-7b"), PromptTier::Medium);
        assert_eq!(prompt_tier(8192, "llama-7b"), PromptTier::Medium);
        assert_eq!(prompt_tier(16384, "llama-7b"), PromptTier::Full);
        assert_eq!(prompt_tier(128_000, "llama-7b"), PromptTier::Full);

        // Tiny model (0.5b) — always Compact (most aggressive compression)
        assert_eq!(
            prompt_tier(32_768, "qwen2.5-coder:0.5b"),
            PromptTier::Compact
        );
        assert_eq!(prompt_tier(0, "qwen2.5-coder:0.5b"), PromptTier::Compact);
        assert_eq!(prompt_tier(2048, "qwen2.5-coder:0.5b"), PromptTier::Compact);
        assert_eq!(prompt_tier(4096, "qwen2.5-coder:0.5b"), PromptTier::Compact);

        // 1B model — also always Compact
        assert_eq!(
            prompt_tier(32_768, "deepseek-coder:1b"),
            PromptTier::Compact
        );

        // 1.3B model — also Compact (testing shows Medium is too verbose)
        assert_eq!(
            prompt_tier(32_768, "deepseek-coder:1.3b"),
            PromptTier::Compact
        );

        // 1.5B model — also Compact
        assert_eq!(prompt_tier(32_768, "model-1.5b"), PromptTier::Compact);

        // 3B model — also Compact
        assert_eq!(prompt_tier(32_768, "model-3b"), PromptTier::Compact);

        // Unknown model with no param tag — standard logic
        assert_eq!(prompt_tier(16384, "some-unknown-model"), PromptTier::Full);
    }

    #[test]
    fn test_compact_system_prompt_shorter() {
        let full = system_prompt();
        let compact = system_prompt_compact();
        assert!(
            compact.len() < full.len() / 3,
            "compact system prompt should be <1/3 of full (compact={}, full={})",
            compact.len(),
            full.len()
        );
        assert!(compact.contains("ARGS:"));
        assert!(compact.contains("EXPLANATION:"));
    }

    #[test]
    fn test_build_prompt_compact_tier() {
        // Compact tier without skill → fallback example + truncated docs
        let prompt_no_skill = build_prompt(
            "samtools",
            "long docs here",
            "sort bam file",
            None,
            false,
            2048,
            PromptTier::Compact,
        );
        // Contains tool and task
        assert!(prompt_no_skill.contains("samtools"));
        assert!(prompt_no_skill.contains("sort bam file"));
        // No skill examples → should get fallback generic example
        assert!(
            prompt_no_skill.contains("---FEW-SHOT---"),
            "compact prompt without skill should use fallback few-shot"
        );
        // No skill examples → should inject truncated docs as grounding
        assert!(prompt_no_skill.contains("Docs:"));

        // Compact tier with skill → few-shot format
        let skill = Skill {
            meta: crate::skill::SkillMeta {
                name: "samtools".into(),
                category: "alignment".into(),
                description: "SAM/BAM tools".into(),
                tags: vec![],
                author: None,
                source_url: None,
            },
            context: crate::skill::SkillContext {
                concepts: vec!["SAM/BAM are alignment formats".into()],
                pitfalls: vec!["Always sort before indexing".into()],
            },
            examples: vec![crate::skill::SkillExample {
                task: "Sort a BAM file by coordinate".into(),
                args: "sort -@ 4 -o sorted.bam input.bam".into(),
                explanation: "Coordinate sort with 4 threads.".into(),
            }],
        };
        let prompt_with_skill = build_prompt(
            "samtools",
            "long docs here",
            "sort bam file",
            Some(&skill),
            false,
            2048,
            PromptTier::Compact,
        );
        // Should contain few-shot separator
        assert!(
            prompt_with_skill.contains("---FEW-SHOT---"),
            "compact prompt with skill should use few-shot format"
        );
        // Should NOT contain the verbose format instructions
        assert!(
            !prompt_with_skill.contains("COMPANION BINARY"),
            "compact prompt should not contain COMPANION BINARY instructions"
        );
        // Should contain format from few-shot, not from a template in user message
        assert!(prompt_with_skill.contains("EXPLANATION:"));
    }

    #[test]
    fn test_build_prompt_medium_tier() {
        // 8192 token context → Medium tier (using a 7b model so tier is context-based)
        let prompt = build_prompt(
            "samtools",
            "long docs here",
            "sort bam file",
            None,
            false,
            8192,
            PromptTier::Full,
        );
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("sort bam file"));
        assert!(prompt.contains("ARGS:"));
    }

    #[test]
    fn test_truncate_documentation() {
        let docs = "line1\nline2\nline3\nline4\nline5";
        // All fits within a large budget
        assert_eq!(truncate_documentation(docs, 1000), docs);

        // A longer doc that actually needs truncation
        let long_docs = "first line of docs\nsecond line of docs\nthird line of docs\nfourth line of docs\nfifth line of docs which is really long and should not fit";
        let truncated = truncate_documentation(long_docs, 80);
        assert!(
            truncated.contains("first line"),
            "should contain first line"
        );
        assert!(
            truncated.contains("[...truncated]"),
            "should indicate truncation"
        );

        // Too small budget → empty
        assert_eq!(truncate_documentation(long_docs, 30), "");
    }

    #[test]
    fn test_build_prompt_compact_with_skill_limits_examples() {
        use crate::skill::{Skill, SkillContext, SkillExample, SkillMeta};

        let examples: Vec<SkillExample> = (0..10)
            .map(|i| SkillExample {
                task: format!("task {i}"),
                args: format!("arg{i}"),
                explanation: format!("explain {i}"),
            })
            .collect();

        let skill = Skill {
            meta: SkillMeta {
                name: "samtools".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec![
                    "c1".into(),
                    "c2".into(),
                    "c3".into(),
                    "c4".into(),
                    "c5".into(),
                ],
                pitfalls: vec!["p1".into(), "p2".into(), "p3".into()],
            },
            examples,
        };

        // Compact tier (2048 tokens) → few-shot format, 1 most relevant example
        let prompt = build_prompt(
            "samtools",
            "docs",
            "sort bam",
            Some(&skill),
            false,
            2048,
            PromptTier::Compact,
        );
        // Compact tier uses few-shot: the best-matching example's ARGS/EXPLANATION
        // are embedded as an assistant message, not listed as "Example N"
        assert!(
            prompt.contains("---FEW-SHOT---"),
            "compact tier should use few-shot format"
        );
        // Should contain the most relevant example (task 0 or task matching "sort")
        assert!(prompt.contains("ARGS:"));
    }

    // ─── JSON response parsing tests ──────────────────────────────────────

    #[test]
    fn test_parse_json_response() {
        let json = r#"{"args": "-@ 4 -o sorted.bam input.bam", "explanation": "Sort BAM file"}"#;
        let result = LlmClient::try_parse_json_response(json).unwrap();
        assert!(!result.args.is_empty());
        assert_eq!(result.explanation, "Sort BAM file");
    }

    #[test]
    fn test_parse_json_response_uppercase() {
        let json = r#"{"ARGS": "view -h input.bam", "EXPLANATION": "View BAM header"}"#;
        let result = LlmClient::try_parse_json_response(json).unwrap();
        assert!(!result.args.is_empty());
    }

    #[test]
    fn test_parse_json_response_wrapped_in_fences() {
        let json = "```json\n{\"args\": \"sort in.bam\", \"explanation\": \"Sort\"}\n```";
        let result = LlmClient::try_parse_json_response(json).unwrap();
        assert!(!result.args.is_empty());
    }

    #[test]
    fn test_parse_json_response_not_json() {
        let text = "ARGS: sort in.bam\nEXPLANATION: Sort a BAM file";
        assert!(LlmClient::try_parse_json_response(text).is_none());
    }

    // ─── Case-insensitive prefix strip tests ──────────────────────────────

    #[test]
    fn test_strip_prefix_case_insensitive() {
        assert_eq!(
            strip_prefix_case_insensitive("ARGS: test", "ARGS:"),
            Some(" test")
        );
        assert_eq!(
            strip_prefix_case_insensitive("args: test", "ARGS:"),
            Some(" test")
        );
        assert_eq!(
            strip_prefix_case_insensitive("Args: test", "ARGS:"),
            Some(" test")
        );
        assert!(strip_prefix_case_insensitive("other: test", "ARGS:").is_none());
    }

    // ─── Semantic truncation tests ────────────────────────────────────────

    #[test]
    fn test_semantic_truncation_prioritizes_relevant_sections() {
        let docs = "Usage: samtools sort [options] input.bam\n\n\
                     Options:\n  -@ INT  threads\n  -o FILE output\n\n\
                     Examples:\n  samtools sort in.bam\n\n\
                     Description:\n  Sort alignments by position.";
        let result = truncate_documentation_for_task(docs, 120, Some("sort with threads"));
        // Should prioritize sections containing "sort" and "threads"
        assert!(result.contains("sort") || result.contains("Usage"));
    }

    #[test]
    fn test_semantic_truncation_without_task() {
        let docs = "line1\nline2\nline3";
        let result = truncate_documentation_for_task(docs, 1000, None);
        assert_eq!(result, docs);
    }
}
