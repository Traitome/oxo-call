---
name: bifrost
category: Variant Graph Construction / Pangenomics
description: A tool for constructing variant graphs from a reference sequence and VCF/BCF variant files, enabling read alignment and variant genotyping on the graph structure.
tags: ['variant-graph', 'pangenome', 'vcf', 'bcf', 'genotyping', 'read-alignment', 'bioinformatics']
author: AI-generated
source_url: https://github.com/nedbat/bifrost
---

## Concepts

- **Variant Graph Structure**: Bifrost constructs a colored variant graph where the reference sequence forms the backbone and alternate alleles are embedded as non-reference paths. Each variant (SNP, indel, structural variant) becomes a bubble in the graph.
- **Input Formats**: The tool accepts FASTA for the reference genome and VCF/BCF files for variants. Variants must be annotated with their chromosome coordinates and must use the same genome build as the reference.
- **Graph Coloring**: Variants in the VCF can be colored with sample-level information, allowing downstream read-to-variant assignment and population-level pangenome analysis.
- **Read Alignment and Genotyping**: Once built, bifrost can align reads directly to the graph structure and perform variant calling by enumerating paths consistent with aligned reads.
- **Output Formats**: Bifrost outputs graph files in JSON format, variant calls in VCF/BCF, and alignment information in standard SAM format for integrated pipelines.

## Pitfalls

- **Reference-VCF Genome Build Mismatch**: Using a VCF with variants from a different genome build (e.g., GRCh37 vs GRCh38) causes graph construction to fail or produce meaningless variant bubbles at incorrect coordinates. Always verify genome build consistency before running bifrost.
- **Large VCF Files without Indexing**: Passing an unindexed VCF with hundreds of thousands of variants causes excessive memory usage and slow graph construction. Index VCF files with tabix before input.
- **Ignoring Ploidy Settings**: Bifrost assumes diploid genotypes by default. For haploid organisms (e.g., mitochondrial DNA) or polyploid species, not specifying the correct ploidy leads to incorrect allele frequency calculations and false heterozygous calls.
- **Overlapping Variant Handling**: Variants that overlap in the input VCF (e.g., conflicting indel and SNP at the same position) may cause graph construction errors if not resolved. Preview and resolve overlapping variants in the VCF beforehand.
- **Insufficient Disk Space for Temporary Files**: Building large variant graphs requires temporary disk space proportional to graph complexity. Running on a full disk causes incomplete graph output and potential data loss.

## Examples

### Build a simple variant graph from a reference and single VCF
**Args:** build --reference ref.fa --vcf variants.vcf --output graph.json
**Explanation:** This constructs a basic variant graph using the FASTA reference and a VCF file containing SNVs and indels. The output JSON represents the graph structure with reference and alternate paths.

### Build a variant graph with multiple VCF files for population-level analysis
**Args:** build --reference ref.fa --vcf sample1.vcf --vcf sample2.vcf --vcf sample3.vcf --output population_graph.json
**Explanation:** Merging multiple VCF files creates a colored pangenome graph where each sample's variants are assigned distinct colors, enabling population structure analysis.

### Align reads to an existing variant graph
**Args:** align --graph graph.json --reads reads.fq --output alignments.sam
**Explanation:** Aligns FASTQ reads to the pre-built variant graph, producing SAM-formatted alignments that indicate which paths (reference or alternate alleles) each read supports.

### Call variants from aligned reads on a graph
**Args:** call --graph graph.json --alignments alignments.scf --output called_variants.vcf
**Explanation:** Genotypes variants by enumerating read support for each graph path, outputting a VCF with allele depths and genotype likelihoods.

### Extract a genomic region from a variant graph for targeted analysis
**Args:** extract --graph graph.json --region chr1:1000000-2000000 --output region_graph.json
**Explanation:** Subsets the full graph to a specific chromosomal region, reducing memory requirements for targeted downstream analysis.

### Build a graph with explicit ploidy for haploid mitochondria
**Args:** build --reference mt_ref.fa --vcf mt_variants.vcf --ploidy 1 --output mt_graph.json
**Explanation:** Specifying ploidy as haploid ensures correct homozygous/heterozygous interpretation for mitochondrial DNA analysis.

### Filter low-quality variants before graph construction
**Args:** filter --input unfiltered.vcf --min-qual 30 --min-depth 10 --output clean.vcf
**Explanation:** Removes low-quality variant calls before graph construction, improving graph robustness and downstream genotyping accuracy.