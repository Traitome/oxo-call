//! Pattern-based flag extraction from --help output.
//! No tool-specific code. Handles GNU, POSIX, Python argparse, Java styles.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagInfo {
    pub short: Option<String>,
    pub long: Option<String>,
    pub value_type: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlagCatalog {
    pub flags: Vec<FlagInfo>,
    pub subcommands: Vec<String>,
    pub usage_line: String,
}

pub fn extract_flags(help: &str) -> FlagCatalog {
    let mut cat = FlagCatalog::default();
    let lines: Vec<&str> = help.lines().collect();

    for l in &lines {
        let t = l.trim().to_lowercase();
        if t.starts_with("usage:") || t.starts_with("usage ") {
            cat.usage_line = l.to_string(); break;
        }
    }
    let mut in_cmds = false;
    for l in &lines {
        let t = l.trim();
        if t.starts_with("Commands:") || t.starts_with("COMMANDS:") || t == "Subcommands:" { in_cmds = true; continue; }
        if in_cmds {
            if t.is_empty() || t.starts_with("Options:") || t.starts_with("Positional") { in_cmds = false; continue; }
            let w = t.split_whitespace().next().unwrap_or("");
            if !w.is_empty() && !w.starts_with('-') && w.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                cat.subcommands.push(w.to_string());
            }
        }
    }

    for l in &lines {
        let t = l.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("---") || t.starts_with("==") { continue; }
        if !t.starts_with('-') { continue; }
        if let Some(f) = parse_one(t)
            && !cat.flags.iter().any(|x| x.long == f.long && x.short == f.short) {
                cat.flags.push(f);
            }
    }
    cat
}

fn parse_one(line: &str) -> Option<FlagInfo> {
    let is_double = line.starts_with("--") && !line.starts_with("---");
    let stripped = line.trim_start_matches('-');

    // --long-name ... (no short form)
    if is_double {
        let end = stripped.find(|c: char| c.is_whitespace() || c == '=').unwrap_or(stripped.len());
        let long = stripped[..end].to_string();
        if long.len() < 2 { return None; }
        let after = stripped[end..].trim();
        let (vt, desc) = type_and_desc(after);
        return Some(FlagInfo { short: None, long: Some(long), value_type: vt, required: false, description: desc });
    }

    // -x, --long-name ... or -x ...
    let first = stripped.chars().next()?;
    if first == '-' { return None; }
    let short = first.to_string();
    let rest = &stripped[first.len_utf8()..];

    // Check for ", --" indicating a long form follows
    if rest.starts_with(", --") || rest.starts_with(",--") {
        let after = rest.trim_start_matches(',').trim().trim_start_matches('-').trim_start_matches('-');
        let end = after.find(|c: char| c.is_whitespace() || c == '=').unwrap_or(after.len());
        let long = after[..end].to_string();
        let after = after[end..].trim();
        let (vt, desc) = type_and_desc(after);
        return Some(FlagInfo { short: Some(short), long: Some(long), value_type: vt, required: false, description: desc });
    }

    // -x ... (short only)
    if rest.starts_with(' ') || rest.starts_with('\t') {
        let (vt, desc) = type_and_desc(rest.trim());
        return Some(FlagInfo { short: Some(short), long: None, value_type: vt, required: false, description: desc });
    }

    // -x (boolean, no space after — but description might follow)
    if rest.is_empty() || rest.starts_with(',') || rest.starts_with('.') || rest.starts_with(';') {
        return Some(FlagInfo { short: Some(short), long: None, value_type: "bool".into(), required: false, description: rest.to_string() });
    }

    None
}

fn type_and_desc(s: &str) -> (String, String) {
    let s = s.trim();
    for kw in &["INT", "int", "STRING", "STR", "string", "FILE", "file",
        "FLOAT", "float", "DIR", "directory", "PATH", "path", "NUMBER"] {
        if let Some(rest) = s.strip_prefix(kw)
            && (rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace())) {
                return (kw.to_string(), rest.trim().to_string());
            }
    }
    for kw in &["<int>", "<string>", "<float>", "<file>", "<dir>", "<path>"] {
        if let Some(rest) = s.strip_prefix(kw) {
            return (kw.to_string(), rest.trim().to_string());
        }
    }
    ("bool".into(), s.to_string())
}

pub fn validate_args(args: &str, catalog: &FlagCatalog) -> Vec<String> {
    let mut issues = vec![];
    let known: Vec<&str> = catalog.flags.iter()
        .filter_map(|f| f.long.as_deref().or(f.short.as_deref()))
        .collect();
    for word in args.split_whitespace() {
        let flag = word.trim_start_matches('-').split('=').next().unwrap_or("");
        if word.starts_with('-') && flag.len() > 1 && !flag.chars().all(|c| c.is_ascii_digit())
            && !known.contains(&flag)
        {
            issues.push(format!("Flag --{} not found in --help", flag));
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gnu_flags() {
        let help = "\
Usage: samtools sort [options]
Options:
  -@, --threads INT    number of threads [0]
  -o, --output FILE    output file
  -n                   sort by name
  -l, --level INT      compression level [0]
  --no-PG              do not add @PG header\n";
        let cat = extract_flags(help);
        let longs: Vec<_> = cat.flags.iter().filter_map(|f| f.long.clone()).collect();
        assert!(longs.contains(&"threads".into()), "longs: {:?}", longs);
        assert!(longs.contains(&"output".into()));
        assert!(longs.contains(&"level".into()));
        assert!(longs.contains(&"no-PG".into()));
        let shorts: Vec<_> = cat.flags.iter().filter_map(|f| f.short.clone()).collect();
        assert!(shorts.contains(&"n".into()), "shorts: {:?}", shorts);
    }

    #[test]
    fn test_python_style() {
        let help = "\
usage: script.py [-h] --input FILE --output FILE [--threads INT]
  -h, --help     show help
  --input FILE   input file
  --output FILE  output file
  --threads INT  number of threads (default: 4)\n";
        let cat = extract_flags(help);
        let longs: Vec<_> = cat.flags.iter().filter_map(|f| f.long.clone()).collect();
        assert!(longs.contains(&"input".into()), "longs: {:?}", longs);
        assert!(longs.contains(&"output".into()));
        assert!(longs.contains(&"threads".into()));
        assert!(longs.contains(&"help".into()));
    }
}
