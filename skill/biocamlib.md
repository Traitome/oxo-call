---
name: biocamlib
category: Variant Analysis & Consensus Generation
description: A bioinformatics toolkit for consensus sequence generation and variant calling from alignment data. Processes BAM/CRAM files to call variants and produce consensus sequences at specified frequencies.
tags: [bioinformatics, variant-calling, consensus, genomics, alignment, vcf, bam]
author: AI-generated
source_url: https://github.com/biocamlib/biocamlib
---

## Concepts

- **Input formats**: biocamlib accepts SAM/BAM/CRAM alignment files as primary input, along with a reference FASTA file for variant calling and consensus generation.
- **Output formats**: Tool produces VCF/BCF for variant calls, FASTA for consensus sequences, and optional pileup summaries. The output format is determined by the specified output flag.
- **Variant calling model**: Uses a Bayesian model to call variants at configurable allele frequency thresholds (default 0.5 for heterozygous calls), supporting both SNV and indel detection.
- **Index requirement**: Before processing, the reference genome must be indexed using the companion `biocamlib-build` tool, which creates .bci index files.
- **Pileup base quality filtering**: The `-q` flag controls minimum base quality score for variant calls, while `-Q` controls minimum mapping quality, filtering low-quality reads from analysis.

## Pitfalls

- **Forgetting reference index**: Running biocamlib without first creating the reference index with `biocamlib-build` causes immediate failure with a "reference not indexed" error, wasting computation time on large datasets.
- **Mismatched reference and alignment**: Using a reference genome that differs from the one used for alignment produces spurious variant calls and incorrect consensus sequences, leading to invalid downstream analyses.
- **Ignoring read group information**: When processing multi-sample BAM files without read group tags (`@RG`), variant calling may fail or produce inconsistent results because samples cannot be properly distinguished.
- **Setting frequency threshold too low**: Using `-f 0.1` to call low-frequency variants without sufficient sequencing depth produces false positives, as sequencing errors appear as variant claims.

## Examples

### Generate consensus sequence from a BAM file

**Args:** `-b /data/sample.bam -r ref.fa -o consensus.fa`
**Explanation:** Generates a consensus FASTA sequence from the alignment file using the specified reference genome, outputting to the requested file.

### Call variants and output VCF

**Args:** `-b sample.bam -r hg38.fa -v output.vcf -f 0.5`
**Explanation:** Calls variants at 50% allele frequency threshold (standard for heterozygous SNVs), outputting results in VCF format to the specified file.

### Filter variants by minimum read depth

**Args:** `-b alignments.bam -r ref.fa -v calls.vcf -d 20`
**Explanation:** Only calls variants where read depth is at least 20x, reducing false positive calls from low-coverage genomic regions.

### Output pileup summary along with variants

**Args:** `-b sample.bam -r ref.fa -v var.vcf -p pileup.txt -Q 30 -q 25`
**Explanation:** Produces detailed pileup output with variants, filtering reads with mapping quality below 30 and base quality below 25.

### Build index for reference genome before variant calling

**Args:** `ref.fa ref`
**Explanation:** Creates the required .bci index file from the reference FASTA file, required preprocessing step before running the main variant calling pipeline.

### Generate low-frequency variant calls for pooled samples

**Args:** `-b pool.bam -r ref.fa -v lowfreq.vcf -f 0.05 -d 50`
**Explanation:** Detects rare variants present at 5% frequency in pooled sequencing data, requiring at least 50x depth to call such low-frequency alleles to suppress sequencing errors.