---
name: bttcmp
category: alignment-evaluation
description: A tool for comparing multiple SAM/BAM alignment files to evaluate and benchmark aligner performance. Computes alignment metrics such as concordance, discordance rates, coverage, and alignment quality scores across input files.
tags: [sam, bam, comparison, benchmarking, alignment-evaluation, metrics]
author: AI-generated
source_url: https://github.com/Ben Langmead/bowtie2/tree/master/utils/bttcmp
---

## Concepts

- **Input Format**: bttcmp accepts multiple SAM, BAM, or CRAM files as positional arguments, optionally specifying which files are "truth" or "baseline" references for comparison against other aligner outputs.
- **Core Metrics**: The tool reports concordance (reads aligned identically across all files), discordance (reads with conflicting alignments), and per-position agreement rates—essential for benchmarking aligner accuracy and reproducibility.
- **Output Modes**: Results can be emitted as human-readable text (default), JSON, or CSV for downstream scripting; JSON mode includes nested precision/recall-style statistics per read category.
- **Reference Handling**: bttcmp requires a FASTA reference file via the `--ref` flag when assessing alignment coordinates, as SAM headers alone may lack sufficient sequence dictionary information.

## Pitfalls

- **Mismatched Read Ordering**: Comparing SAM files that were not sorted identically (e.g., one by coordinate, another by read name) will produce misleading concordance metrics—always ensure inputs share the same sort order using `samtools sort`.
- **Ignoring Secondary/Supplementary Alignments**: By default, bttcmp treats secondary and supplementary alignments as distinct alignments; failing to filter these with SAM flags (e.g., `-F 2304`) can inflate discordance rates when you intended to compare primary alignments only.
- **Reference Index Drift**: If the reference genome used for alignment differs slightly (e.g., different chromosome naming conventions or versions), coordinate-based comparisons will fail silently or yield false discordance—explicitly verify the `@SQ` header lines match across inputs.
- **Inconsistent Read Groups**: When comparing BAM files with embedded read group tags, mismatched `@RG` records cause bttcmp to treat identical reads as distinct, reducing observed concordance without any real alignment difference.

## Examples

### Compare two BAM files and output concordance metrics
**