---
name: bcgtree
category: Phylogenetics / Variant Analysis
description: Constructs phylogenetic trees from VCF/BCF variant call data by analyzing haplotype relationships and genetic distances
tags:
  - vcf
  - bcf
  - phylogenetics
  - tree
  - genetics
  - haplotype
  - variant-calling
author: AI-generated
source_url: https://github.com/samtools/bcftools
---

## Concepts

- **Input formats**: bcgtree accepts both VCF (text) and BCF (binary) files containing variant calls from any variant callers (GATK, FreeBayes, etc.). Files must have valid sample columns in the FORMAT field.
- **Tree construction algorithm**: The tool computes pairwise genetic distances between haplotypes using variant allele frequencies and constructs a neighbor-joining or UPGMA tree based on these distances.
- **Output format**: bcgtree produces Newick-format tree files, which are compatible with most phylogenetic visualization tools (FigTree, iTOL, DendroPy).
- **Sample filtering**: The tool can filter samples based on read depth, genotype quality (GQ), or missingness percentage using inline expressions, improving tree robustness.
- **Reference alignment**: Input VCF/BCF files must be aligned to the same reference genome; bcgtree uses thecontig names and positions to anchor variant sites.

## Pitfalls

- **Using unfiltered variant data**: Feeding raw, unfiltered VCF files into bcgtree introducesfalse positive variants that distort the tree topology, often pulling distantly related samples together artificially.
- **Mismatched chromosome naming**: If the VCF header uses "chr1" but the tool's reference configuration uses "1", bcgtree fails to locate variant sites and produces an empty tree.
- **Insufficient variant density**: With fewer than ~100 shared variant sites between sample pairs, genetic distances become unreliable, resulting in low-support tree branches.
- **Missing sample genotypes**: Samples with excessive missing calls (default threshold 10%) cause bcgtree to exclude them silently, leading to incomplete trees without warning.

## Examples

### Build a phylogenetic tree from a filtered VCF file

**Args:** --samples - --filter "GQ >= 30 && DP > 10" input.vcf.gz output.newick

**Explanation:** This filters variants by genotype quality and read depth before tree construction, ensuring only high-confidence genotypes contribute to the phylogeny.

### Construct a tree from a BCF file using neighbor-joining

**Args:** -s sample_list.txt -m nj input.bcf.gz tree.newick

**Explanation:** Using a sample list restricts the tree to specified samples, and the -m nj flag selects neighbor-joining algorithm for tree reconstruction.

### Build a tree with a minimum variant support threshold

**Args:** --min-variants 50 input.vcf.gz output.newick

**Explanation:** Requiring at least 50 shared variants ensures statistical reliability; trees built with fewer variants have low bootstrap support.

### Exclude samples with high missingness from tree building

**Args:** --max-missing 0.05 input.vcf.gz tree.newick

**Explanation:** Setting max-missing to 5% excludes samples with >5% missing genotypes, preventing them from biasing genetic distance calculations.

### Generate a tree from multiple VCF files merged by sample

**Args:** --merge-samples *.vcf.gz output.newick

**Explanation:** Merging multiple VCF files by sample name combines genotype data across cohorts before constructing a combined phylogenetic tree.