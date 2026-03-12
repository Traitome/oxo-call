//! Simulate reference genome FASTA sequences.
//!
//! Generates small synthetic chromosomes suitable for testing alignment and
//! variant-calling workflows without requiring a full genome download.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::{self, Write};
use std::path::Path;

/// Parameters for simulating a reference FASTA.
#[derive(Debug, Clone)]
pub struct FastaSimParams {
    /// Number of chromosomes / contigs.
    pub n_chroms: usize,
    /// Length of each chromosome (bp).
    pub chrom_len: usize,
    /// GC content fraction (0.0–1.0). Typical: 0.42.
    pub gc_content: f64,
    /// Width of FASTA sequence lines.
    pub line_width: usize,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl Default for FastaSimParams {
    fn default() -> Self {
        Self {
            n_chroms: 5,
            chrom_len: 50_000,
            gc_content: 0.42,
            line_width: 60,
            seed: 42,
        }
    }
}

/// Write a simulated reference FASTA to the given writer.
pub fn write_fasta<W: Write>(writer: &mut W, params: &FastaSimParams) -> io::Result<()> {
    let mut rng = StdRng::seed_from_u64(params.seed);

    for i in 0..params.n_chroms {
        writeln!(writer, ">chr{}", i + 1)?;
        let mut written = 0;
        while written < params.chrom_len {
            let chunk = (params.line_width).min(params.chrom_len - written);
            let mut line = Vec::with_capacity(chunk);
            for _ in 0..chunk {
                let r: f64 = rng.r#gen();
                let base = if r < params.gc_content / 2.0 {
                    b'G'
                } else if r < params.gc_content {
                    b'C'
                } else if r < params.gc_content + (1.0 - params.gc_content) / 2.0 {
                    b'A'
                } else {
                    b'T'
                };
                line.push(base);
            }
            writer.write_all(&line)?;
            writeln!(writer)?;
            written += chunk;
        }
    }
    Ok(())
}

/// Simulate a reference genome FASTA at the given path.
pub fn simulate_fasta(out_path: &Path, params: &FastaSimParams) -> anyhow::Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::File::create(out_path)?;
    write_fasta(&mut f, params)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    #[test]
    fn test_fasta_chromosome_count() {
        let mut buf = Vec::new();
        let params = FastaSimParams {
            n_chroms: 3,
            chrom_len: 100,
            line_width: 60,
            ..Default::default()
        };
        write_fasta(&mut buf, &params).unwrap();
        let header_count = buf.lines().filter(|l| l.as_ref().map(|s: &String| s.starts_with('>')).unwrap_or(false)).count();
        assert_eq!(header_count, 3);
    }

    #[test]
    fn test_fasta_reproducible() {
        let params = FastaSimParams {
            n_chroms: 2,
            chrom_len: 200,
            seed: 99,
            ..Default::default()
        };
        let mut a = Vec::new();
        write_fasta(&mut a, &params).unwrap();
        let mut b = Vec::new();
        write_fasta(&mut b, &params).unwrap();
        assert_eq!(a, b);
    }
}
