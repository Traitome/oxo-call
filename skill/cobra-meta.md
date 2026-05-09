---
name: cobra-meta
category: Comparative genomics / Metagenomics
description: A bioinformatics tool for comparative meta-analysis of genomic data, typically used for cross-sample variant detection, consensus sequence generation, or pooled sequencing analysis. Operates on aligned SAM/BAM files or variant call sets to identify consistent mutations, shared haplotypes, or population-level patterns across multiple samples.
tags:
  - genomics
  - variant-analysis
  - metagenomics
  - comparative
  - bam
  - vcf
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cobra-meta
---

## Concepts

- **Input formats:** cobra-meta accepts sorted BAM files (binary aligned map), VCF/BCF variant files, or plain text mutation lists. For batch processing, provide multiple input files separated by spaces or use a file list (--filelist). The tool infers file format from extensions (.bam, .vcf, .bcf, .tab).

- **Reference-based operation:** Most cobra-meta operations require a reference genome FASTA file (--reference or -r). This is mandatory for BAM alignment interpretation, variant context resolution, and consensus generation. Index the reference with companion tool `cobra-meta-build` if not pre-indexed.

- **Output modes:** The tool supports four primary output modes controlled by --mode: (1) consensus — generates consensus sequences per sample; (2) variants — reports shared/unique variants across samples; (3) matrix — outputs allele frequency matrix; (4) report — produces HTML summary with statistics. Default is variants.

- **Sample weighting:** When processing pooled samples (e.g., metagenomic pools), use --weight to assign per-sample read depths or proportionality factors. Weights affect allele frequency calculations in matrix and consensus modes. Default weight is 1.0 per sample.

## Pitfalls

- **Missing reference index:** Running cobra-meta without pre-building the reference index causes immediate failure with "Fasta index not found" error. Always run `cobra-meta-build reference.fa` before analysis, producing reference.fa.fai and reference.fa.bwt.

- **Unsorted input BAM files:** Input BAM files must be coordinate-sorted (SAMtools sort -n is not acceptable). Passing incorrectly sorted BAMs produces silent inconsistencies in consensus mode and missing variant calls. Validate with `samtools flagstat input.bam`.

- **Conflicting output paths:** Using --output with a path that lacks write permissions or overwriting an existing file without --force will cause the tool to exit with "Permission denied" or "File exists" errors without generating output.

- **Memory exhaustion with large cohorts:** Processing >100 BAM files with default Java heap (--heap-size) leads to OutOfMemoryError. Increase heap with --heap-size 8g or higher for large cohorts, and consider using --chunk-size to parallelize.

- **Chromosome name mismatches:** If BAM/VCF chromosome names (e.g., "chr1" vs "1") do not exactly match the reference, all positions will show as missing. Normalize chromosome naming conventions across all inputs before running.

## Examples

### Generate consensus sequences from three BAM files
**Args:** --mode consensus --reference hg38.fa --input sample1.bam sample2.bam sample3.bam --output consensus/
**Explanation:** This creates consensus FASTA files for each sample based onread pileups at callable sites, using hg38 as the reference backbone.

### Identify shared variants between two VCF files
**Args:** --mode variants --reference hg38.fa --input cohort1.vcf cohort2.vcf --shared-only --min-depth 10
**Explanation:** This outputs variants present in both VCF files with minimum read depth of 10, useful for finding conserved mutations across conditions.

### Produce allele frequency matrix for metagenomic analysis
**Args:** --mode matrix --reference ref.fa --filelist samples.txt --weight 1.0 0.8 1.2 --min-af 0.05
**Explanation:** This generates a tab-delimited matrix of allele frequencies across all positions, respecting per-sample weights defined in the weight flag sequence.

### Generate HTML report with statistics summary
**Args:** --mode report --reference ref.fa --input *.bam --output summary.html --title "Pooled Analysis Run"
**Explanation:** This creates a self-contained HTML report with coverage histograms, variant density plots, and sample similarity metrics.

### Filter variants by quality and annotation
**Args:** --mode variants --input calls.vcf --min-qual 30 --filter "DP>20 && AF>0.1" --anno --output filtered.vcf
**Explanation:** This applies quality and expression-based filters, retaining only high-confidence variants and adding annotation columns to the output VCF.