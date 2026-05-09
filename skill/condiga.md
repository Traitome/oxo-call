---
name: condiga
category: Bioinformatics Tools
description: A tool for consensus sequence generation and condition-based filtering in genomic analysis pipelines. Processes aligned reads to produce conditioned consensus sequences with configurable quality thresholds and coverage filters.
tags:
  - consensus
  - genomics
  - sequence-analysis
  - variant-calling
  - quality-control
author: AI-generated
source_url: https://github.com/condiga/condiga
---

## Concepts

- **Consensus Sequence Model**: condiga processes BAM/CRAM alignments to generate consensus sequences by evaluating base frequencies at each genomic position and applying voting or probabilistic algorithms to call the consensus allele. Positions below the coverage threshold are marked as N or masked.
- **Condition Filtering Engine**: The tool applies user-defined conditions (minimum coverage, allele frequency, quality score) as hard filters or weighted penalties before consensus calling. This prevents low-quality or insufficiently supported bases from influencing the final consensus.
- **I/O Formats**: Input accepts SAM/BAM/CRAM for alignments and VCF/BED for target regions. Output is produced in FASTA, FASTQ, or VCF consensus format. The tool also generates a per-position coverage JSON report for downstream interpretation.
- **Threshold Cascade**: When multiple thresholds are specified (e.g., `--min-coverage` and `--min-frequency`), condiga evaluates them in a cascade where each position must satisfy all conditions sequentially. This prevents partial filtering where one threshold might compensate for another.

## Pitfalls

- **Inconsistent Threshold Units**: Specifying `--min-coverage` without `--max-coverage` when your reference is repetitive can lead to overcalls in low-complexity regions where depth inflates artificially, causing consensus inflation in tandem repeat arrays.
- **Missing Index Files**: Running condiga on an indexed BAM without a corresponding .bai index file causes the tool to scan the entire file sequentially, dramatically increasing runtime on large datasets and producing silent failures on subset analyses.
- **Conflicting Output Formats**: Using `--output-format fasta` with `--emit-gaps` on regions with no coverage produces FASTA files with consecutive N characters that downstream aligners may reject as malformed, breaking pipelined tools without warning.
- **Base Quality Encoding Mismatch**: condiga defaults to Illumina 1.8+ Phred+33 encoding. Specifying input from older datasets using Phred+64 encoding without `--encoding phred64` causes systematic quality score misinterpretation, leading to incorrect consensus calls at low-quality bases.

## Examples

### Generate a simple consensus sequence from an aligned BAM
**Args:** `--input alignment.bam --reference ref.fa --output consensus.fa`
**Explanation:** This processes the BAM file against the reference and outputs a consensus sequence in FASTA format, using default filtering thresholds for coverage and quality.

### Apply strict coverage filtering to remove low-depth regions
**Args:** `--input alignment.bam --reference ref.fa --output consensus_strict.fa --min-coverage 20 --min-quality 30`
**Explanation:** This generates a consensus only where at least 20 reads cover each base and base quality exceeds Phred 30, masking all other positions as N in the output sequence.

### Export per-position coverage statistics as JSON
**Args:** `--input alignment.bam --reference ref.fa --report coverage.json --report-format json`
**Explanation:** This produces a JSON file containing depth and quality metrics at every genomic position, enabling downstream statistical analysis or visualization in R or Python.

### Target specific genomic regions using a BED file
**Args:** `--input alignment.bam --reference ref.fa --output target_consensus.fa --region targets.bed`
**Explanation:** This restricts consensus calling to the genomic intervals specified in the BED file, significantly reducing runtime when only a subset of the reference is of interest.

### Generate a phased consensus accounting for allelic imbalance
**Args:** `--input alignment.bam --reference ref.fa --output phased_consensus.fa --min-frequency 0.7 --phase`
**Explanation:** This calls consensus alleles separately for each haplotype when heterozygous positions meet the 70% frequency threshold, producing two consensus sequences representing parental alleles.