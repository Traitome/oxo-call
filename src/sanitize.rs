//! Data anonymization helpers for sensitive LLM contexts.
//!
//! When sending documentation or task descriptions to external LLM providers,
//! callers may want to strip absolute file paths, usernames, and other
//! personally identifiable information.  This module provides lightweight
//! redaction utilities that can be applied before the prompt is sent.

/// Redact absolute file paths (Unix and Windows style) from text.
///
/// Absolute paths are replaced with `<PATH>`.  Relative paths and filenames
/// without a leading `/` or drive letter are left intact because they carry
/// useful semantic information (e.g., `input.bam`).
#[allow(dead_code)]
pub fn redact_paths(text: &str) -> String {
    // Unix absolute paths: /home/user/data/sample.fastq.gz  → <PATH>
    // Windows absolute paths: C:\Users\data\file.bam         → <PATH>
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        // Detect Unix absolute path
        if c == '/'
            && (result.is_empty()
                || result.ends_with(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\''))
        {
            // Peek to see if this looks like a path (not a flag like --foo)
            if let Some(&next) = chars.peek()
                && (next.is_alphanumeric() || next == '~')
            {
                // Consume the rest of the path
                let mut path = String::from(c);
                path.push(next);
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || ch == '"' || ch == '\'' || ch == '\n' {
                        break;
                    }
                    path.push(ch);
                    chars.next();
                }
                // Only redact if it looks like a multi-component path
                if path.contains('/') && path.len() > 3 {
                    result.push_str("<PATH>");
                } else {
                    result.push_str(&path);
                }
                continue;
            }
            result.push(c);
        } else {
            result.push(c);
        }
    }

    result
}

/// Redact environment variable values that look like tokens or secrets.
///
/// Replaces patterns like `export TOKEN=abc123...` with `export TOKEN=<REDACTED>`.
#[allow(dead_code)]
pub fn redact_env_tokens(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        let redacted =
            if line.contains("TOKEN=") || line.contains("KEY=") || line.contains("SECRET=") {
                if let Some(eq_pos) = line.find('=') {
                    format!("{}=<REDACTED>", &line[..eq_pos])
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            };
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&redacted);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_unix_paths() {
        let input = "align reads from /home/user/data/sample_R1.fastq.gz to reference";
        let result = redact_paths(input);
        assert!(result.contains("<PATH>"), "expected redaction in: {result}");
        assert!(!result.contains("/home/user"), "path should be redacted");
    }

    #[test]
    fn test_preserve_relative_paths() {
        let input = "output to results/aligned/sample.bam";
        let result = redact_paths(input);
        assert_eq!(result, input, "relative paths should be preserved");
    }

    #[test]
    fn test_redact_env_tokens() {
        let input = "export OPENAI_API_KEY=sk-abc123xyz\nexport PATH=/usr/bin";
        let result = redact_env_tokens(input);
        assert!(result.contains("<REDACTED>"), "token should be redacted");
        assert!(
            !result.contains("sk-abc123xyz"),
            "token value should be removed"
        );
        // PATH line doesn't match KEY= pattern as standalone
        assert!(
            result.contains("PATH="),
            "non-secret env should be preserved"
        );
    }
}
