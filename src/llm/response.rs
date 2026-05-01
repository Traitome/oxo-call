//! Response parsing functions for LLM outputs.
//!
//! This module contains all functions related to parsing and validating
//! LLM responses in various formats (ARGS/EXPLANATION, JSON, etc.).

use crate::runner::{is_companion_binary, is_script_executable};

use super::types::{LlmCommandSuggestion, LlmRunVerification, LlmSkillVerification};

// ─── Response parsing ─────────────────────────────────────────────────────────

/// Parse the structured verification response from the LLM.
pub fn parse_verification_response(raw: &str) -> LlmRunVerification {
    let mut status = "success";
    let mut summary = String::new();
    let mut issues: Vec<String> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        None,
        Issues,
        Suggestions,
    }
    let mut section = Section::None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("STATUS:") {
            status = match rest.trim() {
                s if s.contains("fail") => "failure",
                s if s.contains("warn") => "warning",
                _ => "success",
            };
        } else if let Some(rest) = trimmed.strip_prefix("SUMMARY:") {
            summary = rest.trim().to_string();
            section = Section::None;
        } else if trimmed.starts_with("ISSUES:") {
            section = Section::Issues;
        } else if trimmed.starts_with("SUGGESTIONS:") {
            section = Section::Suggestions;
        } else if trimmed.starts_with('-') {
            let item = trimmed.trim_start_matches('-').trim().to_string();
            if item.is_empty() || item.eq_ignore_ascii_case("none") {
                continue;
            }
            match section {
                Section::Issues => issues.push(item),
                Section::Suggestions => suggestions.push(item),
                Section::None => {}
            }
        }
    }

    let success = status != "failure";
    LlmRunVerification {
        success,
        summary,
        issues,
        suggestions,
    }
}

/// Parse the structured skill verification response from the LLM.
pub fn parse_skill_verify_response(raw: &str) -> LlmSkillVerification {
    let mut passed = true;
    let mut summary = String::new();
    let mut issues: Vec<String> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        None,
        Issues,
        Suggestions,
    }
    let mut section = Section::None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("VERDICT:") {
            passed = rest.trim().eq_ignore_ascii_case("pass");
        } else if let Some(rest) = trimmed.strip_prefix("SUMMARY:") {
            summary = rest.trim().to_string();
            section = Section::None;
        } else if trimmed.starts_with("ISSUES:") {
            section = Section::Issues;
        } else if trimmed.starts_with("SUGGESTIONS:") {
            section = Section::Suggestions;
        } else if trimmed.starts_with('-') {
            let item = trimmed.trim_start_matches('-').trim().to_string();
            if item.is_empty() || item.eq_ignore_ascii_case("none") {
                continue;
            }
            match section {
                Section::Issues => issues.push(item),
                Section::Suggestions => suggestions.push(item),
                Section::None => {}
            }
        }
    }

    LlmSkillVerification {
        passed,
        summary,
        issues,
        suggestions,
    }
}

/// Strip leading/trailing markdown code fences from LLM output.
pub fn strip_markdown_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    // Remove opening fence (```markdown, ```md, ```, etc.)
    let body = if let Some(rest) = trimmed.strip_prefix("```") {
        // Skip the fence line
        rest.split_once('\n').map(|x| x.1).unwrap_or(rest)
    } else {
        trimmed
    };
    // Remove closing fence
    let body = if let Some(stripped) = body.trim_end().strip_suffix("```") {
        stripped.trim_end()
    } else {
        body
    };
    body.trim().to_string()
}

/// Check whether a suggestion looks valid enough to return without retrying.
pub fn is_valid_suggestion(suggestion: &LlmCommandSuggestion) -> bool {
    // Require both explanation and non-empty args to be considered valid.
    // Empty args usually indicates the LLM failed to follow the output format.
    !suggestion.explanation.is_empty() && !suggestion.args.is_empty()
}

/// Case-insensitive prefix strip.  Returns the remainder after the prefix,
/// or `None` if the string doesn't start with the prefix (case-insensitive).
/// Uses char-by-char comparison to avoid allocation.
pub fn strip_prefix_case_insensitive<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() < prefix.len() {
        return None;
    }
    // Check each char case-insensitively without allocation
    for (s_char, p) in s.chars().zip(prefix.chars()) {
        if !s_char.eq_ignore_ascii_case(&p) {
            return None;
        }
    }
    // All prefix chars matched, return the remainder
    Some(&s[prefix.len()..])
}

/// Post-process LLM-generated args to fix common mistakes:
/// - Strip the tool name if LLM accidentally included it as the first argument
///   (unless it is a recognised companion binary)
/// - Strip markdown code fences that weak models sometimes add around ARGS
/// - Remove duplicate flags (small models often repeat flag patterns)
pub fn sanitize_args(tool: &str, args: Vec<String>) -> Vec<String> {
    if args.is_empty() {
        return args;
    }

    let mut result = args;

    // If the first arg is exactly the tool name (case-insensitive) and is NOT a
    // companion binary, drop it — the tool name is prepended by the runner.
    if let Some(first) = result.first()
        && first.eq_ignore_ascii_case(tool)
        && !is_companion_binary(tool, first)
    {
        result.remove(0);
    }

    // After each && or || operator, inject the tool name when the following
    // token is not already the tool name, not a companion binary, and not a
    // script executable.  This corrects the common LLM failure where multi-step
    // commands omit the tool name for steps after the first, e.g.:
    //   sort ... && index ...  →  sort ... && samtools index ...
    let mut i = 0;
    while i < result.len() {
        if (result[i] == "&&" || result[i] == "||") && i + 1 < result.len() {
            let next = &result[i + 1];
            let needs_injection = !next.eq_ignore_ascii_case(tool)
                && !is_companion_binary(tool, next)
                && !is_script_executable(next);
            if needs_injection {
                result.insert(i + 1, tool.to_string());
                i += 2; // skip the inserted tool name token
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // Remove duplicate flags (CRITICAL for small models that repeat patterns)
    // Strategy: keep first occurrence of each flag, remove subsequent duplicates
    result = deduplicate_flags(&result);

    result
}

/// Remove duplicate flags from args while preserving flag values and order.
///
/// Small models (especially ≤3B) often generate repeated flag patterns like:
///   `--genomeLoad LoadAndKeep --runThreadN 4 --genomeLoad Remove ...`
/// This function keeps only the first occurrence of each flag.
///
/// Rules:
/// - Flags starting with `-` or `--` are tracked
/// - The first occurrence is kept, subsequent duplicates are removed
/// - Flag values (the token after a flag) are preserved with the first occurrence
/// - Non-flag tokens (positional args, values) are always kept
/// - Shell operators (`&&`, `||`, `|`, `>`) reset the tracking for multi-command chains
fn deduplicate_flags(args: &[String]) -> Vec<String> {
    use std::collections::HashSet;

    let mut seen_flags: HashSet<String> = HashSet::new();
    let mut result = Vec::with_capacity(args.len());
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        // Shell operators reset flag tracking for multi-command chains
        if arg == "&&" || arg == "||" || arg == "|" || arg == ">" || arg == ">>" {
            seen_flags.clear();
            result.push(arg.clone());
            i += 1;
            continue;
        }

        // Check if this is a flag
        if arg.starts_with("--") || arg.starts_with('-') {
            // Extract the base flag name (without value if --flag=value format)
            let flag_base = if let Some(eq_pos) = arg.find('=') {
                &arg[..eq_pos]
            } else {
                arg.as_str()
            };

            // Use the flag as-is for comparison (DO NOT lowercase)
            // -i and -I are DIFFERENT flags (lowercase for R1, uppercase for R2 in paired-end tools)
            // Only normalize long flags (--flag vs --FLAG) which are usually case-insensitive
            let flag_key = if flag_base.starts_with("--") {
                flag_base.to_lowercase() // Long flags are typically case-insensitive
            } else {
                flag_base.to_string() // Short flags preserve case (-i vs -I are different)
            };

            if seen_flags.contains(&flag_key) {
                // Duplicate flag detected - skip it and its value if present
                // Check if next arg is a value (not a flag and not a shell operator)
                if i + 1 < args.len() {
                    let next = &args[i + 1];
                    // If the flag expects a value and we're skipping the flag,
                    // also skip the value if it's not another flag or operator
                    if !next.starts_with('-')
                        && next != "&&"
                        && next != "||"
                        && next != "|"
                        && next != ">"
                        && next != ">>"
                        && !flag_base.contains('=')
                    {
                        i += 1; // Skip the value too
                    }
                }
                i += 1;
                continue;
            }

            // First occurrence - keep it and track it
            seen_flags.insert(flag_key);
            result.push(arg.clone());

            // If this is a flag that takes a value (not --flag=value format),
            // the next token is likely the value - keep it but don't track it as a flag
            if i + 1 < args.len() && !arg.contains('=') {
                let next = &args[i + 1];
                if !next.starts_with('-')
                    && next != "&&"
                    && next != "||"
                    && next != "|"
                    && next != ">"
                    && next != ">>"
                {
                    result.push(next.clone());
                    i += 1; // Skip the value in the next iteration
                }
            }
        } else {
            // Non-flag token - always keep
            result.push(arg.clone());
        }

        i += 1;
    }

    result
}

/// Extract a command from freeform text when the model doesn't follow the
/// ARGS:/EXPLANATION: format.  This is a fallback for small models (≤ 3B)
/// that frequently output explanations in natural language instead of the
/// expected format.
///
/// Heuristics used:
/// 1. Look for code blocks (```...```) — the content is likely the command
/// 2. Look for lines starting with a known tool subcommand (e.g., "sort",
///    "view", "mem", "intersect")
/// 3. Look for the first line that looks like a CLI command (contains `-` flags)
pub fn extract_command_from_freeform(raw: &str) -> String {
    // 1. Try to find content inside code blocks
    let mut in_code_block = false;
    let mut code_block_lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code_block {
                break; // end of code block
            }
            in_code_block = true;
            continue;
        }
        if in_code_block {
            code_block_lines.push(line);
        }
    }
    if !code_block_lines.is_empty() {
        // Return the first non-empty line from the code block
        for line in &code_block_lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // 2. Look for lines that start with a known CLI subcommand or flag pattern
    let subcommand_prefixes = [
        "sort",
        "index",
        "view",
        "filter",
        "merge",
        "intersect",
        "mem",
        "align",
        "trim",
        "run",
        "blastn",
        "blastp",
        "blastx",
        "bamtobed",
        "bedtobam",
        "faidx",
        "dict",
        "flagstat",
        "depth",
        "coverage",
        "mpileup",
        "call",
        "concat",
        "norm",
        "annotate",
        "consensus",
        "query",
        "isec",
        "stats",
    ];
    for line in raw.lines() {
        let trimmed = line.trim();
        // Skip empty lines, explanation lines, and "The" lines
        if trimmed.is_empty() || trimmed.starts_with("EXPLANATION") || trimmed.starts_with("The ") {
            continue;
        }
        for prefix in &subcommand_prefixes {
            if trimmed.starts_with(prefix) {
                return trimmed.to_string();
            }
        }
    }

    // 3. Look for the first line that contains CLI flags (starts with `-`)
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('-') || trimmed.contains(" -") {
            // This might be flags without the subcommand — return as-is
            return trimmed.to_string();
        }
    }

    // 4. Give up — return the first non-empty, non-trivial, non-explanation line
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && trimmed.len() > 3
            && !trimmed.starts_with("The ")
            && !trimmed.starts_with("EXPLANATION")
            && !trimmed.starts_with("ARGS:")
        {
            return trimmed.to_string();
        }
    }

    String::new()
}

/// Strip markdown code fences from the raw ARGS line before parsing.
/// Weak LLMs sometimes wrap the response in backticks or triple-backtick blocks.
pub fn strip_code_fences(s: &str) -> &str {
    let trimmed = s.trim();
    // Triple backtick block: ```...```
    if let Some(inner) = trimmed.strip_prefix("```") {
        // May optionally have a language hint on the first line
        let inner = inner.strip_prefix("bash").unwrap_or(inner);
        let inner = inner.strip_prefix("sh").unwrap_or(inner);
        let inner = inner.trim_start_matches('\n');
        if let Some(inner) = inner.strip_suffix("```") {
            return inner.trim();
        }
        return inner.trim();
    }
    // Single backtick wrapper: `...`
    if let Some(inner) = trimmed.strip_prefix('`')
        && let Some(inner) = inner.strip_suffix('`')
    {
        return inner.trim();
    }
    trimmed
}

// ─── Shell argument parser ────────────────────────────────────────────────────

/// Simple shell-like argument tokenizer that handles single and double quotes.
pub fn parse_shell_args(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            '\\' if !in_single_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Parse LLM response into a command suggestion.
///
/// Tries JSON structured output first, then falls back to standard ARGS:/EXPLANATION: format.
pub fn parse_response(raw: &str) -> crate::error::Result<LlmCommandSuggestion> {
    // ── Try JSON structured output first ──────────────────────────────────
    //
    // Models that support JSON mode (GPT-4+, Claude) may return structured
    // output.  This is more reliable than regex parsing.
    if let Some(suggestion) = try_parse_json_response(raw) {
        return Ok(suggestion);
    }

    // ── Standard ARGS:/EXPLANATION: format ────────────────────────────────
    let mut args_line = String::new();
    let mut explanation_line = String::new();

    for line in raw.lines() {
        let trimmed = line.trim_start();
        // Support case-insensitive prefix matching for common LLM deviations:
        // "ARGS:", "Args:", "args:", "**ARGS:**", "## ARGS:", etc.
        let stripped = trimmed
            .trim_start_matches('*')
            .trim_start_matches('#')
            .trim_start();
        if let Some(rest) = strip_prefix_case_insensitive(stripped, "ARGS:") {
            args_line = rest
                .trim()
                .trim_start_matches('*') // strip residual bold markers in value
                .trim_end_matches('*')
                .trim()
                .to_string();
        } else if let Some(rest) = strip_prefix_case_insensitive(stripped, "EXPLANATION:") {
            explanation_line = rest
                .trim()
                .trim_start_matches('*')
                .trim_end_matches('*')
                .trim()
                .to_string();
        }
    }

    // Treat "(none)" as empty args
    if args_line == "(none)" {
        args_line.clear();
    }

    // Fallback: when the model doesn't output ARGS: format (common for
    // small models like deepseek-coder:1.3b), try to extract the command
    // from the raw response using heuristics.
    if args_line.is_empty() {
        args_line = extract_command_from_freeform(raw);
    }

    // Strip markdown code fences that weak LLMs sometimes add
    let cleaned = strip_code_fences(&args_line);
    let args = parse_shell_args(cleaned);

    Ok(LlmCommandSuggestion {
        args,
        explanation: explanation_line,
        inference_ms: 0.0, // Set by caller (suggest_command)
    })
}

/// Try to parse the LLM response as a JSON object with `args` and `explanation` fields.
///
/// This handles models that support structured/JSON output mode.
/// Returns `None` if the response is not valid JSON or doesn't have the expected shape.
pub fn try_parse_json_response(raw: &str) -> Option<LlmCommandSuggestion> {
    // Try to find JSON in the response (may be wrapped in markdown code fences)
    let trimmed = raw.trim();
    let json_str = if trimmed.starts_with("```json") || trimmed.starts_with("```") {
        // Extract content between code fences
        let start = trimmed.find('{').unwrap_or(0);
        let end = trimmed.rfind('}').map(|i| i + 1).unwrap_or(trimmed.len());
        &trimmed[start..end]
    } else if trimmed.starts_with('{') {
        trimmed
    } else {
        return None;
    };

    let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;

    let args_str = parsed
        .get("args")
        .and_then(|v| v.as_str())
        .or_else(|| parsed.get("ARGS").and_then(|v| v.as_str()))?;

    let explanation = parsed
        .get("explanation")
        .and_then(|v| v.as_str())
        .or_else(|| parsed.get("EXPLANATION").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    let cleaned = strip_code_fences(args_str);
    let args = parse_shell_args(cleaned);

    Some(LlmCommandSuggestion {
        args,
        explanation,
        inference_ms: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_flags_simple() {
        let args: Vec<String> = vec![
            "--genomeLoad".into(),
            "LoadAndKeep".into(),
            "--runThreadN".into(),
            "4".into(),
            "--genomeLoad".into(),
            "Remove".into(),
        ]
        .into_iter()
        .collect();
        let result = deduplicate_flags(&args);
        // First --genomeLoad and its value should be kept, second removed
        assert_eq!(
            result,
            vec!["--genomeLoad", "LoadAndKeep", "--runThreadN", "4"]
        );
    }

    #[test]
    fn test_deduplicate_flags_massive_repetition() {
        // Simulates the real bug: hundreds of repeated flag patterns
        let mut args = vec!["--genomeDir".to_string(), "/star_index".to_string()];
        for _ in 0..50 {
            args.push("--genomeLoad".to_string());
            args.push("LoadAndKeep".to_string());
            args.push("--runThreadN".to_string());
            args.push("4".to_string());
            args.push("--genomeLoad".to_string());
            args.push("Remove".to_string());
        }
        let result = deduplicate_flags(&args);
        // Should have only one occurrence of each flag
        assert_eq!(
            result,
            vec![
                "--genomeDir",
                "/star_index",
                "--genomeLoad",
                "LoadAndKeep",
                "--runThreadN",
                "4"
            ]
        );
    }

    #[test]
    fn test_deduplicate_flags_equals_format() {
        let args: Vec<String> = vec![
            "--threads=4".into(),
            "--output=out.bam".into(),
            "--threads=8".into(), // Duplicate with different value
        ];
        let result = deduplicate_flags(&args);
        // First occurrence kept, second removed
        assert_eq!(result, vec!["--threads=4", "--output=out.bam"]);
    }

    #[test]
    fn test_deduplicate_flags_preserves_non_flags() {
        let args: Vec<String> = vec![
            "sort".into(),
            "-@".into(),
            "4".into(),
            "-o".into(),
            "sorted.bam".into(),
            "input.bam".into(),
        ];
        let result = deduplicate_flags(&args);
        assert_eq!(
            result,
            vec!["sort", "-@", "4", "-o", "sorted.bam", "input.bam"]
        );
    }

    #[test]
    fn test_deduplicate_flags_multi_command_with_operators() {
        let args: Vec<String> = vec![
            "sort".into(),
            "-o".into(),
            "sorted.bam".into(),
            "input.bam".into(),
            "&&".into(),
            "index".into(),
            "-o".into(), // Same flag but in different command segment - should be kept
            "sorted.bam.bai".into(),
            "sorted.bam".into(),
        ];
        let result = deduplicate_flags(&args);
        // After &&, flag tracking resets, so -o should appear twice
        assert_eq!(
            result,
            vec![
                "sort",
                "-o",
                "sorted.bam",
                "input.bam",
                "&&",
                "index",
                "-o",
                "sorted.bam.bai",
                "sorted.bam"
            ]
        );
    }

    #[test]
    fn test_deduplicate_flags_case_insensitive() {
        let args: Vec<String> = vec![
            "--GenomeLoad".into(),
            "LoadAndKeep".into(),
            "--genomeload".into(), // Same flag, different case
            "Remove".into(),
        ];
        let result = deduplicate_flags(&args);
        // Case-insensitive dedup: second occurrence removed
        assert_eq!(result, vec!["--GenomeLoad", "LoadAndKeep"]);
    }

    #[test]
    fn test_sanitize_args_with_dedup() {
        let args: Vec<String> = vec![
            "--genomeLoad".into(),
            "LoadAndKeep".into(),
            "--runThreadN".into(),
            "4".into(),
            "--genomeLoad".into(),
            "Remove".into(),
        ];
        let result = sanitize_args("star", args);
        assert_eq!(
            result,
            vec!["--genomeLoad", "LoadAndKeep", "--runThreadN", "4"]
        );
    }

    #[test]
    fn test_sanitize_args_strips_tool_name_and_dedup() {
        let args: Vec<String> = vec![
            "star".into(), // Tool name should be stripped
            "--genomeLoad".into(),
            "LoadAndKeep".into(),
            "--genomeLoad".into(), // Duplicate
            "Remove".into(),
        ];
        let result = sanitize_args("star", args);
        assert_eq!(result, vec!["--genomeLoad", "LoadAndKeep"]);
    }

    // ── parse_verification_response additional tests ──────────────────────────

    #[test]
    fn test_parse_verification_response_warning_status() {
        let raw = "STATUS: warning\nSUMMARY: Some warnings.\nISSUES:\n- Low coverage\nSUGGESTIONS:\n- Increase depth";
        let v = parse_verification_response(raw);
        assert!(v.success); // warning is not failure
        assert_eq!(v.summary, "Some warnings.");
        assert_eq!(v.issues.len(), 1);
        assert_eq!(v.suggestions.len(), 1);
    }

    #[test]
    fn test_parse_verification_response_failure_status() {
        let raw = "STATUS: failure\nSUMMARY: Died.\nISSUES:\n- Crash\nSUGGESTIONS:\n- Retry";
        let v = parse_verification_response(raw);
        assert!(!v.success);
    }

    #[test]
    fn test_parse_verification_response_none_items_ignored() {
        let raw = "STATUS: success\nSUMMARY: OK\nISSUES:\n- none\nSUGGESTIONS:\n- none";
        let v = parse_verification_response(raw);
        assert!(v.issues.is_empty());
        assert!(v.suggestions.is_empty());
    }

    #[test]
    fn test_parse_verification_response_empty_input() {
        let v = parse_verification_response("");
        assert!(v.success); // default is success
        assert!(v.summary.is_empty());
        assert!(v.issues.is_empty());
    }

    // ── parse_skill_verify_response tests ────────────────────────────────────

    #[test]
    fn test_parse_skill_verify_response_pass() {
        let raw = "VERDICT: pass\nSUMMARY: Great skill.\nISSUES:\n- none\nSUGGESTIONS:\n- none";
        let v = parse_skill_verify_response(raw);
        assert!(v.passed);
        assert_eq!(v.summary, "Great skill.");
        assert!(v.issues.is_empty());
    }

    #[test]
    fn test_parse_skill_verify_response_fail() {
        let raw = "VERDICT: fail\nSUMMARY: Needs work.\nISSUES:\n- Missing examples\nSUGGESTIONS:\n- Add 5+ examples";
        let v = parse_skill_verify_response(raw);
        assert!(!v.passed);
        assert_eq!(v.issues.len(), 1);
        assert_eq!(v.suggestions.len(), 1);
    }

    #[test]
    fn test_parse_skill_verify_response_empty() {
        let v = parse_skill_verify_response("");
        assert!(v.passed); // default is pass
        assert!(v.summary.is_empty());
    }

    // ── strip_markdown_fences tests ───────────────────────────────────────────

    #[test]
    fn test_strip_markdown_fences_basic() {
        let raw = "```\nsome content\n```";
        let stripped = strip_markdown_fences(raw);
        assert_eq!(stripped, "some content");
    }

    #[test]
    fn test_strip_markdown_fences_with_language() {
        let raw = "```markdown\n# Title\nContent\n```";
        let stripped = strip_markdown_fences(raw);
        assert_eq!(stripped, "# Title\nContent");
    }

    #[test]
    fn test_strip_markdown_fences_no_fences() {
        let raw = "plain text";
        let stripped = strip_markdown_fences(raw);
        assert_eq!(stripped, "plain text");
    }

    #[test]
    fn test_strip_markdown_fences_empty() {
        let stripped = strip_markdown_fences("");
        assert!(stripped.is_empty());
    }

    // ── strip_prefix_case_insensitive tests ───────────────────────────────────

    #[test]
    fn test_strip_prefix_case_insensitive_match() {
        let result = strip_prefix_case_insensitive("ARGS: -o out.bam", "args:");
        assert_eq!(result, Some(" -o out.bam"));
    }

    #[test]
    fn test_strip_prefix_case_insensitive_no_match() {
        let result = strip_prefix_case_insensitive("EXPLANATION: text", "args:");
        assert!(result.is_none());
    }

    #[test]
    fn test_strip_prefix_case_insensitive_too_short() {
        let result = strip_prefix_case_insensitive("a", "args:");
        assert!(result.is_none());
    }

    #[test]
    fn test_strip_prefix_case_insensitive_exact() {
        let result = strip_prefix_case_insensitive("ARGS:", "ARGS:");
        assert_eq!(result, Some(""));
    }

    // ── is_valid_suggestion tests ─────────────────────────────────────────────

    #[test]
    fn test_is_valid_suggestion_empty_args() {
        let s = super::super::types::LlmCommandSuggestion {
            args: vec![],
            explanation: "Some explanation".to_string(),
            inference_ms: 0.0,
        };
        assert!(!is_valid_suggestion(&s));
    }

    #[test]
    fn test_is_valid_suggestion_empty_explanation() {
        let s = super::super::types::LlmCommandSuggestion {
            args: vec!["-o".to_string()],
            explanation: String::new(),
            inference_ms: 0.0,
        };
        assert!(!is_valid_suggestion(&s));
    }

    #[test]
    fn test_is_valid_suggestion_both_empty() {
        let s = super::super::types::LlmCommandSuggestion {
            args: vec![],
            explanation: String::new(),
            inference_ms: 0.0,
        };
        assert!(!is_valid_suggestion(&s));
    }

    // ── extract_command_from_freeform tests ───────────────────────────────────

    #[test]
    fn test_extract_command_from_freeform_code_block() {
        let raw = "Here is the command:\n```\nsort -o out.bam in.bam\n```";
        let cmd = extract_command_from_freeform(raw);
        assert_eq!(cmd, "sort -o out.bam in.bam");
    }

    #[test]
    fn test_extract_command_from_freeform_flag_line() {
        let raw = "Use the following flags:\n-o out.bam input.bam";
        let cmd = extract_command_from_freeform(raw);
        assert!(cmd.starts_with('-'));
    }

    #[test]
    fn test_extract_command_from_freeform_empty() {
        let cmd = extract_command_from_freeform("");
        assert!(cmd.is_empty());
    }

    // ── try_parse_json_response tests ─────────────────────────────────────────

    #[test]
    fn test_try_parse_json_response_valid() {
        let raw = r#"{"args": "-o out.bam in.bam", "explanation": "Sort the file"}"#;
        let result = try_parse_json_response(raw);
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(!s.args.is_empty());
        assert_eq!(s.explanation, "Sort the file");
    }

    #[test]
    fn test_try_parse_json_response_invalid() {
        let raw = "ARGS: -o out.bam\nEXPLANATION: Sort the file";
        let result = try_parse_json_response(raw);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_json_response_in_code_fence() {
        let raw = "```json\n{\"args\": \"-o out.bam\", \"explanation\": \"Output file\"}\n```";
        let result = try_parse_json_response(raw);
        assert!(result.is_some());
    }

    // ── strip_code_fences tests ───────────────────────────────────────────────

    #[test]
    fn test_strip_code_fences_bash() {
        let s = "```bash\ncommand -o out\n```";
        assert_eq!(strip_code_fences(s), "command -o out");
    }

    #[test]
    fn test_strip_code_fences_single_backtick() {
        let s = "`command -o out`";
        assert_eq!(strip_code_fences(s), "command -o out");
    }

    #[test]
    fn test_strip_code_fences_no_fences() {
        let s = "command -o out";
        assert_eq!(strip_code_fences(s), "command -o out");
    }

    // ── parse_shell_args tests ────────────────────────────────────────────────

    #[test]
    fn test_parse_shell_args_quoted() {
        let args = parse_shell_args("sort 'my file.bam' -o out.bam");
        assert_eq!(args, vec!["sort", "my file.bam", "-o", "out.bam"]);
    }

    #[test]
    fn test_parse_shell_args_double_quoted() {
        let args = parse_shell_args(r#"echo "hello world""#);
        assert_eq!(args, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_parse_shell_args_empty() {
        let args = parse_shell_args("");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_shell_args_whitespace_only() {
        let args = parse_shell_args("   ");
        assert!(args.is_empty());
    }

    // ── sanitize_args additional tests ───────────────────────────────────────

    #[test]
    fn test_sanitize_args_empty() {
        let result = sanitize_args("samtools", vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_sanitize_args_injects_tool_after_operator() {
        let args: Vec<String> = vec![
            "sort".into(),
            "-o".into(),
            "sorted.bam".into(),
            "in.bam".into(),
            "&&".into(),
            "index".into(), // no tool name here — should be injected
            "sorted.bam".into(),
        ];
        let result = sanitize_args("samtools", args);
        // Find && in result and check next token is "samtools"
        let and_pos = result.iter().position(|a| a == "&&").unwrap();
        assert_eq!(result[and_pos + 1], "samtools");
    }

    #[test]
    fn test_sanitize_args_does_not_strip_companion_binary() {
        // bowtie2-build is a companion binary — should NOT be stripped
        let args: Vec<String> = vec!["bowtie2-build".into(), "ref.fa".into(), "ref_idx".into()];
        let result = sanitize_args("bowtie2", args);
        assert_eq!(result[0], "bowtie2-build");
    }
}
