use crate::error::{OxoError, Result};
use crate::server::ServerConfig;
#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

const DEFAULT_LLM_PROVIDER: &str = "github-copilot";
const DEFAULT_MAX_TOKENS: u32 = 2048;
const DEFAULT_TEMPERATURE: f32 = 0.0;
const ENV_LLM_PROVIDER: &str = "OXO_CALL_LLM_PROVIDER";
const ENV_LLM_API_TOKEN: &str = "OXO_CALL_LLM_API_TOKEN";
const ENV_LLM_API_BASE: &str = "OXO_CALL_LLM_API_BASE";
const ENV_LLM_MODEL: &str = "OXO_CALL_LLM_MODEL";
const ENV_LLM_MAX_TOKENS: &str = "OXO_CALL_LLM_MAX_TOKENS";
const ENV_LLM_TEMPERATURE: &str = "OXO_CALL_LLM_TEMPERATURE";
const ENV_DOCS_AUTO_UPDATE: &str = "OXO_CALL_DOCS_AUTO_UPDATE";

// ─── MCP configuration ────────────────────────────────────────────────────────

/// Configuration for a single MCP skill provider server.
///
/// Register MCP servers in `~/.config/oxo-call/config.toml`:
///
/// ```toml
/// [[mcp.servers]]
/// url  = "http://localhost:3000"
/// name = "local-skills"
///
/// [[mcp.servers]]
/// url     = "https://skills.example.org"
/// name    = "org-skills"
/// api_key = "secret-token"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Base URL of the MCP server (e.g. `http://localhost:3000`).
    pub url: String,
    /// Human-readable label shown in `skill list` and `skill mcp list`.
    /// Defaults to the URL's hostname when not set.
    #[serde(default)]
    pub name: String,
    /// Optional Bearer token sent as `Authorization: Bearer <api_key>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

impl McpServerConfig {
    /// Returns the display name, falling back to the URL if name is empty.
    pub fn name(&self) -> &str {
        if self.name.is_empty() {
            &self.url
        } else {
            &self.name
        }
    }
}

/// Aggregated MCP configuration (list of skill provider servers).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// Registered MCP skill servers, queried in order after community skills
    /// and before built-in skills.
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

// ─── Main config ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub docs: DocsConfig,
    #[serde(default)]
    pub license: LicenseConfig,
    /// MCP skill provider configuration.
    #[serde(default)]
    pub mcp: McpConfig,
    /// Remote server management configuration.
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LLM provider: "github-copilot", "openai", "anthropic", "ollama"
    pub provider: String,
    /// API token (for GitHub Copilot, use your GitHub token with copilot scope)
    pub api_token: Option<String>,
    /// API base URL (override for local/custom endpoints)
    pub api_base: Option<String>,
    /// Model name (e.g. "gpt-4o", "claude-3-5-sonnet-20241022", "gemma2")
    pub model: Option<String>,
    /// Max tokens to generate
    pub max_tokens: u32,
    /// Temperature for generation
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    /// Paths to local documentation directories
    pub local_paths: Vec<PathBuf>,
    /// Remote documentation sources (URL templates, {tool} replaced with tool name)
    pub remote_sources: Vec<String>,
    /// Whether to auto-update docs cache on first use
    pub auto_update: bool,
}

/// License configuration — kept for backward-compatible TOML deserialization.
/// License validation is now file-based (see `src/license.rs`).
/// Unknown TOML keys from older config files (e.g. `license_key`) are silently ignored.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseConfig {
    /// Whether the first-run notice has been shown (no longer displayed).
    #[serde(default)]
    pub notice_shown: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            llm: LlmConfig {
                provider: DEFAULT_LLM_PROVIDER.to_string(),
                api_token: None,
                api_base: None,
                model: None,
                max_tokens: DEFAULT_MAX_TOKENS,
                temperature: DEFAULT_TEMPERATURE,
            },
            docs: DocsConfig {
                local_paths: Vec::new(),
                remote_sources: Vec::new(),
                auto_update: true,
            },
            license: LicenseConfig::default(),
            mcp: McpConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Config {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("io", "traitome", "oxo-call")
    }

    pub fn config_dir() -> Result<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dirs = Self::project_dirs().ok_or_else(|| {
                OxoError::ConfigError("Cannot determine config directory".to_string())
            })?;
            Ok(dirs.config_dir().to_path_buf())
        }
        #[cfg(target_arch = "wasm32")]
        Ok(PathBuf::from("/config/oxo-call"))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn data_dir() -> Result<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(override_dir) = std::env::var("OXO_CALL_DATA_DIR") {
                return Ok(PathBuf::from(override_dir));
            }
            let dirs = Self::project_dirs().ok_or_else(|| {
                OxoError::ConfigError("Cannot determine data directory".to_string())
            })?;
            Ok(dirs.data_dir().to_path_buf())
        }
        #[cfg(target_arch = "wasm32")]
        Ok(PathBuf::from("/data/oxo-call"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let dir = path.parent().ok_or_else(|| {
            OxoError::ConfigError("Config path has no parent directory".to_string())
        })?;
        std::fs::create_dir_all(dir)?;
        let content = toml::to_string_pretty(self)?;
        // Write to a sibling temp file first, then atomically rename into place.
        // This prevents concurrent readers from observing a half-written config.
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, &content)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "llm.provider" => self.llm.provider = value.to_string(),
            "llm.api_token" => self.llm.api_token = Some(value.to_string()),
            "llm.api_base" => self.llm.api_base = Some(value.to_string()),
            "llm.model" => self.llm.model = Some(value.to_string()),
            "llm.max_tokens" => {
                self.llm.max_tokens = value.parse().map_err(|_| {
                    OxoError::ConfigError(format!("Invalid max_tokens value: {value}"))
                })?
            }
            "llm.temperature" => {
                self.llm.temperature = value.parse().map_err(|_| {
                    OxoError::ConfigError(format!("Invalid temperature value: {value}"))
                })?
            }
            "docs.auto_update" => {
                self.docs.auto_update = value.parse().map_err(|_| {
                    OxoError::ConfigError(format!("Invalid auto_update value: {value}"))
                })?
            }
            _ => {
                return Err(OxoError::ConfigError(format!(
                    "Unknown config key: {key}. Valid keys: llm.provider, llm.api_token, llm.api_base, llm.model, llm.max_tokens, llm.temperature, docs.auto_update"
                )));
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<String> {
        self.effective_value(key)
    }

    fn env_string(name: &str) -> Option<String> {
        std::env::var(name).ok().filter(|value| !value.is_empty())
    }

    fn env_parse<T>(name: &str, key: &str) -> Result<Option<T>>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match Self::env_string(name) {
            Some(value) => value.parse::<T>().map(Some).map_err(|e| {
                OxoError::ConfigError(format!(
                    "Invalid value in environment variable {name} for {key}: {e}"
                ))
            }),
            None => Ok(None),
        }
    }

    pub fn effective_provider(&self) -> String {
        Self::env_string(ENV_LLM_PROVIDER).unwrap_or_else(|| self.llm.provider.clone())
    }

    pub fn effective_api_token(&self) -> Option<String> {
        if let Some(token) = Self::env_string(ENV_LLM_API_TOKEN) {
            return Some(token);
        }
        // Backward-compatible provider-specific fallbacks
        let legacy_env_token = match self.effective_provider().as_str() {
            "github-copilot" => std::env::var("GITHUB_TOKEN")
                .or_else(|_| std::env::var("GH_TOKEN"))
                .ok(),
            "openai" => std::env::var("OPENAI_API_KEY").ok(),
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
            _ => std::env::var("OXO_API_TOKEN").ok(),
        };
        if legacy_env_token.is_some() {
            return legacy_env_token;
        }
        if let Some(token) = &self.llm.api_token
            && !token.is_empty()
        {
            return Some(token.clone());
        }
        None
    }

    /// Resolve the effective API base URL for the current provider
    pub fn effective_api_base(&self) -> String {
        if let Some(base) = Self::env_string(ENV_LLM_API_BASE) {
            return base;
        }
        if let Some(base) = &self.llm.api_base
            && !base.is_empty()
        {
            return base.clone();
        }
        match self.effective_provider().as_str() {
            "github-copilot" => "https://api.githubcopilot.com".to_string(),
            "openai" => "https://api.openai.com/v1".to_string(),
            "anthropic" => "https://api.anthropic.com/v1".to_string(),
            "ollama" => "http://localhost:11434/v1".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        }
    }

    /// Resolve the effective model name for the current provider
    pub fn effective_model(&self) -> String {
        if let Some(model) = Self::env_string(ENV_LLM_MODEL) {
            return model;
        }
        if let Some(model) = &self.llm.model
            && !model.is_empty()
        {
            return model.clone();
        }
        match self.effective_provider().as_str() {
            "github-copilot" => "gpt-4o".to_string(),
            "openai" => "gpt-4o".to_string(),
            "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
            "ollama" => "llama3.2".to_string(),
            _ => "gpt-4o".to_string(),
        }
    }

    pub fn effective_max_tokens(&self) -> Result<u32> {
        Ok(Self::env_parse(ENV_LLM_MAX_TOKENS, "llm.max_tokens")?.unwrap_or(self.llm.max_tokens))
    }

    pub fn effective_temperature(&self) -> Result<f32> {
        Ok(
            Self::env_parse(ENV_LLM_TEMPERATURE, "llm.temperature")?
                .unwrap_or(self.llm.temperature),
        )
    }

    pub fn effective_docs_auto_update(&self) -> Result<bool> {
        Ok(Self::env_parse(ENV_DOCS_AUTO_UPDATE, "docs.auto_update")?
            .unwrap_or(self.docs.auto_update))
    }

    pub fn effective_value(&self, key: &str) -> Result<String> {
        match key {
            "llm.provider" => Ok(self.effective_provider()),
            "llm.api_token" => Ok(self.effective_api_token().unwrap_or_default()),
            "llm.api_base" => Ok(self.effective_api_base()),
            "llm.model" => Ok(self.effective_model()),
            "llm.max_tokens" => Ok(self.effective_max_tokens()?.to_string()),
            "llm.temperature" => Ok(self.effective_temperature()?.to_string()),
            "docs.auto_update" => Ok(self.effective_docs_auto_update()?.to_string()),
            _ => Err(OxoError::ConfigError(format!("Unknown config key: {key}"))),
        }
    }

    pub fn effective_source(&self, key: &str) -> Result<String> {
        match key {
            "llm.provider" => {
                if Self::env_string(ENV_LLM_PROVIDER).is_some() {
                    Ok(format!("env:{ENV_LLM_PROVIDER}"))
                } else {
                    Ok("stored config/default".to_string())
                }
            }
            "llm.api_token" => {
                if Self::env_string(ENV_LLM_API_TOKEN).is_some() {
                    return Ok(format!("env:{ENV_LLM_API_TOKEN}"));
                }
                let provider = self.effective_provider();
                let legacy_env = match provider.as_str() {
                    "github-copilot" => std::env::var("GITHUB_TOKEN")
                        .ok()
                        .map(|_| "GITHUB_TOKEN")
                        .or_else(|| std::env::var("GH_TOKEN").ok().map(|_| "GH_TOKEN")),
                    "openai" => std::env::var("OPENAI_API_KEY")
                        .ok()
                        .map(|_| "OPENAI_API_KEY"),
                    "anthropic" => std::env::var("ANTHROPIC_API_KEY")
                        .ok()
                        .map(|_| "ANTHROPIC_API_KEY"),
                    _ => std::env::var("OXO_API_TOKEN").ok().map(|_| "OXO_API_TOKEN"),
                };
                if let Some(name) = legacy_env {
                    Ok(format!("env:{name}"))
                } else if self
                    .llm
                    .api_token
                    .as_deref()
                    .is_some_and(|token| !token.is_empty())
                {
                    Ok("stored config".to_string())
                } else {
                    Ok("unset".to_string())
                }
            }
            "llm.api_base" => {
                if Self::env_string(ENV_LLM_API_BASE).is_some() {
                    Ok(format!("env:{ENV_LLM_API_BASE}"))
                } else if self
                    .llm
                    .api_base
                    .as_deref()
                    .is_some_and(|base| !base.is_empty())
                {
                    Ok("stored config".to_string())
                } else {
                    Ok("provider default".to_string())
                }
            }
            "llm.model" => {
                if Self::env_string(ENV_LLM_MODEL).is_some() {
                    Ok(format!("env:{ENV_LLM_MODEL}"))
                } else if self
                    .llm
                    .model
                    .as_deref()
                    .is_some_and(|model| !model.is_empty())
                {
                    Ok("stored config".to_string())
                } else {
                    Ok("provider default".to_string())
                }
            }
            "llm.max_tokens" => {
                if Self::env_string(ENV_LLM_MAX_TOKENS).is_some() {
                    Ok(format!("env:{ENV_LLM_MAX_TOKENS}"))
                } else {
                    Ok("stored config/default".to_string())
                }
            }
            "llm.temperature" => {
                if Self::env_string(ENV_LLM_TEMPERATURE).is_some() {
                    Ok(format!("env:{ENV_LLM_TEMPERATURE}"))
                } else {
                    Ok("stored config/default".to_string())
                }
            }
            "docs.auto_update" => {
                if Self::env_string(ENV_DOCS_AUTO_UPDATE).is_some() {
                    Ok(format!("env:{ENV_DOCS_AUTO_UPDATE}"))
                } else {
                    Ok("stored config/default".to_string())
                }
            }
            _ => Err(OxoError::ConfigError(format!("Unknown config key: {key}"))),
        }
    }
}
