---
name: cialign
category: sequence_alignment
description: A tool for multiple codon-aware multiple sequence alignment of DNA sequences, designed to preserve reading frames during alignment of coding regions.
tags: [dna-alignment, codon-alignment, multiple-sequencing-alignment, bioinformatics, frame-preservation]
author: AI-generated
source_url: https://github.com/BinaryResearch/CIALIGN
---

## Concepts

- **Input Format**: CIALIGN acceptsDNA sequences in FASTA format; input sequences must be nucleotide sequences (not protein), and all sequences should represent coding regions aligned to the same frame.
- **Frame-Preserving Alignment**: CIALIGN uses codon-based alignment algorithms that maintain the reading frame, inserting gaps in multiples of 3 nucleotides to avoid frameshift mutations in the aligned output.
- **Output Formats**: The tool supports multiple output alignment formats including FASTA, Clustal, and PHYLIP, selectable via command-line flags for downstream phylogenetic analysis.
- **Scoring Matrices**: Custom DNA scoring matrices can be specified to adjust match/mismatch penalties; default scoring is suitable for coding sequence alignment but can be tuned for specific evolutionary models.

## Pitfalls

- **Non-multiples of 3 gaps**: Inserting gaps that are not multiples of 3 will create frameshifts; CIALIGN handles this gracefully by adjusting gaps in multiples of 3, but this may alter your intended alignment structure.
- **Mixed frame sequences**: Providing sequences with different reading frames or untranslated regions will produce incorrect codon alignments; ensure all input sequences are already in the correct reading frame.
- **Insufficient sequence diversity**: Aligning very similar (>99% identity) sequences may not benefit from codon-aware alignment; the tool still processes them but may show minimal advantage over nucleotide-only alignment.
- **Large datasets**: Running CIALIGN on very large numbers of sequences (100+) can be computationally intensive; consider subsampling or using faster alignment methods for exploratory analysis.

## Examples

### Align a set of coding sequences in FASTA format
**Args:** `input.fasta -o aligned.fasta -f fasta`
**Explanation:** Takes coding sequences from input.fasta and outputs a FASTA-formatted alignment file, preserving the reading frame throughout.

### Generate a Clustal-formatted alignment for phylogenetic tree building
**Args:** `sequences.fasta -o output.aln -f clustal`
**Explanation:** Produces a Clustal-style alignment file suitable for direct input to phylogenetic tools like PHYLIP or tree-building software.

### Adjust match/mismatch scoring for closely related sequences
**Args:** ` closely_rel.fasta -o out.fasta --match 2 --mismatch -1`
**Explanation:** Uses custom scoring parameters that give a +2 reward for matches and -1 penalty for mismatches, appropriate for sequences with high similarity.

### Output in PHYLIP format for maximum compatibility
**Args:** `input.fasta -o alignment.phy -f phylip`
**Explanation:** Writes the alignment in PHYLIP sequential format, which is widely accepted by evolutionary analysis programs like PAUP*, MEGA, or BEAST.

### Suppress progress output for pipeline integration
**Args:** `seqs.fasta -o result.fasta -f fasta -q`
**Explanation:** Runs the alignment in quiet mode without printing progress messages, suitable for scripted pipelines where only the final output is needed.

### Handle sequences with N characters as ambiguities
**Args:** `input_ambiguous.fasta -o aligned.fasta -f fasta --ambig N`
**Explanation:** Treats N characters as ambiguous bases rather than as gaps or mismatches during the alignment process, preserving uncertainty in the output.

### Specify a custom output filename for batch processing
**Args:** `batch_input/*.fasta -o results/aligned.fasta -f fasta`
**Explanation:** Processes all sequences from the batch input directory and writes the combined alignment to the specified results directory.