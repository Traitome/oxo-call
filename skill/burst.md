---
name: burst
category: Phylogenetics & Population Genetics
description: BURST (Based Upon Rapid Sequence Typing) is a bioinformatics tool for analyzing multi-locus sequence typing (MLST) data in bacterial populations. It determines clonal complex relationships, identifies founding sequence types, and depicts evolutionary ancestry between sequence types.
tags:
  - mlst
  - phylogenetics
  - population-genetics
  - allelic-profiles
  - bacterial-typing
  - sequence-typing
  - clone-complexes
author: AI-generated
source_url: https://github.com/joe-trott/burst
---

## Concepts

- BURST analyzes allelic profiles from MLST data (typically 7 housekeeping genes) to determine evolutionary relationships between bacterial isolates, identifying founders of clonal complexes and inferring directionality of descent.

- Input files must be in FASTA format containing allele sequences, with each gene in a separate file or combined, and the tool requires a database of known allelic profiles (ST, MLST, or similar format) for comparison to determine "goeBURST" patterns.

- The tool outputs graphical representations (text-based or DOT format) showing nodes as sequence types and edges representing evolutionary links, with founder identification based on the principle that the most frequent ST at each locus is the presumed ancestor.

## Pitfalls

- Running BURST without a properly formatted database of known STs will result in no founder detection; the tool compares input alleles against a reference database to determine relationships, so a missing or empty database yields meaningless output.

- Using inconsistent allele numbering between input sequences and the reference database leads to incorrect phylogenetic assignments, as BURST matches alleles by exact numeric identity rather than by sequence content.

- Specifying the wrong input format (e.g., providing ST numbers instead of allele sequences) causes the run to fail or produce spurious results, because the tool expects nucleotide sequences not pre-computed allele designations.

## Examples

### Determine clonal complex relationships from MLST FASTA files

**Args:** -i all_genes.fas -o output.txt

**Explanation:** This runs BURST on a combined FASTA file containing all seven MLST gene alleles to identify evolutionary relationships and founders among the sequence types present.

### Generate a DOT-format graph for visualization

**Args:** -i all_genes.fas -dot output.dot

**Explanation:** Outputs a DOT file compatible with GraphViz for generating publication-quality images of the phylogenetic network showing clonal complex connections.

### List available databases in the BURST installation

**Args:** -list

**Explanation:** Displays the MLST databases (e.g., pubmlst.org profiles) included with the tool, allowing selection of an appropriate reference for comparison.

### Specify an exact number of SLVs (Single Locus Variants) to display

**Args:** -i all_genes.fas -slv 3 -o network.txt

**Explanation:** Limits the output to show only sequence types differing by at most 3 loci from the main cluster, useful for focused analysis of closely related isolates.

### Run with verbose logging for debugging

**Args:** -v -i all_genes.fas -o debug_output.txt

**Explanation:** Enables verbose output showing each step of the algorithm, helpful when troubleshooting unexpected results or input format issues.