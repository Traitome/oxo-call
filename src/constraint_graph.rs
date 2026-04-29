//! Constraint Graph Module
//!
//! Provides validation of generated arguments against documentation constraints
//! to prevent flag hallucination and enforce parameter rules.

#![allow(dead_code)]

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// A single constraint rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// Parameter is required
    Required {
        flag: String,
        condition: Option<String>,
    },
    /// One of the parameters must be present
    OneOf { flags: Vec<String> },
    /// Parameter requires specific format
    Format { flag: String, pattern: String },
}

impl Constraint {
    /// Check if constraint is satisfied
    pub fn is_satisfied(&self, args: &[(String, Option<String>)]) -> bool {
        match self {
            Constraint::Required { flag, condition } => {
                // Check condition first
                if let Some(cond) = condition
                    && !self.check_condition(args, cond)
                {
                    return true; // Condition not met, constraint doesn't apply
                }
                args.iter().any(|(f, _)| f == flag)
            }
            Constraint::OneOf { flags } => flags.iter().any(|f| args.iter().any(|(af, _)| af == f)),
            Constraint::Format { flag, pattern: _ } => {
                if let Some((_, Some(val))) = args.iter().find(|(f, _)| f == flag) {
                    // Simple pattern check (could be enhanced with regex)
                    !val.is_empty()
                } else {
                    true // Flag not present, format check passes
                }
            }
        }
    }

    fn check_condition(&self, args: &[(String, Option<String>)], condition: &str) -> bool {
        // Simple condition parsing: "flag=value" or "flag_exists"
        if condition.contains('=') {
            let parts: Vec<_> = condition.splitn(2, '=').collect();
            if parts.len() == 2 {
                let flag = parts[0].trim();
                let expected = parts[1].trim();
                return args
                    .iter()
                    .any(|(f, v)| f == flag && v.as_ref().map(|s| s == expected).unwrap_or(false));
            }
        }
        // Just check if flag exists
        args.iter().any(|(f, _)| f == condition)
    }
}

/// Validation violation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    /// Flag not found in documentation
    HallucinatedFlag {
        flag: String,
        suggestion: Option<String>,
    },
    /// Missing required parameter
    MissingRequired {
        flag: String,
        context: Option<String>,
    },
    /// Mutually exclusive flags both present
    MutuallyExclusive { flag1: String, flag2: String },
    /// Missing dependency
    MissingDependency { flag: String, requires: String },
    /// Invalid format
    InvalidFormat {
        flag: String,
        expected: String,
        got: Option<String>,
    },
}

impl Violation {
    /// Get a human-readable description of the violation
    pub fn description(&self) -> String {
        match self {
            Violation::HallucinatedFlag { flag, suggestion } => {
                if let Some(s) = suggestion {
                    format!(
                        "Flag '{}' not found in documentation. Did you mean '{}'?",
                        flag, s
                    )
                } else {
                    format!("Flag '{}' not found in documentation", flag)
                }
            }
            Violation::MissingRequired { flag, context } => {
                if let Some(ctx) = context {
                    format!("Required parameter '{}' missing (context: {})", flag, ctx)
                } else {
                    format!("Required parameter '{}' missing", flag)
                }
            }
            Violation::MutuallyExclusive { flag1, flag2 } => {
                format!("Flags '{}' and '{}' are mutually exclusive", flag1, flag2)
            }
            Violation::MissingDependency { flag, requires } => {
                format!("Flag '{}' requires flag '{}' to be present", flag, requires)
            }
            Violation::InvalidFormat {
                flag,
                expected,
                got,
            } => {
                if let Some(g) = got {
                    format!("Flag '{}' expects format '{}', got '{}'", flag, expected, g)
                } else {
                    format!("Flag '{}' expects format '{}'", flag, expected)
                }
            }
        }
    }
}

/// Validation report
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub violations: Vec<Violation>,
}

impl ValidationReport {
    /// Create a new valid report
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            violations: Vec::new(),
        }
    }

    /// Create a new invalid report with violations
    pub fn invalid(violations: Vec<Violation>) -> Self {
        Self {
            is_valid: false,
            violations,
        }
    }

    /// Add a violation
    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
        self.is_valid = false;
    }

    /// Merge another report
    pub fn merge(&mut self, other: ValidationReport) {
        self.violations.extend(other.violations);
        self.is_valid = self.is_valid && other.is_valid;
    }
}

/// Constraint Graph for parameter validation
#[derive(Debug, Clone, Default)]
pub struct ConstraintGraph {
    /// Valid flags (whitelist for hallucination detection)
    pub valid_flags: HashSet<String>,
    /// Flag aliases (e.g., -o and --output)
    pub flag_aliases: HashMap<String, Vec<String>>,
    /// Subcommand-specific flags
    pub subcommand_flags: HashMap<String, HashSet<String>>,
    /// Required parameters
    pub required: Vec<Constraint>,
    /// Mutually exclusive flag groups
    pub mutually_exclusive: Vec<Vec<String>>,
    /// Dependency rules (flag -> requires)
    pub dependencies: HashMap<String, Vec<String>>,
    /// Format rules
    pub format_rules: Vec<Constraint>,
}

impl ConstraintGraph {
    /// Create a new empty constraint graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a valid flag
    pub fn add_flag(&mut self, flag: impl Into<String>) {
        self.valid_flags.insert(flag.into());
    }

    /// Add a flag alias
    pub fn add_alias(&mut self, flag: impl Into<String>, alias: impl Into<String>) {
        let flag = flag.into();
        let alias = alias.into();
        self.flag_aliases
            .entry(flag.clone())
            .or_default()
            .push(alias.clone());
        // Also add reverse mapping
        self.flag_aliases.entry(alias).or_default().push(flag);
    }

    /// Add a subcommand-specific flag
    pub fn add_subcommand_flag(&mut self, subcommand: impl Into<String>, flag: impl Into<String>) {
        self.subcommand_flags
            .entry(subcommand.into())
            .or_default()
            .insert(flag.into());
    }

    /// Add a required constraint
    pub fn add_required(&mut self, flag: impl Into<String>) {
        self.required.push(Constraint::Required {
            flag: flag.into(),
            condition: None,
        });
    }

    /// Add a conditional required constraint
    pub fn add_required_if(&mut self, flag: impl Into<String>, condition: impl Into<String>) {
        self.required.push(Constraint::Required {
            flag: flag.into(),
            condition: Some(condition.into()),
        });
    }

    /// Add mutually exclusive flags
    pub fn add_mutually_exclusive(&mut self, flags: Vec<String>) {
        self.mutually_exclusive.push(flags);
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, flag: impl Into<String>, requires: impl Into<String>) {
        self.dependencies
            .entry(flag.into())
            .or_default()
            .push(requires.into());
    }

    /// Validate arguments against constraints
    pub fn validate(
        &self,
        args: &[(String, Option<String>)],
        subcommand: Option<&str>,
    ) -> ValidationReport {
        let mut report = ValidationReport::valid();

        // 1. Check for hallucinated flags
        report.merge(self.check_hallucinations(args, subcommand));

        // 2. Check required constraints
        report.merge(self.check_required(args));

        // 3. Check mutually exclusive
        report.merge(self.check_mutually_exclusive(args));

        // 4. Check dependencies
        report.merge(self.check_dependencies(args));

        report
    }

    /// Check for hallucinated flags
    fn check_hallucinations(
        &self,
        args: &[(String, Option<String>)],
        subcommand: Option<&str>,
    ) -> ValidationReport {
        let mut report = ValidationReport::valid();

        // Build valid flag set for this context
        let mut valid = self.valid_flags.clone();

        // Add subcommand-specific flags
        if let Some(sc) = subcommand
            && let Some(sc_flags) = self.subcommand_flags.get(sc)
        {
            valid.extend(sc_flags.iter().cloned());
        }

        // Add aliases
        for (flag, aliases) in &self.flag_aliases {
            valid.insert(flag.clone());
            valid.extend(aliases.iter().cloned());
        }

        // Check each argument
        for (flag, _) in args {
            // Normalize flag
            let normalized = self.normalize_flag(flag);

            if !valid.contains(&normalized) && !valid.contains(flag) {
                // Try to find similar flag
                let suggestion = self.find_similar_flag(flag, &valid);
                report.add_violation(Violation::HallucinatedFlag {
                    flag: flag.clone(),
                    suggestion,
                });
            }
        }

        report
    }

    /// Check required constraints
    fn check_required(&self, args: &[(String, Option<String>)]) -> ValidationReport {
        let mut report = ValidationReport::valid();

        for constraint in &self.required {
            if !constraint.is_satisfied(args)
                && let Constraint::Required { flag, condition } = constraint
            {
                report.add_violation(Violation::MissingRequired {
                    flag: flag.clone(),
                    context: condition.clone(),
                });
            }
        }

        report
    }

    /// Check mutually exclusive constraints
    fn check_mutually_exclusive(&self, args: &[(String, Option<String>)]) -> ValidationReport {
        let mut report = ValidationReport::valid();

        let arg_flags: HashSet<_> = args.iter().map(|(f, _)| f.clone()).collect();

        for group in &self.mutually_exclusive {
            let present: Vec<_> = group
                .iter()
                .filter(|f| arg_flags.contains(*f))
                .cloned()
                .collect();

            if present.len() > 1 {
                // Report first conflict
                report.add_violation(Violation::MutuallyExclusive {
                    flag1: present[0].clone(),
                    flag2: present[1].clone(),
                });
            }
        }

        report
    }

    /// Check dependency constraints
    fn check_dependencies(&self, args: &[(String, Option<String>)]) -> ValidationReport {
        let mut report = ValidationReport::valid();

        let arg_flags: HashSet<_> = args.iter().map(|(f, _)| f.clone()).collect();

        for (flag, requires_list) in &self.dependencies {
            if arg_flags.contains(flag) {
                for requires in requires_list {
                    if !arg_flags.contains(requires) {
                        report.add_violation(Violation::MissingDependency {
                            flag: flag.clone(),
                            requires: requires.clone(),
                        });
                    }
                }
            }
        }

        report
    }

    /// Normalize flag (handle aliases)
    fn normalize_flag(&self, flag: &str) -> String {
        for (canonical, aliases) in &self.flag_aliases {
            if aliases.contains(&flag.to_string()) {
                return canonical.clone();
            }
        }
        flag.to_string()
    }

    /// Find similar flag for suggestion
    fn find_similar_flag(&self, flag: &str, valid: &HashSet<String>) -> Option<String> {
        // Simple edit distance
        let threshold = 2; // Allow up to 2 edits

        valid
            .iter()
            .filter(|v| !v.is_empty())
            .min_by_key(|v| levenshtein_distance(flag, v))
            .filter(|v| levenshtein_distance(flag, v) <= threshold)
            .cloned()
    }

    /// Check if a flag is valid
    pub fn is_valid_flag(&self, flag: &str, subcommand: Option<&str>) -> bool {
        if self.valid_flags.contains(flag) {
            return true;
        }

        // Check subcommand-specific
        if let Some(sc) = subcommand
            && let Some(flags) = self.subcommand_flags.get(sc)
            && flags.contains(flag)
        {
            return true;
        }

        // Check aliases
        for aliases in self.flag_aliases.values() {
            if aliases.contains(&flag.to_string()) {
                return true;
            }
        }

        false
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)
                .min(prev_row[j + 1] + 1)
                .min(prev_row[j] + cost);
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Extract constraint graph from documentation
pub fn extract_from_docs(doc: &str, tool_name: &str) -> ConstraintGraph {
    let mut graph = ConstraintGraph::new();

    // Extract all flags
    extract_flags(doc, &mut graph);

    // Extract constraints
    extract_constraints(doc, &mut graph);

    // Tool-specific extractions
    extract_tool_specific(tool_name, doc, &mut graph);

    graph
}

static FLAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(-{1,2}[a-zA-Z0-9_-]+)(?:\s+|$)").unwrap());

/// Extract all flags from documentation
fn extract_flags(doc: &str, graph: &mut ConstraintGraph) {
    for cap in FLAG_RE.captures_iter(doc) {
        if let Some(m) = cap.get(1) {
            graph.add_flag(m.as_str());
        }
    }
}

/// Extract constraint patterns
fn extract_constraints(doc: &str, graph: &mut ConstraintGraph) {
    let doc_lower = doc.to_lowercase();

    // Required flags (look for REQUIRED markers)
    let required_re = Regex::new(r"(?i)(?:required|mandatory).*?(-{1,2}\w+)").unwrap();
    for cap in required_re.captures_iter(&doc_lower) {
        if let Some(m) = cap.get(1) {
            graph.add_required(m.as_str());
        }
    }

    // Mutually exclusive patterns
    let mutex_re = Regex::new(r"(?i)mutually\s+exclusive.*?(-{1,2}\w+).*?(-{1,2}\w+)").unwrap();
    for cap in mutex_re.captures_iter(&doc_lower) {
        let flags: Vec<_> = cap
            .iter()
            .skip(1)
            .filter_map(|m| m.map(|m| m.as_str().to_string()))
            .collect();
        if flags.len() >= 2 {
            graph.add_mutually_exclusive(flags);
        }
    }

    // Dependency patterns
    let dep_re = Regex::new(r"(?i)(-{1,2}\w+).*?requires.*?(-{1,2}\w+)").unwrap();
    for cap in dep_re.captures_iter(&doc_lower) {
        if let (Some(flag), Some(requires)) = (cap.get(1), cap.get(2)) {
            graph.add_dependency(flag.as_str(), requires.as_str());
        }
    }
}

/// Tool-specific constraint extraction
fn extract_tool_specific(tool_name: &str, doc: &str, graph: &mut ConstraintGraph) {
    let name_lower = tool_name.to_lowercase();

    match name_lower.as_str() {
        "samtools" => {
            // Add known samtools flags
            let global_flags = ["-h", "--help", "-?", "--version", "-V"];
            for flag in &global_flags {
                graph.add_flag(*flag);
            }
        }
        "bcftools" => {
            // bcftools has many common flags
            let common_flags = ["-o", "--output", "-O", "--output-type", "-r", "--regions"];
            for flag in &common_flags {
                graph.add_flag(*flag);
            }
        }
        "bedtools" => {
            // bedtools subcommands have specific flags
            if doc.contains("intersect") {
                let intersect_flags = ["-a", "-b", "-wa", "-wb", "-wo", "-u", "-v"];
                for flag in &intersect_flags {
                    graph.add_subcommand_flag("intersect", *flag);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hallucination_detection() {
        let mut graph = ConstraintGraph::new();
        graph.add_flag("-o");
        graph.add_flag("--output");
        graph.add_flag("-i");

        let args = vec![
            ("-o".to_string(), Some("out.txt".to_string())),
            ("--fake-flag".to_string(), None),
        ];

        let report = graph.validate(&args, None);
        assert!(!report.is_valid);
        assert!(report.violations.iter().any(
            |v| matches!(v, Violation::HallucinatedFlag { flag, .. } if flag == "--fake-flag")
        ));
    }

    #[test]
    fn test_required_constraint() {
        let mut graph = ConstraintGraph::new();
        graph.add_flag("-i");
        graph.add_flag("-o");
        graph.add_required("-i");

        let args_without = vec![("-o".to_string(), Some("out.txt".to_string()))];
        let report = graph.validate(&args_without, None);
        assert!(!report.is_valid);

        let args_with = vec![
            ("-i".to_string(), Some("in.txt".to_string())),
            ("-o".to_string(), Some("out.txt".to_string())),
        ];
        let report = graph.validate(&args_with, None);
        assert!(report.is_valid);
    }

    #[test]
    fn test_mutually_exclusive() {
        let mut graph = ConstraintGraph::new();
        graph.add_flag("-a");
        graph.add_flag("-b");
        graph.add_mutually_exclusive(vec!["-a".to_string(), "-b".to_string()]);

        let args = vec![("-a".to_string(), None), ("-b".to_string(), None)];

        let report = graph.validate(&args, None);
        assert!(!report.is_valid);
        assert!(
            report
                .violations
                .iter()
                .any(|v| matches!(v, Violation::MutuallyExclusive { .. }))
        );
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("sunday", "saturday"), 3);
        assert_eq!(levenshtein_distance("-o", "--output"), 6);
        assert_eq!(levenshtein_distance("", ""), 0);
    }
}
