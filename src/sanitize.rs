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
                // (has a slash *after* the leading one and is longer than 3 chars).
                if path[1..].contains('/') && path.len() > 3 {
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
    // Broad substrings that appear in common secret variable names.
    // These cover TOKEN (→ GH_TOKEN, GITHUB_TOKEN, API_TOKEN, …),
    // KEY (→ API_KEY, APIKEY, …), SECRET, PASSWORD/PASS, CREDENTIAL,
    // AUTH (→ AUTH_TOKEN, …), BEARER, and PRIVATE_KEY.
    const SECRET_PATTERNS: &[&str] = &[
        "TOKEN",
        "KEY",
        "SECRET",
        "PASSWORD",
        "PASS",
        "CREDENTIAL",
        "AUTH",
        "BEARER",
    ];
    let mut result = String::new();
    for line in text.lines() {
        let line_upper = line.to_ascii_uppercase();
        let redacted = if SECRET_PATTERNS.iter().any(|p| line_upper.contains(p)) {
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

    #[test]
    fn test_redact_multiple_paths() {
        let input = "cp /home/user/input.bam /data/output.bam";
        let result = redact_paths(input);
        // Both absolute paths should be redacted
        assert!(
            !result.contains("/home/user"),
            "first path should be redacted"
        );
        assert!(
            !result.contains("/data/output"),
            "second path should be redacted"
        );
    }

    #[test]
    fn test_redact_paths_preserves_flags() {
        let input = "samtools sort --threads 8 --sort-by coordinate";
        let result = redact_paths(input);
        // No absolute paths, should be unchanged
        assert!(result.contains("--threads"));
        assert!(result.contains("--sort-by"));
    }

    #[test]
    fn test_redact_paths_empty_string() {
        let result = redact_paths("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_redact_env_tokens_secret() {
        let input = "MY_SECRET=supersecretvalue";
        let result = redact_env_tokens(input);
        assert!(result.contains("<REDACTED>"));
        assert!(!result.contains("supersecretvalue"));
    }

    #[test]
    fn test_redact_env_tokens_multiline() {
        let input = "NORMAL_VAR=hello\nAPI_TOKEN=abc123\nANOTHER=value";
        let result = redact_env_tokens(input);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        // NORMAL_VAR should be preserved
        assert!(lines[0].contains("NORMAL_VAR=hello"));
        // API_TOKEN should be redacted
        assert!(lines[1].contains("<REDACTED>"));
        // ANOTHER should be preserved
        assert!(lines[2].contains("ANOTHER=value"));
    }

    #[test]
    fn test_redact_env_tokens_no_match() {
        let input = "NORMAL_VARIABLE=just_a_value\nFOO=bar";
        let result = redact_env_tokens(input);
        // Nothing should be redacted
        assert!(!result.contains("<REDACTED>"));
        assert!(result.contains("NORMAL_VARIABLE=just_a_value"));
        assert!(result.contains("FOO=bar"));
    }

    // ─── sanitize edge cases ──────────────────────────────────────────────────

    #[test]
    fn test_redact_paths_short_path_not_redacted() {
        // /x is too short to be a multi-component path (length <= 3, no internal /)
        let input = "run /x binary";
        let result = redact_paths(input);
        // Short path should NOT be redacted (no internal '/' AND short)
        assert!(result.contains("/x"), "short path should not be redacted");
    }

    #[test]
    fn test_redact_paths_slash_followed_by_space_not_consumed() {
        // '/' followed by a space — should not trigger path consumption
        let input = "a / b";
        let result = redact_paths(input);
        // The standalone slash should be preserved as-is
        assert!(result.contains('/'), "standalone slash should be preserved");
    }

    #[test]
    fn test_redact_paths_slash_at_start_of_string() {
        // "/short" has a leading '/' but no internal slash after the first char,
        // so it is NOT a multi-component path and must not be redacted.
        let input = "/short";
        let result = redact_paths(input);
        assert_eq!(
            result, "/short",
            "single-component path should not be redacted"
        );
    }

    #[test]
    fn test_redact_env_tokens_extended_patterns() {
        // Ensure the extended secret patterns are matched
        let cases = [
            ("GH_TOKEN=ghp_abc123", true),
            ("GITHUB_TOKEN=ghp_abc123", true),
            ("BEARER_TOKEN=eyJhbGc", true),
            ("MY_PASSWORD=s3cr3t", true),
            ("MY_PASS=hunter2", true),
            ("PRIVATE_KEY=-----BEGIN", true),
            ("APIKEY=abcdef", true),
            ("CREDENTIAL=abc", true),
            ("AUTH_TOKEN=xyz", true),
            ("NORMAL_VARIABLE=value", false),
            ("FOO_BAR=baz", false),
        ];
        for (input, should_redact) in cases {
            let result = redact_env_tokens(input);
            if should_redact {
                assert!(
                    result.contains("<REDACTED>"),
                    "{input:?} should be redacted but wasn't: {result}"
                );
            } else {
                assert!(
                    !result.contains("<REDACTED>"),
                    "{input:?} should NOT be redacted but was: {result}"
                );
            }
        }
    }
}
