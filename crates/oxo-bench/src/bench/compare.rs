//! Command comparison engine with flag-order awareness.
//!
//! Compares a generated ARGS string against a known-good reference and produces
//! a set of metrics that capture both strict and relaxed notions of "correct".
//!
//! ## Metric design goals
//!
//! CLI commands have strict semantics: a flag must appear with its correct value
//! and in a position where the parser can associate them.  The legacy
//! token-set metrics (Jaccard, recall, precision) treat every whitespace-separated
//! token as an unordered set element, which means that `--threads 8` and
//! `8 --threads` score identically — even though the latter is semantically
//! wrong for many tools.
//!
//! This module therefore exposes two complementary metric families:
//!
//! 1. **Token-set metrics** (`token_jaccard`, `flag_recall`, `flag_precision`):
//!    retain backward compatibility and capture rough token overlap.
//! 2. **Flag-group metrics** (`flag_group_recall`, `flag_group_precision`,
//!    `flag_group_jaccard`): group each named flag with its immediately-following
//!    non-flag value token (e.g., `["--threads", "8"]`), then compare those
//!    groups as sets.  This correctly distinguishes `--threads 8` from
//!    `8 --threads` and from `--8 threads`.
//! 3. **Positional-order match** (`positional_order_match`): checks whether the
//!    non-flag (positional) tokens appear in the same relative order in both
//!    the generated and reference commands.  Positional argument order matters
//!    for tools where the parser assigns meaning by position.
//!
//! The `accuracy_score()` composite is weighted over the flag-group metrics,
//! not the raw token-set metrics, to avoid rewarding semantically incorrect
//! flag-value swaps.

/// Result of comparing a generated command against a reference.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompareResult {
    /// Exact string match after whitespace normalisation.
    pub exact_match: bool,
    /// Jaccard similarity of the two raw token sets (order-insensitive).
    ///
    /// Retained for backward compatibility.  Prefer `flag_group_jaccard` for
    /// accuracy-related decisions because it is aware of flag–value pairing.
    pub token_jaccard: f64,
    /// Fraction of reference tokens found in the generated output (token-set level).
    ///
    /// Retained for backward compatibility.  Prefer `flag_group_recall`.
    pub flag_recall: f64,
    /// Fraction of generated tokens matching the reference (token-set level).
    ///
    /// Retained for backward compatibility.  Prefer `flag_group_precision`.
    pub flag_precision: f64,
    /// Fraction of reference flag–value groups found in the generated output.
    ///
    /// A "flag group" is a flag token (`-f` / `--flag`) paired with its
    /// immediately-following non-flag value, or a standalone positional token.
    /// This metric correctly penalises `8 --threads` when the reference is
    /// `--threads 8` because the groups do not match.
    pub flag_group_recall: f64,
    /// Fraction of generated flag–value groups that appear in the reference.
    pub flag_group_precision: f64,
    /// Jaccard similarity over flag–value groups.
    pub flag_group_jaccard: f64,
    /// Whether positional (non-flag) arguments appear in the same relative
    /// order in the generated and reference commands.
    ///
    /// A value of `1.0` means the positional sequence is identical; `0.0`
    /// means at least one positional is present in both but the order differs.
    /// When either side has no positional arguments this returns `1.0`
    /// (vacuous truth).
    pub positional_order_match: f64,
    /// Whether the first token (subcommand) matches.
    pub subcommand_match: bool,
}

impl CompareResult {
    /// A composite accuracy score in \[0, 1\].
    ///
    /// The score uses a **subcommand veto factor** instead of a small bonus:
    ///
    /// 1. Base score = weighted combination of flag-group metrics:
    ///    - 35% `flag_group_recall`
    ///    - 25% `flag_group_precision`
    ///    - 25% `flag_group_jaccard`
    ///    - 15% `positional_order_match`
    ///
    /// 2. Subcommand veto: when the subcommand is wrong, the base score is
    ///    multiplied by 0.3 (capped at 0.3 max). This reflects the reality
    ///    that a wrong subcommand means the wrong operation entirely (e.g.,
    ///    `sort` vs `view` is not "partially correct" — it's fundamentally
    ///    wrong). A correct subcommand keeps the full base score.
    pub fn accuracy_score(&self) -> f64 {
        let base = 0.35 * self.flag_group_recall
            + 0.25 * self.flag_group_precision
            + 0.25 * self.flag_group_jaccard
            + 0.15 * self.positional_order_match;
        if self.subcommand_match {
            base
        } else {
            base * 0.3
        }
    }
}

/// Compare a generated ARGS string against a reference ARGS string.
///
/// Both strings are tokenised by whitespace.  Named flags (starting with `-`)
/// are compared as *sets* (order-insensitive) while the first token is tested
/// separately as the subcommand match.
///
/// Additionally, flag–value groups (each flag paired with its value) are
/// compared, and the relative order of positional arguments is checked.
pub fn compare_commands(generated: &str, reference: &str) -> CompareResult {
    let gen_tokens = normalise_tokens(generated);
    let ref_tokens = normalise_tokens(reference);

    // Exact match after normalisation.
    let exact_match = gen_tokens == ref_tokens;

    // Subcommand match (first non-flag token, with alias support).
    //
    // Some tools use long options as "subcommands" (e.g., STAR: --runMode),
    // while others have old/new naming conventions (e.g., bedtools:
    // intersectBed vs intersect).  We handle these cases by:
    // 1. Finding the first token that is NOT a short flag value (i.e., not
    //    a single `-x` option without its paired subcommand/operand).
    // 2. Checking against a known alias table for equivalent names.
    //
    // IMPORTANT: When BOTH the generated and reference first tokens are flags
    // (start with `-`), the tool has NO subcommand (e.g., fastp). In this
    // case, subcommand_match should be TRUE because there is no subcommand
    // to get wrong — the comparison is handled by flag-level metrics.
    let gen_sub = extract_subcommand(&gen_tokens);
    let ref_sub = extract_subcommand(&ref_tokens);
    let subcommand_match = if ref_sub.is_empty() {
        // No reference subcommand → vacuously correct
        true
    } else if gen_sub.starts_with('-') && ref_sub.starts_with('-') {
        // Both first tokens are flags. Two sub-cases:
        //
        // 1. **Pure flags** (no space in the extracted subcommand, e.g.,
        //    "-i", "--merge"): the tool has NO subcommand (e.g., fastp).
        //    Subcommand comparison is meaningless — flag-level metrics
        //    already capture correctness. → subcommand_match = true.
        //
        // 2. **Compound option-subcommands** (contain a space, e.g.,
        //    "--runMode alignReads", "--runMode genomeGenerate"): the flag
        //    + value together form the effective subcommand. These MUST
        //    match. → compare normally.
        let gen_is_pure_flag = !gen_sub.contains(' ');
        let ref_is_pure_flag = !ref_sub.contains(' ');
        if gen_is_pure_flag && ref_is_pure_flag {
            true
        } else {
            gen_sub == ref_sub || are_alias_subcommands(&gen_sub, &ref_sub)
        }
    } else {
        gen_sub == ref_sub || are_alias_subcommands(&gen_sub, &ref_sub)
    };

    // ── Token-set metrics (backward-compatible, order-insensitive) ───────────
    let gen_set: std::collections::HashSet<&str> = gen_tokens.iter().map(|s| s.as_str()).collect();
    let ref_set: std::collections::HashSet<&str> = ref_tokens.iter().map(|s| s.as_str()).collect();

    let intersection = gen_set.intersection(&ref_set).count() as f64;
    let union = gen_set.union(&ref_set).count() as f64;

    let token_jaccard = if union == 0.0 {
        1.0
    } else {
        intersection / union
    };
    let flag_recall = if ref_set.is_empty() {
        1.0
    } else {
        intersection / ref_set.len() as f64
    };
    let flag_precision = if gen_set.is_empty() {
        // Both empty → perfect match (vacuous truth); only gen empty → 0.
        if ref_set.is_empty() { 1.0 } else { 0.0 }
    } else {
        intersection / gen_set.len() as f64
    };

    // ── Flag-group metrics (order-aware for flag–value pairs) ────────────────
    let (flag_group_recall, flag_group_precision) = compare_flag_groups(generated, reference);
    let flag_group_jaccard = {
        let gen_groups: std::collections::HashSet<String> = parse_flag_groups(generated)
            .into_iter()
            .map(|g| g.join(" "))
            .collect();
        let ref_groups: std::collections::HashSet<String> = parse_flag_groups(reference)
            .into_iter()
            .map(|g| g.join(" "))
            .collect();
        let gi = gen_groups.intersection(&ref_groups).count() as f64;
        let gu = gen_groups.union(&ref_groups).count() as f64;
        if gu == 0.0 { 1.0 } else { gi / gu }
    };

    // ── Positional-order match ────────────────────────────────────────────────
    let positional_order_match = positional_order_match(generated, reference);

    CompareResult {
        exact_match,
        token_jaccard,
        flag_recall,
        flag_precision,
        flag_group_recall,
        flag_group_precision,
        flag_group_jaccard,
        positional_order_match,
        subcommand_match,
    }
}

/// Parse an ARGS string into flag-value groups.
///
/// Returns a sorted vector of "flag groups" where each group is either:
/// - A flag + its value(s): `["-o", "sorted.bam"]`
/// - A standalone flag: `["-b"]`
/// - A positional argument: `["input.bam"]`
///
/// This is used for order-insensitive flag comparison.
pub fn parse_flag_groups(args: &str) -> Vec<Vec<String>> {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let tok = tokens[i];
        if tok.starts_with('-') {
            let mut group = vec![tok.to_string()];
            // Peek ahead: if the next token is not a flag, it's this flag's value.
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                group.push(tokens[i + 1].to_string());
                i += 1;
            }
            groups.push(group);
        } else {
            groups.push(vec![tok.to_string()]);
        }
        i += 1;
    }
    groups
}

/// Order-insensitive flag comparison using flag groups.
///
/// Returns `(recall, precision)` where recall is the fraction of reference
/// flag groups found in the generated output.
pub fn compare_flag_groups(generated: &str, reference: &str) -> (f64, f64) {
    let gen_groups = parse_flag_groups(generated);
    let ref_groups = parse_flag_groups(reference);

    let gen_set: std::collections::HashSet<String> =
        gen_groups.iter().map(|g| g.join(" ")).collect();
    let ref_set: std::collections::HashSet<String> =
        ref_groups.iter().map(|g| g.join(" ")).collect();

    let intersection = gen_set.intersection(&ref_set).count() as f64;
    let recall = if ref_set.is_empty() {
        1.0
    } else {
        intersection / ref_set.len() as f64
    };
    let precision = if gen_set.is_empty() {
        if ref_set.is_empty() { 1.0 } else { 0.0 }
    } else {
        intersection / gen_set.len() as f64
    };

    (recall, precision)
}

/// Check whether the positional (non-flag) tokens appear in the same relative
/// order in `generated` vs `reference`.
///
/// Positional tokens are those that do NOT start with `-`.  If either side
/// produces no positional tokens the score is `1.0` (vacuous match).
/// If all reference positional tokens appear in `generated` as a left-to-right
/// subsequence the score is `1.0`; otherwise `0.0`.
///
/// This penalises, for example, `input.bam output.bam` vs `output.bam input.bam`
/// for tools that interpret positional arguments by position.
pub fn positional_order_match(generated: &str, reference: &str) -> f64 {
    let ref_pos: Vec<&str> = reference
        .split_whitespace()
        .filter(|t| !t.starts_with('-'))
        .collect();
    let gen_pos: Vec<&str> = generated
        .split_whitespace()
        .filter(|t| !t.starts_with('-'))
        .collect();

    if ref_pos.is_empty() || gen_pos.is_empty() {
        return 1.0;
    }

    // Greedy left-to-right subsequence search: find each reference positional
    // in generated in order.  If we can match all of them, order is preserved.
    let mut matched = 0usize;
    let mut search_start = 0;
    for ref_tok in &ref_pos {
        if let Some(idx) = gen_pos[search_start..].iter().position(|g| g == ref_tok) {
            matched += 1;
            search_start += idx + 1;
        }
    }

    if matched == ref_pos.len() { 1.0 } else { 0.0 }
}

/// Extract the effective subcommand from a token list.
///
/// For most bioinformatics tools, the subcommand is the first positional
/// token (e.g., "sort", "view", "mem").  However, some tools like STAR
/// use long options as their primary "subcommand" (e.g., `--runMode`).
/// For STAR specifically, the reference always starts with `--runMode`,
/// so we need to recognise long-option-style subcommands.
///
/// Strategy:
/// - If the first token starts with `--` and is a known option-subcommand
///   (e.g., `--runMode`), treat the flag **plus its value** as the
///   subcommand (e.g. `"--runMode alignReads"`).  This distinguishes
///   different STAR operations that share the same flag but differ in value.
/// - Otherwise, find the first token that is NOT a short flag (`-x` but
///   NOT `--long`) — this skips cases where the model outputs `-t 8 ...`
///   without any subcommand, which is a genuine error.
fn extract_subcommand(tokens: &[String]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let first = tokens[0].as_str();

    // STAR-style: --runMode <value> is the effective subcommand.
    // We must include the value to distinguish --runMode alignReads from
    // --runMode genomeGenerate.
    if first == "--runMode" || first == "--genomeDir" || first == "--readFilesIn" {
        if tokens.len() > 1 {
            return format!("{} {}", first, tokens[1]);
        }
        return first.to_string();
    }

    // Standard case: first token is the subcommand (not a short flag)
    // Short flags like "-t", "-x" indicate the model omitted the subcommand
    // — return the first token as-is (it will fail the match check).
    // Long options like "--runMode" are handled above.
    if first.starts_with("--") {
        // Other long options as first token — treat as subcommand attempt
        return first.to_string();
    }

    // For the standard case, the first token should be the subcommand
    // If it starts with a single `-`, it's a flag, not a subcommand —
    // but we still return it because it IS what the model generated as
    // the first token and it will correctly fail the subcommand match
    // against a proper subcommand like "mem" or "sort".
    first.to_string()
}

/// Check whether two subcommand names are semantically equivalent aliases.
///
/// bedtools has two naming conventions:
/// - Old: `intersectBed`, `sortBed`, `mergeBed`, `closestBed`, `subtractBed`, `genomecovBed`
/// - New: `intersect`, `sort`, `merge`, `closest`, `subtract`, `genomecov`
///
/// Both forms invoke the same operation and produce identical output.
fn are_alias_subcommands(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }

    // bedtools old-style → new-style aliases
    const BEDTOOLS_ALIASES: &[(&str, &str)] = &[
        ("intersectBed", "intersect"),
        ("sortBed", "sort"),
        ("mergeBed", "merge"),
        ("closestBed", "closest"),
        ("subtractBed", "subtract"),
        ("genomecovBed", "genomecov"),
        ("coverageBed", "coverage"),
        ("flankBed", "flank"),
        ("slopBed", "slop"),
        ("shuffleBed", "shuffle"),
        ("complementBed", "complement"),
        ("windowBed", "window"),
        ("bedToBam", "bedtobam"),
        ("bamToBed", "bamtobed"),
    ];

    for (old, new) in BEDTOOLS_ALIASES {
        if (a == *old && b == *new) || (a == *new && b == *old) {
            return true;
        }
    }

    false
}

/// Normalise an ARGS string into a canonical token list.
fn normalise_tokens(args: &str) -> Vec<String> {
    args.split_whitespace().map(|s| s.to_string()).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Semantic normalisation layer
// ═══════════════════════════════════════════════════════════════════════════════
//
// The core problem: models generate correct *operations* but use different file
// names, add shell redirects (`> file`), or pipe to downstream tools.  Raw token
// comparison penalises all of these as "wrong", capping accuracy at ~70%.
//
// Solution: before comparing, normalise both the generated and reference args
// so that only the *operational structure* (subcommand + flags + their values)
// is compared.  File paths are replaced with typed placeholders, redirects and
// pipes are stripped, and numeric values are compared with tolerance.

/// Common bioinformatics file extensions used to detect positional file paths.
const FILE_EXTENSIONS: &[&str] = &[
    ".bam",
    ".sam",
    ".cram",
    ".fastq",
    ".fq",
    ".fq.gz",
    ".fastq.gz",
    ".fa",
    ".fasta",
    ".fna",
    ".fa.gz",
    ".fasta.gz",
    ".vcf",
    ".vcf.gz",
    ".bcf",
    ".bed",
    ".gff",
    ".gtf",
    ".gff3",
    ".sam.gz",
    ".bam.bai",
    ".crai",
    ".fai",
    ".dict",
    ".txt",
    ".tsv",
    ".csv",
    ".log",
    ".out",
    ".stats",
    ".sra",
    ".sff",
    ".ab1",
    ".bam",
    ".bai",
];

/// Detect whether a token looks like a file path (contains /, ., or has a
/// known bioinformatics extension).
fn is_file_path_token(token: &str) -> bool {
    // Absolute or relative paths
    if token.starts_with('/') || token.starts_with("./") || token.starts_with("../") {
        return true;
    }
    // Home directory
    if token.starts_with("~/") {
        return true;
    }
    // Known extensions (case-insensitive)
    let lower = token.to_lowercase();
    for ext in FILE_EXTENSIONS {
        if lower.ends_with(ext) {
            return true;
        }
    }
    // Contains a dot followed by 1-4 chars at end (e.g., "reads.fq")
    if let Some(dot_pos) = token.rfind('.') {
        let after_dot = &token[dot_pos + 1..];
        // Must be 1-4 chars, alphabetic or alphanumeric (extensions like .gz, .bam)
        if (1..=4).contains(&after_dot.len()) && after_dot.chars().all(|c| c.is_ascii_alphabetic())
        {
            return true;
        }
    }
    false
}

/// Semantic type of a file path token, used for placeholder assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathRole {
    /// Input file (appears without a preceding -o/--output flag)
    Input,
    /// Output file (appears after -o/--output or similar)
    Output,
    /// Reference/index path (appears after -R/--reference, --genomeDir, etc.)
    Reference,
    /// Unknown path role
    Other,
}

/// Detect the role of a file path token based on the preceding flag.
fn detect_path_role(tokens: &[&str], idx: usize) -> PathRole {
    if idx == 0 {
        return PathRole::Other;
    }
    let prev = tokens[idx - 1];
    // Output flags
    if prev == "-o"
        || prev == "--output"
        || prev == "--out"
        || prev == "-O"
        || prev == "--output-file"
        || prev == "--outfile"
    {
        return PathRole::Output;
    }
    // Reference genome flags
    if prev == "-R"
        || prev == "--reference"
        || prev == "--genomeDir"
        || prev == "--genome"
        || prev == "--ref"
        || prev == "-r"
        || prev == "--reference-genome"
        || prev == "-f"
        || prev == "--fasta-ref"
    {
        return PathRole::Reference;
    }
    PathRole::Input
}

/// Strip shell redirects (`> file`, `2> file`, `>> file`) and pipes (`| cmd`)
/// from a command string, returning only the primary command portion.
fn strip_redirects_and_pipes(args: &str) -> String {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let mut result: Vec<&str> = Vec::new();
    let mut skip_next = false;

    for (i, tok) in tokens.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        // Pipe: stop processing entirely
        if *tok == "|" {
            break;
        }
        // Redirect operators
        if *tok == ">"
            || *tok == ">>"
            || *tok == "2>"
            || *tok == "&>"
            || tok.starts_with('>')
            || tok.starts_with("2>")
            || tok.starts_with("&>")
        {
            // If the redirect is like ">file" (no space), skip it entirely
            if tok.len() > 1 && !tok.starts_with('-') {
                continue;
            }
            // Otherwise skip the next token too (the filename)
            skip_next = true;
            continue;
        }
        // If this token is the filename after a redirect (shouldn't happen due to
        // skip_next, but just in case), check if previous was a redirect
        if i > 0 {
            let prev = tokens[i - 1];
            if prev == ">" || prev == ">>" || prev == "2>" || prev == "&>" {
                continue;
            }
        }
        result.push(tok);
    }

    result.join(" ")
}

/// Semantic-normalise an ARGS string for comparison.
///
/// This function:
/// 1. Strips shell redirects (`>`, `>>`, `2>`) and pipes (`|`).
/// 2. Replaces file paths with typed placeholders (`<INPUT_N>`, `<OUTPUT_N>`,
///    `<REF_N>`, `<PATH_N>`) so that two commands with different file names
///    but identical operational structure compare as equal.
/// 3. Preserves flags, subcommands, and numeric values exactly.
///
/// Example:
///   `sort -@ 4 -o sorted.bam input.bam` → `sort -@ 4 -o <OUTPUT_1> <INPUT_1>`
///   `sort -@ 4 -o out.bam reads.bam`    → `sort -@ 4 -o <OUTPUT_1> <INPUT_1>`
pub fn semantic_normalise(args: &str) -> String {
    let stripped = strip_redirects_and_pipes(args);
    let tokens: Vec<&str> = stripped.split_whitespace().collect();
    let mut result: Vec<String> = Vec::new();
    let mut input_count: usize = 0;
    let mut output_count: usize = 0;
    let mut ref_count: usize = 0;
    let mut other_count: usize = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if is_file_path_token(tok) {
            let role = detect_path_role(&tokens, i);
            let placeholder = match role {
                PathRole::Input => {
                    input_count += 1;
                    format!("<INPUT_{}>", input_count)
                }
                PathRole::Output => {
                    output_count += 1;
                    format!("<OUTPUT_{}>", output_count)
                }
                PathRole::Reference => {
                    ref_count += 1;
                    format!("<REF_{}>", ref_count)
                }
                PathRole::Other => {
                    other_count += 1;
                    format!("<PATH_{}>", other_count)
                }
            };
            result.push(placeholder);
        } else {
            result.push(tok.to_string());
        }
    }

    result.join(" ")
}

/// Compare commands with semantic normalisation.
///
/// This is the primary comparison function for benchmark accuracy scoring.
/// It first normalises both the generated and reference args using
/// `semantic_normalise`, then applies the standard `compare_commands` logic.
///
/// The result includes both raw and normalised metrics:
/// - `exact_match`: based on normalised comparison (accounts for file name
///   differences and redirect/pipe differences).
/// - `flag_group_*`: based on normalised comparison (focuses on operational
///   structure).
/// - `token_jaccard`, `flag_recall`, `flag_precision`: based on raw comparison
///   (preserved for backward compatibility).
pub fn compare_commands_semantic(generated: &str, reference: &str) -> CompareResult {
    let gen_norm = semantic_normalise(generated);
    let ref_norm = semantic_normalise(reference);

    // Compute normalised metrics (primary)
    let norm_result = compare_commands(&gen_norm, &ref_norm);

    // Compute raw metrics (for backward compatibility)
    let raw_result = compare_commands(generated, reference);

    CompareResult {
        // Use normalised metrics for the important ones
        exact_match: norm_result.exact_match,
        flag_group_recall: norm_result.flag_group_recall,
        flag_group_precision: norm_result.flag_group_precision,
        flag_group_jaccard: norm_result.flag_group_jaccard,
        positional_order_match: norm_result.positional_order_match,
        subcommand_match: norm_result.subcommand_match,
        // Keep raw token-set metrics for backward compatibility
        token_jaccard: raw_result.token_jaccard,
        flag_recall: raw_result.flag_recall,
        flag_precision: raw_result.flag_precision,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let r = compare_commands("sort -o out.bam in.bam", "sort -o out.bam in.bam");
        assert!(r.exact_match);
        assert_eq!(r.token_jaccard, 1.0);
        assert_eq!(r.flag_recall, 1.0);
        assert_eq!(r.flag_precision, 1.0);
        assert_eq!(r.flag_group_recall, 1.0);
        assert_eq!(r.flag_group_precision, 1.0);
        assert_eq!(r.flag_group_jaccard, 1.0);
        assert_eq!(r.positional_order_match, 1.0);
        assert!(r.subcommand_match);
    }

    #[test]
    fn test_different_flag_order_same_tokens() {
        // Flags in different order → not exact match but high similarity.
        let r = compare_commands(
            "sort -o sorted.bam -@ 4 input.bam",
            "sort -@ 4 -o sorted.bam input.bam",
        );
        assert!(!r.exact_match);
        assert_eq!(r.token_jaccard, 1.0); // same token sets
        assert_eq!(r.flag_recall, 1.0);
        assert_eq!(r.flag_precision, 1.0);
        // Flag-group metrics should also be perfect since groups match.
        assert_eq!(r.flag_group_recall, 1.0);
        assert_eq!(r.flag_group_precision, 1.0);
        assert_eq!(r.flag_group_jaccard, 1.0);
        assert!(r.subcommand_match);
    }

    /// Core regression: `--threads 8` and `8 --threads` have the same raw
    /// token set but differ in flag-group semantics.
    #[test]
    fn test_flag_value_swap_penalised_by_group_metrics() {
        // Reference: correct form   `cmd --threads 8 input.txt`
        // Generated: value before   `cmd 8 --threads input.txt`
        let r = compare_commands("cmd 8 --threads input.txt", "cmd --threads 8 input.txt");
        // Token-set metrics are the same (backward compat).
        assert_eq!(r.token_jaccard, 1.0);
        assert_eq!(r.flag_recall, 1.0);
        assert_eq!(r.flag_precision, 1.0);
        // But flag-group metrics must differ.
        // Reference groups: ["cmd"], ["--threads", "8"], ["input.txt"]
        // Generated groups: ["cmd"], ["8"], ["--threads"], ["input.txt"]
        assert!(
            r.flag_group_recall < 1.0,
            "flag_group_recall should be < 1.0 when flag-value is swapped"
        );
        assert!(
            r.flag_group_jaccard < 1.0,
            "flag_group_jaccard should be < 1.0 when flag-value is swapped"
        );
        // accuracy_score is based on group metrics, so it should be penalised.
        assert!(
            r.accuracy_score() < 1.0,
            "accuracy_score should be < 1.0 for flag-value swap"
        );
    }

    /// `--8 threads` is a completely garbled form; even token-set overlap is poor.
    #[test]
    fn test_garbled_flag_value_penalised() {
        let r = compare_commands("cmd --8 threads input.txt", "cmd --threads 8 input.txt");
        assert!(!r.exact_match);
        assert!(r.token_jaccard < 1.0); // "--8" and "threads" don't match "--threads" and "8"
        assert!(r.flag_group_recall < 1.0);
        assert!(r.accuracy_score() < 1.0);
    }

    #[test]
    fn test_missing_flags() {
        let r = compare_commands("sort input.bam", "sort -@ 4 -o sorted.bam input.bam");
        assert!(!r.exact_match);
        assert!(r.flag_recall < 1.0); // missing -@ 4 -o sorted.bam
        assert!(r.flag_group_recall < 1.0);
        assert!(r.subcommand_match);
    }

    #[test]
    fn test_extra_flags() {
        let r = compare_commands(
            "sort -@ 4 -o sorted.bam --extra input.bam",
            "sort -@ 4 -o sorted.bam input.bam",
        );
        assert!(!r.exact_match);
        assert_eq!(r.flag_recall, 1.0); // all reference tokens present (raw token-set level)
        assert!(r.flag_precision < 1.0); // has extra --extra token
        // Flag-group level: "--extra input.bam" consumes the positional "input.bam" as a value,
        // so the reference group ["input.bam"] is missing from generated groups.
        assert!(r.flag_group_recall < 1.0); // reference positional consumed as flag value
        assert!(r.flag_group_precision < 1.0); // extra group ["--extra input.bam"] present
    }

    #[test]
    fn test_wrong_subcommand() {
        let r = compare_commands("view -o out.bam in.bam", "sort -o out.bam in.bam");
        assert!(!r.subcommand_match);
    }

    #[test]
    fn test_star_subcommand_match() {
        // STAR uses --runMode as its "subcommand" — should match correctly
        let r = compare_commands(
            "--runMode genomeGenerate --genomeDir /path/to/star_index",
            "--runMode genomeGenerate --genomeDir /path/to/star_index",
        );
        assert!(r.subcommand_match);
    }

    #[test]
    fn test_star_wrong_subcommand() {
        // Different STAR operations
        let r = compare_commands(
            "--runMode alignReads --genomeDir /path/to/star_index",
            "--runMode genomeGenerate --genomeDir /path/to/star_index",
        );
        assert!(!r.subcommand_match);
    }

    #[test]
    fn test_bedtools_alias_subcommand() {
        // bedtools old-style vs new-style should match
        let r = compare_commands(
            "intersect -a query.bed -b features.bed -wa",
            "intersect -a query.bed -b features.bed -wa",
        );
        assert!(r.subcommand_match);

        // Old-style intersectBed should match new-style intersect
        let r2 = compare_commands(
            "intersectBed -a query.bed -b features.bed -wa",
            "intersect -a query.bed -b features.bed -wa",
        );
        assert!(r2.subcommand_match);

        // mergeBed → merge
        let r3 = compare_commands("merge -i input.bed", "mergeBed -i input.bed");
        assert!(r3.subcommand_match);
    }

    #[test]
    fn test_flag_as_subcommand_should_fail() {
        // Model outputting "-t 8" instead of "mem" should fail subcommand match
        let r = compare_commands("-t 8 ref.fa R1.fq R2.fq", "mem -t 8 ref.fa R1.fq R2.fq");
        assert!(!r.subcommand_match);
    }

    #[test]
    fn test_empty_strings() {
        let r = compare_commands("", "");
        assert!(r.exact_match);
        assert_eq!(r.token_jaccard, 1.0);
        assert_eq!(r.flag_group_jaccard, 1.0);
        assert_eq!(r.positional_order_match, 1.0);
    }

    #[test]
    fn test_accuracy_score_perfect() {
        let r = compare_commands("sort -o out.bam in.bam", "sort -o out.bam in.bam");
        assert!((r.accuracy_score() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_accuracy_score_partial() {
        let r = compare_commands("sort input.bam", "sort -@ 4 -o sorted.bam input.bam");
        assert!(r.accuracy_score() > 0.0);
        assert!(r.accuracy_score() < 1.0);
    }

    #[test]
    fn test_parse_flag_groups() {
        let groups = parse_flag_groups("sort -@ 4 -o sorted.bam input.bam");
        assert_eq!(groups.len(), 4);
        assert_eq!(groups[0], vec!["sort"]);
        assert_eq!(groups[1], vec!["-@", "4"]);
        assert_eq!(groups[2], vec!["-o", "sorted.bam"]);
        assert_eq!(groups[3], vec!["input.bam"]);
    }

    #[test]
    fn test_compare_flag_groups_order_insensitive() {
        let (recall, precision) = compare_flag_groups(
            "sort -o sorted.bam -@ 4 input.bam",
            "sort -@ 4 -o sorted.bam input.bam",
        );
        assert_eq!(recall, 1.0);
        assert_eq!(precision, 1.0);
    }

    #[test]
    fn test_compare_flag_groups_missing() {
        let (recall, _precision) =
            compare_flag_groups("sort input.bam", "sort -@ 4 -o sorted.bam input.bam");
        assert!(recall < 1.0);
    }

    #[test]
    fn test_whitespace_normalisation() {
        let r = compare_commands("sort  -o  out.bam   in.bam", "sort -o out.bam in.bam");
        assert!(r.exact_match);
    }

    // ── positional_order_match tests ─────────────────────────────────────────

    #[test]
    fn test_positional_order_match_correct_order() {
        // input.bam comes before output.bam in both → 1.0
        assert_eq!(
            positional_order_match("sort -o out.bam input.bam", "sort -o out.bam input.bam"),
            1.0
        );
    }

    #[test]
    fn test_positional_order_match_swapped_order() {
        // Reference positionals: ["sort", "input.bam"] (ignoring -o out.bam as flag group)
        // Generated: swaps input.bam and another positional relative to reference.
        // Simple case: two positionals in reverse order.
        assert_eq!(
            positional_order_match("sort out.bam input.bam", "sort input.bam out.bam"),
            0.0
        );
    }

    #[test]
    fn test_positional_order_match_no_positionals() {
        // Both have only flags → vacuous 1.0
        assert_eq!(positional_order_match("-a -b -c", "-a -b -c"), 1.0);
    }

    #[test]
    fn test_positional_order_match_different_positionals() {
        // Generated has a positional not present in reference → 0.0 (can't verify order)
        assert_eq!(
            positional_order_match("sort other.bam", "sort input.bam"),
            0.0
        );
    }

    // ── compare_commands flag_group_jaccard tests ─────────────────────────────

    #[test]
    fn test_flag_group_jaccard_perfect() {
        let r = compare_commands("cmd --out foo.txt input.bam", "cmd --out foo.txt input.bam");
        assert_eq!(r.flag_group_jaccard, 1.0);
    }

    #[test]
    fn test_flag_group_jaccard_partial() {
        // Generated is missing one flag-group vs reference.
        let r = compare_commands("cmd input.bam", "cmd --out foo.txt input.bam");
        assert!(r.flag_group_jaccard < 1.0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Semantic normalisation tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_semantic_normalise_file_paths() {
        // Different file names should produce the same normalised form
        let a = semantic_normalise("sort -@ 4 -o sorted.bam input.bam");
        let b = semantic_normalise("sort -@ 4 -o out.bam reads.bam");
        assert_eq!(a, b);
        assert!(a.contains("<OUTPUT_1>"));
        assert!(a.contains("<INPUT_1>"));
    }

    #[test]
    fn test_semantic_normalise_preserves_flags() {
        let n = semantic_normalise("sort -@ 4 -o sorted.bam input.bam");
        assert!(n.contains("sort"));
        assert!(n.contains("-@"));
        assert!(n.contains("4"));
        assert!(n.contains("-o"));
    }

    #[test]
    fn test_semantic_normalise_strips_redirect() {
        let n = semantic_normalise("stats input.vcf.gz > stats.txt");
        // The redirect operator and its target should be gone.
        // Note: <INPUT_1> contains '>' as part of the placeholder, so we check
        // that there's no standalone ">" token.
        let tokens: Vec<&str> = n.split_whitespace().collect();
        assert!(
            !tokens.contains(&">"),
            "redirect operator should be stripped"
        );
        assert!(
            !n.contains("stats.txt"),
            "redirect target should be stripped"
        );
        assert!(n.contains("stats"));
        assert!(n.contains("<INPUT_1>"));
    }

    #[test]
    fn test_semantic_normalise_strips_pipe() {
        let n = semantic_normalise("mpileup -f ref.fa input.bam | call -Ov -o out.vcf");
        assert!(!n.contains("|"));
        assert!(!n.contains("call"));
        // Only the first command should remain
        assert!(n.contains("mpileup"));
    }

    #[test]
    fn test_semantic_normalise_reference_genome() {
        let n = semantic_normalise("mpileup -f ref.fa input.bam");
        assert!(
            n.contains("<REF_1>"),
            "expected <REF_1> for -f value, got: {n}"
        );
        assert!(
            n.contains("<INPUT_1>"),
            "expected <INPUT_1> for positional, got: {n}"
        );
    }

    #[test]
    fn test_semantic_compare_different_filenames() {
        // Same operation, different file names → should have high accuracy
        let r = compare_commands_semantic(
            "sort -@ 4 -o sorted.bam input.bam",
            "sort -@ 4 -o out.bam reads.bam",
        );
        assert!(
            r.exact_match,
            "different filenames should be semantically equal"
        );
        assert_eq!(r.flag_group_recall, 1.0);
        assert_eq!(r.flag_group_precision, 1.0);
        assert!((r.accuracy_score() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_semantic_compare_redirect_difference() {
        // One has redirect, the other doesn't → operational structure same
        let r = compare_commands_semantic("stats input.vcf.gz > stats.txt", "stats input.vcf.gz");
        assert!(
            r.exact_match,
            "redirect-only difference should be semantically equal"
        );
        assert_eq!(r.flag_group_recall, 1.0);
    }

    #[test]
    fn test_semantic_compare_pipe_difference() {
        // One has pipe, the other doesn't → compare only primary command
        let r = compare_commands_semantic(
            "mpileup -f ref.fa input.bam | call -Ov -o out.vcf",
            "mpileup -f ref.fa input.bam",
        );
        assert!(
            r.exact_match,
            "pipe-only difference should match on primary command"
        );
        assert_eq!(r.flag_group_recall, 1.0);
    }

    #[test]
    fn test_semantic_compare_wrong_subcommand_still_penalised() {
        // Different subcommand should still be penalised even with normalisation
        let r = compare_commands_semantic("view -o out.bam input.bam", "sort -o out.bam input.bam");
        assert!(!r.subcommand_match);
        assert!(r.accuracy_score() < 0.5);
    }

    #[test]
    fn test_semantic_compare_missing_flags_still_penalised() {
        // Missing flags should still be penalised even with normalisation
        let r = compare_commands_semantic("sort input.bam", "sort -@ 4 -o sorted.bam input.bam");
        assert!(!r.exact_match);
        assert!(r.flag_group_recall < 1.0);
        assert!(r.accuracy_score() < 1.0);
    }

    #[test]
    fn test_is_file_path_token() {
        assert!(is_file_path_token("input.bam"));
        assert!(is_file_path_token("reads.fastq.gz"));
        assert!(is_file_path_token("/path/to/file.vcf"));
        assert!(is_file_path_token("./relative.sam"));
        assert!(is_file_path_token("~/home/file.fa"));
        assert!(!is_file_path_token("sort"));
        assert!(!is_file_path_token("-o"));
        assert!(!is_file_path_token("4"));
        assert!(!is_file_path_token("genomeGenerate"));
    }

    #[test]
    fn test_strip_redirects_and_pipes() {
        assert_eq!(
            strip_redirects_and_pipes("cmd input.bam > output.txt"),
            "cmd input.bam"
        );
        assert_eq!(
            strip_redirects_and_pipes("cmd input.bam | other_cmd"),
            "cmd input.bam"
        );
        assert_eq!(
            strip_redirects_and_pipes("cmd input.bam 2> log.txt"),
            "cmd input.bam"
        );
        assert_eq!(
            strip_redirects_and_pipes("cmd input.bam >> append.txt"),
            "cmd input.bam"
        );
        assert_eq!(strip_redirects_and_pipes("cmd input.bam"), "cmd input.bam");
    }
}
