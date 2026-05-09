---
name: binchicken
category: Metagenomics / Binning
description: A metagenomic binning tool that clusters assembly contigs into draft metagenome-assembled genomes (MAGs) using coverage and sequence composition signals.
tags: [metagenomics, binning, MAGs, assembly, contigs, clustering]
author: AI-generated
source_url: https://github.com/ctgreyhound/binchicken
---

## Concepts

- binchicken takes assembled contigs (FASTA) and mapped read alignments (BAM/CRAM) as primary inputs to compute per-contig coverage depth and variance metrics, which are critical signals for distinguishing species-level bins.
- The tool extracts k-mer frequency profiles (default k=4) from contig sequences to encode nucleotide composition, complementing coverage-based clustering by capturing genomic DNA signatures independent of sequencing depth.
- Clustering proceeds in two phases: an initial over-clustering step that groups contigs loosely by composition, followed by iterative refinement that splits or merges clusters based on coverage coherence and single-copy gene marker presence.
- Output consists of cluster assignment files mapping each contig header to a numeric bin identifier, which can be converted to per-bin FASTA files using companion bin extraction utilities.

## Pitfalls

- Providing contigs shorter than the minimum length threshold (default 1000 bp) causes binchicken to silently exclude them from clustering, resulting in missing contigs from output bins and incomplete MAGs.
- Using an incorrect reference genome for read mapping (e.g., a contaminated or chimeric assembly) propagates coverage outliers that distort bin boundaries, producing chimeras or over-split genomes in the final results.
- Failing to specify read group tags in the BAM file causes coverage calculations to average across all reads without sample separation, reducing the tool's ability to distinguish closely related strains.
- Running binchicken without single-copy gene annotation enabled skips consistency checking, allowing bins with missing or duplicated essential genes to pass as valid MAGs.
- Specifying an excessive number of threads with insufficient memory per thread triggers OOM failures during k-mer matrix computation for large assemblies, particularly when contig N50 exceeds 10 kb.

## Examples

### Cluster contigs using default k-mer and coverage signals
**Args:** `contigs.fasta alignments.bam --outdir binchicken_output`
**Explanation:** This runs binchicken with default settings, using k-mer composition and read coverage to cluster contigs, writing results to the specified output directory.

### Specify a higher minimum contig length for conservative binning
**Args:** `contigs.fasta alignments.bam --min-length 2000 --outdir conservative_bins`
**Explanation:** Increasing the minimum contig length to 2000 bp excludes shorter contigs that may introduce noise in clustering, yielding fewer but higher-quality bins.

### Enable single-copy gene validation for MAG completeness
**Args:** `contigs.fasta alignments.bam --check-scg --outdir scg_validated`
**Explanation:** Adding the single-copy gene check flag instructs binchicken to score each bin against universal marker genes, flagging incomplete or duplicated genomes in the output report.

### Use a custom k-mer size for AT-rich or GC-biased metagenomes
**Args:** `contigs.fasta alignments.bam --kmer-size 6 --outdir custom_kmer`
**Explanation:** Changing k-mer size to 6 captures longer nucleotide patterns that better discriminate composition in metagenomes with extreme GC bias or repetitive sequence elements.

### Limit memory usage for large assemblies on constrained hardware
**Args:** `contigs.fasta alignments.bam --max-memory 32G --threads 8 --outdir memory_limited`
**Explanation:** Restricting memory to 32 GB with 8 threads forces binchicken to process the assembly in chunks, preventing OOM crashes while slowing overall runtime.

### Produce per-bin FASTA files for downstream annotation
**Args:** `contigs.fasta binning_clusters.tsv --generate-bins --outdir fasta_bins`
**Explanation:** Passing the cluster assignment file alongside the contigs and the bin generation flag writes individual FASTA files for each bin, ready for gene prediction or taxonomic classification.