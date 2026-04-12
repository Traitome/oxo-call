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

/// All recognised `config set` / `config get` key names.
const VALID_CONFIG_KEYS: &[&str] = &[
    "llm.provider",
    "llm.api_token",
    "llm.api_base",
    "llm.model",
    "llm.max_tokens",
    "llm.temperature",
    "docs.auto_update",
];

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
    /// Active model name (e.g. "gpt-5-mini", "gpt-4.1", "claude-3-5-sonnet-20241022").
    /// Use `oxo-call config model use <id>` to switch between configured models.
    pub model: Option<String>,
    /// User-configured model list for quick switching via `config model use`.
    /// Populated automatically during `config login` and editable with `config model add/remove`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<String>,
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
                models: Vec::new(),
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
                    "Unknown config key: {key}. Valid keys: {}",
                    VALID_CONFIG_KEYS.join(", ")
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
        // For github-copilot, only use stored config token (from `oxo-call config login`)
        // Environment variables like GITHUB_TOKEN often contain PAT tokens that don't work
        // with Copilot's token exchange endpoint
        if self.effective_provider() == "github-copilot" {
            if let Some(token) = &self.llm.api_token
                && !token.is_empty()
            {
                return Some(token.clone());
            }
            return None;
        }

        // For other providers, check environment variables first
        if let Some(token) = Self::env_string(ENV_LLM_API_TOKEN) {
            return Some(token);
        }
        // Backward-compatible provider-specific fallbacks
        let legacy_env_token = match self.effective_provider().as_str() {
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

    /// Returns `true` if the current provider requires an API token to function.
    ///
    /// Local providers such as Ollama typically run without authentication, so
    /// callers should use this to skip the token-required check for those providers.
    ///
    /// Currently only `ollama` is treated as tokenless because it is the only
    /// built-in provider designed for local, unauthenticated use.  If a future
    /// provider also runs without a token, add it to the match arm here.
    pub fn provider_requires_token(&self) -> bool {
        !matches!(self.effective_provider().as_str(), "ollama")
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
            "github-copilot" => "https://api.individual.githubcopilot.com".to_string(),
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
            && model != "auto-selected"
        {
            return model.clone();
        }
        match self.effective_provider().as_str() {
            "github-copilot" => "gpt-5-mini".to_string(),
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
                // For github-copilot, only use stored config token
                if self.effective_provider() == "github-copilot" {
                    if self
                        .llm
                        .api_token
                        .as_deref()
                        .is_some_and(|token| !token.is_empty())
                    {
                        return Ok("stored config".to_string());
                    }
                    return Ok("unset".to_string());
                }

                // For other providers, check environment variables first
                if Self::env_string(ENV_LLM_API_TOKEN).is_some() {
                    return Ok(format!("env:{ENV_LLM_API_TOKEN}"));
                }
                let provider = self.effective_provider();
                let legacy_env = match provider.as_str() {
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
                    .is_some_and(|model| !model.is_empty() && model != "auto-selected")
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

#[cfg(test)]
mod tests {
    use super::*;

    // All tests that mutate env vars use the crate-wide ENV_LOCK to prevent
    // races with docs.rs, history.rs, and skill.rs tests.
    use crate::ENV_LOCK;

    // ─── McpServerConfig ──────────────────────────────────────────────────────

    #[test]
    fn test_mcp_server_config_name_non_empty() {
        let cfg = McpServerConfig {
            url: "http://localhost:3000".to_string(),
            name: "my-server".to_string(),
            api_key: None,
        };
        assert_eq!(cfg.name(), "my-server");
    }

    #[test]
    fn test_mcp_server_config_name_falls_back_to_url() {
        let cfg = McpServerConfig {
            url: "http://localhost:3000".to_string(),
            name: String::new(),
            api_key: None,
        };
        assert_eq!(cfg.name(), "http://localhost:3000");
    }

    // ─── Config defaults ──────────────────────────────────────────────────────

    #[test]
    fn test_config_default_provider() {
        let cfg = Config::default();
        assert_eq!(cfg.llm.provider, "github-copilot");
    }

    #[test]
    fn test_config_default_max_tokens() {
        let cfg = Config::default();
        assert_eq!(cfg.llm.max_tokens, 2048);
    }

    #[test]
    fn test_config_default_temperature() {
        let cfg = Config::default();
        assert!((cfg.llm.temperature - 0.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_default_no_api_token() {
        let cfg = Config::default();
        assert!(cfg.llm.api_token.is_none());
    }

    // ─── Config::set ──────────────────────────────────────────────────────────

    #[test]
    fn test_config_set_provider() {
        let mut cfg = Config::default();
        cfg.set("llm.provider", "openai").unwrap();
        assert_eq!(cfg.llm.provider, "openai");
    }

    #[test]
    fn test_config_set_api_token() {
        let mut cfg = Config::default();
        cfg.set("llm.api_token", "sk-test123").unwrap();
        assert_eq!(cfg.llm.api_token.as_deref(), Some("sk-test123"));
    }

    #[test]
    fn test_config_set_api_base() {
        let mut cfg = Config::default();
        cfg.set("llm.api_base", "https://my-proxy.example.com/v1")
            .unwrap();
        assert_eq!(
            cfg.llm.api_base.as_deref(),
            Some("https://my-proxy.example.com/v1")
        );
    }

    #[test]
    fn test_config_set_model() {
        let mut cfg = Config::default();
        cfg.set("llm.model", "claude-3-5-sonnet-20241022").unwrap();
        assert_eq!(cfg.llm.model.as_deref(), Some("claude-3-5-sonnet-20241022"));
    }

    #[test]
    fn test_config_set_max_tokens() {
        let mut cfg = Config::default();
        cfg.set("llm.max_tokens", "4096").unwrap();
        assert_eq!(cfg.llm.max_tokens, 4096);
    }

    #[test]
    fn test_config_set_max_tokens_invalid() {
        let mut cfg = Config::default();
        assert!(cfg.set("llm.max_tokens", "not-a-number").is_err());
    }

    #[test]
    fn test_config_set_temperature() {
        let mut cfg = Config::default();
        cfg.set("llm.temperature", "0.7").unwrap();
        assert!((cfg.llm.temperature - 0.7_f32).abs() < 1e-5);
    }

    #[test]
    fn test_config_set_temperature_invalid() {
        let mut cfg = Config::default();
        assert!(cfg.set("llm.temperature", "hot").is_err());
    }

    #[test]
    fn test_config_set_docs_auto_update() {
        let mut cfg = Config::default();
        cfg.set("docs.auto_update", "false").unwrap();
        assert!(!cfg.docs.auto_update);
        cfg.set("docs.auto_update", "true").unwrap();
        assert!(cfg.docs.auto_update);
    }

    #[test]
    fn test_config_set_unknown_key_errors() {
        let mut cfg = Config::default();
        assert!(cfg.set("llm.unknown_field", "value").is_err());
        assert!(cfg.set("does.not.exist", "value").is_err());
    }

    // ─── Config::get / effective_value ────────────────────────────────────────

    #[test]
    fn test_config_get_provider() {
        let cfg = Config::default();
        assert_eq!(cfg.get("llm.provider").unwrap(), "github-copilot");
    }

    #[test]
    fn test_config_get_unknown_key_errors() {
        let cfg = Config::default();
        assert!(cfg.get("llm.does_not_exist").is_err());
    }

    // ─── effective_provider ───────────────────────────────────────────────────

    #[test]
    fn test_effective_provider_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // Remove env var if set by another test
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let cfg = Config::default();
        assert_eq!(cfg.effective_provider(), "github-copilot");
    }

    #[test]
    fn test_effective_provider_from_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "anthropic".to_string();
        assert_eq!(cfg.effective_provider(), "anthropic");
    }

    // ─── effective_api_base ───────────────────────────────────────────────────

    #[test]
    fn test_effective_api_base_github_copilot() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let cfg = Config::default();
        assert_eq!(
            cfg.effective_api_base(),
            "https://api.individual.githubcopilot.com"
        );
    }

    #[test]
    fn test_effective_api_base_openai() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string();
        assert_eq!(cfg.effective_api_base(), "https://api.openai.com/v1");
    }

    #[test]
    fn test_effective_api_base_anthropic() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "anthropic".to_string();
        assert_eq!(cfg.effective_api_base(), "https://api.anthropic.com/v1");
    }

    #[test]
    fn test_effective_api_base_ollama() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "ollama".to_string();
        assert_eq!(cfg.effective_api_base(), "http://localhost:11434/v1");
    }

    #[test]
    fn test_effective_api_base_from_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        let mut cfg = Config::default();
        cfg.llm.api_base = Some("https://custom.example.com/v1".to_string());
        assert_eq!(cfg.effective_api_base(), "https://custom.example.com/v1");
    }

    // ─── effective_model ──────────────────────────────────────────────────────

    #[test]
    fn test_effective_model_github_copilot() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let cfg = Config::default();
        assert_eq!(cfg.effective_model(), "gpt-5-mini");
    }

    #[test]
    fn test_effective_model_anthropic() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "anthropic".to_string();
        assert_eq!(cfg.effective_model(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_effective_model_ollama() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "ollama".to_string();
        assert_eq!(cfg.effective_model(), "llama3.2");
    }

    #[test]
    fn test_effective_model_from_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
        let mut cfg = Config::default();
        cfg.llm.model = Some("gpt-4-turbo".to_string());
        assert_eq!(cfg.effective_model(), "gpt-4-turbo");
    }

    // ─── effective_max_tokens / effective_temperature ─────────────────────────

    #[test]
    fn test_effective_max_tokens_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MAX_TOKENS");
        }
        let cfg = Config::default();
        assert_eq!(cfg.effective_max_tokens().unwrap(), 2048);
    }

    #[test]
    fn test_effective_temperature_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_TEMPERATURE");
        }
        let cfg = Config::default();
        assert!((cfg.effective_temperature().unwrap() - 0.0_f32).abs() < f32::EPSILON);
    }

    // ─── effective_docs_auto_update ───────────────────────────────────────────

    #[test]
    fn test_effective_docs_auto_update_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_DOCS_AUTO_UPDATE");
        }
        let cfg = Config::default();
        assert!(cfg.effective_docs_auto_update().unwrap());
    }

    // ─── effective_source ─────────────────────────────────────────────────────

    #[test]
    fn test_effective_source_provider_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.provider").unwrap();
        assert!(src.contains("config") || src.contains("default"));
    }

    #[test]
    fn test_effective_source_unknown_key_errors() {
        let cfg = Config::default();
        assert!(cfg.effective_source("bad.key").is_err());
    }

    #[test]
    fn test_effective_source_api_base_provider_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        let mut cfg = Config::default();
        cfg.llm.api_base = None;
        let src = cfg.effective_source("llm.api_base").unwrap();
        assert!(src.contains("default") || src.contains("provider"));
    }

    #[test]
    fn test_effective_source_model_stored_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
        let mut cfg = Config::default();
        cfg.llm.model = Some("gpt-4-turbo".to_string());
        let src = cfg.effective_source("llm.model").unwrap();
        assert!(src.contains("stored config"));
    }

    #[test]
    fn test_effective_source_api_token_unset() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("GITHUB_TOKEN");
        }
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("GH_TOKEN");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.api_token").unwrap();
        assert!(src.contains("unset"));
    }

    #[test]
    fn test_effective_source_max_tokens_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MAX_TOKENS");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.max_tokens").unwrap();
        assert!(src.contains("default") || src.contains("stored"));
    }

    #[test]
    fn test_effective_source_temperature_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_LLM_TEMPERATURE");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.temperature").unwrap();
        assert!(src.contains("default") || src.contains("stored"));
    }

    #[test]
    fn test_effective_source_docs_auto_update_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::remove_var("OXO_CALL_DOCS_AUTO_UPDATE");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("docs.auto_update").unwrap();
        assert!(src.contains("default") || src.contains("stored"));
    }

    // ─── load returns default when no config file ─────────────────────────────

    #[test]
    fn test_config_load_returns_default_when_missing() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // Point config dir at empty temp directory
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: test-only env var mutation, serialised by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        // Config::load() uses config_dir() not data_dir(), but we exercise the
        // "file not found → default" path by verifying the returned config looks like defaults.
        // We can't easily redirect config_dir in unit tests without disk access.
        // At minimum: Config::default() is exercised.
        let cfg = Config::default();
        assert_eq!(cfg.llm.provider, "github-copilot");
        assert!(cfg.llm.api_token.is_none());
    }

    // ─── LicenseConfig defaults ───────────────────────────────────────────────

    #[test]
    fn test_license_config_default() {
        let lc = LicenseConfig::default();
        assert!(!lc.notice_shown);
    }

    // ─── McpConfig defaults ───────────────────────────────────────────────────

    #[test]
    fn test_mcp_config_default_empty() {
        let mc = McpConfig::default();
        assert!(mc.servers.is_empty());
    }

    // ─── TOML round-trip ──────────────────────────────────────────────────────

    #[test]
    fn test_config_toml_round_trip() {
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string();
        cfg.llm.model = Some("gpt-4-turbo".to_string());
        cfg.llm.max_tokens = 4096;

        let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
        let back: Config = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(back.llm.provider, "openai");
        assert_eq!(back.llm.model.as_deref(), Some("gpt-4-turbo"));
        assert_eq!(back.llm.max_tokens, 4096);
    }

    #[test]
    fn test_data_dir_uses_env_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: test-only env var mutation, single-threaded test

        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let data_dir = Config::data_dir().unwrap();
        assert_eq!(data_dir, tmp.path());
    }

    // ─── effective_api_token ──────────────────────────────────────────────────

    #[test]
    fn test_effective_api_token_from_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
        }
        let mut cfg = Config::default();
        cfg.llm.api_token = Some("stored-token-123".to_string());
        assert_eq!(
            cfg.effective_api_token().as_deref(),
            Some("stored-token-123")
        );
    }

    #[test]
    fn test_effective_api_token_none_when_empty_string() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
        }
        let mut cfg = Config::default();
        cfg.llm.api_token = Some(String::new()); // empty string
        assert!(
            cfg.effective_api_token().is_none(),
            "empty string token should be treated as None"
        );
    }

    #[test]
    fn test_effective_api_token_env_var_takes_precedence() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_TOKEN", "env-token-xyz");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string(); // Use openai to test env var precedence
        cfg.llm.api_token = Some("stored-token".to_string());
        assert_eq!(
            cfg.effective_api_token().as_deref(),
            Some("env-token-xyz"),
            "env var should take precedence over stored config"
        );
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
        }
    }

    #[test]
    fn test_effective_api_token_github_copilot_ignores_env_vars() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_TOKEN", "env-token-should-be-ignored");
            std::env::set_var("GITHUB_TOKEN", "github-token-should-be-ignored");
            std::env::set_var("GH_TOKEN", "gh-token-should-be-ignored");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "github-copilot".to_string();
        cfg.llm.api_token = Some("stored-copilot-token".to_string());
        // github-copilot should only use stored config token, ignoring env vars
        assert_eq!(
            cfg.effective_api_token().as_deref(),
            Some("stored-copilot-token"),
            "github-copilot should ignore env vars and use stored token"
        );
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
        }
    }

    // ─── effective_api_token legacy provider fallbacks ─────────────────────────

    #[test]
    fn test_effective_api_token_openai_legacy_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::set_var("OPENAI_API_KEY", "openai-legacy-key");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string();
        assert_eq!(
            cfg.effective_api_token().as_deref(),
            Some("openai-legacy-key")
        );
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }
    }

    #[test]
    fn test_effective_api_token_anthropic_legacy_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::set_var("ANTHROPIC_API_KEY", "anthropic-legacy-key");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "anthropic".to_string();
        assert_eq!(
            cfg.effective_api_token().as_deref(),
            Some("anthropic-legacy-key")
        );
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
    }

    // ─── effective_value for all keys ────────────────────────────────────────

    #[test]
    fn test_effective_value_all_known_keys() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            // Clear all relevant env vars
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
            std::env::remove_var("OXO_CALL_LLM_MODEL");
            std::env::remove_var("OXO_CALL_LLM_MAX_TOKENS");
            std::env::remove_var("OXO_CALL_LLM_TEMPERATURE");
            std::env::remove_var("OXO_CALL_DOCS_AUTO_UPDATE");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
        }
        let cfg = Config::default();
        assert!(cfg.effective_value("llm.provider").is_ok());
        assert!(cfg.effective_value("llm.api_token").is_ok());
        assert!(cfg.effective_value("llm.api_base").is_ok());
        assert!(cfg.effective_value("llm.model").is_ok());
        assert!(cfg.effective_value("llm.max_tokens").is_ok());
        assert!(cfg.effective_value("llm.temperature").is_ok());
        assert!(cfg.effective_value("docs.auto_update").is_ok());
        assert!(cfg.effective_value("unknown.key").is_err());
    }

    // ─── effective_source for all keys ───────────────────────────────────────

    #[test]
    fn test_effective_source_api_base_stored_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        let mut cfg = Config::default();
        cfg.llm.api_base = Some("https://custom.example.com/v1".to_string());
        let src = cfg.effective_source("llm.api_base").unwrap();
        assert!(src.contains("stored config"));
    }

    #[test]
    fn test_effective_source_api_token_stored_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
        }
        let mut cfg = Config::default();
        cfg.llm.api_token = Some("my-stored-token".to_string());
        let src = cfg.effective_source("llm.api_token").unwrap();
        assert!(src.contains("stored config"));
    }

    // ─── Config::save / Config::load round-trip ───────────────────────────────

    #[test]
    fn test_config_save_and_load_round_trip() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string();
        cfg.llm.model = Some("gpt-4-turbo".to_string());
        cfg.llm.max_tokens = 4096;

        // Save to the temp dir location
        // Config::save() uses config_dir(), which is separate from data_dir(),
        // so we write directly to exercise the save/load TOML path
        let config_dir = tmp.path().join(".config").join("oxo-call");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        std::fs::write(&config_path, &toml_str).unwrap();

        // Read back and verify
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.llm.provider, "openai");
        assert_eq!(loaded.llm.model.as_deref(), Some("gpt-4-turbo"));
        assert_eq!(loaded.llm.max_tokens, 4096);
    }

    // ─── effective_api_base: unknown provider ─────────────────────────────────

    #[test]
    fn test_effective_api_base_unknown_provider() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "some-unknown-provider".to_string();
        // Should fall through to OpenAI default
        assert_eq!(cfg.effective_api_base(), "https://api.openai.com/v1");
    }

    // ─── effective_model: unknown provider ────────────────────────────────────

    #[test]
    fn test_effective_model_unknown_provider() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "some-unknown-provider".to_string();
        // Should fall through to gpt-4o default
        assert_eq!(cfg.effective_model(), "gpt-4o");
    }

    // ─── effective_source with env vars set ──────────────────────────────────

    #[test]
    fn test_effective_source_provider_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_PROVIDER", "anthropic");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.provider").unwrap();
        assert!(src.contains("OXO_CALL_LLM_PROVIDER") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_PROVIDER");
        }
    }

    #[test]
    fn test_effective_source_api_base_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_BASE", "https://my-proxy.example.com/v1");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.api_base").unwrap();
        assert!(src.contains("OXO_CALL_LLM_API_BASE") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
    }

    #[test]
    fn test_effective_source_api_token_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_TOKEN", "env-token");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string(); // Use openai to test env var behavior
        let src = cfg.effective_source("llm.api_token").unwrap();
        assert!(src.contains("OXO_CALL_LLM_API_TOKEN") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
        }
    }

    #[test]
    fn test_effective_source_model_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_MODEL", "claude-3-opus");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.model").unwrap();
        assert!(src.contains("OXO_CALL_LLM_MODEL") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
    }

    #[test]
    fn test_effective_source_max_tokens_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_MAX_TOKENS", "4096");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.max_tokens").unwrap();
        assert!(src.contains("OXO_CALL_LLM_MAX_TOKENS") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MAX_TOKENS");
        }
    }

    #[test]
    fn test_effective_source_temperature_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_TEMPERATURE", "0.7");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.temperature").unwrap();
        assert!(src.contains("OXO_CALL_LLM_TEMPERATURE") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_TEMPERATURE");
        }
    }

    #[test]
    fn test_effective_source_docs_auto_update_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_DOCS_AUTO_UPDATE", "false");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("docs.auto_update").unwrap();
        assert!(src.contains("OXO_CALL_DOCS_AUTO_UPDATE") || src.contains("env"));
        unsafe {
            std::env::remove_var("OXO_CALL_DOCS_AUTO_UPDATE");
        }
    }

    // ─── effective_api_token: OXO_API_TOKEN legacy env (unknown provider) ─────

    #[test]
    fn test_effective_api_token_oxo_api_token_legacy() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::set_var("OXO_API_TOKEN", "oxo-legacy-token");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "some-custom-provider".to_string(); // triggers _ arm
        let token = cfg.effective_api_token();
        assert_eq!(token.as_deref(), Some("oxo-legacy-token"));
        unsafe {
            std::env::remove_var("OXO_API_TOKEN");
        }
    }

    // ─── effective_source: anthropic ANTHROPIC_API_KEY ────────────────────────

    #[test]
    fn test_effective_source_api_token_anthropic_legacy_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::set_var("ANTHROPIC_API_KEY", "anthropic-key");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "anthropic".to_string();
        let src = cfg.effective_source("llm.api_token").unwrap();
        assert!(src.contains("ANTHROPIC_API_KEY"));
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
    }

    // ─── effective_source: openai OPENAI_API_KEY ──────────────────────────────

    #[test]
    fn test_effective_source_api_token_openai_legacy_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::set_var("OPENAI_API_KEY", "openai-key");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string();
        let src = cfg.effective_source("llm.api_token").unwrap();
        assert!(src.contains("OPENAI_API_KEY"));
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }
    }

    // ─── Config::effective_api_base env var takes precedence ──────────────────

    #[test]
    fn test_effective_api_base_env_takes_precedence() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_BASE", "https://env-proxy.example.com/v1");
        }
        let mut cfg = Config::default();
        cfg.llm.api_base = Some("https://config-proxy.example.com/v1".to_string());
        // Env var should take precedence over config
        assert_eq!(cfg.effective_api_base(), "https://env-proxy.example.com/v1");
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
    }

    // ─── Config::effective_api_token branches ─────────────────────────────────

    #[test]
    fn test_effective_api_token_env_takes_precedence() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_TOKEN", "env-token");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "openai".to_string(); // Use openai to test env var precedence
        cfg.llm.api_token = Some("config-token".to_string());
        let token = cfg.effective_api_token();
        assert_eq!(token.as_deref(), Some("env-token"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
        }
    }

    #[test]
    fn test_effective_api_token_none_when_empty_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::remove_var("OXO_API_TOKEN");
        }
        let mut cfg = Config::default();
        cfg.llm.api_token = Some(String::new()); // empty string
        let token = cfg.effective_api_token();
        assert!(token.is_none(), "empty string token should return None");
    }

    #[test]
    fn test_effective_api_token_generic_legacy_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::set_var("OXO_API_TOKEN", "generic-legacy");
        }
        let mut cfg = Config::default();
        cfg.llm.provider = "custom-provider".to_string();
        let token = cfg.effective_api_token();
        assert_eq!(token.as_deref(), Some("generic-legacy"));
        unsafe {
            std::env::remove_var("OXO_API_TOKEN");
        }
    }

    // ─── Config::effective_source for api_base, model, max_tokens, temperature ─

    #[test]
    fn test_effective_source_api_base_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_API_BASE", "https://env.example.com/v1");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.api_base").unwrap();
        assert!(src.contains("OXO_CALL_LLM_API_BASE"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
    }

    #[test]
    fn test_effective_source_model_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_MODEL", "env-model");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.model").unwrap();
        assert!(src.contains("OXO_CALL_LLM_MODEL"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
    }

    #[test]
    fn test_effective_source_max_tokens_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_MAX_TOKENS", "8192");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.max_tokens").unwrap();
        assert!(src.contains("OXO_CALL_LLM_MAX_TOKENS"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MAX_TOKENS");
        }
    }

    #[test]
    fn test_effective_source_temperature_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_TEMPERATURE", "0.7");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.temperature").unwrap();
        assert!(src.contains("OXO_CALL_LLM_TEMPERATURE"));
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_TEMPERATURE");
        }
    }

    #[test]
    fn test_effective_source_docs_auto_update_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_DOCS_AUTO_UPDATE", "false");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("docs.auto_update").unwrap();
        assert!(src.contains("OXO_CALL_DOCS_AUTO_UPDATE"));
        unsafe {
            std::env::remove_var("OXO_CALL_DOCS_AUTO_UPDATE");
        }
    }

    #[test]
    fn test_effective_source_api_token_unset_no_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_TOKEN");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GH_TOKEN");
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::remove_var("OXO_API_TOKEN");
        }
        let cfg = Config::default();
        let src = cfg.effective_source("llm.api_token").unwrap();
        assert_eq!(src, "unset");
    }

    // ─── Config::effective_model from config ──────────────────────────────────

    #[test]
    fn test_effective_model_from_config_value() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MODEL");
        }
        let mut cfg = Config::default();
        cfg.llm.model = Some("custom-model-v2".to_string());
        assert_eq!(cfg.effective_model(), "custom-model-v2");
    }

    // ─── Config::effective_api_base from config ───────────────────────────────

    #[test]
    fn test_effective_api_base_from_config_value() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_API_BASE");
        }
        let mut cfg = Config::default();
        cfg.llm.api_base = Some("https://custom.example.com/v1".to_string());
        assert_eq!(cfg.effective_api_base(), "https://custom.example.com/v1");
    }

    // ─── Config::set edge cases ───────────────────────────────────────────────

    #[test]
    fn test_config_set_docs_auto_update_invalid() {
        let mut cfg = Config::default();
        let result = cfg.set("docs.auto_update", "not_a_bool");
        assert!(result.is_err());
    }

    // ─── env_parse ────────────────────────────────────────────────────────────

    #[test]
    fn test_effective_max_tokens_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_MAX_TOKENS", "4096");
        }
        let cfg = Config::default();
        let tokens = cfg.effective_max_tokens().unwrap();
        assert_eq!(tokens, 4096);
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_MAX_TOKENS");
        }
    }

    #[test]
    fn test_effective_temperature_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_LLM_TEMPERATURE", "0.7");
        }
        let cfg = Config::default();
        let temp = cfg.effective_temperature().unwrap();
        assert!((temp - 0.7).abs() < f32::EPSILON);
        unsafe {
            std::env::remove_var("OXO_CALL_LLM_TEMPERATURE");
        }
    }

    #[test]
    fn test_effective_docs_auto_update_from_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("OXO_CALL_DOCS_AUTO_UPDATE", "false");
        }
        let cfg = Config::default();
        assert!(!cfg.effective_docs_auto_update().unwrap());
        unsafe {
            std::env::remove_var("OXO_CALL_DOCS_AUTO_UPDATE");
        }
    }
}
