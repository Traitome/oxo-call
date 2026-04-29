//! Auto Fixer Module (HDA)
//!
//! Automatically fixes common issues in generated commands using schema-based
//! validation errors instead of constraint_graph violations.

#![allow(dead_code)]

use crate::command_validator::CommandValidation;
use crate::schema::ValidationError;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct AutoFixer {
    max_iterations: usize,
    aggressive: bool,
}

impl Default for AutoFixer {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoFixer {
    pub fn new() -> Self {
        Self {
            max_iterations: 3,
            aggressive: false,
        }
    }

    pub fn aggressive(mut self) -> Self {
        self.aggressive = true;
        self
    }

    pub fn fix(&self, command: &str, validation: &CommandValidation) -> Option<FixResult> {
        let mut current = command.to_string();
        let mut applied_fixes = Vec::new();
        let mut iterations = 0;

        while iterations < self.max_iterations {
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

    fn identify_fixes(&self, command: &str, validation: &CommandValidation) -> Vec<Fix> {
        let mut fixes = Vec::new();

        for error in &validation.errors {
            match error {
                ValidationError::InvalidFlag { flag, valid_flags } => {
                    if let Some(replacement) = valid_flags.first() {
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
                ValidationError::MissingRequiredFlag { flag } => {
                    fixes.push(Fix {
                        priority: 3,
                        fix_fn: FixFn::AddFlag(flag.clone(), None),
                    });
                }
                ValidationError::ConstraintViolation { message } => {
                    if self.aggressive
                        && let Some(flag) = message.strip_prefix("Mutually exclusive with ")
                    {
                        fixes.push(Fix {
                            priority: 2,
                            fix_fn: FixFn::RemoveFlag(flag.to_string()),
                        });
                    }
                }
                _ => {}
            }
        }

        if validation.detected_subcommand.is_none()
            && let Some(subcmd) = self.infer_subcommand(command)
        {
            fixes.push(Fix {
                priority: 0,
                fix_fn: FixFn::AddSubcommand(subcmd),
            });
        }

        fixes.sort_by_key(|f| f.priority);
        fixes
    }

    fn apply_fix(&self, command: &str, fix: &Fix) -> Option<String> {
        match &fix.fix_fn {
            FixFn::ReplaceFlag(old, new) => {
                let pattern = Regex::new(&regex::escape(old)).ok()?;
                Some(pattern.replace_all(command, new).to_string())
            }
            FixFn::RemoveFlag(flag) => self.remove_flag_with_value(command, flag),
            FixFn::AddFlag(flag, value) => {
                let prefix = if command.starts_with('-') { "" } else { " " };
                let new_flag = match value {
                    Some(v) => format!("{}{} {}", prefix, flag, v),
                    None => format!("{}{}", prefix, flag),
                };
                Some(format!("{}{}", command, new_flag))
            }
            FixFn::AddSubcommand(subcmd) => Some(format!("{} {}", subcmd, command)),
        }
    }

    fn remove_flag_with_value(&self, command: &str, flag: &str) -> Option<String> {
        let pattern1 = format!(r"{}\s+\S+", regex::escape(flag));
        let re1 = Regex::new(&pattern1).ok()?;
        let result = re1.replace(command, "").to_string();
        let result = result.replace(flag, "");
        let result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        Some(result)
    }

    fn infer_subcommand(&self, command: &str) -> Option<String> {
        let lower = command.to_lowercase();
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

    pub fn quick_fix(&self, command: &str, valid_flags: &[String]) -> Option<String> {
        let mut fixed = command.to_string();

        let corrections: Vec<(Regex, &str)> = vec![
            (Regex::new(r"--output-file").ok()?, "-o"),
            (Regex::new(r"--input-file").ok()?, "-i"),
            (Regex::new(r"--threads").ok()?, "-@"),
            (Regex::new(r"--output").ok()?, "-o"),
        ];

        for (pattern, replacement) in corrections {
            fixed = pattern.replace_all(&fixed, replacement).to_string();
        }

        let parts: Vec<&str> = fixed.split_whitespace().collect();
        let mut cleaned = Vec::new();

        for part in parts {
            if part.starts_with('-') {
                let base_flag = part.split('=').next().unwrap_or(part);
                if valid_flags
                    .iter()
                    .any(|f| f == base_flag || base_flag.starts_with(f))
                {
                    cleaned.push(part);
                }
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

#[derive(Debug, Clone)]
pub struct Fix {
    priority: usize,
    fix_fn: FixFn,
}

#[derive(Debug, Clone)]
pub enum FixFn {
    ReplaceFlag(String, String),
    RemoveFlag(String),
    AddFlag(String, Option<String>),
    AddSubcommand(String),
}

#[derive(Debug, Clone)]
pub struct FixResult {
    pub original: String,
    pub fixed: String,
    pub fixes_applied: Vec<Fix>,
    pub iterations: usize,
}

impl FixResult {
    pub fn is_changed(&self) -> bool {
        self.original != self.fixed
    }

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
