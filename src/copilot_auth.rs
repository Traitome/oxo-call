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
}
