---
name: advntr
category: genomics/tandem-repeat-detection
description: A tool for detecting and genotyping variable number tandem repeats (VNTRs) from genome sequencing data using hidden Markov models. Works with Illumina, PacBio, and Oxford Nanopore data.
tags:
  - vntr
  - tandem-repeats
  - genomics
  - structural-variants
  - haplotyping
  - hmm
  - repeat-genotyping
author: AI-generated
source_url: https://github.com/byee4/advntr
---

## Concepts

- **VNTR Detection Model**: `advntr` uses profile hidden Markov models (pHMMs) built from multiple sequence alignments of repeat units. Each VNTR locus has a probabilistic model that scores sequencing reads for repeat count and unit composition, enabling accurate genotyping even in low-complexity regions.

- **Input Formats and Requirements**: The tool accepts aligned reads in BAM/CRAM format with proper coordinate sorting. For `advntr- build`, it requires a reference FASTA file and a repeat definition file specifying motif sequences and expected copy ranges. Unsorted or unindexed BAM files will cause silent failures or crashes.

- **Output Interpretation**: Results are reported in tab-delimited format with columns for locus ID, repeat count estimate, confidence interval, and sequence coverage. The `repeat_count` field uses floating-point values to represent partial repeat units detected at read boundaries; integer values indicate high-confidence genotype calls.

- **Companion Build Tool**: The `advntr- build` subcommand creates binary HMM database files from repeat motif definitions. These databases are locus-specific and must be rebuilt when updating reference genomes or repeat catalogs. The binary format is not portable across advntr versions.

- **Multi-Sample Processing**: `advntr` can process multiple BAM files in a single run by specifying multiple `--read-files` arguments or using a sample sheet. However, memory usage scales linearly with the number of parallel samples; exceeding available RAM produces truncated output files with missing loci.

## Pitfalls

- **Mismatched Reference Genomes**: Using a BAM file aligned to GRCh37 with a database built for GRCh38 (or vice versa) causes systematic false negatives. The tool does not validate reference compatibility at runtime; always verify alignment reference matches the database build reference before running analysis.

- **Insufficient Sequencing Depth**: VNTR loci with fewer than 5 reads supporting the locus are marked as unresolved, not deleted. Downstream scripts that filter on `repeat_count` without checking read support will include unreliable zero-count calls, skewing population frequency estimates.

- **Binary Database Version Drift**: Running `advntr` with a database file built by a different major version produces garbled output with impossible repeat counts (negative values, counts exceeding genome length). Always rebuild databases after upgrading `advntr`.

- **Unsorted BAM Input**: Providing a BAM file sorted by read name instead of genomic coordinate causes the alignment scoring algorithm to miscount spanning reads, resulting in systematically inflated repeat count estimates for long VNTRs. Verify sort order with `samtools view -H input.bam | grep SO:`.

- **Ignoring Confidence Intervals**: Output files include 95% confidence intervals that many users discard. When comparing genotypes across samples, overlapping confidence intervals do not guarantee identical genotypes; use the `--compare` flag for statistical comparison instead of point estimates.

## Examples

### Detect VNTRs from a single BAM file
**Args:** `detect --read-files sample.bam --profile-db vntr_db.bin --output results.tsv`
**Explanation:** Runs VNTR detection on a single Illumina BAM file, outputting genotype calls for all loci in the profile database with confidence intervals.

### Genotype a specific VNTR locus only
**Args:** `detect --read-files sample.bam --profile-db vntr_db.bin --output results.tsv --locus VNTR_001`
**Explanation:** Restricts analysis to a single named locus, reducing runtime and output size when only one VNTR is clinically relevant.

### Build a custom VNTR profile database from a motif file
**Args:** `build --reference hg38.fa --repeat-file custom_vntrs.txt --output my_vntrs.bin`
**Explanation:** Creates a binary HMM profile database from a custom repeat definition file, enabling detection of population-specific or novel VNTRs not in pre-built databases.

### Process multiple samples in parallel
**Args:** `detect --read-files sample1.bam sample2.bam sample3.bam --profile-db vntr_db.bin --output multi_results.tsv --threads 8`
**Explanation:** Processes three BAM files simultaneously using 8 threads, producing a combined output table with sample identifiers as additional columns.

### Filter output to high-confidence calls only
**Args:** `filter --input results.tsv --min-support 10 --min-confidence 0.95 --output filtered.tsv`
**Explanation:** Post-processes detection results to retain only genotype calls with at least 10 supporting reads and 95% confidence interval width below 0.5 repeat units.

### Export genotypes in JSON format for downstream APIs
**Args:** `detect --read-files sample.bam --profile-db vntr_db.bin --output genotypes.json --format json`
**Explanation:** Generates JSON-formatted output suitable for programmatic consumption by web services or visualization pipelines, preserving nested confidence interval structure.

### Compare genotypes between two samples
**Args:** `compare --sample1 sample1_results.tsv --sample2 sample2_results.tsv --output comparison.tsv`
**Explanation:** Performs statistical comparison of VNTR genotypes between two samples, reporting P-values for each locus to identify significantly different repeat counts in case-control studies.