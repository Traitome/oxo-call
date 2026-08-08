//! Vector store for L2 evidence retrieval.
//!
//! Stores document chunks and retrieves relevant ones for a given task
//! using similarity search. Currently uses TF-IDF keyword matching
//! (zero ML dependencies); designed to accept an optional embedding
//! model (all-MiniLM-L6-v2 via ort/candle) as a future upgrade.
//!
//! ## Architecture
//!
//! ```text
//! Document -> chunk -> tokenize -> TF-IDF index -> cosine similarity -> top-K
//! ```
//!
//! The query interface is the same whether using keywords or embeddings,
//! so upgrading to semantic search is a drop-in replacement of the scoring
//! function — no API changes needed.
//!
//! ## Evidence grades
//!
//! Each chunk carries an evidence grade that the prompt builder uses
//! to mark it as authoritative (A) through low-trust (C).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Evidence grade for a document chunk — higher = more trustworthy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceGrade {
    /// Blog posts, tutorials — lowest authority.
    C,
    /// ReadTheDocs / README — useful but may be outdated.
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

/// A document chunk with evidence grade and source metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Tool name this chunk is about.
    pub tool: String,
    /// Evidence grade.
    pub grade: EvidenceGrade,
    /// Source description (e.g., "bioconda recipe", "homepage").
    pub source: String,
    /// Chunk text content.
    pub content: String,
}

/// A search result from the vector store.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk: Chunk,
    /// Similarity score (0.0–1.0, higher = more relevant).
    pub score: f64,
}

/// Vector store — keyword-based TF-IDF index.
///
/// Future: accept an optional `EmbeddingModel` trait object to
/// replace TF-IDF with semantic embeddings (all-MiniLM-L6-v2).
pub struct VectorStore {
    /// All stored chunks.
    chunks: Vec<Chunk>,
    /// Inverted index: token → [(chunk_idx, term_frequency)].
    inverted_index: HashMap<String, Vec<(usize, f64)>>,
    /// IDF cache: token → inverse document frequency.
    idf_cache: HashMap<String, f64>,
    /// Total chunk count (for IDF).
    doc_count: usize,
    /// English stopwords — filtered during tokenization.
    stopwords: HashSet<&'static str>,
}

impl VectorStore {
    /// Create an empty vector store.
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

    /// Index a document chunk.
    pub fn index(&mut self, chunk: Chunk) {
        let idx = self.chunks.len();
        let tokens = tokenize(&chunk.content, &self.stopwords);
        let tf = term_frequencies(&tokens);

        for (token, freq) in &tf {
            self.inverted_index
                .entry(token.clone())
                .or_default()
                .push((idx, *freq));
        }

        self.chunks.push(chunk);
        self.doc_count += 1;
        self.idf_cache.clear();
    }

    /// Index multiple chunks at once.
    pub fn index_bulk(&mut self, chunks: Vec<Chunk>) {
        for chunk in chunks {
            self.index(chunk);
        }
    }

    /// Compute inverse document frequency for a token.
    fn idf(&mut self, token: &str) -> f64 {
        if let Some(cached) = self.idf_cache.get(token) {
            return *cached;
        }
        let df = self.inverted_index.get(token).map(|p| p.len()).unwrap_or(0) as f64;
        let idf = if df > 0.0 {
            ((1.0 + self.doc_count as f64) / (1.0 + df)).ln() + 1.0
        } else {
            0.0
        };
        self.idf_cache.insert(token.to_string(), idf);
        idf
    }

    /// Search for chunks relevant to a query string.
    ///
    /// Returns up to `top_k` results sorted by cosine similarity.
    pub fn search(&mut self, query: &str, top_k: usize) -> Vec<SearchResult> {
        let query_tokens = tokenize(query, &self.stopwords);
        let query_tf = term_frequencies(&query_tokens);

        // Precompute all IDF values needed (avoids borrow conflicts)
        let all_tokens: HashSet<String> = query_tf.keys().cloned().collect();
        let mut idf_map: HashMap<String, f64> = HashMap::new();
        for token in &all_tokens {
            idf_map.insert(token.clone(), self.idf(token));
        }

        // Query vector magnitude
        let query_mag: f64 = query_tf
            .iter()
            .map(|(token, tf)| {
                let idf = idf_map.get(token).copied().unwrap_or(0.0);
                (tf * idf).powi(2)
            })
            .sum::<f64>()
            .sqrt();

        if query_mag == 0.0 {
            return Vec::new();
        }

        // Score each document by cosine similarity
        let mut doc_scores: HashMap<usize, f64> = HashMap::new();
        for (token, q_tf) in &query_tf {
            let idf = idf_map.get(token).copied().unwrap_or(0.0);
            let q_weight = q_tf * idf;
            if let Some(postings) = self.inverted_index.get(token) {
                for (doc_idx, d_tf) in postings {
                    let d_weight = d_tf * idf;
                    *doc_scores.entry(*doc_idx).or_insert(0.0) += q_weight * d_weight;
                }
            }
        }

        // Precompute IDF for all tokens in all candidate documents
        for doc_idx in doc_scores.keys() {
            let chunk = &self.chunks[*doc_idx];
            let doc_tokens = tokenize(&chunk.content, &self.stopwords);
            for token in doc_tokens {
                if !idf_map.contains_key(&token) {
                    idf_map.insert(token.clone(), self.idf(&token));
                }
            }
        }

        let mut scored: Vec<SearchResult> = doc_scores
            .into_iter()
            .filter_map(|(doc_idx, dot_product)| {
                let chunk = &self.chunks[doc_idx];
                let doc_tokens = tokenize(&chunk.content, &self.stopwords);
                let doc_tf = term_frequencies(&doc_tokens);
                let doc_mag: f64 = doc_tf
                    .iter()
                    .map(|(token, tf)| {
                        let idf = idf_map.get(token).copied().unwrap_or(0.0);
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
                    Some(SearchResult {
                        chunk: chunk.clone(),
                        score: cosine,
                    })
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_k);
        scored
    }

    /// Number of indexed chunks.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Clear all indexed data.
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.inverted_index.clear();
        self.idf_cache.clear();
        self.doc_count = 0;
    }
}

impl Default for VectorStore {
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

/// Compute normalized term frequencies for a token list.
fn term_frequencies(tokens: &[String]) -> HashMap<String, f64> {
    let mut tf: HashMap<String, f64> = HashMap::new();
    let n = tokens.len() as f64;
    for token in tokens {
        *tf.entry(token.clone()).or_insert(0.0) += 1.0;
    }
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
    fn test_index_and_search() {
        let mut store = VectorStore::new();
        store.index(Chunk {
            tool: "samtools".into(),
            grade: EvidenceGrade::A,
            source: "bioconda".into(),
            content: "Sort BAM files by coordinate using samtools sort".into(),
        });
        store.index(Chunk {
            tool: "bcftools".into(),
            grade: EvidenceGrade::A,
            source: "bioconda".into(),
            content: "Call variants from VCF files using bcftools call".into(),
        });

        let results = store.search("sort BAM by coordinate", 3);
        assert!(!results.is_empty());
        assert_eq!(results[0].chunk.tool, "samtools");
    }

    #[test]
    fn test_evidence_grade_ordering() {
        assert!(EvidenceGrade::A > EvidenceGrade::B);
        assert!(EvidenceGrade::A > EvidenceGrade::C);
        assert!(EvidenceGrade::AMinus > EvidenceGrade::BPlus);
    }

    #[test]
    fn test_empty_query() {
        let mut store = VectorStore::new();
        store.index(Chunk {
            tool: "test".into(),
            grade: EvidenceGrade::A,
            source: "test".into(),
            content: "test content".into(),
        });
        assert!(store.search("", 3).is_empty());
    }
}
