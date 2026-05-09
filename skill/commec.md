---
name: commec
category: bioinformatics/sequence-analysis
description: A command-line tool for comparing and analyzing sequence communication patterns across genomic datasets. commec identifies conserved sequence motifs, compares expression communication profiles, and generates similarity matrices for downstream evolutionary analysis.
tags:
  - sequence-comparison
  - motif-analysis
  - communication-profiles
  - genomics
  - similarity-matrix
author: AI-generated
source_url: https://github.com/bioinformatics-tools/commec
---

## Concepts

- commec operates on FASTA, FASTQ, and multi-FASTA input files, comparing communication patterns between sequences by analyzing positional frequency distributions and motif co-occurrence networks. The tool outputs similarity scores, pairwise alignments, and cluster assignments in tabular and matrix formats.
- The core data model treats each input sequence as a "communicator" with defined motif weights. Sequences are represented as vectors in motif-space, where distance metrics (Euclidean, Manhattan, or cosine) determine communication similarity scores ranging from 0 (identical) to 1 (maximally divergent).
- commec supports batch processing mode via directory input, processing all sequence files recursively and generating per-file reports plus a summary comparison matrix. The tool appends results to existing output files when the `--append` flag is used, enabling incremental analysis across multiple dataset versions.
- Output formats include CSV (default), TSV, JSON, and Phylip-compatible distance matrices for integration with phylogenetic reconstruction tools. The `--format` flag controls output serialization, while `--quiet` suppresses progress indicators during batch operations.
- The `--window` parameter defines the sequence scanning window size (default: 10bp), influencing motif detection sensitivity. Smaller windows increase computational cost but improve detection of short regulatory elements, while larger windows capture broader communication patterns.

## Pitfalls

- Specifying a window size smaller than the shortest input sequence causes commec to abort with a non-descriptive "sequence length error". Always verify that `--window` value is less than or equal to the minimum sequence length in your input dataset before execution.
- Using the `--append` flag without first verifying existing output file headers results in malformed output where new columns are misaligned with prior headers, corrupting downstream parsing. Always inspect output files in a text editor or validate with commec's built-in `--validate` flag after append operations.
- When processing mixed FASTA/FASTQ datasets, commec silently ignores quality scores and treats all sequences equally, potentially skewing similarity matrices for datasets where quality variation is biologically meaningful. Filter datasets beforehand to ensure sequence type consistency.
- The default cosine distance metric assumes normalized motif vectors, but unnormalized input sequences produce inflated similarity scores for GC-rich genomes. Use `--normalize` flag explicitly in workflows involving AT-rich or GC-biased organisms to ensure comparable results.
- Specifying output directory paths that do not exist without the `--create-dir` flag causes commec to fail without creating parent directories. Pre-create output directories or include `--create-dir` in your command to auto-generate paths.

## Examples

### Compare two sequence files and output similarity matrix
**Args:** `seq1.fasta seq2.fasta --matrix-out similarity.csv --metric cosine`
**Explanation:** This compares motif communication profiles between seq1.fasta and seq2.fasta using cosine distance, outputting a similarity matrix in CSV format to similarity.csv.

### Batch process all FASTA files in a directory with Phylip output
**Args:** `input_dir/ --output batch_results.phylip --format phylip --batch`
**Explanation:** This recursively processes all sequence files in input_dir/, generating a single Phylip-format distance matrix combining all comparisons for phylogenetic downstream analysis.

### Analyze with normalized Euclidean distance and custom window size
**Args:** `dataset.fasta --window 15 --metric euclidean --normalize --output norm_results.csv`
**Explanation:** This runs commec with a 15bp scanning window and normalized Euclidean distance, accounting for vector magnitude differences and improving comparability across AT-rich genomes.

### Append new comparison results to existing output file
**Args:** `new_seqs.fasta --append existing_results.csv --output combined.csv --metric manhattan`
**Explanation:** This adds new sequence comparisons to an existing results file using Manhattan distance, incrementing the analysis without overwriting prior batch results for longitudinal dataset comparison.

### Generate quiet batch output with auto-directory creation
**Args:** `genome_dir/ --batch --quiet --create-dir --output results/ --format json`
**Explanation:** This processes all sequences in genome_dir silently, auto-creating the results directory, and outputs comprehensive JSON-formatted reports suitable for programmatic downstream parsing.

### Validate existing output file integrity before appending
**Args:** `--validate combined_output.csv --check-headers --check-alignment`
**Explanation:** This verifies header consistency and column alignment in an existing output file before any append operations, preventing data corruption during incremental analyses.

### Compare single sequences with verbose motif reporting
**Args:** `query.fasta target.fasta --verbose --motif-report --output motif_analysis.tsv`
**Explanation:** This compares two sequences with detailed verbose output, including per-motif statistics in a TSV file for identifying specific divergent communication regions.