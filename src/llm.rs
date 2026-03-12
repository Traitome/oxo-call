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

    /// Make the raw API call and return the assistant message content.
    async fn call_api(&self, user_prompt: &str) -> Result<String> {
        self.request_text(user_prompt, None, None).await
    }

    async fn request_text(
        &self,
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
            OxoError::LlmError(
                "No API token configured. Set it with:\n  oxo-call config set llm.api_token <token>\n\
                Or set the environment variable OXO_CALL_LLM_API_TOKEN.\n\
                Backward-compatible provider-specific variables still work too: \
                GITHUB_TOKEN / GH_TOKEN, OPENAI_API_KEY, ANTHROPIC_API_KEY, OXO_API_TOKEN."
                    .to_string(),
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
                    content: system_prompt().to_string(),
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
