---
name: mafft
category: phylogenetics
description: Fast and accurate multiple sequence alignment tool for nucleotide and protein sequences
tags: [alignment, multiple-sequence-alignment, msa, phylogenetics, protein, nucleotide]
author: oxo-call built-in
source_url: "https://mafft.cbrc.jp/alignment/software/"
---

## Concepts

- MAFFT has multiple algorithms: FFT-NS-2 (fast, less accurate), L-INS-i (accurate, iterative), G-INS-i (global), E-INS-i (for highly gapped).
- Use --auto to automatically select the best algorithm based on data size; --thread N for parallelism.
- MAFFT outputs aligned FASTA to stdout — always redirect to a file.
- For large datasets (>200 sequences): use --auto or --retree 2; for high accuracy (<200 seqs): use --localpair.
- --localpair is equivalent to L-INS-i (most accurate for sequences with conserved core regions).
- --globalpair is G-INS-i for global alignments (whole sequence aligns); --genafpair for E-INS-i.
- Add --adjustdirectionaccurately to handle mixed-strand nucleotide sequences.
- Use --clustalout for Clustal-format output; --phylipout for PHYLIP format.

## Pitfalls

- MAFFT outputs to stdout — always redirect to a file: mafft --auto sequences.fasta > aligned.fasta.
- --localpair and --globalpair cannot be used together — choose one based on alignment type.
- For more than 200 sequences, --localpair becomes slow — use --auto or --retree 2 instead.
- MAFFT does not handle very long sequences (>50kb) well — use MUSCLE or specialized tools for genomic alignment.
- Mixed DNA/RNA input requires --adjustdirectionaccurately for sequences on different strands.
- MAFFT alignment quality depends on sequence identity — below 20% identity, alignments may be unreliable.

## Examples

### align multiple protein sequences with automatic algorithm selection
**Args:** `--auto --thread 8 proteins.fasta > aligned_proteins.fasta`
**Explanation:** --auto selects algorithm based on data size; output to stdout redirected to aligned_proteins.fasta

### highly accurate multiple sequence alignment for fewer than 200 sequences
**Args:** `--localpair --maxiterate 1000 --thread 8 sequences.fasta > aligned_localpair.fasta`
**Explanation:** --localpair (L-INS-i) most accurate for conserved core sequences; --maxiterate 1000 maximum iterations

### align RNA sequences adjusting for strand orientation
**Args:** `--auto --adjustdirectionaccurately --thread 8 rna_sequences.fasta > aligned_rna.fasta`
**Explanation:** --adjustdirectionaccurately handles sequences on different strands by reverse complementing as needed

### align sequences and output in PHYLIP format for phylogenetic analysis
**Args:** `--auto --thread 8 --phylipout sequences.fasta > aligned.phy`
**Explanation:** --phylipout generates PHYLIP format suitable for RAxML, IQ-TREE phylogenetic tools

### add new sequences to existing alignment
**Args:** `--add new_sequences.fasta --thread 8 existing_alignment.fasta > updated_alignment.fasta`
**Explanation:** --add incorporates new sequences into an existing alignment without re-aligning the original
