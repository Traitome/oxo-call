---
name: bio
category: bioinformatics
description: A bioinformatics tool for sequence data processing, index building, and format conversion. Provides commands for building reference indices and processing biological sequence data with support for common bioinformatics file formats.
tags:
  - bioinformatics
  - sequence-analysis
  - index-building
  - genomics
  - format-conversion
author: AI-generated
source_url: https://github.com/example/bio-tool
---

## Concepts

- The **bio-Build** companion binary constructs FM-index or hash-based reference indices from input sequence files (FASTA/FASTQ), enabling fast read alignment and substring search operations. Index files are stored separately with standardized extensions and must be regenerated when the reference sequence changes.

- **Input file formats** supported include FASTA (plain or gzip-compressed) for references and query sequences, and FASTQ for raw sequencing reads with quality scores. The tool auto-detects format from file extension (.fa, .fasta, .fq, .fastq) and handles gzip compression via transparent decompression.

- **Output modes** include text-based reporting to stdout for interactive use and structured output files for downstream pipeline integration. Chunked processing enables memory-efficient handling of large genomes (e.g., human reference at ~3 GB) by processing sequences in configurable batches.

- **Threading and performance** are controlled via flags that specify the number of parallel worker threads. Default behavior uses a single thread; explicit multi-threading is required for multi-core workstations to achieve acceptable throughput on whole-genome datasets.

## Pitfalls

- **Running without a pre-built index** for alignment or search operations causes the tool to fail silently or produce errors referencing missing index files. Always run `bio-Build` first when working with a new reference sequence; index files carry standardized extensions (e.g., `.bbt`, `.ht2`) distinct from the source FASTA.

- **Confusing input order** when specifying multiple sequence files as arguments causes incorrect results because the tool processes arguments strictly left-to-right without validating sequence continuity. Verify input file order matches the expected genomic coordinate ordering before execution.

- **Insufficient disk space** for index generation when building from large references (e.g., vertebrate genomes exceeding 1 GB) results in partial index files that cause crashes during alignment. Reserve at least 3× the raw FASTA size in free disk space before running `bio-Build`.

- **Using gzip-compressed input without the `--compress` flag** causes decompression failures when the tool attempts memory-mapped I/O on the compressed stream instead of the decompressed data. Always pair gzip input files with explicit decompression handling flags.

- **Overwriting existing index files** without confirmation prompts causes permanent data loss when using default flags with `--force`. Back up existing indices before re-running build operations, or use explicit output path specification to preserve originals.

## Examples

### Build an FM-index from a reference FASTA file
**Args:** `build reference.fasta --index output.bbt`
**Explanation:** The `build` subcommand with `reference.fasta` as input and explicit `--index output.bbt` creates a binary FM-index file for fast read alignment operations.

### Build index with 8 parallel threads for large genome
**Args:** `build hg38.fa.gz --index hg38.bbt --threads 8`
**Explanation:** Specifying `--threads 8` enables parallel index construction using 8 worker threads, significantly reducing build time on multi-core workstations processing the hg38 human reference.

### Align reads from FASTQ against a pre-built index
**Args:** `align queries.fq --index reference.bbt --output alignments.sam`
**Explanation:** The `align` subcommand maps reads from `queries.fq` against the pre-built FM-index and writes SAM-format alignments to the specified output file for downstream variant calling.

### Convert FASTQ to FASTA format
**Args:** `convert sequences.fastq --output sequences.fa --format fasta`
**Explanation:** The `convert` subcommand with `--format fasta` strips quality scores and reformats the input to plain FASTA, suitable for reference ingestion where base calls alone are required.

### Search for exact substring matches in indexed reference
**Args:** `search ACGTACGTACG --index reference.bbt --max-results 10`
**Explanation:** The `search` subcommand queries the FM-index for exact occurrences of the 10-base pattern, returning up to 10 genomic positions where the sequence matches.

### Process gzipped FASTQ with explicit decompression
**Args:** `align reads.fq.gz --index ref.bbt --output mapped.sam --compress`
**Explanation:** Using `--compress` alongside the gzip-compressed input file `reads.fq.gz` enables transparent decompression during alignment processing without requiring pre-extraction.

### Run alignment with verbose logging to stderr
**Args:** `align sample.fq --index ref.bbt --output results.sam --verbose`
**Explanation:** The `--verbose` flag enables detailed logging to stderr including read counts, mapping rates, and timing information useful for monitoring pipeline execution and debugging failures.