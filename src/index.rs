use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::runner::make_spinner;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Legacy DocIndex format for migration - can deserialize either Vec or HashMap.
#[derive(Debug, Clone, Deserialize)]
struct LegacyDocIndex {
    #[serde(deserialize_with = "deserialize_entries_legacy")]
    entries: HashMap<String, IndexEntry>,
}

/// Custom deserializer that handles both Vec and HashMap formats for entries.
fn deserialize_entries_legacy<'de, D>(
    deserializer: D,
) -> std::result::Result<HashMap<String, IndexEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    // Try HashMap first (current format), then Vec (legacy format)
    let value = serde_json::Value::deserialize(deserializer)?;

    // Try as HashMap
    if let Ok(map) = serde_json::from_value::<HashMap<String, IndexEntry>>(value.clone()) {
        return Ok(map);
    }

    // Try as Vec and convert
    if let Ok(vec) = serde_json::from_value::<Vec<IndexEntry>>(value) {
        let map = vec.into_iter().map(|e| (e.tool_name.clone(), e)).collect();
        return Ok(map);
    }

    Err(Error::custom("entries must be either HashMap or Vec"))
}

/// The full documentation index, stored as a JSON manifest.
/// Uses HashMap for O(1) tool lookup instead of O(n) Vec search.
/// Supports both HashMap and legacy Vec formats during deserialization.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DocIndex {
    /// tool_name -> IndexEntry mapping for fast lookup
    entries: HashMap<String, IndexEntry>,
}

// Custom deserialize to handle legacy Vec format migration
impl<'de> Deserialize<'de> for DocIndex {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let legacy = LegacyDocIndex::deserialize(deserializer)?;
        Ok(DocIndex {
            entries: legacy.entries,
        })
    }
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

        // Fast path: well-formed single JSON object (current format).
        // Capture the error so it can be surfaced if all migration attempts fail.
        let original_err = match serde_json::from_str::<DocIndex>(&content) {
            Ok(index) => return Ok(index),
            Err(e) => e,
        };

        // Migration path: legacy versions may have had a save bug that appended
        // each new DocIndex to the file without overwriting, producing multiple
        // concatenated JSON objects with no separator (e.g. `{...}{...}`).
        // Stream through all valid DocIndex objects and take the last one, which
        // has the most up-to-date state.  On success, rewrite the file in the
        // current single-object format so future reads take the fast path.
        let mut stream = serde_json::Deserializer::from_str(&content).into_iter::<LegacyDocIndex>();
        let mut last: Option<DocIndex> = None;
        for item in stream.by_ref() {
            match item {
                Ok(legacy) => {
                    last = Some(DocIndex {
                        entries: legacy.entries,
                    })
                }
                Err(_) => break,
            }
        }
        if let Some(idx) = last {
            // Best-effort repair: rewrite in current format.
            let _ = idx.save();
            return Ok(idx);
        }

        // Last resort: try the legacy bare Vec<IndexEntry> format (plain JSON
        // array without the DocIndex wrapper object).
        if let Ok(entries_vec) = serde_json::from_str::<Vec<IndexEntry>>(&content) {
            // Convert legacy Vec format to HashMap
            let entries: HashMap<String, IndexEntry> = entries_vec
                .into_iter()
                .map(|e| (e.tool_name.clone(), e))
                .collect();
            let idx = DocIndex { entries };
            let _ = idx.save();
            return Ok(idx);
        }

        // File is genuinely unreadable – surface the original parse error.
        Err(original_err.into())
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
        self.entries.get(tool)
    }

    pub fn upsert(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.tool_name.clone(), entry);
    }

    pub fn remove(&mut self, tool: &str) -> bool {
        self.entries.remove(tool).is_some()
    }

    /// Returns all entries as a Vec for listing/iteration.
    pub fn entries_vec(&self) -> Vec<IndexEntry> {
        self.entries.values().cloned().collect()
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
        Ok(index.entries_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::config::Config;
    use crate::docs::DocsFetcher;
    use chrono::Utc;

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
        assert!(idx.get("samtools").is_some());
    }

    #[test]
    fn test_docindex_upsert_replaces_existing_entry() {
        let mut idx = DocIndex::default();
        idx.upsert(make_entry("samtools", 1024));
        idx.upsert(make_entry("samtools", 2048));
        assert_eq!(idx.entries.len(), 1);
        assert_eq!(idx.get("samtools").unwrap().doc_size_bytes, 2048);
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

    // ─── Legacy format migration ──────────────────────────────────────────────

    /// Simulate the v0.8.0 append-mode bug: each `save()` concatenated the new
    /// JSON directly after the previous one with no separator.  The result is
    /// multiple `DocIndex` JSON objects in the same file with no newline between
    /// them, e.g. `{"entries":[...]}{"entries":[...]}`.
    /// `DocIndex::load()` must recover the last (most recent) object.
    #[test]
    fn test_load_migrates_concatenated_objects_no_separator() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        // Build two DocIndex "snapshots" as v0.8.0 would have written them.
        let mut idx1 = DocIndex::default();
        idx1.upsert(make_entry("git", 1000));
        let snap1 = serde_json::to_string_pretty(&idx1).unwrap();

        let mut idx2 = DocIndex::default();
        idx2.upsert(make_entry("git", 1000));
        idx2.upsert(make_entry("samtools", 2000));
        let snap2 = serde_json::to_string_pretty(&idx2).unwrap();

        // Write them concatenated without any separator – the exact byte layout
        // that triggers "trailing characters at line 13 column 2".
        let corrupt = format!("{snap1}{snap2}");
        let index_path = tmp.path().join("index.json");
        std::fs::write(&index_path, corrupt).unwrap();

        let loaded = DocIndex::load().unwrap();
        // Should have recovered the last (most-complete) snapshot.
        assert_eq!(
            loaded.entries.len(),
            2,
            "expected 2 entries from last snapshot"
        );
        assert!(loaded.get("git").is_some());
        assert!(loaded.get("samtools").is_some());

        // The file should now be repaired (single valid JSON object).
        let repaired = std::fs::read_to_string(&index_path).unwrap();
        let re_parsed: DocIndex = serde_json::from_str(&repaired).unwrap();
        assert_eq!(re_parsed.entries.len(), 2);
    }

    /// Same as above but with a newline between the concatenated objects.
    #[test]
    fn test_load_migrates_concatenated_objects_with_newline() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        let mut idx1 = DocIndex::default();
        idx1.upsert(make_entry("bwa", 500));
        let snap1 = serde_json::to_string_pretty(&idx1).unwrap();

        let mut idx2 = DocIndex::default();
        idx2.upsert(make_entry("bwa", 500));
        idx2.upsert(make_entry("gatk", 8000));
        let snap2 = serde_json::to_string_pretty(&idx2).unwrap();

        let corrupt = format!("{snap1}\n{snap2}");
        std::fs::write(tmp.path().join("index.json"), corrupt).unwrap();

        let loaded = DocIndex::load().unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert!(loaded.get("gatk").is_some());
    }

    /// Legacy format: bare `Vec<IndexEntry>` JSON array without the DocIndex
    /// wrapper object.
    #[test]
    fn test_load_migrates_bare_entries_array() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        let entries = vec![make_entry("star", 3000), make_entry("hisat2", 4000)];
        let legacy_json = serde_json::to_string_pretty(&entries).unwrap();
        std::fs::write(tmp.path().join("index.json"), legacy_json).unwrap();

        let loaded = DocIndex::load().unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert!(loaded.get("star").is_some());
        assert!(loaded.get("hisat2").is_some());
    }
}
