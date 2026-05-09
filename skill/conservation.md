---
name: conservation
category: genomics
description: A tool for calculating evolutionary conservation scores from multiple sequence alignments and phylogenetic trees. Outputs per-base or per-residue scores used to identify functionally constrained genomic regions.
tags: [conservation, evolutionary-biology, multiple-sequence-alignment, phylogenetic-scoring, functional-genomics]
author: AI-generated
source_url: https://github.com/ExampleTool/conservation
---

## Concepts

- The tool requires a multiple sequence alignment (MSA) as input, either in FASTA, Stockholm, or ClustalW format. The alignment must contain the reference sequence and sufficient related sequences to calculate meaningful conservation statistics.
- Output formats include per-position scores in WIG, BigWig, or BEDGRAPH format for genome browser visualization, and tabular formats for downstream statistical analysis.
- Conservation scores are calculated using the phylogeny-aware method by default, which accounts for non-independence of sequence evolution. The phylogenetic tree must be provided in Newick format when using phylogeny-based scoring.
- Scoring windows can be configured independently of alignment block sizes, allowing trade-offs between resolution and statistical power. Smaller windows increase resolution but reduce statistical significance.
- The tool supports both nucleotide and amino acid alignments with distinct substitution matrices (NUC. or BLOSUM62) automatically selected based on sequence type detection.

## Pitfalls

- Providing an MSA with fewer than 4 sequences severely underestimates conservation significance, as statistical testing requires adequate degrees of freedom for the null model. Results will show artificially inflated scores with poor p-value calibration.
- Using a nucleotide substitution matrix (NUC) for amino acid alignments, or vice versa, produces systematically biased scores that do not reflect true biochemical conservation. Always verify auto-detection or explicitly specify the correct matrix.
- Overlapping scoring windows that are smaller than the alignment gap structure creates artificial boundaries at indel regions, producing discontinuities that misrepresent functional constraint.
- Failing to provide a compatible phylogenetic tree when using phylogeny-aware scoring causes the tool to fall back to heuristic pairwise distance methods, which systematically underestimate conservation in clustered gene families.
- Output file extensions that do not match the specified format cause downstream tools (UCSC Genome Browser, IGV) to fail or misparse scores, requiring reformatting or manual correction.

## Examples

### Calculate conservation scores from an input MSA with default settings
**Args:** --input-alignment alignments/beta_globin.sto --output conservation_scores.bw --reference ENST00000381295
**Explanation:** This runs the standard conservation calculation using the beta-globin multiple alignment and outputs a BigWig file for direct genome browser upload.

### Generate per-position scores with a custom scoring window
**Args:** --input-alignment chr7_orthologs.fasta --output wigs/chromosome7_conservation --window-size 50 --step 10 --format WIG
**Explanation:** This produces a WIG track with 50bp windows stepped every 10bp, balancing resolution with statistical power for downstream peak calling.

### Calculate conservation using a precomputed phylogenetic tree
**Args:** --input-alignment primate_maf.block --tree phylogeny/primate_tree.newick --output conservation_phylo.bw --method phylogeny-aware
**Explanation:** This runs phylogeny-corrected conservation scoring that accounts for the evolutionary relationships between the aligned primate sequences.

### Analyze amino acid sequence conservation with BLOSUM62 matrix
**Args:** --input-alignment proteins kinase_domain.aln --output kinase_conservation.bed --submatrix BLOSUM62 --sequence-type AA
**Explanation:** This calculates amino acid conservation using BLOSUM62 substitution scores, appropriate for protein domain alignments where biochemical similarity matters.

### Generate both positive and negative strand conservation in BEDGRAPH format
**Args:** --input-alignment regulatory_elements.fasta --output conservation_features.bedgraph --stranded --format BEDGRAPH --threshold 0.7
**Explanation:** This outputs separate conservation tracks for forward and reverse strands at a 0.7 significance threshold, suitable for identifying strand-specific functional elements.

### Calculate conservation with reduced gap penalty for indel-rich regions
**Args:** --input-alignment VNTR_region.sto --output vntr_cons.bw --gap-open-penalty 2 --gap-extend-penalty 0.5 --min-seq-count 5
**Explanation:** This uses lower gap penalties to avoid penalizing repetitive indel variation, with a minimum requirement of 5 sequences to ensure statistical reliability.