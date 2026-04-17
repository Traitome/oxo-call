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
            let (system_prompt, user_prompt) = self.build_prompts(tool, question).await?;

            if self.verbose {
                eprintln!("{}", "─".repeat(60).dimmed());
                eprintln!("{}: {}", "Tool".cyan().bold(), tool);
                eprintln!("{}: {}", "Scenario".cyan().bold(), self.scenario_name());
                eprintln!("{}", "─".repeat(60).dimmed());
            }

            let response = self.call_api(&system_prompt, &user_prompt).await?;

            if json {
                let result = serde_json::json!({
                    "tool": tool,
                    "question": question,
                    "scenario": self.scenario_name(),
                    "response": response,
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("{}", response);
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
                println!("{} {}", "Tool context:".cyan().bold(), tool);
            }
            println!();

            loop {
                let prompt = if let Some(ref tool) = current_tool {
                    format!("{}> ", tool.cyan())
                } else {
                    "oxo> ".to_string()
                };

                print!("{}", prompt);
                io::stdout().flush()?;

                let mut input = String::new();
                let stdin = io::stdin();
                match stdin.lock().read_line(&mut input) {
                    Ok(0) => {
                        println!("\n{}", "Goodbye!".green());
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{} Failed to read input: {}", "error:".red(), e);
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
                        let (system_prompt, user_prompt) =
                            self.build_prompts(&t, &question).await?;

                        self.conversation_history.push(ChatMessage {
                            role: "user".to_string(),
                            content: user_prompt.clone(),
                        });

                        let response = self.call_api_with_history(&system_prompt).await?;

                        println!("\n{}\n", response);

                        self.conversation_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: response,
                        });
                    }
                    None => {
                        println!(
                            "{}",
                            "Please specify a tool or use /tool <name> to set context.".yellow()
                        );
                        println!("{}", "Example: samtools How do I sort a BAM file?".dimmed());
                    }
                }
            }

            Ok(())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn print_welcome(&self) {
        println!(
            "{}",
            "╔════════════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{} {} {}",
            "║".cyan(),
            "oxo-call Interactive Chat".white().bold(),
            "                            ║".cyan()
        );
        println!(
            "{}",
            "╚════════════════════════════════════════════════════════════╝".cyan()
        );
        println!();
        println!("{}", "Commands:".bold());
        println!("  /tool <name>    Set tool context for subsequent questions");
        println!("  /clear          Clear conversation history");
        println!("  /scenario <s>   Change scenario (bare|prompt|skill|doc|full)");
        println!("  /help           Show this help message");
        println!("  /quit, Ctrl+D   Exit the chat");
        println!();
        println!("{}", "Usage:".bold());
        println!("  <tool> <question>    Ask about a specific tool");
        println!("  <question>           Ask about the current tool (if set)");
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
                println!("{}", "Goodbye!".green());
                std::process::exit(0);
            }
            "/help" | "/?" => {
                self.print_welcome();
                true
            }
            "/clear" => {
                self.conversation_history.clear();
                println!("{}", "Conversation history cleared.".green());
                true
            }
            "/tool" => {
                if parts.len() < 2 {
                    println!("{}", "Usage: /tool <name>".yellow());
                    if let Some(ref t) = *current_tool {
                        println!("Current tool: {}", t.cyan());
                    }
                } else {
                    *current_tool = Some(parts[1].to_string());
                    println!("{} {}", "Tool context set to:".green(), parts[1].cyan());
                }
                true
            }
            "/scenario" => {
                if parts.len() < 2 {
                    println!(
                        "{}",
                        "Usage: /scenario (bare|prompt|skill|doc|full)".yellow()
                    );
                    println!("Current scenario: {}", self.scenario_name().cyan());
                } else {
                    match parts[1] {
                        "bare" => {
                            self.scenario = ChatScenario::Bare;
                            println!("{}", "Scenario changed to: bare".green());
                        }
                        "prompt" => {
                            self.scenario = ChatScenario::Prompt;
                            println!("{}", "Scenario changed to: prompt".green());
                        }
                        "skill" => {
                            self.scenario = ChatScenario::Skill;
                            println!("{}", "Scenario changed to: skill".green());
                        }
                        "doc" => {
                            self.scenario = ChatScenario::Doc;
                            println!("{}", "Scenario changed to: doc".green());
                        }
                        "full" => {
                            self.scenario = ChatScenario::Full;
                            println!("{}", "Scenario changed to: full".green());
                        }
                        _ => {
                            println!(
                                "{}",
                                "Invalid scenario. Use: bare, prompt, skill, doc, full".red()
                            );
                        }
                    }
                }
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
