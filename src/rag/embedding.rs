//! Embedding Module
//!
//! Provides text embedding capabilities for vector similarity search.
//! Supports both local (lightweight) and remote (API-based) embedding models.

#![allow(dead_code)]

use std::sync::Arc;

/// Configuration for embedding model
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Model dimension (e.g., 384 for MiniLM, 768 for BERT-base)
    pub dimension: usize,
    /// Max sequence length
    pub max_length: usize,
    /// Whether to normalize embeddings
    pub normalize: bool,
    /// Model type
    pub model_type: EmbeddingModelType,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: 384, // MiniLM-L6-v2 dimension
            max_length: 256,
            normalize: true,
            model_type: EmbeddingModelType::LocalMiniLM,
        }
    }
}

/// Types of embedding models
#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingModelType {
    /// Local MiniLM model (fast, no network)
    LocalMiniLM,
    /// Local GTE model (better quality)
    LocalGTE,
    /// OpenAI API
    OpenAI,
    /// Custom endpoint
    Custom(String),
}

/// Trait for embedding models
pub trait EmbeddingModel: Send + Sync {
    /// Encode text into embedding vector
    fn encode(&self, text: &str) -> Vec<f32>;

    /// Encode multiple texts (batch)
    fn encode_batch(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts.iter().map(|t| self.encode(t)).collect()
    }

    /// Get embedding dimension
    fn dimension(&self) -> usize;
}

/// Local embedding model using rust-bert or fastembed-rs
/// For now, implements a simplified version with keyword-based fallback
pub struct LocalEmbeddingModel {
    config: EmbeddingConfig,
    /// Vocabulary for keyword-based embeddings (fallback)
    vocabulary: std::collections::HashMap<String, usize>,
}

impl LocalEmbeddingModel {
    /// Create new local embedding model
    pub fn new(config: EmbeddingConfig) -> Self {
        let vocabulary = Self::build_vocabulary();
        Self { config, vocabulary }
    }

    /// Build a simple vocabulary for keyword-based embeddings
    fn build_vocabulary() -> std::collections::HashMap<String, usize> {
        let mut vocab = std::collections::HashMap::new();

        // Common bioinformatics operations
        let operations = vec![
            "sort",
            "index",
            "view",
            "filter",
            "merge",
            "intersect",
            "convert",
            "transform",
            "extract",
            "align",
            "map",
            "call",
            "stats",
            "coverage",
            "depth",
            "quality",
            "trim",
            "split",
            "compress",
            "decompress",
            "validate",
            "repair",
            "fix",
        ];

        // File types
        let file_types = vec![
            "bam", "sam", "cram", "fastq", "fasta", "bed", "gtf", "gff", "vcf", "bcf", "gvcf",
            "wig", "bigwig", "bedgraph",
        ];

        // Quality metrics
        let metrics = ["quality", "score", "mapq", "phred", "accuracy", "error"];

        for (i, word) in operations.iter().enumerate() {
            vocab.insert(word.to_string(), i);
        }
        for (i, word) in file_types.iter().enumerate() {
            vocab.insert(word.to_string(), operations.len() + i);
        }
        for (i, word) in metrics.iter().enumerate() {
            vocab.insert(word.to_string(), operations.len() + file_types.len() + i);
        }

        vocab
    }

    /// Create keyword-based embedding (fallback when model not available)
    fn keyword_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.config.dimension];
        let text_lower = text.to_lowercase();

        // Tokenize and count
        for word in text_lower.split_whitespace() {
            if let Some(&idx) = self.vocabulary.get(word)
                && idx < self.config.dimension
            {
                embedding[idx] += 1.0;
            }
        }

        // Normalize
        if self.config.normalize {
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut embedding {
                    *x /= norm;
                }
            }
        }

        embedding
    }
}

impl EmbeddingModel for LocalEmbeddingModel {
    fn encode(&self, text: &str) -> Vec<f32> {
        // For now, use keyword-based embedding
        // In production, this would use actual neural network inference
        self.keyword_embedding(text)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }
}

/// Cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Factory for creating embedding models
pub struct EmbeddingModelFactory;

impl EmbeddingModelFactory {
    /// Create embedding model from config
    pub fn create(config: EmbeddingConfig) -> Arc<dyn EmbeddingModel> {
        match config.model_type {
            EmbeddingModelType::LocalMiniLM | EmbeddingModelType::LocalGTE => {
                Arc::new(LocalEmbeddingModel::new(config))
            }
            _ => {
                // Default to local for now
                Arc::new(LocalEmbeddingModel::new(config))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);

        let c = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_local_embedding() {
        let config = EmbeddingConfig::default();
        let model = LocalEmbeddingModel::new(config);

        let embedding = model.encode("sort bam file");
        assert_eq!(embedding.len(), 384);

        // Should have non-zero values for known words
        assert!(embedding.iter().any(|&x| x > 0.0));
    }

    #[test]
    fn test_similarity_calculation() {
        let config = EmbeddingConfig::default();
        let model = LocalEmbeddingModel::new(config);

        let emb1 = model.encode("sort bam file");
        let emb2 = model.encode("sort bam");
        let emb3 = model.encode("filter fastq");

        let sim12 = cosine_similarity(&emb1, &emb2);
        let sim13 = cosine_similarity(&emb1, &emb3);

        // Similar commands should have higher similarity
        assert!(
            sim12 > sim13,
            "Similar commands should have higher similarity"
        );
    }
}
