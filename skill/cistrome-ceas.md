---
name: cistrome-ceas
category: ChIP-seq Analysis / Epigenomics
description: A tool for analyzing ChIP-seq data to identify enriched genomic regions and annotate them with functional features. CEAS (Conserved Element Annotation System) computes enrichment statistics relative to genome annotations (genes, promoters, TES, exons, introns) and outputs summary reports in multiple formats.
tags: [chip-seq, enrichment, genomics, annotation, epigenetics, conservation]
author: AI-generated
source_url: https://cistrome.org/CEAS/
---

## Concepts

- **Input Requirements**: cistrome-ceas accepts mapped read files in BAM or BED format as primary input, along with a genome annotation file (typically in refgene format) that defines genomic features like promoters, exons, and introns. The genome annotation must match the assembly used for alignment (e.g., hg19, mm9).

- **Output Reports**: The tool generates multiple output files including `species.type` (feature distribution summary), `geneontology` (GO enrichment), and `dwonsample.txt` (detailed statistics). These reports summarize the spatial distribution of enriched regions relative to genomic annotations.

- **Genome Build Matching**: A critical requirement is that the input BAM/BED file and the genome annotation reference must be from the same genome build. Mismatched assemblies will produce meaningless or misleading enrichment statistics because chromosomal coordinates will not align.

- **Statistical Enrichment Calculation**: CEAS computes enrichment by comparing the observed overlap of ChIP peaks with genomic features against expected random distribution. It reports fold-enrichment and p-values for promoters, 5' UTRs, gene bodies, exons, introns, and intergenic regions.

## Pitfalls

- **Using Incorrect Genome Annotation**: Providing a genome annotation file from the wrong build (e.g., using hg38 annotation with hg19-aligned reads) produces completely invalid results since coordinate systems will be misaligned. Always verify the genome build matches between your alignment file and annotation reference.

- **Forgetting to Index BAM Files**: cistrome-ceas requires sorted and indexed BAM files for random access. Using unsorted or unindexed BAM files will cause the tool to fail or produce incomplete results, as it cannot efficiently retrieve reads from specific genomic regions.

- **Specifying Wrong Output Directory**: By default, output files are written to the current working directory. If this directory is not writable or is overwritten by another process, results will be lost. Always explicitly specify an output directory with the `--ofile` flag.

- **Confusing CEAS with Peak Callers**: cistrome-ceas is designed for post-peak-calling annotation and enrichment analysis. It does not perform peak calling itself. Running CEAS directly on raw sequencing reads without prior peak identification will yield no meaningful results.

## Examples

### Analyze ChIP-seq enrichment relative to gene features

**Args:** --bg=treatment.bed --refgene=hg19_refgene.txt --name=experiment1 --ofile=./output/
**Explanation:** This command analyzes a BED file of ChIP peaks against the hg19 RefGene annotation, computing enrichment relative to promoters, exons, introns, and intergenic regions, then writes results to the specified output directory.

### Generate enrichment report for histone modification

**Args:** --bg=H3K4me3_peaks.bed --refgene=mm9_refgene.txt --name=H3K4me3 --ofile=./histone_analysis/
**Explanation:** Analyzes H3K4me3 histone modification peaks in mouse (mm9 assembly) to determine promoter and 5' UTR enrichment patterns, which are expected to be highly enriched for this activating mark.

### Specify custom output filename prefix

**Args:** --bg=input.bed --refgene=hg38_refgene.txt --name=my_analysis --ofile=/results/
**Explanation:** The `--name` flag sets the prefix for all output files, making it easy to identify and organize results from multiple analyses run in the same directory.

### Analyze transcription factor binding site distribution

**Args:** --bg=TF_binding_sites.bed --refgene=hg19_refgene.txt --name=TF_occupancy --ofile=./tf_results/
**Explanation:** Computes the distribution of transcription factor peaks across genomic features, typically showing strong enrichment in promoters and proximal regulatory regions for sequence-specific TFs.

### Use BAM file directly as input

**Args:** --bg=aligned_reads.bam --refgene=hg19_refgene.txt --name=bam_input --ofile=./bam_output/
**Explanation:** When provided with a BAM file, cistrome-ceas will extract read locations and compute enrichment based on the raw signal rather than pre-called peaks, which can be useful for analyzing broad enrichment patterns.