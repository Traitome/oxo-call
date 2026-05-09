---
name: arb-bio
category: bioinformatics/sequence-analysis
description: A versatile bioinformatics utility for processing, filtering, and analyzing biological sequence data in various formats. Supports FASTA, FASTQ, and plain text formats with built-in quality filtering, sequence transformation, and format conversion capabilities.
tags: [sequence-processing, fasta, fastq, quality-filter, bioinformatics, format-conversion]
author: AI-generated
source_url: https://example.com/arb-bio
---

## Concepts

- **Input Format Flexibility**: arb-bio accepts FASTA, FASTQ, and raw sequence files. Auto-detects format from file extensions (.fa, .fasta, .fq, .fastq) or via explicit `--format` flag. Multiple input files can be processed in a single run.

- **Quality-Based Filtering**: When working with FASTQ data, sequences can be filtered by quality score thresholds using `--min-qual` (minimum phred score) and `--qual-window` (sliding window for averaged scores). This removes low-confidence bases before downstream analysis.

- **Sequence Transformation**: The tool provides common sequence manipulations including reverse complement (`--rev-comp`), translation to protein (`--translate`), and case normalization (`--uppercase`, `--lowercase`). These operations maintain sequence integrity and can be chained.

- **Output Formatting**: Results can be written in multiple formats via `--out-format`, supporting FASTA, FASTQ, CSV, and JSON. The tool appends to output files by default unless `--overwrite` is specified.

- **Statistics and Summaries**: The `--stats` flag generates descriptive statistics including base composition, GC content, N content, sequence lengths, and quality distributions without producing sequence output.

## Pitfalls

- **Mismatched Input/Output Formats**: Specifying FASTQ output when input lacks quality scores (FASTA input) causes the tool to exit with an error. Always ensure output format is compatible with input data quality information.

- **Overwriting Output Without Warning**: Using `--overwrite` without checking existing files silently replaces previous results. Always verify output directory contents before running with this flag.

- **Quality Window Larger Than Sequence**: Setting `--qual-window` to a value exceeding sequence lengths produces empty output without a warning message. Review sequence length distributions using `--stats` first.

- **Insufficient Memory for Large Files**: Processing multi-gigabyte sequence files without `--chunk-size` adjustment may cause memory errors. Use `--chunk-size` to process in manageable blocks for large datasets.

- **Encoding Mismatch in FASTQ**: The tool expects phred+33 quality encoding by default. Using data with phred+64 encoding without `--qual-encoding phred64` produces incorrect filtering results.

## Examples

### Filter FASTQ sequences by minimum quality threshold
**Args:** `--input reads.fq --min-qual 20 --out-format fasta --output filtered.fa`
**Explanation:** Removes all sequences containing bases with phred quality scores below 20, outputting remaining sequences in FASTA format (quality scores discarded).

### Convert FASTA to FASTQ with dummy quality scores
**Args:** `--input sequences.fa --out-format fastq --qual-score 30 --output sequences.fq`
**Explanation:** Transforms FASTA input to FASTQ format, assigning a phred quality score of 30 to all bases for compatibility with downstream pipelines requiring FASTQ.

### Generate sequence statistics without filtering
**Args:** `--input dataset.fa --stats --output stats.txt`
**Explanation:** Computes and writes summary statistics including total sequences, base counts, GC content, and sequence length distribution to the specified output file.

### Extract reverse complements of DNA sequences
**Args:** `--input genes.fa --rev-comp --output genes_rev.fa`
**Explanation:** Generates a new FASTA file containing the reverse complement of each input sequence, preserving all headers and order.

### Translate DNA sequences to protein
**Args:** `--input coding_sequences.fa --translate --output proteins.fa`
**Explanation:** Converts DNA coding sequences to amino acid proteins using the standard genetic code, stopping at the first stop codon in each frame.

### Process sequences in chunks to manage memory
**Args:** `--input large_dataset.fq --chunk-size 500000 --output processed/ --stats`
**Explanation:** Processes the large FASTQ file in 500,000-sequence blocks, writing outputs to a directory while generating statistics across all chunks.

### Filter sequences by length range
**Args:** `--input reads.fa --min-length 100 --max-length 1000 --output filtered_length.fa`
**Explanation:** Retains only sequences with lengths between 100 and 1000 bases inclusive, useful for removing adapter contamination or overly short reads.

### Convert multiple input files to JSON format
**Args:** --input sample1.fa sample2.fa --out-format json --output combined.json
**Explanation:** Merges multiple FASTA inputs into a single JSON file with structured sequence records, headers, and metadata for programmatic access.