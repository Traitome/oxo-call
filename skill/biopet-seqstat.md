---
name: biopet-seqstat
category: readprocessing
description: A read statistics tool from the Biopet toolkit that computes comprehensive sequencing metrics from FASTQ and FASTA input files, including read counts, base composition, quality score distributions, and overall sequence quality assessments.
tags: [fastq, fasta, statistics, quality-control, sequencing-metrics]
author: AI-generated
source_url: https://biopet.readthedocs.io/en/latest/tools/seqstat/
---

## Concepts

- **Input Format Flexibility**: biopet-seqstat accepts both FASTQ and FASTA input files, automatically detecting the format based on file extension and content structure. This allows seamless integration into workflows using different sequencing file formats.
- **Per-Position Statistics**: The tool calculates statistics for each position in reads, enabling identification of systematic quality issues at specific cycles (e.g., degraded quality at read ends typical of Illumina data). This produces arrays of per-position quality metrics.
- **Multi-Sample Support**: Multiple input files can be processed simultaneously, producing aggregated statistics across all samples. This is particularly useful for pooled sequencing experiments where combined metrics provide better population-level insights.
- **Output Formats**: Statistics are emitted in multiple machine-readable formats (JSON, TSV), enabling downstream parsing by workflow managers and reporting tools. JSON output nests metrics hierarchically while TSV provides flat tabular data.

## Pitfalls

- **Uncompressed Input Requirement**: Feeding gzip-compressed FASTQ files directly without decompression will cause parsing failures. Always decompress input files first using `gzip -dc` or configure the tool to handle compressed input explicitly.
- **Omitting Output Flag for Downstream Processing**: Running without specifying an output file or format results in output being sent to stdout, which may be truncated in long-running pipeline contexts. Always use explicit output flags to ensure complete result capture.
- **Ignoring Warning Messages About Mixed Quality Encodings**: The tool detects quality score encodings (Phred+33 vs Phred+64) but may produce misleading statistics if encoding is misidentified. Review warning messages and verify encoding matches your sequencer's expected format.
- **Assuming Read Names Are Unique**: Statistics assume unique read identifiers when calculating metrics like duplicate read rates. If paired-end data has read names differing only by /1 or /2 suffixes, this affects read-level metric accuracy.

## Examples

### Generate sequencing statistics from a single FASTQ file
**Args:** -i sample1_R1.fastq.gz -o sample1_stats.json -f json
**Explanation:** Reads the specified FASTQ file and writes comprehensive statistics (read count, length distribution, GC content, quality histograms) to a JSON output file for programmatic parsing.

### Compute statistics from paired-end FASTQ files
**Args:** -i sample1_R1.fastq.gz -i sample1_R2.fastq.gz -o paired_stats.json -f json
**Explanation:** Processes both read files together, generating combined statistics that reflect the full paired-end dataset including both forward and reverse read metrics.

### Export statistics in human-readable TSV format
**Args:** -i sample1_R1.fastq -o sample1.tsv -f tsv
**Explanation:** Writes statistics to a tab-separated file that can be directly opened in spreadsheet applications, with columns representing different metrics and rows for summary values.

### Calculate statistics with custom sample naming for downstream tracking
**Args:** -i experiment_run3.fastq -o run3.json --sample experiment_run3
**Explanation:** Associates the input with a custom sample identifier embedded in output metadata, useful when integrating with LIMS systems or sample tracking databases.

### Process multiple FASTQ files in a single run for batch analysis
**Args:** -i sample1.fastq -i sample2.fastq -i sample3.fastq -o batch_summary.json -f json
**Explanation:** Aggregates statistics across all three input files, producing combined metrics that represent the entire batch and enabling cross-sample comparison of quality metrics.