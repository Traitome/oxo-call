---
name: bellerophon
category: sequence_analysis
description: A tool for identifying local alignments and detecting chimeric or fused sequences by finding split alignments between a query sequence and a target database.
tags:
  - alignment
  - chimera
  - fusion
  - genomics
  - UCSC
  - rearrangement
author: AI-generated
source_url: https://hgdownload.soe.ucsc.edu/admin/exe/
---

## Concepts

- **Input Format**: Bellerophon takes two FASTA input files — a query sequence (the sequence you want to align) and a target database (the reference to align against). The query is typically a potential fusion transcript or suspect chimeric sequence.
- **Output Format**: Results are produced in PSL (Position-Specific Liftover) format, similar to BLAT output. Each alignment hit includes the target region start/end, alignment block sizes, and gap information, allowing identification of split alignments spanning multiple genomic loci.
- **Chimera Detection**: The tool is specifically designed to detect local alignments where a single query aligns to multiple non-contiguous regions of the target — a signature pattern for gene fusions, rearrangements, or chimeric transcripts. The alignment score and block structure indicate whether an observed alignment represents a genuine fusion.
- **Dynamic Programming**: Bellerophon uses Smith-Waterman style dynamic programming to find optimal local alignments, which are reported along with their chromosomal coordinates in the target, enabling downstream analysis of fusion breakpoints.

## Pitfalls

- **Swapped Input Order**: Specifying the target file as query and query as target yields meaningless alignments. The first positional argument must be the query (suspect fusion), and the second must be the target database (e.g., genome).
- **Ignoring Default Parameters**: Running without parameter optimization frequently misses subtle alignments. The word size (-t) and step size parameters critically affect sensitivity; default settings may fail to detect low-identity fusions or short alignment spans.
- **Misinterpreting Single-Hit Results**: Expecting every query to produce multiple alignment blocks leads to missed rearrangements. Many true gene fusions produce only a single alignment block if onegene contains the entire fusion transcript; additional validation with other tools is required.
- **Large Target Databases**: Aligning against an entire genome without indexing often becomes computationally prohibitive. Pre-processing the target into tiled tiles using genomic tiling strategies significantly reduces runtime without sacrificing detection accuracy.

## Examples

### Align a suspected BCR-ABL fusion transcript against the human genome

**Args:** -t=11 -stepSize=5 input/fusion_query.fa input/hg38.fa

**Explanation:** With reduced word size (11) and step size (5), the tool increases alignment sensitivity to detect the BCR-ABL fusion breakpoints, which may contain short conserved regions at the join point.

### Detect all split alignments indicating potential gene rearrangements

**Args:** -minScore=50 -tileSize=6 input/suspect_transcript.fa output/hg38_alignments.psl

**Explanation:** Setting minScore to 50 filters low-quality alignments while capturing split alignments where the query spans multiple genomic loci, a hallmark of genomic rearrangements.

### Find alignment using faster tile size for whole-genome scanning

**Args:** -tileSize=10 input/chimera_query.fa output/genome_alignments.psl

**Explanation:** Using a larger tile size (10) sacrifices some sensitivity but dramatically speeds up alignment against the entire genome, suitable for initial screening before targeted re-analysis.

### Identify alignments with minimum block size to remove micro-aligments

**Args:** -minMatch=25 -tileSize input/query.fa input/hg19.fa

**Explanation:** The minMatch parameter requires at least 25 matching bases per alignment block to filter out spurious micro-alignments caused by repetitive elements or sequencing artifacts.

### Optimize for detecting short conserved fusion breakpoints

**Args:** -t=8 -stepSize=2 -minScore=30 input/fusion_short.fa input/target.fa

**Explanation:** With an extremely reduced word size (8) and step size (2), the tool becomes highly sensitive to short conserved sequences at fusion breakpoints that would otherwise be missed by standard alignment parameters.

### Output raw alignment blocks for downstream fusion analysis

**Args:** -noHead input/query.fa input/target.fa > output/raw_alignments.psl

**Explanation:** Adding the -noHead flag suppresses PSL format header lines, generating machine-readable output suitable for automated parsing and downstream computational fusion detection pipelines.