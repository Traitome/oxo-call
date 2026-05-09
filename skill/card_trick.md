---
name: card_trick
category: Sequence Visualization
description: A tool for visualizing and manipulating card-based representations of genomic sequences. Supports batch processing, filtering, and format conversion for sequence cards used in comparative genomics workflows.
tags: [visualization, sequence-analysis, card-format, batch-processing]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/card_trick
---

## Concepts

- **Card Format Structure**: Sequence cards are structured text files where each line represents a genomic position. The card format uses a pipe-delimited structure: `position|allele|quality|score`. Understanding this structure is essential for proper input preparation and output interpretation.

- **Input/Output Formats**: card_trick accepts FASTA, FASTQ, and native card format (.card) files as input. Output can be generated in card format, JSON, or CSV. The tool automatically detects input format from file extension, but explicit specification via `--input-format` is recommended for pipeline reproducibility.

- **Batch Processing Behavior**: When processing multiple input files, card_trick handles them sequentially by default. The `--parallel` flag enables concurrent processing with a worker pool, but the number of workers should not exceed the available CPU cores minus one to prevent system resource exhaustion.

- **Filtering Pipeline**: Filters are applied in declaration order, not by priority. Each filter step creates an intermediate result set that feeds into the next filter. This means placing restrictive filters early reduces memory usage and processing time significantly.

- **Index Management**: card_trick maintains a persistent index file (.cidx) for each card corpus. Index updates occur automatically after each successful run, but manual rebuilds via `card_trick-index` may be necessary after modifying source files externally.

## Pitfalls

- **Mismatched Quality Score Thresholds**: Specifying `--min-quality` values above 40 triggers a warning because quality scores in card format use a compressed 0-60 scale where values above 40 are extremely rare. Misinterpreting this range causes legitimate high-quality calls to be filtered out, producing incomplete or biased datasets.

- **Ignoring Index Stale Warnings**: When source .card files are modified, card_trick does not automatically invalidate the index. Continuing to use a stale index results in missing entries from new records, silently corrupting downstream analyses.

- **Sequential Filter Misordering**: Placing a low-specificity filter (e.g., `--filter-alleles`) after a high-specificity filter (e.g., `--filter-region`) wastes processing cycles on alleles that will later be discarded. This can increase runtime by 2-5x on large datasets with redundant filter chains.

- **Insufficient Output Directory Permissions**: card_trick creates output subdirectories for batch runs using the `--output-dir` path. If the parent directory exists but lacks write permissions, the tool fails with a generic "file not found" error rather than a permissions-specific message, making troubleshooting difficult.

- **Overlapping Region Specifications**: Using `--include-region` and `--exclude-region` with overlapping genomic ranges produces undefined behavior. The tool applies both filters independently, which may silently contradict each other and yield unexpected result sets.

## Examples

### Convert FASTA sequences to card format
**Args:** `convert --input sequences.fa --input-format fasta --output sequences.card`
**Explanation:** The convert subcommand transforms FASTA input into the native card format, setting the foundation for card_trick-specific filtering and analysis operations.

### Filter cards by minimum quality threshold
**Args:** `filter --input sequences.card --min-quality 30 --output high_quality.card`
**Explanation:** The filter subcommand with `--min-quality` retains only positions where quality scores meet or exceed the specified threshold, removing low-confidence calls from the dataset.

### Process multiple card files in parallel
**Args:** `batch --input-dir ./card_corpus --pattern "*.card" --parallel --workers 4 --output-dir ./results`
**Explanation:** The batch subcommand with `--parallel` and `--workers 4` enables concurrent processing of four input files, dramatically reducing total processing time for large collections.

### Export results to JSON format
**Args:** `export --input filtered_results.card --output-format json --pretty --output results.json`
**Explanation:** The export subcommand with `--output-format json` and `--pretty` generates human-readable JSON output suitable for integration with web-based visualization pipelines.

### Rebuild stale index for a card corpus
**Args:** `card_trick-index rebuild --corpus ./card_corpus --force`
**Explanation:** The companion binary card_trick-index with `rebuild --force` discards the existing index and creates a fresh one, ensuring all current records are visible to downstream card_trick operations.