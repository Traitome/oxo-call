//! Persistent help cache for tool --help output.
//!
//! Caches the output of `<tool> --help` (and `<tool> <subcommand> --help`)
//! on disk so that it survives restarts.  The cache is keyed by
//! `{binary}:{version}:{subcommand}` and auto-invalidates on version change.
//!
//! ## Design
//!
//! - **Storage**: JSONL file in the oxo-call data directory (zero new deps).
//! - **Key**: `binary:version:subcommand` — version change → cache miss.
//! - **TTL**: 30 days from last access (forces periodic refresh).
//! - **Co-located**: Flag catalogs extracted from help are cached alongside
//!   the raw text so the `FlagExtractor` only runs once per version.

use crate::config::Config;
use crate::error::Result;
use crate::flag_extractor::{self, FlagCatalog};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum age of a cache entry before it's considered stale (30 days).
const MAX_AGE_SECS: u64 = 30 * 24 * 3600;

/// Maximum number of entries before oldest are evicted.
const MAX_ENTRIES: usize = 5000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpCacheEntry {
    /// Cache key: `{binary}:{version}:{subcommand}`
    pub key: String,
    /// The binary name.
    pub binary: String,
    /// Detected version string.
    pub version: String,
    /// Subcommand (empty string = top-level help).
    pub subcommand: String,
    /// Raw --help output text.
    pub help_text: String,
    /// Pre-extracted flag catalog (None if extraction failed).
    pub flag_catalog: Option<FlagCatalog>,
    /// Unix timestamp when this entry was created.
    pub created_at: u64,
    /// Unix timestamp of last access.
    pub last_accessed: u64,
}

/// In-memory index with disk backing.
struct Store {
    entries: Vec<HelpCacheEntry>,
    dirty: bool,
    loaded: bool,
}

static STORE: std::sync::LazyLock<Mutex<Store>> = std::sync::LazyLock::new(|| {
    Mutex::new(Store {
        entries: Vec::new(),
        dirty: false,
        loaded: false,
    })
});

fn acquire() -> std::sync::MutexGuard<'static, Store> {
    STORE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Persistent help cache.
pub struct HelpCache;

impl HelpCache {
    fn cache_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("help_cache.jsonl"))
    }

    fn ensure_loaded(store: &mut Store) -> Result<()> {
        if store.loaded {
            return Ok(());
        }
        let path = Self::cache_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<HelpCacheEntry>(line) {
                    store.entries.push(entry);
                }
            }
        }
        store.loaded = true;
        Ok(())
    }

    fn flush(store: &Store) -> Result<()> {
        let path = Self::cache_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = String::with_capacity(store.entries.len() * 512);
        for e in &store.entries {
            out.push_str(&serde_json::to_string(e)?);
            out.push('\n');
        }
        std::fs::write(&path, out)?;
        Ok(())
    }

    /// Build a cache key from binary, version, and optional subcommand.
    pub fn make_key(binary: &str, version: &str, subcommand: Option<&str>) -> String {
        let sc = subcommand.unwrap_or("");
        format!("{binary}:{version}:{sc}")
    }

    /// Look up cached help text by key.
    ///
    /// Returns `None` if not found or expired.  On hit, updates `last_accessed`.
    pub fn get(
        binary: &str,
        version: &str,
        subcommand: Option<&str>,
    ) -> Result<Option<HelpCacheEntry>> {
        let key = Self::make_key(binary, version, subcommand);
        let mut store = acquire();
        Self::ensure_loaded(&mut store)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Find & check age
        let pos = store.entries.iter().position(|e| e.key == key);
        match pos {
            Some(idx) => {
                let age = now.saturating_sub(store.entries[idx].created_at);
                if age > MAX_AGE_SECS {
                    store.entries.remove(idx);
                    store.dirty = true;
                    if let Err(e) = Self::flush(&store) {
                        tracing::warn!("help cache flush failed: {e}");
                    }
                    return Ok(None);
                }
                store.entries[idx].last_accessed = now;
                store.dirty = true;
                Ok(Some(store.entries[idx].clone()))
            }
            None => Ok(None),
        }
    }

    /// Store help text in the cache.
    ///
    /// Also runs `FlagExtractor` on the help text and stores the resulting
    /// catalog so it never needs to be re-extracted.
    pub fn put(
        binary: &str,
        version: &str,
        subcommand: Option<&str>,
        help_text: &str,
    ) -> Result<()> {
        let key = Self::make_key(binary, version, subcommand);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Pre-extract flag catalog
        let flag_catalog = Some(flag_extractor::extract_flags(help_text));

        let entry = HelpCacheEntry {
            key: key.clone(),
            binary: binary.to_string(),
            version: version.to_string(),
            subcommand: subcommand.unwrap_or("").to_string(),
            help_text: help_text.to_string(),
            flag_catalog,
            created_at: now,
            last_accessed: now,
        };

        let mut store = acquire();
        Self::ensure_loaded(&mut store)?;

        // Remove existing entry with same key (update-in-place)
        store.entries.retain(|e| e.key != key);

        // Evict oldest if over capacity
        while store.entries.len() >= MAX_ENTRIES {
            if let Some(oldest_idx) = store
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.created_at)
                .map(|(i, _)| i)
            {
                store.entries.remove(oldest_idx);
            } else {
                break;
            }
        }

        store.entries.push(entry);
        store.dirty = true;
        Self::flush(&store)?;
        Ok(())
    }

    /// Get the flag catalog for a cached entry (avoids re-parsing).
    pub fn get_flag_catalog(
        binary: &str,
        version: &str,
        subcommand: Option<&str>,
    ) -> Result<Option<FlagCatalog>> {
        match Self::get(binary, version, subcommand)? {
            Some(entry) => Ok(entry.flag_catalog),
            None => Ok(None),
        }
    }

    /// Clear all entries from the cache.
    #[allow(dead_code)]
    pub fn clear() -> Result<()> {
        let path = Self::cache_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let mut store = acquire();
        store.entries.clear();
        store.dirty = false;
        store.loaded = false;
        Ok(())
    }

    /// Number of cached entries.
    #[allow(dead_code)]
    pub fn entry_count() -> Result<usize> {
        let mut store = acquire();
        Self::ensure_loaded(&mut store)?;
        Ok(store.entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_key() {
        let key = HelpCache::make_key("samtools", "1.21", Some("sort"));
        assert_eq!(key, "samtools:1.21:sort");
        let key2 = HelpCache::make_key("samtools", "1.21", None);
        assert_eq!(key2, "samtools:1.21:");
    }

    #[test]
    fn test_put_and_get() {
        let _lock = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("OXO_CALL_DATA_DIR", tmp.path().to_string_lossy().as_ref()) };

        // Reset the global store
        {
            let mut store = acquire();
            store.entries.clear();
            store.loaded = false;
            store.dirty = false;
        }

        let help = "Usage: samtools sort [options]\n  -@ INT  threads\n  -o FILE output";
        HelpCache::put("samtools", "1.21", Some("sort"), help).unwrap();

        let entry = HelpCache::get("samtools", "1.21", Some("sort"))
            .unwrap()
            .expect("should find cached entry");
        assert_eq!(entry.binary, "samtools");
        assert_eq!(entry.help_text, help);
        assert!(entry.flag_catalog.is_some());

        // Different version should miss
        let miss = HelpCache::get("samtools", "1.20", Some("sort")).unwrap();
        assert!(miss.is_none());

        unsafe { std::env::remove_var("OXO_CALL_DATA_DIR") };
    }
}
