use crate::config::Config;
use crate::error::{OxoError, Result};
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
use uuid::Uuid;

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
    /// including fresh `--help` output when available. Deduplicates content between sources.
    pub fn combined(&self) -> String {
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
                "No documentation found. Try 'oxo-call docs add <tool>' to build the index, or ensure the tool is installed.".to_string(),
            ));
        }

        // 4. Auto-cache: if we got live help output but have no cached docs yet,
        //    silently persist the help text so future calls are instant.
        if docs.cached_docs.is_none()
            && let Some(help) = &docs.help_output
        {
            let _ = self.save_cache(tool, help);
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
            cache_path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc"),
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
