---
name: biovalid
category: Bioinformatics Quality Control
description: A bioinformatics file format and sequence data validation tool that checks integrity, format compliance, and content quality of common biological sequence file types including FASTA, FASTQ, SAM, BAM, VCF, and BCF.
tags:
  - validation
  - quality-control
  - fasta
  - fastq
  - sam
  - bam
  - vcf
  - bioinformatics
  - file-integrity
  - data-quality
author: AI-generated
source_url: https://github.com/bioinformatics-tools/biovalid
---

## Concepts

- **Multi-format support**: biovalid validates FASTA, FASTQ, SAM, BAM, VCF, and BCF files by inspecting internal structure, field ordering, data types, and value ranges. For FASTQ files it checks ASCII-offset encoding and quality score bounds; for SAM it enforces CIGAR and flag-bit validity; for VCF it validates INFO/FORMAT field syntax and allele representation.
- **Streamable processing**: Input is processed line-by-line or record-by-record without full file loading, making biovalid memory-efficient for very large files (multi-GB). The tool writes a JSON or plain-text report to stdout or a designated output path, annotating each validation error with a 1-based line number, record ID, and error class.
- **Exit code semantics**: The tool exits with code 0 on full validation success, code 1 on format violations, and code 2 on I/O errors (e.g., unreadable input or permission-denied output path). Exit codes can be used directly in pipeline conditional checks.
- **Configurable rule sets**: Strict mode (`--strict`) enforces RFC-compliant field counts and nomenclature; default relaxed mode permits common bioinformatics conventions that deviate slightly from standards (e.g., mixed-case nucleotides in FASTA). Per-format override flags (e.g., `--allow-unknown-tags`) disable specific checks individually.

## Pitfalls

- **Assuming exit code 1 always means file corruption**: Exit code 1 indicates validation failures, which may be non-critical warnings (e.g., unrecognized headers). Pipelines that treat all non-zero exits as fatal errors will prematurely halt on perfectly usable files that contain only advisory issues.
- **Running without `--format` on auto-detected files**: Auto-detection works reliably for standard extensions (.fasta, .fastq, .sam, .vcf) but fails on non-standard extensions (e.g., .txt or .seq), causing biovalid to default to the first format in its internal detection order, which may produce misleading or irrelevant errors.
- **Piping compressed input without decompression**: biovalid does not auto-decompress gzip or BGZF streams when piped via stdin; passing a compressed stream without explicit `--input-compression gzip` flag results in a "magic number mismatch" I/O error reported as exit code 2.
- **Redirecting output to the input file path**: When using shell redirection (e.g., `biovalid input.fq > input.fq`), the output overwrites the input before the tool finishes reading it, producing a truncated-input error or silent data loss. Always redirect to a distinct output path or use `--output` explicitly.

## Examples

### Validate a FASTQ file in default relaxed mode and write report to stdout
**Args:** `seqs.fastq --format fastq --output report.txt`
**Explanation:** biovalid reads the FASTQ file, checks record structure (ID, sequence, separator, quality), and writes all findings to report.txt so the caller can parse validation status without relying solely on exit codes.

### Validate a FASTA file in strict RFC-compliant mode
**Args:** `reference.fasta --format fasta --strict`
**Explanation:** Enabling `--strict` forces biovalid to reject records with lowercase nucleotide bases and non-standard line wrapping, which is appropriate when preparing files for submission to centralized archives.

### Validate a SAM file and stop on the first error encountered
**Args:** `alignments.sam --format sam --strict --fail-fast`
**Explanation:** The `--fail-fast` flag causes biovalid to terminate after the first malformed CIGAR string or invalid flag-bit combination, producing a faster feedback loop during iterative SAM repair workflows.

### Validate a VCF file and output results as machine-readable JSON
**Args:** `variants.vcf --format vcf --output results.json --json`
**Explanation:** Using `--json` encodes each error as a structured JSON object with fields `line`, `column`, `error_class`, and `message`, enabling programmatic post-processing or integration with continuous-integration pipelines.

### Validate a gzip-compressed FASTQ file passed via stdin with explicit decompression
**Args:** `--input-compression gzip