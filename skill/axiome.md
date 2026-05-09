---
name: axiome
category: Bioinformatics-SequenceAnalysis
description: A command-line tool for exploratory sequence analysis and automated hypothesis generation from genomic datasets.
tags:
  - sequence-analysis
  - genomics
  - data-exploration
  - variant-discovery
author: AI-Generated
source_url: https://github.com/bioinformatics-tools/axiome
---

## Concepts

- **Input Formats**: Axiome accepts FASTA (.fa, .fasta), FASTQ (.fq, .fastq), and SAM/BAM alignment files as primary inputs. Multi-file input is supported via glob patterns (e.g., sample_*.fq) or explicit file lists passed with the `--inputs` flag.
- **Output Model**: Analysis results are written to a project directory specified with `--output-dir`. Each run generates three files: `results.tsv` (main findings table), `summary.json` (metadata and statistics), and `log.txt` (verbose run log). The `--format` flag controls additional export formats (CSV, JSON, or HTML report).
- **Reference Genome Handling**: When `--reference` is provided, Axiome builds an in-memory index for accelerated queries. Without a reference, the tool performs de novo clustering on input sequences. Index files are cached in `~/.axiome/cache/` and reused across runs.
- **Execution Modes**: The `--mode` flag selects between `explore` (statistical summaries and anomaly detection), `compare` (pairwise or multi-sample analysis), and `predict` (machine learning-based variant scoring). Each mode has distinct default parameters.
- **Memory Management**: Axiome streams large files to avoid loading entire datasets into memory. The `--chunk-size` parameter (default: 10,000 sequences) controls the streaming window. Increasing this value improves speed at the cost of RAM usage.

## Pitfalls

- **Omitting the `--output-dir` flag**: Results are written to the current working directory by default, which can overwrite existing files or scatter outputs unpredictably. Always specify an explicit output directory to maintain organized results.
- **Using FASTQ inputs with the `explore` mode without `--quality-cutoff`**: Poor-quality reads inflate anomaly detection rates, producing false positives. Set `--quality-cutoff` to at least 20 for typical Illumina data.
- **Forgetting to build a reference index for `--mode compare`**: The tool performs full sequence alignment at runtime if no index exists, increasing runtime by 5–10x. Pre-build indices using `axiome-index` for datasets analyzed repeatedly.
- **Passing compressed archives directly**: Axiome cannot read .gz, .bz2, or .zip archives without decompression. Ungzip input files before processing, or use the companion `axiome-unpack` utility.
- **Misinterpreting `results.tsv` column headers**: The `score` column values are log-scaled by default (base 2). Multiply by `log(2)` to convert to natural log scale if comparing with other tools that report linear scores.

## Examples

### Compute summary statistics for a single FASTQ file
**Args:** `sample_001.fq --mode explore --output-dir ./analysis_run1`
**Explanation:** Runs exploratory analysis on sample_001.fq, generating statistical summaries and anomaly flags in the specified output directory.

### Compare two samples with a reference genome
**Args:** `--inputs sample_A.fq sample_B.fq --reference hg38.fa --mode compare --output-dir ./comparison`
**Explanation:** Performs pairwise comparison of sample_A and sample_B against the hg38 reference genome, identifying differential features.

### Export results as JSON for downstream processing
**Args:** `experiment_data.fasta --mode predict --format json --output-dir ./predictions`
**Explanation:** Runs variant prediction on experiment_data.fasta and exports results in JSON format for programmatic consumption by other pipelines.

### Adjust chunk size for memory-constrained environments
**Args:** `large_dataset.fq --mode explore --chunk-size 5000 --output-dir ./low_mem_run`
**Explanation:** Processes large_dataset.fq in smaller chunks of 5,000 sequences to reduce RAM usage on systems with limited memory.

### Build and cache a reference index explicitly
**Args:** `axiome-index --reference ecoli_k12.fa --output-dir ~/.axiome/cache`
**Explanation:** Pre-builds an index for the E. coli K-12 reference genome and caches it for reuse in subsequent runs.

### Filter low-quality reads before analysis
**Args:** `reads.fq --mode explore --quality-cutoff 30 --output-dir ./filtered_analysis`
**Explanation:** Filters reads with phred-scaled quality below 30 before exploratory analysis, producing more reliable anomaly detection.