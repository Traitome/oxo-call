---
name: addeam
category: DNA Modification Detection
description: A tool for calling adenine modifications (Ad) from nanopore sequencing data using signal-level analysis. Computes modification frequencies and generates per-site modification scores aligned to a reference genome, producing bedMethyl, bedGraph, or BigWig output formats for downstream epigenomic analysis.
tags: [nanopore, epigenetic-modifications, basecalling, adenine-modifications, modkit-suite]
author: AI-generated
source_ url: https://github.com/UCSC-nanopore/addeam
---

## Concepts

- **Signal-based k-mer model**: addeam reads raw nanopore signal data and maps it to a reference genome using a k-mer model that assigns modification probabilities. Each canonical k-mer has a canonical signal level, and deviations from this expected signal indicate chemical modifications on the DNA template.
- **Output formats**: addeam produces bedMethyl (recommended) with columns for chromosome, start, end, motif, coverage, modified fraction, and raw modification scores. This format integrates directly with genome browsers and downstream epigenomic tools like methylartist.
- **Per-read vs. per-site calling**: By default addeam aggregates signals across all reads covering each genomic position to produce per-site modification fractions. The `--mod-binary` output preserves per-read calls for haplotype-resolved or single-molecule analysis.
- **Motif specification**: Modifications are called within a sequence context (motif), for example "GAN" for CpG-adjacent adenine or "A" for all adenines. Specifying the correct motif reduces false positives from non-canonical base events.

## Pitfalls

- **Insufficient read coverage**: Calling modifications with fewer than 20 reads per site produces noisy fractions with high variance. The reported modified fraction may be statistically meaningless and appear as spurious intermediate values (0.3–0.7 range) rather than clean bimodal distributions.
- **Reference misalignments**: If the aligned reference contains insertions or deletions relative to the sequenced reads, signal-to-base mapping becomes misaligned, causing phantom modification calls or systematic suppression of real signals.
- **Using an outdated k-mer model**: The k-mer model shipped with addeam must match the basecaller version used to generate the FAST5 or POD5 input. A model trained on a different chemistry or device version produces systematically biased modification probabilities.
- **Ignoring strand orientation**: Nanopore signal is direction-aware; forward and reverse strand reads have opposite signal polarity. Calling modifications without specifying `---stranded` collapses both strands into one, doubling apparent coverage but producing nonsensical modification fractions.

## Examples

### Call adenine modifications genome-wide with default settings
**Args:** `call-mods --bam input.bam --ref reference.fasta --output modified_sites.bed`
**Explanation:** This runs the standard per-site modification calling workflow, aligning nanopore reads to the reference and outputting a bedMethyl file with modification fractions for all adenines genome-wide.

### Call only adenines within a specific sequence motif
**Args:** `call-mods --bam input.bam --ref reference.fasta --motif GAN --output gan_mods.bed`
**Explanation:** Restricting calls to the GAN motif (guanine-adenine-any base) focuses analysis on a specific epigenetically relevant context and reduces output file size and noise from unrelated signals.

### Filter output by minimum read coverage
**Args:** `call-mods --bam input.bam --ref reference.fasta --output high_conf.bed --min-coverage 50`
**Explanation:** Requiring at least 50 reads per site ensures only statistically robust modification calls appear in the output, eliminating low-confidence sites that would otherwise introduce false positives in downstream analysis.

### Export per-read modification calls in binary format
**Args:** `call-mods --bam input.bam --ref reference.fasta --output reads_mods.modb --mod-binary`
**Explanation:** Generating binary per-read output preserves single-molecule modification information for haplotype-resolved analysis, allowing downstream tools to reconstruct modification patterns on individual DNA molecules.

### Generate BigWig track for genome browser visualization
**Args:** `call-mods --bam input.bam --ref reference.fasta --output mods.bw --bigwig`
**Explanation:** Converting output to BigWig format enables efficient genome browser visualization of modification frequencies across large genomic regions without loading full bedMethyl files.