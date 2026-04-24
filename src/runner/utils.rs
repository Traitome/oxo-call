//! Utility functions for command building, risk assessment, and validation.
//!
//! This module contains helper functions used by the runner for:
//! - Building shell command strings
//! - Detecting companion binaries and script executables
//! - Assessing command risk levels
//! - Validating input files
//! - Creating progress spinners

use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};

/// Check if haystack contains needle case-insensitively without allocation.
/// Uses byte-level matching for ASCII strings (sufficient for command/path matching).
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }

    // Byte-level comparison for ASCII-only matching (command names, paths)
    let hay_bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    let needle_len = needle_bytes.len();

    hay_bytes.windows(needle_len).any(|window| {
        window
            .iter()
            .zip(needle_bytes.iter())
            .all(|(h, n)| h.eq_ignore_ascii_case(n))
    })
}

/// Check if haystack ends with needle case-insensitively without allocation.
/// Uses byte-level matching for ASCII strings (sufficient for file extensions).
fn ends_with_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }

    // Byte-level comparison for ASCII-only matching
    let hay_bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    let offset = hay_bytes.len() - needle_bytes.len();

    hay_bytes[offset..]
        .iter()
        .zip(needle_bytes.iter())
        .all(|(h, n)| h.eq_ignore_ascii_case(n))
}

// ─── Command string building ──────────────────────────────────────────────────

/// Build a shell command string from tool name and arguments.
///
/// Arguments containing spaces or shell metacharacters are single-quoted.
/// Shell operators (&&, ||, |, >, etc.) are passed through unquoted.
/// Builds the string directly without intermediate Vec allocation.
pub(crate) fn build_command_string(tool: &str, args: &[String]) -> String {
    if args.is_empty() {
        return tool.to_string();
    }
    let (eff_tool, eff_args) = effective_command(tool, args);
    if eff_args.is_empty() {
        return eff_tool.to_string();
    }

    // Estimate capacity: tool + args + spaces + quoting overhead
    let estimated_len =
        eff_tool.len() + eff_args.len() + eff_args.iter().map(|a| a.len() + 4).sum::<usize>();
    let mut result = String::with_capacity(estimated_len);
    result.push_str(eff_tool);

    for arg in eff_args {
        result.push(' ');
        if is_shell_operator(arg) {
            result.push_str(arg);
        } else if needs_quoting(arg) {
            result.push('\'');
            for c in arg.chars() {
                if c == '\'' {
                    result.push_str("'\\''");
                } else {
                    result.push(c);
                }
            }
            result.push('\'');
        } else {
            result.push_str(arg);
        }
    }
    result
}

/// Resolve the effective (executable, args) pair.
///
/// When the LLM generates a companion binary as the first argument
/// (e.g., `bowtie2-build` when the tool is `bowtie2`), the companion binary
/// is extracted and used as the actual executable with the remaining slice as
/// its arguments.
pub(crate) fn effective_command<'a>(tool: &'a str, args: &'a [String]) -> (&'a str, &'a [String]) {
    if let Some(first) = args.first() {
        if is_companion_binary(tool, first) {
            return (first.as_str(), &args[1..]);
        }
        // Standalone script executables: if the first arg ends with a known
        // script extension and the stem looks like a binary name (no slashes,
        // no whitespace), treat it as the actual command.
        if is_script_executable(first) {
            return (first.as_str(), &args[1..]);
        }
    }
    (tool, args)
}

/// Script extensions recognised as standalone executables.
const SCRIPT_EXTENSIONS: &[&str] = &[".sh", ".py", ".pl", ".R", ".rb", ".jl"];

/// Returns `true` if `candidate` looks like a standalone script executable.
pub fn is_script_executable(candidate: &str) -> bool {
    // Must not contain path separators.
    if candidate.contains('/') || candidate.contains('\\') {
        return false;
    }
    // Must not start with a dash (flag).
    if candidate.starts_with('-') {
        return false;
    }
    // Check for a known script extension.
    for ext in SCRIPT_EXTENSIONS {
        if let Some(stem) = candidate.strip_suffix(ext) {
            // Stem must be non-empty and look like a binary name.
            return !stem.is_empty()
                && stem
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        }
    }
    false
}

/// Returns `true` if `candidate` looks like a companion binary of `tool`.
pub fn is_companion_binary(tool: &str, candidate: &str) -> bool {
    if candidate.starts_with('-') {
        return false; // CLI flag, not a binary
    }
    // Recognised script extensions that companion binaries may carry.
    const SCRIPT_EXTS: &[&str] = &[".sh", ".py", ".pl", ".R", ".rb", ".jl"];

    // Strip a trailing script extension (if any) to obtain the binary stem.
    let stem = SCRIPT_EXTS
        .iter()
        .find_map(|ext| candidate.strip_suffix(ext))
        .unwrap_or(candidate);

    // The stem must look like a binary name: only alphanumeric, hyphen, underscore chars.
    if !stem
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return false;
    }

    // For script companions, any file-extension companion that contains the
    // tool name anywhere in its stem (case-insensitive) is accepted.
    if candidate != stem {
        // Use contains_ignore_ascii_case without allocation
        if contains_ignore_ascii_case(stem, tool) {
            return true;
        }
    }

    // Forward prefix: {tool}- or {tool}_
    // Check prefix patterns directly without allocation
    if stem.len() > tool.len() + 1 {
        let prefix_part = &stem[..tool.len()];
        if prefix_part.eq_ignore_ascii_case(tool) {
            let delim = stem[tool.len()..].chars().next();
            if delim == Some('-') || delim == Some('_') {
                return true;
            }
        }
    }
    // Reverse suffix: _{tool}
    // Check suffix pattern without allocation
    if stem.len() > tool.len() + 1 {
        let suffix_part = &stem[stem.len() - tool.len()..];
        if suffix_part.eq_ignore_ascii_case(tool) {
            let delim_pos = stem.len() - tool.len() - 1;
            let delim = &stem[delim_pos..delim_pos + 1];
            if delim == "_" {
                return true;
            }
        }
    }
    false
}

/// Returns `true` if `arg` is a standalone shell control operator.
pub(crate) fn is_shell_operator(arg: &str) -> bool {
    matches!(
        arg,
        "&&" | "||" | ";" | ";;" | "|" | ">" | ">>" | "<" | "<<" | "2>" | "2>>"
    )
}

/// Returns `true` if any argument is a standalone shell control operator.
pub(crate) fn args_require_shell(args: &[String]) -> bool {
    args.iter().any(|a| is_shell_operator(a))
}

/// Returns `true` if the argument contains characters that require quoting.
pub(crate) fn needs_quoting(arg: &str) -> bool {
    arg.contains(' ')
        || arg.contains('\t')
        || arg.contains(';')
        || arg.contains('&')
        || arg.contains('|')
        || arg.contains('$')
        || arg.contains('`')
        || arg.contains('(')
        || arg.contains(')')
        || arg.contains('<')
        || arg.contains('>')
        || arg.contains('!')
        || arg.contains('\\')
        || arg.contains('"')
        || arg.contains('\'')
}

/// Compute the SHA-256 hex digest of a string.
pub(crate) fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Detect the version string of a tool by running `tool --version`.
pub(crate) fn detect_tool_version(tool: &str) -> Option<String> {
    use std::process::Command;
    let output = Command::new(tool).arg("--version").output().ok()?;
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let version = version.lines().next().unwrap_or("").trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
}

/// Extract a semantic version number from a version string.
///
/// Parses strings like "samtools 1.17" or "1.17.0" into a comparable version tuple.
/// Prefers version patterns that include a dot (X.Y or X.Y.Z).
pub(crate) fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let version = version.trim();

    // Find all candidate version patterns (digit sequences with dots)
    // Use char_indices to avoid Vec<char> allocation
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    let mut char_indices = version.char_indices().peekable();

    while let Some((i, c)) = char_indices.next() {
        if c.is_ascii_digit() {
            let start = i;
            let mut has_dot = false;
            let mut end = i;

            // Continue while we have digits or dots
            while let Some(&(j, next_c)) = char_indices.peek()
                && (next_c.is_ascii_digit() || next_c == '.')
            {
                if next_c == '.' {
                    has_dot = true;
                }
                end = j + next_c.len_utf8();
                char_indices.next();
            }

            // Only consider patterns with dots (proper versions)
            if has_dot {
                candidates.push((start, end));
            }
        }
    }

    // Use the first valid version pattern with dots
    for (start, end) in candidates {
        let version_str = &version[start..end];
        let parts: Vec<&str> = version_str.split('.').collect();

        if let Some(major_str) = parts.first()
            && let Ok(major) = major_str.parse::<u32>()
        {
            let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            return Some((major, minor, patch));
        }
    }

    None
}

/// Check if a tool version is compatible with skill requirements.
///
/// Returns `Ok(())` if compatible, or an error message describing the incompatibility.
pub fn check_version_compatibility(
    tool_version: &str,
    min_version: Option<&str>,
    max_version: Option<&str>,
) -> Result<(), String> {
    let current = parse_version(tool_version)
        .ok_or_else(|| format!("Could not parse tool version: {}", tool_version))?;

    if let Some(min) = min_version
        && let Some(min_ver) = parse_version(min)
        && current < min_ver
    {
        return Err(format!(
            "Tool version {} is below minimum required version {}",
            tool_version, min
        ));
    }

    if let Some(max) = max_version
        && let Some(max_ver) = parse_version(max)
        && current > max_ver
    {
        return Err(format!(
            "Tool version {} exceeds maximum supported version {}",
            tool_version, max
        ));
    }

    Ok(())
}

// ─── Progress spinner ────────────────────────────────────────────────────────

/// Create a progress spinner with a message.
pub fn make_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")
            .expect("valid progress template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

// ─── Output file detection ───────────────────────────────────────────────────

/// Detect output file paths from command arguments.
/// Uses HashSet directly for deduplication to avoid Vec + retain overhead.
pub(crate) fn detect_output_files(args: &[String]) -> Vec<String> {
    const OUTPUT_FLAGS: &[&str] = &[
        "-o", "--output", "-O", "--out", "-b", "--bam", "-S", "--sam", "--vcf", "--bcf",
    ];

    // Use HashSet directly for deduplication, avoiding Vec + retain clone overhead
    let mut files: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut skip_next = false;

    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        // Check prefix patterns directly without format! allocation
        for &flag in OUTPUT_FLAGS {
            // Check if arg starts with flag then '='
            if arg.len() > flag.len() + 1
                && arg[..flag.len()].eq_ignore_ascii_case(flag)
                && arg[flag.len()..].starts_with('=')
            {
                let val = &arg[flag.len() + 1..];
                if !val.is_empty() {
                    files.insert(val.to_string());
                }
                break;
            }
        }
        // Check for -o file form
        for &flag in OUTPUT_FLAGS {
            if arg == flag
                && let Some(val) = args.get(i + 1)
            {
                files.insert(val.clone());
                skip_next = true;
                break;
            }
        }
        // Heuristic: positional arg that looks like a file path
        if !arg.starts_with('-')
            && arg.contains('.')
            && !arg.contains(';')
            && !arg.contains('&')
            && !arg.contains('|')
        {
            files.insert(arg.clone());
        }
    }

    // Convert HashSet to Vec, limit to 20 entries
    files.into_iter().take(20).collect()
}

// ─── Command risk assessment ──────────────────────────────────────────────────

/// Risk level of a generated command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// No dangerous operations detected.
    Safe,
    /// Potentially risky operations (e.g., force overwrite flags).
    Warning,
    /// Highly dangerous operations (e.g., rm, sudo, dd).
    Dangerous,
}

/// Assess the risk level of a generated command by scanning its arguments.
pub fn assess_command_risk(args: &[String]) -> RiskLevel {
    if args.is_empty() {
        return RiskLevel::Safe;
    }

    const DANGEROUS_COMMANDS: &[&str] = &["rm", "rmdir", "sudo", "dd", "mkfs", "chmod", "chown"];
    const FORCE_FLAGS: &[&str] = &["-f", "--force", "--overwrite", "-y", "--yes"];

    let mut risk = RiskLevel::Safe;

    for (i, arg) in args.iter().enumerate() {
        let is_cmd_position =
            i == 0 || (i > 0 && matches!(args[i - 1].as_str(), "&&" | "||" | ";" | "|"));

        if is_cmd_position {
            for &cmd in DANGEROUS_COMMANDS {
                // Use eq_ignore_ascii_case for direct comparison
                if arg.eq_ignore_ascii_case(cmd) {
                    return RiskLevel::Dangerous;
                }
                // Check path suffix: /rm, /sudo, etc. without format! allocation
                // Check if arg ends with "/" + cmd case-insensitively
                if arg.len() > cmd.len() + 1 {
                    let suffix_start = arg.len() - cmd.len() - 1;
                    // Check for '/' before the command name
                    if arg.as_bytes()[suffix_start] == b'/'
                        && arg[suffix_start + 1..].eq_ignore_ascii_case(cmd)
                    {
                        return RiskLevel::Dangerous;
                    }
                }
            }
        }

        if arg == ">" {
            risk = risk.max_level(RiskLevel::Warning);
        }

        // Use eq_ignore_ascii_case for flag comparison
        for &flag in FORCE_FLAGS {
            if arg.eq_ignore_ascii_case(flag) {
                risk = risk.max_level(RiskLevel::Warning);
            }
        }
    }

    if has_same_input_output(args) {
        risk = risk.max_level(RiskLevel::Warning);
    }

    risk
}

impl RiskLevel {
    /// Returns the higher risk level of self and other.
    fn max_level(self, other: RiskLevel) -> RiskLevel {
        match (self, other) {
            (RiskLevel::Dangerous, _) | (_, RiskLevel::Dangerous) => RiskLevel::Dangerous,
            (RiskLevel::Warning, _) | (_, RiskLevel::Warning) => RiskLevel::Warning,
            _ => RiskLevel::Safe,
        }
    }
}

/// Check if the command appears to use the same file as both input and output.
fn has_same_input_output(args: &[String]) -> bool {
    const OUTPUT_FLAGS: &[&str] = &["-o", "--output", "-O", "--out"];
    let mut output_file: Option<&str> = None;
    let mut output_value_indices = std::collections::HashSet::new();

    for (i, arg) in args.iter().enumerate() {
        for &flag in OUTPUT_FLAGS {
            if arg == flag
                && let Some(val) = args.get(i + 1)
            {
                output_file = Some(val.as_str());
                output_value_indices.insert(i + 1);
            }
            // Check prefix without format! allocation
            if arg.len() > flag.len() + 1
                && arg[..flag.len()].eq_ignore_ascii_case(flag)
                && arg[flag.len()..].starts_with('=')
            {
                let rest = &arg[flag.len() + 1..];
                if !rest.is_empty() {
                    output_file = Some(rest);
                    output_value_indices.insert(i);
                }
            }
        }
    }

    if let Some(out) = output_file {
        for (i, arg) in args.iter().enumerate() {
            if !arg.starts_with('-')
                && arg.as_str() == out
                && arg.contains('.')
                && !output_value_indices.contains(&i)
            {
                return true;
            }
        }
    }

    false
}

/// Return risk warning message for display.
pub fn risk_warning_message(risk: RiskLevel) -> Option<&'static str> {
    match risk {
        RiskLevel::Dangerous => Some(
            "⚠️  DANGEROUS: This command contains destructive operations (rm/sudo/dd). \
             Review carefully before executing!",
        ),
        RiskLevel::Warning => Some(
            "⚠  WARNING: This command uses force flags or may overwrite files. \
             Double-check the arguments.",
        ),
        RiskLevel::Safe => None,
    }
}

// ─── Input file validation ────────────────────────────────────────────────────

/// Scan command args for tokens that look like input file paths and check
/// whether they exist on disk.  Returns a list of file paths that were not found.
pub fn validate_input_files(args: &[String]) -> Vec<String> {
    const INPUT_FLAGS: &[&str] = &[
        "-i",
        "--input",
        "-I",
        "--in",
        "-1",
        "-2",
        "--in1",
        "--in2",
        "-x",
        "-U",
        "--ref",
        "--reference",
        "--genome",
        "--genome-dir",
        "--genomeDir",
        "--sjdbGTFfile",
        "--gtf",
        "--bed",
    ];
    const OUTPUT_FLAGS: &[&str] = &["-o", "--output", "-O", "--out", "-b", "--bam", "-S"];

    let mut missing = Vec::new();
    let mut skip_next = false;
    let mut known_output_indices = std::collections::HashSet::new();

    // First pass: mark indices of values following output flags
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        for &flag in OUTPUT_FLAGS {
            if arg == flag {
                known_output_indices.insert(i + 1);
                skip_next = true;
                break;
            }
            // Check --flag=value form without format! allocation
            if arg.len() > flag.len() + 1
                && &arg[..flag.len()] == flag
                && arg[flag.len()..].starts_with('=')
            {
                let rest = &arg[flag.len() + 1..];
                if !rest.is_empty() {
                    known_output_indices.insert(i);
                }
            }
        }
    }

    // Second pass: check input files
    let mut skip_next = false;
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        // Skip output file values
        if known_output_indices.contains(&i) {
            continue;
        }
        // Check input flags
        for &flag in INPUT_FLAGS {
            if arg == flag
                && let Some(val) = args.get(i + 1)
            {
                if looks_like_file_path(val)
                    && has_bio_extension(val)
                    && !std::path::Path::new(val).exists()
                {
                    missing.push(val.clone());
                }
                skip_next = true;
                break;
            }
            // Check --flag=value form without format! allocation
            if arg.len() > flag.len() + 1
                && &arg[..flag.len()] == flag
                && arg[flag.len()..].starts_with('=')
            {
                let rest = &arg[flag.len() + 1..];
                if !rest.is_empty()
                    && looks_like_file_path(rest)
                    && has_bio_extension(rest)
                    && !std::path::Path::new(rest).exists()
                {
                    missing.push(rest.to_string());
                }
            }
        }
        // Check positional args that look like file paths
        if !arg.starts_with('-')
            && looks_like_file_path(arg)
            && has_bio_extension(arg)
            && !std::path::Path::new(arg).exists()
        {
            missing.push(arg.clone());
        }
    }

    missing
}

/// Heuristic: does this token look like a file path?
fn looks_like_file_path(arg: &str) -> bool {
    arg.contains('.')
        && !arg.contains(';')
        && !arg.contains('&')
        && !arg.contains('|')
        && !arg.contains('>')
        && !arg.contains('<')
        && !arg.starts_with("http://")
        && !arg.starts_with("https://")
}

/// Check if a path has a bioinformatics-relevant file extension.
pub(crate) fn has_bio_extension(path: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        ".bam",
        ".sam",
        ".cram",
        ".fastq",
        ".fq",
        ".fasta",
        ".fa",
        ".fna",
        ".vcf",
        ".bcf",
        ".bed",
        ".gff",
        ".gtf",
        ".bw",
        ".bigwig",
        ".fastq.gz",
        ".fq.gz",
        ".vcf.gz",
        ".bed.gz",
        ".fa.gz",
        ".bai",
        ".csi",
        ".tbi",
        ".idx",
    ];
    EXTENSIONS
        .iter()
        .any(|ext| ends_with_ignore_ascii_case(path, ext))
}
