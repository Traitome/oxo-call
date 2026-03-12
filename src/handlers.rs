//! Extracted command-handler helpers for the CLI dispatch in `main.rs`.
//!
//! These functions are pure display / formatting helpers that were previously
//! inlined in `main.rs`.  Moving them here keeps the dispatcher focused on
//! control flow while putting presentation logic in a dedicated module.

use crate::config;
use crate::index;
use colored::Colorize;

/// Format a configuration value together with its source attribution.
pub fn with_source(value: &str, source: &str) -> String {
    format!("{value} [{}]", source.dimmed())
}

/// Print a formatted table of documentation index entries.
pub fn print_index_table(entries: &[index::IndexEntry]) {
    println!(
        "{:<20} {:<15} {:<12} {}",
        "Tool".bold(),
        "Version".bold(),
        "Size".bold(),
        "Indexed At".bold()
    );
    println!("{}", "─".repeat(70).dimmed());
    for e in entries {
        println!(
            "{:<20} {:<15} {:<12} {}",
            e.tool_name.cyan(),
            e.version.as_deref().unwrap_or("-"),
            format!("{} B", e.doc_size_bytes),
            e.indexed_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }
}

/// Generate context-specific troubleshooting suggestions based on a configuration
/// verification error message.
pub fn config_verify_suggestions(cfg: &config::Config, message: &str) -> Vec<String> {
    let provider = cfg.effective_provider();
    let api_base = cfg.effective_api_base();
    let mut suggestions = Vec::new();

    if message.contains("No API token configured") {
        suggestions.push(
            "Set `llm.api_token` with `oxo-call config set llm.api_token <token>` or export `OXO_CALL_LLM_API_TOKEN`."
                .to_string(),
        );
        suggestions.push(format!(
            "Current provider is `{provider}`. If that is not what you intended, change it with `oxo-call config set llm.provider <provider>` or `OXO_CALL_LLM_PROVIDER`."
        ));
    }
    if message.contains("Personal Access Tokens are not supported") {
        suggestions.push(
            "The selected endpoint rejected a personal access token. For `github-copilot`, use a Copilot-compatible authentication flow/token, or switch to `openai`, `anthropic`, or `ollama`."
                .to_string(),
        );
    }
    if message.contains("401") || message.contains("403") {
        suggestions.push(
            "The token was rejected. Verify that the token matches the selected provider and that the provider setting is correct."
                .to_string(),
        );
    }
    if message.contains("404") {
        suggestions.push(format!(
            "The endpoint `{api_base}` did not expose the expected `/chat/completions` API. Check `llm.api_base`."
        ));
    }
    if message.contains("API base URL must use HTTPS") {
        suggestions.push(
            "Use an `https://` API base for remote providers, or `http://localhost...` / `http://127.0.0.1...` for local Ollama-compatible endpoints."
                .to_string(),
        );
    }
    if message.contains("HTTP request failed") {
        suggestions.push(
            "Check network connectivity, proxy settings, DNS, and whether the configured `llm.api_base` is reachable from this machine."
                .to_string(),
        );
    }
    if message.contains("Failed to parse API response") {
        suggestions.push(
            "The endpoint responded, but not with the expected OpenAI-compatible chat completions JSON. Check the selected provider and `llm.api_base`."
                .to_string(),
        );
    }

    if suggestions.is_empty() {
        suggestions.push(
            "Run `oxo-call config show` to inspect stored vs effective values, then verify the provider, token source, model, and API base."
                .to_string(),
        );
    }

    suggestions
}
