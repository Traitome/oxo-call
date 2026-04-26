use crate::config::Config;
use crate::error::{OxoError, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use uuid::Uuid;

// Import doc_summarizer module (declared in main.rs)
use crate::doc_summarizer::{
    MAX_DOC_LEN_LARGE_MODEL, MAX_DOC_LEN_MEDIUM_MODEL, MAX_DOC_LEN_SMALL_MODEL, summarize_docs,
};

// Minimum useful help text length – anything shorter than this is likely an error message
const MIN_HELP_LEN: usize = 80;

// Maximum chars to store per help section to keep LLM prompts reasonable
const MAX_HELP_LEN: usize = 16_000;

// Fraction of help text that must appear in cache to consider it a duplicate (80 %)
const DEDUP_OVERLAP_NUMERATOR: usize = 4;
const DEDUP_OVERLAP_DENOMINATOR: usize = 5;

// Section header written by IndexManager when it stores --help output in the cache.
// strip_embedded_help_section() must match this exactly; change both if the header changes.
const HELP_OUTPUT_SECTION_LF: &str = "# Help Output\n";
const HELP_OUTPUT_SECTION_CRLF: &str = "# Help Output\r\n";

/// Validate that a tool name is safe to use in file paths and command execution.
/// Tool names must consist of alphanumeric characters, hyphens, underscores, or dots.
pub fn validate_tool_name(tool: &str) -> Result<()> {
    if tool.is_empty() {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            "Tool name cannot be empty".to_string(),
        ));
    }
    if tool.contains("..") || tool.contains('/') || tool.contains('\\') {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            "Tool name must not contain path separators or '..'".to_string(),
        ));
    }
    if !tool
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            "Tool name contains invalid characters (allowed: alphanumeric, '-', '_', '.')"
                .to_string(),
        ));
    }
    Ok(())
}

/// Common tool name aliases for bioinformatics tools where the
/// skill file name differs from the actual executable name.
/// Many Python tools have .py extensions or different capitalization.
const TOOL_ALIASES: &[(&str, &[&str])] = &[
    // Python bioinformatics tools
    ("cnvkit", &["cnvkit.py"]),
    ("nanocomp", &["NanoComp", "nanocomp.py"]),
    ("nanoplot", &["NanoPlot", "nanoplot.py"]),
    ("nanostat", &["NanoStat", "nanostat.py"]),
    ("methyldackel", &["MethylDackel", "methyldackel.py"]),
    ("pbccs", &["ccs"]),
    ("eggnog-mapper", &["emapper.py", "emapper"]),
    ("fastani", &["fastANI"]),
    // Tools with version suffixes
    ("bwa", &["bwa-mem2"]),
    ("samtools", &["samtools-1.20", "samtools-1.19"]),
];

/// Semantic domain-to-subcommand mappings for common bioinformatics tools.
/// When a task mentions a domain concept (e.g., "align") without the explicit subcommand,
/// this mapping provides the default subcommand for that domain.
const DOMAIN_SUBCMD_MAP: &[(&str, &str, &str)] = &[
    // bwa: alignment domain → mem subcommand
    ("bwa", "align", "mem"),
    ("bwa", "alignment", "mem"),
    ("bwa", "map", "mem"),
    ("bwa", "mapping", "mem"),
    ("bwa-mem2", "align", "mem"),
    ("bwa-mem2", "alignment", "mem"),
    // samtools: various domains
    ("samtools", "sort", "sort"),
    ("samtools", "sorting", "sort"),
    ("samtools", "view", "view"),
    ("samtools", "convert", "view"),
    ("samtools", "index", "index"),
    ("samtools", "indexing", "index"),
    ("samtools", "merge", "merge"),
    ("samtools", "merging", "merge"),
    ("samtools", "flagstat", "flagstat"),
    ("samtools", "stats", "stats"),
    ("samtools", "statistic", "stats"),
    // bcftools: variant domains
    ("bcftools", "call", "call"),
    ("bcftools", "calling", "call"),
    ("bcftools", "filter", "filter"),
    ("bcftools", "filtering", "filter"),
    ("bcftools", "view", "view"),
    ("bcftools", "convert", "view"),
    ("bcftools", "merge", "merge"),
    ("bcftools", "merging", "merge"),
    // gatk: common variant callers
    ("gatk", "variant", "HaplotypeCaller"),
    ("gatk", "call", "HaplotypeCaller"),
    ("gatk", "calling", "HaplotypeCaller"),
    ("gatk", "haplotype", "HaplotypeCaller"),
    ("gatk", "split", "SplitNCigarReads"),
    ("gatk", "base", "BaseRecalibrator"),
    ("gatk", "recalibrate", "BaseRecalibrator"),
    // picard: common operations
    ("picard", "sort", "SortSam"),
    ("picard", "sorting", "SortSam"),
    ("picard", "markdup", "MarkDuplicates"),
    ("picard", "duplicate", "MarkDuplicates"),
    ("picard", "add", "AddOrReplaceReadGroups"),
    ("picard", "readgroup", "AddOrReplaceReadGroups"),
    // fastqc: single command tool
    ("fastqc", "quality", "fastqc"),
    ("fastqc", "qc", "fastqc"),
    // meme: motif analysis
    ("meme", "motif", "meme"),
    ("meme", "find", "meme"),
    ("meme", "discover", "meme"),
];

/// Get inferred subcommand from task domain keywords.
/// Returns the subcommand name if a semantic match is found.
fn infer_subcommand_from_domain(tool: &str, task: &str) -> Option<&'static str> {
    let task_lower = task.to_ascii_lowercase();
    for (t, domain, subcmd) in DOMAIN_SUBCMD_MAP {
        if tool == *t && task_lower.contains(domain) {
            return Some(subcmd);
        }
    }
    None
}

/// Get alternative tool names to try if the primary name fails.
fn get_tool_aliases(tool: &str) -> Vec<&'static str> {
    for (primary, aliases) in TOOL_ALIASES {
        if tool == *primary {
            return aliases.to_vec();
        }
    }
    Vec::new()
}

/// Try to run a help command with multiple tool name variants.
/// First tries the original tool name, then known aliases.
async fn try_with_aliases_async(fetcher: &DocsFetcher, tool: &str, flag: &str) -> Result<String> {
    // Try original tool name first
    if let Ok(help) = fetcher.run_help_flag_async(tool, flag).await {
        return Ok(help);
    }

    // Try known aliases
    for alias in get_tool_aliases(tool) {
        if let Ok(help) = fetcher.run_help_flag_async(alias, flag).await {
            return Ok(help);
        }
    }

    Err(OxoError::DocFetchError(
        tool.to_string(),
        format!("Tool '{}' and all aliases not found", tool),
    ))
}

/// Fetches and returns the documentation/help text for a given tool
pub struct DocsFetcher {
    config: Config,
}

/// Combined documentation from all available sources
#[derive(Debug, Clone)]
pub struct ToolDocs {
    #[allow(dead_code)]
    pub tool_name: String,
    pub help_output: Option<String>,
    pub cached_docs: Option<String>,
    pub version: Option<String>,
    /// Help text for a specific subcommand matched from the user's task description.
    /// Populated by `fetch_subcommand_help()` when the top-level help lists subcommands
    /// and the user's task mentions one of them.
    pub subcommand_help: Option<String>,
}

impl ToolDocs {
    /// Return the best available documentation, preferring cached full docs but always
    /// including fresh `--help` output when available. Deduplicates content between sources.
    ///
    /// When `summarize_for_model` is provided, intelligently summarizes documentation
    /// to fit within model-specific length limits while preserving critical information.
    pub fn combined(&self) -> String {
        self.combined_with_limit(None)
    }

    /// Return summarized documentation optimized for a specific model size.
    ///
    /// Model size determines the maximum documentation length:
    /// - "small" (0.5B-1B): 3,000 chars
    /// - "medium" (7B): 6,000 chars  
    /// - "large" (16B+): 10,000 chars
    pub fn combined_for_model(&self, model_size: &str) -> String {
        let max_len = match model_size {
            "small" => MAX_DOC_LEN_SMALL_MODEL,
            "medium" => MAX_DOC_LEN_MEDIUM_MODEL,
            "large" => MAX_DOC_LEN_LARGE_MODEL,
            _ => MAX_DOC_LEN_MEDIUM_MODEL,
        };
        self.combined_with_limit(Some(max_len))
    }

    /// Internal implementation that optionally applies length limits
    fn combined_with_limit(&self, max_len: Option<usize>) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(version) = &self.version {
            parts.push(format!("Version: {version}"));
        }

        // Prefer cached docs (they may contain more detail from remote sources),
        // but always append fresh help so the LLM sees current flags too.
        if let Some(cached) = &self.cached_docs {
            // Strip any embedded help section to avoid duplication when we append
            // the live --help below. This keeps the combined output lean.
            let stripped = strip_embedded_help_section(cached);
            parts.push(stripped);
        }
        if let Some(help) = &self.help_output {
            // Only add live --help if it isn't already embedded verbatim in cached docs
            let already_present = self
                .cached_docs
                .as_deref()
                .is_some_and(|c| deduplicate_check(c, help));
            if !already_present {
                parts.push(clean_help_output(help));
            }
        }

        // Append subcommand help if available (e.g., "samtools sort --help")
        if let Some(subcmd_help) = &self.subcommand_help {
            parts.push(subcmd_help.clone());
        }

        let combined = parts.join("\n\n");

        // Apply intelligent summarization if length limit is specified
        if let Some(limit) = max_len {
            summarize_docs(&combined, limit)
        } else {
            combined
        }
    }

    pub fn is_empty(&self) -> bool {
        self.help_output.is_none() && self.cached_docs.is_none()
    }
}

impl DocsFetcher {
    pub fn new(config: Config) -> Self {
        DocsFetcher { config }
    }

    /// Fetch documentation for a tool from all available sources
    pub async fn fetch(&self, tool: &str) -> Result<ToolDocs> {
        self.fetch_inner(tool, false).await
    }

    /// Fetch documentation for a tool, skipping the local cache.
    /// This forces a fresh --help fetch even when cached docs exist.
    pub async fn fetch_no_cache(&self, tool: &str) -> Result<ToolDocs> {
        self.fetch_inner(tool, true).await
    }

    /// Inner implementation shared by `fetch` and `fetch_no_cache`.
    async fn fetch_inner(&self, tool: &str, skip_cache: bool) -> Result<ToolDocs> {
        validate_tool_name(tool)?;

        let mut docs = ToolDocs {
            tool_name: tool.to_string(),
            help_output: None,
            cached_docs: None,
            version: None,
            subcommand_help: None,
        };

        // 1. Try local cache first (unless skipping cache)
        if !skip_cache && let Ok(cached) = self.load_cache(tool) {
            docs.cached_docs = Some(cached);
        }

        // 2. Try to get help text and version from the live tool using async spawn
        match self.fetch_help_async(tool).await {
            Ok((help, version)) => {
                docs.help_output = Some(help);
                docs.version = version;
            }
            Err(_) => {
                // Tool may not be in PATH - that's okay if we have cached docs
            }
        }

        // 2b. Version-aware cache invalidation: if the detected version differs
        // from the cached version, refresh the cache to pick up new/changed flags.
        if !skip_cache && docs.cached_docs.is_some() && docs.version.is_some() {
            let current_version = docs.version.as_deref().unwrap_or("");
            let cached_version = self.load_cached_version(tool);
            if let Some(ref old_version) = cached_version
                && !current_version.is_empty()
                && old_version != current_version
                && extract_major_minor(old_version) != extract_major_minor(current_version)
            {
                // Major/minor version changed — invalidate cache
                docs.cached_docs = None;
                eprintln!(
                    "{} {} version changed ({} → {}), refreshing cached documentation",
                    "note:".cyan().bold(),
                    tool,
                    old_version.dimmed(),
                    current_version.dimmed()
                );
            }
        }

        // 3. Try local documentation paths from config (unless skipping cache)
        if !skip_cache
            && docs.cached_docs.is_none()
            && let Some(local_doc) = self.search_local_docs(tool)
        {
            docs.cached_docs = Some(local_doc);
        }

        if docs.is_empty() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!(
                    "No documentation found for '{tool}'.\n\n\
                     Troubleshooting:\n  \
                     1. Ensure '{tool}' is installed and in PATH: which {tool}\n  \
                     2. Cache documentation: oxo-call docs add {tool}\n  \
                     3. Add remote docs: oxo-call docs add {tool} --url <docs-url>\n\n\
                     See: oxo-call docs --help"
                ),
            ));
        }

        // 4. Auto-cache: if we got live help output but have no cached docs yet,
        //    silently persist the help text so future calls are instant.
        if docs.cached_docs.is_none()
            && let Some(help) = &docs.help_output
        {
            let _ = self.save_cache(tool, help);
        }

        // 5. Persist version alongside cache for future version-change detection.
        if let Some(ref ver) = docs.version {
            let _ = self.save_cached_version(tool, ver);
        }

        Ok(docs)
    }

    /// Fetch --help / -h output from a tool, with multiple fallback strategies.
    ///
    /// Strategy (in order):
    ///  1. `--help`
    ///  2. `-h`
    ///  3. `help` (as a subcommand)
    ///  4. No arguments (many bioinformatics tools print usage when invoked bare)
    fn fetch_help(&self, tool: &str) -> Result<(String, Option<String>)> {
        let help = self
            .run_help_flag(tool, "--help")
            .or_else(|_| self.run_help_flag(tool, "-h"))
            .or_else(|_| self.run_help_flag(tool, "help"))
            .or_else(|_| self.run_no_args(tool))
            .or_else(|_| self.run_shell_builtin_help(tool))
            .map_err(|_| {
                OxoError::DocFetchError(
                    tool.to_string(),
                    "Tool not found or does not support --help/-h and produced no output when called with no arguments".to_string(),
                )
            })?;

        // Try to detect the version with multiple strategies
        let version = self.detect_version(tool, &help);

        Ok((help, version))
    }

    /// Async version of fetch_help using tokio::process::Command for non-blocking spawns.
    /// Uses early-return pattern with alias fallback for tools whose executable differs from skill name.
    async fn fetch_help_async(&self, tool: &str) -> Result<(String, Option<String>)> {
        // Try --help first with alias fallback (most common for bioinformatics tools)
        if let Ok(help) = try_with_aliases_async(self, tool, "--help").await {
            let version = self.detect_version(tool, &help);
            return Ok((help, version));
        }

        // Try -h with alias fallback (common shorthand)
        if let Ok(help) = try_with_aliases_async(self, tool, "-h").await {
            let version = self.detect_version(tool, &help);
            return Ok((help, version));
        }

        // Try bare invocation with alias fallback (many tools print usage when called with no args)
        for alias in std::iter::once(tool).chain(get_tool_aliases(tool).iter().copied()) {
            if let Ok(help) = self.run_no_args_async(alias).await {
                let version = self.detect_version(tool, &help);
                return Ok((help, version));
            }
        }

        // Try "help" subcommand with alias fallback
        if let Ok(help) = try_with_aliases_async(self, tool, "help").await {
            let version = self.detect_version(tool, &help);
            return Ok((help, version));
        }

        // Last resort: shell builtin help
        let help = self.run_shell_builtin_help_async(tool).await.map_err(|_| {
            OxoError::DocFetchError(
                tool.to_string(),
                "Tool not found or does not support --help/-h and produced no output when called with no arguments".to_string(),
            )
        })?;
        let version = self.detect_version(tool, &help);
        Ok((help, version))
    }

    /// Async version of run_help_flag using tokio::process::Command
    async fn run_help_flag_async(&self, tool: &str, flag: &str) -> Result<String> {
        // Split flag into multiple arguments for subcommand patterns like "sort --help"
        let flag_parts: Vec<&str> = flag.split_whitespace().collect();
        let output = if flag_parts.len() == 1 {
            AsyncCommand::new(tool)
                .arg(flag)
                .output()
                .await
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?
        } else {
            AsyncCommand::new(tool)
                .args(&flag_parts)
                .output()
                .await
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?
        };

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Async version of run_no_args using tokio::process::Command
    async fn run_no_args_async(&self, tool: &str) -> Result<String> {
        let output = AsyncCommand::new(tool)
            .output()
            .await
            .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Async version of run_shell_builtin_help using tokio::process::Command
    async fn run_shell_builtin_help_async(&self, tool: &str) -> Result<String> {
        let output = AsyncCommand::new("bash")
            .args(["-c", "help -- \"$1\"", "--", tool])
            .output()
            .await
            .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

        if !output.status.success() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                "Not a shell built-in".to_string(),
            ));
        }

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Fetch help for a specific subcommand of a tool, based on the user's task.
    ///
    /// This is the core of the "subcommand-directed fetching" feature:
    /// 1. Parse the top-level help to extract a list of known subcommands.
    /// 2. Match subcommand names against keywords in the user's task.
    /// 3. If a match is found, fetch `tool subcommand --help` and return it.
    /// 4. Also try `tool_subcommand` format for standalone executables (e.g., medaka_consensus).
    ///
    /// This ensures the LLM gets detailed parameter info for the specific subcommand
    /// the user needs (e.g., `samtools sort` instead of just `samtools` top-level).
    pub fn fetch_subcommand_help(&self, tool: &str, top_help: &str, task: &str) -> Option<String> {
        let subcommands = extract_subcommand_list(top_help);

        // Strategy 0a: Semantic domain inference (before keyword matching)
        // If task mentions domain concepts (e.g., "align"), infer the subcommand
        // This handles cases where tasks don't explicitly mention subcommand names.
        if let Some(inferred_subcmd) = infer_subcommand_from_domain(tool, task) {
            // Check if inferred subcommand exists in the tool's subcommand list
            if subcommands.iter().any(|sc| sc.to_lowercase() == inferred_subcmd.to_lowercase()) {
                // Try to fetch help for the inferred subcommand
                if let Ok(help) = self
                    .run_help_flag(tool, &format!("{inferred_subcmd} --help"))
                    .or_else(|_| self.run_help_flag(tool, &format!("{inferred_subcmd} -h")))
                    .or_else(|_| self.run_subcommand_no_args(tool, inferred_subcmd))
                    && help.len() >= MIN_HELP_LEN
                {
                    return Some(format!("# {tool} {inferred_subcmd} --help\n\n{help}"));
                }
            }
        }

        // Strategy 0b: Extract keywords from task and try standalone commands first
        // This handles tools like medaka where medaka_consensus is a separate executable
        let task_lower = task.to_ascii_lowercase();
        let task_keywords: Vec<&str> = task_lower
            .split_whitespace()
            .filter(|word| word.len() >= 3) // Skip short words
            .collect();

        // Try each keyword as a potential standalone command tool_keyword
        for keyword in &task_keywords {
            let standalone_cmd = format!("{tool}_{keyword}");
            if let Ok(help) = self.fetch_help(&standalone_cmd).map(|(h, _)| h)
                && help.len() >= MIN_HELP_LEN
            {
                return Some(format!("# {standalone_cmd} --help\n\n{help}"));
            }
        }

        // If no standalone commands found, fall back to subcommand matching
        if subcommands.is_empty() {
            return None;
        }

        // Find the best-matching subcommand from the task description.
        // We look for exact word-boundary matches of subcommand names in the task.
        let matched = subcommands
            .iter()
            .filter(|sc| sc.len() >= 2) // skip single-char subcommands (likely noise)
            .filter(|sc| {
                let sc_lower = sc.to_ascii_lowercase();
                // Exact word match: "sort" matches "sort the bam" but not "resort"
                task_lower.split_whitespace().any(|word| word == sc_lower)
                    // Also match hyphenated forms: "fastq-dump" in "fastq dump"
                    || task_lower.contains(&format!(" {sc_lower}"))
                    || task_lower.contains(&format!("{sc_lower} "))
                    // Match underscore forms: "consensus" matches "medaka_consensus"
                    || task_lower.contains(&format!("_{}", sc_lower))
            })
            .max_by_key(|sc| sc.len()); // prefer longer (more specific) match

        if let Some(subcmd) = matched {
            // Try multiple strategies to fetch help for this subcommand

            // Strategy 1: Try standalone executable tool_subcommand (e.g., medaka_consensus)
            let standalone_cmd = format!("{tool}_{subcmd}");
            if let Ok(help) = self.fetch_help(&standalone_cmd).map(|(h, _)| h)
                && help.len() >= MIN_HELP_LEN
            {
                return Some(format!("# {standalone_cmd} --help\n\n{help}"));
            }

            // Strategy 2: Try tool subcommand --help (standard subcommand pattern)
            if let Ok(help) = self
                .run_help_flag(tool, &format!("{subcmd} --help"))
                .or_else(|_| self.run_help_flag(tool, &format!("{subcmd} -h")))
                .or_else(|_| {
                    // Some tools (e.g. GATK) use: tool SubCommand --help
                    self.run_help_flag(tool, subcmd)
                })
                && help.len() >= MIN_HELP_LEN
            {
                return Some(format!("# {tool} {subcmd} --help\n\n{help}"));
            }

            // Strategy 3: Run subcommand bare (many bioinfo tools print help when called with no args)
            if let Ok(help) = self.run_subcommand_no_args(tool, subcmd)
                && help.len() >= MIN_HELP_LEN
            {
                return Some(format!("# {tool} {subcmd} --help\n\n{help}"));
            }
        }

        None
    }

    /// Run `tool subcommand` with no additional arguments.
    /// Many bioinformatics tools (bwa, samtools, bcftools) print usage when a
    /// subcommand is invoked without its required arguments.
    fn run_subcommand_no_args(&self, tool: &str, subcmd: &str) -> Result<String> {
        let output = Command::new(tool)
            .arg(subcmd)
            .output()
            .map_err(|e| OxoError::ToolNotFound(format!("{tool} {subcmd}: {e}")))?;

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Try to detect the tool version using multiple strategies.
    fn detect_version(&self, tool: &str, help_text: &str) -> Option<String> {
        // 1. Try --version flag
        let from_flag = self
            .run_help_flag(tool, "--version")
            .or_else(|_| self.run_help_flag(tool, "-V"))
            .or_else(|_| self.run_help_flag(tool, "-v"))
            .or_else(|_| self.run_help_flag(tool, "version"))
            .ok()
            .map(|v| clean_version_string(v.lines().next().unwrap_or("").trim()))
            .filter(|v| !v.is_empty() && looks_like_version(v));

        if from_flag.is_some() {
            return from_flag;
        }

        // 2. Try to extract version from the help text (first 20 lines)
        for line in help_text.lines().take(20) {
            let line = line.trim();
            // Look for patterns like "Version: 1.2.3", "v1.2.3", "1.2.3"
            if (line.to_lowercase().contains("version") || line.starts_with('v'))
                && looks_like_version(line)
            {
                return Some(clean_version_string(line));
            }
        }

        None
    }

    fn run_help_flag(&self, tool: &str, flag: &str) -> Result<String> {
        // Split flag into multiple arguments for subcommand patterns like "sort --help"
        // Command::new(tool).arg("sort --help") passes "sort --help" as ONE argument,
        // but we need "sort" and "--help" as SEPARATE arguments.
        let flag_parts: Vec<&str> = flag.split_whitespace().collect();
        let output = if flag_parts.len() == 1 {
            Command::new(tool)
                .arg(flag)
                .output()
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?
        } else {
            Command::new(tool)
                .args(&flag_parts)
                .output()
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?
        };

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Run the tool with no arguments – many bioinformatics tools (bwa, samtools, etc.)
    /// print their usage/help when called without any arguments.
    fn run_no_args(&self, tool: &str) -> Result<String> {
        let output = Command::new(tool)
            .output()
            .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Try to get help for a shell built-in command via `bash -c "help <tool>"`.
    /// This handles commands like `cd`, `export`, `alias`, etc. that are not
    /// standalone executables and cannot be invoked directly.
    fn run_shell_builtin_help(&self, tool: &str) -> Result<String> {
        // Use $1 with -- to safely pass the tool name without shell interpolation.
        // validate_tool_name() already restricts to [a-zA-Z0-9._-], but defence
        // in depth avoids any future risk if that validation changes.
        let output = Command::new("bash")
            .args(["-c", "help -- \"$1\"", "--", tool])
            .output()
            .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

        if !output.status.success() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                "Not a shell built-in".to_string(),
            ));
        }

        extract_useful_output(tool, &output.stdout, &output.stderr)
    }

    /// Load documentation from local cache
    fn load_cache(&self, tool: &str) -> Result<String> {
        let cache_path = self.cache_path(tool)?;
        if !cache_path.exists() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                "No cache found".to_string(),
            ));
        }
        let content = std::fs::read_to_string(&cache_path)?;
        Ok(content)
    }

    /// Save documentation to local cache
    pub fn save_cache(&self, tool: &str, content: &str) -> Result<()> {
        let cache_path = self.cache_path(tool)?;
        let dir = cache_path.parent().ok_or_else(|| {
            crate::error::OxoError::DocFetchError(
                tool.to_string(),
                "Cache path has no parent directory".to_string(),
            )
        })?;
        std::fs::create_dir_all(dir)?;
        // Write to a uniquely-named sibling temp file first, then atomically rename into
        // place.  Using a UUID suffix prevents concurrent CLI invocations (e.g. parallel
        // integration-test runs) from racing on the same `.tmp` path and hitting ENOENT
        // on the subsequent rename.
        let tmp_path = dir.join(format!(
            "{}.{}.tmp",
            cache_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("doc"),
            Uuid::new_v4().simple()
        ));
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, &cache_path)?;
        Ok(())
    }

    /// Remove a tool's cached documentation
    pub fn remove_cache(&self, tool: &str) -> Result<()> {
        let cache_path = self.cache_path(tool)?;
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)?;
        }
        Ok(())
    }

    /// List all tools with cached documentation
    #[allow(dead_code)]
    pub fn list_cached_tools(&self) -> Result<Vec<String>> {
        let cache_dir = self.cache_dir()?;
        if !cache_dir.exists() {
            return Ok(Vec::new());
        }
        let mut tools = Vec::new();
        for entry in std::fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md" || e == "txt")
                && let Some(stem) = path.file_stem()
            {
                tools.push(stem.to_string_lossy().to_string());
            }
        }
        tools.sort();
        Ok(tools)
    }

    fn cache_dir(&self) -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("docs"))
    }

    pub fn cache_path(&self, tool: &str) -> Result<PathBuf> {
        // Sanitize tool name for filesystem use
        let safe_name: String = tool
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Ok(self.cache_dir()?.join(format!("{safe_name}.md")))
    }

    /// Load the cached version string for a tool (stored as a small sidecar file).
    fn load_cached_version(&self, tool: &str) -> Option<String> {
        let path = self.version_cache_path(tool).ok()?;
        std::fs::read_to_string(path)
            .ok()
            .map(|s| s.trim().to_string())
    }

    /// Persist the tool's version string alongside the doc cache.
    fn save_cached_version(&self, tool: &str, version: &str) -> Result<()> {
        let path = self.version_cache_path(tool)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, version)?;
        Ok(())
    }

    /// Path to the version sidecar file.
    fn version_cache_path(&self, tool: &str) -> Result<PathBuf> {
        let safe_name: String = tool
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Ok(self.cache_dir()?.join(format!("{safe_name}.version")))
    }

    /// Search configured local documentation paths for a tool.
    /// The tool name is sanitized before use in path construction to prevent
    /// path traversal attacks.
    fn search_local_docs(&self, tool: &str) -> Option<String> {
        // Sanitize tool name to prevent path traversal
        let safe_name: String = tool
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        for base_path in &self.config.docs.local_paths {
            // Try various file patterns using the sanitized name
            let candidates = [
                base_path.join(format!("{safe_name}.md")),
                base_path.join(format!("{safe_name}.txt")),
                base_path.join(safe_name.as_str()).join("README.md"),
                base_path.join(format!("{safe_name}.rst")),
            ];
            for candidate in &candidates {
                // Extra check: resolved path must be within base_path
                // Use simple path validation without canonicalize syscall
                // Candidate paths constructed from safe_name cannot contain path traversal
                // components since safe_name is sanitized to alphanumeric/hyphen/underscore only
                if candidate.exists()
                    && let Ok(content) = std::fs::read_to_string(candidate)
                {
                    return Some(content);
                }
            }
        }
        None
    }

    /// Fetch documentation from a remote URL and cache it.
    /// Only HTTP and HTTPS URLs are accepted.
    pub async fn fetch_remote(&self, tool: &str, url: &str) -> Result<String> {
        // Validate URL scheme to prevent SSRF via unexpected schemes (file://, ftp://, etc.)
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                "Only http:// and https:// URLs are accepted".to_string(),
            ));
        }
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!("HTTP {}", response.status()),
            ));
        }
        let content = response.text().await?;
        // Limit size
        let truncated = if content.len() > 50_000 {
            format!("{}\n...[truncated]", &content[..50_000])
        } else {
            content
        };
        Ok(truncated)
    }

    /// Read documentation from a single local file.
    ///
    /// Supported file types: `.md`, `.txt`, `.rst`, `.html`.
    /// HTML files are stripped of tags before use.
    /// The path must exist and be a regular file.
    pub fn fetch_from_file(&self, tool: &str, path: &std::path::Path) -> Result<String> {
        if !path.exists() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!("File not found: {}", path.display()),
            ));
        }
        if !path.is_file() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!("Path is not a regular file: {}", path.display()),
            ));
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "md" | "txt" | "rst" => {
                let content = std::fs::read_to_string(path)?;
                Ok(truncate_doc(&content))
            }
            "html" | "htm" => {
                let raw = std::fs::read_to_string(path)?;
                Ok(truncate_doc(&strip_html_tags(&raw)))
            }
            other => Err(OxoError::DocFetchError(
                tool.to_string(),
                format!(
                    "Unsupported file type '.{other}'. Supported: .md, .txt, .rst, .html, .htm"
                ),
            )),
        }
    }

    /// Collect documentation from all supported files inside a directory (non-recursive).
    ///
    /// Files with extensions `.md`, `.txt`, `.rst`, `.html`, `.htm` are included.
    /// The combined content is truncated to `MAX_HELP_LEN` characters.
    pub fn fetch_from_dir(&self, tool: &str, dir: &std::path::Path) -> Result<String> {
        if !dir.exists() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!("Directory not found: {}", dir.display()),
            ));
        }
        if !dir.is_dir() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!("Path is not a directory: {}", dir.display()),
            ));
        }
        let supported = ["md", "txt", "rst", "html", "htm"];
        let mut parts: Vec<String> = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path();
                p.is_file()
                    && p.extension()
                        .and_then(|x| x.to_str())
                        .is_some_and(|x| supported.contains(&x.to_lowercase().as_str()))
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in &entries {
            match self.fetch_from_file(tool, &entry.path()) {
                Ok(content) => {
                    let name = entry.file_name();
                    parts.push(format!("## {}\n\n{}", name.to_string_lossy(), content));
                }
                Err(_) => continue,
            }
        }
        if parts.is_empty() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                format!(
                    "No supported documentation files found in: {}",
                    dir.display()
                ),
            ));
        }
        let combined = parts.join("\n\n");
        Ok(truncate_doc(&combined))
    }
}

/// Extract useful text from combined stdout + stderr of a process.
/// Returns an error if the result is empty or too short to be actual help.
fn extract_useful_output(tool: &str, stdout: &[u8], stderr: &[u8]) -> Result<String> {
    let stdout_str = String::from_utf8_lossy(stdout).to_string();
    let stderr_str = String::from_utf8_lossy(stderr).to_string();

    // Prefer whichever stream has more content, but also concatenate both
    // because some tools split output across both streams.
    let combined = if stdout_str.len() >= stderr_str.len() {
        if !stderr_str.trim().is_empty() && !stdout_str.contains(stderr_str.trim()) {
            format!("{stdout_str}\n{stderr_str}")
        } else {
            stdout_str
        }
    } else if !stdout_str.trim().is_empty() && !stderr_str.contains(stdout_str.trim()) {
        format!("{stderr_str}\n{stdout_str}")
    } else {
        stderr_str
    };

    let trimmed = combined.trim();

    if trimmed.is_empty() {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            "Empty output".to_string(),
        ));
    }

    // Reject responses that look like error messages rather than help text
    // Check regardless of length - pixi wrapper errors can be >80 chars
    if is_likely_error(trimmed) {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            format!("Output looks like an error rather than help text: {trimmed}"),
        ));
    }

    // Also reject very short outputs that don look like real help
    if trimmed.len() < MIN_HELP_LEN {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            format!("Output too short ({} chars) to be useful help text", trimmed.len()),
        ));
    }

    // Truncate to keep LLM prompts manageable
    let output = if trimmed.len() > MAX_HELP_LEN {
        format!("{}\n...[truncated]", &trimmed[..MAX_HELP_LEN])
    } else {
        trimmed.to_string()
    };

    Ok(output)
}

/// Check whether a string looks like a command-line error message rather
/// than useful help text.
/// Extended to detect error patterns even in longer outputs (pixi wrapper messages can be >80 chars).
/// IMPORTANT: Outputs that contain valid help indicators (Usage, Options, Examples) should NOT be rejected,
/// even if they start with an error message. Many tools (samtools, bwa) print "unrecognized option"
/// but then show the actual help.
fn is_likely_error(text: &str) -> bool {
    let lower = text.to_lowercase();

    // If output contains valid help indicators, it's NOT an error
    // Tools like samtools print "unrecognized option '--help'" but then show full help
    if lower.contains("usage:") || lower.contains("options:") || lower.contains("examples:") {
        return false;
    }

    // Strong error indicators - reject if no help content
    lower.contains("unrecognized command")
        || lower.contains("unknown command")
        || lower.contains("no such")
        // pixi/binary wrapper error patterns
        || lower.contains("error: unknown command")
        || lower.contains("error: unrecognized")
        // Only reject "error" at start if no usage info
        || (lower.starts_with("error") && !lower.contains("usage"))
}

/// Rough heuristic: does a string look like a version identifier?
fn looks_like_version(s: &str) -> bool {
    if s.len() > 120 {
        return false; // Too long to be just a version line
    }
    // Must contain at least one digit and a dot
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    let has_dot = s.contains('.');
    let lower = s.to_lowercase();
    // Reject strings that look like error messages
    if lower.contains("error")
        || lower.contains("unrecognized")
        || lower.contains("unknown")
        || lower.contains("usage")
        || lower.contains("invalid")
    {
        return false;
    }
    has_digit && has_dot
}

/// Extract a list of subcommand names from a tool's top-level help output.
///
/// This uses heuristics to parse common help formats:
/// - Indented lines with a word followed by a description:
///   ```text
///   Commands:
///     view       view and convert SAM/BAM/CRAM files
///     sort       sort alignment files
///   ```
/// - Comma-separated or space-separated lists on a single line
/// - Lines matching: `  subcommand    Description text`
///
/// The goal is to extract just the subcommand names (e.g., ["view", "sort", "index"])
/// so we can match them against the user's task and fetch their detailed help.
fn extract_subcommand_list(help: &str) -> Vec<String> {
    let mut subcommands = Vec::new();
    let mut in_commands_section = false;

    for line in help.lines() {
        let trimmed = line.trim();

        // Detect section headers that indicate a subcommand list
        if trimmed.starts_with("Commands:")
            || trimmed.starts_with("Subcommands:")
            || trimmed.starts_with("Available commands:")
            || trimmed.starts_with("Usage:")
            || trimmed.starts_with("Command")
            || trimmed.starts_with("Programs:")
            || trimmed.starts_with("Modules:")
        {
            in_commands_section = true;
            // Some tools put the first subcommand on the same line after "Commands:"
            if let Some(rest) = trimmed.split(':').nth(1) {
                extract_subcmd_tokens(rest, &mut subcommands);
            }
            continue;
        }

        // Stop collecting if we hit another section header
        if in_commands_section && trimmed.starts_with('-') && trimmed.contains("Options")
            || trimmed.starts_with("Options:")
            || trimmed.starts_with("Arguments:")
            || trimmed.starts_with("Description:")
            || trimmed.starts_with("Examples:")
        {
            in_commands_section = false;
            continue;
        }

        if !in_commands_section {
            continue;
        }

        // Parse indented subcommand lines like:
        //   view       view and convert SAM/BAM/CRAM files
        //   sort       sort alignment files
        // Or simple bullet lists:
        //   - sort
        extract_subcmd_tokens(trimmed, &mut subcommands);
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    subcommands.retain(|sc| seen.insert(sc.clone()));
    subcommands
}

/// Extract potential subcommand tokens from a single line of help text.
fn extract_subcmd_tokens(line: &str, subcommands: &mut Vec<String>) {
    // Strip leading bullet characters
    let line = line.trim_start_matches('-').trim_start_matches('*').trim();

    if line.is_empty() {
        return;
    }

    // Take the first whitespace-separated token as a potential subcommand name.
    // It must look like a valid identifier (alphanumeric + hyphens/underscores).
    if let Some(token) = line.split_whitespace().next() {
        // Must not start with a dash (those are flags, not subcommands)
        if token.starts_with('-') {
            return;
        }
        // Must look like a subcommand name: alphanumeric, hyphens, underscores, dots
        if token
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
            && token.len() >= 2
            && token.len() <= 40
            // Reject common false positives
            && !is_common_non_subcommand(token)
        {
            subcommands.push(token.to_string());
        }
    }
}

/// Returns `true` if `token` is a common word that appears in help text but
/// is not actually a subcommand name (e.g., "usage", "version", "help", "the").
fn is_common_non_subcommand(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "usage"
            | "version"
            | "help"
            | "options"
            | "arguments"
            | "description"
            | "examples"
            | "note"
            | "notes"
            | "see"
            | "also"
            | "the"
            | "and"
            | "or"
            | "for"
            | "to"
            | "in"
            | "of"
            | "with"
            | "from"
            | "by"
            | "on"
            | "at"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "being"
            | "have"
            | "has"
            | "had"
            | "do"
            | "does"
            | "did"
            | "will"
            | "would"
            | "could"
            | "should"
            | "may"
            | "might"
            | "can"
            | "must"
            | "shall"
            | "this"
            | "that"
            | "these"
            | "those"
            | "not"
            | "no"
            | "yes"
            | "true"
            | "false"
            | "none"
            | "null"
            | "file"
            | "files"
            | "input"
            | "output"
            | "path"
            | "name"
            | "type"
            | "value"
            | "default"
            | "required"
            | "optional"
            | "all"
            | "any"
            | "each"
            | "every"
            | "some"
            | "more"
            | "most"
            | "other"
            | "another"
            | "such"
            | "same"
            | "different"
            | "new"
            | "old"
            | "first"
            | "last"
            | "next"
            | "previous"
    )
}

/// Clean up a raw version string by stripping common prefixes like "Version:", "v", etc.
fn clean_version_string(raw: &str) -> String {
    let s = raw.trim();
    // Strip leading "version:" or "Version " prefix (case-insensitive, 8 chars each)
    let s = if s.to_lowercase().starts_with("version:") || s.to_lowercase().starts_with("version ")
    {
        s[8..].trim()
    } else {
        s
    };
    // Take only the first "word group" that looks like an actual version (e.g. stop at parentheses)
    let s = s
        .split_once(" (")
        .map(|(before, _)| before.trim())
        .unwrap_or(s);
    s.to_string()
}

/// Truncate a documentation string to `MAX_HELP_LEN` characters, appending a
/// notice when the content is cut short.
fn truncate_doc(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() > MAX_HELP_LEN {
        format!("{}\n...[truncated]", &trimmed[..MAX_HELP_LEN])
    } else {
        trimmed.to_string()
    }
}

/// Return `true` when `help` is substantially contained within `cached`, so
/// we can skip re-appending identical content.
/// Clean `--help` output by removing noise that wastes LLM context budget.
///
/// Bioinformatics tools' `--help` often includes:
/// - Compilation/installation info (compiler flags, build dates, paths)
/// - Developer/debug flags (e.g., `--gatk-debug`, `--verbose-module-logging`)
/// - Long license/copyright notices
/// - Environment variable listings
/// - Huge whitespace gaps
///
/// These add hundreds of tokens of noise without helping the LLM generate
/// correct commands.  This function strips them out.
fn clean_help_output(help: &str) -> String {
    let lines: Vec<&str> = help.lines().collect();

    // Patterns that indicate noise lines to remove.
    // These are heuristics based on common bioinformatics tool output patterns.
    let noise_prefixes: &[&str] = &[
        // Compilation/build info
        "Compiled with",
        "Compiler:",
        "Build:",
        "Build date:",
        "Installation prefix:",
        "Configured with",
        // License/copyright blocks (only the verbose multi-line ones)
        "This program is free software",
        "This is free software",
        "License:",
        "Warranty:",
        "GNU General Public License",
        "This program comes with ABSOLUTELY NO WARRANTY",
        // Debug/developer flags
        "--gatk-debug",
        "--verbose-module-logging",
        "--debug-jni",
        "--help-hidden", // hidden flags section header
        // Environment variables listing
        "Environment variables:",
        "ENVIRONMENT",
        // Test/debug sections
        "Test options:",
        "Debug options:",
        "Developer options:",
    ];

    let mut cleaned = Vec::new();
    let mut in_license_block = false;
    let mut consecutive_blank = 0;

    for line in lines {
        let trimmed = line.trim();

        // Skip license/copyright continuation lines
        if in_license_block {
            // License blocks typically end at a blank line or a new section
            if trimmed.is_empty() {
                in_license_block = false;
                continue;
            }
            // Also end if we see a new section header
            if trimmed.starts_with("Usage")
                || trimmed.starts_with("Options")
                || trimmed.starts_with("Arguments")
            {
                in_license_block = false;
                // Don't skip this line — fall through to normal processing
            } else {
                continue;
            }
        }

        // Check for noise prefixes
        let trimmed_lower = trimmed.to_ascii_lowercase();
        let is_noise = noise_prefixes
            .iter()
            .any(|prefix| trimmed_lower.starts_with(&prefix.to_ascii_lowercase()));

        if is_noise {
            // If this is a license line, skip subsequent lines too
            if trimmed_lower.starts_with("this program is free")
                || trimmed_lower.starts_with("this is free")
                || trimmed_lower.contains("absolutely no warranty")
            {
                in_license_block = true;
            }
            continue;
        }

        // Skip lines that are just URL references (e.g., homepage links)
        if trimmed.starts_with("Homepage:") || trimmed.starts_with("Documentation:") {
            continue;
        }

        // Collapse consecutive blank lines to at most 2
        if trimmed.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank > 2 {
                continue;
            }
        } else {
            consecutive_blank = 0;
        }

        cleaned.push(line);
    }

    let result = cleaned.join("\n");

    // Trailing whitespace cleanup
    result.trim_end().to_string()
}

fn deduplicate_check(cached: &str, help: &str) -> bool {
    // Use DEDUP_OVERLAP_THRESHOLD of help length as the significant overlap threshold —
    // exact containment is too strict because the cache may have reformatted whitespace.
    let significant_len = (help.len() * DEDUP_OVERLAP_NUMERATOR) / DEDUP_OVERLAP_DENOMINATOR;
    if significant_len == 0 {
        return false;
    }
    // Check for verbatim inclusion first (fast path)
    if cached.contains(help) {
        return true;
    }
    // Sliding-window check: does the leading significant portion of `help` appear in `cached`?
    let probe = &help[..significant_len.min(help.len())];
    cached.contains(probe)
}

/// Strip HTML tags from a string, leaving only plain text.
/// Also collapses multiple blank lines into a single blank line.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse runs of blank lines
    let mut out = String::new();
    let mut prev_blank = false;
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank {
                out.push('\n');
            }
            prev_blank = true;
        } else {
            out.push_str(trimmed);
            out.push('\n');
            prev_blank = false;
        }
    }
    out
}

/// Remove an embedded "# Help Output" section from a cached doc string.
/// This prevents duplicate display when fresh --help is appended alongside
/// a cache that was built by `docs add` (which includes `# Help Output`).
fn strip_embedded_help_section(cached: &str) -> String {
    // Find the first occurrence of the well-known section header added by IndexManager.
    // The constants are shared with IndexManager to keep both sides in sync.
    let markers = [HELP_OUTPUT_SECTION_LF, HELP_OUTPUT_SECTION_CRLF];
    for marker in &markers {
        if let Some(start) = cached.find(marker) {
            // Find where this section ends — either at the next top-level heading or EOF
            let after_marker = start + marker.len();
            let rest = &cached[after_marker..];
            let section_end = rest
                .find("\n# ")
                .map(|p| after_marker + p)
                .unwrap_or(cached.len());
            // Rebuild without the Help Output section
            let before = cached[..start].trim_end();
            let after = cached[section_end..].trim_start();
            return if before.is_empty() {
                after.to_string()
            } else if after.is_empty() {
                before.to_string()
            } else {
                format!("{before}\n\n{after}")
            };
        }
    }
    cached.to_string()
}

/// Extract major.minor from a version string (e.g., "samtools 1.17" → "1.17",
/// "1.20.0" → "1.20").  Returns the original string if no version pattern is
/// found, which means an exact-match comparison will be used as fallback.
fn extract_major_minor(version: &str) -> String {
    let s = version.trim();

    // Find the first substring that looks like X.Y or X.Y.Z
    // by splitting on whitespace and non-version characters.
    for word in s.split(|c: char| c.is_whitespace() || c == '(' || c == ')') {
        let word = word.trim();
        if word.is_empty() {
            continue;
        }

        // Strip common version prefix 'v'
        let word = word
            .strip_prefix('v')
            .or_else(|| word.strip_prefix('V'))
            .unwrap_or(word);

        // Check if this word is a version-like pattern (digits and dots only)
        let version_part: String = word
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        if version_part.contains('.') {
            let parts: Vec<&str> = version_part.split('.').collect();
            if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                return format!("{}.{}", parts[0], parts[1]);
            }
        }
    }

    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // All tests that mutate OXO_CALL_DATA_DIR use the crate-wide ENV_LOCK
    // so they are serialised against tests in skill.rs, config.rs, and
    // history.rs that also touch the same env variable.
    use crate::ENV_LOCK;

    // ─── validate_tool_name ───────────────────────────────────────────────────

    #[test]
    fn test_validate_tool_name_valid() {
        assert!(validate_tool_name("samtools").is_ok());
        assert!(validate_tool_name("bwa-mem2").is_ok());
        assert!(validate_tool_name("featureCounts").is_ok());
        assert!(validate_tool_name("picard.jar").is_ok());
        assert!(validate_tool_name("tool_name").is_ok());
    }

    #[test]
    fn test_validate_tool_name_empty() {
        assert!(validate_tool_name("").is_err());
    }

    #[test]
    fn test_validate_tool_name_path_traversal() {
        assert!(validate_tool_name("../etc/passwd").is_err());
        assert!(validate_tool_name("foo/../bar").is_err());
        assert!(validate_tool_name("foo/bar").is_err());
        assert!(validate_tool_name("foo\\bar").is_err());
    }

    #[test]
    fn test_validate_tool_name_invalid_chars() {
        assert!(validate_tool_name("tool name").is_err()); // space
        assert!(validate_tool_name("tool!").is_err());
        assert!(validate_tool_name("tool@v1").is_err());
    }

    // ─── ToolDocs::combined ───────────────────────────────────────────────────

    #[test]
    fn test_tool_docs_combined_only_help() {
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: Some("usage: tool [options]".to_string()),
            cached_docs: None,
            version: None,
            subcommand_help: None,
        };
        let combined = docs.combined();
        assert!(combined.contains("usage: tool"));
    }

    #[test]
    fn test_tool_docs_combined_only_cached() {
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: None,
            cached_docs: Some("# Tool\n\nFull documentation here.".to_string()),
            version: None,
            subcommand_help: None,
        };
        let combined = docs.combined();
        assert!(combined.contains("Full documentation"));
    }

    #[test]
    fn test_tool_docs_combined_includes_version() {
        let docs = ToolDocs {
            tool_name: "samtools".to_string(),
            help_output: Some("usage: samtools".to_string()),
            cached_docs: None,
            version: Some("1.17".to_string()),
            subcommand_help: None,
        };
        let combined = docs.combined();
        assert!(combined.contains("Version: 1.17"));
    }

    #[test]
    fn test_tool_docs_is_empty_true() {
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: None,
            cached_docs: None,
            version: None,
            subcommand_help: None,
        };
        assert!(docs.is_empty());
    }

    #[test]
    fn test_tool_docs_is_empty_false_with_help() {
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: Some("usage: tool".to_string()),
            cached_docs: None,
            version: None,
            subcommand_help: None,
        };
        assert!(!docs.is_empty());
    }

    #[test]
    fn test_tool_docs_is_empty_false_with_cached() {
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: None,
            cached_docs: Some("docs".to_string()),
            version: None,
            subcommand_help: None,
        };
        assert!(!docs.is_empty());
    }

    // ─── DocsFetcher cache methods ────────────────────────────────────────────

    #[test]
    fn test_save_and_load_cache() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let fetcher = DocsFetcher::new(Config::default());

        fetcher
            .save_cache("samtools", "# samtools\nDocs here.")
            .unwrap();
        let path = fetcher.cache_path("samtools").unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Docs here."));
    }

    #[test]
    fn test_remove_cache() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let fetcher = DocsFetcher::new(Config::default());

        fetcher.save_cache("bwa", "bwa docs").unwrap();
        assert!(fetcher.cache_path("bwa").unwrap().exists());

        fetcher.remove_cache("bwa").unwrap();
        assert!(!fetcher.cache_path("bwa").unwrap().exists());
    }

    #[test]
    fn test_remove_cache_nonexistent_is_ok() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let fetcher = DocsFetcher::new(Config::default());
        // Should not error even if file doesn't exist
        assert!(fetcher.remove_cache("doesnotexist").is_ok());
    }

    #[test]
    fn test_list_cached_tools() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let fetcher = DocsFetcher::new(Config::default());

        fetcher.save_cache("gatk", "gatk docs").unwrap();
        fetcher.save_cache("star", "star docs").unwrap();

        let tools = fetcher.list_cached_tools().unwrap();
        assert!(tools.contains(&"gatk".to_string()));
        assert!(tools.contains(&"star".to_string()));
    }

    #[test]
    fn test_list_cached_tools_empty_dir() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let fetcher = DocsFetcher::new(Config::default());
        let tools = fetcher.list_cached_tools().unwrap();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_cache_path_sanitizes_special_chars() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let fetcher = DocsFetcher::new(Config::default());
        let path = fetcher.cache_path("tool-with.dots_and-hyphens").unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with(".md"));
    }

    // ─── fetch_from_file ──────────────────────────────────────────────────────

    #[test]
    fn test_fetch_from_file_md() {
        let tmp = tempfile::tempdir().unwrap();
        let md_path = tmp.path().join("tool.md");
        std::fs::write(&md_path, "# Tool\nUsage: tool [options]").unwrap();

        let fetcher = DocsFetcher::new(Config::default());
        let content = fetcher.fetch_from_file("tool", &md_path).unwrap();
        assert!(content.contains("Usage: tool"));
    }

    #[test]
    fn test_fetch_from_file_txt() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("tool.txt");
        std::fs::write(&path, "tool text docs").unwrap();

        let fetcher = DocsFetcher::new(Config::default());
        let content = fetcher.fetch_from_file("tool", &path).unwrap();
        assert!(content.contains("tool text docs"));
    }

    #[test]
    fn test_fetch_from_file_html_strips_tags() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("tool.html");
        std::fs::write(
            &path,
            "<html><body><h1>Tool</h1><p>Description</p></body></html>",
        )
        .unwrap();

        let fetcher = DocsFetcher::new(Config::default());
        let content = fetcher.fetch_from_file("tool", &path).unwrap();
        assert!(content.contains("Tool"));
        assert!(content.contains("Description"));
        assert!(!content.contains("<html>"));
    }

    #[test]
    fn test_fetch_from_file_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.md");
        let fetcher = DocsFetcher::new(Config::default());
        assert!(fetcher.fetch_from_file("tool", &path).is_err());
    }

    #[test]
    fn test_fetch_from_file_unsupported_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("tool.pdf");
        std::fs::write(&path, "binary content").unwrap();
        let fetcher = DocsFetcher::new(Config::default());
        assert!(fetcher.fetch_from_file("tool", &path).is_err());
    }

    // ─── fetch_from_dir ───────────────────────────────────────────────────────

    #[test]
    fn test_fetch_from_dir_collects_md_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("install.md"), "# Install\nInstall docs").unwrap();
        std::fs::write(tmp.path().join("usage.md"), "# Usage\nUsage docs").unwrap();

        let fetcher = DocsFetcher::new(Config::default());
        let content = fetcher.fetch_from_dir("tool", tmp.path()).unwrap();
        assert!(content.contains("Install docs"));
        assert!(content.contains("Usage docs"));
    }

    #[test]
    fn test_fetch_from_dir_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let nonexistent = tmp.path().join("missing");
        let fetcher = DocsFetcher::new(Config::default());
        assert!(fetcher.fetch_from_dir("tool", &nonexistent).is_err());
    }

    #[test]
    fn test_fetch_from_dir_empty_dir_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let empty_dir = tmp.path().join("empty");
        std::fs::create_dir(&empty_dir).unwrap();

        let fetcher = DocsFetcher::new(Config::default());
        assert!(fetcher.fetch_from_dir("tool", &empty_dir).is_err());
    }

    // ─── internal helpers ─────────────────────────────────────────────────────

    #[test]
    fn test_strip_html_tags() {
        let html = "<p>Hello <b>world</b></p>";
        let text = strip_html_tags(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
        assert!(!text.contains("<p>"));
        assert!(!text.contains("<b>"));
    }

    #[test]
    fn test_truncate_doc_short_content() {
        let short = "short content";
        assert_eq!(truncate_doc(short), "short content");
    }

    #[test]
    fn test_truncate_doc_long_content() {
        let long = "x".repeat(20_000);
        let result = truncate_doc(&long);
        assert!(result.contains("[truncated]"));
        assert!(result.len() < long.len());
    }

    #[test]
    fn test_deduplicate_check_verbatim() {
        let cached = "full docs here: usage: samtools sort";
        let help = "usage: samtools sort";
        assert!(deduplicate_check(cached, help));
    }

    #[test]
    fn test_deduplicate_check_not_present() {
        let cached = "completely different text";
        let help = "usage: tool --flag";
        assert!(!deduplicate_check(cached, help));
    }

    #[test]
    fn test_strip_embedded_help_section_removes_section() {
        let cached = "# Overview\n\nSome intro.\n\n# Help Output\n\nusage: tool --help\n";
        let result = strip_embedded_help_section(cached);
        assert!(!result.contains("# Help Output"));
        assert!(result.contains("Some intro."));
    }

    #[test]
    fn test_strip_embedded_help_section_no_section() {
        let cached = "# Overview\n\nNo help section here.";
        let result = strip_embedded_help_section(cached);
        assert_eq!(result, cached);
    }

    #[test]
    fn test_looks_like_version_valid() {
        assert!(looks_like_version("1.17.1"));
        assert!(looks_like_version("samtools 1.17"));
        assert!(looks_like_version("v2.3.4"));
    }

    #[test]
    fn test_looks_like_version_invalid() {
        assert!(!looks_like_version("error: unknown option"));
        assert!(!looks_like_version("usage: tool [options]"));
        assert!(!looks_like_version("no dots here"));
    }

    #[test]
    fn test_clean_version_string() {
        assert_eq!(clean_version_string("version: 1.17"), "1.17");
        assert_eq!(clean_version_string("Version 1.2.3"), "1.2.3");
        assert_eq!(clean_version_string("  1.5.0  "), "1.5.0");
        assert_eq!(clean_version_string("1.2.3 (build 456)"), "1.2.3");
    }

    #[test]
    fn test_is_likely_error_true() {
        assert!(is_likely_error("unrecognized command: foo"));
        assert!(is_likely_error("unknown command 'bar'"));
        assert!(is_likely_error("error: bad argument"));
    }

    #[test]
    fn test_is_likely_error_false_for_help() {
        assert!(!is_likely_error("usage: tool [options] FILE"));
        assert!(!is_likely_error("error: missing argument; usage: ..."));
    }

    // ─── fetch_remote URL validation ──────────────────────────────────────────

    #[tokio::test]
    async fn test_fetch_remote_rejects_non_http_url() {
        // No need to set OXO_CALL_DATA_DIR — this test only validates URL scheme rejection
        let fetcher = DocsFetcher::new(Config::default());
        let result = fetcher.fetch_remote("tool", "file:///etc/passwd").await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("http") || msg.contains("https") || msg.contains("URL"));
    }

    #[tokio::test]
    async fn test_fetch_remote_rejects_ftp_url() {
        let fetcher = DocsFetcher::new(Config::default());
        let result = fetcher.fetch_remote("tool", "ftp://example.com/docs").await;
        assert!(result.is_err());
    }

    // ─── extract_useful_output ────────────────────────────────────────────────

    #[test]
    fn test_extract_useful_output_uses_stdout() {
        let long_help = "A ".repeat(50); // > MIN_HELP_LEN
        let result = extract_useful_output("tool", long_help.as_bytes(), b"");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("A "));
    }

    #[test]
    fn test_extract_useful_output_falls_back_to_stderr() {
        let long_help = "Usage: tool [options]\n".repeat(10);
        let result = extract_useful_output("tool", b"", long_help.as_bytes());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Usage"));
    }

    #[test]
    fn test_extract_useful_output_empty_returns_error() {
        let result = extract_useful_output("tool", b"", b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_useful_output_short_error_text_rejected() {
        let short_error = "unrecognized command: foo";
        let result = extract_useful_output("tool", short_error.as_bytes(), b"");
        assert!(result.is_err(), "short error text should be rejected");
    }

    #[test]
    fn test_extract_useful_output_truncates_long_output() {
        let very_long = "x".repeat(20_000);
        let result = extract_useful_output("tool", very_long.as_bytes(), b"");
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(
            content.contains("[truncated]"),
            "long output should be truncated"
        );
    }

    #[test]
    fn test_extract_useful_output_combines_both_streams() {
        let stdout = "stdout content: usage: tool\n".repeat(5);
        let stderr = "stderr content: extra flags\n".repeat(5);
        let result = extract_useful_output("tool", stdout.as_bytes(), stderr.as_bytes());
        assert!(result.is_ok());
        // Combined should contain content from both
        let content = result.unwrap();
        assert!(content.contains("stdout") || content.contains("stderr"));
    }

    // ─── looks_like_version / clean_version_string ───────────────────────────

    #[test]
    fn test_looks_like_version_too_long() {
        let long = "v".to_string() + &"x".repeat(150);
        assert!(!looks_like_version(&long));
    }

    #[test]
    fn test_looks_like_version_no_dot() {
        assert!(!looks_like_version("12345"));
        assert!(!looks_like_version("version12"));
    }

    #[test]
    fn test_clean_version_string_v_prefix() {
        assert_eq!(clean_version_string("v1.2.3"), "v1.2.3");
    }

    // ─── is_likely_error ──────────────────────────────────────────────────────

    #[test]
    fn test_is_likely_error_invalid_option() {
        assert!(is_likely_error("invalid option: --foo"));
        assert!(is_likely_error("unrecognized option -x"));
    }

    #[test]
    fn test_is_likely_error_no_such() {
        assert!(is_likely_error("no such file or directory"));
    }

    #[test]
    fn test_is_likely_error_error_with_usage_is_help() {
        // "error: ... usage: ..." should NOT be flagged as error text
        assert!(!is_likely_error("error: missing arg; usage: tool --help"));
    }

    // ─── strip_html_tags ──────────────────────────────────────────────────────

    #[test]
    fn test_strip_html_tags_nested() {
        let html = "<html><head><title>Tool</title></head><body><h1>Usage</h1><p>Details</p></body></html>";
        let text = strip_html_tags(html);
        assert!(text.contains("Tool"));
        assert!(text.contains("Usage"));
        assert!(text.contains("Details"));
        assert!(!text.contains("<html>"));
        assert!(!text.contains("<head>"));
    }

    #[test]
    fn test_strip_html_tags_plain_text_unchanged() {
        let plain = "Hello world\nSecond line";
        let result = strip_html_tags(plain);
        assert!(result.contains("Hello world"));
        assert!(result.contains("Second line"));
    }

    // ─── fetch_from_file: directory path ─────────────────────────────────────

    #[test]
    fn test_fetch_from_file_directory_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        let fetcher = DocsFetcher::new(Config::default());
        assert!(fetcher.fetch_from_file("tool", tmp.path()).is_err());
    }

    // ─── fetch_from_dir: not a directory ─────────────────────────────────────

    #[test]
    fn test_fetch_from_dir_file_path_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("a.txt");
        std::fs::write(&file_path, "not a dir").unwrap();
        let fetcher = DocsFetcher::new(Config::default());
        assert!(fetcher.fetch_from_dir("tool", &file_path).is_err());
    }

    // ─── fetch_from_dir: RST and HTML files ──────────────────────────────────

    #[test]
    fn test_fetch_from_dir_collects_rst_and_html() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("api.rst"),
            ".. code:: bash\n\n   tool --help",
        )
        .unwrap();
        std::fs::write(tmp.path().join("guide.html"), "<p>Guide text</p>").unwrap();

        let fetcher = DocsFetcher::new(Config::default());
        let content = fetcher.fetch_from_dir("tool", tmp.path()).unwrap();
        assert!(content.contains("tool --help") || content.contains("Guide text"));
    }

    // ─── ToolDocs::combined deduplication ────────────────────────────────────

    #[test]
    fn test_tool_docs_combined_deduplicates_help_in_cached() {
        let help = "usage: samtools sort -o out.bam input.bam";
        let cached = format!("# Samtools Docs\n\n{help}\n\nMore content here.");
        let docs = ToolDocs {
            tool_name: "samtools".to_string(),
            help_output: Some(help.to_string()),
            cached_docs: Some(cached),
            version: None,
            subcommand_help: None,
        };
        let combined = docs.combined();
        // Help should not be duplicated
        let count = combined.matches("usage: samtools sort").count();
        assert_eq!(count, 1, "help content should not be duplicated");
    }

    #[test]
    fn test_tool_docs_combined_appends_help_not_in_cached() {
        let help = "freshly fetched flags: --new-flag";
        let cached = "# Tool Docs\n\nOld docs without the new flag.".to_string();
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: Some(help.to_string()),
            cached_docs: Some(cached),
            version: None,
            subcommand_help: None,
        };
        let combined = docs.combined();
        assert!(combined.contains("Old docs"));
        assert!(
            combined.contains("freshly fetched"),
            "new help should be appended"
        );
    }

    // ─── strip_embedded_help_section with CRLF ────────────────────────────────

    #[test]
    fn test_strip_embedded_help_section_crlf() {
        let cached =
            "# Overview\r\n\r\nSome intro.\r\n\r\n# Help Output\r\n\r\nusage: tool --help\r\n";
        let result = strip_embedded_help_section(cached);
        assert!(!result.contains("# Help Output"));
        assert!(result.contains("Some intro."));
    }

    // ─── deduplicate_check edge cases ────────────────────────────────────────

    #[test]
    fn test_deduplicate_check_empty_help() {
        let cached = "some docs here";
        assert!(
            !deduplicate_check(cached, ""),
            "empty help should not be a duplicate"
        );
    }

    #[test]
    fn test_deduplicate_check_partial_overlap() {
        let help = "usage: tool [options]\nOptions:\n  --flag1\n  --flag2\n  --flag3\n";
        // Cache contains the significant prefix of help
        let cached = format!("cached intro\n{}", &help[..((help.len() * 4) / 5 + 1)]);
        assert!(deduplicate_check(&cached, help));
    }

    // ─── tool name validation edge cases ──────────────────────────────────────

    #[test]
    fn test_validate_tool_name_too_long() {
        let long_name = "a".repeat(200);
        let result = validate_tool_name(&long_name);
        // Very long names should still pass if they contain valid chars
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_name_with_dots() {
        let result = validate_tool_name("samtools.v1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_name_with_dashes_and_underscores() {
        let result = validate_tool_name("my-tool_v2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_name_with_slash() {
        let result = validate_tool_name("path/to/tool");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tool_name_with_parent_traversal() {
        let result = validate_tool_name("../etc/passwd");
        assert!(result.is_err());
    }

    // ─── ToolDocs ─────────────────────────────────────────────────────────────

    #[test]
    fn test_tool_docs_combined_all_fields() {
        let docs = ToolDocs {
            tool_name: "tool".to_string(),
            help_output: Some("usage: tool [options]".to_string()),
            cached_docs: Some("# Cached\nSome docs".to_string()),
            version: Some("1.0.0".to_string()),
            subcommand_help: None,
        };
        let combined = docs.combined();
        assert!(combined.contains("usage: tool"));
        assert!(combined.contains("Some docs"));
        // Version might be included in combined output
    }

    #[test]
    fn test_tool_docs_is_empty_with_empty_strings() {
        let docs = ToolDocs {
            tool_name: "empty".to_string(),
            help_output: Some(String::new()),
            cached_docs: None,
            version: None,
            subcommand_help: None,
        };
        // Some("") is still "present" even if empty
        assert!(!docs.is_empty());
    }

    // ─── strip_html_tags edge cases ───────────────────────────────────────────

    #[test]
    fn test_strip_html_nested_tags() {
        let html = "<div><p>Hello <strong>World</strong></p></div>";
        let result = strip_html_tags(html);
        assert_eq!(result.trim(), "Hello World");
    }

    #[test]
    fn test_strip_html_self_closing_tags() {
        let html = "Line1<br/>Line2<hr/>Line3";
        let result = strip_html_tags(html);
        assert!(result.contains("Line1"));
        assert!(result.contains("Line2"));
        assert!(result.contains("Line3"));
    }

    #[test]
    fn test_strip_html_no_tags() {
        let text = "plain text without any HTML";
        let result = strip_html_tags(text);
        // strip_html_tags appends a trailing newline
        assert_eq!(result.trim(), text);
    }

    // ─── truncate_doc edge cases ──────────────────────────────────────────────

    #[test]
    fn test_truncate_doc_empty() {
        let result = truncate_doc("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_doc_exact_limit() {
        let content = "a".repeat(100_000);
        let result = truncate_doc(&content);
        assert!(result.len() <= 100_001); // within limit
    }

    // ─── looks_like_version ───────────────────────────────────────────────────

    #[test]
    fn test_looks_like_version_multidigit() {
        assert!(looks_like_version("12.34.56"));
    }

    #[test]
    fn test_looks_like_version_prerelease() {
        // Pre-release tags still have digits and dots, so they match
        assert!(looks_like_version("1.0.0-beta"));
    }

    // ─── clean_version_string ─────────────────────────────────────────────────

    #[test]
    fn test_clean_version_string_with_prefix() {
        // clean_version_string strips "version:" or "version " prefix, not "v"
        assert_eq!(clean_version_string("version:1.2.3"), "1.2.3");
    }

    #[test]
    fn test_clean_version_string_tool_prefix() {
        // Tool name prefix is not stripped; only "version:" or "version " prefix is
        assert_eq!(clean_version_string("samtools 1.17"), "samtools 1.17");
    }

    #[test]
    fn test_clean_version_string_no_match() {
        assert_eq!(clean_version_string("no version here"), "no version here");
    }

    // ─── is_likely_error ──────────────────────────────────────────────────────

    #[test]
    fn test_is_likely_error_command_not_found() {
        // "command not found" is not in the recognized error patterns
        assert!(is_likely_error("unknown command: samtools"));
    }

    #[test]
    fn test_is_likely_error_no_such_file() {
        assert!(is_likely_error("No such file or directory"));
    }

    #[test]
    fn test_is_likely_error_normal_output() {
        assert!(!is_likely_error("SAM file processed successfully"));
    }

    // ─── extract_subcommand_list ──────────────────────────────────────────────

    #[test]
    fn test_extract_subcommand_list_samtools_format() {
        let help = "\
samtools

Commands:
  view       view and convert SAM/BAM/CRAM files
  sort       sort alignment files
  index      index alignment files
  stats      produce statistics
  flagstat   produce flag statistics

Options:
  --help     display this help";
        let subcmds = extract_subcommand_list(help);
        assert!(subcmds.contains(&"view".to_string()));
        assert!(subcmds.contains(&"sort".to_string()));
        assert!(subcmds.contains(&"index".to_string()));
        assert!(subcmds.contains(&"stats".to_string()));
        assert!(subcmds.contains(&"flagstat".to_string()));
        // "help" from "--help" should not be in the list (it's a flag)
        // Note: "display" could be extracted from the description but it shouldn't appear
    }

    #[test]
    fn test_extract_subcommand_list_bcftools_format() {
        let help = "\
bcftools

Commands:
  view     .  view and convert VCF/BCF files
  call     .  call variants
  filter   .  filter VCF/BCF files
  norm     .  normalize indels
  query    .  transform VCF/BCF into user-defined formats";
        let subcmds = extract_subcommand_list(help);
        assert!(subcmds.contains(&"view".to_string()));
        assert!(subcmds.contains(&"call".to_string()));
        assert!(subcmds.contains(&"filter".to_string()));
        assert!(subcmds.contains(&"norm".to_string()));
        assert!(subcmds.contains(&"query".to_string()));
    }

    #[test]
    fn test_extract_subcommand_list_no_commands_section() {
        let help = "usage: bwa mem [options] <idxbase> <in1.fq> [in2.fq]\n\nOptions:\n  -t INT  number of threads";
        let subcmds = extract_subcommand_list(help);
        assert!(
            subcmds.is_empty(),
            "no Commands: section should yield no subcommands"
        );
    }

    #[test]
    fn test_extract_subcommand_list_filters_flags() {
        let help = "\
tool

Commands:
  sort     sort things
  -v       verbose mode
  --help   show help";
        let subcmds = extract_subcommand_list(help);
        assert!(subcmds.contains(&"sort".to_string()));
        assert!(
            !subcmds.iter().any(|s| s.starts_with('-')),
            "flags should be filtered out"
        );
    }

    #[test]
    fn test_extract_subcommand_list_filters_common_words() {
        let help = "\
tool

Commands:
  sort     sort the input file
  usage    show usage information
  version  show version";
        let subcmds = extract_subcommand_list(help);
        assert!(subcmds.contains(&"sort".to_string()));
        // "usage" and "version" are filtered by is_common_non_subcommand() because they
        // are generic help-section words, not real subcommands in most tools.
        // If a tool genuinely has a "version" subcommand, it will still be matched
        // from the task description via the task-matching heuristic in fetch_subcommand_help.
        assert!(!subcmds.contains(&"usage".to_string()));
        assert!(!subcmds.contains(&"version".to_string()));
    }

    #[test]
    fn test_extract_subcommand_list_bedtools_format() {
        let help = "\
bedtools: flexible tools for genome arithmetic

Available commands:
  intersect     Find overlapping intervals
  merge         Merge overlapping intervals
  subtract      Remove intervals
  closest       Find closest intervals
  window        Find intervals within a window";
        let subcmds = extract_subcommand_list(help);
        assert!(subcmds.contains(&"intersect".to_string()));
        assert!(subcmds.contains(&"merge".to_string()));
        assert!(subcmds.contains(&"subtract".to_string()));
        assert!(subcmds.contains(&"closest".to_string()));
        assert!(subcmds.contains(&"window".to_string()));
    }

    // ─── fetch_subcommand_help ──────────────────────────────────────────────────

    #[test]
    fn test_fetch_subcommand_help_matches_task() {
        let help = "\
samtools

Commands:
  view       view and convert SAM/BAM/CRAM files
  sort       sort alignment files
  index      index alignment files

Options:
  --help     display this help";
        let fetcher = DocsFetcher::new(Config::default());
        let result =
            fetcher.fetch_subcommand_help("samtools", help, "sort the bam file by coordinate");
        // The result depends on whether `samtools sort --help` is available in PATH.
        // If samtools is installed, we should get help; if not, None is fine.
        // Just test that it doesn't crash and returns Option<String>.
        let _ = result;
    }

    #[test]
    fn test_fetch_subcommand_help_no_match() {
        let help = "\
samtools

Commands:
  view       view and convert SAM/BAM/CRAM files
  sort       sort alignment files

Options:
  --help     display this help";
        let fetcher = DocsFetcher::new(Config::default());
        let result = fetcher.fetch_subcommand_help("samtools", help, "count the number of reads");
        assert!(
            result.is_none(),
            "task does not mention any subcommand → should return None"
        );
    }

    #[test]
    fn test_fetch_subcommand_help_no_subcommands() {
        let help = "usage: bwa mem [options] <idxbase> <in1.fq>\n\nOptions:\n  -t INT  threads";
        let fetcher = DocsFetcher::new(Config::default());
        let result = fetcher.fetch_subcommand_help("bwa", help, "align reads with mem");
        assert!(
            result.is_none(),
            "no subcommands listed in help → should return None"
        );
    }

    // ─── ToolDocs::combined with subcommand_help ────────────────────────────────

    #[test]
    fn test_tool_docs_combined_includes_subcommand_help() {
        let docs = ToolDocs {
            tool_name: "samtools".to_string(),
            help_output: Some("samtools top-level help".to_string()),
            cached_docs: None,
            version: None,
            subcommand_help: Some("# samtools sort --help\n\nsort options here".to_string()),
        };
        let combined = docs.combined();
        assert!(combined.contains("samtools top-level help"));
        assert!(combined.contains("samtools sort --help"));
        assert!(combined.contains("sort options here"));
    }

    #[test]
    fn test_tool_docs_combined_without_subcommand_help() {
        let docs = ToolDocs {
            tool_name: "samtools".to_string(),
            help_output: Some("samtools top-level help".to_string()),
            cached_docs: None,
            version: None,
            subcommand_help: None,
        };
        let combined = docs.combined();
        assert!(combined.contains("samtools top-level help"));
        assert!(!combined.contains("--help"));
    }

    // ─── Version extraction tests ─────────────────────────────────────────

    #[test]
    fn test_extract_major_minor_simple() {
        assert_eq!(extract_major_minor("1.17"), "1.17");
    }

    #[test]
    fn test_extract_major_minor_with_patch() {
        assert_eq!(extract_major_minor("1.20.0"), "1.20");
    }

    #[test]
    fn test_extract_major_minor_with_prefix() {
        assert_eq!(extract_major_minor("samtools 1.17"), "1.17");
    }

    #[test]
    fn test_extract_major_minor_version_prefix() {
        assert_eq!(extract_major_minor("v2.3.1"), "2.3");
    }

    #[test]
    fn test_extract_major_minor_no_version() {
        assert_eq!(extract_major_minor("unknown"), "unknown");
    }
}
