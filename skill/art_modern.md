---
name: art_modern
category: Sequence Analysis
description: A bioinformatics tool for processing and analyzing modern high-throughput sequencing data, supporting various NGS formats and providing quality control, read processing, and variant analysis capabilities.
tags: [sequencing, NGS, quality-control, variant-analysis, bioinformatics]
author: AI-generated
source_url: https://github.com/example/art_modern
---

## Concepts

- **Input Formats**: art_modern accepts FASTQ, FASTA, SAM, BAM, and VCF files as input, automatically detecting format by file extension and content magic bytes; compressed formats (.gz, .bz2) are supported natively.
- **Data Model**: The tool operates on read-level data with per-base quality scores stored as Phred scores, supporting both single-end and paired-end reads with proper mate pairing information preserved throughout processing.
- **Output Modes**: Results can be exported in multiple formats including filtered FASTQ, alignment BAM, variant call VCF, and summary statistics JSON/TSV, with configurable verbosity levels for logging and progress reporting.
- **Threading**: Multi-threaded execution is supported via the `-t/--threads` flag, with automatic load balancing across processing stages; default uses all available CPU cores.

## Pitfalls

- **Mismatched Read Types**: Specifying paired-end mode for single-end data (or vice versa) causes index corruption and incorrect mate pairing information in output files, leading to downstream analysis failures.
- **Quality Score Encoding**: Failing to specify `--phred-offset` when input uses Illumina quality encoding (offset 33) versus Sanger (offset 33) causes complete misinterpretation of quality scores, invalidating all downstream filtering.
- **Resource Exhaustion**: Not limiting memory usage with `--max-memory` on systems with limited RAM causes OOM crashes, especially when processing large BAM files without indexing first.
- **Index Mismatch**: Using a reference index built with a different tool version or mismatch to the `-r/--reference` sequence causes silent errors in variant calling and incorrect alignment positioning.

## Examples

### Filter reads by minimum quality threshold
**Args:** `-i input.fq -o filtered.fq -q 30`
**Explanation:** This filters input FASTQ reads, retaining only those where all bases meet the minimum Phred quality score of 30, outputting clean reads to the specified file.

### Convert between quality score encodings
**Args:** `-i input.fq --phred-offset 64 --output-offset 33 -o converted.fq`
**Explanation:** Converts quality scores from Illumina encoding (offset 64) to Sanger encoding (offset 33), necessary when moving data between tools with different encoding expectations.

### Generate summary statistics for a dataset
**Args:** `-i input.fq --stats -o report.json`
**Explanation:** Computes comprehensive quality metrics including per-base distributions, read length histograms, and GC content, exporting results in JSON format for downstream parsing.

### Process paired-end reads with proper mate handling
**Args:** `-i read1.fq -i2 read2.fq -o out1.fq -o2 out2.fq --paired -q 20`
**Explanation:** Processes both paired-end input files simultaneously, maintaining proper mate pair synchronization and filtering reads with minimum quality 20 in both files.

### Specify custom thread count for parallel processing
**Args:** `-i input.fq -o output.fq -t 8 --stats`
**Explanation:** Uses 8 parallel threads for accelerated processing with simultaneous statistics generation, balancing resource usage against processing speed for large datasets.