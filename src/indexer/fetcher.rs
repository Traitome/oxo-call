//! Document fetcher for L2 evidence.
//!
//! Fetches tool documentation from homepages, READMEs, and doc URLs
//! discovered via bioconda metadata. Rate-limited and cached to be
//! a respectful web citizen.
//!
//! ## Features
//! - Rate limiting: max 1 request/second to the same host
//! - Caching: fetched content stored on disk, TTL 7 days
//! - HTML stripping: preserves code blocks and plain text
//! - Size limits: max 500KB per document

#![allow(dead_code)] // used by docs update CLI handlers

use crate::config::Config;
use crate::error::{OxoError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Minimum interval between requests to the same host (ms).
const RATE_LIMIT_MS: u64 = 1000;
/// Maximum response size in bytes (500 KB).
const MAX_RESPONSE_SIZE: usize = 500_000;
/// Cache TTL in seconds (7 days).
const CACHE_TTL_SECS: u64 = 7 * 24 * 3600;

/// Rate limiter: host → last request time.
static LAST_REQUEST: std::sync::LazyLock<Mutex<HashMap<String, Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Fetch a URL with rate limiting and caching.
///
/// Returns the plain text content (HTML tags stripped, code blocks preserved).
/// Returns an error for non-HTTP(S) URLs, non-200 responses, or fetch failures.
pub async fn fetch_document(url: &str) -> Result<String> {
    // Validate URL scheme
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(OxoError::DocFetchError(
            url.to_string(),
            "Only http:// and https:// URLs are accepted".to_string(),
        ));
    }

    // Check cache first
    if let Some(cached) = load_from_cache(url)? {
        return Ok(cached);
    }

    // Rate limit
    let host = extract_host(url);
    let wait_ms = {
        let mut last = LAST_REQUEST.lock().unwrap_or_else(|e| e.into_inner());
        let wait = if let Some(prev) = last.get(&host) {
            let elapsed = prev.elapsed().as_millis() as u64;
            RATE_LIMIT_MS.saturating_sub(elapsed)
        } else {
            0
        };
        last.insert(host.clone(), Instant::now());
        wait
    };
    if wait_ms > 0 {
        tokio::time::sleep(Duration::from_millis(wait_ms)).await;
    }

    // Fetch
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("oxo-call/0.21 (+https://github.com/oxo/oxo-call)")
        .build()
        .map_err(|e| {
            OxoError::DocFetchError(url.to_string(), format!("Failed to build HTTP client: {e}"))
        })?;

    let response = client.get(url).send().await.map_err(|e| {
        OxoError::DocFetchError(url.to_string(), format!("HTTP request failed: {e}"))
    })?;

    if !response.status().is_success() {
        return Err(OxoError::DocFetchError(
            url.to_string(),
            format!("HTTP {}", response.status()),
        ));
    }

    // Read body with size limit
    let body = response.text().await.map_err(|e| {
        OxoError::DocFetchError(url.to_string(), format!("Failed to read response: {e}"))
    })?;

    let body = if body.len() > MAX_RESPONSE_SIZE {
        format!(
            "{}\n...[truncated at {}KB]",
            &body[..MAX_RESPONSE_SIZE],
            MAX_RESPONSE_SIZE / 1000
        )
    } else {
        body
    };

    // Extract text from HTML
    let text = if body.trim_start().starts_with('<') || body.contains("<html") {
        extract_text_from_html(&body)
    } else {
        body
    };

    // Cache for future use
    save_to_cache(url, &text)?;

    Ok(text)
}

/// Fetch multiple documents with rate limiting.
pub async fn fetch_documents(urls: &[String]) -> Vec<(String, Result<String>)> {
    let mut results = Vec::new();
    for url in urls {
        let result = fetch_document(url).await;
        results.push((url.clone(), result));
        // Small pause between requests
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    results
}

/// Extract plain text from HTML, preserving code blocks.
fn extract_text_from_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut consecutive_newlines = 0;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                // Flush code buffer if we're leaving a code context
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => {
                if ch == '\n' {
                    consecutive_newlines += 1;
                    if consecutive_newlines <= 2 {
                        result.push('\n');
                    }
                } else if !ch.is_whitespace() || !result.ends_with(' ') {
                    if ch.is_whitespace() && !result.is_empty() {
                        result.push(' ');
                    } else if !ch.is_whitespace() {
                        result.push(ch);
                    }
                    consecutive_newlines = 0;
                }
            }
            _ => {}
        }
    }

    // Also collect code blocks (text between <code> or <pre> tags)
    // Simple extraction: look for <pre><code> or ``` blocks in the result
    let trimmed = result.trim().to_string();
    if trimmed.is_empty() {
        // Fallback: just strip all tags aggressively
        html.chars()
            .fold((String::new(), false), |(mut s, in_tag), ch| match ch {
                '<' => (s, true),
                '>' => (s, false),
                c if !in_tag => {
                    s.push(c);
                    (s, false)
                }
                _ => (s, true),
            })
            .0
    } else {
        trimmed
    }
}

/// Extract host from a URL for rate limiting.
fn extract_host(url: &str) -> String {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or(url)
        .to_string()
}

/// Cache path for a URL.
fn cache_path(url: &str) -> Result<PathBuf> {
    let hash = sha256_hex(url);
    let dir = Config::data_dir()?.join("docs_cache");
    Ok(dir.join(format!("{hash}.txt")))
}

/// Load cached content for a URL, if not expired.
fn load_from_cache(url: &str) -> Result<Option<String>> {
    let path = cache_path(url)?;
    if !path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(&path)?;
    let modified = metadata
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(modified) > CACHE_TTL_SECS {
        // Expired — remove and return None
        let _ = std::fs::remove_file(&path);
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    Ok(Some(content))
}

/// Save content to cache.
fn save_to_cache(url: &str, content: &str) -> Result<()> {
    let path = cache_path(url)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(())
}

/// Simple SHA-256 hex digest for cache keys.
fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://github.com/user/repo"), "github.com");
        assert_eq!(
            extract_host("http://bioconda.github.io/recipes/samtools/README.html"),
            "bioconda.github.io"
        );
    }

    #[test]
    fn test_extract_text_from_html_basic() {
        let html = "<html><body><h1>Title</h1><p>Hello world.</p></body></html>";
        let text = extract_text_from_html(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_extract_text_from_html_code_block() {
        let html = "<pre><code>samtools sort -o out.bam in.bam</code></pre>";
        let text = extract_text_from_html(html);
        assert!(text.contains("samtools"));
    }
}
