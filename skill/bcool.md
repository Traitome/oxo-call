---
name: bcool
category: sequence_analysis
description: A tool for computing evolutionary conservation scores ("coolness") of nucleotide sequences across species using phylogenetic methods
tags: conservation, phylogenetics, comparative-genomics, sequence-analysis, multiple-sequence-alignment
author: AI-generated
source_url: https://github.com/erxen/bcool
---

## Concepts

- **Input Data Model**: bcool accepts multiple sequence alignment (MSA) files in FASTA, Clustal, Stockholm, or EMBL format. It also accepts BED files with genomic coordinates when paired with a multi-species genome database.
- **Coolness Scoring Algorithm**: The tool computes a numeric conservation score (0-1 scale) for each position by comparing observed substitutions against expected substitutions from a provided phylogenetic tree using a maximum likelihood framework.
- **Output Formats**: Results are written to stdout or a specified file in simple per-position format, BEDGRAPH for genome browser visualization, or JSON for programmatic parsing.
- **Phylogenetic Tree Requirement**: A Newick-formatted phylogenetic tree describing species relationships is strongly recommended; without it, bcool uses a default star phylogeny which underestimates conservation in phylogenetically distant comparisons.

## Pitfalls

- **Omitting the phylogenetic tree**: Without a `-t` tree file, bcool assumes equal evolutionary distances between all sequences, causing scores to be artificially deflated for deeply diverged species.
- **Using misaligned sequences**: Alignments with gaps, poor alignment quality, or sequences of different lengths produce spurious conservation scores since bcool treats gap characters as missing data rather than actual evolutionary events.
- **Specifying wrong sequence type**: Mistakenly specifying nucleotide mode (`-m n`) when analyzing amino acid alignments (or vice versa) yields meaningless scores because the substitution rate models differ.
- **Insufficient sequence diversity**: Comparing fewer than 3 species provides no phylogenetic signal; the algorithm still runs but the resulting scores lack statistical significance and are flagged with warning messages.
- **Memory limits with large alignments**: Alignments with thousands of sequences or megabase-length regions can exhaust default memory allocations, causing execution failures without the `-memory` flag adjustment.

## Examples

### Computing conservation from a FASTA MSA with a custom tree
**Args:** `-i alignment.fasta -t species_tree.nwk -o conservation_scores.txt`
**Explanation:** This runs bcool on a multiple sequence alignment file, applying the user-provided species phylogeny to calculate evolutionary conservation scores per alignment column.

### Using BED coordinates to extract conservation across genomes
**Args:** `-bed regions.bed -genome-db hg38 -ref-species human -o conservation.bedgraph`
**Explanation:** This mode takes genomic coordinates from a BED file and queries the internal conservation database to retrieve pre-computed conservation scores for each specified region.

### Analyzing with sliding window for smooth visualization
**Args:** `-i alignment.fasta -t species_tree.nwk -window 50 -step 10 -o windowed_scores.txt`
**Explanation:** This applies a sliding window approach (50 bp window, 10 bp step) rather than per-position scoring, producing smoothed conservation values ideal for plotting across large genomic intervals.

### Specifying amino acid mode for protein alignment
**Args:** `-i proteinMSA.fasta -m aa -t species_tree.nwk -o protein_coolness.txt`
**Explanation:** This explicitly sets the sequence type to amino acid, ensuring bcool uses the correct substitution model (e.g., LG or WAG) instead of the default nucleotide model.

### Limiting output to specific genomic range
**Args:** `-i alignment.fasta -t species_tree.nwk -chrom chr1:1000000-2000000 -o range_scores.txt`
**Explanation:** This restricts analysis and output to positions falling within the specified genomic range, useful for targeting conservation analysis to particular loci of interest.

### Adjusting memory for large alignments
**Args:** `-i huge_alignment.fasta -t species_tree.nwk -memory 8192 -o large_results.txt`
**Explanation:** This allocates 8 GB of memory to handle an alignment file with many sequences or very long sequence lengths, preventing out-of-memory failures during computation.