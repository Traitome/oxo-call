//! Type definitions for LLM client.
//!
//! This module contains all data structures, enums, and traits used by the
//! LLM subsystem.

use serde::{Deserialize, Serialize};

/// A parsed LLM response with command arguments and explanation
#[derive(Debug, Clone)]
pub struct LlmCommandSuggestion {
    pub args: Vec<String>,
    pub explanation: String,
    /// Cumulative time (ms) spent in LLM API inference calls for this
    /// suggestion.  When retries occur, all attempts are summed.
    /// Zero indicates the result was served from cache.
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
#[allow(async_fn_in_trait, dead_code)]
pub trait LlmProvider {
    /// Send a chat completion request and return the assistant's raw text.
    async fn chat_completion(
        &self,
        system: &str,
        user_prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> crate::error::Result<String>;

    /// Human-readable provider name (e.g. "openai", "anthropic").
    fn name(&self) -> &str;
}

#[derive(Debug, Serialize)]
pub(crate) struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ChatMessage {
    pub role: String,
    pub content: String,
    /// Thinking/reasoning content from models like qwen3.5, deepseek-r1
    /// When content is empty, the actual response may be in reasoning
    #[serde(default)]
    pub reasoning: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatChoice {
    pub message: ChatMessage,
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

// ─── SSE streaming response types ────────────────────────────────────────────

/// A single server-sent event (SSE) chunk from an OpenAI-compatible streaming
/// chat completion response.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamChunkResponse {
    pub choices: Vec<StreamChoice>,
}

/// One choice inside a streaming chunk.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamChoice {
    pub delta: StreamDelta,
    /// Present on the final chunk.
    #[serde(default)]
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

/// The incremental delta in a streaming choice.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamDelta {
    /// Incremental text content (may be absent on the first or last chunk).
    #[serde(default)]
    pub content: Option<String>,
    /// Thinking/reasoning content from models like qwen3.5, deepseek-r1
    #[serde(default)]
    pub reasoning: Option<String>,
}

/// Extended chat request that includes the optional `stream` flag.
#[derive(Debug, Serialize)]
pub(crate) struct ChatRequestStreaming {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
}

// ─── Ollama native API types ─────────────────────────────────────────────────

/// Ollama native `/api/chat` request format.
#[derive(Debug, Serialize)]
pub(crate) struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
    /// Disable thinking/reasoning mode for thinking models (qwen3.5, deepseek-r1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<bool>,
}

/// A single message in the ollama native chat format.
#[derive(Debug, Serialize)]
pub(crate) struct OllamaChatMessage {
    pub role: String,
    pub content: String,
}

/// Ollama generation options.
#[derive(Debug, Serialize)]
pub(crate) struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
}

/// Ollama native `/api/chat` response format.
#[derive(Debug, Deserialize)]
pub(crate) struct OllamaChatResponse {
    pub message: OllamaChatResponseMessage,
    #[serde(default)]
    #[allow(dead_code)]
    pub done: bool,
}

/// Message in ollama native chat response.
#[derive(Debug, Deserialize)]
pub(crate) struct OllamaChatResponseMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: String,
    /// Thinking/reasoning content from thinking models (qwen3.5, deepseek-r1)
    /// When content is empty, the actual response may be in thinking
    #[serde(default)]
    pub thinking: Option<String>,
}
