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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_command_suggestion_fields() {
        let s = LlmCommandSuggestion {
            args: vec!["-o".to_string(), "out.bam".to_string()],
            explanation: "Sort the BAM file".to_string(),
            inference_ms: 123.4,
        };
        assert_eq!(s.args.len(), 2);
        assert_eq!(s.explanation, "Sort the BAM file");
        assert!((s.inference_ms - 123.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_llm_command_suggestion_zero_inference() {
        let s = LlmCommandSuggestion {
            args: vec![],
            explanation: String::new(),
            inference_ms: 0.0,
        };
        assert!(s.args.is_empty());
        assert!(s.explanation.is_empty());
        assert_eq!(s.inference_ms, 0.0);
    }

    #[test]
    fn test_llm_run_verification_success() {
        let v = LlmRunVerification {
            success: true,
            summary: "All good".to_string(),
            issues: vec![],
            suggestions: vec![],
        };
        assert!(v.success);
        assert!(v.issues.is_empty());
    }

    #[test]
    fn test_llm_run_verification_failure() {
        let v = LlmRunVerification {
            success: false,
            summary: "Command failed".to_string(),
            issues: vec!["Missing file".to_string()],
            suggestions: vec!["Check path".to_string()],
        };
        assert!(!v.success);
        assert_eq!(v.issues.len(), 1);
        assert_eq!(v.suggestions.len(), 1);
    }

    #[test]
    fn test_llm_skill_verification_passed() {
        let v = LlmSkillVerification {
            passed: true,
            summary: "Skill looks good".to_string(),
            issues: vec![],
            suggestions: vec![],
        };
        assert!(v.passed);
        assert!(v.issues.is_empty());
    }

    #[test]
    fn test_llm_skill_verification_failed() {
        let v = LlmSkillVerification {
            passed: false,
            summary: "Needs improvement".to_string(),
            issues: vec!["Missing examples".to_string(), "No pitfalls".to_string()],
            suggestions: vec!["Add 3+ examples".to_string()],
        };
        assert!(!v.passed);
        assert_eq!(v.issues.len(), 2);
    }

    #[test]
    fn test_prompt_tier_variants() {
        assert_eq!(PromptTier::Full, PromptTier::Full);
        assert_eq!(PromptTier::Medium, PromptTier::Medium);
        assert_eq!(PromptTier::Compact, PromptTier::Compact);
        assert_ne!(PromptTier::Full, PromptTier::Compact);
    }

    #[test]
    fn test_chat_message_serialize() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            reasoning: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"user\""));
        assert!(json.contains("\"Hello\""));
    }

    #[test]
    fn test_chat_message_with_reasoning() {
        let msg = ChatMessage {
            role: "assistant".to_string(),
            content: "The answer is 42".to_string(),
            reasoning: Some("I thought about it...".to_string()),
        };
        assert_eq!(msg.reasoning, Some("I thought about it...".to_string()));
    }

    #[test]
    fn test_chat_request_serialize() {
        let req = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
                reasoning: None,
            }],
            max_tokens: 512,
            temperature: 0.2,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"gpt-4\""));
        assert!(json.contains("512"));
    }

    #[test]
    fn test_chat_request_streaming_has_stream_field() {
        let req = ChatRequestStreaming {
            model: "gpt-4".to_string(),
            messages: vec![],
            max_tokens: 256,
            temperature: 0.0,
            stream: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"stream\":true"));
    }

    #[test]
    fn test_ollama_chat_request_serialize() {
        let req = OllamaChatRequest {
            model: "qwen2.5-coder:7b".to_string(),
            messages: vec![OllamaChatMessage {
                role: "user".to_string(),
                content: "sort a bam file".to_string(),
            }],
            stream: false,
            options: None,
            think: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("qwen2.5-coder:7b"));
        assert!(json.contains("\"stream\":false"));
    }

    #[test]
    fn test_ollama_options_skip_none_fields() {
        let opts = OllamaOptions {
            num_ctx: Some(4096),
            temperature: None,
            num_predict: None,
        };
        let json = serde_json::to_string(&opts).unwrap();
        assert!(json.contains("\"num_ctx\":4096"));
        assert!(!json.contains("temperature"));
    }

    #[test]
    fn test_stream_delta_deserialize_content() {
        let json = r#"{"content":"hello"}"#;
        let delta: StreamDelta = serde_json::from_str(json).unwrap();
        assert_eq!(delta.content, Some("hello".to_string()));
        assert!(delta.reasoning.is_none());
    }

    #[test]
    fn test_stream_delta_deserialize_reasoning() {
        let json = r#"{"reasoning":"thinking..."}"#;
        let delta: StreamDelta = serde_json::from_str(json).unwrap();
        assert!(delta.content.is_none());
        assert_eq!(delta.reasoning, Some("thinking...".to_string()));
    }

    #[test]
    fn test_llm_verification_result_fields() {
        let r = LlmVerificationResult {
            provider: "openai".to_string(),
            api_base: "https://api.openai.com".to_string(),
            model: "gpt-4".to_string(),
            response_preview: "ARGS: -o out.bam".to_string(),
        };
        assert_eq!(r.provider, "openai");
        assert_eq!(r.model, "gpt-4");
    }
}
