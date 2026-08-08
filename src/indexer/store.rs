#![allow(dead_code)]
//! Keyword-based document store for L2 evidence retrieval.
//!
//! Stores document chunks with evidence grades and provides
//! TF-IDF weighted keyword search to find relevant documentation
//! for a given task description.
//!
//! Zero ML dependencies — uses simple term frequency with
//! inverse document frequency weighting over the chunk corpus.
//!
//! NB: DocStore is used by library consumers and tests; the binary
//! uses BiocondaIndex directly.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Evidence grade for a document chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceGrade {
    /// Blog posts, tutorials — lowest authority.
    C,
    /// ReadTheDocs / README — good but may be outdated.
    B,
    /// Man page — usually accurate.
    BPlus,
    /// Bioconda recipe + build script — structured, versioned.
    AMinus,
    /// Official tool docs (homepage, manual) — highest authority.
    A,
}

impl EvidenceGrade {
    pub fn label(&self) -> &'static str {
        match self {
            Self::A => "A (Official docs)",
            Self::AMinus => "A- (Bioconda)",
            Self::BPlus => "B+ (Man page)",
            Self::B => "B (ReadTheDocs/README)",
            Self::C => "C (Blog/tutorial)",
        }
    }
}

/// A document chunk with metadata and evidence grade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMeta {
    /// The tool this chunk is about.
    pub tool: String,
    /// Evidence grade.
    pub grade: EvidenceGrade,
    /// Source label (e.g., "bioconda recipe", "homepage").
    pub source: String,
    /// The actual text content.
    pub content: String,
}

/// A search result from the document store.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub chunk: ChunkMeta,
    pub score: f64,
}

/// TF-IDF weighted keyword index over document chunks.
///
/// Builds an inverted index mapping normalized tokens to
/// (chunk_index, term_frequency) pairs, then scores queries
/// using TF-IDF cosine similarity.
pub struct DocStore {
    /// All stored chunks.
    chunks: Vec<ChunkMeta>,
    /// token → Vec<(chunk_idx, term_frequency)>
    inverted_index: HashMap<String, Vec<(usize, f64)>>,
    /// IDF cache: token → inverse document frequency
    idf_cache: HashMap<String, f64>,
    /// Total number of documents (for IDF).
    doc_count: usize,
    /// English stopwords (very common words with low signal).
    stopwords: HashSet<&'static str>,
}

impl DocStore {
    /// Create an empty document store.
    pub fn new() -> Self {
        let stopwords: HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "having", "do", "does", "did", "doing", "will", "would", "could", "should",
            "may", "might", "must", "shall", "can", "need", "dare", "ought", "used", "to", "of",
            "in", "for", "on", "with", "at", "by", "from", "as", "into", "through", "during",
            "before", "after", "above", "below", "between", "out", "off", "over", "under", "again",
            "further", "then", "once", "here", "there", "when", "where", "why", "how", "all",
            "both", "each", "few", "more", "most", "other", "some", "such", "no", "nor", "not",
            "only", "own", "same", "so", "than", "too", "very", "just", "because", "about", "up",
            "it", "its", "and", "but", "or", "if", "while", "that", "this", "these", "those",
            "which", "what", "who", "whom", "whose",
        ]
        .iter()
        .copied()
        .collect();

        Self {
            chunks: Vec::new(),
            inverted_index: HashMap::new(),
            idf_cache: HashMap::new(),
            doc_count: 0,
            stopwords,
        }
    }

    /// Add a document chunk to the store.
    ///
    /// Automatically tokenizes and indexes the content.
    pub fn add(&mut self, chunk: ChunkMeta) {
        let idx = self.chunks.len();
        let tokens = tokenize(&chunk.content, &self.stopwords);

        // Update inverted index with term frequencies
        let tf = term_frequencies(&tokens);
        for (token, freq) in &tf {
            self.inverted_index
                .entry(token.clone())
                .or_default()
                .push((idx, *freq));
        }

        self.chunks.push(chunk);
        self.doc_count += 1;

        // Clear IDF cache since doc count changed
        self.idf_cache.clear();
    }

    /// Add multiple chunks at once.
    pub fn add_bulk(&mut self, chunks: Vec<ChunkMeta>) {
        for chunk in chunks {
            self.add(chunk);
        }
    }

    /// Compute IDF for a token.
    fn idf(&mut self, token: &str) -> f64 {
        if let Some(cached) = self.idf_cache.get(token) {
            return *cached;
        }
        let df = self
            .inverted_index
            .get(token)
            .map(|postings| postings.len())
            .unwrap_or(0) as f64;
        let idf = if df > 0.0 {
            ((1.0 + self.doc_count as f64) / (1.0 + df)).ln() + 1.0
        } else {
            0.0
        };
        self.idf_cache.insert(token.to_string(), idf);
        idf
    }

    /// Query the store with a task description.
    ///
    /// Returns up to `top_k` results sorted by TF-IDF score, with
    /// a minimum score threshold to filter noise.
    pub fn query(&mut self, task: &str, top_k: usize) -> Vec<IndexEntry> {
        let query_tokens = tokenize(task, &self.stopwords);
        let query_tf = term_frequencies(&query_tokens);

        // Compute query vector magnitude
        let query_mag: f64 = query_tf
            .iter()
            .map(|(token, tf)| {
                let idf = self.idf(token);
                (tf * idf).powi(2)
            })
            .sum::<f64>()
            .sqrt();

        if query_mag == 0.0 {
            return Vec::new();
        }

        // Score each document
        let mut scores: Vec<(usize, f64)> = Vec::new();
        let mut doc_scores: HashMap<usize, f64> = HashMap::new();

        for (token, q_tf) in &query_tf {
            let idf = self.idf(token);
            let q_weight = q_tf * idf;

            if let Some(postings) = self.inverted_index.get(token) {
                for (doc_idx, d_tf) in postings {
                    let d_weight = d_tf * idf;
                    *doc_scores.entry(*doc_idx).or_insert(0.0) += q_weight * d_weight;
                }
            }
        }

        // Normalize by document magnitude and collect results
        for (doc_idx, dot_product) in doc_scores {
            // Compute document magnitude (cached would be better, but fine for now)
            let chunk = &self.chunks[doc_idx];
            let doc_tokens = tokenize(&chunk.content, &self.stopwords);
            let doc_tf = term_frequencies(&doc_tokens);
            let doc_mag: f64 = doc_tf
                .iter()
                .map(|(token, tf)| {
                    let idf = self.idf(token);
                    (tf * idf).powi(2)
                })
                .sum::<f64>()
                .sqrt();

            let cosine = if doc_mag > 0.0 && query_mag > 0.0 {
                dot_product / (query_mag * doc_mag)
            } else {
                0.0
            };

            if cosine > 0.05 {
                // minimum cosine threshold
                scores.push((doc_idx, cosine));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);

        scores
            .into_iter()
            .map(|(idx, score)| IndexEntry {
                chunk: self.chunks[idx].clone(),
                score,
            })
            .collect()
    }

    /// Number of indexed chunks.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

impl Default for DocStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Tokenize text: lowercase, split on non-alphanumeric, filter stopwords and short tokens.
fn tokenize(text: &str, stopwords: &HashSet<&str>) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .filter(|t| !stopwords.contains(t))
        .map(|t| t.to_string())
        .collect()
}

/// Compute term frequencies for a token list.
#[allow(dead_code)]
fn term_frequencies(tokens: &[String]) -> HashMap<String, f64> {
    let mut tf: HashMap<String, f64> = HashMap::new();
    let n = tokens.len() as f64;
    for token in tokens {
        *tf.entry(token.clone()).or_insert(0.0) += 1.0;
    }
    // Normalize by document length
    if n > 0.0 {
        for freq in tf.values_mut() {
            *freq /= n;
        }
    }
    tf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_filters_stopwords() {
        let stopwords: HashSet<&str> = ["the", "a", "is", "for"].iter().copied().collect();
        let tokens = tokenize("the samtools is a tool for sorting", &stopwords);
        assert!(tokens.contains(&"samtools".to_string()));
        assert!(tokens.contains(&"tool".to_string()));
        assert!(tokens.contains(&"sorting".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"for".to_string()));
    }

    #[test]
    fn test_add_and_query() {
        let mut store = DocStore::new();
        store.add(ChunkMeta {
            tool: "samtools".into(),
            grade: EvidenceGrade::A,
            source: "bioconda".into(),
            content: "samtools is a tool for working with SAM BAM and CRAM files. sort sorts BAM files by coordinate.".into(),
        });
        store.add(ChunkMeta {
            tool: "samtools".into(),
            grade: EvidenceGrade::B,
            source: "homepage".into(),
            content: "samtools view converts between SAM and BAM formats with optional filtering."
                .into(),
        });
        store.add(ChunkMeta {
            tool: "bcftools".into(),
            grade: EvidenceGrade::A,
            source: "bioconda".into(),
            content: "bcftools is a tool for working with VCF BCF files. call calls variants."
                .into(),
        });

        // Query for BAM sorting
        let results = store.query("sort BAM file by coordinate", 3);
        assert!(!results.is_empty(), "should find samtools sort chunk");
        assert_eq!(results[0].chunk.tool, "samtools");
        assert!(results[0].chunk.content.contains("sort"));

        // Query for VCF
        let results = store.query("call variants from VCF", 3);
        assert!(!results.is_empty(), "should find bcftools chunk");
        assert_eq!(results[0].chunk.tool, "bcftools");
    }

    #[test]
    fn test_empty_query() {
        let mut store = DocStore::new();
        store.add(ChunkMeta {
            tool: "test".into(),
            grade: EvidenceGrade::A,
            source: "test".into(),
            content: "test content".into(),
        });
        let results = store.query("", 3);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evidence_grade_ordering() {
        assert!(EvidenceGrade::A > EvidenceGrade::B);
        assert!(EvidenceGrade::A > EvidenceGrade::C);
        assert!(EvidenceGrade::AMinus > EvidenceGrade::BPlus);
    }
}
