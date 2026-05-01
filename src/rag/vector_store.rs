//! Vector Store Module
//!
//! In-memory vector storage with similarity search capabilities.
//! Supports CRUD operations and nearest neighbor search.

#![allow(dead_code)]

use crate::rag::embedding::cosine_similarity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single entry in the vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique identifier
    pub id: String,
    /// Original text
    pub text: String,
    /// Embedding vector
    pub embedding: Vec<f32>,
    /// Metadata (tool name, subcommand, etc.)
    pub metadata: HashMap<String, String>,
    /// Timestamp
    pub timestamp: u64,
}

impl VectorEntry {
    /// Create a new vector entry
    pub fn new(id: impl Into<String>, text: impl Into<String>, embedding: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            embedding,
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Found entry
    pub entry: VectorEntry,
    /// Similarity score (0-1)
    pub score: f32,
    /// Distance (optional)
    pub distance: Option<f32>,
}

/// In-memory vector store
#[derive(Debug, Clone, Default)]
pub struct VectorStore {
    /// Storage of entries
    entries: HashMap<String, VectorEntry>,
    /// Embedding dimension
    dimension: usize,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new(dimension: usize) -> Self {
        Self {
            entries: HashMap::new(),
            dimension,
        }
    }

    /// Add an entry to the store
    pub fn add(&mut self, entry: VectorEntry) -> Result<(), String> {
        if entry.embedding.len() != self.dimension {
            return Err(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                entry.embedding.len()
            ));
        }

        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Add multiple entries
    pub fn add_batch(&mut self, entries: Vec<VectorEntry>) -> Vec<Result<(), String>> {
        entries.into_iter().map(|e| self.add(e)).collect()
    }

    /// Get entry by ID
    pub fn get(&self, id: &str) -> Option<&VectorEntry> {
        self.entries.get(id)
    }

    /// Remove entry by ID
    pub fn remove(&mut self, id: &str) -> Option<VectorEntry> {
        self.entries.remove(id)
    }

    /// Update an entry
    pub fn update(&mut self, entry: VectorEntry) -> Result<(), String> {
        if !self.entries.contains_key(&entry.id) {
            return Err(format!("Entry {} not found", entry.id));
        }
        self.add(entry)
    }

    /// Search for similar vectors using cosine similarity
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        if query.len() != self.dimension {
            return Vec::new();
        }

        let mut results: Vec<SearchResult> = self
            .entries
            .values()
            .map(|entry| {
                let score = cosine_similarity(query, &entry.embedding);
                SearchResult {
                    entry: entry.clone(),
                    score,
                    distance: Some(1.0 - score),
                }
            })
            .filter(|r| r.score > 0.0)
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Return top_k
        results.into_iter().take(top_k).collect()
    }

    /// Search with metadata filter
    pub fn search_with_filter<F>(&self, query: &[f32], top_k: usize, filter: F) -> Vec<SearchResult>
    where
        F: Fn(&VectorEntry) -> bool,
    {
        if query.len() != self.dimension {
            return Vec::new();
        }

        let mut results: Vec<SearchResult> = self
            .entries
            .values()
            .filter(|entry| filter(entry))
            .map(|entry| {
                let score = cosine_similarity(query, &entry.embedding);
                SearchResult {
                    entry: entry.clone(),
                    score,
                    distance: Some(1.0 - score),
                }
            })
            .filter(|r| r.score > 0.0)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.into_iter().take(top_k).collect()
    }

    /// Get all entries
    pub fn all(&self) -> Vec<&VectorEntry> {
        self.entries.values().collect()
    }

    /// Get entries by metadata
    pub fn get_by_metadata(&self, key: &str, value: &str) -> Vec<&VectorEntry> {
        self.entries
            .values()
            .filter(|e| e.metadata.get(key).map(|v| v == value).unwrap_or(false))
            .collect()
    }

    /// Count of entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Save to JSON (for persistence)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        // For simplicity, just serialize entries
        // In production, might want to use more efficient format
        serde_json::to_string(&self.entries)
    }

    /// Load from JSON
    pub fn from_json(json: &str, dimension: usize) -> Result<Self, serde_json::Error> {
        let entries: HashMap<String, VectorEntry> = serde_json::from_str(json)?;
        Ok(Self { entries, dimension })
    }
}

/// Builder for vector store
pub struct VectorStoreBuilder {
    dimension: usize,
    initial_capacity: Option<usize>,
}

impl VectorStoreBuilder {
    /// Create new builder
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            initial_capacity: None,
        }
    }

    /// Set initial capacity
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = Some(capacity);
        self
    }

    /// Build the store
    pub fn build(self) -> VectorStore {
        VectorStore::new(self.dimension)
    }
}

/// Index types for optimized search
#[derive(Debug, Clone)]
pub enum IndexType {
    /// Flat index (brute force)
    Flat,
    /// HNSW index for approximate search
    Hnsw {
        /// Number of neighbors
        m: usize,
        /// EF construction parameter
        ef_construction: usize,
    },
}

/// Advanced vector store with indexing
#[derive(Debug)]
pub struct IndexedVectorStore {
    /// Base store
    store: VectorStore,
    /// Index type
    index_type: IndexType,
}

impl IndexedVectorStore {
    /// Create new indexed store
    pub fn new(dimension: usize, index_type: IndexType) -> Self {
        Self {
            store: VectorStore::new(dimension),
            index_type,
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: VectorEntry) -> Result<(), String> {
        self.store.add(entry)
    }

    /// Search
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        match self.index_type {
            IndexType::Flat => self.store.search(query, top_k),
            IndexType::Hnsw { .. } => {
                // For now, fall back to flat search
                // In production, would use HNSW algorithm
                self.store.search(query, top_k)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: &str, embedding: Vec<f32>) -> VectorEntry {
        VectorEntry::new(id, format!("text {}", id), embedding)
    }

    #[test]
    fn test_add_and_get() {
        let mut store = VectorStore::new(3);
        let entry = create_test_entry("1", vec![1.0, 0.0, 0.0]);

        store.add(entry.clone()).unwrap();
        assert_eq!(store.count(), 1);

        let retrieved = store.get("1").unwrap();
        assert_eq!(retrieved.id, "1");
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut store = VectorStore::new(3);
        let entry = create_test_entry("1", vec![1.0, 0.0]); // Wrong dimension

        assert!(store.add(entry).is_err());
    }

    #[test]
    fn test_search() {
        let mut store = VectorStore::new(3);

        store
            .add(create_test_entry("1", vec![1.0, 0.0, 0.0]))
            .unwrap();
        store
            .add(create_test_entry("2", vec![0.0, 1.0, 0.0]))
            .unwrap();
        store
            .add(create_test_entry("3", vec![0.9, 0.1, 0.0]))
            .unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let results = store.search(&query, 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entry.id, "1"); // Exact match
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_metadata_filter() {
        let mut store = VectorStore::new(3);

        let entry1 = create_test_entry("1", vec![1.0, 0.0, 0.0])
            .with_metadata("tool", "samtools")
            .with_metadata("subcommand", "sort");

        let entry2 = create_test_entry("2", vec![0.0, 1.0, 0.0])
            .with_metadata("tool", "samtools")
            .with_metadata("subcommand", "view");

        store.add(entry1).unwrap();
        store.add(entry2).unwrap();

        let samtools = store.get_by_metadata("tool", "samtools");
        assert_eq!(samtools.len(), 2);

        let sort = store.get_by_metadata("subcommand", "sort");
        assert_eq!(sort.len(), 1);
    }
}
