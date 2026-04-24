//! Unified streaming display with spinner and live preview.
//!
//! This module provides a clean streaming output experience:
//! - Spinner animation during streaming
//! - Live preview of content (last 1-2 lines, displayed inline)
//! - Clean finish without clearing shell prompt
//!
//! # Terminal Safety
//!
//! The spinner message is strictly single-line to avoid terminal control issues.
//! Long preview content is truncated to fit within a reasonable terminal width.
//! All width calculations use **terminal display width** (CJK = 2 columns, emoji = 2 columns).

use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::RwLock;
use unicode_width::UnicodeWidthStr;

/// Maximum terminal display columns for the spinner message (prevents line wrap).
/// Most terminals are 80+ columns wide; this leaves room for spinner + prefix.
const MAX_MESSAGE_WIDTH: usize = 70;

/// Maximum terminal display columns for each preview line.
const MAX_PREVIEW_LINE_WIDTH: usize = 40;

/// Truncate a string so its terminal display width is at most `max_width` columns,
/// appending "..." if truncated. Safe for CJK, emoji, and all Unicode.
fn truncate_display(s: &str, max_width: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width <= max_width {
        return s.to_string();
    }
    // Reserve 3 columns for "..."
    let target = max_width.saturating_sub(3);
    let mut result = String::new();
    let mut current_width = 0;
    for ch in s.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > target {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }
    result.push_str("...");
    result
}

/// Streaming display configuration
pub struct StreamingDisplayConfig {
    /// Spinner message prefix
    pub message: String,
    /// Maximum preview lines (default: 2, displayed inline)
    pub max_preview_lines: usize,
    /// Enable live preview
    pub show_preview: bool,
}

impl Default for StreamingDisplayConfig {
    fn default() -> Self {
        Self {
            message: "Thinking".to_string(),
            max_preview_lines: 2,
            show_preview: true,
        }
    }
}

/// Streaming display state
struct StreamingState {
    /// Collected content
    content: String,
    /// Preview lines for display
    preview_lines: Vec<String>,
}

/// Unified streaming display with spinner and live preview
pub struct StreamingDisplay {
    spinner: ProgressBar,
    config: StreamingDisplayConfig,
    state: Arc<RwLock<StreamingState>>,
}

impl StreamingDisplay {
    /// Create a new streaming display
    pub fn new(config: StreamingDisplayConfig) -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
                .template("{spinner:.cyan} {msg}")
                .expect("valid progress template"),
        );

        // Truncate initial message if too long (by display width)
        let msg = truncate_display(&config.message, MAX_MESSAGE_WIDTH);
        spinner.set_message(msg);
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let state = Arc::new(RwLock::new(StreamingState {
            content: String::new(),
            preview_lines: Vec::new(),
        }));

        Self {
            spinner,
            config,
            state,
        }
    }

    /// Create with default config
    #[allow(dead_code)]
    pub fn with_message(message: &str) -> Self {
        Self::new(StreamingDisplayConfig {
            message: message.to_string(),
            ..Default::default()
        })
    }

    /// Add content to the streaming display
    pub async fn add_content(&self, text: &str) {
        let mut state = self.state.write().await;
        state.content.push_str(text);

        if self.config.show_preview {
            // Keep last N lines for preview
            let all_lines: Vec<&str> = state.content.lines().collect();
            let start = all_lines
                .len()
                .saturating_sub(self.config.max_preview_lines);
            state.preview_lines = all_lines[start..]
                .iter()
                .map(|s| truncate_display(s, MAX_PREVIEW_LINE_WIDTH))
                .collect();

            // Build the full message: prefix + preview
            let preview = state.preview_lines.join(" → ");
            let full_msg = if preview.is_empty() {
                self.config.message.clone()
            } else {
                format!("{}: {}", self.config.message, preview)
            };

            // CRITICAL: Truncate to single line to prevent terminal wrap issues.
            // Long messages can wrap to multiple lines, and finish_and_clear()
            // only clears the current line, leaving artifacts above.
            let truncated = truncate_display(&full_msg, MAX_MESSAGE_WIDTH);

            self.spinner.set_message(truncated);
        }
    }

    /// Finish streaming and get the collected content
    pub async fn finish(self) -> String {
        // Just clear the spinner line (single line, safe)
        self.spinner.finish_and_clear();
        let state = self.state.read().await;
        state.content.trim().to_string()
    }

    /// Get current content without finishing
    #[allow(dead_code)]
    pub async fn content(&self) -> String {
        let state = self.state.read().await;
        state.content.clone()
    }
}

/// Read SSE stream with streaming display
pub async fn read_sse_with_display(
    response: reqwest::Response,
    config: StreamingDisplayConfig,
) -> Result<String, String> {
    use crate::llm::types::StreamChunkResponse;
    use futures_util::StreamExt;

    let display = StreamingDisplay::new(config);
    let mut stream = response.bytes_stream();
    let mut line_buf = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream read error: {e}"))?;
        let text = String::from_utf8_lossy(&chunk);
        line_buf.push_str(&text);

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
                        chunk_tokens.push_str(content);
                    }
                }
            }
        }

        if !chunk_tokens.is_empty() {
            display.add_content(&chunk_tokens).await;
        }
    }

    Ok(display.finish().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_display_config_default() {
        let config = StreamingDisplayConfig::default();
        assert_eq!(config.max_preview_lines, 2);
        assert!(config.show_preview);
    }

    #[test]
    fn test_streaming_display_creation() {
        let _display = StreamingDisplay::with_message("Test");
        // Just verify it doesn't panic
    }

    #[test]
    fn test_message_truncation_on_creation() {
        // Create with a very long message
        let long_msg = "x".repeat(100);
        let _display = StreamingDisplay::new(StreamingDisplayConfig {
            message: long_msg.clone(),
            ..Default::default()
        });
        // Should not panic - the message is truncated internally
    }

    #[tokio::test]
    async fn test_add_content_truncates_long_preview() {
        let display = StreamingDisplay::new(StreamingDisplayConfig {
            message: "Thinking".to_string(),
            max_preview_lines: 2,
            show_preview: true,
        });

        // Add very long content
        let long_line = "a".repeat(100);
        display.add_content(&long_line).await;

        // The preview should be truncated (no panic)
        let state = display.state.read().await;
        assert!(state.content.contains(&long_line));
    }

    #[tokio::test]
    async fn test_add_content_multiple_lines() {
        let display = StreamingDisplay::new(StreamingDisplayConfig {
            message: "Test".to_string(),
            max_preview_lines: 2,
            show_preview: true,
        });

        display.add_content("Line 1\nLine 2\nLine 3").await;

        let state = display.state.read().await;
        // Should keep last 2 lines
        assert!(state.preview_lines.len() <= 2);
    }

    #[test]
    fn test_max_message_width_constant() {
        // Ensure the constant is reasonable for typical terminals
        const { assert!(MAX_MESSAGE_WIDTH >= 60) };
        const { assert!(MAX_MESSAGE_WIDTH <= 100) };
    }

    #[test]
    fn test_truncate_display_ascii() {
        // ASCII: each char = 1 display column
        let s = "Hello World";
        assert_eq!(truncate_display(s, 20), "Hello World");

        let long = "a".repeat(100);
        let truncated = truncate_display(&long, 10);
        assert_eq!(truncated, "aaaaaaa...");
        assert_eq!(UnicodeWidthStr::width(truncated.as_str()), 10);
    }

    #[test]
    fn test_truncate_display_cjk() {
        // CJK: each char = 2 display columns
        let s = "你好世界测试";
        // 5 CJK chars = 10 display columns, truncate to 8 => 2 CJK (4 cols) + "..." (3 cols) = 7 cols
        let truncated = truncate_display(s, 8);
        assert!(truncated.ends_with("..."));
        assert!(UnicodeWidthStr::width(truncated.as_str()) <= 8);

        // Fits within limit
        assert_eq!(truncate_display("你好", 10), "你好");
    }

    #[test]
    fn test_truncate_display_mixed() {
        // Mixed ASCII + CJK
        let s = "Hello你好World世界";
        // Display width: 5 + 4 + 5 + 4 = 18
        let truncated = truncate_display(s, 10);
        assert!(truncated.ends_with("..."));
        assert!(UnicodeWidthStr::width(truncated.as_str()) <= 10);
    }

    #[test]
    fn test_truncate_display_emoji() {
        // Emoji: typically 2 display columns
        let emoji = "🎉🎊🎈🎁🎀";
        let truncated = truncate_display(emoji, 5);
        assert!(truncated.starts_with("🎉"));
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_display_no_truncation() {
        assert_eq!(truncate_display("Hello", 10), "Hello");
        assert_eq!(truncate_display("你好", 10), "你好");
    }

    #[tokio::test]
    async fn test_add_content_with_multibyte_chars() {
        let display = StreamingDisplay::new(StreamingDisplayConfig {
            message: "Processing".to_string(),
            max_preview_lines: 2,
            show_preview: true,
        });

        // Add content with multi-byte chars (CJK, arrows, emoji)
        display.add_content("第一行 → 数据\n第二行 → 更多 🎉").await;

        // Should not panic, content should be stored
        let state = display.state.read().await;
        assert!(state.content.contains("→"));
        assert!(state.content.contains("🎉"));
        assert!(state.content.contains("第一行"));
    }

    #[test]
    fn test_truncate_display_chinese_long_line() {
        // Simulate real streaming content with Chinese
        // Create a string that's definitely longer than MAX_MESSAGE_WIDTH display columns
        let s = "- `<路径>`：开始查找的目录。如果不指定，默认从当前目录开始。这是额外的中文内容确保超过限制。";
        let width = UnicodeWidthStr::width(s);
        assert!(
            width > MAX_MESSAGE_WIDTH,
            "Test string should exceed width limit"
        );

        let truncated = truncate_display(s, MAX_MESSAGE_WIDTH);
        assert!(UnicodeWidthStr::width(truncated.as_str()) <= MAX_MESSAGE_WIDTH);
        assert!(truncated.ends_with("..."));
    }
}
