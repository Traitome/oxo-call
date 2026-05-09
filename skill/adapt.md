---
name: adapt
category: Evolutionary Biology / Positive Selection Detection
description: A tool for detecting signatures of adaptive protein evolution in coding sequences using statistical methods to identify amino acid sites under positive selection.
tags:
  - positive-selection
  - dn-ds
  - codeml
  - evolutionary-biology
  - protein-evolution
  - statistical-tests
  - bioinformatics
  - phylogenetics
author: AI-generated
source_url: https://github.com/嗅觉生物学/adapt
---

## Concepts

- The tool analyzes ratios of nonsynonymous (dN) to synonymous (dS) substitutions to detect positive selection, where dN/dS > 1 indicates adaptive evolution at specific codon sites.
- Input alignments must be codon-aligned sequences in standard formats (FASTA, PHYLIP Interleaved) with an accompanying phylogenetic tree in Newick format to enable likelihood-based statistical inference.
- The statistical framework uses the Andersen-Nielsen test and likelihood ratio tests (LRT) to compute site-specific dN/dS ratios (ω) with confidence intervals derived from approximate chi-square distributions.

## Pitfalls

- Providing nucleotide alignments instead of codon-aligned sequences causes systematic errors in dN/dS estimation because the tool cannot correctly partition synonymous and nonsynonymous changes at each codon position.
- Specifying an incorrect evolutionary model (e.g., M0 when M2a or M8 is appropriate) leads to failure to detect episodic positive selection, resulting in false negatives for adaptive sites that experience selection only in specific lineages.
- Using insufficient sequence sampling (fewer than 10 sequences) produces unreliable likelihood estimates with wide confidence intervals, causing the tool to miss genuine positive selection signals or report false positives due to low statistical power.
- Specifying an unrooted phylogenetic tree causes the tool to fail with a segmentation fault when attempting to root ancestral sequences, as the branch-length optimization algorithm requires a rooted topology.
- Misinterpreting the output p-value as definitive evidence rather than a statistical approximation leads to overconfidence in results, especially when multiple testing corrections (Bonferroni or FDR) are not applied across the multiple codon sites tested.

## Examples

### Detect positive selection in a viral gene alignment
**Args:** -i HIVgag_codon.fasta -t tree.nwk -o results.txt -m M2a
**Explanation:** The M2a model (Model 2 Alternate) allows for a proportion of sites with ω > 1, making it suitable for detecting variable selective pressure across sites in viral sequences.

### Generate site-by-site selection scores with significance testing
**Args:** -i protein_alignment.fasta -t rooted_tree.nwk --model M8 --site-logical -p 0.05
**Explanation:** The M8 model (Model 8) with site-likelihood output provides posterior probabilities for each codon site being under positive selection, enabling identification of specific adaptive residues.

### Compare evolutionary models using likelihood ratio tests
**Args:** -i dataset.fasta -t species_tree.nwk --compare M0 M2a M8 --out lrt_results.csv
**Explanation:** The model comparison feature computes likelihood ratio test statistics between nested models to determine whether the more complex model significantly improves the fit, indicating the presence of positive selection.

### Analyze a subset of branches for lineage-specific selection
**Args:** -i alignment.fasta -t tree.nwk --branch-test --foregrounds Human_Clade -m M2
**Explanation:** Branch-site models identify lineages that have experienced accelerated evolution by testing whether specific branches in the phylogeny show elevated dN/dS ratios compared to the background branches.

### Export results in BED format for genome browser visualization
**Args:** -i coding_alignment.fasta -t tree.nwk -o adaptive_sites.bed --format bed --score posterior
**Explanation:** Exporting results in BED format allows visualization of positively selected sites in a genomic context, with the posterior probability score indicating the strength of evidence for each site.