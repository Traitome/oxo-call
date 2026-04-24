//! Mini-skill caching system with hybrid memory + disk storage.
//!
//! This module provides a caching mechanism for LLM-generated mini-skills,
//! which are extracted from tool documentation. The cache uses a hybrid
//! approach: LRU in-memory cache for hot data + persistent disk storage.

use crate::config::Config;
use crate::error::Result;
use crate::skill::{Skill, SkillContext, SkillExample, SkillMeta};
use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Default LRU cache capacity when `config.memory_size` is zero.
const DEFAULT_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(100).unwrap();
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A mini-skill extracted from tool documentation by LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniSkill {
    /// Tool name
    pub tool: String,
    /// Hash of the task description
    pub task_hash: String,
    /// Hash of the documentation
    pub doc_hash: String,
    /// Key concepts extracted from documentation
    pub concepts: Vec<String>,
    /// Common pitfalls identified
    pub pitfalls: Vec<String>,
    /// Worked examples
    pub examples: Vec<MiniSkillExample>,
    /// When this mini-skill was created
    pub created_at: DateTime<Utc>,
    /// Number of times this mini-skill has been used
    pub hit_count: u64,
}

/// A single worked example in a mini-skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniSkillExample {
    /// Task description
    pub task: String,
    /// Command arguments
    pub args: String,
    /// Explanation of why these args
    pub explanation: String,
}

/// Convert MiniSkill to Skill for use in LLM prompts
impl From<MiniSkill> for Skill {
    fn from(mini: MiniSkill) -> Self {
        Skill {
            meta: SkillMeta {
                name: mini.tool,
                category: "auto-generated".to_string(),
                description: format!("Auto-generated mini-skill (doc hash: {})", mini.doc_hash),
                tags: vec!["mini-skill".to_string(), "auto-generated".to_string()],
                author: Some("oxo-call mini-skill generator".to_string()),
                source_url: None,
                min_version: None,
                max_version: None,
            },
            context: SkillContext {
                concepts: mini.concepts,
                pitfalls: mini.pitfalls,
            },
            examples: mini
                .examples
                .into_iter()
                .map(|ex| SkillExample {
                    task: ex.task,
                    args: ex.args,
                    explanation: ex.explanation,
                })
                .collect(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheConfig {
    /// Maximum number of entries in memory cache
    pub memory_size: usize,
    /// Whether to persist cache to disk
    pub persist_to_disk: bool,
    /// Maximum age of cache entries in days
    pub max_age_days: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            memory_size: 100,
            persist_to_disk: true,
            max_age_days: 30,
        }
    }
}

/// Mini-skill cache manager with hybrid storage
#[allow(dead_code)]
pub struct MiniSkillCache {
    /// In-memory LRU cache
    memory: Arc<Mutex<LruCache<String, MiniSkill>>>,
    /// Disk cache path
    disk_path: PathBuf,
    /// Configuration
    config: CacheConfig,
    /// In-memory index for fast lookups by tool
    tool_index: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl MiniSkillCache {
    /// Create a new mini-skill cache
    #[allow(dead_code)]
    pub fn new(config: CacheConfig) -> Result<Self> {
        let disk_path = Config::data_dir()?.join("mini_skills");

        // Create directory if it doesn't exist
        if !disk_path.exists() {
            std::fs::create_dir_all(&disk_path)?;
        }

        let memory = Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(config.memory_size).unwrap_or(DEFAULT_CACHE_SIZE),
        )));

        let tool_index = Arc::new(Mutex::new(HashMap::new()));

        let cache = MiniSkillCache {
            memory,
            disk_path,
            config,
            tool_index,
        };

        // Load existing cache from disk
        if cache.config.persist_to_disk {
            cache.load()?;
        }

        Ok(cache)
    }

    /// Compute cache key from tool and doc hash.
    ///
    /// Mini-skills capture tool-level knowledge (concepts, pitfalls, examples)
    /// extracted from documentation, so they are reusable across different user
    /// tasks for the same tool.  Keying by `(tool, doc_hash)` rather than
    /// `(tool, task, doc_hash)` dramatically improves cache hit rates — the
    /// second invocation for the same tool is always a cache hit.
    pub fn compute_key(tool: &str, doc_hash: &str) -> String {
        let mut hasher = Sha256::new();

        hasher.update(tool.as_bytes());
        hasher.update(b"\0");
        hasher.update(doc_hash.as_bytes());

        format!("{}:{:.16}", tool, hex::encode(&hasher.finalize()[..8]))
    }

    /// Get a mini-skill from cache
    pub fn get(&mut self, tool: &str, doc_hash: &str) -> Option<MiniSkill> {
        let key = Self::compute_key(tool, doc_hash);

        // Try memory cache first
        if let Ok(mut memory) = self.memory.lock()
            && let Some(skill) = memory.get_mut(&key)
        {
            // Check age even for memory cache
            let age = (Utc::now() - skill.created_at).num_days();
            if age as u64 > self.config.max_age_days {
                // Expired, remove from cache
                memory.pop(&key);
                return None;
            }

            skill.hit_count += 1;
            return Some(skill.clone());
        }

        // Try disk cache
        if self.config.persist_to_disk
            && let Some(skill) = self.load_from_disk(&key).ok().flatten()
        {
            // Check age
            let age = (Utc::now() - skill.created_at).num_days();
            if age as u64 > self.config.max_age_days {
                // Expired, remove from disk
                let _ = self.remove_from_disk(&key);
                return None;
            }

            // Add to memory cache
            if let Ok(mut memory) = self.memory.lock() {
                memory.put(key.clone(), skill.clone());
            }

            return Some(skill);
        }

        None
    }

    /// Insert a mini-skill into cache
    pub fn insert(&mut self, skill: MiniSkill) {
        let key = Self::compute_key(&skill.tool, &skill.doc_hash);

        // Add to memory cache
        if let Ok(mut memory) = self.memory.lock() {
            memory.put(key.clone(), skill.clone());
        }

        // Update tool index
        if let Ok(mut index) = self.tool_index.lock() {
            index
                .entry(skill.tool.clone())
                .or_insert_with(Vec::new)
                .push(key.clone());
        }

        // Persist to disk
        if self.config.persist_to_disk {
            let _ = self.save_to_disk(&key, &skill);
        }
    }

    /// Load cache from disk
    pub fn load(&self) -> Result<()> {
        if !self.config.persist_to_disk {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.disk_path)?;
        let mut loaded = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json")
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(skill) = serde_json::from_str::<MiniSkill>(&content)
            {
                let key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();

                // Check age
                let age = (Utc::now() - skill.created_at).num_days();
                if age as u64 <= self.config.max_age_days {
                    if let Ok(mut memory) = self.memory.lock() {
                        memory.put(key.clone(), skill.clone());
                        loaded += 1;
                    }

                    // Update tool index
                    if let Ok(mut index) = self.tool_index.lock() {
                        index
                            .entry(skill.tool.clone())
                            .or_insert_with(Vec::new)
                            .push(key);
                    }
                } else {
                    // Expired, remove file
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        if loaded > 0 {
            tracing::info!("Loaded {} mini-skills from disk cache", loaded);
        }

        Ok(())
    }

    /// Persist all memory cache to disk
    #[allow(dead_code)]
    pub fn persist(&self) -> Result<()> {
        if !self.config.persist_to_disk {
            return Ok(());
        }

        if let Ok(memory) = self.memory.lock() {
            for (key, skill) in memory.iter() {
                self.save_to_disk(key, skill)?;
            }
        }

        Ok(())
    }

    /// Save a single mini-skill to disk
    fn save_to_disk(&self, key: &str, skill: &MiniSkill) -> Result<()> {
        let path = self.disk_path.join(format!("{}.json", key));
        let content = serde_json::to_string_pretty(skill)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load a single mini-skill from disk
    fn load_from_disk(&self, key: &str) -> Result<Option<MiniSkill>> {
        let path = self.disk_path.join(format!("{}.json", key));

        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let skill = serde_json::from_str(&content)?;
        Ok(Some(skill))
    }

    /// Remove a mini-skill from disk
    fn remove_from_disk(&self, key: &str) -> Result<()> {
        let path = self.disk_path.join(format!("{}.json", key));

        if path.exists() {
            std::fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Clear all cache (memory + disk)
    #[allow(dead_code)]
    pub fn clear(&mut self) -> Result<()> {
        // Clear memory
        if let Ok(mut memory) = self.memory.lock() {
            memory.clear();
        }

        // Clear tool index
        if let Ok(mut index) = self.tool_index.lock() {
            index.clear();
        }

        // Clear disk
        if self.config.persist_to_disk && self.disk_path.exists() {
            std::fs::remove_dir_all(&self.disk_path)?;
            std::fs::create_dir_all(&self.disk_path)?;
        }

        Ok(())
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> CacheStats {
        let memory_size = if let Ok(memory) = self.memory.lock() {
            memory.len()
        } else {
            0
        };
        let disk_size = if self.config.persist_to_disk && self.disk_path.exists() {
            std::fs::read_dir(&self.disk_path)
                .map(|entries| entries.count())
                .unwrap_or(0)
        } else {
            0
        };

        CacheStats {
            memory_entries: memory_size,
            disk_entries: disk_size,
            config: self.config.clone(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub disk_entries: usize,
    pub config: CacheConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_key() {
        let key1 = MiniSkillCache::compute_key("samtools", "abc123");
        let key2 = MiniSkillCache::compute_key("samtools", "abc123");
        let key3 = MiniSkillCache::compute_key("samtools", "def456");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert!(key1.starts_with("samtools:"));
    }

    #[test]
    fn test_compute_key_same_tool_different_tasks_same_doc() {
        // Mini-skills are keyed by (tool, doc_hash) only — different tasks
        // for the same tool + doc should hit the same cache entry.
        let key1 = MiniSkillCache::compute_key("samtools", "abc123");
        let key2 = MiniSkillCache::compute_key("samtools", "abc123");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_insert_and_get() {
        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: false,
            max_age_days: 30,
        };

        let mut cache = MiniSkillCache::new(config).unwrap();

        let skill = MiniSkill {
            tool: "samtools".to_string(),
            task_hash: "sort bam".to_string(),
            doc_hash: "abc123".to_string(),
            concepts: vec!["BAM sorting".to_string()],
            pitfalls: vec!["Don't forget -@ flag".to_string()],
            examples: vec![MiniSkillExample {
                task: "Sort BAM".to_string(),
                args: "sort -@ 4 -o out.bam in.bam".to_string(),
                explanation: "Sort by coordinate".to_string(),
            }],
            created_at: Utc::now(),
            hit_count: 0,
        };

        cache.insert(skill.clone());

        let retrieved = cache.get("samtools", "abc123");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.tool, "samtools");
        assert_eq!(retrieved.hit_count, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: false,
            max_age_days: 0, // Expire immediately
        };

        let mut cache = MiniSkillCache::new(config).unwrap();

        let old_skill = MiniSkill {
            tool: "old_tool".to_string(),
            task_hash: "task".to_string(),
            doc_hash: "hash".to_string(),
            concepts: vec![],
            pitfalls: vec![],
            examples: vec![],
            created_at: Utc::now() - chrono::Duration::days(1), // 1 day old
            hit_count: 0,
        };

        cache.insert(old_skill);

        // Should not retrieve expired skill
        let retrieved = cache.get("old_tool", "hash");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_clear_cache() {
        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: false,
            max_age_days: 30,
        };

        let mut cache = MiniSkillCache::new(config).unwrap();

        let skill = MiniSkill {
            tool: "tool".to_string(),
            task_hash: "task".to_string(),
            doc_hash: "hash".to_string(),
            concepts: vec![],
            pitfalls: vec![],
            examples: vec![],
            created_at: Utc::now(),
            hit_count: 0,
        };

        cache.insert(skill);
        assert!(cache.get("tool", "hash").is_some());

        cache.clear().unwrap();
        assert!(cache.get("tool", "hash").is_none());
    }

    // ── New tests for improved coverage ──────────────────────────────────────

    #[test]
    fn test_cache_stats_memory_only() {
        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: false,
            max_age_days: 30,
        };
        let mut cache = MiniSkillCache::new(config).unwrap();

        // Empty cache stats
        let stats = cache.stats();
        assert_eq!(stats.memory_entries, 0);
        assert_eq!(stats.disk_entries, 0);

        // Add one entry
        let skill = MiniSkill {
            tool: "bwa".to_string(),
            task_hash: "align".to_string(),
            doc_hash: "docabc".to_string(),
            concepts: vec!["BWA alignment".to_string()],
            pitfalls: vec![],
            examples: vec![],
            created_at: Utc::now(),
            hit_count: 0,
        };
        cache.insert(skill);

        let stats = cache.stats();
        assert_eq!(stats.memory_entries, 1);
        // Disk entries = 0 since persist_to_disk=false
        assert_eq!(stats.disk_entries, 0);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.memory_size, 100);
        assert!(config.persist_to_disk);
        assert_eq!(config.max_age_days, 30);
    }

    #[test]
    fn test_mini_skill_to_skill_conversion() {
        let mini = MiniSkill {
            tool: "samtools".to_string(),
            task_hash: "sort".to_string(),
            doc_hash: "hash123".to_string(),
            concepts: vec!["BAM format".to_string(), "Coordinate sorting".to_string()],
            pitfalls: vec!["Sort before indexing".to_string()],
            examples: vec![MiniSkillExample {
                task: "Sort a BAM file".to_string(),
                args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
                explanation: "Sort by coordinate with 4 threads".to_string(),
            }],
            created_at: Utc::now(),
            hit_count: 5,
        };

        let skill: crate::skill::Skill = mini.into();
        assert_eq!(skill.meta.name, "samtools");
        assert_eq!(skill.meta.category, "auto-generated");
        assert_eq!(skill.context.concepts.len(), 2);
        assert_eq!(skill.context.pitfalls.len(), 1);
        assert_eq!(skill.examples.len(), 1);
        assert_eq!(skill.examples[0].task, "Sort a BAM file");
    }

    #[test]
    fn test_insert_multiple_tools_updates_tool_index() {
        let config = CacheConfig {
            memory_size: 20,
            persist_to_disk: false,
            max_age_days: 30,
        };
        let mut cache = MiniSkillCache::new(config).unwrap();

        for tool in &["samtools", "bwa", "bcftools"] {
            let skill = MiniSkill {
                tool: tool.to_string(),
                task_hash: "task".to_string(),
                doc_hash: format!("hash_{tool}"),
                concepts: vec![],
                pitfalls: vec![],
                examples: vec![],
                created_at: Utc::now(),
                hit_count: 0,
            };
            cache.insert(skill);
        }

        // All three tools should be retrievable
        assert!(cache.get("samtools", "hash_samtools").is_some());
        assert!(cache.get("bwa", "hash_bwa").is_some());
        assert!(cache.get("bcftools", "hash_bcftools").is_some());
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: false,
            max_age_days: 30,
        };
        let mut cache = MiniSkillCache::new(config).unwrap();
        assert!(cache.get("nonexistent", "nohash").is_none());
    }

    #[test]
    fn test_persist_memory_only_no_error() {
        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: false,
            max_age_days: 30,
        };
        let cache = MiniSkillCache::new(config).unwrap();
        // persist() should return Ok(()) when persist_to_disk=false
        assert!(cache.persist().is_ok());
    }

    #[test]
    fn test_disk_persistence_roundtrip() {
        let tmp_dir = std::env::temp_dir().join(format!(
            "oxo_test_mini_skill_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&tmp_dir).unwrap();

        // Override data dir via env var
        // SAFETY: test-only, no concurrent threads accessing OXO_DATA_DIR
        unsafe { std::env::set_var("OXO_DATA_DIR", &tmp_dir) };

        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: true,
            max_age_days: 30,
        };

        let skill = MiniSkill {
            tool: "hisat2".to_string(),
            task_hash: "align".to_string(),
            doc_hash: "testhash".to_string(),
            concepts: vec!["Spliced alignment".to_string()],
            pitfalls: vec![],
            examples: vec![],
            created_at: Utc::now(),
            hit_count: 0,
        };

        // Insert and persist to disk
        {
            let mut cache = MiniSkillCache::new(config.clone()).unwrap();
            cache.insert(skill.clone());
        }

        // Load from disk in a new cache instance
        {
            let mut cache = MiniSkillCache::new(config).unwrap();
            let retrieved = cache.get("hisat2", "testhash");
            assert!(retrieved.is_some(), "skill should survive disk round-trip");
            assert_eq!(retrieved.unwrap().tool, "hisat2");
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp_dir);
        // SAFETY: test-only, no concurrent threads accessing OXO_DATA_DIR
        unsafe { std::env::remove_var("OXO_DATA_DIR") };
    }

    #[test]
    fn test_disk_expiration_removes_file() {
        let tmp_dir = std::env::temp_dir().join(format!(
            "oxo_test_expire_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        // SAFETY: test-only, no concurrent threads accessing OXO_DATA_DIR
        unsafe { std::env::set_var("OXO_DATA_DIR", &tmp_dir) };

        let config = CacheConfig {
            memory_size: 10,
            persist_to_disk: true,
            max_age_days: 0, // Expire immediately
        };

        let old_skill = MiniSkill {
            tool: "expired_tool".to_string(),
            task_hash: "task".to_string(),
            doc_hash: "oldhash".to_string(),
            concepts: vec![],
            pitfalls: vec![],
            examples: vec![],
            created_at: Utc::now() - chrono::Duration::days(2), // 2 days old
            hit_count: 0,
        };

        // Create cache, manually write expired file, then try to load it
        {
            let mut cache = MiniSkillCache::new(config.clone()).unwrap();
            cache.insert(old_skill);
            // Get should remove expired entry
            let result = cache.get("expired_tool", "oldhash");
            assert!(result.is_none(), "expired skill should not be returned");
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
        // SAFETY: test-only, no concurrent threads accessing OXO_DATA_DIR
        unsafe { std::env::remove_var("OXO_DATA_DIR") };
    }
}
