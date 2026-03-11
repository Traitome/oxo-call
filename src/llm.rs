use crate::config::Config;
use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};

/// A parsed LLM response with command arguments and explanation
#[derive(Debug, Clone)]
pub struct LlmCommandSuggestion {
    pub args: Vec<String>,
    pub explanation: String,
    #[allow(dead_code)]
    pub raw_response: String,
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

/// Shared system prompt for bioinformatics tool orchestration
fn system_prompt() -> &'static str {
    "You are an expert bioinformatics tool orchestrator. \
     Your job is to translate a user's natural-language task description into \
     the exact command-line arguments for a specific bioinformatics tool. \
     You are precise, concise, and only suggest arguments that are explicitly \
     supported by the provided documentation. \
     When generating arguments, use the most appropriate flags and values based \
     on the task description. \
     If input files, output files, or other required parameters are mentioned in \
     the task description but not in the documentation, include them verbatim. \
     Never hallucinate flags or options that are not in the documentation."
}

/// Build the user prompt for command generation
fn build_prompt(tool: &str, documentation: &str, task: &str) -> String {
    format!(
        r#"## Tool: {tool}

## Documentation:
{documentation}

## Task Description:
{task}

## Instructions:
Based on the documentation above, generate the command-line arguments for `{tool}` to accomplish the task.

Respond in EXACTLY this format (no other text):
ARGS: <arguments for {tool}, space-separated, without the tool name itself>
EXPLANATION: <one or two sentences explaining what these arguments do and why they were chosen>

If the task cannot be accomplished with the available flags, explain why in the EXPLANATION field and leave ARGS empty.
If input/output files are mentioned in the task but not reflected in flags, include them as positional arguments."#
    )
}

pub struct LlmClient {
    config: Config,
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        LlmClient {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Generate command arguments for a tool given its documentation and task description
    pub async fn suggest_command(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
    ) -> Result<LlmCommandSuggestion> {
        let token = self
            .config
            .effective_api_token()
            .ok_or_else(|| OxoError::LlmError(
                "No API token configured. Set it with:\n  oxo-call config set llm.api_token <token>\n\
                    Or set the environment variable GITHUB_TOKEN (for GitHub Copilot), \
                    OPENAI_API_KEY (for OpenAI), or ANTHROPIC_API_KEY (for Anthropic).".to_string()
            ))?;

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
                content: build_prompt(tool, documentation, task),
            },
        ];

        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens: self.config.llm.max_tokens,
            temperature: self.config.llm.temperature,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Set authorization header based on provider
        req_builder = match self.config.llm.provider.as_str() {
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

        let raw_response = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Self::parse_response(&raw_response)
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

        // Parse args string into a Vec<String>, respecting quoted strings
        let args = parse_shell_args(&args_line);

        Ok(LlmCommandSuggestion {
            args,
            explanation: explanation_line,
            raw_response: raw.to_string(),
        })
    }
}

/// Simple shell-like argument parser that handles quoted strings
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
