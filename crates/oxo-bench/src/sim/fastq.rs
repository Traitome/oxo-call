//! Simulate paired-end FASTQ reads for benchmarking purposes.
//!
//! The generated reads contain random nucleotides with realistic quality score
//! profiles (Phred+33 encoding), suitable for testing QC tools (fastp, FastQC)
//! and aligners without requiring real sequencing data.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::{self, Write};
use std::path::Path;

/// Parameters for simulating paired-end FASTQ reads.
#[derive(Debug, Clone)]
pub struct FastqSimParams {
    /// Number of read pairs to generate.
    pub n_reads: usize,
    /// Length of each read (bp). Typical: 150 (Illumina short-read).
    pub read_len: usize,
    /// Fraction of low-quality bases to inject (0.0–1.0). Default: 0.02.
    pub error_rate: f64,
    /// Fraction of reads with a synthetic adapter at the 3' end. Default: 0.05.
    pub adapter_rate: f64,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl Default for FastqSimParams {
    fn default() -> Self {
        Self {
            n_reads: 10_000,
            read_len: 150,
            error_rate: 0.02,
            adapter_rate: 0.05,
            seed: 42,
        }
    }
}

const ILLUMINA_ADAPTER: &[u8] = b"AGATCGGAAGAGCACACGTCTGAACTCCAGTCA";
const BASES: &[u8] = b"ACGT";

/// Quality score profile: high quality in the middle, lower at both ends.
fn quality_profile(pos: usize, read_len: usize) -> u8 {
    let frac = pos as f64 / read_len as f64;
    // Simulate typical Illumina quality: ~38 in the middle, drops at ends.
    let q = if frac < 0.05 {
        25.0 + 10.0 * (frac / 0.05)
    } else if frac > 0.85 {
        35.0 - 15.0 * ((frac - 0.85) / 0.15)
    } else {
        37.0
    };
    q as u8
}

/// Generate a single simulated read into the provided buffer.
fn gen_read(
    rng: &mut StdRng,
    read_len: usize,
    error_rate: f64,
    add_adapter: bool,
) -> (Vec<u8>, Vec<u8>) {
    let effective_len = if add_adapter {
        (read_len.saturating_sub(ILLUMINA_ADAPTER.len())).max(50)
    } else {
        read_len
    };

    let mut seq = Vec::with_capacity(read_len);
    let mut qual = Vec::with_capacity(read_len);

    for pos in 0..effective_len {
        let base = BASES[rng.r#gen::<usize>() % 4];
        seq.push(base);
        let q = if rng.r#gen::<f64>() < error_rate {
            // Low-quality base
            (rng.r#gen::<u8>() % 13) + 2
        } else {
            quality_profile(pos, effective_len)
        };
        qual.push(q + 33); // Phred+33 encoding
    }

    if add_adapter {
        let adapter_take = read_len - effective_len;
        seq.extend_from_slice(&ILLUMINA_ADAPTER[..adapter_take.min(ILLUMINA_ADAPTER.len())]);
        // Adapter bases get low quality.
        for _ in 0..adapter_take.min(ILLUMINA_ADAPTER.len()) {
            qual.push(15 + 33);
        }
        // Pad to read_len if needed.
        while seq.len() < read_len {
            seq.push(b'N');
            qual.push(2 + 33);
        }
    }

    seq.truncate(read_len);
    qual.truncate(read_len);
    (seq, qual)
}

/// Write simulated paired-end FASTQ reads to two writers (R1, R2).
///
/// # Errors
/// Returns [`io::Error`] if writing to either writer fails.
pub fn write_paired_fastq<W1: Write, W2: Write>(
    r1_writer: &mut W1,
    r2_writer: &mut W2,
    params: &FastqSimParams,
) -> io::Result<()> {
    let mut rng = StdRng::seed_from_u64(params.seed);

    for i in 0..params.n_reads {
        let add_adapter = rng.r#gen::<f64>() < params.adapter_rate;
        let (seq1, qual1) = gen_read(&mut rng, params.read_len, params.error_rate, add_adapter);
        let (seq2, qual2) = gen_read(&mut rng, params.read_len, params.error_rate, false);

        // R1
        writeln!(r1_writer, "@read_{i}/1")?;
        r1_writer.write_all(&seq1)?;
        writeln!(r1_writer)?;
        writeln!(r1_writer, "+")?;
        r1_writer.write_all(&qual1)?;
        writeln!(r1_writer)?;

        // R2
        writeln!(r2_writer, "@read_{i}/2")?;
        r2_writer.write_all(&seq2)?;
        writeln!(r2_writer)?;
        writeln!(r2_writer, "+")?;
        r2_writer.write_all(&qual2)?;
        writeln!(r2_writer)?;
    }
    Ok(())
}

/// Simulate paired-end FASTQ files at the given output directory.
///
/// Creates `{sample}_R1.fastq` and `{sample}_R2.fastq` files.
///
/// # Errors
/// Returns an error if the directory cannot be created or files cannot be written.
pub fn simulate_paired_fastq(
    out_dir: &Path,
    sample: &str,
    params: &FastqSimParams,
) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    std::fs::create_dir_all(out_dir)?;

    let r1_path = out_dir.join(format!("{sample}_R1.fastq"));
    let r2_path = out_dir.join(format!("{sample}_R2.fastq"));

    let mut r1 = std::fs::File::create(&r1_path)?;
    let mut r2 = std::fs::File::create(&r2_path)?;

    write_paired_fastq(&mut r1, &mut r2, params)?;
    Ok((r1_path, r2_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    #[test]
    fn test_simulate_paired_fastq_line_count() {
        let mut r1 = Vec::new();
        let mut r2 = Vec::new();
        let params = FastqSimParams {
            n_reads: 10,
            read_len: 50,
            ..Default::default()
        };
        write_paired_fastq(&mut r1, &mut r2, &params).unwrap();
        // Each read uses 4 lines in FASTQ format.
        let lines_r1 = r1.lines().count();
        assert_eq!(lines_r1, 40);
        let lines_r2 = r2.lines().count();
        assert_eq!(lines_r2, 40);
    }

    #[test]
    fn test_simulate_paired_fastq_reproducible() {
        let params = FastqSimParams {
            n_reads: 5,
            read_len: 30,
            seed: 123,
            ..Default::default()
        };
        let mut r1a = Vec::new();
        let mut r2a = Vec::new();
        write_paired_fastq(&mut r1a, &mut r2a, &params).unwrap();

        let mut r1b = Vec::new();
        let mut r2b = Vec::new();
        write_paired_fastq(&mut r1b, &mut r2b, &params).unwrap();

        assert_eq!(r1a, r1b);
        assert_eq!(r2a, r2b);
    }

    #[test]
    fn test_read_length_correct() {
        let mut r1 = Vec::new();
        let mut r2 = Vec::new();
        let params = FastqSimParams {
            n_reads: 3,
            read_len: 75,
            adapter_rate: 0.0,
            ..Default::default()
        };
        write_paired_fastq(&mut r1, &mut r2, &params).unwrap();
        // Check that every sequence line is exactly read_len characters.
        let lines: Vec<String> = r1.lines().map(|l| l.unwrap()).collect();
        for chunk in lines.chunks(4) {
            assert_eq!(chunk[1].len(), 75, "sequence length should be 75");
            assert_eq!(chunk[3].len(), 75, "quality length should be 75");
        }
    }
}
