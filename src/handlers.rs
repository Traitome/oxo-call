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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_source_contains_value_and_source() {
        let result = with_source("gpt-4o", "stored config");
        assert!(result.contains("gpt-4o"));
        assert!(result.contains("stored config"));
    }

    #[test]
    fn test_with_source_format() {
        let result = with_source("openai", "env:OXO_CALL_LLM_PROVIDER");
        // Should have both value and source somewhere in the string
        assert!(result.contains("openai"));
        assert!(result.contains("env:OXO_CALL_LLM_PROVIDER"));
    }

    fn default_cfg() -> config::Config {
        config::Config::default()
    }

    #[test]
    fn test_no_api_token_suggestions() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "No API token configured for provider");
        assert!(
            suggestions.len() >= 2,
            "should have at least 2 suggestions for missing token"
        );
        let combined = suggestions.join(" ");
        assert!(combined.contains("llm.api_token"));
        assert!(combined.contains("provider"));
    }

    #[test]
    fn test_personal_access_token_suggestion() {
        let cfg = default_cfg();
        let suggestions =
            config_verify_suggestions(&cfg, "Personal Access Tokens are not supported");
        let combined = suggestions.join(" ");
        assert!(combined.contains("github-copilot") || combined.contains("endpoint"));
    }

    #[test]
    fn test_401_suggestion() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "HTTP 401 Unauthorized");
        let combined = suggestions.join(" ");
        assert!(combined.contains("rejected") || combined.contains("token"));
    }

    #[test]
    fn test_403_suggestion() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "HTTP 403 Forbidden");
        let combined = suggestions.join(" ");
        assert!(combined.contains("rejected") || combined.contains("token"));
    }

    #[test]
    fn test_404_suggestion_contains_api_base() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "endpoint returned 404");
        let combined = suggestions.join(" ");
        assert!(combined.contains("chat/completions") || combined.contains("api_base"));
    }

    #[test]
    fn test_https_suggestion() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "API base URL must use HTTPS");
        let combined = suggestions.join(" ");
        assert!(combined.contains("https://"));
    }

    #[test]
    fn test_http_request_failed_suggestion() {
        let cfg = default_cfg();
        let suggestions =
            config_verify_suggestions(&cfg, "HTTP request failed: connection refused");
        let combined = suggestions.join(" ");
        assert!(combined.contains("network") || combined.contains("connectivity"));
    }

    #[test]
    fn test_parse_api_response_suggestion() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "Failed to parse API response");
        let combined = suggestions.join(" ");
        assert!(combined.contains("OpenAI") || combined.contains("provider"));
    }

    #[test]
    fn test_unknown_error_returns_generic_suggestion() {
        let cfg = default_cfg();
        let suggestions = config_verify_suggestions(&cfg, "some completely unknown error");
        assert!(!suggestions.is_empty());
        let combined = suggestions.join(" ");
        assert!(combined.contains("config show"));
    }

    // ─── print_index_table ────────────────────────────────────────────────────

    #[test]
    fn test_print_index_table_no_panic_empty() {
        print_index_table(&[]);
    }

    #[test]
    fn test_print_index_table_no_panic_with_entries() {
        use chrono::Utc;
        let entries = vec![
            index::IndexEntry {
                tool_name: "samtools".to_string(),
                version: Some("1.17".to_string()),
                indexed_at: Utc::now(),
                doc_size_bytes: 1024,
                sources: vec!["help".to_string()],
            },
            index::IndexEntry {
                tool_name: "bwa".to_string(),
                version: None,
                indexed_at: Utc::now(),
                doc_size_bytes: 512,
                sources: vec![],
            },
        ];
        print_index_table(&entries);
    }
}
