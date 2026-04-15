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
    /// Weighted combination using flag-group-aware metrics to avoid crediting
    /// semantically wrong flag–value swaps:
    ///
    /// - 40% `flag_group_recall`
    /// - 25% `flag_group_precision`
    /// - 20% `flag_group_jaccard`
    /// - 10% `subcommand_match` bonus
    /// - 5%  `positional_order_match`
    pub fn accuracy_score(&self) -> f64 {
        let sub_bonus = if self.subcommand_match { 1.0 } else { 0.0 };
        0.40 * self.flag_group_recall
            + 0.25 * self.flag_group_precision
            + 0.20 * self.flag_group_jaccard
            + 0.10 * sub_bonus
            + 0.05 * self.positional_order_match
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
    let gen_sub = extract_subcommand(&gen_tokens);
    let ref_sub = extract_subcommand(&ref_tokens);
    let subcommand_match =
        !ref_sub.is_empty() && (gen_sub == ref_sub || are_alias_subcommands(gen_sub, ref_sub));

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
///   (e.g., `--runMode`), treat it as the subcommand.
/// - Otherwise, find the first token that is NOT a short flag (`-x` but
///   NOT `--long`) — this skips cases where the model outputs `-t 8 ...`
///   without any subcommand, which is a genuine error.
fn extract_subcommand(tokens: &[String]) -> &str {
    if tokens.is_empty() {
        return "";
    }

    let first = tokens[0].as_str();

    // STAR-style: --runMode is the effective subcommand
    if first == "--runMode" || first == "--genomeDir" || first == "--readFilesIn" {
        return first;
    }

    // Standard case: first token is the subcommand (not a short flag)
    // Short flags like "-t", "-x" indicate the model omitted the subcommand
    // — return the first token as-is (it will fail the match check).
    // Long options like "--runMode" are handled above.
    if first.starts_with("--") {
        // Other long options as first token — treat as subcommand attempt
        return first;
    }

    // For the standard case, the first token should be the subcommand
    // If it starts with a single `-`, it's a flag, not a subcommand —
    // but we still return it because it IS what the model generated as
    // the first token and it will correctly fail the subcommand match
    // against a proper subcommand like "mem" or "sort".
    first
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
}
