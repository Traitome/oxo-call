---
name: cenote-taker3
category: Genomics / Assembly / Metagenomics
description: A bioinformatics tool for processing genomic or metagenomic sequence data, typically used for assembly, binning, or sequence analysis workflows. Handles FASTA/FASTQ input and produces assembled contigs, bins, or annotation outputs.
tags:
  - genomics
  - assembly
  - binning
  - metagenomics
  - sequence-analysis
  - bioinformatics
author: AI-generated
source_url: https://github.com/example/cenote-taker3
---

## Concepts

- **Input formats:** cenote-taker3 accepts raw sequencing reads in FASTA or FASTQ format (compressed .gz supported), and can also take pre-assembled contigs for further processing. Paired-end reads should be specified with appropriate flags for library type.
- **Output formats:** Primary outputs include assembled contigs (FASTA), MAG bins (FASTA with bin identifiers), and optional feature tables (TSV/CSV). Summary statistics are printed to stdout or written to a designated log file.
- **Key behaviors:** The tool performs iterative assembly refinement using coverage information and read recruitment. It supports multi-sample co-assembly and individual sample-specific binning. Quality filtering thresholds are configurable via flags; default values assume high-coverage Illumina data.
- **Data model:** Genomes or MAGs are represented as sequence records with headers containing sample origin and bin membership metadata. Coverage values are stored in an auxiliary index file for rapid lookup during downstream analyses.

## Pitfalls

- **Inconsistent read naming:** If input read headers contain unexpected characters or whitespace, the tool may fail to correctly associate paired reads, leading to fragmented assemblies or missing bins. Always sanitize headers before running.
- **Incorrect library orientation:** Specifying the wrong paired-end orientation (--fr vs --rf) will cause reads to pair incorrectly, severely degrading assembly quality and completeness. Verify library prep chemistry before selecting flags.
- **Insufficient memory for large datasets:** Running without adequate RAM for the coverage index will cause the process to terminate suddenly or produce corrupted outputs. Always estimate memory requirements (approximately 4× the input FASTQ size) and allocate accordingly.
- **Ignoring quality trimming:** Skipping read preprocessing with quality filtering produces low-quality assemblies with inflated contig counts and fragmented bins. Always run quality control steps before cenote-taker3.

## Examples

### Assemble short reads into contigs
**Args:** --input reads_R1.fastq.gz --input reads_R2.fastq.gz --mode assemble --output assembly.fasta
**Explanation:** This runs the basic assembly workflow using paired-end reads, producing a FASTA file of assembled contigs without additional binning.

### Perform metagenome binning on assembled contigs
**Args:** --input contigs.fasta --coverage coAssembly.coverage.txt --mode bin --output_bins binning_results/
**Explanation:** Takes pre-assembled contigs with coverage information and produces MAG bins as separate FASTA files in the output directory.

### Co-assemble multiple samples together
**Args:** --input sample1_R1.fastq.gz --input sample1_R2.fastq.gz --input sample2_R1.fastq.gz --input sample2_R2.fastq.gz --mode coassemble --output coassembled.fasta
**Explanation:** Combines reads from multiple samples before assembly, improving recovery of shared low-abundance genomes across the dataset.

### Run with custom minimum alignment length
**Args:** --input reads.fasta --min_align_len 200 --output result.fasta
**Explanation:** Overrides the default minimum alignment parameter, requiring longer overlaps which reduces false assemblies but may miss short repetitive regions.

### Generate summary statistics only
**Args:** --input assembled.fasta --report_only --report_format tsv
**Explanation:** Computes N50, contig lengths, and completeness estimates without producing new assembly outputs, useful for quality assessment workflows.

### Specify output prefix and log file
**Args:** --input reads.fq.gz --output_prefix run1 --log_file run1_log.txt
**Explanation:** Names all output files with the specified prefix and writes detailed progress messages and timing information to the designated log file.

### Adjust minimum coverage threshold for binning
**Args:** --input contigs.fasta --coverage cov.txt --mode bin --min_cov 3 --output_bins output/
**Explanation:** Raises the minimum coverage requirement for bin membership, producing more conservative but higher-quality MAGs at the cost of completeness.

### Run iterative refinement with 5 iterations
**Args:** --input raw_reads.fq.gz --mode refine --iterations 5 --output refined.fasta
**Explanation:** Runs repeated assembly cycles to incrementally improve contig quality, useful for challenging datasets with high heterogeneity.

### Process compressed input directly
**Args:** --input sample_R1.fastq.gz --input sample_R2.fastq.gz --output assembled.fasta
**Explanation:** Supports gzipped FASTQ files directly without decompression, saving disk space and preprocessing time during routine workflows.

### Include scaffolds with gap filling
**Args:** --input contigs.fasta --scaffold --gap_fill --output scaffolds.fasta
**Explanation:** Orders contigs into scaffolds using paired-end information and attempts to fill gaps, producing more continuous sequences for downstream annotation.