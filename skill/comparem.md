---
name: comparem
category: Genome Comparison and Phylogenomics
description: A tool for comparing microbial genomes and computing Average Amino Acid Identity (AAI) to assess genomic similarity and phylogenetic relationships. Comparem takes genome assemblies or gene catalogs and produces pairwise similarity scores, distance matrices, and hierarchical clustering for taxonomy and phylogenomics workflows.
tags:
  - genome-comparison
  - AAI
  - ANI
  - phylogenomics
  - microbial-genomics
  - sequence-similarity
  - clustering
  - taxonomy
author: AI-Generated
source_url: https://github.com/merenlab/comparem
---

## Concepts

- Comparem computes Average Amino Acid Identity (AAI) by aligning reciprocal best hits between protein-coding genes of genome pairs. It uses DIAMOND or BLAST-style alignment underneath, scoring amino acid identities at each position and averaging across aligned gene pairs to yield a genome-level similarity percentage (typically 60-100% range区分 closely related strains from distantly related species).
- Input files are genome assemblies in FASTA format (nucleotide) or pre-indexed gene catalogs. Comparem requires amino acid sequences for AAI computation, so nucleotide genomes are first translated in all six reading frames using Prodigal before comparison. Gene calls are stored in internal JSON format for downstream scoring.
- Output includes a pairwise similarity matrix (CSV/TSV), a dendrogram Newick file for hierarchical clustering, and log files detailing which gene pairs contributed to each score. The matrix can be imported directly into phylogenetics tools like FastTree or IQ-TREE for tree building.
- Comparem supports batch processing where you specify an input directory containing multiple genome FASTA files. It automatically generates all pairwise comparisons, producing an N×N distance matrix for N genomes. Runtime scales quadratically with genome count, so consider subsampling for large sets (>50 genomes).
- The tool distinguishes between "core" gene sets (shared across all genomes) and "accessory" genes when calculating composite scores. A minimum hit coverage threshold (default 70%) filters out fragmented or truncated alignments that would artificially deflate AAI values.

## Pitfalls

- Feeding nucleotide FASTA files without coding sequence annotation causes Comparem to attempt translation in all six frames, which is slow and often yields incorrect gene models. Always provide annotated genomes (with GenBank/EMBL features) or use the companion tool to pre-call genes with Prodigal before comparison.
- Setting the minimum amino acid identity threshold too high (e.g., 98%) when comparingdraft assemblies from short-read data results in zero hits for all but near-identical strains. For cross-species AAI, values of 60-80% are more appropriate, while intra-species ANI typically requires 95-99%.
- Running Comparem on genomes with high levels of contamination or contigs from multiple species skews the AAI matrix because contaminating sequences introduce non-homologous gene pairs. Always screen input assemblies with CheckM or QUAST first to estimate genome completeness and contamination.
- Forgetting to index the database with `comparem-annotate` before batch comparison causes Comparem to re-run gene calling and indexing for each genome repeatedly, multiplying runtime by N. Pre-indexing all genomes once saves hours in large batch runs.
- Using DIAMOND in sensitive mode without adjusting gap open/extension penalties for divergent genomes causes alignment trimming and underestimates AAI for distantly related pairs. The default comparem parameters assume reasonably close genomes; for deep phylogenetic comparisons, increase the e-value cutoff and gap penalties.

## Examples

### Compare two microbial genome assemblies and output AAI scores
**Args:** `aai -1 genome_A.fna -2 genome_B.fna -o pairwise_results.csv`
**Explanation:** This computes the Average Amino Acid Identity between two annotated genome FASTA files and writes the pairwise score to a CSV for downstream interpretation.

### Batch compare all genomes in a directory with pre-indexed gene calls
**Args:** `aai -I genomes_index/ -o all_vs_all_matrix.tsv --pre-indexed`
**Explanation:** The pre-indexed flag skips gene calling and translation steps for all genomes in the specified directory, producing a complete N×N similarity matrix in one run.

### Generate a phylogenetic tree from genome AAI values
**Args:** `cluster -M distance_matrix.tsv -o tree.newick --method ward`
**Explanation:** This takes an existing distance matrix and produces hierarchical clustering with Ward linkage, outputting a Newick-formatted tree suitable for visualization in FigTree or iTOL.

### Adjust alignment stringency for divergent genome pairs
**Args:** `aai -1 distant_genome_1.fna -2 distant_genome_2.fna --min-coverage 50 --e-value 1e-5 -o aai_divergent.csv`
**Explanation:** Lowering the coverage threshold and increasing the e-value tolerance allows more fragmented alignments to count, preventing zero AAI scores for truly related but divergent genomes.

### Parallelize batch comparison across multiple CPU cores
**Args:** `aai -I genome_directory/ -o batch_matrix.tsv --pre-indexed --threads 16`
**Explanation:** Specifying 16 threads enables parallel processing of pairwise comparisons, significantly reducing wall-clock time when comparing large genome sets on multi-core machines.

---