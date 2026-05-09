---
name: chewiesnake
category: phylogenetic-analysis
description: A bioinformatics tool for bacterial core genome MLST (cgMLST) and phylogenetic analysis using allelic profiles. chewiesnake processes allele call matrices from chewBBACA or similar tools to compute phylogenetic trees, calculate pairwise genetic distances, and perform population genetics analyses. It supports multiple output formats and can integrate with Snakemake workflows for automated pipelines.
tags:
- bacterial-typing
- cgmlst
- phylogenetics
- allelic-profiles
- population-genetics
- chewbbaca
author: AI-generated
source_url: https://github.com/bfrgyio/chewiesnake
---

## Concepts

- **Allelic Profile Input**: chewiesnake accepts tab-separated files containing allele call matrices where rows represent isolates (or strains) and columns represent gene loci. Missing alleles should be encoded as 'N' or '0', and the first column must contain isolate identifiers. The tool treats each numeric value as a distinct allele designation, computing distances based on allelic differences across shared loci.

- **Distance-Based Phylogeny**: The core algorithm computes the number of allelic differences (also called pairwise SNP equivalents) between isolate pairs across all shared gene loci. The resulting distance matrix can be exported in multiple formats for downstream tree construction using packages like RapidNJ, Neighbor, or IQ-TREE. Distance calculation excludes loci where either isolate has missing data.

- **Output Formats**: chewiesnake generates multiple output files including: (1) a distance matrix in TSV format for tree building tools, (2) a summary statistics file with measures like mean pairwise distance and allelic diversity per locus, and (3) optional cluster files using thresholds defined by the user. The tool can also produce input for GrapeTree visualization.

- **Threshold-Based Clustering**: Users can specify one or more distance thresholds (e.g., `--threshold 10,50,100`) to assign isolates into clusters. An isolate is assigned to a cluster if its distance to at least one cluster member falls below the threshold. This approach approximates the hierarchical clustering used in conventional cgMLST schemes.

## Pitfalls

- **Inconsistent Missing Allele Encoding**: Using different placeholders for missing data across input files (e.g., 'N', '0', '-', '.') causes chewiesnake to treat them as distinct allele values rather than missing data, inflating pairwise distances. Always standardize missing allele representation to a single value before running the tool.

- **Loci with No Variation**: Including gene loci where all isolates have identical allele calls adds computational overhead without contributing to phylogenetic resolution. These monomorphic loci should be filtered prior to analysis using the `--min-alleles 2` flag or preprocessed, otherwise the distance matrix may contain inflated zero-difference entries that skew clustering results.

- **Incorrect Threshold Clustering Boundaries**: Setting a distance threshold too low (e.g., below the mean within-strain distance) results in every isolate forming its own singleton cluster, while an overly permissive threshold merges epidemiologically unrelated isolates into the same cluster. Always validate threshold selection against known reference strains or a validation dataset before interpreting population structure.

- **Mixed Allele Nomenclature**: If allele calls use different numbering schemes across datasets (e.g., one file uses sequential integers and another uses locus prefixes like "abc1234_1"), chewiesnake treats them as distinct alleles even when they represent the same gene variant. Preprocessing with standardized nomenclature is essential before combining datasets.

## Examples

### Computing a distance matrix from an allele call matrix

**Args:** `--input isolates_alleles.tsv --output chewie_distances.tsv`

**Explanation:** This command reads the allele call matrix file and computes pairwise allelic differences between all isolates, outputting a symmetric distance matrix in TSV format suitable for tree-building tools like RapidNJ or IQ-TREE.

### Generating a neighbor-joining tree

**Args:** `--input isolates_alleles.tsv --nj-tree tree.nwk --format newick`

**Explanation:** Instead of exporting the raw distance matrix, this command directly constructs a neighbor-joining tree using the built-in RapidNJ wrapper and saves it in Newick format for visualization in FigTree or iTOL.

### Applying multiple clustering thresholds

**Args:** `--input isolates_alleles.tsv --threshold 5,10,25,50 --cluster-output clusters/`

**Explanation:** This command assigns isolates to clusters at four different distance thresholds (5, 10, 25, and 50 allelic differences) and writes each threshold's clustering results to separate files in the specified output directory.

### Excluding monomorphic loci from analysis

**Args:** `--input isolates_alleles.tsv --output chewie_distances.tsv --min-alleles 2`

**Explanation:** The `--min-alleles 2` filter removes any gene loci where only one allele is present across all isolates, ensuring the distance calculation reflects true variation and reducing noisy zero-difference entries in the output matrix.

### Filtering isolates based on locus coverage

**Args:** `--input isolates_alleles.tsv --output chewie_distances.tsv --min-loci 500`

**Explanation:** This command excludes any isolate with fewer than 500 gene loci successfully called from the distance calculation, preventing isolates with excessive missing data from skewing pairwise distances and cluster assignments.

### Exporting population diversity statistics

**Args:** `--input isolates_alleles.tsv --stats diversity_stats.tsv --per-locus-stats`

**Explanation:** This command computes summary statistics including mean pairwise distance, allelic diversity per locus, and the number of unique alleles at each locus, writing the results to a tab-separated file for downstream epidemiological interpretation.

### Creating input for GrapeTree visualization

**Args:** `--input isolates_alleles.tsv --grapetree-input grapetree.tsv --distance-matrix dm.tsv`

**Explanation:** Exporting in GrapeTree format creates compatible input files for the MSTree viewer, allowing interactive visualization of minimum spanning trees with adjustable distance thresholds directly in a web browser.