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
pub fn strip_prefix_case_insensitive<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = s.to_ascii_lowercase();
    let prefix_lower = prefix.to_ascii_lowercase();
    if lower.starts_with(&prefix_lower) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

/// Post-process LLM-generated args to fix common mistakes:
/// - Strip the tool name if LLM accidentally included it as the first argument
///   (unless it is a recognised companion binary)
/// - Strip markdown code fences that weak models sometimes add around ARGS
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
