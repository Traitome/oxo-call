//! GitHub Copilot authentication module.
//!
//! This module implements the complete authentication flow for GitHub Copilot API:
//! 1. OAuth Device Flow using Copilot CLI's GitHub App (client_id: Iv1.b507a08c87ecfe98)
//! 2. Token exchange to get a short-lived Copilot session token
//! 3. Automatic token refresh before expiration
//!
//! The key insight is that `/copilot_internal/v2/token` only accepts GitHub App user
//! tokens (`ghu_`), not OAuth App tokens (`gho_`) or Personal Access Tokens.

use crate::error::{OxoError, Result};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// GitHub Copilot CLI's GitHub App Client ID.
/// This produces `ghu_` tokens that are accepted by the Copilot internal API.
pub const COPILOT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";

/// Token exchange endpoint.
const TOKEN_EXCHANGE_URL: &str = "https://api.github.com/copilot_internal/v2/token";

/// Device code endpoint.
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";

/// Token polling endpoint.
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// OAuth scope for Copilot.
const COPILOT_SCOPE: &str = "read:user";

/// Buffer time before token expiration to trigger refresh (5 minutes).
const REFRESH_BUFFER_SECS: u64 = 300;

// ─── Wire Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Response from the Copilot token exchange endpoint.
#[derive(Debug, Deserialize)]
pub struct CopilotTokenResponse {
    /// The session token (format: "tid=...;exp=...;...")
    pub token: String,
    /// Seconds until the next refresh should happen.
    pub refresh_in: u64,
}

/// Cached Copilot session token with expiration tracking.
#[derive(Debug, Clone)]
pub struct CachedCopilotToken {
    /// The session token for API calls.
    pub session_token: String,
    /// When this token expires.
    pub expires_at: Instant,
    /// The GitHub token used to obtain this session token.
    pub github_token: String,
}

impl CachedCopilotToken {
    /// Check if the token needs refresh (within buffer time of expiration).
    pub fn needs_refresh(&self) -> bool {
        Instant::now() >= self.expires_at - Duration::from_secs(REFRESH_BUFFER_SECS)
    }
}

/// Token manager for GitHub Copilot with automatic refresh.
#[derive(Debug)]
pub struct CopilotTokenManager {
    /// HTTP client for API calls.
    client: reqwest::Client,
    /// Cached session token.
    cache: Arc<RwLock<Option<CachedCopilotToken>>>,
}

impl Default for CopilotTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CopilotTokenManager {
    /// Create a new token manager.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Run the OAuth device flow to get a GitHub token.
    pub async fn run_device_flow(&self) -> Result<String> {
        // Step 1: Request device code
        let dc_body = format!(
            "client_id={}&scope={}",
            url_encode(COPILOT_CLIENT_ID),
            url_encode(COPILOT_SCOPE)
        );

        let dc_resp = self
            .client
            .post(DEVICE_CODE_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(dc_body)
            .send()
            .await
            .map_err(|e| OxoError::ConfigError(format!("Device code request failed: {e}")))?;

        if !dc_resp.status().is_success() {
            let status = dc_resp.status();
            let body = dc_resp.text().await.unwrap_or_default();
            return Err(OxoError::ConfigError(format!(
                "Device code request returned {status}: {body}"
            )));
        }

        let dc: DeviceCodeResponse = dc_resp.json().await.map_err(|e| {
            OxoError::ConfigError(format!("Failed to parse device code response: {e}"))
        })?;

        // Step 2: Show instructions to user
        println!();
        println!("  {}", "Open this URL in your browser:".bold());
        println!("    {}", dc.verification_uri.cyan());
        println!();
        println!("  {}", "Enter this one-time code when prompted:".bold());
        println!("    {}", dc.user_code.yellow().bold());
        println!();
        println!(
            "  Waiting for authorization (expires in {}s)…",
            dc.expires_in
        );

        // Step 3: Poll for token
        let interval = Duration::from_secs(dc.interval.max(5));
        let deadline = Instant::now() + Duration::from_secs(dc.expires_in.saturating_sub(5));

        loop {
            if Instant::now() >= deadline {
                return Err(OxoError::ConfigError(
                    "Device code expired before authorization completed.".to_string(),
                ));
            }

            tokio::time::sleep(interval).await;

            let tok_body = format!(
                "client_id={}&device_code={}&grant_type={}",
                url_encode(COPILOT_CLIENT_ID),
                url_encode(&dc.device_code),
                url_encode("urn:ietf:params:oauth:grant-type:device_code"),
            );

            let tok_resp = self
                .client
                .post(TOKEN_URL)
                .header("Accept", "application/json")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(tok_body)
                .send()
                .await
                .map_err(|e| OxoError::ConfigError(format!("Token poll request failed: {e}")))?;

            let tok: OAuthTokenResponse = tok_resp.json().await.map_err(|e| {
                OxoError::ConfigError(format!("Failed to parse token response: {e}"))
            })?;

            if let Some(token) = tok.access_token.filter(|t| !t.is_empty()) {
                println!();
                println!("  {} Authorization successful!", "✓".green());
                return Ok(token);
            }

            match tok.error.as_deref() {
                Some("authorization_pending") => {
                    // Keep polling
                }
                Some("slow_down") => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Some("expired_token") => {
                    return Err(OxoError::ConfigError(
                        "Device code expired. Please run login again.".to_string(),
                    ));
                }
                Some("access_denied") => {
                    return Err(OxoError::ConfigError(
                        "Authorization was denied.".to_string(),
                    ));
                }
                Some(other) => {
                    let desc = tok.error_description.as_deref().unwrap_or("");
                    return Err(OxoError::ConfigError(format!(
                        "OAuth error '{other}': {desc}"
                    )));
                }
                None => {}
            }
        }
    }

    /// Exchange a GitHub token for a Copilot session token.
    pub async fn exchange_token(&self, github_token: &str) -> Result<CopilotTokenResponse> {
        let resp = self
            .client
            .get(TOKEN_EXCHANGE_URL)
            .header("Authorization", format!("token {github_token}"))
            .header("Accept", "application/json")
            .header("User-Agent", "oxo-call/1.0")
            .header("Editor-Version", "vscode/1.85.0")
            .header("Editor-Plugin-Version", "copilot/1.0.0")
            .send()
            .await
            .map_err(|e| {
                OxoError::LlmError(format!("Copilot token exchange request failed: {e}"))
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OxoError::LlmError(format!(
                "Copilot token exchange failed (HTTP {status}): {body}"
            )));
        }

        let token_resp: CopilotTokenResponse = resp.json().await.map_err(|e| {
            OxoError::LlmError(format!("Failed to parse Copilot token response: {e}"))
        })?;

        Ok(token_resp)
    }

    /// Get a valid session token, refreshing if necessary.
    /// This is the main entry point for API calls.
    pub async fn get_session_token(&self, github_token: &str) -> Result<String> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref()
                && cached.github_token == github_token
                && !cached.needs_refresh()
            {
                return Ok(cached.session_token.clone());
            }
        }

        // Need to refresh or obtain new token
        let token_resp = self.exchange_token(github_token).await?;

        let cached = CachedCopilotToken {
            session_token: token_resp.token.clone(),
            expires_at: Instant::now() + Duration::from_secs(token_resp.refresh_in),
            github_token: github_token.to_string(),
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(cached);
        }

        Ok(token_resp.token)
    }
}

/// URL-encode a string for form-urlencoded bodies.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ─── GitHub Models catalog ────────────────────────────────────────────────────

/// URL of the public GitHub Models catalog endpoint (no authentication required).
pub const GITHUB_MODELS_CATALOG_URL: &str = "https://models.github.ai/catalog/models";

/// A single entry in the Copilot model-selection prompt.
#[derive(Debug, Clone)]
pub struct CopilotModelEntry {
    /// Model ID used in API requests (e.g. `"gpt-5-mini"`).
    pub id: String,
    /// Human-readable description shown in the selection list.
    pub description: String,
    /// `true` when the model is in the "low" (relaxed-rate-limit) tier.
    pub is_low_tier: bool,
}

/// GitHub Copilot models available as a *static fallback* when the live catalog
/// cannot be reached.  Each entry is `(model_id, display_description, is_low_tier)`.
///
/// This list contains only models confirmed available on GitHub Copilot.
/// During `config login` the tool first tries to fetch the live catalog from
/// [`GITHUB_MODELS_CATALOG_URL`] and only falls back to this list on failure.
pub const COPILOT_MODELS_FALLBACK: &[(&str, &str, bool)] = &[
    // ── OpenAI ──────────────────────────────────────────────────────────────────
    (
        "gpt-5-mini",
        "OpenAI gpt-5-mini          · OpenAI, fast lightweight",
        false,
    ),
    (
        "gpt-4.1",
        "OpenAI GPT-4.1             · OpenAI, general-purpose",
        false,
    ),
    (
        "gpt-4.1-mini",
        "OpenAI GPT-4.1-mini        · OpenAI, balanced",
        true,
    ),
    (
        "gpt-4.1-nano",
        "OpenAI GPT-4.1-nano        · OpenAI, ultra-fast",
        true,
    ),
    (
        "gpt-4o",
        "OpenAI GPT-4o              · OpenAI, multimodal",
        false,
    ),
    (
        "gpt-4o-mini",
        "OpenAI GPT-4o mini         · OpenAI, lightweight",
        true,
    ),
    (
        "o3",
        "OpenAI o3                  · OpenAI, deep reasoning",
        false,
    ),
    (
        "o3-mini",
        "OpenAI o3-mini             · OpenAI, fast reasoning",
        false,
    ),
    (
        "o4-mini",
        "OpenAI o4-mini             · OpenAI, agentic reasoning",
        false,
    ),
    // ── Anthropic (Copilot-exclusive) ───────────────────────────────────────────
    (
        "claude-haiku-4.5",
        "Claude Haiku 4.5           · Anthropic, fast",
        false,
    ),
    (
        "claude-sonnet-4",
        "Claude Sonnet 4            · Anthropic",
        false,
    ),
    (
        "claude-sonnet-4.5",
        "Claude Sonnet 4.5          · Anthropic, agent tasks",
        false,
    ),
    // ── Google (Copilot-exclusive) ───────────────────────────────────────────────
    (
        "gemini-2.5-pro",
        "Gemini 2.5 Pro             · Google, deep reasoning",
        false,
    ),
];

/// Convert the static `COPILOT_MODELS_FALLBACK` slice into owned [`CopilotModelEntry`] values.
pub fn fallback_model_entries() -> Vec<CopilotModelEntry> {
    COPILOT_MODELS_FALLBACK
        .iter()
        .map(|&(id, desc, is_low)| CopilotModelEntry {
            id: id.to_string(),
            description: desc.to_string(),
            is_low_tier: is_low,
        })
        .collect()
}

/// A GitHub Models catalog entry (only the fields needed for filtering).
#[derive(Debug, serde::Deserialize)]
struct CatalogModel {
    id: String,
    name: String,
    publisher: String,
    #[serde(default)]
    rate_limit_tier: Option<String>,
    #[serde(default)]
    supported_output_modalities: Option<Vec<String>>,
    #[serde(default)]
    capabilities: Option<Vec<String>>,
}

/// Returns `true` for text-output, streaming-capable models (i.e., chat models).
/// Filters out embeddings, image-generation, and other non-chat modalities.
fn is_chat_capable(m: &CatalogModel) -> bool {
    let has_text_out = m
        .supported_output_modalities
        .as_deref()
        .map(|mods| mods.iter().any(|s| s == "text"))
        .unwrap_or(false);
    let has_streaming = m
        .capabilities
        .as_deref()
        .map(|caps| caps.iter().any(|s| s == "streaming"))
        .unwrap_or(false);
    has_text_out && has_streaming
}

/// Convert a raw [`CatalogModel`] into a [`CopilotModelEntry`].
///
/// Strips the `publisher/` prefix from the model ID (Copilot uses the short form).
fn catalog_model_to_entry(m: CatalogModel) -> CopilotModelEntry {
    // Copilot uses the short model ID (without the "publisher/" prefix).
    let short_id =
        m.id.split_once('/')
            .map(|(_publisher, model_id)| model_id)
            .unwrap_or(&m.id)
            .to_string();
    let tier = m.rate_limit_tier.as_deref().unwrap_or("high");
    let is_low_tier = tier == "low";
    // Display: use the official name from the catalog.
    let description = format!("{:<30} · {}", m.name, m.publisher);
    CopilotModelEntry {
        id: short_id,
        description,
        is_low_tier,
    }
}

/// Promote the default model (`gpt-5-mini`) to the first position in the list.
pub fn promote_default_model(entries: &mut Vec<CopilotModelEntry>) {
    if let Some(pos) = entries.iter().position(|e| e.id == "gpt-5-mini") {
        let item = entries.remove(pos);
        entries.insert(0, item);
    }
}

/// Known Copilot-exclusive models (Anthropic, Google) not listed in the
/// GitHub Models catalog.  These are appended to the live catalog list if
/// absent.
const COPILOT_EXCLUSIVE: &[(&str, &str, bool)] = &[
    (
        "claude-haiku-4.5",
        "Claude Haiku 4.5               · Anthropic, fast",
        false,
    ),
    (
        "claude-sonnet-4",
        "Claude Sonnet 4                · Anthropic",
        false,
    ),
    (
        "claude-sonnet-4.5",
        "Claude Sonnet 4.5              · Anthropic, agent tasks",
        false,
    ),
    (
        "gemini-2.5-pro",
        "Gemini 2.5 Pro                 · Google, deep reasoning",
        false,
    ),
];

/// Fetch the live GitHub Models catalog and return a list of
/// [`CopilotModelEntry`] values ready for the interactive model-selection
/// prompt.
///
/// The catalog endpoint ([`GITHUB_MODELS_CATALOG_URL`]) is publicly accessible
/// without authentication.  Returns `None` on any network or parse error so
/// the caller can fall back to [`COPILOT_MODELS_FALLBACK`] via
/// [`fallback_model_entries`].
pub async fn fetch_github_catalog_models() -> Option<Vec<CopilotModelEntry>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let resp = client
        .get(GITHUB_MODELS_CATALOG_URL)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .header("User-Agent", "oxo-call/1.0")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let models: Vec<CatalogModel> = resp.json().await.ok()?;

    let mut result: Vec<CopilotModelEntry> = models
        .into_iter()
        .filter(is_chat_capable)
        .map(catalog_model_to_entry)
        .collect();

    // Append known Copilot-exclusive models not listed in the GitHub catalog.
    for (id, desc, is_low) in COPILOT_EXCLUSIVE {
        if !result.iter().any(|e| e.id == *id) {
            result.push(CopilotModelEntry {
                id: id.to_string(),
                description: desc.to_string(),
                is_low_tier: *is_low,
            });
        }
    }

    // Always promote gpt-5-mini to the first position (it is the default).
    promote_default_model(&mut result);

    Some(result)
}

/// Global token manager instance.
static TOKEN_MANAGER: std::sync::OnceLock<CopilotTokenManager> = std::sync::OnceLock::new();

/// Get the global token manager instance.
pub fn get_token_manager() -> &'static CopilotTokenManager {
    TOKEN_MANAGER.get_or_init(CopilotTokenManager::new)
}

// ─── Styling helpers (same as colored crate) ───────────────────────────────────

trait ColorExt {
    fn bold(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn cyan(&self) -> String;
}

impl ColorExt for str {
    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[0m", self)
    }

    fn green(&self) -> String {
        format!("\x1b[32m{}\x1b[0m", self)
    }

    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }

    fn cyan(&self) -> String {
        format!("\x1b[36m{}\x1b[0m", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello"), "hello");
        assert_eq!(url_encode("read:user"), "read%3Auser");
        assert_eq!(url_encode("hello world"), "hello+world");
    }

    // ── Catalog filtering tests ───────────────────────────────────────────────

    fn make_catalog_model(
        id: &str,
        name: &str,
        publisher: &str,
        modalities: Option<Vec<&str>>,
        capabilities: Option<Vec<&str>>,
        tier: Option<&str>,
    ) -> CatalogModel {
        CatalogModel {
            id: id.to_string(),
            name: name.to_string(),
            publisher: publisher.to_string(),
            rate_limit_tier: tier.map(|s| s.to_string()),
            supported_output_modalities: modalities
                .map(|v| v.into_iter().map(|s| s.to_string()).collect()),
            capabilities: capabilities.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
        }
    }

    #[test]
    fn test_is_chat_capable_accepts_text_streaming() {
        let m = make_catalog_model(
            "openai/gpt-5-mini",
            "GPT-5 Mini",
            "OpenAI",
            Some(vec!["text"]),
            Some(vec!["streaming", "tool_call"]),
            Some("high"),
        );
        assert!(is_chat_capable(&m));
    }

    #[test]
    fn test_is_chat_capable_rejects_missing_text_output() {
        // embeddings / image-gen models have no text output
        let m = make_catalog_model(
            "openai/text-embedding-3-small",
            "Text Embedding 3 Small",
            "OpenAI",
            Some(vec!["embedding"]),
            Some(vec!["streaming"]),
            Some("high"),
        );
        assert!(!is_chat_capable(&m));
    }

    #[test]
    fn test_is_chat_capable_rejects_missing_streaming() {
        let m = make_catalog_model(
            "openai/some-batch-model",
            "Batch Model",
            "OpenAI",
            Some(vec!["text"]),
            Some(vec!["tool_call"]), // no streaming
            Some("high"),
        );
        assert!(!is_chat_capable(&m));
    }

    #[test]
    fn test_is_chat_capable_rejects_none_modalities() {
        let m = make_catalog_model(
            "openai/unknown",
            "Unknown",
            "OpenAI",
            None, // no output modalities field
            Some(vec!["streaming"]),
            None,
        );
        assert!(!is_chat_capable(&m));
    }

    #[test]
    fn test_catalog_model_to_entry_strips_publisher_prefix() {
        let m = make_catalog_model(
            "openai/gpt-5-mini",
            "GPT-5 Mini",
            "OpenAI",
            Some(vec!["text"]),
            Some(vec!["streaming"]),
            Some("high"),
        );
        let entry = catalog_model_to_entry(m);
        assert_eq!(entry.id, "gpt-5-mini");
        assert!(!entry.is_low_tier);
    }

    #[test]
    fn test_catalog_model_to_entry_no_prefix() {
        // Some catalog entries may not have the publisher/ prefix
        let m = make_catalog_model(
            "gpt-4o",
            "GPT-4o",
            "OpenAI",
            Some(vec!["text"]),
            Some(vec!["streaming"]),
            Some("low"),
        );
        let entry = catalog_model_to_entry(m);
        assert_eq!(entry.id, "gpt-4o");
        assert!(entry.is_low_tier);
    }

    #[test]
    fn test_catalog_model_to_entry_low_tier_flag() {
        let m_low = make_catalog_model(
            "openai/gpt-4.1-mini",
            "GPT-4.1 Mini",
            "OpenAI",
            Some(vec!["text"]),
            Some(vec!["streaming"]),
            Some("low"),
        );
        let m_high = make_catalog_model(
            "openai/gpt-4.1",
            "GPT-4.1",
            "OpenAI",
            Some(vec!["text"]),
            Some(vec!["streaming"]),
            Some("high"),
        );
        assert!(catalog_model_to_entry(m_low).is_low_tier);
        assert!(!catalog_model_to_entry(m_high).is_low_tier);
    }

    #[test]
    fn test_promote_default_model_moves_to_front() {
        let mut entries = vec![
            CopilotModelEntry {
                id: "gpt-4.1".to_string(),
                description: "GPT-4.1".to_string(),
                is_low_tier: false,
            },
            CopilotModelEntry {
                id: "gpt-5-mini".to_string(),
                description: "GPT-5 Mini".to_string(),
                is_low_tier: false,
            },
            CopilotModelEntry {
                id: "claude-sonnet-4".to_string(),
                description: "Claude Sonnet 4".to_string(),
                is_low_tier: false,
            },
        ];
        promote_default_model(&mut entries);
        assert_eq!(entries[0].id, "gpt-5-mini");
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_promote_default_model_no_op_when_absent() {
        let mut entries = vec![
            CopilotModelEntry {
                id: "gpt-4.1".to_string(),
                description: "GPT-4.1".to_string(),
                is_low_tier: false,
            },
            CopilotModelEntry {
                id: "claude-sonnet-4".to_string(),
                description: "Claude Sonnet 4".to_string(),
                is_low_tier: false,
            },
        ];
        promote_default_model(&mut entries);
        // order unchanged
        assert_eq!(entries[0].id, "gpt-4.1");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_fallback_model_entries_round_trips() {
        let entries = fallback_model_entries();
        // All static entries should be present
        assert_eq!(entries.len(), COPILOT_MODELS_FALLBACK.len());
        for (i, (id, desc, is_low)) in COPILOT_MODELS_FALLBACK.iter().enumerate() {
            assert_eq!(entries[i].id, *id);
            assert_eq!(entries[i].description, *desc);
            assert_eq!(entries[i].is_low_tier, *is_low);
        }
    }

    #[test]
    fn test_fallback_first_entry_is_gpt5_mini() {
        // gpt-5-mini must always be the first entry (default model).
        assert_eq!(COPILOT_MODELS_FALLBACK[0].0, "gpt-5-mini");
    }
}
