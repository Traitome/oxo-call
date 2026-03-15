use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::runner::make_spinner;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Metadata entry for a tool's documentation index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub tool_name: String,
    pub version: Option<String>,
    pub indexed_at: DateTime<Utc>,
    pub doc_size_bytes: usize,
    pub sources: Vec<String>,
}

/// The full documentation index, stored as a JSON manifest
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocIndex {
    pub entries: Vec<IndexEntry>,
}

impl DocIndex {
    fn index_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("index.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::index_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let index: DocIndex = serde_json::from_str(&content)?;
        Ok(index)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::index_path()?;
        let dir = path
            .parent()
            .ok_or_else(|| OxoError::IndexError("Index path has no parent directory".into()))?;
        std::fs::create_dir_all(dir)?;
        let content = serde_json::to_string_pretty(self)?;
        // Write to a uniquely-named sibling temp file first, then atomically rename into
        // place.  Using a UUID suffix prevents concurrent CLI invocations (e.g. parallel
        // integration-test runs) from racing on the same `.tmp` path and hitting ENOENT
        // on the subsequent rename.
        let tmp_path = dir.join(format!("index.{}.tmp", Uuid::new_v4().simple()));
        std::fs::write(&tmp_path, &content)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, tool: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.tool_name == tool)
    }

    pub fn upsert(&mut self, entry: IndexEntry) {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| e.tool_name == entry.tool_name)
        {
            self.entries[pos] = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn remove(&mut self, tool: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.tool_name != tool);
        self.entries.len() < before
    }
}

pub struct IndexManager {
    fetcher: DocsFetcher,
}

impl IndexManager {
    pub fn new(config: Config) -> Self {
        IndexManager {
            fetcher: DocsFetcher::new(config),
        }
    }

    /// Add or update a tool in the documentation index.
    ///
    /// Documentation sources (all optional, combined in order):
    /// - Live `--help` output from the installed tool
    /// - `url`: remote documentation page (HTTP/HTTPS)
    /// - `file`: path to a local documentation file (.md/.txt/.rst/.html)
    /// - `dir`: directory containing documentation files (non-recursive)
    pub async fn add(
        &self,
        tool: &str,
        url: Option<&str>,
        file: Option<&std::path::Path>,
        dir: Option<&std::path::Path>,
    ) -> Result<IndexEntry> {
        let spinner = make_spinner(&format!("Fetching documentation for '{tool}'..."));

        let mut sources: Vec<String> = Vec::new();
        let mut combined_doc = String::new();
        let mut version: Option<String> = None;

        // Try to get help output from the installed tool
        match self.fetcher.fetch(tool).await {
            Ok(docs) => {
                if let Some(v) = docs.version {
                    version = Some(v);
                }
                if let Some(help) = docs.help_output {
                    sources.push("--help".to_string());
                    combined_doc.push_str("# Help Output\n\n");
                    combined_doc.push_str(&help);
                    combined_doc.push_str("\n\n");
                }
                if let Some(cached) = docs.cached_docs {
                    sources.push("cache".to_string());
                    combined_doc.push_str("# Cached Documentation\n\n");
                    combined_doc.push_str(&cached);
                    combined_doc.push_str("\n\n");
                }
            }
            Err(OxoError::ToolNotFound(_)) => {
                // Tool may not be installed; try remote if URL provided
            }
            Err(OxoError::DocFetchError(_, _)) => {
                // No help output available, continue with URL/file/dir
            }
            Err(e) => {
                spinner.finish_and_clear();
                return Err(e);
            }
        }

        // Optionally fetch from a remote URL
        if let Some(url) = url {
            spinner.set_message(format!("Fetching remote docs from {url}..."));
            match self.fetcher.fetch_remote(tool, url).await {
                Ok(remote_content) => {
                    sources.push(format!("remote:{url}"));
                    combined_doc.push_str("# Remote Documentation\n\n");
                    combined_doc.push_str(&remote_content);
                    combined_doc.push_str("\n\n");
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    return Err(e);
                }
            }
        }

        // Optionally read from a local file
        if let Some(file_path) = file {
            spinner.set_message(format!("Reading docs from {}...", file_path.display()));
            match self.fetcher.fetch_from_file(tool, file_path) {
                Ok(file_content) => {
                    sources.push(format!("file:{}", file_path.display()));
                    combined_doc.push_str("# Local File Documentation\n\n");
                    combined_doc.push_str(&file_content);
                    combined_doc.push_str("\n\n");
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    return Err(e);
                }
            }
        }

        // Optionally scan a local directory
        if let Some(dir_path) = dir {
            spinner.set_message(format!("Scanning docs from {}...", dir_path.display()));
            match self.fetcher.fetch_from_dir(tool, dir_path) {
                Ok(dir_content) => {
                    sources.push(format!("dir:{}", dir_path.display()));
                    combined_doc.push_str("# Directory Documentation\n\n");
                    combined_doc.push_str(&dir_content);
                    combined_doc.push_str("\n\n");
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    return Err(e);
                }
            }
        }

        spinner.finish_and_clear();

        if combined_doc.is_empty() {
            return Err(OxoError::IndexError(format!(
                "Could not retrieve any documentation for '{tool}'. \
                Make sure the tool is installed or provide --url/--file/--dir."
            )));
        }

        // Save combined docs to cache
        self.fetcher.save_cache(tool, &combined_doc)?;

        let entry = IndexEntry {
            tool_name: tool.to_string(),
            version,
            indexed_at: Utc::now(),
            doc_size_bytes: combined_doc.len(),
            sources,
        };

        let mut index = DocIndex::load()?;
        index.upsert(entry.clone());
        index.save()?;

        Ok(entry)
    }

    /// Remove a tool from the documentation index
    pub fn remove(&self, tool: &str) -> Result<()> {
        let mut index = DocIndex::load()?;
        let removed = index.remove(tool);
        if !removed {
            return Err(OxoError::IndexError(format!(
                "Tool '{tool}' is not in the index"
            )));
        }
        index.save()?;
        self.fetcher.remove_cache(tool)?;
        Ok(())
    }

    /// List all indexed tools
    pub fn list(&self) -> Result<Vec<IndexEntry>> {
        let index = DocIndex::load()?;
        Ok(index.entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::docs::DocsFetcher;
    use chrono::Utc;
    use std::sync::Mutex;

    // Mutex to serialize tests that mutate OXO_CALL_DATA_DIR
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn make_entry(tool: &str, size: usize) -> IndexEntry {
        IndexEntry {
            tool_name: tool.to_string(),
            version: Some("1.0".to_string()),
            indexed_at: Utc::now(),
            doc_size_bytes: size,
            sources: vec!["help".to_string()],
        }
    }

    // ─── DocIndex unit tests ──────────────────────────────────────────────────

    #[test]
    fn test_docindex_default_empty() {
        let idx = DocIndex::default();
        assert!(idx.entries.is_empty());
    }

    #[test]
    fn test_docindex_upsert_adds_new_entry() {
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        assert_eq!(idx.entries.len(), 1);
        assert_eq!(idx.entries[0].tool_name, "samtools");
    }

    #[test]
    fn test_docindex_upsert_replaces_existing_entry() {
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        idx.upsert(make_entry("samtools", 2048));
        assert_eq!(idx.entries.len(), 1);
        assert_eq!(idx.entries[0].doc_size_bytes, 2048);
    }

    #[test]
    fn test_docindex_get_existing() {
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("bwa", 512));
        let entry = idx.get("bwa");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().tool_name, "bwa");
    }

    #[test]
    fn test_docindex_get_missing() {
        let idx = DocIndex::default();
        assert!(idx.get("nonexistent").is_none());
    }

    #[test]
    fn test_docindex_remove_existing() {
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("gatk", 4096));
        let removed = idx.remove("gatk");
        assert!(removed);
        assert!(idx.entries.is_empty());
    }

    #[test]
    fn test_docindex_remove_missing_returns_false() {
        let mut idx = DocIndex::default();
        let removed = idx.remove("doesnotexist");
        assert!(!removed);
    }

    #[test]
    fn test_docindex_remove_leaves_others_intact() {
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        idx.upsert(make_entry("bwa", 512));
        idx.upsert(make_entry("gatk", 4096));
        idx.remove("bwa");
        assert_eq!(idx.entries.len(), 2);
        assert!(idx.get("samtools").is_some());
        assert!(idx.get("gatk").is_some());
        assert!(idx.get("bwa").is_none());
    }

    // ─── DocIndex save/load round-trip ────────────────────────────────────────

    #[test]
    fn test_docindex_save_load_round_trip() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        idx.upsert(IndexEntry {
            tool_name: "bwa".to_string(),
            version: None,
            indexed_at: Utc::now(),
            doc_size_bytes: 512,
            sources: vec!["remote".to_string(), "help".to_string()],
        });
        idx.save().unwrap();

        let loaded = DocIndex::load().unwrap();
        assert_eq!(loaded.entries.len(), 2);
        let sam = loaded.get("samtools").unwrap();
        assert_eq!(sam.version.as_deref(), Some("1.0"));
        let bwa = loaded.get("bwa").unwrap();
        assert!(bwa.version.is_none());
        assert_eq!(bwa.sources.len(), 2);
    }

    #[test]
    fn test_docindex_load_returns_empty_when_no_file() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let loaded = DocIndex::load().unwrap();
        assert!(loaded.entries.is_empty());
    }

    // ─── IndexManager::list ───────────────────────────────────────────────────

    #[test]
    fn test_index_manager_list_empty() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let config = Config::default();
        let mgr = IndexManager::new(config);
        let entries = mgr.list().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_index_manager_list_after_upsert() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        // Manually create and save an index
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        idx.upsert(make_entry("bwa", 512));
        idx.save().unwrap();

        let config = Config::default();
        let mgr = IndexManager::new(config);
        let entries = mgr.list().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.tool_name == "samtools"));
        assert!(entries.iter().any(|e| e.tool_name == "bwa"));
    }

    // ─── IndexManager::remove ─────────────────────────────────────────────────

    #[test]
    fn test_index_manager_remove_nonexistent_errors() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let config = Config::default();
        let mgr = IndexManager::new(config);
        let result = mgr.remove("nonexistent_tool");
        assert!(result.is_err(), "removing nonexistent tool should error");
    }

    #[test]
    fn test_index_manager_remove_existing() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        // Save an entry and a cache file for it
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        idx.save().unwrap();

        // Create the cache file
        let fetcher = DocsFetcher::new(Config::default());
        fetcher.save_cache("samtools", "samtools docs").unwrap();

        let config = Config::default();
        let mgr = IndexManager::new(config);
        mgr.remove("samtools").unwrap();

        // Verify it's gone from the index
        let entries = mgr.list().unwrap();
        assert!(entries.iter().all(|e| e.tool_name != "samtools"));
    }

    // ─── IndexEntry serialization ─────────────────────────────────────────────

    #[test]
    fn test_index_entry_serialization() {
        let entry = make_entry("gatk", 8192);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"tool_name\":\"gatk\""));
        assert!(json.contains("\"doc_size_bytes\":8192"));

        let back: IndexEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tool_name, "gatk");
        assert_eq!(back.doc_size_bytes, 8192);
    }

    #[test]
    fn test_index_entry_no_version_serializes() {
        let entry = IndexEntry {
            tool_name: "bwa".to_string(),
            version: None,
            indexed_at: Utc::now(),
            doc_size_bytes: 512,
            sources: vec![],
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: IndexEntry = serde_json::from_str(&json).unwrap();
        assert!(back.version.is_none());
    }
}
