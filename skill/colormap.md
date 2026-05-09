---
name: colormap
category: sequence_analysis
description: Converts color space data to base space sequence for Oxford Nanopore reads, commonly used with nanopolish output for variant calling and haplotype analysis.
tags: [color-space, nanopore, base-space, conversion, variant-calling, haplotype]
author: AI-generated
source_url: https://github.com/nanopolish/nanopolish
---

## Concepts

- **Color Space Encoding**: Colormap interprets data in a 4-color encoding scheme (A, C, G, T represented by distinct colors) where each base is encoded relative to the preceding base, requiring a primer base for initialization.
- **Input Format Requirements**: Input files must contain color space encoded sequences (typically from nanopolish) with a reference primer base prepended; the tool cannot process standard base space FASTQ directly without prior conversion.
- **Output Modes**: The tool supports both verbose (showing per-base confidence and mapping quality) and summary (aggregated statistics) output modes, controlled via the `--verbose` and `--summary` flags respectively.
- **Companion Binary Usage**: After converting color space to base space, use `colormap-call` (the variant caller) for actualSNP detection, as colormap alone only performs sequence conversion.

## Pitfalls

- **Missing Primer Base**: Providing color space sequences without the initial primer base causes the first 2-3 bases of output to be incorrect or contain invalid characters, corrupting downstream variant calls.
- **Confusing Color Space with Base Space**: Attempting to map directly to a reference using color space data produces false indels and mismatches because the alignment algorithms expect base space sequences, leading to incorrect variant calls.
- **Inconsistent Encoding Schemes**: Using different color encoding dictionaries ( dye-based vs. diode-based) between the input data and colormap settings produces silently corrupted base translations.
- **Large File Memory Usage**: Processing whole-read color space files without chunking (超过10,000 reads at once) can exhaust available RAM on systems with limited memory, causing crashes or OOM kills.
- **Duplicate Sequences**: Failing to deduplicate reads before conversion may artifactually inflate allele frequency estimates in downstream analysis, as identical converted sequences appear multiple times.

## Examples

### Convert a color space FASTQ to base space

**Args:** `--input reads.cs.fastq --output reads.basespace.fastq --format fastq`

**Explanation:** Reads standard color space FASTQ and outputs equivalent nucleotide sequences, enabling standard alignment tools to process the data.

### Enable verbose per-base quality output

**Args:** `--input alignments.cs.tsv --verbose --output converted.tsv`

**Explanation:** Outputs detailed per-base information including the original color encoding, converted base, and quality scores for debugging conversion issues.

### Process multiple input files with parallel threads

**Args:** `--input-dir color_space/ --output-dir base_space/ --threads 8 --batch-size 5000`

**Explanation:** Processes directory of color space files using 8 parallel threads in batches of 5000 reads each, significantly improving throughput on multicore systems.

### Specify a non-default color encoding dictionary

**Args:** `--input reads.cs.fastq --output reads.basespace.fastq --encoding dye-nucleotide --primer GATGG`

**Explanation:** Uses custom dye-based color encoding with GATGG as the primer sequence when the input data uses non-standard color encoding.

### Generate summary statistics for downstream allele frequency analysis

**Args:** `--input variants.cs.tsv --summary --output variant_summary.tsv`

**Explanation:** Produces aggregated statistics showing base conversion rates, coverage per position, and called allele frequencies suitable for population analysis.