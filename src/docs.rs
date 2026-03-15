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
        };

        // 1. Try local cache first (unless skipping cache)
        if !skip_cache && let Ok(cached) = self.load_cache(tool) {
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

    /// Try to get help for a shell built-in command via `bash -c "help <tool>"`.
    /// This handles commands like `cd`, `export`, `alias`, etc. that are not
    /// standalone executables and cannot be invoked directly.
    fn run_shell_builtin_help(&self, tool: &str) -> Result<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
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
}
