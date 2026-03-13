use crate::config::Config;
use crate::error::{OxoError, Result};
use crate::skill::Skill;
use serde::{Deserialize, Serialize};

/// A parsed LLM response with command arguments and explanation
#[derive(Debug, Clone)]
pub struct LlmCommandSuggestion {
    pub args: Vec<String>,
    pub explanation: String,
    #[allow(dead_code)]
    pub raw_response: String,
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

// ─── System prompt ────────────────────────────────────────────────────────────

fn system_prompt() -> &'static str {
    "You are an expert bioinformatics command-line assistant with deep knowledge of \
     genomics, transcriptomics, epigenomics, metagenomics, and single-cell biology. \
     Your task is to translate the user's task description into the exact command-line \
     arguments for the specified bioinformatics tool. \
     The task description may be written in any language (English, Chinese, Japanese, \
     Korean, etc.) — understand it fully regardless of language. \
     Rules: \
     (1) Only use flags/options explicitly present in the provided documentation or examples. \
     (2) Never include the tool name itself in ARGS — it is prepended automatically. \
     (3) Always include any file names or paths mentioned in the task description. \
     (4) Prefer complete, production-ready commands with appropriate thread counts and output files. \
     (5) If the task is ambiguous, choose the most common bioinformatics convention \
         (e.g., paired-end, coordinate-sorted BAM, human hg38 genome build). \
     (6) Never hallucinate flags that are not in the documentation. \
     (7) For multi-step tools (configure+run workflows), include both steps joined with &&. \
     (8) Use best practices: include -@ or -t flags for multithreading when available, \
         use -o for output files, and include index/reference files when required by the tool. \
     (9) Always match file format flags to the actual input/output types \
         (BAM vs SAM, gzipped vs plain, paired-end vs single-end). \
     (10) When the task mentions library strandedness, set the correct strand flag for the tool. \
     (11) ARGS must always be valid CLI flags/values (ASCII, tool-specific syntax). \
          EXPLANATION should be written in the same language as the task description."
}

// ─── User prompt ─────────────────────────────────────────────────────────────

/// Build the enriched user prompt, injecting skill knowledge when available.
fn build_prompt(tool: &str, documentation: &str, task: &str, skill: Option<&Skill>) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("# Tool: `{tool}`\n\n"));

    // Inject skill knowledge (concepts, pitfalls, examples) before the raw docs.
    // This primes the LLM with expert knowledge before it reads the potentially
    // noisy --help output — especially important for small/weak models.
    if let Some(skill) = skill {
        let section = skill.to_prompt_section();
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

    // Strict format instructions — critical for reliable parsing with weak LLMs
    prompt.push_str(
        "## Output Format (STRICT — do not add any other text)\n\
         Respond with EXACTLY two lines:\n\
         \n\
         ARGS: <all command-line arguments, space-separated, WITHOUT the tool name itself>\n\
         EXPLANATION: <one concise sentence explaining what the command does>\n\
         \n\
         RULES:\n\
         - ARGS must NOT start with the tool name\n\
         - ARGS must only contain valid CLI flags and values (ASCII, tool syntax)\n\
         - EXPLANATION should be written in the same language as the Task above\n\
         - Include every file path mentioned in the task\n\
         - Use only flags documented above or shown in the skill examples\n\
         - Prefer flags from the skill examples when they match the task\n\
         - If no arguments are needed, write: ARGS: (none)\n\
         - Do NOT add markdown, code fences, or extra explanation\n",
    );

    prompt
}

// ─── Task optimization prompt ─────────────────────────────────────────────────

/// Build a prompt that asks the LLM to expand and clarify a raw task description
/// into a precise, unambiguous bioinformatics instruction.
fn build_task_optimization_prompt(tool: &str, raw_task: &str) -> String {
    format!(
        "# Task Optimization Request\n\n\
         Tool: `{tool}`\n\
         User's original task description: {raw_task}\n\n\
         Your job is to rewrite the task description as a precise, complete bioinformatics \
         instruction. The rewritten task should:\n\
         - Clarify any ambiguous terms (e.g., 'sort bam' → 'sort BAM by coordinate using \
           samtools sort and output to sorted.bam')\n\
         - Infer reasonable defaults (paired-end, hg38, 8 threads, gzipped output, etc.) \
           when not specified\n\
         - Preserve all file names and paths mentioned in the original task\n\
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
    "You are an expert bioinformatics QC analyst. Your task is to analyze the output \
     of a bioinformatics command execution and determine whether it completed \
     successfully. You understand common error patterns, expected output structures, \
     and tool-specific behaviors. Respond in the same language as the task description."
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
         Analyze whether this command ran successfully. Consider:\n\
         1. Exit code (0 = success for most tools; some tools use non-zero for warnings)\n\
         2. Error keywords in stderr (e.g., ERROR, FATAL, Exception, Traceback, \
            Segmentation fault, Killed, Out of memory)\n\
         3. Missing expected output files or zero-byte outputs\n\
         4. Tool-specific patterns (e.g., samtools warnings about truncated BAM, \
            STAR alignment rate < 50%%, GATK MalformedRead)\n\n\
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

/// Build a corrective retry prompt when the first attempt had an invalid response.
fn build_retry_prompt(
    tool: &str,
    documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    prev_raw: &str,
) -> String {
    let base = build_prompt(tool, documentation, task, skill);
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
    pub async fn suggest_command(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
    ) -> Result<LlmCommandSuggestion> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            const MAX_RETRIES: usize = 2;

            let mut last_raw = String::new();

            for attempt in 0..=MAX_RETRIES {
                let user_prompt = if attempt == 0 {
                    build_prompt(tool, documentation, task, skill)
                } else {
                    build_retry_prompt(tool, documentation, task, skill, &last_raw)
                };

                let raw = self.call_api(&user_prompt).await?;
                let suggestion = Self::parse_response(&raw)?;

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
    async fn call_api(&self, user_prompt: &str) -> Result<String> {
        self.request_with_system(system_prompt(), user_prompt, None, None)
            .await
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
            let token = self.config.effective_api_token().ok_or_else(|| {
                let token_hint = match provider.as_str() {
                    "github-copilot" => "  For GitHub Copilot, use a GitHub token with copilot scope:\n    https://github.com/settings/tokens",
                    "openai" => "  For OpenAI, create an API key at:\n    https://platform.openai.com/api-keys",
                    "anthropic" => "  For Anthropic, create an API key at:\n    https://console.anthropic.com/settings/keys",
                    "ollama" => "  For Ollama (local), no token is usually needed.\n    Set OXO_CALL_LLM_API_TOKEN if your instance requires auth.",
                    _ => "  Check your provider's documentation for token setup.",
                };
                OxoError::LlmError(
                    format!(
                        "No API token configured for provider '{provider}'.\n\n\
                        Option 1 — Set via config (recommended):\n  \
                          oxo-call config set llm.api_token <your-token>\n\n\
                        Option 2 — Set via environment variable:\n  \
                          export OXO_CALL_LLM_API_TOKEN=<your-token>\n\n\
                        How to get a token:\n{token_hint}\n\n\
                        Test your setup: oxo-call config verify"
                    ),
                )
            })?;

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

            req_builder = match provider.as_str() {
                "anthropic" => req_builder
                    .header("x-api-key", &token)
                    .header("anthropic-version", "2023-06-01"),
                _ => req_builder.header("Authorization", format!("Bearer {token}")),
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

    fn parse_response(raw: &str) -> Result<LlmCommandSuggestion> {
        let mut args_line = String::new();
        let mut explanation_line = String::new();

        for line in raw.lines() {
            if let Some(rest) = line.strip_prefix("ARGS:") {
                args_line = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("EXPLANATION:") {
                explanation_line = rest.trim().to_string();
            }
        }

        // Treat "(none)" as empty args
        if args_line == "(none)" {
            args_line.clear();
        }

        let args = parse_shell_args(&args_line);

        Ok(LlmCommandSuggestion {
            args,
            explanation: explanation_line,
            raw_response: raw.to_string(),
        })
    }
}

/// Check whether a suggestion looks valid enough to return without retrying.
fn is_valid_suggestion(suggestion: &LlmCommandSuggestion) -> bool {
    // At minimum we need an explanation (ARGS can legitimately be empty)
    !suggestion.explanation.is_empty()
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
                    args.push(current.clone());
                    current.clear();
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
}
