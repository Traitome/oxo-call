---
name: bcftools-snvphyl-plugin
category: variant-calling/phylogenomics
description: A bcftools plugin for SNVPhyl (Single Nucleotide Variant Phylogenomics) that identifies single nucleotide variants from read alignments and builds multi-sample SNV matrices for downstream phylogenetic inference.
tags:
  - bcftools
  - snvphyl
  - snp-calling
  - phylogenomics
  - microbial-genomics
  - variant-detection
  - multi-sample-analysis
author: AI-generated
source_url: https://github.com/applied-genomics/bcftools-snvs
---

## Concepts

- **SNV identification from BAM alignments**: The plugin scans read alignments in BAM/CRAM files against a reference genome to call single nucleotide variants. It requires reads to be properly mapped, with base quality filtering and duplicate removal handled upstream. The core model uses a likelihood ratio test to distinguish true SNVs from sequencing or alignment errors.

- **Two-component workflow**: `bcftools snvphyl` (in bcftools view mode) calls variants per sample, while `bcftools snvphyl-build` aggregates per-sample VCFs into a multi-FASTA or SNV matrix where columns are samples and rows are positions. This separation allows incremental addition of new genomes to an existing matrix.

- **Reference dependency**: The reference genome must match the sequence the reads were aligned to. Any structural differences (rearrangements, insertions/deletions) between the alignment reference and the true sample sequence will manifest as false positive SNVs or missing call sites.

- **Output formats**: The build component emits multi-FASTA (one sequence per sample), PHYLIP (sequential or interleaved), or a compact SNP matrix format. Metadata such as the reference coordinate system and sample names are embedded or tracked separately.

- **Position reporting**: A position is reported in the output only when at least one sample carries a variant allele. Homoplasies (non-unique mutations) are flagged but not excluded by default, so downstream phylogenetic tools must handle them according to the evolutionary model in use.

## Pitfalls

- **Reference mismatch**: Passing the wrong reference genome (e.g., a close relative strain instead of the actual alignment reference) causes spurious SNV calls wherever the two references differ. Always verify that the `-r/--reference` FASTA is identical to the reference used during read alignment.

- **Ignoring repeat-mask regions**: Using `--repeat-mask` without a proper mask file, or omitting it entirely when working with repetitive genomes, leads to alignment-derived SNVs in low-complexity regions that are biologically uninformative and can mislead phylogenetic reconstruction.

- **Insufficient minimum depth**: Setting `--min-depth` too low (e.g., below 3–5) permits calls supported by only a handful of reads, increasing the false-positive SNV rate due to random sequencing errors. The optimal threshold depends on sequencing depth; for 30× average coverage, a minimum of 10 reads is a conservative starting point.

- **Inconsistent VCF input to snvphyl-build**: When building a matrix from multiple samples, all per-sample VCFs must be called against the **same reference coordinate system**. Mixing VCFs from different references (even with identical coordinate ordering) results in a misaligned multi-FASTA where variant positions are shuffled across samples.

- **Missing sample names in output**: If per-sample VCFs do not carry distinct sample names in the VCF header, the build component may label sequences as generic entries (e.g., "SAMPLE"), making it impossible to trace phylogenetic signals back to individual strains without post-hoc annotation.

- **Duplicate sample names**: Providing two or more VCF files with the same sample name to `snvphyl-build` silently overwrites earlier entries in the output matrix, producing a matrix with fewer sequences than expected and potentially biased downstream trees.

## Examples

### Call SNVs from a single BAM file using default settings
**Args:** `-r REFERENCE.fa -s SAMPLE_ID input.bcf`
**Explanation:** This invokes the snvphyl plugin via bcftools view, calling SNVs from input.bcf using the provided reference and assigning the output sample name SAMPLE_ID.

### Call SNVs with elevated base and mapping quality thresholds
**Args:** `-r REFERENCE.fa -s SAMPLE_ID --baseq 20 --mapq 20 input.bcf`
**Explanation:** Filtering reads by base quality 20 and mapping quality 20 reduces false-positive calls from sequencing errors or multimapping reads, at the cost of losing legitimate calls in low-quality regions.

### Call SNVs requiring a minimum read depth of 10
**Args:** `-r REFERENCE.fa -s SAMPLE_ID --min-depth 10 input.bcf`
**Explanation:** Requiring at least 10 reads covering a candidate site excludes low-coverage positions where random errors dominate the allele frequency estimate.

### Build a multi-FASTA SNV matrix from two per-sample VCFs
**Args:** `build -o snv_matrix.fasta sample1.vcf.gz sample2.vcf.gz`
**Explanation:** The build subcommand aggregates variant calls across samples, writing one FASTA sequence per sample with `N` for homozygous reference and the called allele for variants.

### Build a PHYLIP-formatted SNV matrix with relaxed variant inclusion
**Args:** `build --format phylip -o snv_matrix.phy --ref-allele-freq 0.25 sample1.vcf.gz sample2.vcf.gz`
**Explanation:** The PHYLIP output is suitable for standard phylogenetic tools (e.g., RAxML, IQ-Tree); the relaxed reference allele frequency allows heterozygous or low-frequency alleles to be included, which is relevant for mixed-culture or within-host population analyses.

### Call SNVs from multiple BAMs in parallel using GNU Parallel
**Args:** `-r REFERENCE.fa -s SAMPLE_ID --baseq 20 --min-depth 10 input.bcf`
**Explanation:** The plugin operates per-sample, so parallelizing over multiple BAMs with GNU Parallel dramatically reduces wall-clock time for large strain collections, provided each job gets adequate RAM.

### Exclude SNVs in repetitive regions using a repeat mask
**Args:** `-r REFERENCE.fa -s SAMPLE_ID --repeat-mask repeats.bed input.bcf`
**Explanation:** The repeat mask BED file instructs the plugin to skip or flag variant calls that overlap low-complexity or repetitive genomic regions, preventing alignment artifacts from corrupting the SNV matrix.

### Build a compact SNP matrix (binary presence/absence) for rapid distance estimation
**Args:** `build --matrix-out --format binary -o snv_binary.tsv sample1.vcf.gz sample2.vcf.gz`
**Explanation:** The binary matrix format encodes 0/1 for reference/variant alleles per position, enabling fast computation of pairwise SNV distances for initial screening of clonal relatedness before full phylogenetic analysis.