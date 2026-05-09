---
name: cifi
category: File Format Utilities
description: A command-line tool for validating, converting, and manipulating common bioinformatics file formats (FASTA, FASTQ, VCF, BAM). Supports quality filtering, format conversion, sequence extraction, and batch processing operations.
tags:
- bioinformatics
- file-conversion
- fasta
- fastq
- vcf
- quality-control
- sequence-analysis
- format-validation
author: AI-generated
source_url:
---

## Concepts

- **Input/Output Formats**: cifi supports FASTA (.fa/.fna), FASTQ (.fq/.fastq), VCF (.vcf), and SAM/BAM (.sam/.bam) formats. Input format is auto-detected from file extension; use `--input-format` to override auto-detection.
- **Quality Filtering**: For FASTQ files, you can filter reads by quality scores using `--min-quality` (Phred score threshold) and `--max-n` (maximum allowed N bases per read). Reads failing these thresholds are excluded from output.
- **Batch Processing**: Multiple input files can be processed in a single run using glob patterns (e.g., `sample_*.fastq`) or by specifying `--input-list` with a file containing one input path per line. Output files are written to the directory specified by `--output-dir`.
- **Sequence Operations**: cifi can extract specific sequences by ID (`--extract-id`), reverse-complement sequences (`--rev-comp`), translate to protein (`--translate`), and trim adapters/subsequences with `--trim-start` and `--trim-end`.
- **Statistics Generation**: The `--stats` flag outputs a JSON report containing read counts, base composition, quality distributions, and format-specific metrics. Use `--stats-output` to save to a file.

## Pitfalls

- **Overwriting Output Files**: By default, cifi overwrites existing output files without warning. Use `--force` to permit overwrites, or set `--output-mode` to `fail-if-exists` to prevent accidental data loss.
- **Insufficient Memory for Large Files**: Processing large BAM/VCF files requires significant RAM. For files >10GB, use the `--chunk-size` parameter to process in segments and avoid OOM errors.
- **Mismatched Input Format**: Specifying an incorrect `--input-format` causes silent failures or corrupted output. Always verify format matches the actual file structure; use `--validate` to check integrity before conversion.
- **Missing Output Directory**: If `--output-dir` does not exist, cifi fails with an error. Create the directory beforehand or use `--create-dir` to allow automatic creation.
- **Quality Score Encoding Mismatch**: FASTQ files may use either Illumina 1.8+ (Phred+33) or older Sanger (Phred+64) quality encoding. Use `--quality-offset` to specify the correct encoding; default is auto-detection which may mislabel older files.

## Examples

### Convert a FASTQ file to FASTA format
**Args:** `input/sample1.fastq --output-dir output/ --convert-to fasta`
**Explanation:** Converts FASTQ sequences to FASTA format, stripping quality scores. Useful when you only need sequence data for alignment or analysis pipelines.

### Filter FASTQ reads by minimum quality threshold
**Args:** `input/reads.fq --output-dir filtered/ --min-quality 30`
**Explanation:** Removes reads containing any base with Phred quality score below 30, retaining only high-confidence sequences for downstream analysis.

### Extract specific sequences by ID from a FASTA file
**Args:** `input/sequences.fa --extract-id SRR123456 --output-dir extracted/`
**Explanation:** Pulls the sequence with exact ID "SRR123456" from a multi-sequence FASTA file into a new output file.

### Generate statistics for a VCF file
**Args:** `input/variants.vcf --stats --stats-output variant_report.json`
**Explanation:** Outputs a JSON report containing variant counts, sample-wise heterozygosity, transition/transversion ratios, and other VCF-specific metrics.

### Reverse-complement DNA sequences
**Args:** `input/genes.fa --output-dir revcomp/ --rev-comp`
**Explanation:** Transforms each sequence to its reverse complement, maintaining the same order and header. Essential for strand-specific analyses.

### Batch process multiple FASTQ files with a glob pattern
**Args:** `sample_*.fq --output-dir batch_out/ --convert-to fasta`
**Explanation:** Processes all files matching the glob pattern `sample_*.fq` in the current directory, converting each to FASTA format. Output files are named according to input basenames.

### Trim first 10 bases from each read
**Args:** `input/reads.fq --output-dir trimmed/ --trim-start 10`
**Explanation:** Removes the first 10 bases from every read. Useful for removing adapter contamination or low-quality leading bases.

### Validate BAM file integrity before processing
**Args:** `input/alignments.bam --validate`
**Explanation:** Checks the BAM file for corruption, missing indexes, and format compliance without producing output, reporting any issues to stderr.