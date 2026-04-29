//! Auto Fixer Module
//!
//! Automatically fixes common issues in generated commands without requiring
//! LLM regeneration. Provides fast, deterministic fixes for known patterns.

#![allow(dead_code)]

use crate::command_validator::CommandValidation;
use crate::constraint_graph::Violation;
use regex::Regex;

/// Auto Fixer for command corrections
#[derive(Debug, Clone)]
pub struct AutoFixer {
    /// Maximum fix iterations
    max_iterations: usize,
    /// Enable aggressive fixes
    aggressive: bool,
}

impl Default for AutoFixer {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoFixer {
    /// Create a new auto fixer
    pub fn new() -> Self {
        Self {
            max_iterations: 3,
            aggressive: false,
        }
    }

    /// Enable aggressive fixing
    pub fn aggressive(mut self) -> Self {
        self.aggressive = true;
        self
    }

    /// Try to fix a command automatically
    pub fn fix(&self, command: &str, validation: &CommandValidation) -> Option<FixResult> {
        let mut current = command.to_string();
        let mut applied_fixes = Vec::new();
        let mut iterations = 0;

        while iterations < self.max_iterations {
            // Try to apply fixes for current violations
            let fixes = self.identify_fixes(&current, validation);

            if fixes.is_empty() {
                break;
            }

            for fix in fixes {
                if let Some(fixed) = self.apply_fix(&current, &fix)
                    && fixed != current
                {
                    applied_fixes.push(fix);
                    current = fixed;
                }
            }

            iterations += 1;
        }

        // Check if the fix improved the command
        if current != command {
            Some(FixResult {
                original: command.to_string(),
                fixed: current,
                fixes_applied: applied_fixes,
                iterations,
            })
        } else {
            None
        }
    }

    /// Identify possible fixes for violations
    fn identify_fixes(&self, command: &str, validation: &CommandValidation) -> Vec<Fix> {
        let mut fixes = Vec::new();

        for violation in &validation.violations {
            match violation {
                Violation::HallucinatedFlag { flag, suggestion } => {
                    if let Some(replacement) = suggestion {
                        fixes.push(Fix {
                            priority: 1,
                            fix_fn: FixFn::ReplaceFlag(flag.clone(), replacement.clone()),
                        });
                    } else {
                        fixes.push(Fix {
                            priority: 2,
                            fix_fn: FixFn::RemoveFlag(flag.clone()),
                        });
                    }
                }
                Violation::MissingRequired { flag, context: _ } => {
                    fixes.push(Fix {
                        priority: 3,
                        fix_fn: FixFn::AddFlag(flag.clone(), None),
                    });
                }
                Violation::MutuallyExclusive { flag1: _, flag2 } => {
                    // Remove the second flag in aggressive mode
                    if self.aggressive {
                        fixes.push(Fix {
                            priority: 2,
                            fix_fn: FixFn::RemoveFlag(flag2.clone()),
                        });
                    }
                }
                _ => {}
            }
        }

        // Check for missing subcommand
        if validation.detected_subcommand.is_none()
            && let Some(subcmd) = self.infer_subcommand(command)
        {
            fixes.push(Fix {
                priority: 0, // Highest priority
                fix_fn: FixFn::AddSubcommand(subcmd),
            });
        }

        // Sort by priority
        fixes.sort_by_key(|f| f.priority);

        fixes
    }

    /// Apply a single fix
    fn apply_fix(&self, command: &str, fix: &Fix) -> Option<String> {
        match &fix.fix_fn {
            FixFn::ReplaceFlag(old, new) => {
                let pattern = Regex::new(&regex::escape(old)).ok()?;
                Some(pattern.replace_all(command, new).to_string())
            }
            FixFn::RemoveFlag(flag) => self.remove_flag_with_value(command, flag),
            FixFn::AddFlag(flag, value) => {
                let prefix = if command.starts_with('-') {
                    // Missing subcommand case - need to handle differently
                    ""
                } else {
                    " "
                };

                let new_flag = match value {
                    Some(v) => format!("{}{} {}", prefix, flag, v),
                    None => format!("{}{}", prefix, flag),
                };

                Some(format!("{}{}", command, new_flag))
            }
            FixFn::AddSubcommand(subcmd) => Some(format!("{} {}", subcmd, command)),
        }
    }

    /// Remove a flag and its value if present
    fn remove_flag_with_value(&self, command: &str, flag: &str) -> Option<String> {
        // Try to match flag with value: -f value or --flag value
        let pattern1 = format!(r"{}\s+\S+", regex::escape(flag));
        let re1 = Regex::new(&pattern1).ok()?;

        let result = re1.replace(command, "").to_string();

        // Also try removing just the flag (in case it was already removed or has no value)
        let result = result.replace(flag, "");

        // Clean up extra whitespace
        let result = result.split_whitespace().collect::<Vec<_>>().join(" ");

        Some(result)
    }

    /// Try to infer subcommand from context
    fn infer_subcommand(&self, command: &str) -> Option<String> {
        let lower = command.to_lowercase();

        // Common patterns
        if lower.contains("sort") {
            Some("sort".to_string())
        } else if lower.contains("view") || lower.contains("show") {
            Some("view".to_string())
        } else if lower.contains("intersect") {
            Some("intersect".to_string())
        } else if lower.contains("merge") {
            Some("merge".to_string())
        } else if lower.contains("index") {
            Some("index".to_string())
        } else if lower.contains("filter") || lower.contains("qual") {
            Some("view".to_string())
        } else {
            None
        }
    }

    /// Quick fix for common hallucination patterns
    pub fn quick_fix(&self, command: &str, valid_flags: &[String]) -> Option<String> {
        let mut fixed = command.to_string();

        // Common hallucination patterns and their likely correct forms
        let corrections: Vec<(Regex, &str)> = vec![
            (Regex::new(r"--output-file").ok()?, "-o"),
            (Regex::new(r"--input-file").ok()?, "-i"),
            (Regex::new(r"--threads").ok()?, "-@"),
            (Regex::new(r"--output").ok()?, "-o"),
        ];

        for (pattern, replacement) in corrections {
            fixed = pattern.replace_all(&fixed, replacement).to_string();
        }

        // Remove flags that are definitely not in valid_flags
        let parts: Vec<&str> = fixed.split_whitespace().collect();
        let mut cleaned = Vec::new();

        for part in parts {
            if part.starts_with('-') {
                // Check if this flag or its base form is valid
                let base_flag = part.split('=').next().unwrap_or(part);
                if valid_flags
                    .iter()
                    .any(|f| f == base_flag || base_flag.starts_with(f))
                {
                    cleaned.push(part);
                }
                // Otherwise skip (remove hallucinated flag)
            } else {
                cleaned.push(part);
            }
        }

        let result = cleaned.join(" ");

        if result != command {
            Some(result)
        } else {
            None
        }
    }
}

/// Fix definition
#[derive(Debug, Clone)]
pub struct Fix {
    /// Priority (lower = higher priority)
    priority: usize,
    /// Fix function
    fix_fn: FixFn,
}

/// Fix function type
#[derive(Debug, Clone)]
pub enum FixFn {
    /// Replace one flag with another
    ReplaceFlag(String, String),
    /// Remove a flag
    RemoveFlag(String),
    /// Add a flag (with optional value)
    AddFlag(String, Option<String>),
    /// Add subcommand at beginning
    AddSubcommand(String),
}

/// Result of a fix operation
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Original command
    pub original: String,
    /// Fixed command
    pub fixed: String,
    /// List of fixes applied
    pub fixes_applied: Vec<Fix>,
    /// Number of iterations
    pub iterations: usize,
}

impl FixResult {
    /// Check if the fix was successful (command changed)
    pub fn is_changed(&self) -> bool {
        self.original != self.fixed
    }

    /// Get summary of changes
    pub fn summary(&self) -> String {
        if self.fixes_applied.is_empty() {
            "No changes needed".to_string()
        } else {
            format!(
                "Applied {} fix(es) in {} iteration(s)",
                self.fixes_applied.len(),
                self.iterations
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_flag_with_value() {
        let fixer = AutoFixer::new();

        let result = fixer.remove_flag_with_value("sort -o out.bam in.bam", "-o");
        assert_eq!(result, Some("sort in.bam".to_string()));
    }

    #[test]
    fn test_add_subcommand() {
        let fixer = AutoFixer::new();
        let fix = Fix {
            priority: 0,
            fix_fn: FixFn::AddSubcommand("sort".to_string()),
        };

        let result = fixer.apply_fix("-o out.bam in.bam", &fix);
        assert_eq!(result, Some("sort -o out.bam in.bam".to_string()));
    }

    #[test]
    fn test_quick_fix_removes_invalid_flags() {
        let fixer = AutoFixer::new();
        let valid_flags = vec!["-o".to_string(), "-@".to_string()];

        let result = fixer.quick_fix("sort --invalid -o out.bam", &valid_flags);
        assert!(result.is_some());
        let fixed = result.unwrap();
        assert!(!fixed.contains("--invalid"));
        assert!(fixed.contains("-o"));
    }
}
