---
name: biopet-validatefastq
category: Quality Control
description: Validates FASTQ file format integrity, checks quality score encoding, and verifies sequence/quality score pairing for next-generation sequencing data.
tags:
  - fastq
  - validation
  - sequencing
  - qc
  - bioinformatics
  - ngs
author: AI-generated
source_url: https://github.com/biopet/biopet
---

## Concepts

- **FASTQ Format Structure**: The tool validates the four-line FASTQ record format: header line (starting with @), sequence line, plus line (starting with +), and quality score line. Each record must have matching sequence and quality score lengths.
- **Quality Score Encoding Detection**: biopet-validatefastq automatically detects Phred+33 (Sanger), Phred+64 (Illumina 1.3-1.7), or Octal+33 encodings by analyzing quality score character ranges and can report mismatches.
- **Paired-End File Validation**: When validating paired-end FASTQ files, the tool verifies that read identifiers match between forward (R1) and reverse (R2) files, ensuring proper pairing for downstream analysis.
- **Error Reporting**: The tool provides detailed error messages including line numbers, problematic sequences, and specific format violations (e.g., invalid characters in quality scores, malformed headers).

## Pitfalls

- **Ignoring Quality Score Warnings**: Running the tool without addressing quality score encoding mismatches can result in incorrect base caller confidence values being used in variant calling or expression analysis, leading to false positives or missed detections.
- **Skipping Paired-File Consistency Check**: Failing to validate that R1 and R2 files have identical read counts and matching identifiers will cause tools like BWA or STAR to fail or produce corrupted alignments.
- **Not Verifying File Completeness**: If a FASTQ file is truncated or corrupted mid-record, downstream pipelines may process incomplete data without raising errors, resulting in biased or missing results in final outputs.

## Examples

### Validate a single FASTQ file for basic format issues
**Args:** `-i sample.fastq`
**Explanation:** Validates the FASTQ file structure without verbose output, reporting only critical format errors.

### Check quality score encoding version
**Args:** `-i sample.fastq --encoding illumina`
**Explanation:** Explicitly checks against Illumina 1.3-1.7 Phred+64 encoding, flagging any quality scores that fall outside the expected ASCII range.

### Validate paired-end files with read name matching
**Args:** `-i sample_R1.fastq -i sample_R2.fastq --paired`
**Explanation:** Validates both FASTQ files simultaneously and verifies that read identifiers in R1 exactly match those in R2.

### Enable verbose output for debugging format issues
**Args:** `-i sample.fastq -v`
**Explanation:** Provides detailed line-by-line validation output with specific error locations and character-by-character analysis for troubleshooting.

### Validate multiple files in batch mode
**Args:** `-i sample1.fastq -i sample2.fastq -i sample3.fastq --batch`
**Explanation:** Validates all three FASTQ files in a single run, reporting individual file status and aggregate validation results.

### Check for invalid nucleotide characters
**Args:** `-i sample.fastq --strict-sequence`
**Explanation:** Enforces strict nucleotide validation (A, T, G, C, N only), flagging any non-standard IUPAC codes or invalid characters in sequence lines.

### Generate validation report to file
**Args:** `-i sample.fastq -o validation_report.txt`
**Explanation:** Writes validation results to the specified output file instead of stdout, useful for integration into automated pipeline logging.