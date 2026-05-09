---
name: coinfinder
category: Comparative Genomics
description: Find conserved or coinherited genes across multiple bacterial genomes to identify core genomes, phylogenetic markers, and potential vaccine candidates.
tags:
  - gene conservation
  - core genome
  - pangenome
  - phylogenetics
  - bacterial genomics
author: AI-generated
source_url: https://github.com/dsh肢/coinfinder
---

## Concepts

- **Gene presence/absence matrix**: coinfinder analyzes multiple genomes and builds a binary matrix indicating which genes are present (1) or absent (0) in each genome. This forms the basis for identifying conserved and accessory genes.
- **Input formats**: Accepts genome annotations in GFF3, GenBank (GBK), or FASTA formats. Each genome should be in a separate file or directory, and gene calls must include unique locus tags.
- **Coinheritance detection**: Identifies genes that show identical presence/absence patterns across genomes, suggesting functional linkage or co-evolution. The tool uses statistical clustering to group genes with similar patterns.
- **Output provides multiple matrices**: Core genes (present in all genomes), accessory genes (present in subset), and gene clusters with their prevalence statistics across the input genomes.
- **Statistical significance**: UsesFisher's exact test or similar to determine if gene co-occurrence is statistically significant, filtering out random associations.

## Pitfalls

- **Insufficient genome diversity**: Using fewer than 5-10 closely related genomes produces unreliable statistics and may identify only highly conserved genes without meaningful accessory patterns.
- **Incompatible annotation versions**: Mixing genomes annotated with different gene callers or annotation pipelines produces inconsistent locus tags, causing genes to be mislabeled as absent when they're actually present but differently annotated.
- **Ignoring paralogs**: Failing to handle gene duplications (paralogs) incorrectly inflates core gene counts since coinfinder treats each copy as a separate gene; use preprocessing to collapse paralog families.
- **Overinterpreting absence as true absence**: Gene absence in draft assemblies may represent assembly gaps rather than true biological absence, particularly in repetitive or GC-rich regions.
- **Inconsistent genome quality**: Mixing high-quality complete genomes with draft assemblies biases the analysis toward genes in complete genomes appearing as "core" while others appear falsely accessory.

## Examples

### Identify core genes shared across all input genomes

**Args:** -i ./genomes -o core_genes --core --threshold 1.0

**Explanation:** This runs coinfinder to find genes present in 100% of the input genomes, outputting a core gene list for downstream phylogenetic analysis.

### Find accessory genes present in 50-90% of strains

**Args:** -i ./genomes -o accessory_genes --accessory --min 0.5 --max 0.9

**Explanation:** Identifies genes present in a subset of genomes (accessory genome) with prevalence between 50% and 90%, useful for identifying strain-specific virulence factors.

### Generate gene presence/absence matrix in tabular format

**Args:** -i ./genomes -o matrix.tsv --matrix --format tsv

**Explanation:** Exports the complete gene presence/absence matrix as a TSV file for import into R or Python for custom visualization and downstream statistical analysis.

### Filter results to statistically significant coinherited gene clusters

**Args:** -i ./genomes -o coinherited --coinheritance --pvalue 0.05

**Explanation:** Identifies gene clusters showing significant coinheritance patterns with statistical filtering using the specified p-value threshold.

### Generate phylogenetic tree based on core gene concatenations

**Args:** -i ./genomes -o phylogeny.tree --core --concat --tree FastTree

**Explanation:** Uses identified core genes to build a concatenated alignment and infers a phylogenetic tree using FastTree for speciestree or strain relationship visualization.