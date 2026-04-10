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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_not_found_display() {
        let e = OxoError::ToolNotFound("samtools".to_string());
        assert_eq!(e.to_string(), "Tool 'samtools' not found in PATH");
    }

    #[test]
    fn test_doc_fetch_error_display() {
        let e = OxoError::DocFetchError("bwa".to_string(), "timeout".to_string());
        assert_eq!(
            e.to_string(),
            "Failed to fetch documentation for 'bwa': timeout"
        );
    }

    #[test]
    fn test_llm_error_display() {
        let e = OxoError::LlmError("rate limit exceeded".to_string());
        assert_eq!(e.to_string(), "LLM request failed: rate limit exceeded");
    }

    #[test]
    fn test_config_error_display() {
        let e = OxoError::ConfigError("missing api_token".to_string());
        assert_eq!(e.to_string(), "Configuration error: missing api_token");
    }

    #[test]
    fn test_index_error_display() {
        let e = OxoError::IndexError("corrupt index".to_string());
        assert_eq!(e.to_string(), "Index error: corrupt index");
    }

    #[test]
    fn test_execution_error_display() {
        let e = OxoError::ExecutionError("exit code 1".to_string());
        assert_eq!(e.to_string(), "Command execution failed: exit code 1");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e: OxoError = io_err.into();
        assert!(e.to_string().contains("IO error"));
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad json").unwrap_err();
        let e: OxoError = json_err.into();
        assert!(e.to_string().contains("JSON error"));
    }

    #[test]
    fn test_from_toml_de_error() {
        let toml_err = toml::from_str::<toml::Value>("bad = [[[").unwrap_err();
        let e: OxoError = toml_err.into();
        assert!(e.to_string().contains("TOML deserialization error"));
    }

    #[test]
    fn test_result_type_alias() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(OxoError::LlmError("test".to_string()));
        assert!(err.is_err());
    }
}
