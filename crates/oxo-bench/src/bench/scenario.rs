//! Skill file parsing and benchmark scenario / usage-description generation.
//!
//! This module turns the built-in skill Markdown files (in `skills/`) into:
//!
//! 1. **Reference commands** — 10 scenarios per tool, each with a known-good
//!    `ARGS` string extracted (or derived) from the skill examples.
//! 2. **Usage descriptions** — 10 diverse English paraphrases per scenario,
//!    simulating users of different experience levels.
//!
//! Both artefacts are exported as CSV so that humans can review / tweak them
//! before running the actual LLM evaluation.

use std::io::Write;
use std::path::Path;

// ── Data types ───────────────────────────────────────────────────────────────

/// Parsed representation of a single skill Markdown file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillFile {
    pub name: String,
    pub category: String,
    pub description: String,
    pub tags: Vec<String>,
    pub source_url: String,
    pub examples: Vec<SkillExample>,
}

/// One example block from a skill file (`### heading` → `**Args:**` → `**Explanation:**`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillExample {
    pub task: String,
    pub args: String,
    pub explanation: String,
}

/// A benchmark scenario: one specific flag-combination for a tool with a
/// known-good reference command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scenario {
    pub tool: String,
    pub scenario_id: String,
    pub reference_args: String,
    pub task_description: String,
    pub category: String,
}

/// A single usage description — one of 10 paraphrases for a given scenario,
/// written at a particular user-experience level.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageDescription {
    pub tool: String,
    pub scenario_id: String,
    pub desc_id: String,
    pub user_level: String,
    pub description: String,
}

// ── Skill file parsing ───────────────────────────────────────────────────────

/// Parse a single skill Markdown file into a [`SkillFile`].
///
/// Expected layout:
/// ```text
/// ---
/// name: samtools
/// category: alignment
/// description: ...
/// tags: [bam, sam]
/// author: ...
/// source_url: "..."
/// ---
/// ## Concepts
/// ...
/// ## Examples
/// ### task description
/// **Args:** `flags`
/// **Explanation:** text
/// ```
pub fn parse_skill_file(content: &str) -> anyhow::Result<SkillFile> {
    // ── 1. Extract YAML front-matter ──────────────────────────────────────
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("Missing YAML front-matter delimiters");
    }
    let front = parts[1].trim();
    let body = parts[2];

    let name = extract_yaml_value(front, "name").unwrap_or_default();
    let category = extract_yaml_value(front, "category").unwrap_or_default();
    let description = extract_yaml_value(front, "description").unwrap_or_default();
    let source_url = extract_yaml_value(front, "source_url")
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();

    let tags = extract_yaml_value(front, "tags")
        .map(|s| {
            s.trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // ── 2. Parse examples ─────────────────────────────────────────────────
    let examples = parse_examples(body);

    Ok(SkillFile {
        name,
        category,
        description,
        tags,
        source_url,
        examples,
    })
}

/// Extract a simple `key: value` from YAML-like text.
fn extract_yaml_value(yaml: &str, key: &str) -> Option<String> {
    for line in yaml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix(':') {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

/// Parse the `## Examples` section of a skill file body.
fn parse_examples(body: &str) -> Vec<SkillExample> {
    let mut examples = Vec::new();
    let mut in_examples = false;
    let mut current_task: Option<String> = None;
    let mut current_args: Option<String> = None;
    let mut current_explanation: Option<String> = None;

    for line in body.lines() {
        let trimmed = line.trim();

        // Detect ## Examples section.
        if trimmed.starts_with("## ") {
            if trimmed.eq_ignore_ascii_case("## examples")
                || trimmed.eq_ignore_ascii_case("## Examples")
                || trimmed.starts_with("## Example")
            {
                in_examples = true;
            } else if in_examples {
                // Another ## section after examples → stop.
                break;
            }
            continue;
        }

        if !in_examples {
            continue;
        }

        // ### heading → new example.
        if let Some(heading) = trimmed.strip_prefix("### ") {
            // Flush previous example.
            if let (Some(task), Some(args)) = (current_task.take(), current_args.take()) {
                examples.push(SkillExample {
                    task,
                    args,
                    explanation: current_explanation.take().unwrap_or_default(),
                });
            }
            current_task = Some(heading.to_string());
            current_args = None;
            current_explanation = None;
            continue;
        }

        // **Args:** `...`
        if let Some(rest) = trimmed
            .strip_prefix("**Args:**")
            .or_else(|| trimmed.strip_prefix("**Args: **"))
        {
            let args = rest
                .trim()
                .trim_start_matches('`')
                .trim_end_matches('`')
                .to_string();
            current_args = Some(args);
            continue;
        }

        // **Explanation:** ...
        if let Some(rest) = trimmed
            .strip_prefix("**Explanation:**")
            .or_else(|| trimmed.strip_prefix("**Explanation: **"))
        {
            current_explanation = Some(rest.trim().to_string());
        }
    }

    // Flush final example.
    if let (Some(task), Some(args)) = (current_task, current_args) {
        examples.push(SkillExample {
            task,
            args,
            explanation: current_explanation.unwrap_or_default(),
        });
    }

    examples
}

/// Load all skill files from a directory (files matching `*.md`).
pub fn load_skills_from_dir(dir: &Path) -> anyhow::Result<Vec<SkillFile>> {
    let mut skills = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        match parse_skill_file(&content) {
            Ok(skill) if !skill.examples.is_empty() => skills.push(skill),
            Ok(_) => {
                eprintln!(
                    "  warning: {} has no examples, skipping",
                    entry.path().display()
                );
            }
            Err(e) => {
                eprintln!("  warning: failed to parse {}: {e}", entry.path().display());
            }
        }
    }
    Ok(skills)
}

// ── Benchmark tool exclusion ─────────────────────────────────────────────────

/// Tools excluded from benchmarks.
///
/// These fall into three categories per the project guidelines:
/// - **Package managers** — conda, mamba, pip, pixi, cargo, docker, singularity
/// - **HPC schedulers** — slurm, pbs, sge, lsf, htcondor, kubectl
/// - **AI assistants** — claude, openclaw
///
/// The benchmark targets basic bash/shell commands + bioconda bioinformatics
/// tools only.
pub const EXCLUDED_TOOLS: &[&str] = &[
    // Package managers / containers
    "conda",
    "mamba",
    "pip",
    "pixi",
    "cargo",
    "docker",
    "singularity",
    // HPC schedulers
    "slurm",
    "pbs",
    "sge",
    "lsf",
    "htcondor",
    "kubectl",
    // AI assistants
    "claude",
    "openclaw",
];

/// Return `true` if the tool should be excluded from benchmark evaluation.
pub fn is_excluded_tool(name: &str) -> bool {
    EXCLUDED_TOOLS.contains(&name)
}

/// Load skill files from a directory, excluding tools not suitable for
/// benchmarking (package managers, HPC schedulers, AI assistants).
pub fn load_skills_for_bench(dir: &Path) -> anyhow::Result<Vec<SkillFile>> {
    let mut skills = load_skills_from_dir(dir)?;
    let before = skills.len();
    skills.retain(|s| !is_excluded_tool(&s.name));
    let excluded = before - skills.len();
    if excluded > 0 {
        eprintln!("  info: excluded {excluded} non-benchmark tool(s) (pkg managers, HPC, AI)");
    }
    Ok(skills)
}

// ── File token extraction & task enrichment ──────────────────────────────────

/// Known file extensions common in bioinformatics, genomics, and general CLI
/// workflows.  Used by [`extract_file_tokens`] to identify file-like tokens
/// inside a reference-args string.
const KNOWN_FILE_EXTENSIONS: &[&str] = &[
    // Alignment / mapping
    "bam",
    "sam",
    "cram",
    "bai",
    "csi",
    // Sequence
    "fasta",
    "fa",
    "fna",
    "faa",
    "fastq",
    "fq",
    // Compressed
    "gz",
    "bz2",
    "xz",
    "zip",
    "tar",
    // Variant calling
    "vcf",
    "bcf",
    // Annotation
    "gff",
    "gff3",
    "gtf",
    "bed",
    "bedgraph",
    "bedmethyl",
    // Tabular / text
    "txt",
    "tsv",
    "csv",
    "log",
    "out",
    "tbl",
    "table",
    "results",
    // Report / visualisation
    "html",
    "pdf",
    "png",
    "svg",
    "json",
    "xml",
    // Config
    "yaml",
    "yml",
    "toml",
    "conf",
    "cfg",
    "config",
    "ini",
    // Index files
    "idx",
    "tbi",
    "fai",
    "dict",
    "index",
    "mmi",
    // Genomics-specific
    "bw",
    "bigwig",
    "wig",
    "tdf",
    "paf",
    "delta",
    "coords",
    "hmm",
    "meme",
    "sfs",
    "2dsfs",
    "saf",
    "pileup",
    "nwk",
    "phy",
    "bracken",
    "starch",
    "cool",
    "h5",
    "hdf5",
    "loom",
    "h5ad",
    "npz",
    "sra",
    "gbk",
    "maf",
    "m8",
    "gfa",
    "snf",
    "sig",
    "meryl",
    "iso",
    "cnn",
    "cns",
    "cnr",
    "5m",
    "db",
    "dmp",
    "snps",
    "msh",
    "map",
    "add",
    "sto",
    "excl",
    "list",
    "in",
    // Script / companion-binary extensions (important for the LLM)
    "py",
    "pl",
    "sh",
    "jl",
    "nf",
    "lua",
    // R scripts (case-sensitive, handled separately)
    "r",
    "rmd",
    // Archive / Java
    "jar",
    // Env
    "env",
];

/// Simple shell-like tokenisation: split by whitespace while respecting
/// single- and double-quoted spans.
fn shell_tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in s.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(ch);
            }
            '"' if !in_single => {
                in_double = !in_double;
                current.push(ch);
            }
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Return `true` when `token` looks like a file path (contains a dot with a
/// known extension).
fn is_file_like_token(token: &str) -> bool {
    if token.is_empty() || token.starts_with('-') || token.contains("://") {
        return false;
    }
    if !token.contains('.') {
        return false;
    }
    // Skip pure numbers (e.g. "1e6", "0.05")
    if token.parse::<f64>().is_ok() {
        return false;
    }

    // Skip tokens that look like qualified function / method calls (e.g.
    // `Pkg.add`, `sys.stdin`, `os.path`) — the "extension" part is a
    // function/attribute name, not a file extension.  A file extension is
    // typically all-lowercase and 1-6 chars; reject tokens where the part
    // after the last dot is a common programming method name.
    if let Some(stem) = token.rsplit('.').nth(1) {
        // If the stem starts with an uppercase letter and ends with a
        // lower-case part, it's likely Module.function (e.g. Pkg.add).
        if stem.starts_with(|ch: char| ch.is_ascii_uppercase()) {
            let ext_part = token.rsplit('.').next().unwrap_or("");
            let common_methods = [
                "add", "install", "load", "read", "write", "open", "close", "get", "set", "run",
                "call", "new", "init", "start", "stop", "stdin", "stdout", "stderr", "path",
                "join",
            ];
            if common_methods.contains(&ext_part.to_lowercase().as_str()) {
                return false;
            }
        }
    }

    // Check for known extension.  We compare lower-case except for the
    // special R extensions (.R, .Rmd) which are matched case-insensitively
    // by the lower-cased KNOWN_FILE_EXTENSIONS list.
    let lower = token.to_lowercase();
    KNOWN_FILE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

/// Shell metacharacters to strip from candidate tokens before checking if
/// they look like file paths.
const SHELL_METACHARS: &[char] = &[
    '\'', '"', '(', ')', '<', '>', ';', '&', '|', '`', '$', '{', '}',
];

/// Characters used to split sub-tokens inside quoted expressions (commas,
/// semicolons, parentheses, etc.) so that file names embedded in function
/// calls like `rmarkdown::render('report.Rmd', ...)` are discovered.
const INNER_SPLIT_CHARS: &[char] = &[',', ';', '(', ')', '[', ']'];

/// Extract file-like tokens from a reference-args string.
///
/// Handles bare tokens (`input.bam`), key=value pairs (`in=reads.fastq.gz`),
/// `--flag=value` forms (`--output=results.vcf`), colon-separated fields
/// (`ILLUMINACLIP:TruSeq3-PE.fa:2:30:10`), and files embedded in quoted
/// shell commands (`-c 'sort file.txt'`).  Also splits by commas,
/// semicolons, and parentheses inside quoted strings so that files inside
/// function calls (e.g. `render('report.Rmd', ...)`) are found.
///
/// Returns unique tokens in the order they first appear.
pub fn extract_file_tokens(args: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let try_add =
        |candidate: &str, seen: &mut std::collections::HashSet<String>, files: &mut Vec<String>| {
            let c = candidate.trim_matches(|ch: char| SHELL_METACHARS.contains(&ch));
            if is_file_like_token(c) && seen.insert(c.to_string()) {
                files.push(c.to_string());
            }
            // Also try colon-separated sub-parts (e.g. ILLUMINACLIP:TruSeq3-PE.fa:2:30:10,
            // or chromsizes.txt:5000).
            if c.contains(':') {
                for part in c.split(':') {
                    let part = part.trim_matches(|ch: char| SHELL_METACHARS.contains(&ch));
                    if is_file_like_token(part) && seen.insert(part.to_string()) {
                        files.push(part.to_string());
                    }
                }
            }
        };

    for raw in shell_tokenize(args) {
        // For quoted strings, also search inside the quoted content.
        let is_quoted = (raw.starts_with('\'') && raw.ends_with('\''))
            || (raw.starts_with('"') && raw.ends_with('"'));
        if is_quoted && raw.len() > 2 {
            let inner = &raw[1..raw.len() - 1];
            // Split by whitespace first, then by inner-split chars
            // (commas, parens, semicolons) to find files inside function
            // calls like render('report.Rmd', output_format=...).
            for sub in inner.split_whitespace() {
                try_add(sub, &mut seen, &mut files);
                // Further split by commas, semicolons, parentheses.
                for part in sub.split(|ch: char| INNER_SPLIT_CHARS.contains(&ch)) {
                    let part = part.trim();
                    if !part.is_empty() {
                        try_add(part, &mut seen, &mut files);
                    }
                }
            }
            continue;
        }

        // Collect candidate values: either the whole token or the rhs of `=`.
        let candidates: Vec<&str> = if raw.contains('=') {
            // Both `key=value` and `--flag=value` forms.
            raw.splitn(2, '=')
                .nth(1)
                .map(|v| vec![v])
                .unwrap_or_default()
        } else {
            vec![raw.as_str()]
        };

        for c in &candidates {
            try_add(c, &mut seen, &mut files);
        }
    }
    files
}

/// Extract package / module identifiers from a reference-args string.
///
/// Recognises:
///
/// - **R packages** — names inside `install.packages('PKG')`,
///   `packageVersion('PKG')`, `BiocManager::install(c('PKG1','PKG2'))`,
///   `library(PKG)`, `require(PKG)`, `pak::pkg_install('user/repo')`.
/// - **Python modules** — `import MODULE`, `from MODULE import ...`.
///
/// Returns unique identifiers in the order they first appear.  Identifiers
/// that look like file paths (contain a known extension) are excluded since
/// they are already handled by [`extract_file_tokens`].
pub fn extract_package_identifiers(args: &str) -> Vec<String> {
    let mut pkgs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Helper: add a candidate if it looks like a valid package/module name
    // and is not a known file.
    let mut try_add = |name: &str| {
        let name = name.trim().trim_matches(|ch: char| {
            ch == '\'' || ch == '"' || ch == '(' || ch == ')' || ch == ',' || ch == ' '
        });
        if name.is_empty() || name.len() < 2 {
            return;
        }
        // Skip if it looks like a file (already handled by extract_file_tokens).
        if is_file_like_token(name) {
            return;
        }
        // Skip URLs.
        if name.contains("://") {
            return;
        }
        // Skip pure numbers or very short strings.
        if name.parse::<f64>().is_ok() {
            return;
        }
        // Must look like a valid identifier: starts with a letter, contains
        // only alphanumeric, dots, hyphens, underscores, or slashes (for
        // user/repo GitHub references).
        if !name.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
            return;
        }
        if !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ".-_/".contains(ch))
        {
            return;
        }
        if seen.insert(name.to_string()) {
            pkgs.push(name.to_string());
        }
    };

    // ── R package patterns ───────────────────────────────────────────────
    let r_patterns = [
        "install.packages(",
        "packageVersion(",
        "BiocManager::install(",
        "library(",
        "require(",
        "pak::pkg_install(",
        "pak::pak(",
        "remotes::install_github(",
        "devtools::install_github(",
        "rmarkdown::render(",
    ];

    for pat in &r_patterns {
        let search = args;
        let mut start = 0;
        while let Some(pos) = search[start..].find(pat) {
            let abs_pos = start + pos + pat.len();
            // Find the closing paren (handle nested parens for c(...)).
            if let Some(close) = find_matching_paren(&search[abs_pos..]) {
                let inner = &search[abs_pos..abs_pos + close];
                // Split by comma and extract quoted names.
                for part in inner.split(',') {
                    let part = part
                        .trim()
                        .trim_matches(|ch: char| ch == '\'' || ch == '"' || ch == ' ');
                    // Handle c('A','B') wrapper.
                    let part = part.strip_prefix("c(").unwrap_or(part);
                    let part = part
                        .trim_matches(|ch: char| ch == '\'' || ch == '"' || ch == '(' || ch == ')');
                    try_add(part);
                }
            }
            start = abs_pos;
        }
    }

    // ── Python import patterns ───────────────────────────────────────────
    // `import json`, `from os import path`, `import sys`
    for keyword in &["import ", "from "] {
        let search = args;
        let mut start = 0;
        while let Some(pos) = search[start..].find(keyword) {
            let abs_pos = start + pos;
            // Ensure the keyword is preceded by whitespace, semicolon,
            // quote, or is at the start of the string.
            let valid_prefix = abs_pos == 0 || {
                let prev = search.as_bytes()[abs_pos - 1];
                prev.is_ascii_whitespace() || prev == b';' || prev == b'"' || prev == b'\''
            };
            if valid_prefix {
                let after = &search[abs_pos + keyword.len()..];
                // Grab the first word (the module name).
                let mod_name: String = after
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.')
                    .collect();
                if !mod_name.is_empty() {
                    // Only include the top-level module name for brevity.
                    let top = mod_name.split('.').next().unwrap_or(&mod_name);
                    try_add(top);
                }
            }
            start = abs_pos + keyword.len();
        }
    }

    pkgs
}

/// Find the position of the matching closing parenthesis for an opening
/// paren that has already been consumed.  Returns `None` if not found.
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut in_single = false;
    let mut in_double = false;
    for (i, ch) in s.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '(' if !in_single && !in_double => depth += 1,
            ')' if !in_single && !in_double => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Enrich a task description by appending file/path tokens **and** package
/// identifiers from `args` that are not already mentioned in `task`.
///
/// If no new tokens need to be added the original task string is returned
/// unchanged.
pub fn enrich_task_with_files(task: &str, args: &str) -> String {
    let files = extract_file_tokens(args);
    let packages = extract_package_identifiers(args);

    let missing: Vec<&str> = files
        .iter()
        .chain(packages.iter())
        .filter(|f| !task.contains(f.as_str()))
        .map(|s| s.as_str())
        .collect();

    if missing.is_empty() {
        return task.to_string();
    }

    // Deduplicate while preserving order (files already deduped, but
    // packages may overlap).
    let mut seen = std::collections::HashSet::new();
    let deduped: Vec<&str> = missing.into_iter().filter(|s| seen.insert(*s)).collect();

    format!("{} ({})", task, deduped.join(", "))
}

// ── Shell metacharacter stripping ─────────────────────────────────────────────

/// Strip shell metacharacters from a reference-args string to produce a clean
/// single-command invocation suitable for benchmark evaluation.
///
/// Specifically:
/// - **Pipes** (`|`): only the first command in the pipeline is kept
///   (e.g. `mpileup -f ref.fa input.bam | call -m` → `mpileup -f ref.fa input.bam`).
/// - **Output redirections** (`>`, `>>`, `2>`, `2>>`): the redirection and its
///   target are removed.
/// - **Input redirections** (`<`): the `<` operator is removed but the file
///   target is kept (it becomes a positional argument).
/// - **Tee** (`| tee file`): treated as a pipe and removed.
/// - **Subshell / process substitution** is left as-is when inside quotes
///   (the shell_tokenize function handles this).
///
/// The function preserves content inside single- and double-quoted strings
/// to avoid mangling embedded shell commands (e.g. `awk '{print $1}'`).
pub fn strip_shell_metacharacters(args: &str) -> String {
    // Step 1: Truncate at the first unquoted pipe `|`.
    let truncated = truncate_at_pipe(args);

    // Step 2: Remove output redirections (>, >>, 2>, 2>>).
    let cleaned = remove_redirections(truncated.trim());

    cleaned.trim().to_string()
}

/// Truncate a command string at the first unquoted pipe (`|`).
///
/// Handles single- and double-quoted strings so that pipes inside quotes
/// (e.g. `awk -F'|' '{print $1}'`) are not treated as pipeline separators.
fn truncate_at_pipe(s: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;

    for (i, ch) in s.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '|' if !in_single && !in_double => return &s[..i],
            _ => {}
        }
    }
    s
}

/// Remove output-redirection operators and their target tokens.
///
/// Handles `>file`, `> file`, `>>file`, `>> file`, `2>file`, `2> file`,
/// `2>>file`, and `2>> file`.  The `<` (input redirect) operator is removed
/// but its target is kept as a positional argument.
fn remove_redirections(s: &str) -> String {
    let tokens = shell_tokenize(s);
    let mut result = Vec::new();
    let mut skip_next = false;

    for token in &tokens {
        if skip_next {
            skip_next = false;
            continue;
        }

        let t = token.as_str();

        // Patterns: ">", ">>", "2>", "2>>" — skip this token and the next one
        if t == ">" || t == ">>" || t == "2>" || t == "2>>" {
            skip_next = true;
            continue;
        }

        // Pattern: ">file" or ">>file" — skip entire token
        if t.starts_with(">>") || (t.starts_with('>') && t.len() > 1) {
            continue;
        }

        // Pattern: "2>file" or "2>>file" — skip entire token
        if t.starts_with("2>>") || (t.starts_with("2>") && t.len() > 2) {
            continue;
        }

        // Pattern: "<" — remove the operator but keep the target file
        if t == "<" {
            // Keep the next token (the file) as a positional argument
            continue;
        }

        // Pattern: "<file" — strip the < prefix, keep the file
        if t.starts_with('<') && t.len() > 1 {
            let file = &t[1..];
            if !file.is_empty() {
                result.push(file.to_string());
            }
            continue;
        }

        result.push(token.clone());
    }

    result.join(" ")
}

// ── Boilerplate flag stripping ────────────────────────────────────────────────

/// Common threading/parallelism flag patterns that appear in skill-file
/// examples but are not part of the *task* being described.  These are
/// matched as `(flag_regex, value_regex)` pairs against tokenised args.
///
/// The list is intentionally conservative — only patterns that are
/// **universally** boilerplate (threading, quality-filter defaults) and
/// that would never convey task semantics.
///
/// Each entry: `(tool_name, &[(flag_token_regex, value_token_regex)])`.
///
/// NOTE: This is a curated, tool-specific map.  Tools not listed here are
/// left untouched.  If a tool uses `-p` for *ploidy* (freebayes) or
/// `-p` for *port* (ssh), it must **not** be in this table.
use regex::Regex;
use std::sync::LazyLock;

/// A compiled boilerplate-flag pattern: matches against `reference_args`.
struct BoilerplatePattern {
    /// Regex matching the flag+value span (with optional leading whitespace).
    re: Regex,
}

/// Tool-specific boilerplate patterns.
static TOOL_BOILERPLATE: LazyLock<
    std::collections::HashMap<&'static str, Vec<BoilerplatePattern>>,
> = LazyLock::new(|| {
    // Helper: build patterns that match at start-of-string or after whitespace.
    fn pats(raw: &[&str]) -> Vec<BoilerplatePattern> {
        raw.iter()
            .flat_map(|p| {
                vec![
                    // mid-string: preceded by whitespace
                    BoilerplatePattern {
                        re: Regex::new(&format!(r"\s+{p}")).unwrap(),
                    },
                    // start-of-string
                    BoilerplatePattern {
                        re: Regex::new(&format!(r"^{p}\s*")).unwrap(),
                    },
                ]
            })
            .collect()
    }

    let mut m = std::collections::HashMap::new();

    // ── Threading flags ──────────────────────────────────────────────
    m.insert("admixture", pats(&[r"-j\d+"]));
    m.insert(
        "angsd",
        pats(&[
            r"-nThreads\s+\d+",
            r"-P\s+\d+",
            r"-minMapQ\s+\d+",
            r"-minQ\s+\d+",
            r"-minInd\s+\d+",
        ]),
    );
    m.insert("arriba", pats(&[r"--runThreadN\s+\d+"]));
    m.insert("bakta", pats(&[r"--threads\s+\d+"]));
    m.insert("bismark", pats(&[r"-p\s+\d+"]));
    m.insert("bowtie2", pats(&[r"-p\s+\d+"]));
    m.insert("bracken", pats(&[r"-t\s+\d+"]));
    m.insert("bwa", pats(&[r"-t\s+\d+"]));
    m.insert("bwa-mem2", pats(&[r"-t\s+\d+"]));
    m.insert("cellsnp-lite", pats(&[r"-p\s+\d+"]));
    m.insert("centrifuge", pats(&[r"-p\s+\d+"]));
    m.insert("checkm2", pats(&[r"--threads\s+\d+"]));
    m.insert("chopper", pats(&[r"--threads\s+\d+"]));
    m.insert("chromap", pats(&[r"-t\s+\d+"]));
    m.insert("cnvkit", pats(&[r"-p\s+\d+"]));
    m.insert("cutadapt", pats(&[r"-j\s+\d+"]));
    m.insert("deeptools", pats(&[r"-p\s+\d+"]));
    m.insert("diamond", pats(&[r"--threads\s+\d+"]));
    m.insert("fastani", pats(&[r"--threads\s+\d+"]));
    m.insert("fastq-screen", pats(&[r"--threads\s+\d+"]));
    m.insert("fastqc", pats(&[r"-t\s+\d+"]));
    m.insert("featurecounts", pats(&[r"-T\s+\d+"]));
    m.insert("flye", pats(&[r"--threads\s+\d+"]));
    m.insert("gtdbtk", pats(&[r"--cpus\s+\d+"]));
    m.insert("hifiasm", pats(&[r"-t\s+\d+"]));
    m.insert("hisat2", pats(&[r"-p\s+\d+"]));
    m.insert("homer", pats(&[r"-p\s+\d+"]));
    m.insert("iqtree2", pats(&[r"-T\s+(?:AUTO|\d+)"]));
    m.insert("java", pats(&[r"--threads\s+\d+"]));
    m.insert("kallisto", pats(&[r"--threads[= ]\d+"]));
    m.insert("kb", pats(&[r"-t\s+\d+"]));
    m.insert("kraken2", pats(&[r"--threads\s+\d+"]));
    m.insert("liftoff", pats(&[r"-p\s+\d+"]));
    m.insert("mafft", pats(&[r"--thread\s+\d+"]));
    m.insert("mash", pats(&[r"-p\s+\d+"]));
    m.insert("medaka", pats(&[r"-t\s+\d+"]));
    m.insert("metabat2", pats(&[r"-t\s+\d+"]));
    m.insert("miniasm", pats(&[r"-t\s+\d+"]));
    m.insert("minimap2", pats(&[r"-t\s+\d+"]));
    m.insert("mmseqs2", pats(&[r"--threads\s+\d+"]));
    m.insert("modkit", pats(&[r"--threads\s+\d+", r"-t\s+\d+"]));
    m.insert("mosdepth", pats(&[r"-t\s+\d+"]));
    m.insert("nanocomp", pats(&[r"--threads\s+\d+"]));
    m.insert("nanoplot", pats(&[r"--threads\s+\d+"]));
    m.insert("nanostat", pats(&[r"-t\s+\d+"]));
    m.insert("orthofinder", pats(&[r"-t\s+\d+"]));
    m.insert("pbccs", pats(&[r"-j\s+\d+", r"--min-rq\s+[\d.]+"]));
    m.insert("pbfusion", pats(&[r"--threads\s+\d+"]));
    m.insert("pbmm2", pats(&[r"-j\s+\d+", r"--sort-threads\s+\d+"]));
    m.insert("pilon", pats(&[r"--threads\s+\d+"]));
    m.insert("porechop", pats(&[r"--threads\s+\d+"]));
    m.insert("prokka", pats(&[r"--cpus\s+\d+"]));
    m.insert("quast", pats(&[r"--threads\s+\d+"]));
    m.insert("racon", pats(&[r"-t\s+\d+"]));
    m.insert("rsem", pats(&[r"--num-threads\s+\d+"]));
    m.insert("salmon", pats(&[r"--threads\s+\d+", r"-p\s+\d+"]));
    m.insert("samtools", pats(&[r"-@\s*\d+"]));
    m.insert("seqkit", pats(&[r"-j\s+\d+"]));
    m.insert("shapeit4", pats(&[r"--thread\s+\d+"]));
    m.insert("snakemake", pats(&[r"--cores\s+\d+"]));
    m.insert("sniffles", pats(&[r"--threads\s+\d+"]));
    m.insert("spades", pats(&[r"--threads\s+\d+"]));
    m.insert("star", pats(&[r"--runThreadN\s+\d+"]));
    m.insert("strelka2", pats(&[r"-j\s+\d+"]));
    m.insert("stringtie", pats(&[r"-p\s+\d+"]));
    m.insert("trim_galore", pats(&[r"--cores\s+\d+"]));
    m.insert("vcfanno", pats(&[r"-p\s+\d+"]));
    m.insert("verkko", pats(&[r"--threads\s+\d+"]));
    m.insert("wtdbg2", pats(&[r"-t\s+\d+"]));

    m
});

/// Strip the parenthetical file-list suffix from a task description so that
/// file names like `filtered.txt` don't accidentally trigger keyword checks.
fn strip_file_list_suffix(desc: &str) -> &str {
    // The enriched task has the form "some text (file1, file2)".
    // Find the last '(' that is followed only by file-like tokens and ')'.
    if let Some(pos) = desc.rfind(" (") {
        &desc[..pos]
    } else {
        desc
    }
}

/// Return `true` when the task description mentions threading/parallelism
/// concepts, meaning the threading flags in reference_args are intentional.
fn task_mentions_threading(task: &str) -> bool {
    let base = strip_file_list_suffix(task).to_lowercase();
    if base.contains("thread")
        || base.contains("parallel")
        || base.contains("concurrent")
        || base.contains("multithread")
        || base.contains("-j")
    {
        return true;
    }
    // Word-boundary aware checks to avoid false positives
    // (e.g. "Score" containing "core").
    let core_re = Regex::new(r"\bcores?\b").unwrap();
    let cpu_re = Regex::new(r"\bcpus?\b").unwrap();
    core_re.is_match(&base) || cpu_re.is_match(&base)
}

/// Return `true` when the task description mentions quality filtering.
fn task_mentions_quality_filter(task: &str) -> bool {
    let base = strip_file_list_suffix(task).to_lowercase();
    base.contains("quality")
        || base.contains("filter")
        || base.contains("mapq")
        || base.contains("min-rq")
}

/// Strip boilerplate flags (threading, quality-filter defaults, etc.) from
/// `reference_args` that are **not** described in `task_description`.
///
/// This ensures the benchmark fairly evaluates LLM command generation:
/// the LLM should only be expected to produce flags that are inferable
/// from the natural-language task description.
pub fn strip_boilerplate_flags(tool: &str, args: &str, task: &str) -> String {
    let patterns = match TOOL_BOILERPLATE.get(tool) {
        Some(pats) => pats,
        None => return args.to_string(),
    };

    // For ANGSD, quality-filter patterns are in the same table;
    // skip quality-filter stripping if the task mentions quality/filtering.
    let skip_quality = tool == "angsd" && task_mentions_quality_filter(task);

    // Skip threading stripping if the task mentions threading.
    let skip_threading = task_mentions_threading(task);

    // If both are skipped, nothing to do.
    if skip_quality && skip_threading {
        return args.to_string();
    }

    let mut result = args.to_string();
    for bp in patterns {
        let pat_str = bp.re.as_str();
        // Classify: quality-filter patterns for ANGSD
        let is_quality = pat_str.contains("minMapQ")
            || pat_str.contains("minQ")
            || pat_str.contains("minInd")
            || pat_str.contains("min-rq");
        if is_quality && skip_quality {
            continue;
        }
        if !is_quality && skip_threading {
            continue;
        }
        // Start-of-string patterns replace with empty; mid-string with space.
        if pat_str.starts_with('^') {
            result = bp.re.replace(&result, "").to_string();
        } else {
            result = bp.re.replace(&result, " ").to_string();
        }
    }

    // Collapse multiple spaces and trim.
    let re_multi_space = Regex::new(r"  +").unwrap();
    re_multi_space.replace_all(result.trim(), " ").to_string()
}

// ── Scenario generation ──────────────────────────────────────────────────────

/// Number of scenarios to generate per tool.
pub const SCENARIOS_PER_TOOL: usize = 10;

/// Generate exactly [`SCENARIOS_PER_TOOL`] scenarios from a parsed skill file.
///
/// If the skill has fewer than 10 examples the remaining slots are filled with
/// synthesised variants that recombine flags from existing examples.
///
/// **Boilerplate stripping**: Threading, quality-filter defaults, and other
/// performance-only flags that are not described in the task heading are
/// automatically removed from `reference_args` so that the benchmark fairly
/// evaluates the LLM's ability to infer flags from the task description.
///
/// File and path tokens in reference_args are **substituted** with alternative
/// names to avoid information leakage from the original skill examples.  The
/// task description is then enriched with the substituted names so the LLM has
/// the correct filenames.
pub fn generate_scenarios(skill: &SkillFile) -> Vec<Scenario> {
    let mut scenarios: Vec<Scenario> = skill
        .examples
        .iter()
        .take(SCENARIOS_PER_TOOL)
        .enumerate()
        .map(|(i, ex)| {
            let cleaned = strip_shell_metacharacters(&ex.args);
            let cleaned = strip_boilerplate_flags(&skill.name, &cleaned, &ex.task);
            Scenario {
                tool: skill.name.clone(),
                scenario_id: format!("{}_{:02}", skill.name, i + 1),
                reference_args: cleaned,
                task_description: ex.task.clone(),
                category: skill.category.clone(),
            }
        })
        .collect();

    // Pad to SCENARIOS_PER_TOOL with synthesised variants.
    if scenarios.len() < SCENARIOS_PER_TOOL && !skill.examples.is_empty() {
        let originals = scenarios.clone();
        let mut idx = scenarios.len();
        let mut source_idx = 0;

        while scenarios.len() < SCENARIOS_PER_TOOL {
            let base = &originals[source_idx % originals.len()];
            let variant = synthesise_variant(base, idx);
            scenarios.push(variant);
            idx += 1;
            source_idx += 1;
        }
    }

    // ── Substitute file/path tokens to avoid information leakage ─────────
    for scenario in &mut scenarios {
        substitute_file_tokens(scenario);
    }

    // ── Enrich task descriptions with file references ─────────────────────
    for scenario in &mut scenarios {
        scenario.task_description =
            enrich_task_with_files(&scenario.task_description, &scenario.reference_args);
    }

    scenarios
}

// ── File token substitution (anti-leakage) ───────────────────────────────────

/// Alternative base names for file substitution, grouped by semantic role.
/// These replace the original base names from skill.md examples so that
/// benchmark scenarios use different (but realistic) file names.
const ALT_BASES: &[&str] = &[
    "sample",
    "reads",
    "genome",
    "results",
    "aligned",
    "filtered",
    "merged",
    "variants",
    "counts",
    "matrix",
    "report",
    "analysis",
    "processed",
    "raw",
    "reference",
    "assembly",
    "trimmed",
    "sorted",
    "annotated",
    "mapping",
];

/// Deterministic hash for selecting alternative names.
fn scenario_hash(input: &str) -> u64 {
    let mut h: u64 = 14695981039346656037; // FNV-1a offset basis
    for byte in input.bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(1099511628211); // FNV-1a prime
    }
    h
}

/// Generate an alternative base name for a file token, preserving the
/// extension and any directory prefix.
///
/// E.g. `input.bam` → `sample.bam`, `data/reads.fastq.gz` → `data/mapping.fastq.gz`
#[cfg(test)]
fn alternative_filename(original: &str, scenario_id: &str) -> String {
    // Split off directory prefix if present.
    let (dir_prefix, filename) = if let Some(pos) = original.rfind('/') {
        (&original[..=pos], &original[pos + 1..])
    } else {
        ("", original)
    };

    // Split filename into base and extension(s).
    // Handle compound extensions like .fastq.gz, .vcf.gz, .tar.gz
    let (base, ext) = split_base_ext(filename);

    // Pick an alternative base name deterministically.
    let h = scenario_hash(&format!("{scenario_id}:{original}"));
    let alt_base = ALT_BASES[h as usize % ALT_BASES.len()];

    // Avoid collision: if the alternative is the same as the original base,
    // pick the next one.
    let alt_base = if alt_base == base {
        ALT_BASES[(h as usize + 1) % ALT_BASES.len()]
    } else {
        alt_base
    };

    format!("{dir_prefix}{alt_base}{ext}")
}

/// Like [`alternative_filename`] but ensures the generated name is unique
/// within the current scenario by checking against `used`.  If the initial
/// candidate collides, it rotates through `ALT_BASES` until a unique name is
/// found.
fn alternative_filename_unique(
    original: &str,
    scenario_id: &str,
    used: &mut std::collections::HashSet<String>,
) -> String {
    let (dir_prefix, filename) = if let Some(pos) = original.rfind('/') {
        (&original[..=pos], &original[pos + 1..])
    } else {
        ("", original)
    };

    let (base, ext) = split_base_ext(filename);
    let h = scenario_hash(&format!("{scenario_id}:{original}"));

    // Try up to ALT_BASES.len() rotations to find a unique name.
    for offset in 0..ALT_BASES.len() {
        let candidate = ALT_BASES[(h as usize + offset) % ALT_BASES.len()];
        // Skip if same as the original base name.
        if candidate == base {
            continue;
        }
        let full = format!("{dir_prefix}{candidate}{ext}");
        if used.insert(full.clone()) {
            return full;
        }
    }

    // Fallback: if all ALT_BASES exhausted (unlikely with 20 candidates),
    // append a numeric suffix.
    let fallback = format!("{dir_prefix}{base}_{}{ext}", h % 100);
    used.insert(fallback.clone());
    fallback
}

/// Split a filename into (base, extension_including_dot).
/// Handles compound extensions: "reads.fastq.gz" → ("reads", ".fastq.gz").
fn split_base_ext(filename: &str) -> (&str, &str) {
    // Known compound extensions (checked first).
    let compound_exts = [
        ".fastq.gz",
        ".fasta.gz",
        ".fa.gz",
        ".fq.gz",
        ".vcf.gz",
        ".bed.gz",
        ".gff.gz",
        ".gtf.gz",
        ".sam.gz",
        ".tsv.gz",
        ".csv.gz",
        ".txt.gz",
        ".tar.gz",
        ".tar.bz2",
        ".tar.xz",
        ".saf.idx",
        ".saf.gz",
    ];

    let lower = filename.to_lowercase();
    for ext in &compound_exts {
        if lower.ends_with(ext) {
            let base_end = filename.len() - ext.len();
            return (&filename[..base_end], &filename[base_end..]);
        }
    }

    // Single extension.
    if let Some(dot_pos) = filename.rfind('.') {
        if dot_pos > 0 {
            return (&filename[..dot_pos], &filename[dot_pos..]);
        }
    }

    (filename, "")
}

/// Recognised script extensions for companion-binary / script-executable
/// detection within the benchmark scenario generator.  These are tokens that
/// should **never** be renamed by the anti-leakage substitution because they
/// are part of the command syntax, not data file names.
const SCRIPT_EXTS: &[&str] = &[".sh", ".py", ".pl", ".R", ".rb", ".jl", ".nf", ".lua"];

/// Return `true` when `token` looks like a script executable name
/// (e.g. `agat_convert_sp_gff2gtf.pl`, `bbduk.sh`, `infer_experiment.py`).
fn is_script_name(token: &str) -> bool {
    // Must not contain path separators (then it's a path, not a bare script name).
    if token.contains('/') || token.contains('\\') {
        return false;
    }
    for ext in SCRIPT_EXTS {
        if let Some(stem) = token.strip_suffix(ext) {
            return !stem.is_empty()
                && stem
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        }
    }
    false
}

/// Return `true` when `token` is a path to a script (e.g.
/// `strelka_germline/runWorkflow.py`) — these should not be renamed because
/// they are part of the command invocation, not data files.
fn is_script_path(token: &str) -> bool {
    for ext in SCRIPT_EXTS {
        if token.ends_with(ext) {
            // The basename (after the last /) should look like a script name.
            if let Some(pos) = token.rfind('/') {
                let basename = &token[pos + 1..];
                if let Some(stem) = basename.strip_suffix(ext) {
                    return !stem.is_empty()
                        && stem
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
                }
            }
            return false;
        }
    }
    false
}

/// Extract the first whitespace-separated token from `args`.  This is the
/// subcommand or script/binary name and must never be renamed.
fn first_arg_token(args: &str) -> &str {
    args.split_whitespace().next().unwrap_or("")
}

/// Replace file/path tokens in a scenario's `reference_args` (and
/// `task_description`) with alternative names to prevent information leakage
/// from the original skill file examples.
///
/// **Tokens that are never substituted**:
/// - The first token in `reference_args` (subcommand / script / companion binary)
/// - Any token that looks like a script executable (`.pl`, `.py`, `.sh`, …)
///
/// **Collision avoidance**: If two different original tokens would map to the
/// same alternative name, the second one is rotated to the next candidate
/// until a unique name is found.
///
/// The substitution is deterministic (based on the scenario_id) so that the
/// benchmark is fully reproducible.
fn substitute_file_tokens(scenario: &mut Scenario) {
    let file_tokens = extract_file_tokens(&scenario.reference_args);
    if file_tokens.is_empty() {
        return;
    }

    // Identify the leading command token — never substitute it.
    let first = first_arg_token(&scenario.reference_args);

    // Track already-used alternative names (keyed by extension) to avoid
    // two different originals mapping to the same alternative.
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Build the substitution map (original → alternative).
    let mut subs: Vec<(String, String)> = Vec::new();
    for token in &file_tokens {
        // Skip the leading command/subcommand/script token.
        if token == first {
            continue;
        }
        // Skip tokens that look like script executables (part of command
        // syntax, not data files).
        if is_script_name(token) || is_script_path(token) {
            continue;
        }
        let alt = alternative_filename_unique(token, &scenario.scenario_id, &mut used);
        if alt != *token {
            subs.push((token.clone(), alt));
        }
    }

    if subs.is_empty() {
        return;
    }

    // Sort by decreasing length so that longer paths are replaced first
    // (e.g. "data/input.bam" before "input.bam").
    subs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    // Apply substitutions to reference_args.
    let mut new_args = scenario.reference_args.clone();
    for (from, to) in &subs {
        new_args = new_args.replace(from.as_str(), to.as_str());
    }
    scenario.reference_args = new_args;

    // Also apply to task_description (file names may be embedded in the task
    // text, e.g. "sort input.bam by coordinate").
    let mut new_task = scenario.task_description.clone();
    for (from, to) in &subs {
        new_task = new_task.replace(from.as_str(), to.as_str());
    }
    scenario.task_description = new_task;
}

/// Synthetic variant types used to pad scenarios to [`SCENARIOS_PER_TOOL`].
///
/// When a tool has fewer than 10 skill examples, variants are generated by
/// rephrasing the task description **without modifying the reference args**.
/// This avoids injecting flags (like `--verbose` or `-t 4`) that many tools
/// do not support, which would create invalid reference commands.
///
/// Each variant type applies a **structural transformation** to the task
/// description (not just a suffix) to produce genuinely different phrasing
/// that tests the LLM's ability to understand diverse input.
const VARIANT_QUESTION: usize = 0;
const VARIANT_GOAL: usize = 1;
const VARIANT_OUTCOME: usize = 2;
const VARIANT_NEEDED: usize = 3;
const VARIANT_CONTEXTUAL: usize = 4;
const NUM_VARIANT_TYPES: usize = 5;

/// Create a synthetic variant of a scenario.
///
/// Variants only change the **task description** (not the reference args) to
/// provide description-level diversity.  The reference_args are copied verbatim
/// from the source scenario so they remain valid commands.
///
/// Unlike the previous suffix-based approach ("sort bam for a typical analysis"),
/// each variant applies a structural rewrite that produces semantically
/// distinct phrasing, reducing description homogeneity in the benchmark.
fn synthesise_variant(base: &Scenario, idx: usize) -> Scenario {
    let variant_type = idx % NUM_VARIANT_TYPES;
    let task = &base.task_description;
    let lc = lowercase_first_char(task);

    let new_task = match variant_type {
        // "sort bam by coordinate" → "How can I sort bam by coordinate?"
        VARIANT_QUESTION => format!("How can I {}?", lc),
        // "sort bam by coordinate" → "I need to sort bam by coordinate"
        VARIANT_GOAL => format!("I need to {}", lc),
        // "sort bam by coordinate" → "My goal is to sort bam by coordinate"
        VARIANT_OUTCOME => format!("My goal is to {}", lc),
        // "sort bam by coordinate" → "I want the result of sorting bam by coordinate"
        VARIANT_NEEDED => format!(
            "I want the result of {}{}",
            lc.trim_end_matches('.'),
            if lc.ends_with('.') { "" } else { "" }
        ),
        // "sort bam by coordinate" → "In my analysis, I need to sort bam by coordinate"
        VARIANT_CONTEXTUAL => format!("In my analysis, I need to {}", lc),
        _ => task.clone(),
    };

    Scenario {
        tool: base.tool.clone(),
        scenario_id: format!("{}_{:02}", base.tool, idx + 1),
        reference_args: base.reference_args.clone(),
        task_description: new_task,
        category: base.category.clone(),
    }
}

// ── Description generation ───────────────────────────────────────────────────

/// Number of descriptions to generate per scenario.
pub const DESCRIPTIONS_PER_SCENARIO: usize = 10;

/// User-level labels (one per generated description).
pub const USER_LEVELS: [&str; DESCRIPTIONS_PER_SCENARIO] = [
    "original",
    "beginner",
    "student",
    "polite",
    "sysadmin",
    "goal_oriented",
    "expert",
    "detailed",
    "informal",
    "alternative",
];

/// Synonym tables for semantic rewriting of common bioinformatics verbs.
/// Each entry maps a verb to alternative phrasings, enabling genuine
/// semantic diversity rather than mere template substitution.
///
/// Design rules for synonyms:
/// - Keep synonyms **shorter than or equal to** the original verb to avoid
///   sentence corruption when the synonym is inserted in-place.
/// - Avoid synonyms that contain words likely to already appear in the
///   sentence (e.g., "build an index for" is bad because "build" and "for"
///   are likely already present).
/// - Multi-word synonyms are allowed only when they won't create duplication.
const VERB_SYNONYMS: &[(&str, &[&str])] = &[
    ("sort", &["reorder", "organize", "arrange"]),
    ("align", &["map", "match"]),
    ("index", &["create an index for", "build an index for"]),
    ("filter", &["select", "subset", "screen"]),
    ("merge", &["combine", "join", "concatenate"]),
    ("convert", &["transform", "translate"]),
    ("trim", &["clip", "clean"]),
    ("call", &["detect", "identify", "genotype"]),
    ("quantify", &["count", "measure"]),
    ("assemble", &["reconstruct", "build"]),
    ("annotate", &["label", "tag"]),
    ("compare", &["contrast", "diff"]),
    ("extract", &["retrieve", "isolate"]),
    ("generate", &["produce", "create"]),
    ("run", &["execute", "perform"]),
    ("compute", &["calculate", "estimate"]),
    ("remove", &["eliminate", "exclude"]),
    ("view", &["inspect", "examine"]),
    ("download", &["fetch", "retrieve"]),
    ("count", &["tally", "enumerate"]),
];

/// Find a synonym rewrite for the first matchable verb in the task description.
/// Returns `None` if no synonym applies (allowing the caller to fall back to
/// a structural rewrite).
///
/// **Duplication guard**: If a multi-word synonym contains a word that already
/// appears in the task, that synonym is skipped to avoid producing sentences
/// like "build genome build an index for" (from "index" → "build an index for"
/// when "build" is already in the sentence).
fn rewrite_with_synonym(task: &str) -> Option<String> {
    let lower = task.to_lowercase();
    for (verb, synonyms) in VERB_SYNONYMS {
        if let Some(pos) = lower.find(verb) {
            // Verify it's a word boundary (not a substring of another word).
            let before_ok = pos == 0 || !lower.as_bytes()[pos - 1].is_ascii_alphabetic();
            let after_end = pos + verb.len();
            let after_ok =
                after_end >= lower.len() || !lower.as_bytes()[after_end].is_ascii_alphabetic();
            if before_ok && after_ok {
                // Pick a synonym deterministically based on the task hash.
                let h = scenario_hash(task);
                // Try each synonym starting from the hash-selected one,
                // skipping any that would introduce duplicated words.
                for offset in 0..synonyms.len() {
                    let syn = synonyms[(h as usize + offset) % synonyms.len()];
                    // Duplication guard: check if any word in the multi-word
                    // synonym already appears in the task (excluding the verb
                    // being replaced and common stop words).
                    let syn_words: Vec<&str> = syn.split_whitespace().collect();
                    let mut has_dup = false;
                    for w in &syn_words {
                        // Skip short words (a, an, for, to, of, etc.)
                        if w.len() <= 2 {
                            continue;
                        }
                        // Skip the verb being replaced (it's expected to be in the task)
                        if *w == *verb {
                            continue;
                        }
                        // Check if this word already appears in the task
                        if lower.contains(&w.to_lowercase()) {
                            has_dup = true;
                            break;
                        }
                    }
                    if has_dup {
                        continue; // Try next synonym
                    }

                    let mut result = task[..pos].to_string();
                    result.push_str(syn);
                    result.push_str(&task[pos + verb.len()..]);
                    // Preserve original capitalisation of the first character.
                    if task
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        let mut chars = result.chars();
                        if let Some(c) = chars.next() {
                            result = c.to_uppercase().to_string() + chars.as_str();
                        }
                    }
                    return Some(result);
                }
                // All synonyms had duplication — skip this verb entirely.
                return None;
            }
        }
    }
    None
}

/// Generate [`DESCRIPTIONS_PER_SCENARIO`] diverse English descriptions for a
/// single [`Scenario`], simulating users of different experience levels.
///
/// Design principles:
/// - Each variant applies a **structurally different** phrasing strategy,
///   not merely a suffix or prefix to the original text.
/// - The "alternative" variant uses **verb synonym substitution** when possible,
///   producing genuinely different words (e.g., "sort" → "reorder") rather
///   than just reordering the same words.
/// - The "detailed" variant injects **context from bench_scenarios** when
///   available, making the description closer to real user requests that
///   mention specific experimental setups.
pub fn generate_descriptions(scenario: &Scenario) -> Vec<UsageDescription> {
    let task = &scenario.task_description;
    let tool = &scenario.tool;
    let lc = lowercase_first_char(task);

    let variants: Vec<String> = vec![
        // 1  original — skill-file wording
        task.to_string(),
        // 2  beginner — question form
        format!("How do I {}?", lc),
        // 3  student — simple request
        format!("I need to {}", lc),
        // 4  polite — imperative with please
        format!("Please {}", lc),
        // 5  sysadmin — mentions tool
        format!("Use {} to {}", tool, lc),
        // 6  goal-oriented — focus on outcome
        format!("I want to {}", lc),
        // 7  expert — terse (removes filler words)
        make_terse(task),
        // 8  detailed — adds extra context from reference args
        make_detailed(task, &scenario.reference_args),
        // 9  informal — casual wording
        make_casual(task),
        // 10 alternative — synonym-based rewrite or structural rephrase
        make_alternative_enhanced(task),
    ];

    variants
        .into_iter()
        .enumerate()
        .map(|(i, desc)| UsageDescription {
            tool: tool.clone(),
            scenario_id: scenario.scenario_id.clone(),
            desc_id: format!("{}_{:02}", scenario.scenario_id, i + 1),
            user_level: USER_LEVELS[i].to_string(),
            description: desc,
        })
        .collect()
}

/// Lower-case the first character of a string (for embedding in sentences).
fn lowercase_first_char(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
    }
}

/// Expert-level terse description: remove articles and filler words.
fn make_terse(task: &str) -> String {
    let stopwords: std::collections::HashSet<&str> = [
        "a", "an", "the", "for", "with", "and", "in", "to", "from", "using", "into",
    ]
    .into_iter()
    .collect();
    let result: Vec<&str> = task
        .split_whitespace()
        .filter(|w| !stopwords.contains(&w.to_lowercase().as_str()))
        .collect();
    result.join(" ")
}

/// Detailed description: adds explanatory context from the reference args,
/// including experimental context when detectable from file paths.
fn make_detailed(task: &str, reference_args: &str) -> String {
    let has_threads = reference_args.contains("-t ")
        || reference_args.contains("-@ ")
        || reference_args.contains("--threads")
        || reference_args.contains("-p ");
    let has_output = reference_args.contains("-o ") || reference_args.contains("--output");

    // Detect experimental context from file extensions and reference args.
    let exp_context = detect_experiment_context(reference_args);

    let mut desc = format!("I have data that I need to process: {}", task);
    if let Some(ctx) = exp_context {
        desc = format!(
            "I have {} data that I need to process: {}",
            ctx,
            lowercase_first_char(task)
        );
    }
    if has_threads {
        desc.push_str(", utilizing multiple CPU threads for speed");
    }
    if has_output {
        desc.push_str(", saving the result to a specified output file");
    }
    desc
}

/// Detect the experimental context from reference args by looking for
/// characteristic file patterns, subcommands, and flags that indicate
/// the type of bioinformatics analysis being performed.
///
/// This enables generating more realistic task descriptions that
/// mention the experimental setup (e.g., "I have RNA-seq data...")
/// rather than just the abstract operation.
fn detect_experiment_context(reference_args: &str) -> Option<&'static str> {
    let lower = reference_args.to_lowercase();

    // RNA-seq indicators
    if lower.contains("rnaseq")
        || lower.contains("rna-seq")
        || lower.contains("star") && lower.contains("--runmode")
        || lower.contains("featurecounts")
        || lower.contains("htseq")
        || lower.contains("salmon")
        || lower.contains("kallisto")
        || lower.contains("stringtie")
        || lower.contains("cufflinks")
    {
        return Some("RNA-seq");
    }

    // Whole-genome sequencing indicators
    if lower.contains("wgs")
        || lower.contains("whole-genome")
        || lower.contains("gatk") && lower.contains("haplotypecaller")
        || lower.contains("deepvariant")
        || lower.contains("bcftools") && lower.contains("mpileup")
    {
        return Some("whole-genome sequencing");
    }

    // ATAC-seq indicators
    if lower.contains("atac")
        || lower.contains("macs2") && lower.contains("broad")
        || lower.contains("genrich")
        || lower.contains("hmmratac")
    {
        return Some("ATAC-seq");
    }

    // ChIP-seq indicators
    if lower.contains("chip")
        || lower.contains("macs2") && lower.contains("callpeak")
        || lower.contains("sicer")
        || lower.contains("broadpeak")
    {
        return Some("ChIP-seq");
    }

    // Bisulfite sequencing indicators
    if lower.contains("bismark")
        || lower.contains("bsmap")
        || lower.contains("methyldackel")
        || lower.contains("methylation")
    {
        return Some("bisulfite sequencing");
    }

    // Metagenomics indicators
    if lower.contains("kraken")
        || lower.contains("metaphlan")
        || lower.contains("humann")
        || lower.contains("bracken")
    {
        return Some("metagenomics");
    }

    // 16S/amplicon indicators
    if lower.contains("qiime")
        || lower.contains("dada2")
        || lower.contains("16s")
        || lower.contains("amplicon")
    {
        return Some("16S amplicon");
    }

    None
}

/// Casual / informal description.
fn make_casual(task: &str) -> String {
    let lc = lowercase_first_char(task);
    format!("Hey, can you help me {}?", lc)
}

/// Enhanced alternative phrasing: first attempts verb synonym substitution
/// for genuine semantic diversity; falls back to clause reordering or
/// structural rewrite when no synonyms apply.
fn make_alternative_enhanced(task: &str) -> String {
    // Strategy 1: Verb synonym substitution (e.g., "sort" → "reorder").
    if let Some(rewritten) = rewrite_with_synonym(task) {
        return rewritten;
    }

    // Strategy 2: Clause reordering (swap on conjunctions).
    if let Some(pos) = task.find(" and ") {
        let (first, second) = task.split_at(pos);
        let second = &second[5..]; // skip " and "
        return format!("{} after {}", capitalize_first(second.trim()), first.trim());
    }
    if let Some(pos) = task.find(" with ") {
        let (first, second) = task.split_at(pos);
        let second = &second[6..]; // skip " with "
        return format!(
            "With {}, {}",
            second.trim(),
            lowercase_first_char(first.trim())
        );
    }
    if let Some(pos) = task.find(" to ") {
        let (first, second) = task.split_at(pos);
        let second = &second[4..]; // skip " to "
        return format!(
            "Output {} by performing: {}",
            second.trim(),
            lowercase_first_char(first.trim())
        );
    }

    // Strategy 3: Goal-oriented reframing (changes sentence structure).
    let lc = lowercase_first_char(task);
    format!("The desired outcome is: {}", lc)
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

// ── CSV export ───────────────────────────────────────────────────────────────

/// Write reference-command scenarios to CSV.
///
/// Columns: `tool,scenario_id,reference_args,task_description,category`
pub fn write_scenarios_csv<W: Write>(
    writer: &mut W,
    scenarios: &[Scenario],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "tool,scenario_id,reference_args,task_description,category"
    )?;
    for s in scenarios {
        let args_esc = csv_escape(&s.reference_args);
        let task_esc = csv_escape(&s.task_description);
        writeln!(
            writer,
            "{},{},{},{},{}",
            s.tool, s.scenario_id, args_esc, task_esc, s.category
        )?;
    }
    Ok(())
}

/// Write usage descriptions to CSV.
///
/// Columns: `tool,scenario_id,desc_id,user_level,description`
pub fn write_descriptions_csv<W: Write>(
    writer: &mut W,
    descriptions: &[UsageDescription],
) -> std::io::Result<()> {
    writeln!(writer, "tool,scenario_id,desc_id,user_level,description")?;
    for d in descriptions {
        let desc_esc = csv_escape(&d.description);
        writeln!(
            writer,
            "{},{},{},{},{}",
            d.tool, d.scenario_id, d.desc_id, d.user_level, desc_esc
        )?;
    }
    Ok(())
}

/// RFC 4180 CSV field escaping: double any internal quotes and wrap in quotes.
/// Also normalises carriage returns to prevent CSV corruption.
fn csv_escape(field: &str) -> String {
    let normalized = field.replace('\r', " ");
    if normalized.contains(',') || normalized.contains('"') || normalized.contains('\n') {
        format!("\"{}\"", normalized.replace('"', "\"\""))
    } else {
        normalized
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SKILL: &str = r#"---
name: testtool
category: testing
description: A test tool for unit tests
tags: [test, unit]
author: oxo-call built-in
source_url: "https://example.com"
---

## Concepts

- Concept one for testing.
- Concept two for testing.

## Pitfalls

- Pitfall one.

## Examples

### run basic analysis on input.txt
**Args:** `analyze -i input.txt -o output.txt`
**Explanation:** basic analysis with input/output

### run analysis with 4 threads
**Args:** `analyze -i input.txt -o output.txt -t 4`
**Explanation:** multi-threaded analysis

### run verbose analysis
**Args:** `analyze -v -i input.txt`
**Explanation:** verbose mode shows detailed progress

### generate summary report
**Args:** `report --summary input.txt`
**Explanation:** creates a concise summary

### merge two files
**Args:** `merge file1.txt file2.txt -o merged.txt`
**Explanation:** combine two input files
"#;

    #[test]
    fn test_parse_skill_file_metadata() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        assert_eq!(skill.name, "testtool");
        assert_eq!(skill.category, "testing");
        assert_eq!(skill.description, "A test tool for unit tests");
        assert_eq!(skill.tags, vec!["test", "unit"]);
        assert_eq!(skill.source_url, "https://example.com");
    }

    #[test]
    fn test_parse_skill_file_examples() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        assert_eq!(skill.examples.len(), 5);
        assert_eq!(skill.examples[0].task, "run basic analysis on input.txt");
        assert_eq!(skill.examples[0].args, "analyze -i input.txt -o output.txt");
        assert_eq!(
            skill.examples[0].explanation,
            "basic analysis with input/output"
        );
    }

    #[test]
    fn test_generate_scenarios_pads_to_ten() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        assert_eq!(
            scenarios.len(),
            SCENARIOS_PER_TOOL,
            "should always produce exactly {} scenarios",
            SCENARIOS_PER_TOOL
        );
        // First 5 come from the original examples (with substituted file names).
        // Verify the args contain the expected flags (the file names are substituted).
        assert!(scenarios[0].reference_args.contains("analyze -i "));
        assert!(scenarios[0].reference_args.contains(".txt"));
        // All scenario IDs are unique.
        let ids: std::collections::HashSet<&str> =
            scenarios.iter().map(|s| s.scenario_id.as_str()).collect();
        assert_eq!(ids.len(), SCENARIOS_PER_TOOL);
    }

    #[test]
    fn test_generate_scenarios_truncates_to_ten() {
        // Build a skill with 14 examples.
        let mut skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        for i in 6..=14 {
            skill.examples.push(SkillExample {
                task: format!("extra task {i}"),
                args: format!("extra --arg {i}"),
                explanation: format!("explanation {i}"),
            });
        }
        assert_eq!(skill.examples.len(), 14);
        let scenarios = generate_scenarios(&skill);
        assert_eq!(scenarios.len(), SCENARIOS_PER_TOOL);
    }

    #[test]
    fn test_generate_descriptions_count() {
        let scenario = Scenario {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            reference_args: "analyze -i input.txt -o output.txt".to_string(),
            task_description: "run basic analysis on input.txt".to_string(),
            category: "testing".to_string(),
        };
        let descs = generate_descriptions(&scenario);
        assert_eq!(descs.len(), DESCRIPTIONS_PER_SCENARIO);
        // All desc IDs are unique.
        let ids: std::collections::HashSet<&str> =
            descs.iter().map(|d| d.desc_id.as_str()).collect();
        assert_eq!(ids.len(), DESCRIPTIONS_PER_SCENARIO);
    }

    #[test]
    fn test_generate_descriptions_user_levels() {
        let scenario = Scenario {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            reference_args: "analyze -i input.txt".to_string(),
            task_description: "run basic analysis on input.txt".to_string(),
            category: "testing".to_string(),
        };
        let descs = generate_descriptions(&scenario);
        let levels: Vec<&str> = descs.iter().map(|d| d.user_level.as_str()).collect();
        assert_eq!(levels, USER_LEVELS);
    }

    #[test]
    fn test_generate_descriptions_diversity() {
        let scenario = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_01".to_string(),
            reference_args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
            task_description: "sort a BAM file by genomic coordinates".to_string(),
            category: "alignment".to_string(),
        };
        let descs = generate_descriptions(&scenario);
        // All descriptions should be non-empty.
        for d in &descs {
            assert!(!d.description.is_empty(), "description should be non-empty");
        }
        // Descriptions should be diverse (not all identical).
        let unique: std::collections::HashSet<&str> =
            descs.iter().map(|d| d.description.as_str()).collect();
        assert!(
            unique.len() >= 8,
            "at least 8 unique descriptions expected, got {}",
            unique.len()
        );
    }

    #[test]
    fn test_write_scenarios_csv() {
        let scenarios = vec![Scenario {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            reference_args: "analyze -i input.txt".to_string(),
            task_description: "run analysis".to_string(),
            category: "testing".to_string(),
        }];
        let mut buf = Vec::new();
        write_scenarios_csv(&mut buf, &scenarios).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("tool,scenario_id,reference_args,task_description,category"));
        assert!(text.contains("testtool,testtool_01"));
    }

    #[test]
    fn test_write_descriptions_csv() {
        let descs = vec![UsageDescription {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            desc_id: "testtool_01_01".to_string(),
            user_level: "beginner".to_string(),
            description: "How do I run analysis?".to_string(),
        }];
        let mut buf = Vec::new();
        write_descriptions_csv(&mut buf, &descs).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("tool,scenario_id,desc_id,user_level,description"));
        assert!(text.contains("testtool_01_01"));
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("has, comma"), "\"has, comma\"");
        assert_eq!(csv_escape("has \"quote\""), "\"has \"\"quote\"\"\"");
    }

    #[test]
    fn test_make_terse() {
        assert_eq!(
            make_terse("sort a BAM file by genomic coordinates"),
            "sort BAM file by genomic coordinates"
        );
    }

    #[test]
    fn test_make_alternative_enhanced_with_synonym() {
        // "sort" should be rewritten with a synonym like "reorder"/"organize"/"arrange"
        let alt = make_alternative_enhanced("sort a BAM file by coordinate");
        assert!(
            alt.contains("reorder") || alt.contains("organize") || alt.contains("arrange"),
            "Expected synonym replacement, got: {alt}"
        );
    }

    #[test]
    fn test_make_alternative_enhanced_with_and() {
        let alt = make_alternative_enhanced("trim reads and filter low quality");
        // Either synonym or clause reordering
        assert_ne!(alt, "trim reads and filter low quality");
    }

    #[test]
    fn test_make_alternative_enhanced_with_with() {
        let alt = make_alternative_enhanced("align reads with 8 threads");
        // Either synonym ("map reads with 8 threads") or clause reordering
        assert_ne!(alt, "align reads with 8 threads");
    }

    #[test]
    fn test_make_alternative_enhanced_with_to() {
        let alt = make_alternative_enhanced("convert SAM to BAM format");
        // Either synonym ("transform SAM to BAM format") or clause reordering
        assert_ne!(alt, "convert SAM to BAM format");
    }

    #[test]
    fn test_make_alternative_enhanced_fallback() {
        // "index" matches the synonym table → "create an index for"
        let alt = make_alternative_enhanced("index sorted.bam");
        assert!(
            alt.contains("create an index for")
                || alt.contains("desired outcome")
                || alt.contains("Perform:"),
            "Expected synonym or fallback phrasing, got: {alt}"
        );
    }

    #[test]
    fn test_full_pipeline_skill_to_descriptions() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        assert_eq!(scenarios.len(), SCENARIOS_PER_TOOL);

        let mut all_descs = Vec::new();
        for scenario in &scenarios {
            let descs = generate_descriptions(scenario);
            assert_eq!(descs.len(), DESCRIPTIONS_PER_SCENARIO);
            all_descs.extend(descs);
        }
        assert_eq!(
            all_descs.len(),
            SCENARIOS_PER_TOOL * DESCRIPTIONS_PER_SCENARIO,
            "should produce exactly {} descriptions per tool",
            SCENARIOS_PER_TOOL * DESCRIPTIONS_PER_SCENARIO
        );
    }

    #[test]
    fn test_load_skills_from_repo_dir() {
        // This test uses the real skills/ directory from the repository.
        let skills_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("skills");
        if !skills_dir.exists() {
            return; // skip if not in the repo
        }
        let skills = load_skills_from_dir(&skills_dir).unwrap();
        assert!(
            skills.len() >= 100,
            "expected at least 100 skills from repo, got {}",
            skills.len()
        );
        // Every skill should have at least 1 example.
        for skill in &skills {
            assert!(
                !skill.examples.is_empty(),
                "skill '{}' should have examples",
                skill.name
            );
        }
    }

    #[test]
    fn test_synthesise_variant_never_modifies_args() {
        let base = Scenario {
            tool: "bcftools".to_string(),
            scenario_id: "bcftools_01".to_string(),
            reference_args: "mpileup -f ref.fa input.bam | bcftools call -mv -o out.vcf"
                .to_string(),
            task_description: "call variants".to_string(),
            category: "variant-calling".to_string(),
        };
        // All variant types must preserve reference_args exactly.
        for idx in 0..NUM_VARIANT_TYPES {
            let v = synthesise_variant(&base, idx);
            assert_eq!(
                v.reference_args, base.reference_args,
                "variant idx={idx} must not modify reference_args"
            );
        }
    }

    #[test]
    fn test_synthesise_variant_rephrases_task() {
        let base = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_01".to_string(),
            reference_args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
            task_description: "sort BAM".to_string(),
            category: "alignment".to_string(),
        };
        // Variant 0 should be a question form ("How can I sort BAM?").
        let v = synthesise_variant(&base, 0);
        assert!(
            v.task_description.contains("How can I") || v.task_description != base.task_description,
            "variant 0 should rephrase the task: {}",
            v.task_description
        );
        // Args must be unchanged.
        assert_eq!(v.reference_args, base.reference_args);
    }

    #[test]
    fn test_is_excluded_tool() {
        // Package managers
        assert!(is_excluded_tool("conda"));
        assert!(is_excluded_tool("docker"));
        assert!(is_excluded_tool("pip"));
        // HPC schedulers
        assert!(is_excluded_tool("slurm"));
        assert!(is_excluded_tool("kubectl"));
        // AI assistants
        assert!(is_excluded_tool("claude"));
        assert!(is_excluded_tool("openclaw"));
        // Non-excluded tools
        assert!(!is_excluded_tool("samtools"));
        assert!(!is_excluded_tool("bwa"));
        assert!(!is_excluded_tool("bash"));
        assert!(!is_excluded_tool("fastp"));
    }

    #[test]
    fn test_load_skills_for_bench_excludes_tools() {
        let skills_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("skills");
        if !skills_dir.exists() {
            return;
        }
        let all_skills = load_skills_from_dir(&skills_dir).unwrap();
        let bench_skills = load_skills_for_bench(&skills_dir).unwrap();
        // Should have fewer skills after exclusion.
        assert!(
            bench_skills.len() < all_skills.len(),
            "bench skills ({}) should be fewer than all skills ({})",
            bench_skills.len(),
            all_skills.len()
        );
        // No excluded tools should appear.
        for skill in &bench_skills {
            assert!(
                !is_excluded_tool(&skill.name),
                "excluded tool '{}' should not be in bench skills",
                skill.name
            );
        }
    }

    // ── File extraction & task enrichment tests ──────────────────────────────

    #[test]
    fn test_extract_file_tokens_simple() {
        let files = extract_file_tokens("sort -@ 4 -o sorted.bam input.bam");
        assert_eq!(files, vec!["sorted.bam", "input.bam"]);
    }

    #[test]
    fn test_extract_file_tokens_key_value() {
        let files =
            extract_file_tokens("bbduk.sh in=R1.fastq.gz out=R1_trimmed.fastq.gz ref=adapters.fa");
        assert!(files.contains(&"R1.fastq.gz".to_string()));
        assert!(files.contains(&"R1_trimmed.fastq.gz".to_string()));
        assert!(files.contains(&"adapters.fa".to_string()));
        assert!(files.contains(&"bbduk.sh".to_string()));
    }

    #[test]
    fn test_extract_file_tokens_skips_flags_and_numbers() {
        let files = extract_file_tokens("--threads 8 --seed=42 -j8 1e-6");
        assert!(files.is_empty(), "should not extract flags or numbers");
    }

    #[test]
    fn test_extract_file_tokens_piped_command() {
        let files = extract_file_tokens(
            "mpileup -f reference.fa -O u input.bam | bcftools call -m -v -O z -o variants.vcf.gz",
        );
        assert!(files.contains(&"reference.fa".to_string()));
        assert!(files.contains(&"input.bam".to_string()));
        assert!(files.contains(&"variants.vcf.gz".to_string()));
    }

    #[test]
    fn test_extract_file_tokens_redirect() {
        let files = extract_file_tokens("stats input.bam > stats.txt");
        assert!(files.contains(&"input.bam".to_string()));
        assert!(files.contains(&"stats.txt".to_string()));
    }

    #[test]
    fn test_extract_file_tokens_skips_urls() {
        let files = extract_file_tokens("-I https://example.com");
        assert!(files.is_empty(), "URLs should not be extracted as files");
    }

    #[test]
    fn test_extract_file_tokens_colon_separated() {
        let files = extract_file_tokens("ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 LEADING:3 TRAILING:3");
        assert!(
            files.contains(&"TruSeq3-PE.fa".to_string()),
            "should extract file from ILLUMINACLIP:file:params"
        );
    }

    #[test]
    fn test_extract_file_tokens_colon_file_number() {
        let files = extract_file_tokens("chromsizes.txt:5000");
        assert!(
            files.contains(&"chromsizes.txt".to_string()),
            "should extract file from file:number"
        );
    }

    #[test]
    fn test_extract_file_tokens_quoted_shell_command() {
        let files = extract_file_tokens("-c 'diff <(sort file1.txt) <(sort file2.txt)'");
        // file1.txt and file2.txt are inside subshell, but within the
        // single-quoted string they should be discovered.
        assert!(
            files.contains(&"file1.txt".to_string()),
            "should find files inside quoted shell command"
        );
        assert!(
            files.contains(&"file2.txt".to_string()),
            "should find files inside quoted shell command"
        );
    }

    #[test]
    fn test_extract_file_tokens_no_duplicates() {
        let files = extract_file_tokens("view -b input.bam -o input.bam");
        assert_eq!(files.len(), 1, "duplicate files should be deduplicated");
    }

    #[test]
    fn test_is_file_like_token_common_extensions() {
        assert!(is_file_like_token("input.bam"));
        assert!(is_file_like_token("reads.fastq.gz"));
        assert!(is_file_like_token("reference.fa"));
        assert!(is_file_like_token("output.vcf"));
        assert!(is_file_like_token("script.py"));
        assert!(is_file_like_token("annotation.gff3"));
    }

    #[test]
    fn test_is_file_like_token_rejects_non_files() {
        assert!(!is_file_like_token("--output"));
        assert!(!is_file_like_token("-o"));
        assert!(!is_file_like_token("42"));
        assert!(!is_file_like_token(""));
        assert!(!is_file_like_token("https://example.com"));
    }

    #[test]
    fn test_enrich_task_no_files() {
        let result = enrich_task_with_files("run analysis", "--threads 8 -j4");
        assert_eq!(result, "run analysis", "no files → no change");
    }

    #[test]
    fn test_enrich_task_with_missing_files() {
        let result = enrich_task_with_files(
            "sort a BAM file by genomic coordinates",
            "sort -@ 4 -o sorted.bam input.bam",
        );
        assert!(
            result.contains("sorted.bam"),
            "enriched task should mention sorted.bam"
        );
        assert!(
            result.contains("input.bam"),
            "enriched task should mention input.bam"
        );
    }

    #[test]
    fn test_enrich_task_skips_already_mentioned() {
        let result = enrich_task_with_files(
            "run basic analysis on input.txt",
            "analyze -i input.txt -o output.txt",
        );
        assert!(
            result.contains("output.txt"),
            "should add missing output.txt"
        );
        // input.txt is already in the task, so no parenthetical needed for it
        // but output.txt triggers the parenthetical
        assert!(result.starts_with("run basic analysis on input.txt ("));
    }

    #[test]
    fn test_enrich_task_all_files_present() {
        let result = enrich_task_with_files(
            "sort input.bam to sorted.bam",
            "sort -o sorted.bam input.bam",
        );
        assert_eq!(
            result, "sort input.bam to sorted.bam",
            "all files already present → no change"
        );
    }

    #[test]
    fn test_generate_scenarios_enriches_tasks() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        // First scenario: original task was "run basic analysis on input.txt",
        // args were "analyze -i input.txt -o output.txt".
        // After substitution, file names change but extensions stay .txt.
        // The enrichment step adds any file tokens from args that are missing
        // from the task description.
        let task = &scenarios[0].task_description;
        let args = &scenarios[0].reference_args;
        // All file tokens from args should appear in the enriched task.
        let file_tokens = extract_file_tokens(args);
        for ft in &file_tokens {
            assert!(
                task.contains(ft.as_str()),
                "task should contain substituted file '{}', got: {}",
                ft,
                task
            );
        }
    }

    #[test]
    fn test_generate_scenarios_enriched_tasks_flow_to_descriptions() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        let descs = generate_descriptions(&scenarios[0]);
        // Extract file tokens from the scenario's reference_args.
        let file_tokens = extract_file_tokens(&scenarios[0].reference_args);
        assert!(!file_tokens.is_empty(), "should have file tokens in args");
        // All description variants that include the full task should
        // contain at least one of the file tokens from the enriched task.
        let has_any_file = descs.iter().all(|d| {
            file_tokens
                .iter()
                .any(|ft| d.description.contains(ft.as_str()))
                || d.user_level == "expert" // expert level strips filler words
        });
        assert!(
            has_any_file,
            "descriptions should propagate file references from enriched tasks"
        );
    }

    // ── Package / identifier extraction tests ────────────────────────────────

    #[test]
    fn test_extract_package_identifiers_r_install_packages() {
        let pkgs = extract_package_identifiers(
            "Rscript -e \"install.packages('ggplot2', repos='https://cloud.r-project.org')\"",
        );
        assert!(
            pkgs.contains(&"ggplot2".to_string()),
            "should extract ggplot2 from install.packages(): got {:?}",
            pkgs
        );
    }

    #[test]
    fn test_extract_package_identifiers_r_biocmanager() {
        let pkgs =
            extract_package_identifiers("Rscript -e \"BiocManager::install(c('DESeq2','edgeR'))\"");
        assert!(
            pkgs.contains(&"DESeq2".to_string()),
            "should extract DESeq2: got {:?}",
            pkgs
        );
        assert!(
            pkgs.contains(&"edgeR".to_string()),
            "should extract edgeR: got {:?}",
            pkgs
        );
    }

    #[test]
    fn test_extract_package_identifiers_r_package_version() {
        let pkgs = extract_package_identifiers("Rscript -e \"packageVersion('DESeq2')\"");
        assert!(
            pkgs.contains(&"DESeq2".to_string()),
            "should extract DESeq2 from packageVersion(): got {:?}",
            pkgs
        );
    }

    #[test]
    fn test_extract_package_identifiers_python_import() {
        let pkgs = extract_package_identifiers(
            "-c \"import json,sys; data=json.load(sys.stdin); print(data)\"",
        );
        assert!(
            pkgs.contains(&"json".to_string()),
            "should extract json from import: got {:?}",
            pkgs
        );
    }

    #[test]
    fn test_extract_package_identifiers_skips_urls() {
        let pkgs = extract_package_identifiers(
            "Rscript -e \"install.packages('ggplot2', repos='https://cloud.r-project.org')\"",
        );
        // Should NOT contain the URL
        for pkg in &pkgs {
            assert!(
                !pkg.contains("://"),
                "should not extract URLs as packages: got {:?}",
                pkgs
            );
        }
    }

    #[test]
    fn test_extract_package_identifiers_skips_files() {
        let pkgs = extract_package_identifiers(
            "Rscript -e \"rmarkdown::render('report.Rmd', output_format='html_document')\"",
        );
        // report.Rmd is a file, not a package — it should be excluded from
        // package identifiers (handled by extract_file_tokens instead).
        assert!(
            !pkgs.contains(&"report.Rmd".to_string()),
            "should not extract files as packages: got {:?}",
            pkgs
        );
    }

    #[test]
    fn test_extract_package_identifiers_github_repo() {
        let pkgs = extract_package_identifiers("Rscript -e \"pak::pkg_install('user/repo')\"");
        assert!(
            pkgs.contains(&"user/repo".to_string()),
            "should extract GitHub repo reference: got {:?}",
            pkgs
        );
    }

    #[test]
    fn test_extract_file_tokens_rmd_in_r_expression() {
        let files = extract_file_tokens(
            "Rscript -e \"rmarkdown::render('report.Rmd', output_format='html_document')\"",
        );
        assert!(
            files.contains(&"report.Rmd".to_string()),
            "should extract report.Rmd from R expression: got {:?}",
            files
        );
    }

    #[test]
    fn test_enrich_task_with_r_packages() {
        let result = enrich_task_with_files(
            "check installed package version",
            "Rscript -e \"packageVersion('DESeq2')\"",
        );
        assert!(
            result.contains("DESeq2"),
            "enriched task should mention DESeq2, got: {}",
            result
        );
    }

    #[test]
    fn test_enrich_task_with_bioconductor_packages() {
        let result = enrich_task_with_files(
            "install Bioconductor packages",
            "Rscript -e \"BiocManager::install(c('DESeq2','edgeR'))\"",
        );
        assert!(
            result.contains("DESeq2"),
            "enriched task should mention DESeq2, got: {}",
            result
        );
        assert!(
            result.contains("edgeR"),
            "enriched task should mention edgeR, got: {}",
            result
        );
    }

    #[test]
    fn test_enrich_task_with_rmd_file() {
        let result = enrich_task_with_files(
            "render an Rmarkdown document to HTML",
            "Rscript -e \"rmarkdown::render('report.Rmd', output_format='html_document')\"",
        );
        assert!(
            result.contains("report.Rmd"),
            "enriched task should mention report.Rmd, got: {}",
            result
        );
    }

    // ── File token substitution tests ────────────────────────────────────────

    #[test]
    fn test_split_base_ext_simple() {
        let (base, ext) = split_base_ext("input.bam");
        assert_eq!(base, "input");
        assert_eq!(ext, ".bam");
    }

    #[test]
    fn test_split_base_ext_compound() {
        let (base, ext) = split_base_ext("reads.fastq.gz");
        assert_eq!(base, "reads");
        assert_eq!(ext, ".fastq.gz");
    }

    #[test]
    fn test_split_base_ext_no_ext() {
        let (base, ext) = split_base_ext("noext");
        assert_eq!(base, "noext");
        assert_eq!(ext, "");
    }

    #[test]
    fn test_alternative_filename_preserves_extension() {
        let alt = alternative_filename("input.bam", "samtools_01");
        assert!(
            alt.ends_with(".bam"),
            "should preserve .bam extension: {alt}"
        );
        assert_ne!(alt, "input.bam", "should differ from original");
    }

    #[test]
    fn test_alternative_filename_compound_extension() {
        let alt = alternative_filename("reads.fastq.gz", "bwa_01");
        assert!(
            alt.ends_with(".fastq.gz"),
            "should preserve .fastq.gz: {alt}"
        );
        assert_ne!(alt, "reads.fastq.gz");
    }

    #[test]
    fn test_alternative_filename_preserves_directory() {
        let alt = alternative_filename("data/input.bam", "samtools_01");
        assert!(alt.starts_with("data/"), "should preserve directory: {alt}");
        assert!(alt.ends_with(".bam"));
    }

    #[test]
    fn test_alternative_filename_deterministic() {
        let a1 = alternative_filename("input.bam", "samtools_01");
        let a2 = alternative_filename("input.bam", "samtools_01");
        assert_eq!(a1, a2, "should be deterministic");
    }

    #[test]
    fn test_substitute_file_tokens_changes_filenames() {
        let mut scenario = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_01".to_string(),
            reference_args: "sort -o sorted.bam input.bam".to_string(),
            task_description: "sort input.bam by genomic coordinates".to_string(),
            category: "alignment".to_string(),
        };
        let original_args = scenario.reference_args.clone();
        substitute_file_tokens(&mut scenario);
        // File names should be different from originals.
        assert_ne!(
            scenario.reference_args, original_args,
            "file tokens should be substituted"
        );
        // Extensions should be preserved.
        assert!(
            scenario.reference_args.contains(".bam"),
            "should still contain .bam extension"
        );
        // Flags should be preserved.
        assert!(
            scenario.reference_args.contains("sort -o "),
            "flags should be preserved"
        );
    }

    #[test]
    fn test_substitute_file_tokens_updates_task_description() {
        let mut scenario = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_02".to_string(),
            reference_args: "sort -o sorted.bam input.bam".to_string(),
            task_description: "sort input.bam by coordinate".to_string(),
            category: "alignment".to_string(),
        };
        substitute_file_tokens(&mut scenario);
        // Task description should not mention the original file names.
        assert!(
            !scenario.task_description.contains("input.bam"),
            "task should not contain original filename: {}",
            scenario.task_description
        );
    }

    #[test]
    fn test_substitute_preserves_args_without_files() {
        let mut scenario = Scenario {
            tool: "date".to_string(),
            scenario_id: "date_01".to_string(),
            reference_args: "+%Y-%m-%d".to_string(),
            task_description: "show current date".to_string(),
            category: "utility".to_string(),
        };
        let original_args = scenario.reference_args.clone();
        substitute_file_tokens(&mut scenario);
        assert_eq!(
            scenario.reference_args, original_args,
            "args without file tokens should be unchanged"
        );
    }

    #[test]
    fn test_generate_scenarios_substitutes_files() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        // The original skill has "input.txt" in examples — after substitution,
        // scenarios should NOT contain "input.txt".
        for s in &scenarios {
            assert!(
                !s.reference_args.contains("input.txt"),
                "scenario {} should not contain original filename 'input.txt': {}",
                s.scenario_id,
                s.reference_args
            );
        }
    }

    #[test]
    fn test_substitute_preserves_script_names() {
        // AGAT-style scenario where the first token is a script executable.
        let mut scenario = Scenario {
            tool: "agat".to_string(),
            scenario_id: "agat_01".to_string(),
            reference_args: "agat_convert_sp_gff2gtf.pl --gff annotation.gff3 -o annotation.gtf"
                .to_string(),
            task_description: "convert GFF3 to GTF format with agat_convert_sp_gff2gtf.pl"
                .to_string(),
            category: "annotation".to_string(),
        };
        substitute_file_tokens(&mut scenario);
        // Script name must be preserved.
        assert!(
            scenario
                .reference_args
                .contains("agat_convert_sp_gff2gtf.pl"),
            "script name should not be substituted: {}",
            scenario.reference_args
        );
        // But data files should be substituted.
        assert!(
            !scenario.reference_args.contains("annotation.gff3"),
            "data file should be substituted: {}",
            scenario.reference_args
        );
    }

    #[test]
    fn test_is_script_name() {
        assert!(is_script_name("agat_convert_sp_gff2gtf.pl"));
        assert!(is_script_name("bbduk.sh"));
        assert!(is_script_name("infer_experiment.py"));
        assert!(is_script_name("configureStrelkaGermlineWorkflow.py"));
        // Not script names:
        assert!(!is_script_name("input.bam"));
        assert!(!is_script_name("reads.fastq.gz"));
        assert!(!is_script_name("/path/to/script.py"));
        assert!(!is_script_name("-f"));
    }

    #[test]
    fn test_is_script_path() {
        assert!(is_script_path("strelka_germline/runWorkflow.py"));
        assert!(is_script_path("strelka_somatic/runWorkflow.py"));
        assert!(is_script_path("scripts/run_analysis.sh"));
        // Not script paths:
        assert!(!is_script_path("runWorkflow.py")); // no path separator
        assert!(!is_script_path("data/reads.fastq.gz")); // not a script ext
        assert!(!is_script_path("sorted.bam"));
    }

    #[test]
    fn test_synthesise_variant_never_adds_invalid_flags() {
        // Ensure no variant ever adds --verbose, -t, -o, or --quiet to args.
        let base = Scenario {
            tool: "fastqc".to_string(),
            scenario_id: "fastqc_01".to_string(),
            reference_args: "sample.fastq.gz -o qc_results/".to_string(),
            task_description: "run quality control".to_string(),
            category: "qc".to_string(),
        };
        for idx in 0..20 {
            let v = synthesise_variant(&base, idx);
            assert_eq!(
                v.reference_args, base.reference_args,
                "variant idx={idx} must not modify reference_args"
            );
        }
    }

    // ── Shell metacharacter stripping tests ──────────────────────────────────

    #[test]
    fn test_strip_shell_metacharacters_no_change() {
        assert_eq!(
            strip_shell_metacharacters("sort -@ 4 -o sorted.bam input.bam"),
            "sort -@ 4 -o sorted.bam input.bam"
        );
    }

    #[test]
    fn test_strip_shell_metacharacters_pipe() {
        assert_eq!(
            strip_shell_metacharacters("mpileup -f ref.fa input.bam | bcftools call -m"),
            "mpileup -f ref.fa input.bam"
        );
    }

    #[test]
    fn test_strip_shell_metacharacters_redirect() {
        assert_eq!(
            strip_shell_metacharacters("sort -@ 4 input.bam > sorted.bam"),
            "sort -@ 4 input.bam"
        );
    }

    #[test]
    fn test_strip_shell_metacharacters_pipe_and_redirect() {
        assert_eq!(
            strip_shell_metacharacters("view -bS input.sam | sort -o sorted.bam > /dev/null"),
            "view -bS input.sam"
        );
    }

    #[test]
    fn test_strip_shell_metacharacters_preserves_quoted_pipe() {
        // Pipes inside quotes should not be treated as shell operators.
        assert_eq!(
            strip_shell_metacharacters("awk -F'|' '{print $1}' input.txt"),
            "awk -F'|' '{print $1}' input.txt"
        );
    }

    #[test]
    fn test_strip_shell_metacharacters_stderr_redirect() {
        assert_eq!(
            strip_shell_metacharacters("run -i input.bam 2> error.log"),
            "run -i input.bam"
        );
    }

    #[test]
    fn test_strip_shell_metacharacters_append_redirect() {
        assert_eq!(
            strip_shell_metacharacters("run -i input.bam >> output.log"),
            "run -i input.bam"
        );
    }

    #[test]
    fn test_csv_escape_carriage_return() {
        let result = csv_escape("hello\rworld");
        assert_eq!(result, "hello world");
    }
}
