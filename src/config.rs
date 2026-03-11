use crate::error::{OxoError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_LLM_PROVIDER: &str = "github-copilot";
const DEFAULT_MAX_TOKENS: u32 = 2048;
const DEFAULT_TEMPERATURE: f32 = 0.2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub docs: DocsConfig,
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
        }
    }
}

impl Config {
    pub fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("io", "traitome", "oxo-call")
    }

    pub fn config_path() -> Result<PathBuf> {
        let dirs = Self::project_dirs()
            .ok_or_else(|| OxoError::ConfigError("Cannot determine config directory".to_string()))?;
        Ok(dirs.config_dir().join("config.toml"))
    }

    pub fn data_dir() -> Result<PathBuf> {
        let dirs = Self::project_dirs()
            .ok_or_else(|| OxoError::ConfigError("Cannot determine data directory".to_string()))?;
        Ok(dirs.data_dir().to_path_buf())
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
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
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
        match key {
            "llm.provider" => Ok(self.llm.provider.clone()),
            "llm.api_token" => Ok(self.llm.api_token.clone().unwrap_or_default()),
            "llm.api_base" => Ok(self.llm.api_base.clone().unwrap_or_default()),
            "llm.model" => Ok(self.llm.model.clone().unwrap_or_default()),
            "llm.max_tokens" => Ok(self.llm.max_tokens.to_string()),
            "llm.temperature" => Ok(self.llm.temperature.to_string()),
            "docs.auto_update" => Ok(self.docs.auto_update.to_string()),
            _ => Err(OxoError::ConfigError(format!(
                "Unknown config key: {key}"
            ))),
        }
    }

    /// Resolve the effective API token from config or environment variables
    pub fn effective_api_token(&self) -> Option<String> {
        if let Some(token) = &self.llm.api_token
            && !token.is_empty() {
                return Some(token.clone());
            }
        // Fallback to environment variables
        match self.llm.provider.as_str() {
            "github-copilot" => std::env::var("GITHUB_TOKEN")
                .or_else(|_| std::env::var("GH_TOKEN"))
                .ok(),
            "openai" => std::env::var("OPENAI_API_KEY").ok(),
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
            _ => std::env::var("OXO_API_TOKEN").ok(),
        }
    }

    /// Resolve the effective API base URL for the current provider
    pub fn effective_api_base(&self) -> String {
        if let Some(base) = &self.llm.api_base
            && !base.is_empty() {
                return base.clone();
            }
        match self.llm.provider.as_str() {
            "github-copilot" => "https://api.githubcopilot.com".to_string(),
            "openai" => "https://api.openai.com/v1".to_string(),
            "anthropic" => "https://api.anthropic.com/v1".to_string(),
            "ollama" => "http://localhost:11434/v1".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        }
    }

    /// Resolve the effective model name for the current provider
    pub fn effective_model(&self) -> String {
        if let Some(model) = &self.llm.model
            && !model.is_empty() {
                return model.clone();
            }
        match self.llm.provider.as_str() {
            "github-copilot" => "gpt-4o".to_string(),
            "openai" => "gpt-4o".to_string(),
            "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
            "ollama" => "llama3.2".to_string(),
            _ => "gpt-4o".to_string(),
        }
    }
}
