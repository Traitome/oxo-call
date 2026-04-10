//! GitHub OAuth device-flow authentication helper.
//!
//! This module provides two ways to obtain a GitHub OAuth token that is
//! accepted by the GitHub Copilot API:
//!
//! 1. **`gh` CLI shortcut** – if the user has the GitHub CLI installed and is
//!    already authenticated (`gh auth login`), `gh auth token` returns the
//!    current session token immediately without any interactive prompt.
//!
//! 2. **Device-flow** – for users without the `gh` CLI, an OAuth 2.0 device
//!    authorization grant (RFC 8628) lets the user approve access in their
//!    browser while the CLI waits.  The caller must supply a GitHub OAuth App
//!    `client_id`; no `client_secret` is required for the device flow.

use crate::error::{OxoError, Result};
use serde::Deserialize;
use std::time::Duration;

// ── GitHub OAuth endpoints ────────────────────────────────────────────────────

const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// Buffer subtracted from the device code's `expires_in` to avoid racing the
/// expiry window during the last polling iteration.
const EXPIRY_BUFFER_SECS: u64 = 5;

/// Minimum polling interval enforced even if the server returns a shorter value.
/// RFC 8628 recommends at least 5 seconds between poll attempts.
const MIN_POLL_INTERVAL_SECS: u64 = 5;

// ── Wire types ───────────────────────────────────────────────────────────────

/// Percent-encode a string for use in an `application/x-www-form-urlencoded` body.
///
/// Encodes all characters that are not unreserved URI characters (A-Z, a-z,
/// 0-9, `-`, `_`, `.`, `~`), and converts spaces to `+`.
fn urlenc(s: &str) -> String {
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

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Try to obtain a GitHub OAuth token from the `gh` CLI (`gh auth token`).
///
/// Returns the token on success.  Returns an error when the `gh` binary is not
/// found, when it exits non-zero, or when its output is empty.
pub fn try_gh_auth_token() -> Result<String> {
    let output = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .map_err(|e| {
            OxoError::ConfigError(format!(
                "Could not run `gh auth token`: {e}. \
                 Is the GitHub CLI installed? (https://cli.github.com)"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OxoError::ConfigError(format!(
            "`gh auth token` failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Err(OxoError::ConfigError(
            "`gh auth token` returned an empty token. \
             Run `gh auth login` first."
                .to_string(),
        ));
    }
    Ok(token)
}

/// Run the GitHub OAuth 2.0 device-authorization flow and return the resulting
/// access token.
///
/// # Arguments
/// * `client` – a `reqwest` async client
/// * `client_id` – the GitHub OAuth App client ID (no secret required)
/// * `scope` – space-separated OAuth scopes (e.g. `"read:user"`)
///
/// Prints the user-facing verification URL and one-time code, then polls
/// `TOKEN_URL` until the user completes authorization or the code expires.
#[cfg(not(target_arch = "wasm32"))]
pub async fn run_device_flow(
    client: &reqwest::Client,
    client_id: &str,
    scope: &str,
) -> Result<String> {
    // Step 1 – request a device code
    let dc_body = format!("client_id={}&scope={}", urlenc(client_id), urlenc(scope));
    let dc_resp = client
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

    let dc: DeviceCodeResponse = dc_resp
        .json()
        .await
        .map_err(|e| OxoError::ConfigError(format!("Failed to parse device code response: {e}")))?;

    // Step 2 – show the one-time code to the user
    println!();
    println!("  Open this URL in your browser:");
    println!("    {}", dc.verification_uri);
    println!();
    println!("  Enter this one-time code when prompted:");
    println!("    {}", dc.user_code);
    println!();
    println!(
        "  Waiting for authorization (expires in {}s)…",
        dc.expires_in
    );

    // Step 3 – poll for the token
    let interval = Duration::from_secs(dc.interval.max(MIN_POLL_INTERVAL_SECS));
    let deadline = std::time::Instant::now()
        + Duration::from_secs(dc.expires_in.saturating_sub(EXPIRY_BUFFER_SECS));

    loop {
        if std::time::Instant::now() >= deadline {
            return Err(OxoError::ConfigError(
                "Device code expired before the user completed authorization.".to_string(),
            ));
        }

        tokio::time::sleep(interval).await;

        let tok_resp = client
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "client_id={}&device_code={}&grant_type={}",
                urlenc(client_id),
                urlenc(dc.device_code.as_str()),
                urlenc("urn:ietf:params:oauth:grant-type:device_code"),
            ))
            .send()
            .await
            .map_err(|e| OxoError::ConfigError(format!("Token poll request failed: {e}")))?;

        let tok: TokenResponse = tok_resp
            .json()
            .await
            .map_err(|e| OxoError::ConfigError(format!("Failed to parse token response: {e}")))?;

        if let Some(token) = tok.access_token.filter(|t| !t.is_empty()) {
            return Ok(token);
        }

        match tok.error.as_deref() {
            Some("authorization_pending") => {
                // User hasn't approved yet – keep polling
            }
            Some("slow_down") => {
                // Server asked us to back off
                tokio::time::sleep(Duration::from_secs(MIN_POLL_INTERVAL_SECS)).await;
            }
            Some("expired_token") => {
                return Err(OxoError::ConfigError(
                    "The device code expired. Please run `oxo-call config login` again."
                        .to_string(),
                ));
            }
            Some("access_denied") => {
                return Err(OxoError::ConfigError(
                    "Authorization was denied by the user.".to_string(),
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

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlenc_plain_ascii_unchanged() {
        assert_eq!(urlenc("hello"), "hello");
        assert_eq!(urlenc("abc123"), "abc123");
        assert_eq!(urlenc("read:user"), "read%3Auser");
    }

    #[test]
    fn test_urlenc_space_becomes_plus() {
        assert_eq!(urlenc("hello world"), "hello+world");
    }

    #[test]
    fn test_urlenc_special_characters() {
        // ':' is encoded as %3A
        assert_eq!(urlenc(":"), "%3A");
        // '/' is encoded as %2F
        assert_eq!(urlenc("/"), "%2F");
        // '&' is encoded as %26
        assert_eq!(urlenc("a&b"), "a%26b");
    }

    #[test]
    fn test_urlenc_unreserved_characters_pass_through() {
        // A-Z a-z 0-9 - _ . ~ are unreserved and must not be encoded.
        let unreserved = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
        assert_eq!(urlenc(unreserved), unreserved);
    }

    /// try_gh_auth_token returns an error when the `gh` binary is absent from PATH.
    #[test]
    fn test_try_gh_auth_token_missing_binary_returns_error() {
        // Use an absolute path that does not exist to simulate a missing binary.
        let result = std::process::Command::new("/nonexistent_binary_xyz/gh")
            .args(["auth", "token"])
            .output();
        // The system call itself fails, which is what try_gh_auth_token wraps.
        assert!(result.is_err(), "expected OS error for nonexistent binary");
    }
}
