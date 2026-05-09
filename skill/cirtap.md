---
name: cirtap
category: Sequence Analysis / CRISPR
description: A bioinformatics tool for processing and analyzing CRISPR amplicon sequencing data. Cirtap performs read alignment, variant detection, and guide RNA efficiency analysis from amplicon-based sequencing experiments.
tags:
  - CRISPR
  - amplicon
  - sequencing
  - variant-calling
  - guide-rna
  - bioinformatics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cirtap
---

## Concepts

- **Input Formats**: Cirtap accepts FASTQ or BAM alignment files from CRISPR amplicon sequencing experiments, along with a target guide RNA sequence file in FASTA format for reference-based analysis.
- **Data Model**: The tool maintains an internal alignment matrix tracking edit distances, insertions, deletions, and substitutions relative to the guide RNA target sequence for each sequencing read.
- **Output Formats**: Primary outputs include a VCF file for detected variants, a summary CSV with per-guide efficiency scores, and an optional filtered BAM for downstream visualization.
- **Key Behavior**: By default, cirtap performs strict edit distance filtering (max distance ≤3) and considers only reads with perfect match at the PAM-proximal region for efficiency calculations.

## Pitfalls

- **Specifying incorrect reference sequences**: Using a guide RNA sequence with typos or wrong orientation will cause all reads to appear as mismatches, producing zero efficiency scores — this mimics poor guide design rather than actual experimental failure.
- **Omitting the PAM site specification**: Without explicit `--pam` flag, cirtap assumes NGG PAM by default; non-NGG Cas variants (e.g., Cas12a with TTTV) will be mis-analyzed, leading to incorrect variant calls.
- **Processing unfiltered BAM files**: Feeding raw alignment files with PCR duplicates inflates variant allele frequencies; always enable duplicate marking with `--dup-marking` or pre-filterduplicates.
- **Mismatching reference genome versions**: Using outdated or mismatched reference sequences causes coordinate misalignment in downstream analyses and produces false positive variants.

## Examples

### Analyze amplicon FASTQ file with default settings
**Args:** `--input reads.fastq --reference guider.fasta --output results/`
**Explanation:** Processes the FASTQ file against the provided guide RNA reference, outputs variant calls and efficiency scores to the results directory using default filtering parameters (max edit distance 3, NGG PAM).

### Specify alternative PAM motif for Cas12a experiments
**Args:** `--input amplicons.bam --reference guider.fasta --pam TTTV --output cas12a_results/
**Explanation:** Analyzes the BAM file with Cas12a's TTTV PAM motif instead of the default NGG, ensuring accurate variant detection for non-NGG CRISPR systems.

### Generate detailed per-read edit frequency table
**Args:** `--input reads.fastq --reference guider.fasta --verbose-edits --output edit_details.tsv`
**Explanation:** Produces a tab-separated file with every read's individual edit distance, position, and type for manual inspection beyond summary statistics.

### Adjust edit distance threshold for tolerant analysis
**Args:** `--input reads.fq --reference grna.fa --max-edit-dist 5 --output lenient_results/
**Explanation:** Increases the maximum allowed edit distance from default 3 to 5, useful for low-efficiency guides or highly variable cell populations.

### Enable duplicate marking for PCR-heavy samples
**Args:** --input amplicons.fastq --reference target.fa --dup-marking --output deduped_stats/
**Explanation:** Marks and excludes PCR duplicate reads before variant calling, preventing inflated allele frequency estimates from over-amplified templates.

### Export results in compressed VCF format
**Args:** --input reads.fastq --reference guider.fasta --vcf-compressed --output results.vcf.gz
**Explanation:** Generates a bgzip-compressed VCF file suitable for direct use with downstream tools like bcftools without requiring additional compression steps.

### Run parallel analysis on multiple samples
**Args:** --batch samples.txt --reference grna.fa --threads 8 --output batch_results/
**Explanation:** Processes multiple samples listed in samples.txt in parallel using 8 threads, suitable for high-throughput CRISPR screening datasets.
---