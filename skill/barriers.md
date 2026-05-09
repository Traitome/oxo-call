---
name: barriers
category: Population Genetics / Phylogenetics
description: A bioinformatics tool for detecting recombination barriers and genetic discontinuities in nucleotide or amino acid sequence alignments, used to identify genomic regions where evolutionary constraints or structural features create discontinuities in phylogenetic signal.
tags: [recombination, phylogenetics, genetic-barriers, sequence-analysis, evolutionary-biology, population-genetics]
author: AI-generated
source_url: https://github.com/veg/barriers
---

## Concepts

- **Input Format**: Alignments in FASTA, NEXUS, or PHYLIP format containing nucleotide or amino acid sequences from multiple taxa or individuals. The tool requires pre-aligned sequences with no gap characters in the analysis region.
- **Data Model**: barriers treats each alignment column as a potential barrier site, computing likelihood ratios to detect sudden changes in phylogenetic reconstruction quality that suggest recombination or horizontal gene transfer.
- **Output Types**: Produces a CSV or text report listing barrier positions, associated p-values, and confidence intervals, optionally generating a graphical map of barrier locations across the alignment.
- **Statistical Method**: Uses a sliding window approach with comparative likelihood testing to identify significant discontinuities, where a barrier is called when window-specific phylogenies differ significantly from the global tree topology.
- **Companion Binary**: The `barriers-build` companion binary prepares alignment files and computes necessary distance matrices before the main barrier detection analysis.

## Pitfalls

- **Unaligned Sequences**: Running barriers on unaligned or poorly aligned sequences produces invalid barrier calls because phylogenetic incongruence from alignment errors is misinterpreted as biological recombination barriers.
- **Excessive Missing Data**: Alignments with more than 20% missing data (N or ? characters) cause the sliding window analysis to fail or produce unreliable p-values, as insufficient sites remain for robust tree reconstruction.
- **Insufficient Taxon Sampling**: With fewer than 4 sequences, barriers cannot compute meaningful topological comparisons and will either error out or report spurious single-point barriers.
- **Window Size Mismatch**: Setting the window size too small (less than 50bp for nucleotide data) increases false positives due to random phylogenetic noise, while overly large windows miss narrow recombination breakpoints.
- **Ignoring Rate Variation**: Failing to account for codon-based or site-specific substitution rate variation produces false barrier calls where fast-evolving sites create apparent phylogenetic discontinuities unrelated to recombination.

## Examples

### Detect recombination barriers in a viral gene alignment
**Args:** -i viral_coat_protein.fasta -o barriers_results.csv -w 100 --stat chi2
**Explanation:** Runs barrier detection with a 100-base sliding window on a viral coat protein alignment, outputting results in CSV format with chi-square statistical testing.

### Analyze amino acid sequences for protein-level barriers
**Args:** -i protein_alignment.phy --aa -w 50 -p 0.01 --verbose
**Explanation:** Analyzes amino acid sequence input with a narrower 50-codon window, applying a stricter p-value threshold of 0.01 for significance reporting.

### Generate a barrier map with confidence intervals
**Args:** -i sequences.fasta --bootstrap 100 --ci --outmap barrier_map.txt
**Explanation:** Computes 100 bootstrap replicates to generate confidence intervals for barrier locations and produces a text-based map showing barrier positions with confidence bounds.

### Run barriers with custom evolutionary model
**Args:** -i alignment.fasta --model GTR+G --site-rates gamma --ncat 4
**Explanation:** Applies the GTR+Gamma substitution model with 4 rate categories for more accurate phylogeny reconstruction during barrier detection.

### Use companion binary to prepare distance matrix first
**Args:** -i alignment.fasta -o distance_matrix.bin --freq --threads 4
**Explanation:** Uses barriers-build to compute a frequency-based distance matrix from the input alignment, specifying 4 parallel threads for faster processing.

### Combine multiple barrier analyses with merging
**Args:** -i gene1.fasta gene2.fasta gene3.fasta --merge-summary combined_barriers.tsv
**Explanation:** Processes three separate gene alignments and produces a merged summary file showing barrier positions across all analyzed regions.

### Limit analysis to specific alignment regions
**Args:** -i alignment.fasta --region 100-500 --window 75 -o region_results.txt
**Explanation:** Restricts barrier detection to alignment positions 100-500 using a 75-base window, useful for targeted analysis of specific genomic regions.