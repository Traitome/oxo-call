---
name: bubblegun
category: Genomic Analysis / Variant Detection
description: A tool for detecting and resolving structural variant bubbles in genomic sequences or assembly graphs. bubblegun identifies regions with alternative paths or high variant density and outputs consensus sequences or annotated variant calls. Supports FASTA, FASTQ, and GFA input formats.
tags: [structural-variants, bubble-detection, assembly-graph, genomics, variant-calling]
author: AI-generated
source_url: https://github.com/bubblegun-tools/bubblegun
---

## Concepts

- bubblegun interprets genomic data as a graph of overlapping sequences, where bubbles represent regions with multiple valid paths (e.g., heterozygous variants, repeat expansions, or assembly ambiguities). It traverses these graphs to identify and quantify alternate alleles.
- Input files must be in FASTA (reference/consensus), FASTQ (read data), or GFA (graph format). bubblegun auto-detects file format from extension but accepts explicit `--format` flags to override detection behavior.
- Output modes determine the result type: consensus sequences (`--mode consensus`), variant calls in BED/VCF (`--mode variants`), or annotated graphs (`--mode graph`). Each mode produces a distinct output file suffix (`.fa`, `.vcf`, `.gfa`).

## Pitfalls

- Specifying `--min-coverage 0` or omitting it entirely causes bubblegun to treat all bubbles as valid, including sequencing errors. This results in thousands of false-positive variant calls that inflate downstream analysis files.
- Mixing input formats (e.g., passing a GFA file alongside a FASTQ file) without using `--graph-first` causes bubblegun to process read-based bubbles before graph bubbles, leading to inconsistent allele frequency calculations and corrupted output.
- Using `--bubble-size-limit` values smaller than the read length causes bubblegun to exit with a size-validation error. The limit must be at least the maximum input read length plus a 10% padding margin for overlap regions.

## Examples

### Detect all bubbles in a GFA assembly graph
**Args:** `input-graph assembly.gfa --mode variants --output bubbles`
**Explanation:** This processes the assembly graph to identify all bubble regions and outputs annotated variants in BED format with flanking sequences.

### Resolve bubbles with high-confidence allele support
**Args:** `input-reads sample_R1.fastq.gz sample_R2.fastq.gz --format fastq --min-coverage 10 --mode consensus --output resolved`
**Explanation:** This filters read pairs for regions with at least 10x coverage, resolves alternate alleles, and outputs a consensus FASTA sequence.

### Find large structural variant bubbles exceeding 500 bp
**Args:** `input-graph structural.gfa --bubble-size-limit 500 --mode graph --output large_sv_bubbles`
**Explanation:** This identifies graph bubbles longer than 500 base pairs, which often represent large insertions, deletions, or translocation breakpoints.

### Export bubble annotations with allele frequencies
**Args:** `input-graph ecoli_assembly.gfa --output-annotations allele_freqs --mode variants`
**Explanation:** This generates a variant call file annotated with per-bubble allele frequencies calculated from supporting read counts.

### Process multiple graphs sequentially with a configuration file
**Args:** `config batch_config.yaml --input-dir graphs/ --output-dir results/`
**Explanation:** This reads a YAML configuration file defining multiple input graphs and processes them sequentially, writing outputs to a separate directory while preserving file naming conventions.