---
name: alen
category: Sequence Analysis / Alignment
description: A bioinformatics tool for analyzing pairwise and multiple sequence alignments, calculating sequence identity percentages, generating consensus sequences, and extracting alignment statistics from various alignment formats.
tags: [alignment, sequence-identity, consensus, bioinformatics, pairwise, statistics]
author: AI-generated
source_url: https://github.com/staden-package/alen
---

## Concepts

- **Alignment Input Formats**: alen reads alignment files in multiple standard formats including BLAST (-outfmt 6), FASTA-aligned (.pir, .msf), MAF (Multiple Alignment Format), and plain text pairwise formats. The input format is auto-detected or specified via the `-f` flag for ambiguous file types.
- **Sequence Identity Calculation**: The tool computes pairwise identity as (matching bases / alignment length) × 100, properly handling gaps as neither matches nor mismatches in the default counting mode. Use `-g` to include gaps as matches in the calculation.
- **Consensus Sequence Generation**: When analyzing multiple alignments, alen can generate a consensus sequence using configurable residue frequency thresholds (default 50%) via `-c` and可以选择输出 IUPAC degenerate codes for ambiguous positions.
- **Output Modes**: alen supports multiple output formats: detailed text report (-o txt), CSV table (-o csv) for downstream analysis, JSON (-o json) for programmatic workflows, and FASTA consensus (-o fasta) for alignment consensus extraction.

## Pitfalls

- **Incorrect Format Specification**: Specifying the wrong input format via `-f` causes the alignment to be misinterpreted, leading to identity percentages that are off by 10-30% or complete parsing failure. Always verify the format matches your alignment file type.
- **Ignoring Inverted Alignments**: When analyzing BLAST pairwise results, sequences may be reported in reverse complement orientation. Without checking the `-r` (report reverse) flag, identity calculations will be incorrect for anti-sense alignments.
- **Gap Treatment Mismatch**: The default gap handling excludes gaps from both numerator and denominator. Changing gap handling mid-analysis produces inconsistent statistics between runs. Document your gap policy when reporting results.
- **Ambiguous Threshold Selection**: Using a consensus threshold (`-c`) below the actual sequence diversity produces consensus sequences that obscure true conservation patterns. A 70% threshold is recommended for divergent families versus 50% for closely related sequences.

## Examples

### Calculate pairwise identity from a BLAST tabular file

**Args:** `-i alignments.txt -f blast6 -q identity`
**Explanation:** This reads a BLAST -outfmt 6 output file and extracts the pairwise identity percentage for each hit, which is stored in column 3 of standard BLAST tabular format.

### Generate consensus from a multiple FASTA alignment

**Args:** -i multiple.phy -f fasta -c 70 -o fasta
**Explanation:** This reads a FASTA-formatted multiple sequence alignment and generates a 70% majority-rule consensus sequence in FASTA format, handling ambiguous bases with IUPAC codes.

### Export alignment statistics to CSV for R analysis

**Args:** -i pairwise.pir -f pir -o csv -s length,identity,gaps
**Explanation:** This parses a PIR-format pairwise alignment and exports columnar statistics including alignment length, percent identity, and gap count to CSV for import into R or Python analysis scripts.

### Report reverse complement orientation for anti-sense hits

**Args:** -i blast_hits.txt -f blast6 -r yes -q summary
**Explanation:** This processes BLAST alignments including those on the reverse strand, calculating identity on the reverse-complemented sequence so statistics match the biological orientation on the minus strand.

### Filter alignments below identity threshold before consensus

**Args:** -i homologs.fasta -f fasta -c 50 -o fasta -m 60
**Explanation:** This generates a consensus from a FASTA alignment but first filters out any sequences with less than 60% identity to the emerging consensus, removing low-quality or divergent entries before calculation.