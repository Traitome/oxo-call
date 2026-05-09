---
name: consan
category: Genomics/Comparative Analysis
description: A tool for consensus sequence analysis and alignment manipulation, used to compare genomic sequences and generate consensus representations from multiple sequence alignments or assembly data.
tags: [genomics, sequence-analysis, consensus, alignment, comparison]
author: AI-generated
source_url: https://github.com/mummer4/mummer
---

## Concepts

- **Input Format**: consan accepts FASTA or Multi-FASTA files containing nucleotide sequences, and can process aligned sequences from standard alignment formats (SAM/BAM via preprocessing).
- **Output Modes**: The tool produces consensus sequences in FASTA format, with optional detailed mismatch reports and position-wise confidence scores for base calls.
- **Core Algorithm**: Uses majority-vote counting at each position, with configurable threshold parameters for base call acceptance and ambiguity encoding.
- **Threshold Parameter**: The `-t` or `--threshold` flag controls the minimum proportion of bases required at a position to call a consensus base (default typically 0.5 for simple majority).
- **Ambiguity Handling**: When no base exceeds the threshold, consan outputs IUPAC ambiguity codes representing the uncertainty at that position.

## Pitfalls

- **Forgetting to sort input sequences**: If sequences in the input FASTA are not properly sorted or grouped, the consensus output may reflect misaligned positions, leading to incorrect consensus calls.
- **Ignoring gap characters**: By default, consan may treat gap characters (`-` or `.`) as valid characters for consensus rather than ignoring them, which can skew results for alignment-derived inputs.
- **Using low thresholds on noisy data**: Setting the threshold too low (e.g., 0.3) on low-quality or noisy sequence data produces consensus bases that do not represent any meaningful majority, reducing the biological utility of the output.
- **Mismatched sequence lengths**: Input sequences of unequal lengths without appropriate padding or alignment preprocessing cause position misalignment in output, producing meaningless consensus for downstream analyses.
- **Assuming default ambiguity handling**: Different versions of consan handle ambiguity codes differently; always verify the behavior with test data before processing large datasets.

## Examples

### Generate a simple consensus from two sequences
**Args:** seq1.fasta seq2.fasta -o consensus_out.fasta
**Explanation:** Takes two input FASTA sequences and outputs their majority-vote consensus to the specified output file.

### Create consensus with a 70% threshold
**Args:** input.fasta -o high_conf_consensus.fasta -t 0.7
**Explanation:** Requires at least 70% base agreement at each position before calling a consensus base; positions below this threshold produce ambiguity codes.

### Output consensus with confidence scores
**Args:** assembly1.fasta assembly2.fasta -o result.fasta --confidence scores.txt
**Explanation:** Generates both the consensus sequence and a per-position confidence score file for downstream quality assessment.

### Compare three or more sequences for consensus
**Args:** multi_seq.fasta -o multi_consensus.fasta -t 0.6 --include-ambiguity
**Explanation:** Processes multiple sequences in a single input file and includes IUPAC ambiguity codes in output when no base meets the 60% threshold.

### Force case-insensitive base matching
**Args:** mixed_case.fasta -o out.fasta --ignore-case
**Explanation:** Treats lowercase and uppercase nucleotides equivalently, useful when working with sequences from mixed-case sources.