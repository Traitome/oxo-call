---
name: cnvkit
category: genomics (copy number variation)
description: A Python toolkit for analyzing copy number variations (CNVs) from high-throughput sequencing data. It transforms coverage maps from BAM or CRAM files into copy number estimates, visualizes results, and identifies genomic regions with copy number alterations.
tags: [cnv, copy number, variant calling, genomics, sequencing, coverage, ngs, bioinformatics]
author: AI-generated
source_url: https://cnvkit.readthedocs.io/
---

## Concepts

- **CNVKit operates on a three-stage workflow**: First, it collects read coverage in target and anti-target regions using a reference genome (`batch` command). Second, it normalizes and segments the coverage data to produce copy number ratio files (`.cnr`). Third, it genotypes each sample to identify specific copy number states (`call` command).
- **Input formats are specificity-dependent**: Target regions must be provided as a BED file with chromosome, start, and end columns. The reference genome must be in FASTA format and indexed. Sample BAM/CRAM files must be coordinate-sorted and indexed. GC content correction requires a pre-computed `.gc` reference file generated from the target regions.
- **Output artifacts follow a predictable naming convention**: Coverage reference files (`.cnn`) are generated per target and anti-target. Copy number ratio files (`.cnr`) contain log2 ratios and copy number states. Segmented files (`.cns`) contain discrete copy number calls per genomic bin. The `.call` column in outputs uses integers where 0 = deletion, 2 = diploid, 3 = gain.
- **Companion binaries extend functionality**: `cnvkit.py` is the main entry point, but `cnvkit-export` (via the `export` subcommand) converts results to formats like VCF, SEG, or Theta. Visualization tools include `heatmap` and `scatter` for plotting copy number profiles across samples.

## Pitfalls

- **Using mismatched reference genomes between BAM files and the provided FASTA**: If the alignment reference differs from the target BED file coordinate system, all coordinates will be offset, producing nonsense copy number estimates. Always verify that the BAM header @SQ lines match the reference genome name and length.
- **Forgetting to generate anti-target (antitarget) regions**: Without anti-targets, CNVKit cannot compute background coverage for the "off-target" reads that fill gaps between exons. Skipping antitarget creation leads to inflated copy number noise in regions between targets and reduced sensitivity for detecting focal amplifications.
- **Applying batch results without a paired normal reference**: Running `batch` without the `--normal` flag uses all samples as a pooled reference, which dilutes real copy number variants present in multiple samples. For tumor samples, always provide a pool of normals or a specific normal sample to establish the diploid baseline.
- **Calling copy number variants without segmenting first**: The `call` command works on raw `.cnr` files without segmentation, creating noisy calls with hundreds of tiny segments. Always run the `segment` command (or use `batch --segment`) before `call` to produce clinically interpretable copy number states.

## Examples

### Generate coverage reference files from target and anti-target BED files
**Args:** autobin