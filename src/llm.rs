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

// ─── Skill reviewer prompts ───────────────────────────────────────────────────

/// System prompt for the skill reviewer / editor persona.
fn skill_reviewer_system_prompt() -> &'static str {
    "You are an expert bioinformatics skill author for the oxo-call tool. \
     You deeply understand the oxo-call skill file format (YAML front-matter + Markdown sections) \
     and how skills are used to improve LLM command generation quality. \
     A high-quality skill file must have: \
     (1) Complete YAML front-matter with name, category, description, tags, author, source_url. \
     (2) A '## Concepts' section with ≥3 bullet points covering key data model and paradigm concepts. \
     (3) A '## Pitfalls' section with ≥3 bullet points covering common mistakes and their consequences. \
     (4) An '## Examples' section with ≥5 subsections, each starting with '### <task description>', \
         followed by '**Args:** `<flags>`' and '**Explanation:** <sentence>'. \
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
         - '## Concepts' section with ≥3 specific, actionable bullet points about the tool's data model and key behaviors\n\
         - '## Pitfalls' section with ≥3 bullet points about common mistakes and their consequences\n\
         - '## Examples' section with ≥5 realistic subsections, each:\n\
             ### <task description in plain English>\n\
             **Args:** `<exact CLI flags without tool name>`\n\
             **Explanation:** <one sentence explaining why these flags>\n\n\
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

    /// Ask the LLM to review a skill file for quality and completeness.
    ///
    /// Returns a structured `LlmSkillVerification` with findings and suggestions.
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
    pub async fn generate_shell_command(&self, description: &str) -> Result<(String, String)> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "LLM API calls are not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let system = "You are a shell command expert. \
                Given a plain-English description, produce a single shell command \
                (or short pipeline) that accomplishes the task on a Linux/macOS system. \
                Reply with exactly two lines and nothing else:\n\
                COMMAND: <the shell command>\n\
                EXPLANATION: <one-sentence explanation>";

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
        };
        assert!(is_valid_suggestion(&s));
    }

    #[test]
    fn test_is_valid_suggestion_empty_explanation() {
        let s = LlmCommandSuggestion {
            args: vec!["-o".to_string()],
            explanation: String::new(),
            raw_response: "ARGS: -o\nEXPLANATION:".to_string(),
        };
        assert!(!is_valid_suggestion(&s));
    }

    #[test]
    fn test_is_valid_suggestion_empty_args_but_has_explanation() {
        let s = LlmCommandSuggestion {
            args: vec![],
            explanation: "Run the tool with default arguments.".to_string(),
            raw_response: "ARGS:\nEXPLANATION: Run the tool with default arguments.".to_string(),
        };
        // ARGS can be empty, explanation is what matters
        assert!(is_valid_suggestion(&s));
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
        let prompt = build_prompt("samtools", "docs", "sort bam", Some(&skill));
        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("concept 1"));
        assert!(prompt.contains("pitfall 1"));
        assert!(prompt.contains("sort bam"));
    }

    #[test]
    fn test_build_prompt_format_instructions() {
        let prompt = build_prompt("bwa", "bwa mem --help", "align reads", None);
        assert!(
            prompt.contains("ARGS:"),
            "should contain ARGS: format instruction"
        );
        assert!(
            prompt.contains("EXPLANATION:"),
            "should contain EXPLANATION: format instruction"
        );
        assert!(prompt.contains("RULES:"), "should contain RULES section");
    }

    // ─── build_retry_prompt ───────────────────────────────────────────────────

    #[test]
    fn test_build_retry_prompt_contains_prev_response() {
        let prev = "THIS IS WRONG FORMAT";
        let prompt = build_retry_prompt("samtools", "docs", "sort bam", None, prev);
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
                .suggest_command("samtools", "samtools --help output", "sort bam", None)
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
                .suggest_command("samtools", "docs", "sort", None)
                .await;

            assert!(result.is_err(), "500 should produce an error");
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
                .suggest_command("samtools", "docs", "sort bam", None)
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
                .suggest_command("samtools", "docs", "sort", None)
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
                .suggest_command("samtools", "docs", "sort", None)
                .await;

            // Empty choices → empty string → parse_response returns empty suggestion
            assert!(result.is_ok());
        }
    }
}
