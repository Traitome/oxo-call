---
name: catnip-seq
category: bioinformatics/sequence-analysis
description: A versatile command-line tool for sequence filtering, transformation, and basic analysis operations on FASTA/FASTQ files. Supports batch processing, format conversion, and sequence quality control.
tags:
  - sequence-analysis
  - fasta
  - fastq
  - bioinformatics
  - filtering
  - format-conversion
author: AI-generated
source_url: https://github.com/catnip-seq/catnip-seq
---

## Concepts

- **Input/Output Formats**: catnip-seq natively supports FASTA and FASTQ formats for both input and output. Use `-i/--input` for input files and `-o/--output` for output files. The tool automatically detects format based on file extension (.fa, .fasta, .fq, .fastq) but can be explicitly specified with `--format fasta` or `--format fastq`.
- **Filtering Logic**: Sequence filtering operations use standard bioinformatics thresholds including sequence length (`--min-length`, `--max-length`), GC content range (`--min-gc`, `--max-gc`), and N content percentage (`--max-n`). Multiple filters can be combined in a single run for compound filtering.
- **Sequence Transformation**: Available transformations include reverse complement (`--rev-comp`), sequence trimming (`--trim-left`, `--trim-right`), case normalization (`--uppercase`, `--lowercase`), and sequence renaming with pattern matching (`--rename`).
- **Batch Processing**: The tool processes multiple input files when given a file list or wildcard patterns. Use `@filelist.txt` to specify a file containing one input path per line. Output is written to corresponding files in the specified output directory.

## Pitfalls

- **Overwriting Output Without Backup**: Using `-o` with an existing output file path without the `--force` flag will cause catnip-seq to prompt for confirmation, but in pipeline scripts this can cause hanging. Always use `--force` when automating or specify unique output names to prevent accidental data loss.
- **Misinterpreting GC Content Threshold Units**: GC content thresholds are specified as percentages (0-100), not as decimal fractions. Using `--min-gc 0.4` will filter sequences with GC content >= 0.4%, which is almost never intended; use `--min-gc 40` instead.
- **Memory Limits with Large FASTQ Files**: When processing gzipped FASTQ files with many sequences, the default buffer size may be insufficient. Use `--buffer-size 512M` or larger for files >10GB to prevent out-of-memory errors during parsing.
- **Incompatible Filtering Combinations**: Combining `--complement` with other filtering operations can produce unexpected results because complement first inverts the entire sequence set before applying subsequent filters, changing the effective input dataset.

## Examples

### Filter sequences shorter than 100 bases
**Args:** `--input sequences.fa --min-length 100 --output filtered.fa`
**Explanation:** This removes all sequences with fewer than 100 nucleotides from the input FASTA file, keeping only longer sequences in the output.

### Convert FASTQ to FASTA format
**Args:** `--input reads.fq --output reads.fa --convert fasta`
**Explanation:** Converts the input FASTQ file (with quality scores) to FASTA format by stripping quality lines, useful when quality data is not needed for downstream analysis.

### Extract sequences with GC content between 40% and 60%
**Args:** `--input genome.fa --min-gc 40 --max-gc 60 --output balanced_gc.fa`
**Explanation:** Filters input sequences to retain only those with moderate GC content, often used to remove extreme GC bias sequences before assembly or annotation.

### Reverse complement all sequences in a file
**Args:** `--input genes.fa --rev-comp --output genes_rc.fa`
**Explanation:** Transforms each sequence to its reverse complement (A↔T, G↔C on the opposite strand), required for certain alignment or comparative analysis workflows.

### Trim 10 bases from both ends of sequences
**Args:** `--input reads.fq --trim-left 10 --trim-right 10 --output trimmed.fq`
**Explanation:** Removes the specified number of nucleotides from both the 5' and 3' ends of each sequence, commonly used to remove adapter contamination or low-quality terminal bases.

### Process multiple files using a list
**Args:** `--input @files.txt --output-dir ./processed --force`
**Explanation:** Reads input file paths from filelist.txt and processes each file, writing results to the specified output directory with preserved basenames.