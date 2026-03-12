//! Omics scenario definitions and multi-sample data simulators.
//!
//! Each [`OmicsScenario`] encapsulates a specific experimental design commonly
//! used in bioinformatics benchmarks (RNA-seq, WGS, ATAC-seq, scRNA-seq, etc.)
//! and provides a method to generate the corresponding synthetic input data.

use crate::sim::fastq::{FastqSimParams, simulate_paired_fastq};
use std::path::{Path, PathBuf};

/// Describes a bioinformatics experimental scenario used for benchmarking.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OmicsScenario {
    /// Unique scenario identifier (e.g. "rnaseq_3s_pe150").
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Assay type: "rnaseq", "wgs", "atacseq", "metagenomics", "scrnaseq", "amplicon16s", etc.
    pub assay: String,
    /// Sample names.
    pub samples: Vec<String>,
    /// Read length (bp).
    pub read_len: usize,
    /// Number of read pairs per sample.
    pub reads_per_sample: usize,
    /// Simulated error rate (fraction of bases).
    pub error_rate: f64,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl OmicsScenario {
    /// Create a standard RNA-seq scenario with `n` samples.
    pub fn rnaseq(n_samples: usize) -> Self {
        Self {
            id: format!("rnaseq_{n_samples}s_pe150"),
            description: format!("Bulk RNA-seq with {n_samples} paired-end samples (150 bp)"),
            assay: "rnaseq".to_string(),
            samples: (1..=n_samples).map(|i| format!("sample{i:02}")).collect(),
            read_len: 150,
            reads_per_sample: 5_000,
            error_rate: 0.01,
            seed: 42,
        }
    }

    /// Create a WGS scenario with `n` samples.
    pub fn wgs(n_samples: usize) -> Self {
        Self {
            id: format!("wgs_{n_samples}s_pe150"),
            description: format!(
                "Whole-genome sequencing with {n_samples} paired-end samples (150 bp)"
            ),
            assay: "wgs".to_string(),
            samples: (1..=n_samples).map(|i| format!("sample{i:02}")).collect(),
            read_len: 150,
            reads_per_sample: 50_000,
            error_rate: 0.005,
            seed: 43,
        }
    }

    /// Create an ATAC-seq scenario with `n` samples.
    pub fn atacseq(n_samples: usize) -> Self {
        Self {
            id: format!("atacseq_{n_samples}s_pe50"),
            description: format!("ATAC-seq with {n_samples} paired-end samples (50 bp)"),
            assay: "atacseq".to_string(),
            samples: (1..=n_samples).map(|i| format!("sample{i:02}")).collect(),
            read_len: 50,
            reads_per_sample: 20_000,
            error_rate: 0.01,
            seed: 44,
        }
    }

    /// Create a metagenomics shotgun scenario with `n` samples.
    pub fn metagenomics(n_samples: usize) -> Self {
        Self {
            id: format!("metagenomics_{n_samples}s_pe150"),
            description: format!(
                "Shotgun metagenomics with {n_samples} paired-end samples (150 bp)"
            ),
            assay: "metagenomics".to_string(),
            samples: (1..=n_samples).map(|i| format!("sample{i:02}")).collect(),
            read_len: 150,
            reads_per_sample: 10_000,
            error_rate: 0.015,
            seed: 45,
        }
    }

    /// Create a ChIP-seq scenario with `n` IP samples.
    pub fn chipseq(n_samples: usize) -> Self {
        Self {
            id: format!("chipseq_{n_samples}s_pe75"),
            description: format!("ChIP-seq with {n_samples} paired-end samples (75 bp)"),
            assay: "chipseq".to_string(),
            samples: (1..=n_samples).map(|i| format!("H3K27ac_rep{i}")).collect(),
            read_len: 75,
            reads_per_sample: 15_000,
            error_rate: 0.01,
            seed: 46,
        }
    }

    /// Create a WGBS methylation scenario with `n` samples.
    pub fn methylseq(n_samples: usize) -> Self {
        Self {
            id: format!("methylseq_{n_samples}s_pe150"),
            description: format!("WGBS methylation with {n_samples} paired-end samples (150 bp)"),
            assay: "methylseq".to_string(),
            samples: (1..=n_samples).map(|i| format!("sample{i:02}")).collect(),
            read_len: 150,
            reads_per_sample: 30_000,
            error_rate: 0.01,
            seed: 47,
        }
    }

    /// Create a scRNA-seq 10x Chromium scenario with `n` samples.
    pub fn scrnaseq(n_samples: usize) -> Self {
        Self {
            id: format!("scrnaseq_{n_samples}s_10xv3"),
            description: format!("scRNA-seq 10x Chromium v3 with {n_samples} libraries"),
            assay: "scrnaseq".to_string(),
            samples: (1..=n_samples).map(|i| format!("lib{i:02}")).collect(),
            read_len: 150,
            reads_per_sample: 25_000,
            error_rate: 0.01,
            seed: 48,
        }
    }

    /// Create a 16S amplicon scenario with `n` samples.
    pub fn amplicon16s(n_samples: usize) -> Self {
        Self {
            id: format!("16s_{n_samples}s_pe250"),
            description: format!("16S V4 amplicon with {n_samples} paired-end samples (250 bp)"),
            assay: "amplicon16s".to_string(),
            samples: (1..=n_samples).map(|i| format!("sample{i:02}")).collect(),
            read_len: 250,
            reads_per_sample: 8_000,
            error_rate: 0.02,
            seed: 49,
        }
    }
}

/// Simulate all input files for a given [`OmicsScenario`] into `out_dir/data/`.
///
/// Returns a list of generated file pairs `(R1_path, R2_path)`.
pub fn simulate_scenario(
    scenario: &OmicsScenario,
    out_dir: &Path,
) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
    let data_dir = out_dir.join("data");
    std::fs::create_dir_all(&data_dir)?;

    let mut results = Vec::new();
    for (i, sample) in scenario.samples.iter().enumerate() {
        // Stagger seeds so samples are different but still reproducible.
        let params = FastqSimParams {
            n_reads: scenario.reads_per_sample,
            read_len: scenario.read_len,
            error_rate: scenario.error_rate,
            adapter_rate: 0.05,
            seed: scenario.seed + i as u64,
        };
        let (r1, r2) = simulate_paired_fastq(&data_dir, sample, &params)?;
        results.push((r1, r2));
    }
    Ok(results)
}

/// Return the set of canonical benchmark scenarios used for oxo-call evaluation.
///
/// These cover all major omics assay types and span a range of sample counts and
/// read lengths, mirroring realistic experimental designs.
pub fn canonical_scenarios() -> Vec<OmicsScenario> {
    vec![
        OmicsScenario::rnaseq(3),
        OmicsScenario::rnaseq(10),
        OmicsScenario::wgs(2),
        OmicsScenario::atacseq(3),
        OmicsScenario::metagenomics(4),
        OmicsScenario::chipseq(3),
        OmicsScenario::methylseq(2),
        OmicsScenario::scrnaseq(2),
        OmicsScenario::amplicon16s(6),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_simulate_rnaseq_scenario() {
        let tmp = TempDir::new().unwrap();
        let scenario = OmicsScenario::rnaseq(2);
        let files = simulate_scenario(&scenario, tmp.path()).unwrap();
        assert_eq!(files.len(), 2);
        for (r1, r2) in &files {
            assert!(r1.exists(), "R1 file should exist: {}", r1.display());
            assert!(r2.exists(), "R2 file should exist: {}", r2.display());
        }
    }

    #[test]
    fn test_canonical_scenarios_count() {
        let scenarios = canonical_scenarios();
        assert!(!scenarios.is_empty());
    }

    #[test]
    fn test_scenario_ids_unique() {
        let scenarios = canonical_scenarios();
        let ids: std::collections::HashSet<&str> =
            scenarios.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids.len(), scenarios.len(), "Scenario IDs must be unique");
    }
}
