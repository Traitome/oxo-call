---
name: coconet-binning
category: Metagenomics / Binning
description: A coverage-based binning tool that groups metagenomic assembly contigs into taxonomic bins using coverage profiles and compositional signatures. Integrates with the CoCoNet pipeline for metagenome analysis.
tags: metagenomics, binning, assembly, coverage, contigs, taxonomic-binning
author: AI-generated
source_url: https://github.com/metanetgenomics/coconet-binning
---

## Concepts

- **Input Format**: coconet-binning accepts two main input types: (1) coverage files in BED or tab-separated format with columns for contig name, coverage value, and optional depth metrics, and (2) FASTA/FASTQ files containing assembled contigs. The coverage file defines the abundance profile for each contig across samples.
- **Binning Algorithm**: The tool uses a combination of coverage abundance profiles and tetranucleotide frequency signatures to cluster contigs into bins. Coverage vectors from multiple samples provide co-abundance patterns, while k-mer compositions capture genomic signatures. This dual-approach improves bin separation for complex metagenomes.
- **Output Files**: The tool produces three output files: (1) a contigs-to-bins mapping file with one contig per line and its assigned bin ID, (2) a bin statistics file containing mean coverage, length, and GC content per bin, and (3) an optional FASTA file with contigs grouped by bin.
- **Compatibility**: coconet-binning is designed to work with assemblies from common metagenomic assemblers such as metaSPAdes, MEGAHIT, and Canu. It requires coverage values calculated from read mapping with tools like BWA-MEM or Bowtie2.

## Pitfalls

- **Insufficient Sample Depth**: Running binning with fewer than 3 samples severely reduces co-abundance signal quality, resulting in merged bins or split true genomes. Always use coverage from multiple samples (>3 recommended) for reliable binning.
- **Mismatched Contig Names**: If contig names in the coverage file do not exactly match those in the FASTA file (including case and special characters), the tool silently drops those contigs from analysis. Verify name matching before running.
- **Inconsistent Coverage Scale**: Using coverage values from different mapping runs without normalizing to reads-per-base or aligning to the same total read count introduces bias, causing poor bin separation. Always normalize coverage inputs.
- **Low-Complexity Regions**: Including contigs with very low GC variation or repetitive sequences leads to false bin associations. Filter out contigs shorter than the specified minimum length (default 1000 bp) before binning.
- **Missing Bin Statistics**: Not reviewing the bin statistics output can miss low-quality bins with abnormal coverage or extremely low contig counts, which may represent contamination or chimeras.

## Examples

### Basic binning with a single coverage file
**Args:** --coverage assembly_coverage.tsv --contigs assembled_contigs.fa --output my_bins
**Explanation:** Runs binning using one coverage profile and the assembled contigs, outputting results to the my_bins directory. Works for simple datasets but provides limited co-abundance signal.

### Binning with multiple sample coverage files
**Args:** --coverage sample1_cov.tsv --coverage sample2_cov.tsv --coverage sample3_cov.tsv --contigs assembled.fa --output multi_sample_bins
**Explanation:** Provides three coverage profiles from different samples, enabling robust co-abundance clustering. Multiple samples dramatically improve bin quality for complex metagenomes.

### Adjusting minimum contig length threshold
**Args:** --coverage cov.tsv --contigs contigs.fa --min-length 2000 --output filtered_bins
**Explanation:** Sets the minimum contig length to 2000 bp, excluding shorter contigs from binning. Reduces noise from fragmented assemblies and improves bin completeness.

### Using custom k-mer size for tetranucleotide frequencies
**Args:** --coverage cov.tsv --contigs contigs.fa --kmer-size 6 --output kmer6_bins
**Explanation:** Uses 6-mers instead of default 4-mers for compositional signatures. Larger k-mers capture more specific patterns but require longer contigs for reliable frequency estimation.

### Running with automatic bin number detection
**Args:** --coverage cov.tsv --contigs contigs.fa --auto-bin --output auto_bins
**Explanation:** Enables automatic bin number estimation using cluster validation metrics. The tool determines the optimal number of bins rather than requiring a user-specified count.

### Generating binned contig FASTA files for downstream analysis
**Args:** --coverage cov.tsv --contigs contigs.fa --output-bins-fasta --output bin_fa
**Explanation:** Produces individual FASTA files for each bin, each containing all contigs assigned to that bin. Useful for bin-specific annotation or MAG reconstruction.

### Suppressing tetranucleotide frequency use
**Args:** --coverage cov.tsv --contigs contigs.fa --coverage-only --output cov_only_bins
**Explanation:** Disables compositional binning and uses only coverage profiles for clustering. Use this when contigs are too short for reliable k-mer frequency estimation.

### Setting a specific number of output bins
**Args:** --coverage cov.tsv --contigs contigs.fa --num-bins 50 --output targeted_bins
**Explanation:** Forces the algorithm to produce exactly 50 bins. Use when downstream analysis requires a fixed number of bins or when sample complexity is known.

### Running with verbose logging for debugging
**Args:** --coverage cov.tsv --contigs contigs.fa --output debug_bins --verbose
**Explanation:** Enables detailed logging output, showing clustering iterations and statistics. Essential for troubleshooting poor binning results or understanding algorithm behavior.