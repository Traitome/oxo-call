//! LLM provider implementation.
//!
//! This module contains the `LlmClient` struct and its implementation for
//! interacting with various LLM providers (OpenAI, Anthropic, GitHub Copilot, Ollama).

use crate::config::Config;
use crate::copilot_auth;
use crate::doc_processor::{FlagEntry, StructuredDoc};
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::schema::CliSchema; // HDA: Schema passed from runner, not parsed here
use crate::skill::{Skill, SkillManager};
use sha2::Digest;

use super::prompt::{
    build_prompt, build_retry_prompt, build_skill_generate_prompt,
    build_skill_generate_prompt_enhanced, build_skill_polish_prompt, build_skill_verify_prompt,
    build_task_optimization_prompt, build_verification_prompt, skill_generator_system_prompt,
    skill_reviewer_system_prompt, system_prompt, system_prompt_compact, system_prompt_medium,
    verification_system_prompt,
};
use super::response::{
    is_valid_suggestion, parse_response, parse_skill_verify_response, parse_verification_response,
    sanitize_args, strip_markdown_fences,
};
use super::streaming::apply_provider_auth_headers;
use super::types::{
    ChatMessage, ChatRequest, ChatRequestStreaming, ChatResponse, LlmCommandSuggestion,
    LlmRunVerification, LlmSkillVerification, LlmVerificationResult, PromptTier,
};
use crate::streaming_display;

pub struct LlmClient {
    pub(crate) config: Config,
    client: reqwest::Client,
    /// Whether to use SSE streaming for LLM responses.
    /// When true, tokens are printed to stderr as they arrive.
    stream_enabled: bool,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_max_idle_per_host(16)
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to build configured HTTP client: {e}; using defaults");
                reqwest::Client::new()
            });
        let stream_enabled = config.llm.stream;
        LlmClient {
            config,
            client,
            stream_enabled,
        }
    }

    /// Disable streaming for this client instance (convenience for `--no-stream`).
    pub fn set_no_stream(&mut self, no_stream: bool) {
        if no_stream {
            self.stream_enabled = false;
        }
    }

    /// Generate command arguments, using skill knowledge for better prompts.
    /// Retries up to `MAX_RETRIES` times when the response format is invalid.
    ///
    /// When the model has a known context window, prompts are adaptively
    /// compressed to fit — see [`PromptTier`].
    ///
    /// When `structured_doc` is provided (from `DocProcessor::clean_and_structure`),
    /// the prompt gains doc-extracted examples as few-shot demonstrations and a
    /// compact flag catalog — critical for ≤3B model accuracy.
    /// When `schema` is provided (from HDA), it provides the whitelist of valid flags
    /// parsed from raw docs (parsed in runner for accuracy).
    #[allow(clippy::too_many_arguments)]
    pub async fn suggest_command(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
        schema: Option<&CliSchema>,
    ) -> Result<LlmCommandSuggestion> {
        const MAX_RETRIES: usize = 2;

        let context_window = self.config.effective_context_window();
        let tier = self.config.effective_prompt_tier();
        let model = self.config.effective_model();
        let profile = crate::config::get_model_profile(&model);
        let temperature = Some(profile.optimal_temperature);

        // Compute docs hash for cache key
        let docs_hash = if documentation.is_empty() {
            None
        } else {
            Some(hex::encode(sha2::Sha256::digest(documentation.as_bytes())))
        };
        // Use skill name reference directly without clone
        let skill_name: Option<&str> = skill.as_ref().map(|s| s.meta.name.as_str());

        // Try cache lookup first
        if let Ok(Some(cached)) = crate::cache::LlmCache::lookup(
            tool,
            task,
            docs_hash.as_deref(),
            skill_name, // Already Option<&str>
            &model,
        )
        .await
        {
            // Cache hit - return cached response
            // Parse cached args string into Vec<String>
            let args_vec = cached.args.split_whitespace().map(String::from).collect();
            return Ok(LlmCommandSuggestion {
                args: args_vec,
                explanation: cached.explanation,
                inference_ms: 0.0, // Cache hit has no inference time
            });
        }

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
                // HDA: Schema is passed from runner (parsed from raw docs), use directly
                let prompt = build_prompt(
                    tool,
                    effective_docs,
                    task,
                    skill,
                    no_prompt,
                    context_window,
                    tier,
                    structured_doc,
                    schema, // HDA: Schema whitelist passed from runner
                );
                // Debug: write prompt to file for analysis
                if std::env::var("OXO_DEBUG_PROMPT").is_ok() {
                    let _ = std::fs::write("/tmp/oxo_prompt_debug.txt", &prompt);
                }
                prompt
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
                    structured_doc,
                    schema, // HDA: Schema whitelist passed from runner
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
            let raw = self
                .call_api(&user_prompt, no_prompt, tier, temperature)
                .await?;
            total_inference_ms += api_start.elapsed().as_secs_f64() * 1000.0;

            // Detect empty/blank responses (model was overwhelmed)
            if raw.trim().is_empty() {
                had_empty_output = true;
            }

            let mut suggestion = parse_response(&raw)?;
            suggestion.inference_ms = total_inference_ms;

            // Post-process: strip accidental tool name prefix
            suggestion.args = sanitize_args(tool, suggestion.args);

            // Post-process: validate flags against doc catalog when available
            if let Some(sdoc) = structured_doc
                && !sdoc.flag_catalog.is_empty()
            {
                suggestion.args = validate_flags_against_catalog(
                    &suggestion.args,
                    &sdoc.flag_catalog,
                    &sdoc.quick_flags,
                );
            }

            if is_valid_suggestion(&suggestion) {
                // Store successful result in cache
                let args_str = suggestion.args.join(" ");
                let _ = crate::cache::LlmCache::store(
                    tool,
                    task,
                    docs_hash.as_deref(),
                    skill_name, // Already Option<&str>
                    &model,
                    &args_str,
                    &suggestion.explanation,
                )
                .await;
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

    pub async fn verify_configuration(&self) -> Result<LlmVerificationResult> {
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

    /// Use the LLM to optimize/expand a raw task description into a precise instruction.
    ///
    /// Returns the refined task text on success, or falls back to the original task
    /// if the LLM response is not parseable.  Errors from the API are propagated.
    pub async fn optimize_task(&self, tool: &str, raw_task: &str) -> Result<String> {
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

    /// Make a raw chat completion call with custom system prompt.
    ///
    /// This is a low-level API for specialized workflows (e.g., mini-skill generation).
    pub async fn chat_completion(
        &self,
        system: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<String> {
        self.request_with_system(system, user_prompt, max_tokens, temperature)
            .await
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

    /// Make the raw API call and return the assistant message content.
    /// When no_prompt is true (bare mode), no system prompt is sent to test raw LLM capability.
    async fn call_api(
        &self,
        user_prompt: &str,
        no_prompt: bool,
        tier: PromptTier,
        temperature: Option<f32>,
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
            let raw = self
                .request_few_shot(sys_prompt, user_prompt, temperature)
                .await?;
            return Ok(raw);
        }

        self.request_with_system(sys_prompt, user_prompt, None, temperature)
            .await
    }

    /// Send a few-shot request using multi-turn messages.
    ///
    /// The `user_prompt` is split at `---FEW-SHOT---` boundaries to create
    /// user/assistant message pairs.  This is critical for small models (≤ 3B)
    /// which cannot reliably follow output format instructions in a single
    /// user prompt, but can imitate the format when shown an assistant example.
    async fn request_few_shot(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        let provider = self.config.effective_provider();

        // Use ollama native API for better compatibility with thinking models
        if provider == "ollama" {
            return self
                .request_ollama_native(sys_prompt, user_prompt, temperature)
                .await;
        }

        let token = if self.config.provider_requires_token() {
            self.config
                .effective_api_token()
                .ok_or_else(|| OxoError::LlmError("No API token configured".to_string()))?
        } else {
            String::new()
        };
        let api_base = self.config.effective_api_base();
        let model = self.config.effective_model();
        // Build URL efficiently without format! allocation
        let mut url = String::with_capacity(api_base.len() + 18);
        url.push_str(&api_base);
        url.push_str("/chat/completions");

        // Build messages: system + alternating user/assistant pairs + final user
        let mut messages = Vec::new();

        if !sys_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: sys_prompt.to_string(),
                reasoning: None,
            });
        }

        // Split at few-shot boundaries.
        let parts: Vec<&str> = user_prompt
            .split("\n\n---FEW-SHOT---\n\n")
            .filter(|p| !p.is_empty())
            .collect();

        if parts.len() >= 2 {
            let mut is_assistant = false;
            for part in &parts {
                if is_assistant {
                    messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: part.to_string(),
                        reasoning: None,
                    });
                } else {
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: part.to_string(),
                        reasoning: None,
                    });
                }
                is_assistant = !is_assistant;
            }
        } else {
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
                reasoning: None,
            });
        }

        let max_tokens = self.config.effective_max_tokens()?;
        let temp = temperature.unwrap_or_else(|| {
            let profile = crate::config::get_model_profile(&model);
            profile.optimal_temperature
        });

        // Auth handling
        let auth_token = if provider == "github-copilot" {
            let manager = copilot_auth::get_token_manager();
            manager.get_session_token(&token).await?
        } else {
            token.clone()
        };

        // ── Streaming path ────────────────────────────────────────────
        if self.stream_enabled {
            let request = ChatRequestStreaming {
                model: model.clone(),
                messages,
                max_tokens,
                temperature: temp,
                stream: true,
            };

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json");

            req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

            let resp = req_builder.json(&request).send().await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(OxoError::LlmError(format!(
                    "LLM API error: {status} — {body}"
                )));
            }

            // Use StreamingDisplay for spinner + live preview + final clear
            let content = streaming_display::read_sse_with_display(
                resp,
                streaming_display::StreamingDisplayConfig {
                    message: "Generating command".to_string(),
                    max_preview_lines: 2,
                    show_preview: true,
                },
            )
            .await
            .map_err(OxoError::LlmError)?;
            return Ok(content);
        }

        // ── Non-streaming path ────────────────────────────────────────
        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens,
            temperature: temp,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

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
            .map(|c| {
                // For thinking models (qwen3.5, deepseek-r1), the actual response
                // may be in the reasoning field when content is empty
                if c.message.content.trim().is_empty()
                    && let Some(ref reasoning) = c.message.reasoning
                {
                    return reasoning.clone();
                }
                c.message.content.clone()
            })
            .unwrap_or_default();

        Ok(content.trim().to_string())
    }

    /// Send a request using ollama native `/api/chat` endpoint.
    ///
    /// This is more reliable for ollama models, especially thinking models
    /// like qwen3.5 and deepseek-r1 which have issues with the OpenAI-compatible
    /// `/v1/chat/completions` API (reasoning/content separation).
    async fn request_ollama_native(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        use super::types::{
            OllamaChatMessage, OllamaChatRequest, OllamaChatResponse, OllamaOptions,
        };

        let api_base = self.config.effective_api_base();
        // Strip /v1 suffix if present (ollama native API doesn't use it)
        let api_base = api_base.trim_end_matches("/v1");
        // Build URL efficiently without format! allocation
        let mut url = String::with_capacity(api_base.len() + 10);
        url.push_str(api_base);
        url.push_str("/api/chat");
        let model = self.config.effective_model();

        // Build messages
        let mut messages = Vec::new();
        if !sys_prompt.is_empty() {
            messages.push(OllamaChatMessage {
                role: "system".to_string(),
                content: sys_prompt.to_string(),
            });
        }

        // Split at few-shot boundaries
        let parts: Vec<&str> = user_prompt
            .split("\n\n---FEW-SHOT---\n\n")
            .filter(|p| !p.is_empty())
            .collect();

        if parts.len() >= 2 {
            let mut is_assistant = false;
            for part in &parts {
                messages.push(OllamaChatMessage {
                    role: if is_assistant { "assistant" } else { "user" }.to_string(),
                    content: part.to_string(),
                });
                is_assistant = !is_assistant;
            }
        } else {
            messages.push(OllamaChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            });
        }

        let max_tokens = self.config.effective_max_tokens()?;
        let temp = temperature.unwrap_or_else(|| {
            let profile = crate::config::get_model_profile(&model);
            profile.optimal_temperature
        });

        let request = OllamaChatRequest {
            model: model.clone(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                num_ctx: Some(max_tokens),
                temperature: Some(temp),
                num_predict: Some(max_tokens),
            }),
            // Disable thinking mode for all ollama requests to ensure
            // the model returns actual content instead of just reasoning
            think: Some(false),
        };

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(OxoError::LlmError(format!(
                "Ollama API error: {status} — {body}"
            )));
        }

        let ollama_resp: OllamaChatResponse = serde_json::from_str(&body)?;
        let content = if ollama_resp.message.content.trim().is_empty() {
            // Fallback to thinking field when content is empty
            ollama_resp.message.thinking.clone().unwrap_or_default()
        } else {
            ollama_resp.message.content.clone()
        };
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
    ///
    /// When `self.stream_enabled` is true, the request is sent with `"stream": true`
    /// and tokens are printed to stderr as they arrive (SSE protocol).
    async fn request_with_system(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
        max_tokens_override: Option<u32>,
        temperature_override: Option<f32>,
    ) -> Result<String> {
        let provider = self.config.effective_provider();

        // Use ollama native API for better compatibility
        if provider == "ollama" {
            return self
                .request_ollama_native(sys_prompt, user_prompt, temperature_override)
                .await;
        }

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
            // For token-optional providers (e.g. Ollama), always use an empty
            // string so that no Authorization header is sent.  This avoids
            // leaking a leftover token from a previous provider configuration.
            String::new()
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
        // Build URL efficiently without format! allocation
        let mut url = String::with_capacity(api_base.len() + 18);
        url.push_str(&api_base);
        url.push_str("/chat/completions");

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: sys_prompt.to_string(),
                reasoning: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
                reasoning: None,
            },
        ];

        let max_tokens = max_tokens_override.unwrap_or(self.config.effective_max_tokens()?);
        let temperature = temperature_override.unwrap_or_else(|| {
            // Use model-specific optimal temperature as fallback
            let profile = crate::config::get_model_profile(&model);
            profile.optimal_temperature
        });

        // For github-copilot, we need to exchange the GitHub token for a Copilot session token
        let auth_token = if provider == "github-copilot" {
            let manager = copilot_auth::get_token_manager();
            manager.get_session_token(&token).await?
        } else {
            token.clone()
        };

        // ── Streaming path ────────────────────────────────────────────────
        if self.stream_enabled {
            let request = ChatRequestStreaming {
                model: model.clone(),
                messages,
                max_tokens,
                temperature,
                stream: true,
            };

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json");

            req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

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

            // Use StreamingDisplay for spinner + live preview + final clear
            let content = streaming_display::read_sse_with_display(
                response,
                streaming_display::StreamingDisplayConfig {
                    message: "Processing".to_string(),
                    max_preview_lines: 2,
                    show_preview: true,
                },
            )
            .await
            .map_err(OxoError::LlmError)?;
            return Ok(content);
        }

        // ── Non-streaming path (original) ─────────────────────────────────
        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens,
            temperature,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

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
            .map(|c| {
                // For thinking models, use reasoning when content is empty
                if c.message.content.trim().is_empty()
                    && let Some(ref reasoning) = c.message.reasoning
                {
                    return reasoning.clone();
                }
                c.message.content.clone()
            })
            .unwrap_or_default())
    }

    /// Ask the LLM to review a skill file for quality and completeness.
    ///
    /// Returns a structured `LlmSkillVerification` with findings and suggestions.
    pub async fn verify_skill(
        &self,
        tool: &str,
        skill_content: &str,
    ) -> Result<LlmSkillVerification> {
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

    /// Ask the LLM to rewrite and improve a skill file, returning the enhanced Markdown.
    ///
    /// The LLM is instructed to preserve the tool name and all correct information
    /// while adding missing concepts/pitfalls/examples, fixing format issues, and
    /// improving clarity.
    pub async fn polish_skill(&self, tool: &str, skill_content: &str) -> Result<String> {
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

    /// Use LLM to generate an initial skill template pre-filled with domain knowledge.
    ///
    /// Returns a Markdown-format skill file (YAML front-matter + body sections).
    pub async fn generate_skill_template(&self, tool: &str) -> Result<String> {
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

    /// Use LLM to generate a skill template with skill-generator workflow integration.
    ///
    /// This enhanced version:
    /// 1. Loads the skill-generator skill content for workflow guidance
    /// 2. Fetches local --help output for the tool (if available)
    /// 3. Uses structured prompts optimized for small model compatibility
    ///
    /// Returns a Markdown-format skill file (YAML front-matter + body sections).
    pub async fn generate_skill_template_enhanced(&self, tool: &str) -> Result<String> {
        // 1. Load skill-generator skill content
        let skill_mgr = SkillManager::new(self.config.clone());
        let generator_skill = skill_mgr.load("skill-generator");

        let generator_skill_content = generator_skill.as_ref().map(|s| {
            // Render the skill as prompt section
            s.to_prompt_section()
        });

        // 2. Fetch local --help output
        let docs_fetcher = DocsFetcher::new(self.config.clone());
        let tool_docs = docs_fetcher.fetch(tool).await.ok();
        let help_output = tool_docs.as_ref().and_then(|d| d.help_output.as_ref());

        // 3. Build enhanced prompt
        let user_prompt = build_skill_generate_prompt_enhanced(
            tool,
            help_output.map(|s| s.as_str()),
            generator_skill_content.as_deref(),
        );

        // 4. Call LLM with skill-generator system prompt
        let raw = self
            .request_with_system(
                skill_generator_system_prompt(),
                &user_prompt,
                Some(8192), // Larger budget for comprehensive skill generation
                Some(0.3),  // Lower temperature for structured output
            )
            .await?;

        Ok(strip_markdown_fences(&raw))
    }

    /// Generate a shell command from a plain-English description.
    ///
    /// Returns `(command, explanation)`.  The command is a ready-to-run shell
    /// string; the explanation is a brief one-liner.
    pub async fn generate_shell_command(&self, description: &str) -> Result<(String, String)> {
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

// ─── LlmProvider trait implementation ─────────────────────────────────────────

impl super::types::LlmProvider for LlmClient {
    async fn chat_completion(
        &self,
        system: &str,
        user_prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String> {
        self.request_with_system(system, user_prompt, Some(max_tokens), Some(temperature))
            .await
    }

    fn name(&self) -> &str {
        match self.config.effective_provider().as_str() {
            "openai" => "openai",
            "anthropic" => "anthropic",
            "github-copilot" => "github-copilot",
            "ollama" => "ollama",
            "deepseek" => "deepseek",
            "moonshot" | "kimi" => "moonshot",
            "zhipu" | "glm" => "zhipu",
            "minimax" => "minimax",
            _ => "custom",
        }
    }
}

/// Validate LLM-generated flags against the documentation flag catalog.
///
/// This is a post-processing step that catches hallucinated flags:
/// - Flags present in the catalog or quick_flags → kept as-is
/// - Common well-known flags (e.g., `-o`, `-t`, `--output`) → kept
/// - Subcommands (no `-` prefix) → kept
/// - Values (follow a flag) → kept
/// - Unknown flags → kept but logged via tracing
///
/// This is a **soft** validation — we don't remove unknown flags because
/// the model might use correct flags not captured by our simple regex-based
/// extraction. The catalog in the prompt prevents hallucination at
/// generation time; this layer catches what slips through.
fn validate_flags_against_catalog(
    args: &[String],
    catalog: &[FlagEntry],
    quick_flags: &[String],
) -> Vec<String> {
    // Build a set of known flags from the catalog.
    let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in catalog {
        for part in entry.flag.split([',', ' ', '\t']) {
            let part = part.trim();
            if part.starts_with('-') {
                known.insert(part.trim_end_matches('=').to_string());
            }
        }
    }
    // Also accept quick_flags as known.
    for qf in quick_flags {
        for part in qf.split([',', ' ', '\t']) {
            let part = part.trim();
            if part.starts_with('-') {
                known.insert(part.trim_end_matches('=').to_string());
            }
        }
    }

    // Universal flags that most CLI tools accept.
    for &universal in &[
        "-h",
        "--help",
        "-v",
        "--version",
        "-V",
        "--verbose",
        "-q",
        "--quiet",
        "-o",
        "--output",
        "-i",
        "--input",
        "-t",
        "--threads",
        "-f",
        "--force",
    ] {
        known.insert(universal.to_string());
    }

    // Walk args and log unknown flags (but keep them).
    for arg in args {
        if !arg.starts_with('-') {
            continue;
        }
        let flag = if arg.starts_with("--") {
            arg.split('=').next().unwrap_or(arg)
        } else {
            arg.as_str()
        };

        if !known.contains(flag) {
            tracing::debug!("Flag not in doc catalog (may still be valid): {flag}");
        }
    }

    args.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_llm_client_new_defaults() {
        let client = LlmClient::new(Config::default());
        // stream_enabled should match config default (true)
        assert!(client.stream_enabled);
    }

    #[test]
    fn test_set_no_stream_disables_streaming() {
        let cases: Vec<(bool, bool)> = vec![
            (true, false),   // set no_stream=true → stream_enabled=false
            (false, true),   // set no_stream=false → stream_enabled unchanged (true by default)
        ];
        for (no_stream, expected_stream) in cases {
            let mut client = LlmClient::new(Config::default());
            client.set_no_stream(no_stream);
            assert_eq!(
                client.stream_enabled, expected_stream,
                "no_stream={no_stream} → stream_enabled should be {expected_stream}"
            );
        }
    }

    #[test]
    fn test_validate_flags_against_catalog_keeps_all() {
        use crate::doc_processor::FlagEntry;
        let catalog = vec![
            FlagEntry { flag: "-o".to_string(), description: "output".to_string() },
            FlagEntry { flag: "--threads".to_string(), description: "threads".to_string() },
        ];
        let quick_flags: Vec<String> = vec![];
        let args: Vec<String> = vec![
            "-o".to_string(),
            "out.bam".to_string(),
            "--threads".to_string(),
            "4".to_string(),
        ];
        let result = validate_flags_against_catalog(&args, &catalog, &quick_flags);
        // validate_flags_against_catalog is a soft pass-through: all args are returned.
        assert_eq!(result.len(), args.len());
    }

    #[test]
    fn test_validate_flags_unknown_flag_still_kept() {
        use crate::doc_processor::FlagEntry;
        let catalog: Vec<FlagEntry> = vec![];
        let quick_flags: Vec<String> = vec![];
        let args: Vec<String> = vec!["--unknown-flag".to_string(), "value".to_string()];
        let result = validate_flags_against_catalog(&args, &catalog, &quick_flags);
        // Soft validation: keeps unknown flags too.
        assert_eq!(result, args);
    }

    #[test]
    fn test_validate_flags_universal_flags_kept() {
        use crate::doc_processor::FlagEntry;
        let catalog: Vec<FlagEntry> = vec![];
        let quick_flags: Vec<String> = vec![];
        // Universal flags should be recognised even with empty catalog.
        let args: Vec<String> = vec![
            "-h".to_string(),
            "--help".to_string(),
            "-v".to_string(),
            "--version".to_string(),
        ];
        let result = validate_flags_against_catalog(&args, &catalog, &quick_flags);
        assert_eq!(result.len(), args.len());
    }
}
