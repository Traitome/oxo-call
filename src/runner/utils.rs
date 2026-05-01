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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ignore_ascii_case_basic() {
        assert!(contains_ignore_ascii_case("Sort BAM file", "sort"));
        assert!(contains_ignore_ascii_case("Sort BAM file", "bam"));
        assert!(!contains_ignore_ascii_case("Sort BAM file", "xyz"));
    }

    #[test]
    fn test_contains_ignore_ascii_case_empty_needle() {
        assert!(contains_ignore_ascii_case("any text", ""));
    }

    #[test]
    fn test_contains_ignore_ascii_case_short_haystack() {
        assert!(!contains_ignore_ascii_case("ab", "abc"));
    }

    #[test]
    fn test_ends_with_ignore_ascii_case() {
        assert!(ends_with_ignore_ascii_case("file.BAM", ".bam"));
        assert!(ends_with_ignore_ascii_case("file.fastq.gz", ".gz"));
        assert!(!ends_with_ignore_ascii_case("file.txt", ".bam"));
    }

    #[test]
    fn test_ends_with_ignore_ascii_case_empty_needle() {
        assert!(ends_with_ignore_ascii_case("anything", ""));
    }

    #[test]
    fn test_ends_with_ignore_ascii_case_short_haystack() {
        assert!(!ends_with_ignore_ascii_case("a", ".bam"));
    }

    #[test]
    fn test_build_command_string_simple() {
        let args: Vec<String> = vec!["sort".to_string(), "-o".to_string(), "out.bam".to_string()];
        let result = build_command_string("samtools", &args);
        assert!(result.starts_with("samtools"));
        assert!(result.contains("sort"));
    }

    #[test]
    fn test_build_command_string_no_args() {
        let result = build_command_string("ls", &[]);
        assert_eq!(result, "ls");
    }

    #[test]
    fn test_build_command_string_with_spaces() {
        let args: Vec<String> = vec!["my file.bam".to_string()];
        let result = build_command_string("cat", &args);
        assert!(result.contains("'my file.bam'"));
    }

    #[test]
    fn test_build_command_string_with_shell_operator() {
        let args: Vec<String> = vec![
            "sort".to_string(),
            "in.bam".to_string(),
            ">".to_string(),
            "out.bam".to_string(),
        ];
        let result = build_command_string("samtools", &args);
        assert!(result.contains(" > "));
    }

    #[test]
    fn test_build_command_string_with_single_quotes() {
        let args: Vec<String> = vec!["it's".to_string()];
        let result = build_command_string("echo", &args);
        assert!(result.contains("'\\''"));
    }

    #[test]
    fn test_effective_command_no_companion() {
        let args: Vec<String> = vec!["sort".to_string(), "in.bam".to_string()];
        let (tool, eff_args) = effective_command("samtools", &args);
        assert_eq!(tool, "samtools");
        assert_eq!(eff_args.len(), 2);
    }

    #[test]
    fn test_effective_command_companion_binary() {
        let args: Vec<String> = vec!["bowtie2-build".to_string(), "ref.fa".to_string()];
        let (tool, eff_args) = effective_command("bowtie2", &args);
        assert_eq!(tool, "bowtie2-build");
        assert_eq!(eff_args.len(), 1);
    }

    #[test]
    fn test_effective_command_script_executable() {
        let args: Vec<String> = vec!["run_bwa.sh".to_string(), "ref.fa".to_string()];
        let (tool, eff_args) = effective_command("bwa", &args);
        assert_eq!(tool, "run_bwa.sh");
        assert_eq!(eff_args.len(), 1);
    }

    #[test]
    fn test_is_script_executable() {
        assert!(is_script_executable("run_bwa.sh"));
        assert!(is_script_executable("process.py"));
        assert!(is_script_executable("analyze.R"));
        assert!(!is_script_executable("path/to/run.sh"));
        assert!(!is_script_executable("-flag.sh"));
        assert!(!is_script_executable("noext"));
        assert!(!is_script_executable(".sh"));
    }

    #[test]
    fn test_is_companion_binary_forward_prefix() {
        assert!(is_companion_binary("bowtie2", "bowtie2-build"));
        assert!(is_companion_binary("samtools", "samtools_sort"));
    }

    #[test]
    fn test_is_companion_binary_reverse_suffix() {
        assert!(is_companion_binary("bwa", "run_bwa"));
    }

    #[test]
    fn test_is_companion_binary_flag() {
        assert!(!is_companion_binary("samtools", "-sort"));
    }

    #[test]
    fn test_is_companion_binary_script() {
        assert!(is_companion_binary("bwa", "bwa_align.sh"));
    }

    #[test]
    fn test_is_companion_binary_unrelated() {
        assert!(!is_companion_binary("samtools", "bwa-mem"));
    }

    #[test]
    fn test_is_shell_operator() {
        assert!(is_shell_operator("&&"));
        assert!(is_shell_operator("||"));
        assert!(is_shell_operator("|"));
        assert!(is_shell_operator(">"));
        assert!(is_shell_operator(">>"));
        assert!(is_shell_operator("2>"));
        assert!(!is_shell_operator("sort"));
        assert!(!is_shell_operator("-o"));
    }

    #[test]
    fn test_args_require_shell() {
        assert!(args_require_shell(&[
            "sort".to_string(),
            "|".to_string(),
            "grep".to_string()
        ]));
        assert!(!args_require_shell(&[
            "sort".to_string(),
            "-o".to_string(),
            "out.bam".to_string()
        ]));
    }

    #[test]
    fn test_needs_quoting() {
        assert!(needs_quoting("my file.bam"));
        assert!(needs_quoting("$HOME"));
        assert!(needs_quoting("a;b"));
        assert!(!needs_quoting("simple.bam"));
        assert!(!needs_quoting("-o"));
    }

    #[test]
    fn test_sha256_hex() {
        let h = sha256_hex("hello");
        assert_eq!(h.len(), 64);
        let h2 = sha256_hex("hello");
        assert_eq!(h, h2, "deterministic");
        let h3 = sha256_hex("world");
        assert_ne!(h, h3, "different inputs");
    }

    #[test]
    fn test_parse_version_simple() {
        assert_eq!(parse_version("1.17.0"), Some((1, 17, 0)));
        assert_eq!(parse_version("1.17"), Some((1, 17, 0)));
        assert_eq!(parse_version("2.0.1"), Some((2, 0, 1)));
    }

    #[test]
    fn test_parse_version_with_prefix() {
        assert_eq!(parse_version("samtools 1.17"), Some((1, 17, 0)));
        assert_eq!(parse_version("bwa-mem2 version 2.2.1"), Some((2, 2, 1)));
    }

    #[test]
    fn test_parse_version_no_dots() {
        assert_eq!(parse_version("42"), None);
    }

    #[test]
    fn test_parse_version_empty() {
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_check_version_compatible() {
        assert!(check_version_compatibility("1.17.0", Some("1.0.0"), None).is_ok());
        assert!(check_version_compatibility("1.17.0", None, Some("2.0.0")).is_ok());
        assert!(check_version_compatibility("1.17.0", Some("2.0.0"), None).is_err());
        assert!(check_version_compatibility("3.0.0", None, Some("2.0.0")).is_err());
    }

    #[test]
    fn test_check_version_compatible_no_constraints() {
        assert!(check_version_compatibility("1.0.0", None, None).is_ok());
    }

    #[test]
    fn test_check_version_compatible_unparseable() {
        assert!(check_version_compatibility("no-version", Some("1.0.0"), None).is_err());
    }

    #[test]
    fn test_make_spinner() {
        let pb = make_spinner("testing");
        pb.finish_and_clear();
    }

    #[test]
    fn test_detect_output_files_dash_o() {
        let args: Vec<String> = vec!["-o".to_string(), "out.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(files.contains(&"out.bam".to_string()));
    }

    #[test]
    fn test_detect_output_files_equals() {
        let args: Vec<String> = vec!["--output=out.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(files.contains(&"out.bam".to_string()));
    }

    #[test]
    fn test_detect_output_files_positional() {
        let args: Vec<String> = vec!["input.bam".to_string(), "output.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(files.contains(&"input.bam".to_string()));
        assert!(files.contains(&"output.bam".to_string()));
    }

    #[test]
    fn test_detect_output_files_empty() {
        let files = detect_output_files(&[]);
        assert!(files.is_empty());
    }

    #[test]
    fn test_assess_command_risk_safe() {
        let args: Vec<String> = vec!["sort".to_string(), "-o".to_string(), "out.bam".to_string()];
        assert_eq!(assess_command_risk(&args), RiskLevel::Safe);
    }

    #[test]
    fn test_assess_command_risk_dangerous_rm() {
        let args: Vec<String> = vec!["rm".to_string(), "-rf".to_string(), "/".to_string()];
        assert_eq!(assess_command_risk(&args), RiskLevel::Dangerous);
    }

    #[test]
    fn test_assess_command_risk_dangerous_sudo() {
        let args: Vec<String> = vec!["sudo".to_string(), "apt".to_string(), "install".to_string()];
        assert_eq!(assess_command_risk(&args), RiskLevel::Dangerous);
    }

    #[test]
    fn test_assess_command_risk_dangerous_via_path() {
        let args: Vec<String> = vec!["/bin/rm".to_string(), "file.txt".to_string()];
        assert_eq!(assess_command_risk(&args), RiskLevel::Dangerous);
    }

    #[test]
    fn test_assess_command_risk_warning_force() {
        let args: Vec<String> = vec![
            "cp".to_string(),
            "--force".to_string(),
            "a".to_string(),
            "b".to_string(),
        ];
        assert_eq!(assess_command_risk(&args), RiskLevel::Warning);
    }

    #[test]
    fn test_assess_command_risk_warning_redirect() {
        let args: Vec<String> = vec![
            "echo".to_string(),
            "hello".to_string(),
            ">".to_string(),
            "file.txt".to_string(),
        ];
        assert_eq!(assess_command_risk(&args), RiskLevel::Warning);
    }

    #[test]
    fn test_assess_command_risk_empty() {
        assert_eq!(assess_command_risk(&[]), RiskLevel::Safe);
    }

    #[test]
    fn test_risk_level_max() {
        assert_eq!(
            RiskLevel::Safe.max_level(RiskLevel::Warning),
            RiskLevel::Warning
        );
        assert_eq!(
            RiskLevel::Warning.max_level(RiskLevel::Dangerous),
            RiskLevel::Dangerous
        );
        assert_eq!(RiskLevel::Safe.max_level(RiskLevel::Safe), RiskLevel::Safe);
    }

    #[test]
    fn test_risk_warning_message() {
        assert!(risk_warning_message(RiskLevel::Dangerous).is_some());
        assert!(risk_warning_message(RiskLevel::Warning).is_some());
        assert!(risk_warning_message(RiskLevel::Safe).is_none());
    }

    #[test]
    fn test_has_same_input_output() {
        let args: Vec<String> = vec![
            "-o".to_string(),
            "file.bam".to_string(),
            "file.bam".to_string(),
        ];
        assert!(has_same_input_output(&args));
    }

    #[test]
    fn test_has_different_input_output() {
        let args: Vec<String> = vec![
            "-o".to_string(),
            "out.bam".to_string(),
            "in.bam".to_string(),
        ];
        assert!(!has_same_input_output(&args));
    }

    #[test]
    fn test_validate_input_files_nonexistent() {
        let args: Vec<String> = vec!["nonexistent_file.bam".to_string()];
        let missing = validate_input_files(&args);
        assert!(missing.contains(&"nonexistent_file.bam".to_string()));
    }

    #[test]
    fn test_validate_input_files_existing() {
        let tmp = tempfile::NamedTempFile::with_suffix(".bam").unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let args: Vec<String> = vec![path.clone()];
        let missing = validate_input_files(&args);
        assert!(!missing.contains(&path));
    }

    #[test]
    fn test_validate_input_files_empty() {
        let missing = validate_input_files(&[]);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_looks_like_file_path() {
        assert!(looks_like_file_path("file.bam"));
        assert!(looks_like_file_path("/path/to/file.fastq.gz"));
        assert!(!looks_like_file_path("noextension"));
        assert!(!looks_like_file_path("http://example.com/file.bam"));
        assert!(!looks_like_file_path("a;b.txt"));
    }

    #[test]
    fn test_has_bio_extension() {
        assert!(has_bio_extension("file.bam"));
        assert!(has_bio_extension("file.fastq.gz"));
        assert!(has_bio_extension("file.VCF"));
        assert!(has_bio_extension("file.fa"));
        assert!(!has_bio_extension("file.txt"));
        assert!(!has_bio_extension("file.csv"));
    }

    #[test]
    fn test_detect_tool_version_nonexistent() {
        let result = detect_tool_version("nonexistent_tool_xyz_123");
        assert!(result.is_none());
    }

    #[test]
    fn test_build_command_string_table() {
        let cases: Vec<(&str, Vec<&str>, &str)> = vec![
            ("samtools", vec!["sort", "-o", "out.bam", "in.bam"], "samtools sort -o out.bam in.bam"),
            ("echo", vec![], "echo"),
            ("cat", vec!["file with spaces.txt"], "cat 'file with spaces.txt'"),
            ("sh", vec!["-c", "echo hello"], "sh -c 'echo hello'"),
        ];
        for (tool, args, expected) in cases {
            let args: Vec<String> = args.into_iter().map(String::from).collect();
            let result = build_command_string(tool, &args);
            assert_eq!(result, expected, "build_command_string({tool}, {args:?})");
        }
    }

    #[test]
    fn test_is_shell_operator_table() {
        let operators = vec!["&&", "||", ";", ";;", "|", ">", ">>", "<", "<<", "2>", "2>>"];
        let non_operators = vec!["-o", "--output", "output.bam", "&&cmd", ""];
        for op in operators {
            assert!(is_shell_operator(op), "{op:?} should be a shell operator");
        }
        for non_op in non_operators {
            assert!(!is_shell_operator(non_op), "{non_op:?} should NOT be a shell operator");
        }
    }

    #[test]
    fn test_needs_quoting_table() {
        let cases: Vec<(&str, bool)> = vec![
            ("simple", false),
            ("with space", true),
            ("with\ttab", true),
            ("with;semi", true),
            ("with&amp", true),
            ("with|pipe", true),
            ("with$var", true),
            ("with`backtick", true),
            ("with(paren)", true),
            ("with<angle>", true),
            ("with!exclaim", true),
            ("with\\backslash", true),
            ("with\"quote", true),
            ("with'apos", true),
        ];
        for (arg, expected) in cases {
            assert_eq!(needs_quoting(arg), expected, "needs_quoting({arg:?}) should be {expected}");
        }
    }

    #[test]
    fn test_sha256_hex_known_values() {
        let cases: Vec<(&str, &str)> = vec![
            ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
            ("hello", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
        ];
        for (input, expected) in cases {
            let result = sha256_hex(input);
            assert_eq!(result, expected, "sha256_hex({input:?})");
        }
    }

    #[test]
    fn test_parse_version_table() {
        let cases: Vec<(&str, Option<(u32, u32, u32)>)> = vec![
            ("1.17.0", Some((1, 17, 0))),
            ("samtools 1.17", Some((1, 17, 0))),
            ("2.0", Some((2, 0, 0))),
            ("0.7.17-r1188", Some((0, 7, 17))),
            ("no-dots-here", None),
            ("", None),
            ("4.5.6.7", Some((4, 5, 6))),
        ];
        for (input, expected) in cases {
            let result = parse_version(input);
            assert_eq!(result, expected, "parse_version({input:?})");
        }
    }

    #[test]
    fn test_is_companion_binary_table() {
        let cases: Vec<(&str, &str, bool)> = vec![
            ("bowtie2", "bowtie2-build", true),
            ("samtools", "samtools", false),  // Same name, not a companion
            ("bwa", "-o", false),              // Flag, not a binary
            ("star", "STAR_index", true),      // Contains tool name
            ("gatk", "gatk-4.0.sh", true),    // Script companion
            ("hisat2", "hisat2-build", true),  // Forward prefix
        ];
        for (tool, candidate, expected) in cases {
            let result = is_companion_binary(tool, candidate);
            assert_eq!(result, expected, "is_companion_binary({tool:?}, {candidate:?})");
        }
    }

    #[test]
    fn test_is_script_executable_table() {
        let cases: Vec<(&str, bool)> = vec![
            ("run_pipeline.sh", true),
            ("analyze.py", true),
            ("process.pl", true),
            ("analyze.R", true),
            ("script.rb", true),
            ("model.jl", true),
            ("-o", false),               // Flag, not executable
            ("/path/to/script.sh", false), // Has path separator
            ("noext", false),             // No script extension
            (".sh", false),               // Empty stem
        ];
        for (candidate, expected) in cases {
            let result = is_script_executable(candidate);
            assert_eq!(result, expected, "is_script_executable({candidate:?})");
        }
    }
}
