//! MCP (Model Context Protocol) skill provider.
//!
//! This module implements a minimal MCP client that fetches skill content from
//! any MCP-compatible server.  Only the subset of the protocol needed for
//! read-only skill discovery is implemented:
//!
//! - `initialize` — protocol handshake
//! - `resources/list` — discover available skill resources
//! - `resources/read` — fetch a skill by its URI
//!
//! ## Transport
//!
//! HTTP POST with JSON-RPC 2.0 payloads (the "streamable HTTP" transport
//! introduced in MCP spec 2024-11-05 / 2025-03-26).  Every call is a
//! self-contained stateless POST, so no SSE session management is needed for
//! the read-only skill path.
//!
//! ## Skill URI convention
//!
//! An MCP server acting as a skill provider should expose skill resources with
//! URIs of the form `skill://<tool>` (e.g. `skill://samtools`).  Servers that
//! expose all their resources as `text/markdown` with a plain tool name are also
//! supported as a fallback.
//!
//! ## Configuration
//!
//! MCP servers are registered in `~/.config/oxo-call/config.toml` under the
//! `[mcp]` section:
//!
//! ```toml
//! [[mcp.servers]]
//! url  = "http://localhost:3000"
//! name = "local-skills"
//!
//! [[mcp.servers]]
//! url     = "https://skills.example.org/mcp"
//! name    = "org-skills"
//! api_key = "secret-token"
//! ```

use crate::config::McpServerConfig;
use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// Default HTTP timeout for MCP requests.
const MCP_TIMEOUT_SECS: u64 = 5;

// ─── JSON-RPC 2.0 wire types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: u64,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Deserialize)]
struct RpcResponse {
    #[allow(dead_code)]
    id: Option<Value>,
    result: Option<Value>,
    error: Option<Value>,
}

// ─── MCP client ───────────────────────────────────────────────────────────────

/// Minimal stateless MCP client for skill discovery and retrieval.
///
/// Uses HTTP POST + JSON-RPC 2.0. No persistent session or SSE streaming is
/// required because oxo-call only performs read-only resource operations.
pub struct McpClient {
    config: McpServerConfig,
    #[cfg(not(target_arch = "wasm32"))]
    http: reqwest::Client,
}

impl McpClient {
    /// Create a new client for the given MCP server configuration.
    pub fn new(config: McpServerConfig) -> Self {
        McpClient {
            config,
            #[cfg(not(target_arch = "wasm32"))]
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(MCP_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
        }
    }

    /// The JSON-RPC endpoint URL.
    ///
    /// If the configured URL already ends in `/mcp`, use it as-is;
    /// otherwise append `/mcp` (the conventional MCP path).
    fn endpoint(&self) -> String {
        let base = self.config.url.trim_end_matches('/');
        if base.ends_with("/mcp") {
            base.to_string()
        } else {
            format!("{base}/mcp")
        }
    }

    // ── Internal HTTP helper ──────────────────────────────────────────────

    #[cfg(not(target_arch = "wasm32"))]
    async fn send(&self, method: &str, params: Option<Value>, id: u64) -> Result<Value> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };

        let mut builder = self
            .http
            .post(self.endpoint())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("MCP-Protocol-Version", "2024-11-05")
            .json(&req);

        if let Some(key) = &self.config.api_key {
            builder = builder.header("Authorization", format!("Bearer {key}"));
        }

        let response = builder.send().await.map_err(|e| {
            OxoError::IndexError(format!(
                "MCP server '{}' unreachable: {e}",
                self.config.name()
            ))
        })?;

        if !response.status().is_success() {
            return Err(OxoError::IndexError(format!(
                "MCP server '{}' returned HTTP {}",
                self.config.name(),
                response.status()
            )));
        }

        let rpc: RpcResponse = response.json().await.map_err(|e| {
            OxoError::IndexError(format!(
                "MCP server '{}' returned invalid JSON: {e}",
                self.config.name()
            ))
        })?;

        if let Some(err) = rpc.error {
            return Err(OxoError::IndexError(format!(
                "MCP error from '{}': {err}",
                self.config.name()
            )));
        }

        rpc.result.ok_or_else(|| {
            OxoError::IndexError(format!(
                "MCP server '{}' returned empty result for '{method}'",
                self.config.name()
            ))
        })
    }

    // ── Public MCP operations ─────────────────────────────────────────────

    /// Perform the MCP `initialize` handshake.
    ///
    /// Returns `(server_name, server_version)` for display purposes.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn initialize(&self) -> Result<(String, String)> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "oxo-call",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let result = self.send("initialize", Some(params), 1).await?;
        let name = result["serverInfo"]["name"]
            .as_str()
            .unwrap_or(&self.config.url)
            .to_string();
        let version = result["serverInfo"]["version"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        Ok((name, version))
    }

    /// Wasm32-compatible stub: MCP HTTP transport is not available in WebAssembly.
    #[cfg(target_arch = "wasm32")]
    pub async fn initialize(&self) -> Result<(String, String)> {
        Err(OxoError::IndexError(
            "MCP is not supported in WebAssembly".to_string(),
        ))
    }

    /// Call `resources/list` to discover skill resources on this server.
    ///
    /// Returns a list of `(uri, tool_name, description)` triples.  Only
    /// resources with a `skill://` URI scheme or a `text/markdown` MIME type
    /// are included.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn list_skill_resources(&self) -> Result<Vec<McpSkillEntry>> {
        let result = self.send("resources/list", None, 2).await?;
        let resources = match result["resources"].as_array() {
            Some(r) => r.clone(),
            None => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        for res in &resources {
            let uri = res["uri"].as_str().unwrap_or("").to_string();
            let name = res["name"].as_str().unwrap_or("").to_string();
            let description = res["description"].as_str().unwrap_or("").to_string();
            let mime = res["mimeType"].as_str().unwrap_or("");

            if let Some(tool) = uri.strip_prefix("skill://") {
                entries.push(McpSkillEntry {
                    tool: tool.to_string(),
                    description,
                    uri,
                });
            } else if mime == "text/markdown" && !name.is_empty() {
                entries.push(McpSkillEntry {
                    tool: name,
                    description,
                    uri,
                });
            }
        }
        Ok(entries)
    }

    /// Wasm32-compatible stub: MCP HTTP transport is not available in WebAssembly.
    #[cfg(target_arch = "wasm32")]
    pub async fn list_skill_resources(&self) -> Result<Vec<McpSkillEntry>> {
        Err(OxoError::IndexError(
            "MCP is not supported in WebAssembly".to_string(),
        ))
    }

    /// Call `resources/read` to fetch the Markdown content for a skill URI.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn read_resource(&self, uri: &str) -> Result<String> {
        let params = json!({ "uri": uri });
        let result = self.send("resources/read", Some(params), 3).await?;

        // MCP resources/read response shape:
        //   { "contents": [{ "uri": "...", "text": "...", "mimeType": "..." }] }
        let contents = result["contents"].as_array().cloned().unwrap_or_default();
        for item in &contents {
            if let Some(text) = item["text"].as_str()
                && !text.is_empty()
            {
                return Ok(text.to_string());
            }
        }
        Err(OxoError::IndexError(format!(
            "MCP server '{}' returned empty content for '{uri}'",
            self.config.name()
        )))
    }

    /// Convenience: resolve a tool name → URI → Markdown content.
    ///
    /// First tries the canonical `skill://<tool>` URI directly; if that fails,
    /// falls back to scanning the resource list for a matching tool name.
    ///
    /// Returns `None` if the tool is not available on this server.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn fetch_skill(&self, tool: &str) -> Option<String> {
        // Fast path: try canonical URI first
        let canonical = format!("skill://{tool}");
        if let Ok(content) = self.read_resource(&canonical).await {
            return Some(content);
        }

        // Slow path: scan resource list
        if let Ok(entries) = self.list_skill_resources().await {
            let tool_lc = tool.to_ascii_lowercase();
            for entry in entries {
                if entry.tool.to_ascii_lowercase() == tool_lc
                    && let Ok(content) = self.read_resource(&entry.uri).await
                {
                    return Some(content);
                }
            }
        }
        None
    }
}

// ─── Helper types ──────────────────────────────────────────────────────────────

/// A single skill resource discovered via `resources/list`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct McpSkillEntry {
    /// The resource URI (e.g. `skill://samtools`)
    pub uri: String,
    /// Canonical tool name derived from the URI or resource name
    pub tool: String,
    /// Optional description from the resource metadata
    pub description: String,
}
