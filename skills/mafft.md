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
- --add adds new sequences to existing alignment; --addfragments aligns unaligned fragments to existing alignment.
- --merge combines two or more existing alignments without re-aligning.
- --seed uses a pre-aligned region as anchor for alignment.
- --reorder sorts output by sequence similarity; --inputorder preserves input order.
- --op and --ep control gap opening and extension penalties for fine-tuning alignment.
- --maxiterate sets the number of iterative refinement cycles (higher = more accurate but slower).

## Pitfalls
- MAFFT outputs to stdout — always redirect to a file: mafft --auto sequences.fasta > aligned.fasta.
- --localpair and --globalpair cannot be used together — choose one based on alignment type.
- For more than 200 sequences, --localpair becomes slow — use --auto or --retree 2 instead.
- MAFFT does not handle very long sequences (>50kb) well — use MUSCLE or specialized tools for genomic alignment.
- Mixed DNA/RNA input requires --adjustdirectionaccurately for sequences on different strands.
- MAFFT alignment quality depends on sequence identity — below 20% identity, alignments may be unreliable.
- --add requires existing alignment as reference; new sequences are aligned to it but original alignment may shift.
- --addfragments is for unaligned sequences only; using it with pre-aligned sequences gives incorrect results.
- --thread -1 uses all available CPU cores; explicitly set for reproducibility across different machines.
- --adjustdirectionaccurately is slow but necessary for mixed-strand nucleotide data; don't use for protein.
- --maxiterate 1000 is maximum; beyond 1000 iterations, improvements are minimal.
- --6merpair is faster than default for very large datasets but less accurate.

## Examples

### align multiple protein sequences with automatic algorithm selection
**Args:** `--auto --thread 8 proteins.fasta > aligned_proteins.fasta`
**Explanation:** mafft command; --auto algorithm selection; --thread 8 threads; proteins.fasta input FASTA; output to aligned_proteins.fasta

### highly accurate multiple sequence alignment for fewer than 200 sequences
**Args:** `--localpair --maxiterate 1000 --thread 8 sequences.fasta > aligned_localpair.fasta`
**Explanation:** mafft command; --localpair L-INS-i algorithm; --maxiterate 1000 iterations; --thread 8 threads; sequences.fasta input FASTA; output to aligned_localpair.fasta

### align RNA sequences adjusting for strand orientation
**Args:** `--auto --adjustdirectionaccurately --thread 8 rna_sequences.fasta > aligned_rna.fasta`
**Explanation:** mafft command; --auto algorithm selection; --adjustdirectionaccurately strand orientation; --thread 8 threads; rna_sequences.fasta input FASTA; output to aligned_rna.fasta

### align sequences and output in PHYLIP format for phylogenetic analysis
**Args:** `--auto --thread 8 --phylipout sequences.fasta > aligned.phy`
**Explanation:** mafft command; --auto algorithm selection; --thread 8 threads; --phylipout PHYLIP format output; sequences.fasta input FASTA; output to aligned.phy

### add new sequences to existing alignment
**Args:** `--add new_sequences.fasta --thread 8 existing_alignment.fasta > updated_alignment.fasta`
**Explanation:** mafft command; --add new_sequences.fasta add sequences; --thread 8 threads; existing_alignment.fasta existing alignment; output to updated_alignment.fasta

### align unaligned fragment sequences to existing alignment
**Args:** `--addfragments fragments.fasta --reorder --thread 8 existing_alignment.fasta > updated.fasta`
**Explanation:** mafft command; --addfragments fragments.fasta add unaligned fragments; --reorder sort by similarity; --thread 8 threads; existing_alignment.fasta existing alignment; output to updated.fasta

### merge two existing alignments without re-aligning
**Args:** `--merge alignment1.fasta alignment2.fasta > merged_alignment.fasta`
**Explanation:** mafft command; --merge combine alignments; alignment1.fasta alignment2.fasta input alignments; output to merged_alignment.fasta

### use seed alignment to anchor the alignment process
**Args:** `--seed seed_alignment.fasta --auto --thread 8 sequences.fasta > anchored_alignment.fasta`
**Explanation:** mafft command; --seed seed_alignment.fasta seed alignment; --auto algorithm selection; --thread 8 threads; sequences.fasta input FASTA; output to anchored_alignment.fasta

### align with custom gap penalties for fine-tuning
**Args:** `--auto --op 2.0 --ep 0.5 --thread 8 sequences.fasta > aligned_custom.fasta`
**Explanation:** mafft command; --auto algorithm selection; --op 2.0 gap opening penalty; --ep 0.5 extension penalty; --thread 8 threads; sequences.fasta input FASTA; output to aligned_custom.fasta

### fast alignment for very large datasets
**Args:** `--retree 2 --maxiterate 0 --thread -1 large_dataset.fasta > aligned_fast.fasta`
**Explanation:** mafft command; --retree 2 guide tree iterations; --maxiterate 0 skip refinement; --thread -1 all cores; large_dataset.fasta input FASTA; output to aligned_fast.fasta

### align highly gapped sequences with E-INS-i algorithm
**Args:** `--genafpair --maxiterate 1000 --thread 8 gapped_sequences.fasta > aligned_gapped.fasta`
**Explanation:** mafft command; --genafpair E-INS-i algorithm; --maxiterate 1000 iterations; --thread 8 threads; gapped_sequences.fasta input FASTA; output to aligned_gapped.fasta

### output in Clustal format for compatibility with legacy tools
**Args:** `--auto --clustalout --thread 8 sequences.fasta > aligned.clustal`
**Explanation:** mafft command; --auto algorithm selection; --clustalout Clustal format; --thread 8 threads; sequences.fasta input FASTA; output to aligned.clustal
