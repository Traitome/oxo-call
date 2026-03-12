use crate::config::Config;
use crate::error::{OxoError, Result};
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

// Minimum useful help text length – anything shorter than this is likely an error message
const MIN_HELP_LEN: usize = 80;

// Maximum chars to store per help section to keep LLM prompts reasonable
const MAX_HELP_LEN: usize = 16_000;

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
}

impl ToolDocs {
    /// Return the best available documentation, preferring cached full docs but always
    /// including fresh `--help` output when available.
    pub fn combined(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(version) = &self.version {
            parts.push(format!("Version: {version}"));
        }

        // Prefer cached docs (they may contain more detail from remote sources),
        // but always append fresh help so the LLM sees current flags too.
        if let Some(cached) = &self.cached_docs {
            parts.push(cached.clone());
        }
        if let Some(help) = &self.help_output {
            // Only add if we don't already have it embedded in cached docs
            if self.cached_docs.is_none()
                || !self
                    .cached_docs
                    .as_deref()
                    .unwrap_or("")
                    .contains(help.as_str())
            {
                parts.push(help.clone());
            }
        }

        parts.join("\n\n")
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
        validate_tool_name(tool)?;

        let mut docs = ToolDocs {
            tool_name: tool.to_string(),
            help_output: None,
            cached_docs: None,
            version: None,
        };

        // 1. Try local cache first
        if let Ok(cached) = self.load_cache(tool) {
            docs.cached_docs = Some(cached);
        }

        // 2. Try to get help text and version from the live tool
        match self.fetch_help(tool) {
            Ok((help, version)) => {
                docs.help_output = Some(help);
                docs.version = version;
            }
            Err(_) => {
                // Tool may not be in PATH - that's okay if we have cached docs
            }
        }

        // 3. Try local documentation paths from config
        if docs.cached_docs.is_none()
            && let Some(local_doc) = self.search_local_docs(tool)
        {
            docs.cached_docs = Some(local_doc);
        }

        if docs.is_empty() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                "No documentation found. Try 'oxo-call index add <tool>' to build the index, or ensure the tool is installed.".to_string(),
            ));
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            let output = Command::new(tool)
                .arg(flag)
                .output()
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

            extract_useful_output(tool, &output.stdout, &output.stderr)
        }
        #[cfg(target_arch = "wasm32")]
        Err(OxoError::ToolNotFound(format!(
            "{tool}: process execution is not supported in WebAssembly"
        )))
    }

    /// Run the tool with no arguments – many bioinformatics tools (bwa, samtools, etc.)
    /// print their usage/help when called without any arguments.
    fn run_no_args(&self, tool: &str) -> Result<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let output = Command::new(tool)
                .output()
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

            extract_useful_output(tool, &output.stdout, &output.stderr)
        }
        #[cfg(target_arch = "wasm32")]
        Err(OxoError::ToolNotFound(format!(
            "{tool}: process execution is not supported in WebAssembly"
        )))
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
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&cache_path, content)?;
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
                if let Ok(canonical_base) = base_path.canonicalize()
                    && let Ok(canonical_candidate) = candidate.canonicalize()
                    && !canonical_candidate.starts_with(&canonical_base)
                {
                    continue;
                }
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
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            "Remote documentation fetching is not supported in WebAssembly".to_string(),
        ));
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    if trimmed.len() < MIN_HELP_LEN && is_likely_error(trimmed) {
        return Err(OxoError::DocFetchError(
            tool.to_string(),
            format!("Output looks like an error rather than help text: {trimmed}"),
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

/// Check whether a short string looks like a command-line error message rather
/// than useful help text.
fn is_likely_error(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("unrecognized command")
        || lower.contains("unknown command")
        || lower.contains("invalid option")
        || lower.contains("unrecognized option")
        || lower.contains("no such")
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
