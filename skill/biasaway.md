---
name: biasaway
category: Genomics / Background Generation
description: A tool for generating matched random genomic backgrounds for enrichment analysis. Creates control regions that preserve genomic characteristics (GC content, sequence composition, region length) to ensure statistical rigor in ChIP-seq, ATAC-seq, and other genomic enrichment analyses.
tags:
  - genomics
  - background-generation
  - enrichment-analysis
  - random-sampling
  - gc-content
  - chromatin-biology
author: AI-generated
source_url: https://github.com/marcaurele/biasaway
---

## Concepts

- **Input formats:** biasaway accepts BED files containing genomic coordinates (chromosome, start, end) as the target set, and optionally a genome assembly name for chromosome length lookup. It can also accept FASTA sequences for sequence composition matching.
- **Matching algorithms:** The tool provides multiple matching strategies including GC content matching (`--match gc`), dinucleotide frequency matching (`--match dinuc`), and length distribution matching. The `-k` flag specifies the matching method.
- **Output format:** Generates a BED file containing shuffled/randomized genomic coordinates that match the statistical properties of the input regions. Regions are sampled from the same genome but avoiding overlap with original targets.
- **Genome-aware shuffling:** When provided with a genome assembly (via `-g` or `--genome`), biasaway ensures generated regions stay within valid chromosome bounds and use chromosome-specific GC content distributions.
- **Iteration control:** The `-n` flag controls the number of randomized datasets to generate, which is useful for empirical p-value calculation in enrichment analyses.

## Pitfalls

- **Using an unsupported or misspelled genome name:** Specifying a genome that isn't in biasaway's built-in database (e.g., `--genome hg38` instead of `hg38` or `GRCh38`) will fail silently or produce errors about missing chromosome information, resulting in no output.
- **Overlapping target regions in output:** If the input BED file contains many overlapping regions, the pool of valid genomic space for sampling may be insufficient, causing biasaway to produce fewer regions than requested or crash with a sampling error.
- **Mismatch between input chromosome naming conventions:** Using `chr1` style names in input but requesting a genome build that uses `1` style (or vice versa) will cause zero matches, as chromosome names won't align; always verify naming consistency.
- **Insufficient genomic space for large sample requests:** Requesting a very large number of random regions (`-n` with high values) from a small genome or restricted chromosome may fail because there isn't enough non-overlapping space to sample from.

## Examples

### Generate 10 random background regions matching GC content
**Args:** `input_peaks.bed -o random_background.bed -n 10 -k gc`
**Explanation:** Creates 10 randomized genomic regions that preserve the GC content distribution of the input peak file, useful as a background control for ChIP-seq enrichment.

### Generate backgrounds matched on dinucleotide frequency
**Args:** `chip_peaks.bed -o dinuc_matched.bed -k dinuc`
**Explanation:** Uses dinucleotide frequency matching to preserve local sequence composition patterns critical for transcription factor binding site analysis.

### Create backgrounds for a specific genome assembly
**Args:** `atac_peaks.bed -o random_atac.bed -g hg38 -k gc`
**Explanation:** Generates GC-matched random regions using chromosome bounds and GC distributions from the hg38 genome build, ensuring coordinates are valid.

### Generate multiple randomized backgrounds for empirical p-values
**Args:** `peaks.bed -o empirical_backgrounds.bed -n 100 -k gc`
**Explanation:** Creates 100 different randomized background sets to enable empirical p-value calculation, providing statistical significance of observed enrichment.

### Generate length-matched random regions without sequence composition matching
**Args:** `enhancers.bed -o shuffled_enhancers.bed -k length`
**Explanation:** Produces random regions that match only the length distribution of input enhancers without GC or sequence composition constraints, useful when length is the primary confound.