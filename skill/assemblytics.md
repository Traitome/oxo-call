---
name: assemblytics
category: variant-calling
description: Detect and analyze structural variants by comparing genome assemblies to a reference using anchor-based analysis. Identifies insertions, deletions, duplications, and other large-scale variations between assemblies.
tags:
  - structural-variants
  - assembly-comparison
  - genome-analysis
  - variant-calling
  - sanger-tol
author: AI-generated
source_url: https://github.com/sanger-tol/assemblytics
---

## Concepts

- **Anchor-Based Detection**: Assemblytics identifies structural variants by finding "anchor" regions - contiguous sequences of identical bases shared between the reference and query assembly. Variants are called when the alignment deviates from these anchors by more than the specified threshold.

- **Input Requirements**: The tool requires a reference genome in FASTA format and a query assembly aligned using MUMmer (`.sam` or `.aln` format). The alignment step must be performed separately before running Assemblytics, typically using `nucmer` from MUMmer with appropriate filters.

- **Output Formats**: Assemblytics produces multiple output files including a VCF file with variant calls (containing insertion, deletion, and duplication events), a BED file with coordinates suitable for genome browsers, and a coverage file showing anchor density across the genome.

- **Key Parameters**: The minimum variant size (`--min_variant`) filters out small variants below the threshold; window size (`--window_size`) defines the anchor detection window (default 10000bp); and maximum gap (`--max_gap`) controls the maximum distance between anchors to consider as a single event.

## Pitfalls

- **Running Without Prior Alignment**: Attempting to run Assemblytics directly on unaligned FASTA files will fail. The query assembly must first be aligned to the reference using `nucmer` or similar alignment tool, producing a SAM/ALN file that serves as input.

- **Inverting Reference and Query**: Specifying the reference and query in the wrong order will produce incorrect variant calls - insertions become deletions and vice versa. Always verify that the first input is the established reference and the second is the new assembly.

- **Setting Min_variant Too Low**: Using a minimum variant size smaller than the expected alignment noise will generate numerous false positive variants. Values below 50bp typically capture alignment artifacts rather than true structural variants.

- **Incorrect Window Size**: Setting a window size too small may fragment true variants into multiple smaller calls, while an oversized window may merge distinct variants into single events, losing resolution.

- **Memory Intensive for Large Genomes**: Processing chromosome-scale assemblies without sufficient memory can cause crashes. Assemblytics loads anchor data into memory; for mammalian genomes, allocate at least 16GB RAM.

## Examples

### Detect structural variants from a MUMmer alignment