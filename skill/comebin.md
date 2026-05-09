---
name: come-bin
category: binning
description: A metagenomic binning tool that clusters assembled contigs into draft genomes using k-mer composition profiles and differential coverage patterns from read alignments.
tags:
  - metagenomics
  - binning
  - assembly
  - microbial
author: AI-generated
source_url: https://github.com/refresh-bio/come-bin
---

## Concepts

- **Input Requirements**: Comebin requires assembled contigs in FASTA format and read-alignment coverage data (BAM/SAM format). Both inputs are essential because the tool bins contigs by combining sequence composition (k-mer signatures) with abundance patterns across multiple samples.
- **Coverage Profile Calculation**: The tool computes per-contig coverage by parsing BAM/SAM files to extract read depth and breadth statistics. Multiple samples yield differential coverage profiles, which are more discriminative for binning closely related strains.
- **Output Formats**: Comebin generates a clustering file containing two columns (contig ID and bin assignment number) plus an optional stats file with within-bin statistics such as average coverage and k-mer profile variance.
- **Algorithm Basis**: The binning algorithm groups contigs based on Euclidean distance between combined feature vectors (k-mer frequencies + normalized coverage), using hierarchical clustering with an adaptive distance threshold.
- **Companion Binary**: `comebin-build` constructs k-mer profile indexes from reference genomes or training contigs to improve binning accuracy when reference-based initialization is desired.

## Pitfalls

- **Insufficient Read Depth**: Metagenomic samples with low sequencing depth produce noisy coverage estimates, causing comebin to merge unrelated contigs into single bins. Consequence: inflated bins with low completion and high contamination metrics.
- **K-mer Size Mismatch**: Using a k-mer size that is too small generates similar profiles for unrelated sequences (e.g., k=3), while k-mer sizes too large (e.g., k=31) produce sparse profiles with many zero counts. Consequence: bins with mixed species or fragmented single genomes.
- **Unpaired Input Files**: Providing BAM files that are not coordinate-sorted or do not contain MD tags causes coverage computation to fail silently or produce zero values. Consequence: all contigs receive identical fake coverage, resulting in bins based only on composition.
- **Ignoring Fragment Length**: When computing coverage, the tool accounts for read length automatically, but用户提供 providing reads shorter than 50 bp increases variance in per-contig depth estimates. Consequence: unstable bin assignments across replicates.
- **Single-Sample Limitation**: Running comebin on one sample without biological replicates eliminates differential coverage signal, degrading performance for strain-level binning. Consequence: species-level bins merge highly similar strains.

## Examples

### Binning contigs with default settings from a single metagenome
**Args:** assemble contigs.fasta mapreads.bam output_dir
**Explanation:** This runs comebin with default k-mer size (k=4) and automatic distance threshold on one BAM file, suitable when quick initial results are needed.

### Binning with differential coverage using multiple samples
**Args:** assemble contigs.fasta sample1.bam sample2.bam sample3.bam output_dir
**Explanation:** Providing three or more BAM files enables differential coverage binning, dramatically improving resolution for closely related strains in complex communities.

### Specifying custom k-mer size for higher resolution
**Args:** assemble contigs.fasta mapreads.bam output_dir -k 6
**Explanation:** Increasing k-mer size to 6 captures more detailed sequence signatures, useful for datasets with high genomic diversity and few repeats.

### Building k-mer index for reference-assisted binning
**Args:** come-bin-build ref_genomes.fa index_dir -k 5
**Explanation:** The companion binary `come-bin-build` creates an index from known reference genomes to initialize binning, which improves accuracy when partial references are available.

### Adjusting clustering distance threshold for tighter bins
**Args:** assemble contigs.fasta mapreads.bam output_dir -d 0.15
**Explanation:** Lowering the distance threshold to 0.15 produces more bins with fewer contigs per bin, useful when expecting high species richness and wanting to avoid cross-species merges.

### Exporting CheckM-compatible bin directories
**Args:** assemble contigs.fasta mapreads.bam output_dir --format bins
**Explanation:** The `--format bins` flag writes each bin as a separate FASTA file in a `bins/` subdirectory, which can be directly evaluated by CheckM for completion and contamination estimates.