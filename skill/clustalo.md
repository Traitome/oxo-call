---
name: clustalo
category: sequence_alignment
description: Clustal Omega is a fast and scalable multiple sequence alignment tool for DNA, RNA, and protein sequences. It uses a guide tree approach with HMM profile-profile alignments to produce high-quality alignments for large sequence sets.
tags: [multiple_sequence_alignment, msa, bioinformatics, sequence_analysis, dna, rna, protein]
author: AI-generated
source_url: https://www.clustal.org/omega/
---

## Concepts

- **Input formats**: Clustal Omega accepts multiple input formats including FASTA, Clustal, EMBL, GCG, GDE, IG/Stanford, PIR, Phylip, and GTT. Sequences can be provided via stdin, making it compatible with Unix pipes for chaining with other tools.
- **Output formats**: Output alignments can be generated in several formats (Aln, Clustal, FASTA, GCG, PHYLIP, SELEX, VTF) with optional features including sequence numbers, alignment as consensus, and colorized output by conservation.
- **Algorithm**: Clustal Omega builds a guide tree from pairwise distances computed using k-mer counting, then performs progressive alignment. For difficult alignments, it uses HMM profile-profile iterations to improve accuracy, controlled by the `--max-hmm-iterations` flag.
- **Performance**: The tool supports multi-threading via `--threads` to parallelize the computationally intensive guide tree calculation, enabling efficient handling of large datasets (thousands of sequences).

## Pitfalls

- **Mismatched sequence type**: Using `--seqtype` incorrectly (e.g., specifying DNA for protein sequences) results in inappropriate scoring matrices and poor alignment quality. Always verify the correct sequence type before alignment.
- **Large alignments without threads**: Aligning thousands of sequences without `--threads` wastes available CPU resources and can take hours. For large datasets, enable threading to significantly reduce runtime.
- **Missing output format specification**: Default output may be in Clustal format, which some downstream tools cannot parse. Always specify `--outfmt` to ensure compatibility with subsequent analysis pipelines.
- **Ignoring iteration need**: Highly divergent sequence sets may require HMM iterations (`--max-hmm-iterations`) to converge to a proper alignment. Skipping iterations can lead to misaligned regions and false homologies.
- **Input file format errors**: Sequences with inconsistent line lengths, special characters in headers, or mixed case can cause parsing failures. Clean and validate input files before running alignment.

## Examples

### Align protein sequences from a FASTA file
**Args:** `-i inputproteins.fasta -o aligned.fasta --outfmt fasta`
**Explanation:** Aligns protein sequences using FASTA input and output, ensuring downstream tools can parse the result directly.

### Create a Clustal-format alignment with sequence numbers
**Args:** `-i sequences.fasta -o alignment.aln --outfmt clustal --number`
**Explanation:** Outputs alignment in Clustal format with sequence position numbers for easy reference and manual inspection.

### Run with 8 threads for faster processing
**Args:** `-i large_set.fasta -o output.aln --threads 8`
**Explanation:** Enables multi-threaded execution to parallelize guide tree computation, drastically reducing runtime for large sequence sets.

### Perform profile-profile alignment with iterations
**Args:** --hmm1 profile1.hmm --hmm2 profile2.hmm -o profile_align.aln --max-hmm-iterations 2
**Explanation:** Aligns two HMM profiles with iterative refinement to improve accuracy for distantly related protein families.

### Generate PHYLIP format for phylogenetic analysis
**Args:** `-i sequences.fasta -o phylip_output.phy --outfmt phylip`
**Explanation:** Outputs alignment in PHYLIP format, which is required by many phylogenetic tree reconstruction programs like RAxML or PhyML.

### Run alignment with DNA scoring and auto-detection
**Args:** `-i dna_sequences.fasta --seqtype DNA -o dna_aligned.fasta`
**Explanation:** Explicitly sets DNA sequence type to apply appropriate scoring matrix, ensuring accurate alignment of nucleotide sequences.

### Output colorized alignment highlighting conservation
**Args:** `-i sequences.fasta -o colored.aln --color --consistency 5`
**Explanation:** Generates color output showing conservation levels across the alignment, useful for visualizing conserved domains.