//! Shared SSE streaming utilities for LLM API responses.
//!
//! This module provides the core SSE (Server-Sent Events) stream reader used by
//! `LlmClient`, `ChatSession`, and the workflow generator.  Keeping it in one
//! place avoids duplicating the parsing and stderr-printing logic.

use crate::error::{OxoError, Result};
use crate::llm::types::StreamChunkResponse;

/// Read an SSE (Server-Sent Events) stream from an OpenAI-compatible API,
/// printing each content token to stderr as it arrives.
///
/// Returns the full accumulated response text.
pub async fn read_sse_stream(response: reqwest::Response) -> Result<String> {
    use futures_util::StreamExt;
    use std::io::Write;

    let mut collected = String::new();
    let mut stream = response.bytes_stream();
    // Buffer for incomplete SSE lines that span multiple chunks.
    let mut line_buf = String::new();
    let mut printed_any = false;

    while let Some(chunk_result) = stream.next().await {
        let chunk =
            chunk_result.map_err(|e| OxoError::LlmError(format!("Stream read error: {e}")))?;
        let text = String::from_utf8_lossy(&chunk);
        line_buf.push_str(&text);

        // Process complete lines from the buffer.
        // Collect tokens from this chunk, then write them all at once
        // (keeping the stderr lock scope synchronous — no .await inside).
        let mut chunk_tokens = String::new();
        while let Some(newline_pos) = line_buf.find('\n') {
            let line = line_buf[..newline_pos].trim().to_string();
            line_buf = line_buf[newline_pos + 1..].to_string();

            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }

            if let Some(json_str) = line.strip_prefix("data: ")
                && let Ok(chunk_resp) = serde_json::from_str::<StreamChunkResponse>(json_str)
            {
                for choice in &chunk_resp.choices {
                    if let Some(ref content) = choice.delta.content {
                        collected.push_str(content);
                        chunk_tokens.push_str(content);
                    }
                }
            }
        }

        if !chunk_tokens.is_empty() {
            let stderr = std::io::stderr();
            let mut lock = stderr.lock();
            let _ = lock.write_all(chunk_tokens.as_bytes());
            let _ = lock.flush();
            printed_any = true;
        }
    }

    // Add a trailing newline if we printed streaming tokens.
    if printed_any {
        let stderr = std::io::stderr();
        let mut lock = stderr.lock();
        let _ = lock.write_all(b"\n");
        let _ = lock.flush();
    }

    Ok(collected)
}

/// Apply provider-specific authentication headers to a request builder.
///
/// This is shared between `LlmClient`, `ChatSession`, and the workflow generator.
pub fn apply_provider_auth_headers(
    req_builder: reqwest::RequestBuilder,
    provider: &str,
    auth_token: &str,
) -> reqwest::RequestBuilder {
    match provider {
        "anthropic" => req_builder
            .header("x-api-key", auth_token)
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
    }
}
