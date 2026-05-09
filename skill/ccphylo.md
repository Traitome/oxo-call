---
name: ccphylo
category: Phylogenetics
description: A tool for coalescent-based phylogenetic inference and analysis, enabling users to reconstruct evolutionary histories from genetic sequences using species trees and gene tree methods.
tags: [phylogenetics, coalescent, evolutionary-biology, tree-inference, molecular-evolution]
author: AI-generated
source_url: https://github.com/ccphylo/ccphylo
---

## Concepts

- **Coalescent Framework**: ccphylo implements the multispecies coalescent model, which accounts for incomplete lineage sorting by modeling the history of gene copies as they coalesce backward in time within ancestral populations, providing more accurate species tree estimation than concatenated analyses.
- **Input Formats**: The tool accepts multiple sequence alignments in FASTA, NEXUS, or PHYLIP format, with optional population assignment files indicating which sequences belong to the same species, enabling proper modeling of within-species variation.
- **Output Options**: Results include species tree topology with branch lengths in coalescent units, posterior probabilities for clades from Bayesian inference, and gene tree distributions showing topological variation across loci.
- **Parallel Execution**: Large datasets can leverage multi-threaded processing for both heuristic tree searches and Bayesian MCMC chains, significantly reducing computation time on multi-core systems.

## Pitfalls

- **Missing Population Assignments**: Running the analysis without a proper population/species assignment file causes the tool to treat all sequences as independent samples, leading to incorrect coalescent modeling and severely biased species tree estimates with inflated branch lengths.
- **Mismatched Sequence Lengths**: Using alignments with unequal sequence lengths or excessive missing data triggers alignment errors or crashes; the tool requires all sequences to be of identical length with minimal gaps.
- **Ignoring Recombination**: Applying ccphylo to recombining sequences (e.g., nuclear genes in eukaryotes) violates the coalescent assumption of single-locus inheritance, producing spurious results where gene trees reflect recombination history rather than true genealogical relationships.
- **Insufficient Iterations**: Setting MCMC chain lengths too short leads to poor mixing and unreliable posterior estimates; convergence diagnostics (ESS > 200) should be verified before interpreting results.

## Examples

### Estimate a species tree from multiple gene alignments
**Args:** --input gene_alignments/ --output species_tree.tre --method bayesian
**Explanation:** This runs Bayesian multispecies coalescent inference across all gene alignments in the input directory, producing a topology with posterior probabilities for each clade.

### Run a quick maximum likelihood species tree search
**Args:** --input gene_alignments/ --output ml_tree.tre --method ml --threads 8
**Explanation:** This performs a fast heuristic maximum likelihood search for the species tree, using 8 CPU threads to parallelize the analysis and find the best-scoring topology.

### Calculate gene tree discordance statistics
**Args:** --species-tree species_tree.tre --gene-trees gene_trees.tre --discordance-output discordance.tsv
**Explanation:** This quantifies conflict between the species tree and individual gene trees, outputting statistics on topological discordance and identifying loci with high incongruence.

### Analyze a single gene alignment with population priors
**Args:** --input single_gene.fasta --populations pop_file.txt --output single_tree.tre --method bayesian
**Explanation:** This uses population assignment information to properly model the coalescent process within and between species for a single locus, improving branch length estimation.

### Generate a consensus tree from multiple runs
**Args:** --chains chain1.tre chain2.tre chain3.tre --consensus consensus.tre --burnin 1000
**Explanation:** This combines posterior tree samples from multiple independent MCMC runs, discarding the first 1000 trees as burn-in to produce a consensus topology with clade support values.

### Compute divergence times in coalescent units
**Args:** --input alignments/ --clock-rate 1e-8 --time-tree timetree.tre --method bayesian
**Explanation:** This estimates a timed phylogenetic tree using a molecular clock rate of 1e-8 substitutions per site per year, converting branch lengths from coalescent units to absolute time.