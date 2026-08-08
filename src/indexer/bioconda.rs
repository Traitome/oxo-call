//! Bioconda metadata index.
//!
//! Loads `bioconda_tools_metadata.jsonl` and provides O(1) tool name → metadata lookup.
//! Extracts binary names from build scripts and maps aliases.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// A single bioconda recipe entry.
#[derive(Debug, Clone, Deserialize)]
pub struct BiocondaEntry {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub home: String,
    #[serde(default)]
    pub doc_url: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
}

/// In-memory index mapping tool names to bioconda metadata.
///
/// Also builds a reverse alias index so common shortcuts
/// (e.g., "iqtree" → "iqtree2") resolve correctly.
pub struct BiocondaIndex {
    /// tool_name → entry
    entries: HashMap<String, BiocondaEntry>,
    /// alias → canonical tool name
    aliases: HashMap<String, String>,
}

impl BiocondaIndex {
    /// Load the index from the bundled JSONL file.
    pub fn load() -> Result<Self, String> {
        // Search for the data file: cwd, ../data, ../../data
        let candidates = &[
            "data/bioconda_tools_metadata.jsonl",
            "../data/bioconda_tools_metadata.jsonl",
            "../../data/bioconda_tools_metadata.jsonl",
        ];
        let path = candidates
            .iter()
            .find(|p| Path::new(p).exists())
            .ok_or_else(|| {
                "bioconda_tools_metadata.jsonl not found. \
                 Run 'oxo-call docs fetch-new --from-bioconda' to download it."
                    .to_string()
            })?;

        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read {path}: {e}"))?;

        let mut entries: HashMap<String, BiocondaEntry> = HashMap::new();
        let mut aliases: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<BiocondaEntry>(trimmed) {
                // Build alias map: lowercased name → canonical name
                let lower = entry.name.to_lowercase();
                aliases
                    .entry(lower.clone())
                    .or_insert_with(|| entry.name.clone());
                // Also add hyphen/underscore variants
                if entry.name.contains('-') {
                    aliases
                        .entry(entry.name.replace('-', "_"))
                        .or_insert_with(|| entry.name.clone());
                }
                if entry.name.contains('_') {
                    aliases
                        .entry(entry.name.replace('_', "-"))
                        .or_insert_with(|| entry.name.clone());
                }
                entries.entry(entry.name.clone()).or_insert_with(|| entry);
            }
        }

        Ok(Self { entries, aliases })
    }

    /// Look up a tool by name or alias. Returns None if not in bioconda.
    pub fn lookup(&self, name: &str) -> Option<&BiocondaEntry> {
        // Direct lookup
        if let Some(e) = self.entries.get(name) {
            return Some(e);
        }
        // Alias lookup
        let lower = name.to_lowercase();
        if let Some(canon) = self.aliases.get(&lower)
            && let Some(e) = self.entries.get(canon)
        {
            return Some(e);
        }
        // Hyphen/underscore swap
        if name.contains('-') {
            let swapped = name.replace('-', "_");
            if let Some(e) = self.entries.get(&swapped) {
                return Some(e);
            }
        }
        if name.contains('_') {
            let swapped = name.replace('_', "-");
            if let Some(e) = self.entries.get(&swapped) {
                return Some(e);
            }
        }
        None
    }

    /// Get the bioconda entry for a tool, if indexed.
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&BiocondaEntry> {
        self.lookup(name)
    }

    /// Build a documentation string from the bioconda metadata.
    /// This is used as L2 evidence in the prompt.
    pub fn to_doc_string(&self, name: &str) -> Option<String> {
        let e = self.lookup(name)?;
        let mut doc = format!("# {} v{} (bioconda)\n", e.name, e.version);
        if !e.summary.is_empty() {
            doc.push_str(&format!("Summary: {}\n", e.summary));
        }
        if !e.description.is_empty() {
            doc.push_str(&format!(
                "Description: {}\n",
                e.description.chars().take(2000).collect::<String>()
            ));
        }
        if !e.home.is_empty() {
            doc.push_str(&format!("Homepage: {}\n", e.home));
        }
        if !e.doc_url.is_empty() {
            doc.push_str(&format!("Documentation: {}\n", e.doc_url));
        }
        if !e.license.is_empty() {
            doc.push_str(&format!("License: {}\n", e.license));
        }
        Some(doc)
    }

    /// Search for tools matching a keyword query.
    /// Returns (tool_name, relevance_score) sorted by relevance.
    #[allow(dead_code)]
    pub fn search(&self, query: &str) -> Vec<(&str, f64)> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(&str, f64)> = self
            .entries
            .iter()
            .map(|(name, entry)| {
                let name_lower = name.to_lowercase();
                let mut score = 0.0;

                // Exact name match
                if name_lower == query_lower {
                    score += 10.0;
                }
                // Name contains query
                if name_lower.contains(&query_lower) {
                    score += 5.0;
                }

                // Term matching in summary and description
                let searchable = format!(
                    "{} {} {}",
                    name_lower,
                    entry.summary.to_lowercase(),
                    entry.description.to_lowercase()
                );
                for term in &query_terms {
                    if searchable.contains(term) {
                        score += 1.0;
                    }
                }

                (name.as_str(), score)
            })
            .filter(|(_, s)| *s > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    /// Number of indexed tools.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_index() -> BiocondaIndex {
        // Build a minimal in-memory index for testing
        let jsonl = r#"{"name":"samtools","version":"1.17","summary":"SAM tools","home":"http://www.htslib.org/","doc_url":"","description":"Tools for manipulating SAM/BAM/CRAM files.","license":"MIT"}
{"name":"iqtree2","version":"2.2.2.7","summary":"IQ-TREE - phylogenetic inference","home":"http://www.iqtree.org/","doc_url":"http://www.iqtree.org/doc/","description":"Efficient phylogenomic software by maximum likelihood.","license":"GPL-2.0"}
{"name":"humann3","version":"3.9","summary":"HUMAnN 3.0","home":"https://github.com/biobakery/humann","doc_url":"","description":"HUMAnN: The HMP Unified Metabolic Analysis Network.","license":"MIT"}"#;
        let mut entries = HashMap::new();
        let mut aliases = HashMap::new();
        for line in jsonl.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<BiocondaEntry>(line) {
                let lower = entry.name.to_lowercase();
                aliases
                    .entry(lower.clone())
                    .or_insert_with(|| entry.name.clone());
                entries.entry(entry.name.clone()).or_insert_with(|| entry);
            }
        }
        BiocondaIndex { entries, aliases }
    }

    #[test]
    fn test_lookup_direct() {
        let idx = test_index();
        assert!(idx.lookup("samtools").is_some());
        assert_eq!(idx.lookup("samtools").unwrap().version, "1.17");
    }

    #[test]
    fn test_lookup_case_insensitive_alias() {
        let idx = test_index();
        assert!(idx.lookup("Samtools").is_some());
    }

    #[test]
    fn test_lookup_nonexistent() {
        let idx = test_index();
        assert!(idx.lookup("nonexistent_tool").is_none());
    }

    #[test]
    fn test_search_by_keyword() {
        let idx = test_index();
        let results = idx.search("phylogenetic");
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "iqtree2");
    }

    #[test]
    fn test_to_doc_string() {
        let idx = test_index();
        let doc = idx.to_doc_string("samtools").unwrap();
        assert!(doc.contains("SAM tools"));
        assert!(doc.contains("1.17"));
        assert!(doc.contains("bioconda"));
    }

    #[test]
    fn test_len() {
        let idx = test_index();
        assert_eq!(idx.len(), 3);
    }
}
