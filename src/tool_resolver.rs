use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecord {
    pub name: String,
    pub resolved_path: PathBuf,
    pub interpreter: Option<Interpreter>,
    pub is_path_dependent: bool,
    pub global_path: Option<PathBuf>,
    pub version: Option<String>,
    pub companion_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Interpreter {
    Python { path: PathBuf },
    Rscript { path: PathBuf },
    Perl { path: PathBuf },
    Bash { path: PathBuf },
    None,
}

impl ToolRecord {
    pub fn effective_name(&self) -> &str {
        if self.is_path_dependent {
            self.resolved_path.to_str().unwrap_or(&self.name)
        } else {
            &self.name
        }
    }
}

pub fn validate_tool_name(tool: &str) -> Result<()> {
    if tool.is_empty() {
        return Err(color_eyre::eyre::eyre!("Tool name cannot be empty"));
    }
    if tool.contains("..") {
        return Err(color_eyre::eyre::eyre!(
            "Tool name must not contain path traversal ('..')"
        ));
    }
    Ok(())
}

pub fn is_path_dependent(tool: &str) -> bool {
    tool.contains('/') || tool.contains('\\')
}

pub fn resolve_tool(tool: &str) -> Result<ToolRecord> {
    validate_tool_name(tool)?;

    let path_dependent = is_path_dependent(tool);

    if path_dependent {
        resolve_path_dependent(tool)
    } else {
        resolve_global(tool)
    }
}

fn resolve_path_dependent(tool: &str) -> Result<ToolRecord> {
    let path = PathBuf::from(tool);
    let abs_path = if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir()?.join(&path)
    };

    if !abs_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Path-dependent tool not found: {}",
            abs_path.display()
        ));
    }

    let interpreter = detect_interpreter(&abs_path);

    let version = detect_version(tool, &interpreter);

    Ok(ToolRecord {
        name: tool.to_string(),
        resolved_path: abs_path,
        interpreter: Some(interpreter),
        is_path_dependent: true,
        global_path: None,
        version,
        companion_tools: Vec::new(),
    })
}

fn resolve_global(tool: &str) -> Result<ToolRecord> {
    let global_path = which_tool(tool);

    let interpreter = global_path
        .as_ref()
        .map(detect_interpreter)
        .unwrap_or(Interpreter::None);

    let version = detect_version(tool, &interpreter);

    let companion_tools = discover_companions(tool);

    Ok(ToolRecord {
        name: tool.to_string(),
        resolved_path: global_path.clone().unwrap_or_else(|| PathBuf::from(tool)),
        interpreter: if global_path.is_some() {
            Some(interpreter)
        } else {
            None
        },
        is_path_dependent: false,
        global_path,
        version,
        companion_tools,
    })
}

fn which_tool(tool: &str) -> Option<PathBuf> {
    std::process::Command::new("which")
        .arg(tool)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| PathBuf::from(s.trim()))
            } else {
                None
            }
        })
}

fn detect_interpreter(path: &PathBuf) -> Interpreter {
    if let Ok(content) = std::fs::read_to_string(path) {
        let first_line = content.lines().next().unwrap_or("");
        if first_line.starts_with("#!") {
            let shebang = first_line.trim_start_matches("#!").trim();
            if shebang.contains("python") {
                let py_path = resolve_shebang_interpreter(shebang, "python");
                return Interpreter::Python { path: py_path };
            }
            if shebang.contains("Rscript") {
                let r_path = resolve_shebang_interpreter(shebang, "Rscript");
                return Interpreter::Rscript { path: r_path };
            }
            if shebang.contains("perl") {
                let pl_path = resolve_shebang_interpreter(shebang, "perl");
                return Interpreter::Perl { path: pl_path };
            }
            if shebang.contains("bash") || shebang.contains("sh") {
                let sh_path = resolve_shebang_interpreter(shebang, "bash");
                return Interpreter::Bash { path: sh_path };
            }
        }
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "py" => Interpreter::Python {
            path: which_tool("python3").unwrap_or_else(|| PathBuf::from("python3")),
        },
        "r" => Interpreter::Rscript {
            path: which_tool("Rscript").unwrap_or_else(|| PathBuf::from("Rscript")),
        },
        "pl" => Interpreter::Perl {
            path: which_tool("perl").unwrap_or_else(|| PathBuf::from("perl")),
        },
        "sh" => Interpreter::Bash {
            path: which_tool("bash").unwrap_or_else(|| PathBuf::from("bash")),
        },
        _ => Interpreter::None,
    }
}

fn resolve_shebang_interpreter(shebang: &str, name: &str) -> PathBuf {
    if shebang.contains("/usr/bin/env")
        && let Some(part) = shebang.split_whitespace().find(|p| p.contains(name))
    {
        return PathBuf::from(part);
    }
    let parts: Vec<&str> = shebang.split_whitespace().collect();
    if let Some(first) = parts.first() {
        return PathBuf::from(*first);
    }
    PathBuf::from(name)
}

fn detect_version(tool: &str, interpreter: &Interpreter) -> Option<String> {
    match interpreter {
        Interpreter::Python { .. } => return try_detect_version(tool, &["--version", "-v"]),
        Interpreter::Rscript { .. } => return try_detect_version(tool, &["--version"]),
        Interpreter::Perl { .. } => return try_detect_version(tool, &["--version", "-v"]),
        Interpreter::Bash { .. } => return None,
        Interpreter::None => {}
    }
    try_detect_version(tool, &["--version", "-v", "-V", "version"])
}

fn try_detect_version(tool: &str, flags: &[&str]) -> Option<String> {
    for flag in flags {
        if let Ok(output) = std::process::Command::new(tool).arg(flag).output()
            && (output.status.success() || output.status.code() == Some(0))
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{stdout}{stderr}");
            let first_line = combined.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() && first_line.len() < 200 {
                return Some(first_line.to_string());
            }
        }
    }
    None
}

fn discover_companions(tool: &str) -> Vec<String> {
    let mut companions = Vec::new();

    let path_var = std::env::var("PATH").unwrap_or_default();
    let path_dirs: Vec<PathBuf> = std::env::split_paths(&path_var).collect();

    let tool_lower = tool.to_lowercase();
    let mut seen = std::collections::HashSet::new();

    for dir in &path_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = match name.to_str() {
                    Some(s) => s,
                    None => continue,
                };

                let name_lower = name_str.to_lowercase();
                if !name_lower.starts_with(&tool_lower) {
                    continue;
                }
                if name_lower == tool_lower {
                    continue;
                }

                let suffix = &name_str[tool.len()..];
                if suffix.is_empty() {
                    continue;
                }

                let is_companion = match suffix.chars().next() {
                    Some('-') | Some('_') => {
                        let rest = &suffix[1..];
                        !rest.is_empty()
                            && rest
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                    }
                    Some('.') => {
                        let rest = &suffix[1..];
                        matches!(rest, "py" | "sh" | "pl" | "R" | "rb" | "jl")
                    }
                    _ => false,
                };

                if is_companion && seen.insert(name_str.to_string()) {
                    companions.push(name_str.to_string());
                }
            }
        }
    }

    companions.sort();
    companions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_validate_tool_name_valid() {
        assert!(validate_tool_name("samtools").is_ok());
        assert!(validate_tool_name("bwa-mem2").is_ok());
        assert!(validate_tool_name("featureCounts").is_ok());
    }

    #[test]
    fn test_validate_tool_name_empty() {
        assert!(validate_tool_name("").is_err());
    }

    #[test]
    fn test_validate_tool_name_traversal() {
        assert!(validate_tool_name("../etc/passwd").is_err());
        assert!(validate_tool_name("foo/../bar").is_err());
    }

    #[test]
    fn test_validate_tool_name_path_allowed() {
        assert!(validate_tool_name("./scripts/run.sh").is_ok());
        assert!(validate_tool_name("/usr/local/bin/tool").is_ok());
        assert!(validate_tool_name("path/to/tool").is_ok());
    }

    #[test]
    fn test_is_path_dependent() {
        assert!(is_path_dependent("./tool"));
        assert!(is_path_dependent("/usr/bin/tool"));
        assert!(is_path_dependent("path\\to\\tool"));
        assert!(!is_path_dependent("samtools"));
        assert!(!is_path_dependent("bwa-mem2"));
    }

    #[test]
    fn test_resolve_global_samtools() {
        if which_tool("samtools").is_some() {
            let record = resolve_tool("samtools").unwrap();
            assert_eq!(record.name, "samtools");
            assert!(!record.is_path_dependent);
            assert!(record.global_path.is_some());
        }
    }

    #[test]
    fn test_discover_companions_bowtie2() {
        if which_tool("bowtie2").is_some() {
            let companions = discover_companions("bowtie2");
            assert!(
                companions.contains(&"bowtie2-build".to_string())
                    || companions.contains(&"bowtie2-inspect".to_string())
            );
        }
    }

    #[test]
    fn test_effective_name_global() {
        let record = ToolRecord {
            name: "samtools".to_string(),
            resolved_path: PathBuf::from("/usr/bin/samtools"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(PathBuf::from("/usr/bin/samtools")),
            version: None,
            companion_tools: Vec::new(),
        };
        assert_eq!(record.effective_name(), "samtools");
    }

    #[test]
    fn test_effective_name_path_dependent() {
        let record = ToolRecord {
            name: "./run.sh".to_string(),
            resolved_path: PathBuf::from("/home/user/run.sh"),
            interpreter: Some(Interpreter::Bash {
                path: PathBuf::from("/bin/bash"),
            }),
            is_path_dependent: true,
            global_path: None,
            version: None,
            companion_tools: Vec::new(),
        };
        assert_eq!(record.effective_name(), "/home/user/run.sh");
    }

    #[test]
    fn test_resolve_path_dependent_nonexistent() {
        let result = resolve_path_dependent("/nonexistent/path/tool_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_interpreter_python_extension() {
        let dir = std::env::temp_dir().join("oxo_test_interpreter_py");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script.py");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "print('hello')").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Python { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_r_extension() {
        let dir = std::env::temp_dir().join("oxo_test_interpreter_r");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script.R");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "cat('hello')").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Rscript { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_shebang_python() {
        let dir = std::env::temp_dir().join("oxo_test_shebang_py");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "#!/usr/bin/env python3").unwrap();
        writeln!(f, "print('hello')").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Python { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_shebang_bash() {
        let dir = std::env::temp_dir().join("oxo_test_shebang_bash");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "#!/bin/bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Bash { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_shebang_perl() {
        let dir = std::env::temp_dir().join("oxo_test_shebang_perl");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "#!/usr/bin/perl").unwrap();
        writeln!(f, "print 'hello';").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Perl { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_shebang_rscript() {
        let dir = std::env::temp_dir().join("oxo_test_shebang_rscript");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "#!/usr/bin/env Rscript").unwrap();
        writeln!(f, "cat('hello')").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Rscript { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_no_extension_no_shebang() {
        let dir = std::env::temp_dir().join("oxo_test_no_interp");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_binary");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "binary content").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::None));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_sh_extension() {
        let dir = std::env::temp_dir().join("oxo_test_sh_ext");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script.sh");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "echo hello").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Bash { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_interpreter_pl_extension() {
        let dir = std::env::temp_dir().join("oxo_test_pl_ext");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_script.pl");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "print 'hello';").unwrap();
        let interp = detect_interpreter(&file_path);
        assert!(matches!(interp, Interpreter::Perl { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_resolve_shebang_interpreter_env() {
        let result = resolve_shebang_interpreter("/usr/bin/env python3", "python");
        assert!(result.to_str().unwrap().contains("python"));
    }

    #[test]
    fn test_resolve_shebang_interpreter_direct() {
        let result = resolve_shebang_interpreter("/usr/bin/python3", "python");
        assert_eq!(result, PathBuf::from("/usr/bin/python3"));
    }

    #[test]
    fn test_resolve_shebang_interpreter_empty() {
        let result = resolve_shebang_interpreter("", "python");
        assert_eq!(result, PathBuf::from("python"));
    }

    #[test]
    fn test_resolve_path_dependent_relative() {
        let dir = std::env::temp_dir().join("oxo_test_relative");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("tool.sh");
        std::fs::File::create(&file_path).ok();
        let abs = std::env::current_dir().unwrap().join(&dir).join("tool.sh");
        if abs.exists() {
            let record = resolve_path_dependent(abs.to_str().unwrap()).unwrap();
            assert!(record.is_path_dependent);
            assert!(record.resolved_path.exists());
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_discover_companions_no_match() {
        let companions = discover_companions("nonexistent_tool_xyz_12345");
        assert!(companions.is_empty());
    }

    #[test]
    fn test_which_tool_nonexistent() {
        assert!(which_tool("nonexistent_tool_xyz_12345").is_none());
    }

    #[test]
    fn test_try_detect_version_nonexistent() {
        let result = try_detect_version("nonexistent_tool_xyz_12345", &["--version"]);
        assert!(result.is_none());
    }
}
