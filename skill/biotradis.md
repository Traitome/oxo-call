---
name: biotradis
category: functional_genomics
description: Analyze transposon insertion sequencing (TraDIS) data to identify essential genes, insertion sites, and mutation frequencies in bacterial genomes.
tags: tradis, transposon, insertion_sequencing, essential_genes, bacterial_genomics, mutagenesis
author: AI-generated
source_url: https://github.com/sanger-pathogens/BioTradis
---

## Concepts

- BioTradis processes sorted BAM files from TraDIS experiments, where each read represents a transposon insertion site. The tool maps insertions to the reference genome and identifies genomic regions with high or low insertion frequencies.
- All BioTradis tools require a GFF3 annotation file to map transposon insertions to specific genes and genomic features. Without a proper GFF3, gene-level summaries will be absent from outputs.
- Insertion frequency is calculated as reads per kilobase per million reads (RKPM), normalized to the total number of reads in the dataset. This normalization allows comparison between different samples.
- The tool can analyze multiple samples simultaneously using the --samples flag, enabling comparative analysis between conditions or replicates.

## Pitfalls

- Using unsorted BAM files will cause all BioTradis tools to fail silently or produce incorrect results. Always ensure input BAM files are coordinate-sorted using tools like samtools sort.
- Specifying an incorrect or outdated GFF3 annotation file will lead to misattributed gene names and incorrect feature counts in output tables.
- Running BioTradis on BAM files with duplicate reads that have not been properly marked can artificially inflate insertion counts at specific sites.
- Using reference genomes with multiple contigs without setting appropriate flags may result in incomplete analysis of insertions in smaller scaffolds or plasmids.
- Failing to set an adequate read cutoff (--minreads) can include statistical noise from low-frequency artifacts, while setting it too high may exclude genuine rare insertions.

## Examples

### Identify insertion sites in a BAM file
**Args:** --bam sample.bam --gff annotation.gff3 --output insert_sites
**Explanation:** Maps all transposon insertions in the BAM file to genomic coordinates and outputs a table with insertion positions, counts, and associated genes.

### Generate an insertion plot for a genomic region
**Args:** --bam sample.bam --gff annotation.gff3 --gene geneA --plot geneA_insertions.png
**Explanation:** Creates a visualization showing the distribution of transposon insertions within and around a specified gene, useful for validating essential regions.

### Compare insertions between two conditions
**Args:** --bam treat.bam,control.bam --gff annotation.gff3 --samples treated,control --compare --output comparison
**Explanation:** Compares insertion frequencies between treated and control samples, identifying genes with significantly different insertion patterns.

### Summarize insertions per gene across the genome
**Args:** --bam sample.bam --gff annotation.gff3 --genes --output gene_summary
**Explanation:** Produces a tab-separated file with each gene's total insertions, RKPM-normalized values, and gene annotations for downstream analysis.

### Generate a condensed insertion profile
**Args:** --bam sample.bam --gff annotation.gff3 --condense --output condensed_profile --minreads 10
**Explanation:** Condenses individual insertion sites into gene-level summaries, filtering out sites with fewer than 10 reads to reduce noise.

### Analyze insertions across multiple samples with a specified cutoff
**Args:** --bam sample1.bam,sample2.bam,sample3.bam --gff annotation.gff3 --samples S1,S2,S3 --minreads 50 --output multisample_analysis
**Explanation:** Analyzes three replicates together with a minimum read threshold of 50, outputting combined statistics for each gene across all samples.

### Extract insertions in a specific genomic region
**Args:** --bam sample.bam --gff annotation.gff3 --region chr1:100000-200000 --bedout region_insertions.bed
**Explanation:** Extracts all transposon insertions within a defined chromosomal region and saves them in BED format for use in other tools.

### Generate a genome-wide insertion density plot
**Args:** --bam sample.bam --gff annotation.gff3 --genomeplot --output genome_wide.png
**Explanation:** Creates a circular or linear genome-wide plot showing insertion density across the entire chromosome, highlighting hot and cold spots.