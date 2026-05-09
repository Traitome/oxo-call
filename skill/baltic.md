---
name: baltic
category: phylogenetics
description: A Python library and command-line tool for processing, analyzing, and visualizing phylogenetic trees, particularly for phylodynamic analysis of pathogen evolution. Baltic reads Newick trees, BEAST output files, and tip metadata to estimate mutation rates, coalescent times, and evolutionary dates.
tags:
- phylogenetics
- phylogeny
- evolution
- timetree
- beast
- phylodynamics
- newick
- coalescent
- mutation-rate
author: AI-generated
source_url: https://github.com/evogytis/baltic
---

## Concepts

- Baltic reads phylogenetic trees in Newick format and can process BEAST XML/LOG output files to extract evolutionary parameters like mutation rates and divergence times
- Tip dates can be specified via command-line dates, external date files, or parsed from sequence names (ISO format like `Sample_2020-01-15`) for timetree reconstruction
- The tool supports both nucleotide and amino acid substitution models, requiring correct specification of the state count (4 for nucleotide, 20 for amino acid) for accurate branch length dating

## Pitfalls

- Using incorrectly formatted or inconsistent tip date formats will cause dating calculations to fail or produce nonsensical evolutionary dates, leading to negative branch lengths
- Forgetting to specify the correct nucleotide state count (default 4) when analyzing amino acid alignments results in underestimated mutation rates by approximately 5-fold
- Running timetree estimation without tip date information produces uncalibrated trees with arbitrary branch length units, making evolutionary conclusions impossible
- Not providing the output prefix when processing multiple sequences causes files to overwrite each other, resulting in lost data

## Examples

### Build a timetree from a Newick file with tip dates provided as a text file
**Args:** -t sequences.tree -d tip_dates.txt -o dated_tree.nexus
**Explanation:** This instructs Baltic to calibrate branch lengths using tip date metadata from an external file, producing a timetree with absolute dates

### Calculate mutation rate from a dated phylogeny
**Args:** -t dated_tree.nexus --mutation-rate --output mut_rate.txt
**Explanation:** This calculates the evolutionary mutation rate from a pre-dated phylogenetic tree, useful for understanding pathogen evolution speed

### Process BEAST log file to extract evolutionary statistics
**Args:** -i beast_mcmc.log --stats coalescent,height --json stats.json
**Explanation:** This extracts coalescent times and tree height statistics from BEAST MCMC output and saves them in JSON format for downstream analysis

### Visualize a phylogenetic tree with tip dates
**Args:** -t input.tree -d "2020-01-01" --plot --scale time --pdf tree_vis.pdf
**Explanation:** This creates a PDF visualization of the phylogenetic tree with time-scaled branches and tip labels annotated with sampling dates

### Convert a Newick tree to nexus format with dates
**Args:** -t unconstrained.tree --nexus --date-format iso --output converted.nex
**Explanation:** This converts a standard Newick tree to Nexus format while preserving date information in ISO format for BEAST compatibility