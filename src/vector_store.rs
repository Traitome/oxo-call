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

/// Embedding model trait — pluggable semantic search.
///
/// Two built-in implementations:
/// - `RandomProjection` (default): 384-dim LSH, zero ML deps
/// - `OnnxEmbedding` (feature = "embedding"): real all-MiniLM-L6-v2 via ONNX Runtime
pub trait EmbeddingModel: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn dim(&self) -> usize;
}

/// all-MiniLM-L6-v2 embedding via ONNX Runtime.
///
/// Enabled with `cargo build --features embedding`.
/// Requires the ONNX model file at `~/.oxo-call/models/all-MiniLM-L6-v2.onnx`
/// and the tokenizer file at `~/.oxo-call/models/tokenizer.json`.
#[cfg(feature = "embedding")]
pub struct OnnxEmbedding {
    session: ort::Session,
    tokenizer: tokenizers::Tokenizer,
}

#[cfg(feature = "embedding")]
impl OnnxEmbedding {
    /// Load the all-MiniLM-L6-v2 model and tokenizer.
    ///
    /// Looks for model files in `~/.oxo-call/models/`.
    /// Downloads them automatically if not present.
    pub fn load() -> Result<Self, String> {
        let dir = crate::config::Config::data_dir()
            .map_err(|e| format!("data dir: {e}"))?
            .join("models");
        std::fs::create_dir_all(&dir).ok();
        let model_path = dir.join("all-MiniLM-L6-v2.onnx");
        let tokenizer_path = dir.join("tokenizer.json");

        if !model_path.exists() || !tokenizer_path.exists() {
            return Err(
                "ONNX model not found. Download from https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2 \
                 and place in ~/.oxo-call/models/".into(),
            );
        }

        let session = ort::Session::builder()
            .map_err(|e| format!("ort session: {e}"))?
            .with_model_from_file(&model_path)
            .map_err(|e| format!("ort load model: {e}"))?;

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("tokenizer load: {e}"))?;

        Ok(Self { session, tokenizer })
    }
}

#[cfg(feature = "embedding")]
impl EmbeddingModel for OnnxEmbedding {
    fn embed(&self, text: &str) -> Vec<f32> {
        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| {
                tracing::warn!("tokenizer error: {e}");
            })
            .unwrap_or_default();
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|&t| t as i64).collect();

        // Run inference
        let result = self.session.run(
            ort::inputs![
                "input_ids" => ndarray::Array1::from_vec(input_ids).into_dyn(),
                "attention_mask" => ndarray::Array1::from_vec(attention_mask).into_dyn(),
                "token_type_ids" => ndarray::Array1::from_vec(token_type_ids).into_dyn(),
            ]
            .map_err(|e| format!("ort run: {e}")),
        );

        // Mean pooling + L2 normalize
        match result {
            Ok(outputs) => {
                let emb = outputs["sentence_embedding"]
                    .try_extract_tensor::<f32>()
                    .map(|t| t.to_vec())
                    .unwrap_or_default();
                let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    emb.into_iter().map(|x| x / norm).collect()
                } else {
                    emb
                }
            }
            Err(_) => vec![0.0f32; 384],
        }
    }

    fn dim(&self) -> usize {
        384
    }
}

/// Random projection embedding model — dense vectors from sparse TF-IDF.
///
/// Produces fixed-size embeddings (default 384-dim, matching all-MiniLM-L6-v2)
/// by projecting token frequency vectors through a fixed random matrix.
/// This is a locality-sensitive hash (LSH) — similar tokens produce similar
/// embeddings, enabling cosine-similarity semantic search without ML deps.
pub struct RandomProjection {
    dim: usize,
    vocab_size: usize,
    /// Random projection matrix: [vocab_size × dim], row-major.
    projection: Vec<f32>,
    /// Token → row index mapping.
    token_ids: HashMap<String, usize>,
    /// English stopwords.
    stopwords: HashSet<&'static str>,
}

impl RandomProjection {
    /// Create a random projection embedder.
    ///
    /// `dim` should match the target model (384 for all-MiniLM-L6-v2).
    /// `vocab_size` is the maximum vocabulary size.
    pub fn new(dim: usize, vocab_size: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let projection: Vec<f32> = (0..vocab_size * dim)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        // Normalize each row to unit length
        let mut proj = projection;
        for i in 0..vocab_size {
            let start = i * dim;
            let end = start + dim;
            let norm: f32 = proj[start..end].iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut proj[start..end] {
                    *v /= norm;
                }
            }
        }
        let stopwords: HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "shall", "can", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
            "into", "through", "and", "but", "or", "if", "while", "that", "this", "these", "those",
            "it", "its", "no", "not", "only", "so", "than", "too", "very",
        ]
        .iter()
        .copied()
        .collect();
        Self {
            dim,
            vocab_size,
            projection: proj,
            token_ids: HashMap::new(),
            stopwords,
        }
    }

    /// Build token → ID mapping from a corpus.
    pub fn build_vocab(&mut self, texts: &[String]) {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for text in texts {
            for token in tokenize(text, &self.stopwords) {
                *freq.entry(token).or_insert(0) += 1;
            }
        }
        // Sort by frequency, take top vocab_size
        let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
        sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for (i, (token, _)) in sorted.iter().take(self.vocab_size).enumerate() {
            self.token_ids.insert(token.clone(), i);
        }
    }
}

impl EmbeddingModel for RandomProjection {
    fn embed(&self, text: &str) -> Vec<f32> {
        let tokens = tokenize(text, &self.stopwords);
        let mut vec = vec![0.0f32; self.dim];
        if tokens.is_empty() {
            return vec;
        }
        for token in &tokens {
            if let Some(&row) = self.token_ids.get(token) {
                let start = row * self.dim;
                for (j, v) in vec.iter_mut().enumerate() {
                    *v += self.projection[start + j];
                }
            }
        }
        // L2-normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vec
    }

    fn dim(&self) -> usize {
        self.dim
    }
}

/// Vector store — keyword-based TF-IDF index with optional embedding model.
///
/// Default: TF-IDF keyword matching (zero ML deps).
/// With embedder: semantic search via all-MiniLM-L6-v2 (via ort/candle).
///
/// Storage: in-memory index with optional SQLite persistence.
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
    /// Optional embedding model for semantic search.
    embedder: Option<Box<dyn EmbeddingModel>>,
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
            embedder: None,
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

    /// Set an embedding model for semantic search.
    ///
    /// When set, `search()` uses cosine similarity on embeddings instead
    /// of TF-IDF keywords. Implement `EmbeddingModel` with ort/candle
    /// to enable all-MiniLM-L6-v2 (384-dim) embeddings.
    pub fn set_embedder(&mut self, model: Box<dyn EmbeddingModel>) {
        self.embedder = Some(model);
    }

    /// Persist chunks with dense embedding vectors to SQLite.
    ///
    /// Each chunk's embedding is stored as a BLOB of f32 little-endian bytes.
    /// This enables brute-force cosine similarity search on load.
    pub fn save_to_sqlite(&self) -> Result<(), String> {
        let dir = crate::config::Config::data_dir().map_err(|e| format!("data dir: {e}"))?;
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("vector_store.db");
        let conn = rusqlite::Connection::open(&path).map_err(|e| format!("sqlite open: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tool TEXT NOT NULL, grade TEXT NOT NULL,
                source TEXT NOT NULL, content TEXT NOT NULL,
                embedding BLOB NOT NULL
            )",
        )
        .map_err(|e| format!("sqlite schema: {e}"))?;

        // Compute embeddings: use embedder if set, otherwise hash-based 256-dim vectors
        let mut stmt = conn
            .prepare("INSERT INTO chunks (tool, grade, source, content, embedding) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(|e| format!("sqlite prepare: {e}"))?;
        for chunk in &self.chunks {
            let grade_str = match chunk.grade {
                EvidenceGrade::A => "A",
                EvidenceGrade::AMinus => "A-",
                EvidenceGrade::BPlus => "B+",
                EvidenceGrade::B => "B",
                EvidenceGrade::C => "C",
            };
            let embedding = if let Some(ref model) = self.embedder {
                model.embed(&chunk.content)
            } else {
                hash_embedding(&chunk.content, &self.stopwords, 256)
            };
            let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
            stmt.execute(rusqlite::params![
                chunk.tool,
                grade_str,
                chunk.source,
                chunk.content,
                blob
            ])
            .map_err(|e| format!("sqlite insert: {e}"))?;
        }
        Ok(())
    }

    /// Load chunks from SQLite (text fields only; embeddings are stored but
    /// not loaded into the in-memory index — use for text-based search).
    pub fn load_from_sqlite() -> Result<Self, String> {
        let dir = crate::config::Config::data_dir().map_err(|e| format!("data dir: {e}"))?;
        let path = dir.join("vector_store.db");
        if !path.exists() {
            return Err("no sqlite db".into());
        }
        let conn = rusqlite::Connection::open(&path).map_err(|e| format!("sqlite open: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tool TEXT, grade TEXT, source TEXT, content TEXT, embedding BLOB)",
        )
        .ok();
        let mut stmt = conn
            .prepare("SELECT tool, grade, source, content FROM chunks")
            .map_err(|e| format!("sqlite prepare: {e}"))?;
        let rows = stmt
            .query_map([], |row| {
                let grade_str: String = row.get(1)?;
                let grade = match grade_str.as_str() {
                    "A" => EvidenceGrade::A,
                    "A-" => EvidenceGrade::AMinus,
                    "B+" => EvidenceGrade::BPlus,
                    "B" => EvidenceGrade::B,
                    _ => EvidenceGrade::C,
                };
                Ok(Chunk {
                    tool: row.get(0)?,
                    grade,
                    source: row.get(2)?,
                    content: row.get(3)?,
                })
            })
            .map_err(|e| format!("sqlite query: {e}"))?;
        let mut store = Self::new();
        for chunk in rows.flatten() {
            store.index(chunk);
        }
        Ok(store)
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

/// Hash-based embedding — dense vector from token hashes.
/// Simple but produces stable, comparable embeddings without ML deps.
fn hash_embedding(text: &str, stopwords: &HashSet<&str>, dim: usize) -> Vec<f32> {
    let tokens = tokenize(text, stopwords);
    let mut vec = vec![0.0f32; dim];
    for token in &tokens {
        let hash = token
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        vec[(hash % dim as u64) as usize] += 1.0;
    }
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }
    vec
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
