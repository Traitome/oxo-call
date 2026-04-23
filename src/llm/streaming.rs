//! Shared SSE streaming utilities for LLM API responses.
//!
//! This module provides the core SSE (Server-Sent Events) stream reader used by
//! `LlmClient`, `ChatSession`, and the workflow generator.  Keeping it in one
//! place avoids duplicating the parsing and output logic.

use crate::error::{OxoError, Result};
use crate::llm::types::StreamChunkResponse;

/// Output destination for streaming tokens.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StreamOutput {
    /// Write tokens to stderr (default, for progress/preview).
    Stderr,
    /// Write tokens to stdout (for chat mode where content is the primary output).
    Stdout,
    /// Collect tokens silently without printing (for spinner + final render flow).
    Silent,
}

/// Read an SSE (Server-Sent Events) stream from an OpenAI-compatible API,
/// printing each content token to stderr as it arrives.
///
/// Returns the full accumulated response text.
#[allow(dead_code)]
pub async fn read_sse_stream(response: reqwest::Response) -> Result<String> {
    read_sse_stream_to(response, StreamOutput::Stderr).await
}

/// Read an SSE stream with configurable output destination.
///
/// When `output` is `Stderr`, tokens are printed as a progress preview.
/// When `output` is `Stdout`, tokens are printed as the primary content
/// (used by chat mode to avoid double-display).
#[allow(dead_code)]
pub async fn read_sse_stream_to(
    response: reqwest::Response,
    output: StreamOutput,
) -> Result<String> {
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
        // (keeping the lock scope synchronous — no .await inside).
        let mut chunk_tokens = String::new();
        while let Some(newline_pos) = line_buf.find('\n') {
            // Get trimmed line slice (no allocation)
            let line_slice = line_buf[..newline_pos].trim();

            // Handle empty/DONE lines early to avoid further processing
            if line_slice.is_empty() || line_slice == "data: [DONE]" {
                // Safe: newline_pos + 1 is valid UTF-8 boundary (after '\n')
                line_buf.drain(..newline_pos + 1);
                continue;
            }

            // Check for data prefix and parse JSON
            if let Some(json_str) = line_slice.strip_prefix("data: ")
                && let Ok(chunk_resp) = serde_json::from_str::<StreamChunkResponse>(json_str)
            {
                for choice in &chunk_resp.choices {
                    // Handle normal content
                    if let Some(ref content) = choice.delta.content {
                        collected.push_str(content);
                        chunk_tokens.push_str(content);
                    }
                    // Handle reasoning field for thinking models (qwen3.5, deepseek-r1)
                    if (choice.delta.content.is_none()
                        || choice.delta.content.as_ref().is_some_and(|c| c.is_empty()))
                        && let Some(ref reasoning) = choice.delta.reasoning
                    {
                        collected.push_str(reasoning);
                        chunk_tokens.push_str(reasoning);
                    }
                }
            }

            // Drain processed line from buffer (no allocation)
            line_buf.drain(..newline_pos + 1);
        }

        if !chunk_tokens.is_empty() {
            match output {
                StreamOutput::Stderr => {
                    let stderr = std::io::stderr();
                    let mut lock = stderr.lock();
                    let _ = lock.write_all(chunk_tokens.as_bytes());
                    let _ = lock.flush();
                    printed_any = true;
                }
                StreamOutput::Stdout => {
                    let stdout = std::io::stdout();
                    let mut lock = stdout.lock();
                    let _ = lock.write_all(chunk_tokens.as_bytes());
                    let _ = lock.flush();
                    printed_any = true;
                }
                StreamOutput::Silent => {
                    // Collect only, no output — spinner handles progress display
                }
            }
        }
    }

    // Add a trailing newline if we printed streaming tokens.
    if printed_any {
        match output {
            StreamOutput::Stderr => {
                let stderr = std::io::stderr();
                let mut lock = stderr.lock();
                let _ = lock.write_all(b"\n");
                let _ = lock.flush();
            }
            StreamOutput::Stdout => {
                let stdout = std::io::stdout();
                let mut lock = stdout.lock();
                let _ = lock.write_all(b"\n");
                let _ = lock.flush();
            }
            StreamOutput::Silent => {}
        }
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
