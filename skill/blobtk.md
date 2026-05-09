---
name: blobtk
category: Bioinformatics Tools
description: A bioinformatics toolkit for processing and analyzing genomic data. Provides utilities for format conversion, quality control, filtering, and statistical analysis of sequencing data.
tags: [genomics, sequence-analysis, fastq, fasta, bioinformatics, quality-control]
author: AI-generated
source_url: https://github.com/blobtk
---

## Concepts

- **Subcommand Architecture**: blobtk uses a subcommand model where operations are specified as the first positional argument (e.g., `blobtk fastq`, `blobtk align`, `blobtk stats`). Each subcommand has its own set of flags and arguments.
- **Input/Output Formats**: Supports common sequencing formats including FASTQ, FASTA, SAM, BAM, and VCF. Format auto-detection is based on file extensions, but the `-f` flag can explicitly specify input format when needed.
- **Streaming and File I/O**: Many subcommands support stdin/stdout streaming with `-` as the input/output path, enabling pipeline composition with other tools like `grep`, `awk`, or other bioinformatics utilities.
- **Index Building**: The companion binary `blobtk-build` creates indices for efficient sequence lookup in large FASTA/FASTQ databases. Generated indices use a custom binary format with `.bti` extension.

## Pitfalls

- **Omitting Format Flags**: Not specifying `-f` when input files have non-standard extensions can lead to format misdetection, causing parse errors or silent data corruption. Always verify the auto-detected format with the `--verbose` flag.
- **Forgetting Compression Formats**: blobtk expects gzipped FASTQ files to have `.fq.gz` or `.fastq.gz` extensions. Using `.gz` alone may not be recognized, resulting in parse failures.
- **Index Version Mismatch**: Using an index built with an older blobtk version against a newer binary may cause alignment errors or crashes. Rebuild indices when upgrading blobtk.
- **Memory-Intensive Operations**: Processing whole-genome datasets without the `-t` flag for thread allocation can lead to excessive memory usage. Always specify thread count based on available system resources.

## Examples

### Convert FASTQ to FASTA format
**Args:** `fastq2fasta -i input.fq -o output.fa`
**Explanation:** Converts reads from FASTQ format to FASTA format, stripping quality scores and keeping only sequence identifiers and nucleotide data.

### Filter reads by quality threshold
**Args:** `filter -i input.fq -o filtered.fq -q 30`
**Explanation:** Retains only reads where all bases have Phred quality scores of 30 or higher, useful for downstream analysis requiring high-confidence sequences.

### Generate sequencing statistics
**Args:** `stats -i input.fq --verbose`
**Explanation:** Outputs detailed statistics including read counts, base composition, average quality, and GC content in verbose mode for thorough data characterization.

### Build index for FASTA database
**Args:** `blobtk-build reference.fa -o reference.bti`
**Explanation:** Creates a binary index file from the reference FASTA for fast sequence lookup in subsequent alignment or search operations.

### Extract specific genomic region
**Args:** `extract -i alignments.bam -c chr1:1000000-2000000 -o region.bam`
**Explanation:** Extracts all alignments falling within chromosome 1 positions 1,000,000 to 2,000,000, using 1-based inclusive coordinates.

### Sort BAM alignments by coordinate
**Args:** `sort -i unsorted.bam -o sorted.bam -s coordinate`
**Explanation:** Sorts input BAM file by genomic coordinate, producing an output file suitable for variant calling or visualization tools.

### Stream processing with pipe
**Args:** `filter -i - -q 20 | fastq2fasta -o -`
**Explanation:** Filters stdin input for quality threshold 20 and converts to FASTA output to stdout, enablingUnix pipeline composition without intermediate files.

### Specify thread count for parallel processing
**Args:** `align -i reads.fq -r ref.fa -o alignments.sam -t 8`
**Explanation:** Runs alignment with 8 threads, utilizing multiple cores for faster processing of large read sets.