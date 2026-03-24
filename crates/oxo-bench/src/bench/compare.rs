//! Command comparison engine with flag-order awareness.
//!
//! Compares a generated ARGS string against a known-good reference and produces
//! a set of metrics that capture both strict and relaxed notions of "correct".

/// Result of comparing a generated command against a reference.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompareResult {
    /// Exact string match after whitespace normalisation.
    pub exact_match: bool,
    /// Jaccard similarity of the two token sets (order-insensitive).
    pub token_jaccard: f64,
    /// Fraction of reference tokens found in the generated output.
    pub flag_recall: f64,
    /// Fraction of generated tokens that also appear in the reference.
    pub flag_precision: f64,
    /// Whether the first token (subcommand) matches.
    pub subcommand_match: bool,
}

impl CompareResult {
    /// A simple composite accuracy score in \[0, 1\].
    ///
    /// Weighted combination: 40% flag_recall + 30% flag_precision + 20% Jaccard +
    /// 10% subcommand bonus.
    pub fn accuracy_score(&self) -> f64 {
        let sub_bonus = if self.subcommand_match { 1.0 } else { 0.0 };
        0.40 * self.flag_recall
            + 0.30 * self.flag_precision
            + 0.20 * self.token_jaccard
            + 0.10 * sub_bonus
    }
}

/// Compare a generated ARGS string against a reference ARGS string.
///
/// Both strings are tokenised by whitespace.  Named flags (starting with `-`)
/// are compared as *sets* (order-insensitive) while the first token is tested
/// separately as the subcommand match.
pub fn compare_commands(generated: &str, reference: &str) -> CompareResult {
    let gen_tokens = normalise_tokens(generated);
    let ref_tokens = normalise_tokens(reference);

    // Exact match after normalisation.
    let exact_match = gen_tokens == ref_tokens;

    // Subcommand match (first non-flag token).
    let gen_sub = gen_tokens.first().map(String::as_str).unwrap_or("");
    let ref_sub = ref_tokens.first().map(String::as_str).unwrap_or("");
    let subcommand_match = !ref_sub.is_empty() && gen_sub == ref_sub;

    // Token sets for Jaccard / precision / recall.
    // Edge case: when both sets are empty (both args are empty), we treat that
    // as a perfect match (Jaccard = 1.0) because the generator correctly
    // produced no flags when none were expected.
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

    CompareResult {
        exact_match,
        token_jaccard,
        flag_recall,
        flag_precision,
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
        assert!(r.subcommand_match);
    }

    #[test]
    fn test_missing_flags() {
        let r = compare_commands("sort input.bam", "sort -@ 4 -o sorted.bam input.bam");
        assert!(!r.exact_match);
        assert!(r.flag_recall < 1.0); // missing -@ 4 -o sorted.bam
        assert!(r.subcommand_match);
    }

    #[test]
    fn test_extra_flags() {
        let r = compare_commands(
            "sort -@ 4 -o sorted.bam --extra input.bam",
            "sort -@ 4 -o sorted.bam input.bam",
        );
        assert!(!r.exact_match);
        assert_eq!(r.flag_recall, 1.0); // all reference flags present
        assert!(r.flag_precision < 1.0); // has extra --extra
    }

    #[test]
    fn test_wrong_subcommand() {
        let r = compare_commands("view -o out.bam in.bam", "sort -o out.bam in.bam");
        assert!(!r.subcommand_match);
    }

    #[test]
    fn test_empty_strings() {
        let r = compare_commands("", "");
        assert!(r.exact_match);
        assert_eq!(r.token_jaccard, 1.0);
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
}
