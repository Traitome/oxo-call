//! Interactive chat with AI about bioinformatics tools.
//!
//! This module provides the `chat` subcommand functionality:
//! - Single-shot Q&A: `oxo-call chat <tool> <question>`
//! - Interactive multi-turn chat: `oxo-call chat -i`
//!
//! Scenarios control what context is injected:
//! - `bare`: Plain chat (no prompt/docs/skill)
//! - `prompt`: oxo-call system prompt only
//! - `skill`: Load skill file only
//! - `doc`: Load tool documentation only
//! - `full`: Load everything (default)

use crate::cli::ChatScenario;
use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::llm::streaming::apply_provider_auth_headers;
use crate::llm::types::{ChatMessage, ChatRequest, ChatRequestStreaming, ChatResponse};
use crate::markdown;
use crate::skill::SkillManager;
use crate::streaming_display;
use colored::Colorize;

pub struct ChatSession {
    config: Config,
    fetcher: DocsFetcher,
    skill_manager: SkillManager,
    verbose: bool,
    no_cache: bool,
    scenario: ChatScenario,
    client: reqwest::Client,
    conversation_history: Vec<ChatMessage>,
}

impl ChatSession {
    pub fn new(config: Config) -> Self {
        ChatSession {
            fetcher: DocsFetcher::new(config.clone()),
            skill_manager: SkillManager::new(config.clone()),
            config,
            verbose: false,
            no_cache: false,
            scenario: ChatScenario::Full,
            client: reqwest::Client::new(),
            conversation_history: Vec::new(),
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_no_cache(mut self, no_cache: bool) -> Self {
        self.no_cache = no_cache;
        self
    }

    pub fn with_scenario(mut self, scenario: ChatScenario) -> Self {
        self.scenario = scenario;
        self
    }

    /// Run single-shot Q&A (non-interactive mode).
    pub async fn run_single(&self, tool: &str, question: &str, json: bool) -> Result<()> {
        if self.verbose {
            eprintln!("{}", "─".repeat(60).dimmed());
            eprintln!("{}: {}", "Tool".cyan().bold(), tool);
            eprintln!("{}: {}", "Scenario".cyan().bold(), self.scenario_name());
            eprintln!(
                "{}: {}",
                "Model".cyan().bold(),
                self.config.effective_model()
            );
            eprintln!("{}", "─".repeat(60).dimmed());
        }

        let spinner = crate::runner::make_spinner(&format!("Preparing context for '{tool}'..."));

        let prompts_result = self.build_prompts(tool, question).await;

        spinner.finish_and_clear();

        let (system_prompt, user_prompt) = prompts_result?;

        // StreamingDisplay handles spinner + preview internally.
        // Non-streaming mode uses a simple spinner.
        let api_result = self.call_api(&system_prompt, &user_prompt).await;

        let response = api_result?;

        if json {
            let result = serde_json::json!({
                "tool": tool,
                "question": question,
                "scenario": self.scenario_name(),
                "response": response,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("JSON serialization")
            );
        } else {
            println!();
            markdown::render_markdown(&response);
            println!();
        }

        Ok(())
    }

    /// Run interactive multi-turn chat session.
    pub async fn run_interactive(&mut self, initial_tool: Option<&str>) -> Result<()> {
        let mut current_tool = initial_tool.map(String::from);

        self.print_welcome();

        if let Some(tool) = &current_tool {
            println!(
                "  {} {}",
                "🔧 Tool context:".cyan().bold(),
                tool.white().bold()
            );
        }
        println!(
            "  {} {}",
            "📋 Scenario:".dimmed(),
            self.scenario_name().dimmed()
        );
        println!(
            "  {} {}",
            "🤖 Model:".dimmed(),
            self.config.effective_model().dimmed()
        );
        println!();

        // Set up rustyline editor for proper terminal line editing
        let mut rl = rustyline::DefaultEditor::new()
            .map_err(|e| OxoError::LlmError(format!("Failed to init line editor: {e}")))?;

        loop {
            let prompt = if let Some(ref tool) = current_tool {
                format!("{} {} ", "▶", tool)
            } else {
                "oxo▶ ".to_string()
            };

            let readline = rl.readline(&prompt);
            match readline {
                Ok(line) => {
                    let input = line.trim().to_string();
                    if input.is_empty() {
                        continue;
                    }
                    rl.add_history_entry(&input).ok();

                    if self.handle_command(&input, &mut current_tool) {
                        continue;
                    }

                    match current_tool.as_deref() {
                        Some(t) => {
                            // Tool context is set: answer with tool-specific knowledge.
                            let spinner = crate::runner::make_spinner(&format!(
                                "Loading context for '{t}'..."
                            ));

                            let prompts_result = self.build_prompts(t, &input).await;

                            spinner.finish_and_clear();

                            let (system_prompt, user_prompt) = match prompts_result {
                                Ok(p) => p,
                                Err(e) => {
                                    eprintln!("  {} {}", "✖ Context error:".red().bold(), e);
                                    continue;
                                }
                            };

                            self.conversation_history.push(ChatMessage {
                                role: "user".to_string(),
                                content: user_prompt.clone(),
                                reasoning: None,
                            });

                            // StreamingDisplay handles spinner + preview internally
                            let api_result = self.call_api_with_history(&system_prompt).await;

                            match api_result {
                                Ok(response) => {
                                    println!();
                                    println!("{}", "─".repeat(60).dimmed());
                                    markdown::render_markdown(&response);
                                    println!("{}", "─".repeat(60).dimmed());
                                    println!();

                                    self.conversation_history.push(ChatMessage {
                                        role: "assistant".to_string(),
                                        content: response,
                                        reasoning: None,
                                    });
                                }
                                Err(e) => {
                                    // Remove the user message we just added since the
                                    // API call failed.
                                    self.conversation_history.pop();
                                    eprintln!("\n  {} {}\n", "✖ LLM error:".red().bold(), e);
                                }
                            }
                        }
                        None => {
                            // No tool context: answer as a general-purpose assistant.
                            let system_prompt = self.build_general_system_prompt();

                            self.conversation_history.push(ChatMessage {
                                role: "user".to_string(),
                                content: input.clone(),
                                reasoning: None,
                            });

                            // StreamingDisplay handles spinner + preview internally
                            let api_result = self.call_api_with_history(&system_prompt).await;

                            match api_result {
                                Ok(response) => {
                                    println!();
                                    println!("{}", "─".repeat(60).dimmed());
                                    markdown::render_markdown(&response);
                                    println!("{}", "─".repeat(60).dimmed());
                                    println!();

                                    self.conversation_history.push(ChatMessage {
                                        role: "assistant".to_string(),
                                        content: response,
                                        reasoning: None,
                                    });
                                }
                                Err(e) => {
                                    // Remove the user message we just added since the
                                    // API call failed.
                                    self.conversation_history.pop();
                                    eprintln!("\n  {} {}\n", "✖ LLM error:".red().bold(), e);
                                }
                            }
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("\n{}", "👋 Goodbye!".green().bold());
                    break;
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("{}", " (Ctrl+C)".dimmed());
                    continue;
                }
                Err(e) => {
                    eprintln!("{} Failed to read input: {}", "✖ error:".red().bold(), e);
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Run single-shot general Q&A without a specific tool context.
    pub async fn run_single_general(&self, question: &str, json: bool) -> Result<()> {
        if self.verbose {
            eprintln!("{}", "─".repeat(60).dimmed());
            eprintln!("{}: general", "Mode".cyan().bold());
            eprintln!("{}: {}", "Scenario".cyan().bold(), self.scenario_name());
            eprintln!(
                "{}: {}",
                "Model".cyan().bold(),
                self.config.effective_model()
            );
            eprintln!("{}", "─".repeat(60).dimmed());
        }

        let spinner = crate::runner::make_spinner("Thinking...");

        let system_prompt = self.build_general_system_prompt();
        let api_result = self.call_api(&system_prompt, question).await;

        spinner.finish_and_clear();

        let response = api_result?;

        if json {
            let result = serde_json::json!({
                "question": question,
                "scenario": self.scenario_name(),
                "response": response,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("JSON serialization")
            );
        } else {
            println!();
            markdown::render_markdown(&response);
            println!();
        }

        Ok(())
    }

    fn print_welcome(&self) {
        println!();
        // Title bar: 60 chars wide (inside borders), centered
        let title = "🧬 oxo-call Interactive Chat";
        let inner_width = 58; // 60 - 2 border chars
        // Use terminal display width (emoji like 🧬 takes 2 columns)
        let title_display_width = unicode_width::UnicodeWidthStr::width(title);
        let left_pad = (inner_width - title_display_width) / 2;
        let right_pad = inner_width - title_display_width - left_pad;
        let top_bottom = "═".repeat(inner_width);
        println!("  {}{}{}", "╔".cyan(), top_bottom.cyan(), "╗".cyan());
        println!(
            "  {}{}{}{}{}",
            "║".cyan(),
            " ".repeat(left_pad),
            title.white().bold(),
            " ".repeat(right_pad),
            "║".cyan()
        );
        println!("  {}{}{}", "╚".cyan(), top_bottom.cyan(), "╝".cyan());
        println!();
        println!("  {}", "Commands:".bold().underline());
        println!(
            "    {}  Set tool context for subsequent questions",
            "/tool <name>".cyan()
        );
        println!("    {}  Clear conversation history", "/clear".cyan());
        println!("    {}  Show conversation message count", "/history".cyan());
        println!(
            "    {}  Display or change the current LLM model",
            "/model [name]".cyan()
        );
        println!(
            "    {}  Change scenario (bare|prompt|skill|doc|full)",
            "/scenario <s>".cyan()
        );
        println!("    {}  Show this help message", "/help".cyan());
        println!("    {}  Exit the chat", "/quit, Ctrl+D".cyan());
        println!();
        println!("  {}", "Usage:".bold().underline());
        println!(
            "    {}  Ask any question (general mode, no tool context required)",
            "<question>".white()
        );
        println!(
            "    {}  Set a tool context, then ask tool-specific questions",
            "/tool <name>".white()
        );
        println!();
    }

    fn handle_command(&mut self, input: &str, current_tool: &mut Option<String>) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return true;
        }

        match parts[0] {
            "/quit" | "/exit" => {
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
            "/help" | "/?" => {
                self.print_welcome();
                true
            }
            "/clear" => {
                self.conversation_history.clear();
                println!(
                    "  {} {}",
                    "✔".green(),
                    "Conversation history cleared.".green()
                );
                true
            }
            "/history" => {
                let count = self.conversation_history.len();
                let approx_exchanges = count / 2;
                println!(
                    "  {} {} messages (~{} exchanges)",
                    "📜".dimmed(),
                    count,
                    approx_exchanges,
                );
                true
            }
            "/model" => {
                if parts.len() < 2 {
                    println!(
                        "  {} {}",
                        "🤖 Current model:".cyan().bold(),
                        self.config.effective_model().white().bold()
                    );
                } else {
                    self.config.llm.model = Some(parts[1].to_string());
                    println!(
                        "  {} {}",
                        "✔ Model changed to:".green(),
                        parts[1].cyan().bold()
                    );
                }
                true
            }
            "/tool" => {
                if parts.len() < 2 {
                    println!("{}", "  ⚠ Usage: /tool <name>".yellow());
                    if let Some(ref t) = *current_tool {
                        println!("  🔧 Current tool: {}", t.cyan().bold());
                    } else {
                        println!("  {}", "(no tool set)".dimmed());
                    }
                } else {
                    *current_tool = Some(parts[1].to_string());
                    println!(
                        "  {} {}",
                        "✔ Tool context set to:".green(),
                        parts[1].cyan().bold()
                    );
                }
                true
            }
            "/scenario" => {
                if parts.len() < 2 {
                    println!(
                        "{}",
                        "  ⚠ Usage: /scenario (bare|prompt|skill|doc|full)".yellow()
                    );
                    println!(
                        "  📋 Current scenario: {}",
                        self.scenario_name().cyan().bold()
                    );
                } else {
                    match parts[1] {
                        "bare" => {
                            self.scenario = ChatScenario::Bare;
                            println!("{}", "  ✔ Scenario changed to: bare".green());
                        }
                        "doc" => {
                            self.scenario = ChatScenario::Doc;
                            println!("{}", "  ✔ Scenario changed to: doc".green());
                        }
                        "full" => {
                            self.scenario = ChatScenario::Full;
                            println!("{}", "  ✔ Scenario changed to: full".green());
                        }
                        _ => {
                            println!(
                                "  {}",
                                "✖ Invalid scenario. Use: bare, doc, full".red()
                            );
                        }
                    }
                }
                true
            }
            _ if parts[0].starts_with('/') => {
                println!(
                    "  {} Unknown command '{}'. Type {} for help.",
                    "⚠".yellow(),
                    parts[0].yellow(),
                    "/help".cyan()
                );
                true
            }
            _ => false,
        }
    }

    fn scenario_name(&self) -> &'static str {
        match self.scenario {
            ChatScenario::Bare => "bare",
            ChatScenario::Doc => "doc",
            ChatScenario::Full => "full",
        }
    }

    async fn build_prompts(&self, tool: &str, question: &str) -> Result<(String, String)> {
        let system_prompt = self.build_system_prompt();
        let context = self.build_context(tool).await?;
        let user_prompt = if context.is_empty() {
            format!("Tool: {}\n\nQuestion: {}", tool, question)
        } else {
            format!("Tool: {}\n\n{}\n\nQuestion: {}", tool, context, question)
        };

        Ok((system_prompt, user_prompt))
    }

    fn build_system_prompt(&self) -> String {
        match self.scenario {
            ChatScenario::Bare => String::new(),
            ChatScenario::Doc | ChatScenario::Full => {
                "Reply with ONLY:\n\
                 ARGS: sort -o output.bam input.bam\n\
                 EXPLANATION: Sorts BAM.\n\
                 Replace args. Maximum 20 words. No preamble."
                    .to_string()
            }
        }
    }

    /// System prompt used when there is no specific tool context (general conversation mode).
    fn build_general_system_prompt(&self) -> String {
        match self.scenario {
            ChatScenario::Bare => String::new(),
            _ => "Reply with ONLY 1-2 sentences. Maximum 20 words. No preamble.".to_string(),
        }
    }

    async fn build_context(&self, tool: &str) -> Result<String> {
        let mut context_parts = Vec::new();

        match self.scenario {
            ChatScenario::Bare => {}
            ChatScenario::Doc => {
                let docs = self.fetcher.fetch(tool).await?;
                if let Some(help) = docs.help_output {
                    let truncated = if help.len() > 4000 {
                        // Safe UTF-8 truncation: use chars() to avoid splitting multi-byte characters
                        let safe_end = help
                            .char_indices()
                            .nth(4000)
                            .map(|(i, _)| i)
                            .unwrap_or(help.len());
                        format!("{}...\n[Documentation truncated]", &help[..safe_end])
                    } else {
                        help
                    };
                    context_parts.push(format!("## Tool Documentation\n{}", truncated));
                }
            }
            ChatScenario::Full => {
                if let Some(skill) = self.skill_manager.load(tool) {
                    context_parts
                        .push(format!("## Skill Knowledge\n{}", skill.to_prompt_section()));
                }

                let docs = self.fetcher.fetch(tool).await?;
                if let Some(help) = docs.help_output {
                    let truncated = if help.len() > 4000 {
                        // Safe UTF-8 truncation: use chars() to avoid splitting multi-byte characters
                        let safe_end = help
                            .char_indices()
                            .nth(4000)
                            .map(|(i, _)| i)
                            .unwrap_or(help.len());
                        format!("{}...\n[Documentation truncated]", &help[..safe_end])
                    } else {
                        help
                    };
                    context_parts.push(format!("## Tool Documentation\n{}", truncated));
                }
            }
        }

        Ok(context_parts.join("\n\n"))
    }

    async fn call_api(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let provider = self.config.effective_provider();
        let token_opt = self.config.effective_api_token();
        let token = if self.config.provider_requires_token() {
            token_opt.ok_or_else(|| {
                OxoError::LlmError(
                    "No API token configured. Run: oxo-call config login".to_string(),
                )
            })?
        } else {
            token_opt.unwrap_or_default()
        };

        let api_base = self.config.effective_api_base();
        let model = self.config.effective_model();
        let url = format!("{api_base}/chat/completions");

        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
                reasoning: None,
            });
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
            reasoning: None,
        });

        let auth_token = Self::resolve_auth_token(&provider, &token).await?;
        let max_tokens = self.config.effective_max_tokens()?;

        // ── Streaming path ────────────────────────────────────────────
        if self.config.llm.stream {
            let request = ChatRequestStreaming {
                model: model.clone(),
                messages,
                max_tokens,
                temperature: 0.1,
                stream: true,
            };

            let req_builder = apply_provider_auth_headers(
                self.client
                    .post(&url)
                    .header("Content-Type", "application/json"),
                &provider,
                &auth_token,
            );
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
                    message: "Thinking".to_string(),
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
            temperature: 0.1,
        };

        let req_builder = apply_provider_auth_headers(
            self.client
                .post(&url)
                .header("Content-Type", "application/json"),
            &provider,
            &auth_token,
        );
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

    async fn call_api_with_history(&self, system_prompt: &str) -> Result<String> {
        let provider = self.config.effective_provider();
        let token_opt = self.config.effective_api_token();
        let token = if self.config.provider_requires_token() {
            token_opt.ok_or_else(|| {
                OxoError::LlmError(
                    "No API token configured. Run: oxo-call config login".to_string(),
                )
            })?
        } else {
            token_opt.unwrap_or_default()
        };

        let api_base = self.config.effective_api_base();
        let model = self.config.effective_model();
        let url = format!("{api_base}/chat/completions");

        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
                reasoning: None,
            });
        }
        messages.extend(self.conversation_history.clone());

        let auth_token = Self::resolve_auth_token(&provider, &token).await?;
        let max_tokens = self.config.effective_max_tokens()?;

        // ── Streaming path ────────────────────────────────────────────
        if self.config.llm.stream {
            let request = ChatRequestStreaming {
                model: model.clone(),
                messages,
                max_tokens,
                temperature: 0.1,
                stream: true,
            };

            let req_builder = apply_provider_auth_headers(
                self.client
                    .post(&url)
                    .header("Content-Type", "application/json"),
                &provider,
                &auth_token,
            );
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
                    message: "Thinking".to_string(),
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
            temperature: 0.1,
        };

        let req_builder = apply_provider_auth_headers(
            self.client
                .post(&url)
                .header("Content-Type", "application/json"),
            &provider,
            &auth_token,
        );
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

    /// Resolve the auth token for the given provider.
    async fn resolve_auth_token(provider: &str, token: &str) -> Result<String> {
        if provider == "github-copilot" {
            let manager = crate::copilot_auth::get_token_manager();
            manager.get_session_token(token).await
        } else {
            Ok(token.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_name_returns_correct_string() {
        let config = Config::default();
        let session = ChatSession::new(config);

        assert_eq!(session.scenario_name(), "full");

        let session = session.with_scenario(ChatScenario::Bare);
        assert_eq!(session.scenario_name(), "bare");

        let session = session.with_scenario(ChatScenario::Doc);
        assert_eq!(session.scenario_name(), "doc");

        let session = session.with_scenario(ChatScenario::Full);
        assert_eq!(session.scenario_name(), "full");
    }

    #[test]
    fn test_build_system_prompt_bare_is_empty() {
        let config = Config::default();
        let session = ChatSession::new(config).with_scenario(ChatScenario::Bare);
        assert!(session.build_system_prompt().is_empty());
    }

    #[test]
    fn test_build_system_prompt_non_bare_is_nonempty() {
        let config = Config::default();

        let s1 = ChatSession::new(config.clone()).with_scenario(ChatScenario::Doc);
        assert!(!s1.build_system_prompt().is_empty());

        let s2 = ChatSession::new(config).with_scenario(ChatScenario::Full);
        assert!(!s2.build_system_prompt().is_empty());
    }

    #[test]
    fn test_handle_command_clear() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        session.conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
            reasoning: None,
        });
        assert_eq!(session.conversation_history.len(), 1);

        let mut tool = None;
        let handled = session.handle_command("/clear", &mut tool);
        assert!(handled);
        assert!(session.conversation_history.is_empty());
    }

    #[test]
    fn test_handle_command_tool_set() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool: Option<String> = None;

        let handled = session.handle_command("/tool samtools", &mut tool);
        assert!(handled);
        assert_eq!(tool, Some("samtools".to_string()));
    }

    #[test]
    fn test_handle_command_tool_no_arg() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool: Option<String> = None;

        let handled = session.handle_command("/tool", &mut tool);
        assert!(handled);
        assert_eq!(tool, None);
    }

    #[test]
    fn test_handle_command_scenario_change() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        session.handle_command("/scenario bare", &mut tool);
        assert_eq!(session.scenario_name(), "bare");

        session.handle_command("/scenario prompt", &mut tool);
        assert_eq!(session.scenario_name(), "prompt");

        session.handle_command("/scenario skill", &mut tool);
        assert_eq!(session.scenario_name(), "skill");

        session.handle_command("/scenario doc", &mut tool);
        assert_eq!(session.scenario_name(), "doc");

        session.handle_command("/scenario full", &mut tool);
        assert_eq!(session.scenario_name(), "full");
    }

    #[test]
    fn test_handle_command_invalid_scenario() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        session.handle_command("/scenario invalid", &mut tool);
        assert_eq!(session.scenario_name(), "full"); // unchanged
    }

    #[test]
    fn test_handle_command_help() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        assert!(session.handle_command("/help", &mut tool));
        assert!(session.handle_command("/?", &mut tool));
    }

    #[test]
    fn test_handle_command_unknown_slash() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        assert!(session.handle_command("/unknown", &mut tool));
    }

    #[test]
    fn test_handle_command_non_command() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        assert!(!session.handle_command("samtools sort", &mut tool));
    }

    #[test]
    fn test_handle_command_history() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        assert!(session.handle_command("/history", &mut tool));

        session.conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: "test".to_string(),
            reasoning: None,
        });
        session.conversation_history.push(ChatMessage {
            role: "assistant".to_string(),
            content: "reply".to_string(),
            reasoning: None,
        });

        assert!(session.handle_command("/history", &mut tool));
    }

    #[test]
    fn test_handle_command_model_show() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        assert!(session.handle_command("/model", &mut tool));
    }

    #[test]
    fn test_handle_command_model_change() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        let mut tool = None;

        session.handle_command("/model gpt-4", &mut tool);
        assert_eq!(session.config.llm.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_with_verbose() {
        let config = Config::default();
        let session = ChatSession::new(config).with_verbose(true);
        assert!(session.verbose);
    }

    #[test]
    fn test_with_no_cache() {
        let config = Config::default();
        let session = ChatSession::new(config).with_no_cache(true);
        assert!(session.no_cache);
    }

    #[test]
    fn test_render_markdown_does_not_panic() {
        // Ensure the rendering function handles various markdown content
        markdown::render_markdown("# Heading\n\nSome **bold** and *italic* text.");
        markdown::render_markdown("```bash\nsamtools sort input.bam\n```");
        markdown::render_markdown("");
        markdown::render_markdown("Plain text with no markdown.");
        markdown::render_markdown("- item 1\n- item 2\n- item 3");
    }

    #[test]
    fn test_build_general_system_prompt_bare_is_empty() {
        let config = Config::default();
        let session = ChatSession::new(config).with_scenario(ChatScenario::Bare);
        assert!(session.build_general_system_prompt().is_empty());
    }

    #[test]
    fn test_build_general_system_prompt_non_bare_is_nonempty() {
        let config = Config::default();

        for scenario in [
            ChatScenario::Doc,
            ChatScenario::Full,
        ] {
            let session = ChatSession::new(config.clone()).with_scenario(scenario);
            let prompt = session.build_general_system_prompt();
            assert!(
                !prompt.is_empty(),
                "General system prompt should not be empty for scenario {:?}",
                session.scenario_name()
            );
            // Should mention brevity
            assert!(
                prompt.contains("sentence") || prompt.contains("word"),
                "General prompt should mention brevity: {prompt}"
            );
        }
    }
}
