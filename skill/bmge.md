---
name: bmge
category: Sequence Filtering and Quality Control
description: BMGE (Block Mapping and Generation Engine) is a bioinformatics tool for selecting regions of an alignment that are suitable for phylogenetic inference. It removes positions with too many gaps, missing data, or mutational saturation, and can infer a corrected alignment based on entropy and gap thresholds. Operates on FASTA, Phylip, and Nexus alignment formats.
tags: [alignment-filtering, phylogenetic-quality-control, entropy-based-filtering, gap-removal, substitution-saturation, bioinformatics, sequence-analysis]
author: AI-generated
source_url: https://gitlab.be/jieun/BMGE
---

## Concepts

- BMGE evaluates each column (position) of a multiple sequence alignment and computes a score based on entropy, gap frequency, and the degree of character state conservation. Positions exceeding a gap threshold (controlled by `-h`) or an entropy threshold (controlled by `-t`) are flagged and optionally removed from the output alignment, producing a trimmed file suitable for phylogenetic tree building.
- Input alignments can be provided in three accepted formats: FASTA (via `-i`), Phylip sequential or interleaved (via `-if` / `-il`), and Nexus (via `-s NEXUS`). Output format is selectable with `-o`, with options including FASTA, Phylip sequential (`-of`), Phylip interleaved (`-ol`), and Nexus (`-s NEXUS`).
- The tool supports a saturation-aware filtering mode via `-sm` (saturation mask), which additionally estimates substitution saturation for each codon position and removes those showing strong saturation signals. This is especially useful for third-codon-position (wobble) sites in protein-coding alignments where multiple hits are likely.
- BMGE can perform alignment trimming using the `-w` (window) option, which applies a sliding-window approach over the alignment: any column whose score falls below the entropy threshold is discarded, but neighboring columns in the same window may be retained if the average score is acceptable. This balances column-wise stringency with regional coherence.
- The `-c` (cutoff) option sets the fraction of gap characters allowed per column (default `0.5`). Columns with a gap fraction strictly greater than this cutoff are removed. Tuning `-c` is important when alignments have moderate numbers of missing sequences, as overly aggressive filtering with a low cutoff can strip away nearly all informative sites.

## Pitfalls

- Using a very low entropy cutoff (e.g., `-t 0.1`) can remove the majority of columns, leaving an alignment too sparse to resolve phylogenetic relationships reliably. The resulting tree will have poor bootstrap support and may be topologically misleading.
- BMGE does not automatically handle sequence label duplication across partitions in a Nexus file. If you have a multi-partition Nexus file, ensure each sequence label is unique per partition before running BMGE, or use a format converter (e.g., `seqmagick`) to merge partitions into a single alignment first.
- The `-sm` saturation mask option requires aligned codons where each codon triplet corresponds to three consecutive columns. If your alignment is not codon-aligned (e.g., it is a protein alignment or a non-coding DNA alignment), enabling `-sm` will produce meaningless saturation scores and corrupt the filtering logic, leading to either an over-filtered or under-filtered output.
- Running BMGE without specifying an output file (no `-o` flag) writes results to standard output. If you redirect output with shell redirection (e.g., `bmge -i input.aln > output.aln`) and the alignment uses Nexus format with line breaks in sequence data, the redirection may omit trailing newlines, making the output malformed for downstream tools like RAxML or PAUP.
- The `-w` window size option must be set to an integer greater than zero. Providing `-w 0` or a negative window size causes BMGE to skip all filtering entirely and output the unaltered input alignment without warning, wasting compute time and producing no trimming.

## Examples

### Filter a FASTA alignment with default entropy and gap thresholds
**Args:** `-i aligned_seqs.fasta -o filtered_seqs.fasta`
**Explanation:** This reads a FASTA alignment, removes columns where gap fraction exceeds 50% or entropy exceeds 1.0 (defaults), and writes the cleaned alignment to the specified output file.

### Trim a Phylip interleaved alignment with a stricter gap cutoff
**Args:** `-if input.phy -o trimmed.phy -c 0.3 -t 0.5`
**Explanation:** Specifying `-c 0.3` enforces a more aggressive gap removal (columns with >30% gaps are deleted) and `-t 0.5` uses a stricter entropy threshold, producing a higher-quality but smaller alignment.

### Apply saturation-aware filtering on a codon-aligned Nexus file
**Args:** `-s NEXUS -i codons.nex -o saturated_masked.nex -sm -t 0.6`
**Explanation:** The saturation mask mode (`-sm`) evaluates substitution saturation per codon position in a Nexus alignment and removes positions, particularly third-codon sites, with high saturation, making the output suitable for phylogenetic analysis of protein-coding sequences.

### Trim an alignment using a sliding window for regional coherence
**Args:** `-i input.phy -if -o output.phy -w 5 -t 0.4 -c 0.4`
**Explanation:** The sliding window (`-w 5`) ensures that a column is removed only if its score is poor and the average of the surrounding 5-column window also falls below the threshold, preserving locally conserved blocks that might otherwise be lost under strict column-wise filtering.

### Convert a FASTA alignment to Nexus format without filtering
**Args:** `-i aligned.fasta -s NEXUS -o converted.nex`
**Explanation:** Specifying only the input format and `-s NEXUS` without custom thresholds triggers minimal processing, effectively converting the alignment from FASTA to Nexus format for compatibility with tools like MrBayes or BEAST that require Nexus input.

### Remove highly gapped columns from a protein alignment
**Args:** `-i protein_align.fasta -o clean_protein.fasta -c 0.2`
**Explanation:** Using a low gap cutoff of 0.2 removes columns where more than 20% of sequences have a gap character, which is appropriate for protein alignments where missing data can bias substitution models during tree inference.