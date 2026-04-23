//! LLM response caching based on semantic hash.
//!
//! This module provides a caching mechanism for LLM responses to avoid redundant API calls
//! for similar or identical prompts. The cache uses a semantic hash of the prompt components
//! (tool, task, documentation hash, skill name, model) as the key.
//!
//! # Architecture
//!
//! Lookups are O(1) via an in-memory `HashMap` that is lazily loaded from the
//! on-disk JSONL file.  Writes append to the JSONL file **and** update the
//! in-memory map so subsequent lookups within the same process are instant.
//!
//! # Cache Priority
//!
//! When cache is enabled, the priority order is:
//! 1. Cache hit (exact match)
//! 2. User preferences from command history
//! 3. Fresh LLM call
//!
//! # Configuration
//!
//! Cache is disabled by default for independent benchmarking. Enable via:
//! ```bash
//! oxo-call config set llm.cache_enabled true
//! ```

use crate::config::Config;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum age of cache entries (7 days)
const CACHE_MAX_AGE_DAYS: u64 = 7;

/// Maximum number of entries to keep in cache (LRU eviction at write time)
const CACHE_MAX_ENTRIES: usize = 10_000;

/// A single cache entry storing the LLM response and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Semantic hash of the prompt components
    pub hash: String,
    /// Tool name
    pub tool: String,
    /// Task description
    pub task: String,
    /// Generated command arguments
    pub args: String,
    /// Explanation text
    pub explanation: String,
    /// LLM model used
    pub model: String,
    /// Timestamp when the entry was created
    pub created_at: u64,
    /// Number of times this cache entry has been used
    pub hit_count: u64,
}

/// In-memory cache state backed by a JSONL file on disk.
struct MemoryCache {
    entries: HashMap<String, CacheEntry>,
    loaded: bool,
    /// Track whether in-memory state differs from disk (hit-count updates, evictions)
    dirty: bool,
}

impl MemoryCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            loaded: false,
            dirty: false,
        }
    }
}

/// Global in-memory cache singleton.  Lazily populated from disk on first
/// access, then kept in sync by write-through on `store()`.
static CACHE: std::sync::LazyLock<Mutex<MemoryCache>> =
    std::sync::LazyLock::new(|| Mutex::new(MemoryCache::new()));

/// Acquire the global cache mutex, recovering from a poisoned lock with a warning.
fn acquire_cache() -> std::sync::MutexGuard<'static, MemoryCache> {
    CACHE.lock().unwrap_or_else(|e| {
        tracing::warn!("Cache mutex was poisoned — recovering");
        e.into_inner()
    })
}

/// Cache storage manager
pub struct LlmCache;

impl LlmCache {
    /// Get the path to the cache file
    fn cache_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("llm_cache.jsonl"))
    }

    /// Load all entries from disk into the in-memory map (if not yet loaded).
    fn ensure_loaded(mem: &mut MemoryCache) -> Result<()> {
        if mem.loaded {
            return Ok(());
        }
        let path = Self::cache_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<CacheEntry>(line) {
                    mem.entries.insert(entry.hash.clone(), entry);
                }
            }
        }
        mem.loaded = true;
        Ok(())
    }

    /// Flush the in-memory map back to disk as JSONL.
    /// Used by sync() for explicit persistence (not during lookup for performance).
    #[allow(dead_code)]
    fn flush_to_disk(mem: &MemoryCache) -> Result<()> {
        let path = Self::cache_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut lines = Vec::with_capacity(mem.entries.len());
        for entry in mem.entries.values() {
            lines.push(serde_json::to_string(entry)?);
        }
        std::fs::write(&path, lines.join("\n") + "\n")?;
        Ok(())
    }

    /// Explicitly sync dirty in-memory state to disk.
    /// Call this before program exit or on explicit flush request.
    /// Note: Not currently called automatically; hit-count updates deferred for performance.
    ///       Data persisted on store() via append, dirty state kept in memory for speed.
    #[allow(dead_code)]
    pub fn sync() -> Result<()> {
        let mut mem = acquire_cache();
        if !mem.dirty {
            return Ok(()); // No changes to persist
        }
        Self::flush_to_disk(&mem)?;
        mem.dirty = false;
        Ok(())
    }

    /// Compute a semantic hash from prompt components.
    ///
    /// The hash is based on:
    /// - Tool name
    /// - Task description (normalized)
    /// - Documentation hash (if available)
    /// - Skill name (if used)
    /// - Model identifier
    pub fn compute_hash(
        tool: &str,
        task: &str,
        docs_hash: Option<&str>,
        skill_name: Option<&str>,
        model: &str,
    ) -> String {
        let mut hasher = Sha256::new();

        // Update hasher with each component
        hasher.update(tool.as_bytes());
        hasher.update(b"\0"); // Separator

        // Normalize task: lowercase, trim, collapse whitespace
        let normalized_task = task
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        hasher.update(normalized_task.as_bytes());
        hasher.update(b"\0");

        if let Some(docs) = docs_hash {
            hasher.update(docs.as_bytes());
        }
        hasher.update(b"\0");

        if let Some(skill) = skill_name {
            hasher.update(skill.as_bytes());
        }
        hasher.update(b"\0");

        hasher.update(model.as_bytes());

        hex::encode(hasher.finalize())
    }

    /// Look up a cached response by hash — O(1) via in-memory HashMap.
    ///
    /// Returns None if:
    /// - Cache is disabled
    /// - No matching entry exists
    /// - Entry is older than CACHE_MAX_AGE_DAYS
    pub fn lookup(
        tool: &str,
        task: &str,
        docs_hash: Option<&str>,
        skill_name: Option<&str>,
        model: &str,
    ) -> Result<Option<CacheEntry>> {
        // Check if cache is enabled
        let config = Config::load()?;
        if !config.llm.cache_enabled {
            return Ok(None);
        }

        let hash = Self::compute_hash(tool, task, docs_hash, skill_name, model);

        let mut mem = acquire_cache();
        Self::ensure_loaded(&mut mem)?;

        let entry = match mem.entries.get(&hash) {
            Some(e) => e.clone(),
            None => return Ok(None),
        };

        // Check age
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();
        let age_days = (now - entry.created_at) / (24 * 3600);

        if age_days > CACHE_MAX_AGE_DAYS {
            mem.entries.remove(&hash);
            mem.dirty = true; // Mark dirty, defer flush to explicit sync
            return Ok(None);
        }

        // Increment hit count in-memory (defer persistence for performance).
        let updated = CacheEntry {
            hit_count: entry.hit_count + 1,
            ..entry
        };
        mem.entries.insert(hash, updated.clone());
        mem.dirty = true; // Mark dirty, defer flush to explicit sync

        Ok(Some(updated))
    }

    /// Store a new cache entry — O(1) write-through to memory + append to disk.
    pub fn store(
        tool: &str,
        task: &str,
        docs_hash: Option<&str>,
        skill_name: Option<&str>,
        model: &str,
        args: &str,
        explanation: &str,
    ) -> Result<()> {
        // Check if cache is enabled
        let config = Config::load()?;
        if !config.llm.cache_enabled {
            return Ok(());
        }

        let hash = Self::compute_hash(tool, task, docs_hash, skill_name, model);
        let path = Self::cache_path()?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let entry = CacheEntry {
            hash: hash.clone(),
            tool: tool.to_string(),
            task: task.to_string(),
            args: args.to_string(),
            explanation: explanation.to_string(),
            model: model.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX epoch")
                .as_secs(),
            hit_count: 0,
        };

        // Write-through: update in-memory map.
        {
            let mut mem = acquire_cache();
            Self::ensure_loaded(&mut mem)?;

            // Evict oldest entries when over capacity.
            if mem.entries.len() >= CACHE_MAX_ENTRIES {
                let oldest_hash = mem
                    .entries
                    .values()
                    .min_by_key(|e| e.created_at)
                    .map(|e| e.hash.clone());
                if let Some(h) = oldest_hash {
                    mem.entries.remove(&h);
                }
            }

            mem.entries.insert(hash, entry.clone());
        }

        // Append to disk file (WAL-style).
        let json = serde_json::to_string(&entry)?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?
            .write_all(format!("{json}\n").as_bytes())?;

        Ok(())
    }

    /// Clear all cache entries.
    #[allow(dead_code)]
    pub fn clear() -> Result<()> {
        let path = Self::cache_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        // Also clear the in-memory map.
        let mut mem = acquire_cache();
        mem.entries.clear();
        mem.loaded = false;
        Ok(())
    }

    /// Get cache statistics — O(n) over the in-memory map.
    #[allow(dead_code)]
    pub fn stats() -> Result<CacheStats> {
        let mut mem = acquire_cache();
        Self::ensure_loaded(&mut mem)?;

        let total_entries = mem.entries.len();
        if total_entries == 0 {
            return Ok(CacheStats {
                total_entries: 0,
                total_hits: 0,
                oldest_entry_age_days: 0,
            });
        }

        let mut total_hits: u64 = 0;
        let mut oldest_timestamp = u64::MAX;
        for entry in mem.entries.values() {
            total_hits += entry.hit_count;
            if entry.created_at < oldest_timestamp {
                oldest_timestamp = entry.created_at;
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();
        let oldest_entry_age_days = if oldest_timestamp == u64::MAX {
            0
        } else {
            (now - oldest_timestamp) / (24 * 3600)
        };

        Ok(CacheStats {
            total_entries,
            total_hits,
            oldest_entry_age_days,
        })
    }
}

/// Cache statistics
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u64,
    pub oldest_entry_age_days: u64,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_deterministic() {
        let h1 = LlmCache::compute_hash("samtools", "sort bam file", None, None, "gpt-4");
        let h2 = LlmCache::compute_hash("samtools", "sort bam file", None, None, "gpt-4");
        assert_eq!(h1, h2, "Same inputs should produce same hash");
    }

    #[test]
    fn test_compute_hash_different_for_different_tools() {
        let h1 = LlmCache::compute_hash("samtools", "sort", None, None, "gpt-4");
        let h2 = LlmCache::compute_hash("bcftools", "sort", None, None, "gpt-4");
        assert_ne!(h1, h2, "Different tools should produce different hashes");
    }

    #[test]
    fn test_compute_hash_normalizes_task() {
        // "sort  BAM file" and "SORT BAM FILE" should produce the same hash
        let h1 = LlmCache::compute_hash("samtools", "sort  BAM file", None, None, "gpt-4");
        let h2 = LlmCache::compute_hash("samtools", "SORT BAM FILE", None, None, "gpt-4");
        assert_eq!(h1, h2, "Task normalization should make these equal");
    }

    #[test]
    fn test_compute_hash_different_for_different_models() {
        let h1 = LlmCache::compute_hash("samtools", "sort", None, None, "gpt-4");
        let h2 = LlmCache::compute_hash("samtools", "sort", None, None, "gpt-3.5");
        assert_ne!(h1, h2, "Different models should produce different hashes");
    }

    #[test]
    fn test_compute_hash_includes_docs_hash() {
        let h1 = LlmCache::compute_hash("samtools", "sort", Some("abc123"), None, "gpt-4");
        let h2 = LlmCache::compute_hash("samtools", "sort", None, None, "gpt-4");
        assert_ne!(h1, h2, "docs_hash should affect the hash");
    }

    #[test]
    fn test_compute_hash_includes_skill_name() {
        let h1 = LlmCache::compute_hash("samtools", "sort", None, Some("samtools-sort"), "gpt-4");
        let h2 = LlmCache::compute_hash("samtools", "sort", None, None, "gpt-4");
        assert_ne!(h1, h2, "skill_name should affect the hash");
    }

    #[test]
    fn test_compute_hash_is_sha256_hex() {
        let h = LlmCache::compute_hash("tool", "task", None, None, "model");
        assert_eq!(h.len(), 64, "SHA-256 hex digest should be 64 chars");
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex"
        );
    }

    #[test]
    fn test_cache_entry_serialization() {
        let entry = CacheEntry {
            hash: "abc".to_string(),
            tool: "samtools".to_string(),
            task: "sort".to_string(),
            args: "-o out.bam in.bam".to_string(),
            explanation: "Sort a BAM file".to_string(),
            model: "gpt-4".to_string(),
            created_at: 1700000000,
            hit_count: 5,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: CacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hash, "abc");
        assert_eq!(deserialized.tool, "samtools");
        assert_eq!(deserialized.hit_count, 5);
    }

    #[test]
    fn test_cache_stats_serialization() {
        let stats = CacheStats {
            total_entries: 42,
            total_hits: 100,
            oldest_entry_age_days: 3,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: CacheStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_entries, 42);
        assert_eq!(deserialized.total_hits, 100);
        assert_eq!(deserialized.oldest_entry_age_days, 3);
    }
}
