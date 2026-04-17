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
use crate::llm::types::{ChatMessage, ChatRequest, ChatResponse};
use crate::skill::SkillManager;
use colored::Colorize;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, BufRead, Write};

/// Render markdown text to the terminal using termimad.
///
/// Falls back to plain text if rendering fails.
#[cfg(not(target_arch = "wasm32"))]
fn render_markdown(text: &str) {
    use termimad::MadSkin;
    let skin = MadSkin::default();
    // print_text writes directly to stdout with ANSI styling
    skin.print_text(text);
}

pub struct ChatSession {
    config: Config,
    fetcher: DocsFetcher,
    skill_manager: SkillManager,
    verbose: bool,
    no_cache: bool,
    scenario: ChatScenario,
    #[cfg(not(target_arch = "wasm32"))]
    client: reqwest::Client,
    #[cfg(not(target_arch = "wasm32"))]
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
            #[cfg(not(target_arch = "wasm32"))]
            client: reqwest::Client::new(),
            #[cfg(not(target_arch = "wasm32"))]
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
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn run_single(&self, tool: &str, question: &str, json: bool) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "Chat is not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
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

            let spinner =
                crate::runner::make_spinner(&format!("Preparing context for '{tool}'..."));

            let prompts_result = self.build_prompts(tool, question).await;

            spinner.finish_and_clear();

            let (system_prompt, user_prompt) = prompts_result?;

            let spinner = crate::runner::make_spinner("Waiting for LLM response...");

            let api_result = self.call_api(&system_prompt, &user_prompt).await;

            spinner.finish_and_clear();

            let response = api_result?;

            if json {
                let result = serde_json::json!({
                    "tool": tool,
                    "question": question,
                    "scenario": self.scenario_name(),
                    "response": response,
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!();
                render_markdown(&response);
                println!();
            }

            Ok(())
        }
    }

    /// Run interactive multi-turn chat session.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub async fn run_interactive(&mut self, initial_tool: Option<&str>) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::LlmError(
            "Interactive chat is not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
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

            loop {
                let prompt = if let Some(ref tool) = current_tool {
                    format!("{} {} ", "▶".green(), tool.cyan().bold())
                } else {
                    format!("{} ", "oxo▶".cyan().bold())
                };

                print!("{}", prompt);
                io::stdout().flush()?;

                let mut input = String::new();
                let stdin = io::stdin();
                match stdin.lock().read_line(&mut input) {
                    Ok(0) => {
                        println!("\n{}", "👋 Goodbye!".green().bold());
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{} Failed to read input: {}", "✖ error:".red().bold(), e);
                        continue;
                    }
                }

                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                if self.handle_command(input, &mut current_tool) {
                    continue;
                }

                let (tool, question) = self.parse_input(input, current_tool.as_deref());

                match tool {
                    Some(t) => {
                        let spinner =
                            crate::runner::make_spinner(&format!("Loading context for '{t}'..."));

                        let prompts_result = self.build_prompts(&t, &question).await;

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
                        });

                        let spinner = crate::runner::make_spinner("Thinking...");

                        let api_result = self.call_api_with_history(&system_prompt).await;

                        spinner.finish_and_clear();

                        match api_result {
                            Ok(response) => {
                                println!();
                                println!("{}", "─".repeat(60).dimmed());
                                render_markdown(&response);
                                println!("{}", "─".repeat(60).dimmed());
                                println!();

                                self.conversation_history.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: response,
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
                        println!(
                            "  {}",
                            "⚠ Please specify a tool or use /tool <name> to set context.".yellow()
                        );
                        println!(
                            "  {}",
                            "Example: samtools How do I sort a BAM file?".dimmed()
                        );
                    }
                }
            }

            Ok(())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn print_welcome(&self) {
        println!();
        println!(
            "  {}",
            "╔══════════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "  {} {} {}",
            "║".cyan(),
            "🧬 oxo-call Interactive Chat".white().bold(),
            "                         ║".cyan()
        );
        println!(
            "  {}",
            "╚══════════════════════════════════════════════════════════╝".cyan()
        );
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
            "    {}    Ask about a specific tool",
            "<tool> <question>".white()
        );
        println!(
            "    {}           Ask about the current tool (if set)",
            "<question>".white()
        );
        println!();
    }

    #[cfg(not(target_arch = "wasm32"))]
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
                let pairs = count / 2;
                println!(
                    "  {} {} messages ({} exchanges)",
                    "📜".dimmed(),
                    count,
                    pairs,
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
                        "prompt" => {
                            self.scenario = ChatScenario::Prompt;
                            println!("{}", "  ✔ Scenario changed to: prompt".green());
                        }
                        "skill" => {
                            self.scenario = ChatScenario::Skill;
                            println!("{}", "  ✔ Scenario changed to: skill".green());
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
                                "✖ Invalid scenario. Use: bare, prompt, skill, doc, full".red()
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

    #[cfg(not(target_arch = "wasm32"))]
    fn parse_input(&self, input: &str, current_tool: Option<&str>) -> (Option<String>, String) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return (None, String::new());
        }

        if parts[0].starts_with('/') {
            return (None, String::new());
        }

        if let Some(tool) = current_tool {
            (Some(tool.to_string()), input.to_string())
        } else if parts.len() >= 2 {
            let tool = parts[0].to_string();
            let question = parts[1..].join(" ");
            (Some(tool), question)
        } else {
            (None, input.to_string())
        }
    }

    fn scenario_name(&self) -> &'static str {
        match self.scenario {
            ChatScenario::Bare => "bare",
            ChatScenario::Prompt => "prompt",
            ChatScenario::Skill => "skill",
            ChatScenario::Doc => "doc",
            ChatScenario::Full => "full",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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
            ChatScenario::Prompt | ChatScenario::Skill | ChatScenario::Doc | ChatScenario::Full => {
                "You are a helpful bioinformatics assistant. Answer questions about bioinformatics tools, \
                 workflows, and concepts clearly and accurately. When discussing specific tools, \
                 reference their documentation and common usage patterns. Be concise but thorough."
                    .to_string()
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn build_context(&self, tool: &str) -> Result<String> {
        let mut context_parts = Vec::new();

        match self.scenario {
            ChatScenario::Bare | ChatScenario::Prompt => {}
            ChatScenario::Skill => {
                if let Some(skill) = self.skill_manager.load(tool) {
                    context_parts
                        .push(format!("## Skill Knowledge\n{}", skill.to_prompt_section()));
                }
            }
            ChatScenario::Doc => {
                let docs = self.fetcher.fetch(tool).await?;
                if let Some(help) = docs.help_output {
                    let truncated = if help.len() > 4000 {
                        format!("{}...\n[Documentation truncated]", &help[..4000])
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
                        format!("{}...\n[Documentation truncated]", &help[..4000])
                    } else {
                        help
                    };
                    context_parts.push(format!("## Tool Documentation\n{}", truncated));
                }
            }
        }

        Ok(context_parts.join("\n\n"))
    }

    #[cfg(not(target_arch = "wasm32"))]
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
            });
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        });

        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens: self.config.effective_max_tokens()?,
            temperature: 0.7,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        let auth_token = if provider == "github-copilot" {
            let manager = crate::copilot_auth::get_token_manager();
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

    #[cfg(not(target_arch = "wasm32"))]
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
            });
        }
        messages.extend(self.conversation_history.clone());

        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens: self.config.effective_max_tokens()?,
            temperature: 0.7,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        let auth_token = if provider == "github-copilot" {
            let manager = crate::copilot_auth::get_token_manager();
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

        let session = session.with_scenario(ChatScenario::Prompt);
        assert_eq!(session.scenario_name(), "prompt");

        let session = session.with_scenario(ChatScenario::Skill);
        assert_eq!(session.scenario_name(), "skill");

        let session = session.with_scenario(ChatScenario::Doc);
        assert_eq!(session.scenario_name(), "doc");

        let session = session.with_scenario(ChatScenario::Full);
        assert_eq!(session.scenario_name(), "full");
    }

    #[test]
    fn test_parse_input_with_tool_context() {
        let config = Config::default();
        let session = ChatSession::new(config);

        let (tool, question) = session.parse_input("How do I sort?", Some("samtools"));
        assert_eq!(tool, Some("samtools".to_string()));
        assert_eq!(question, "How do I sort?");
    }

    #[test]
    fn test_parse_input_without_tool_context() {
        let config = Config::default();
        let session = ChatSession::new(config);

        let (tool, question) = session.parse_input("samtools How do I sort?", None);
        assert_eq!(tool, Some("samtools".to_string()));
        assert_eq!(question, "How do I sort?");
    }

    #[test]
    fn test_parse_input_single_word_no_context() {
        let config = Config::default();
        let session = ChatSession::new(config);

        let (tool, question) = session.parse_input("hello", None);
        assert_eq!(tool, None);
        assert_eq!(question, "hello");
    }

    #[test]
    fn test_parse_input_slash_command_returns_none() {
        let config = Config::default();
        let session = ChatSession::new(config);

        let (tool, question) = session.parse_input("/help", None);
        assert_eq!(tool, None);
        assert_eq!(question, "");
    }

    #[test]
    fn test_parse_input_empty_string() {
        let config = Config::default();
        let session = ChatSession::new(config);

        let (tool, question) = session.parse_input("", None);
        assert_eq!(tool, None);
        assert_eq!(question, "");
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

        let s1 = ChatSession::new(config.clone()).with_scenario(ChatScenario::Prompt);
        assert!(!s1.build_system_prompt().is_empty());

        let s2 = ChatSession::new(config.clone()).with_scenario(ChatScenario::Skill);
        assert!(!s2.build_system_prompt().is_empty());

        let s3 = ChatSession::new(config.clone()).with_scenario(ChatScenario::Doc);
        assert!(!s3.build_system_prompt().is_empty());

        let s4 = ChatSession::new(config).with_scenario(ChatScenario::Full);
        assert!(!s4.build_system_prompt().is_empty());
    }

    #[test]
    fn test_handle_command_clear() {
        let config = Config::default();
        let mut session = ChatSession::new(config);
        session.conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
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
        });
        session.conversation_history.push(ChatMessage {
            role: "assistant".to_string(),
            content: "reply".to_string(),
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
        render_markdown("# Heading\n\nSome **bold** and *italic* text.");
        render_markdown("```bash\nsamtools sort input.bam\n```");
        render_markdown("");
        render_markdown("Plain text with no markdown.");
        render_markdown("- item 1\n- item 2\n- item 3");
    }
}
