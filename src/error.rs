use thiserror::Error;

#[derive(Debug, Error)]
pub enum OxoError {
    #[error("Tool '{0}' not found in PATH")]
    ToolNotFound(String),

    #[error("Failed to fetch documentation for '{0}': {1}")]
    DocFetchError(String, String),

    #[error("LLM request failed: {0}")]
    LlmError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[allow(dead_code)]
    #[error("Command execution failed: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSeError(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, OxoError>;
