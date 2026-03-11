use crate::config::Config;
use crate::error::{OxoError, Result};
use std::path::PathBuf;
use std::process::Command;

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
            "Tool name contains invalid characters (allowed: alphanumeric, '-', '_', '.')".to_string(),
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
    /// Return the best available documentation, preferring cached full docs
    pub fn combined(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(version) = &self.version {
            parts.push(format!("Version: {version}"));
        }

        if let Some(cached) = &self.cached_docs {
            parts.push(cached.clone());
        } else if let Some(help) = &self.help_output {
            parts.push(help.clone());
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

        // 2. Try --help / -h (always refresh from live tool)
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
            && let Some(local_doc) = self.search_local_docs(tool) {
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

    /// Fetch --help / -h output from a tool
    fn fetch_help(&self, tool: &str) -> Result<(String, Option<String>)> {
        // Try --help first
        let help = self
            .run_help_flag(tool, "--help")
            .or_else(|_| self.run_help_flag(tool, "-h"))
            .or_else(|_| self.run_help_flag(tool, "help"))
            .map_err(|_| {
                OxoError::DocFetchError(
                    tool.to_string(),
                    "Tool not found or does not support --help/-h".to_string(),
                )
            })?;

        // Try to get version
        let version = self
            .run_help_flag(tool, "--version")
            .or_else(|_| self.run_help_flag(tool, "-V"))
            .ok()
            .map(|v| v.lines().next().unwrap_or("").trim().to_string())
            .filter(|v| !v.is_empty());

        Ok((help, version))
    }

    fn run_help_flag(&self, tool: &str, flag: &str) -> Result<String> {
        let output = Command::new(tool)
            .arg(flag)
            .output()
            .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

        // Many tools print help to stderr
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined = if stdout.len() >= stderr.len() {
            stdout
        } else {
            stderr
        };

        if combined.trim().is_empty() {
            return Err(OxoError::DocFetchError(
                tool.to_string(),
                "Empty help output".to_string(),
            ));
        }

        // Limit to 8000 chars to avoid overly long prompts
        let truncated = if combined.len() > 8000 {
            format!("{}\n...[truncated]", &combined[..8000])
        } else {
            combined
        };

        Ok(truncated)
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
                && let Some(stem) = path.file_stem() {
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
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
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
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
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
                        && !canonical_candidate.starts_with(&canonical_base) {
                            continue;
                        }
                if candidate.exists()
                    && let Ok(content) = std::fs::read_to_string(candidate) {
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
        let truncated = if content.len() > 50000 {
            format!("{}\n...[truncated]", &content[..50000])
        } else {
            content
        };
        Ok(truncated)
    }
}
