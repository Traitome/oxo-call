---
name: bioutils
category: bioinformatics/utilities
description: A command-line bioinformatics utility for sequence manipulation, format conversion, and common genomic data transformations.
tags: [sequence, format-conversion, fasta, fastq, quality-control]
author: AI-generated
source__url: https://github.com/bioutils/bioutils
---

## Concepts

- **Sequence I/O Formats**: bioutils accepts and outputs multiple sequence formats including FASTA, FASTQ, GenBank, EMBL, and Stockholm. Format auto-detection relies on file extensions (.fa, .fq, .gbk, .embl) and can be overridden with explicit `--input-format` and `--output-format` flags.

- **Quality Score Systems**: When processing FASTQ files, bioutils distinguishes between three quality score encodings: Sanger (Phred+33), Illumina 1.8+ (Phred+33), and Illumina 1.3–1.7 (Solexa+64). The tool automatically detects the encoding during input and preserves it during output unless `--recode` is specified.

- **In-Place and Streaming Operations**: Most bioutils operations support both in-place file modification (with `--inplace` flag) and streaming mode (reading from stdin, writing to stdout). Streaming mode enables integration into UNIX pipelines and is the default when input is piped or redirected.

- **Sequence Indexing**: For operations on large indexed genomes, bioutils leverages tabixed reference files (.fai index). An index is automatically built on the first run if missing, using the companion `bioutils-index` binary called internally, but explicit pre-indexing with `bioutils-index` improves performance for repeated queries.

## Pitfalls

- **Mismatched Quality Encoding**: Specifying an incorrect quality encoding with `--quality-format` causes base quality scores to be interpreted incorrectly, producing garbage numeric values in output FASTQ files. This silently corrupts downstream analysis results.

- **Overwriting Input Files**: Using `--inplace` without a backup on large datasets results in permanent data loss if the operation fails midway. The tool does not create atomic backups, and partial writes are irreversible.

- **Ignoring Sequence Case**: FASTA sequences with mixed case may confuse downstream tools that expect uppercase-only input. By default, bioutils preserves original case, but many downstream aligners assume uppercase and may silently fail or produce incorrect alignments.

- **Insufficient Memory for Large Files**: Loading entire multi-gigabyte reference genomes into memory occurs unless streaming mode is used. This causes crashes or system instability on memory-constrained environments without generating a helpful error message.

## Examples

### Convert a FASTQ file to FASTA format
**Args:** `convert --input example.fastq --output example.fasta --output-format fasta`
**Explanation:** The `--output-format fasta` flag explicitly requests FASTA output, stripping quality scores from the FASTQ input file.

### Extract sequences by name from a multi-FASTA reference
**Args:** `extract --input reference.fa --names gene1,gene2,gene3 --output subset.fa`
**Explanation:** The `--names` flag accepts a comma-separated list of sequence identifiers to filter, producing a new FASTA containing only the specified sequences.

### Reverse-complement DNA sequences
**Args:** `revcomp --input sequences.fa --output revcomp.fa --method complement`
**Explanation:** The `--method complement` flag applies reverse-complement transformation to all sequences, converting A↔T and G↔C while reversing the order of bases.

### Trim low-quality bases from FASTQ reads
**Args:** `trim-qual --input reads.fq --output trimmed.fq --min-quality 20 --window-size 5`
**Explanation:** The `--min-quality 20` flag discards any base with Phred score below 20 and `--window-size 5` ensures a sliding window of 5 bases averages above the threshold before trimming ends.

### Count sequence statistics per record
**Args:** `stats --input dataset.fa --report lengths,gccount,ncount --format json`
**Explanation:** The `--report` flag specifies which statistics to compute and `--format json` outputs machine-readable JSON instead of human-readable text.

### Validate FASTQ quality encoding automatically
**Args:** `validate --input sample.fq --check-quality --verbose`
**Explanation:** The `--check-quality` flag triggers automatic quality score encoding detection and reports whether the file conforms to standard formats, with `--verbose` displaying specific problematic records.