---
name: AMAS
category: Sequence Analysis and Alignment
description: AMAS (Algorithms for Molecular Sequences) is a toolkit for analyzing amino acid and nucleotide sequences, providing functions for format conversion, sequence statistics, composition analysis, and alignment-based phylogenetics. It processes multiple sequence formats and outputs analysis results in various formats.
tags:
  - sequence-analysis
  - multiple-sequence-alignment
  - format-conversion
  - phylogenetic
  - amino-acid
  - nucleotide
  - bioinformatics
author: AI-generated
source_url: https://github.com/marekjs/AMAS
---

## Concepts

- AMAS operates on multiple sequence formats including FASTA, Phylip (sequential and interleaved), Nexus, and Stockholm, automatically detecting input format from file extension or content. The `amas-run` command processes alignments and returns statistics, while `amas-compare` aligns sequences using MAFFT backend.
- Sequence statistics include length distribution, composition percentages, pairwise distances, and coverage metrics calculated per-sequence and per-alignment. For amino acid sequences, AMAS reports residue frequencies and conservation scores; for nucleotide sequences, it calculates GC/AT content and codon usage when applicable.
- The toolkit uses a master-worker parallelization model via Python's multiprocessing for batch operations on multiple alignment files, significantly reducing runtime on multi-core systems. Results are written to JSON, CSV, or tabular text formats specified by output flags.
- AMAS integrates with common phylogenetic workflows by producing substitution model statistics, likelihood ratio test values, and Akaike Information Criterion (AIC) scores that inform model selection in tools like RAxML or IQ-TREE.

## Pitfalls

- Specifying an output file path that does not exist without the `-dir` flag causes AMAS to fail silently, writing results to the current working directory instead of the intended location. This leads to missing output files and wasted compute time.
- Using `amas-compare` on alignments with excessive missing data (gaps >50%) produces unreliable distance estimates, inflating phylogenetic tree branch lengths and producing artifactual clustering. Always trim or mask problem regions before comparison.
- Mixing amino acid and nucleotide sequences in a single input file without explicit format specification causes AMAS to default to nucleotide mode, corrupting amino acid sequences during translation and producing nonsensical composition statistics.
- Specifying the wrong format flag (e.g., `-f phylip` for a Nexus file) results in parsing errors or silent misreading where sequence coordinates are shifted, fundamentally corrupting downstream analyses. Always verify input format matches file content structure.
- Overwriting existing output files with the `-w` flag without confirmation prompts can cause irreversible data loss when running batch operations, especially when output directories contain results from previous analyses.

## Examples

### Compute sequence statistics for a FASTA alignment file
**Args:** `run -i alignment.fasta -f fasta -out stats.txt -summary`
**Explanation:** This runs the AMAS statistics engine on a FASTA-formatted multiple sequence alignment, outputting per-sequence and alignment-wide composition and length statistics to the specified file.

### Batch process multiple Phylip alignments with JSON output
**Args:** `run -i *.phy -f phylip -out results/ -format json`
**Explanation:** This processes all Phylip-formatted alignment files matching the wildcard pattern, writing individual JSON result files for each alignment into the specified output directory for downstream programmatic analysis.

### Compare amino acid sequences and compute pairwise distances
**Args:** `compare -i protein_alignment.fasta -f fasta -d pdist -metric blosum62`
**Explanation:** This calculates pairwise distance matrices using the BLOSUM62 substitution matrix, appropriate for closely related amino acid sequences, enabling construction of phylogenetic trees with distance-based methods.

### Convert alignment format from FASTA to Phylip interleaved
**Args:** `run -i sequences.fasta -f fasta -convert phyli -o output.phy`
**Explanation:** This reads a FASTA alignment and writes it in Phylip interleaved format, useful for compatibility with phylogenetic software packages like PAUP* or PHYLA that require specific format variants.

### Generate model selection statistics for phylogenetic analysis
**Args:** `run -i orthologs.fasta -f fasta -model-test -out model_results.csv`
**Explanation:** This performs model testing statistics including log-likelihood, AIC, and BIC scores for common substitution models, producing output that directly informs model selection in maximum-likelihood tree building tools.