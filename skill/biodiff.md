---
name: biodiff
category: Sequence Comparison
description: A bioinformatics tool for comparing biological sequences (DNA, RNA, protein) and detecting differences including SNPs, indels, and structural variations between files or against a reference.
tags:
  - sequence-analysis
  - variant-detection
  - sequence-comparison
  - bioinformatics
  - genomics
author: AI-generated
source_url: https://github.com/biodiff/biodiff
---

## Concepts

- **Input formats**: biodiff accepts FASTA, FASTQ, and plain text sequence formats. Sequences can be provided as multiple files or as a file with a reference sequence for comparison.
- **Comparison modes**: The tool supports three primary modes—pairwise alignment (file vs. file), reference-based comparison (query vs. reference), and consensus generation (multiple sequences merged with variant reporting).
- **Output types**: Results are reported in three formats: human-readable text showing aligned sequences with mismatch annotations, JSON for programmatic processing, and VCF-style variant calls for downstream genomics pipelines.
- **Scoring parameters**: Customizable gap opening/extension penalties, match/mismatch scores, and minimum variant quality thresholds affect alignment sensitivity and specificity.

## Pitfalls

- **Mismatched sequence encodings**: Comparing DNA sequences in different case (upper vs. lower) or with mixed nucleotide encodings (N vs. -) can cause false negative differences or inflated variant counts.
- **Reference sequence orientation**: Failing to specify strand orientation when comparing against a reverse-complemented reference produces inverted alignments and incorrect variant positions.
- **Threshold misconfiguration**: Setting the minimum variant quality too low introduces sequencing errors as false positives; setting it too high may filter out genuine low-frequency variants in mixed populations.
- **File format inconsistencies**: Mixing FASTA headers with identical sequence identifiers across multiple files causes alignment ambiguity and unpredictable variant attribution.

## Examples

### Compare two FASTA files and show differences in text format
**Args:** `file1.fasta file2.fasta --output diff_report.txt --format text`
**Explanation:** Aligns sequences from both files and writes a human-readable report showing aligned sequences with mismatch annotations.

### Generate machine-readable JSON output with variant positions
**Args:** `query.fasta reference.fasta --format json --output variants.json`
**Explanation:** Produces JSON output containing variant coordinates, types (SNP/indel), and allele information for programmatic downstream processing.

### Compare sequences with custom gap penalties
**Args:** `seq1.fasta seq2.fasta --gap-open 10 --gap-extend 0.5 --output aligned.txt`
**Explanation:** Applies custom affine gap scoring parameters during alignment to detect indels more sensitively based on the specified penalties.

### Filter variants by minimum quality threshold
**Args:** `sample.fasta reference.fasta --min-quality 30 --output highq_variants.txt`
**Explanation:** Includes only variants with quality scores of 30 or higher, reducing false positives from sequencing errors in the output.

### Produce VCF-style output for variant calling pipelines
**Args:** `reads.fasta reference.fasta --format vcf --output calls.vcf`
**Explanation:** Generates variant calls in VCF format, compatible with standard genomics downstream analysis tools and variant databases.