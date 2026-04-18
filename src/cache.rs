//! LLM response caching based on semantic hash.
//!
//! This module provides a caching mechanism for LLM responses to avoid redundant API calls
//! for similar or identical prompts. The cache uses a semantic hash of the prompt components
//! (tool, task, documentation hash, skill name, model) as the key.
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
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum age of cache entries (7 days)
const CACHE_MAX_AGE_DAYS: u64 = 7;

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

/// Cache storage manager
pub struct LlmCache;

impl LlmCache {
    /// Get the path to the cache file
    fn cache_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("llm_cache.jsonl"))
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

        format!("{:x}", hasher.finalize())
    }

    /// Look up a cached response by hash.
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
        let path = Self::cache_path()?;

        if !path.exists() {
            return Ok(None);
        }

        // Read cache file and search for matching entry
        let content = std::fs::read_to_string(&path)?;
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(line)
                && entry.hash == hash
            {
                // Check age
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let age_days = (now - entry.created_at) / (24 * 3600);

                if age_days <= CACHE_MAX_AGE_DAYS {
                    // Increment hit count
                    let updated = CacheEntry {
                        hit_count: entry.hit_count + 1,
                        ..entry.clone()
                    };
                    Self::update_entry(&updated)?;
                    return Ok(Some(updated));
                } else {
                    // Entry too old, remove it
                    Self::remove_entry(&hash)?;
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    /// Store a new cache entry.
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
            hash,
            tool: tool.to_string(),
            task: task.to_string(),
            args: args.to_string(),
            explanation: explanation.to_string(),
            model: model.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            hit_count: 0,
        };

        // Append to cache file
        let json = serde_json::to_string(&entry)?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?
            .write_all(format!("{}\n", json).as_bytes())?;

        Ok(())
    }

    /// Update an existing cache entry (increment hit count).
    fn update_entry(entry: &CacheEntry) -> Result<()> {
        let path = Self::cache_path()?;
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&path)?;
        let mut updated_lines = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(existing) = serde_json::from_str::<CacheEntry>(line) {
                if existing.hash == entry.hash {
                    updated_lines.push(serde_json::to_string(entry)?);
                } else {
                    updated_lines.push(line.to_string());
                }
            }
        }

        std::fs::write(&path, updated_lines.join("\n") + "\n")?;
        Ok(())
    }

    /// Remove a cache entry by hash.
    fn remove_entry(hash: &str) -> Result<()> {
        let path = Self::cache_path()?;
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&path)?;
        let mut filtered_lines = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(line)
                && entry.hash != hash
            {
                filtered_lines.push(line.to_string());
            }
        }

        std::fs::write(&path, filtered_lines.join("\n") + "\n")?;
        Ok(())
    }

    /// Clear all cache entries.
    #[allow(dead_code)]
    pub fn clear() -> Result<()> {
        let path = Self::cache_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Get cache statistics.
    #[allow(dead_code)]
    pub fn stats() -> Result<CacheStats> {
        let path = Self::cache_path()?;
        if !path.exists() {
            return Ok(CacheStats {
                total_entries: 0,
                total_hits: 0,
                oldest_entry_age_days: 0,
            });
        }

        let content = std::fs::read_to_string(&path)?;
        let mut total_entries = 0;
        let mut total_hits = 0;
        let mut oldest_timestamp = u64::MAX;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(line) {
                total_entries += 1;
                total_hits += entry.hit_count;
                if entry.created_at < oldest_timestamp {
                    oldest_timestamp = entry.created_at;
                }
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
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

use std::io::Write;

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
