---
name: art
category: Read Simulation
description: Bioinformatics tool that simulates next-generation sequencing (NGS) reads from a reference sequence, supporting amplicon-based sequencing (ART) error profiles for Illumina, Roche 454, and Ion Torrent platforms.
tags:
  - read simulation
  - NGS
  - amplicon sequencing
  - synthetic data
  - testing
  - variant calling
author: AI-generated
source_url: https://github.com/yhong/ART
---

## Concepts

- **Input Format**: ART takes a reference genome in FASTA format (single or multiple sequences) as input, simulating reads directly from each sequence in the file.
- **Output Format**: Produces simulated paired-end or single-end FASTQ reads with realistic base-call quality scores (Phred scores) that mirror the error profile of the specified sequencing platform.
- **Platform Profiles**: Supports three built-in error profiles—Illumina (MiSeq/HiSeq), Roche 454, and Ion Torrent—each with characteristic indel and substitution error rates that change with base position and quality.
- **Coverage Control**: Uses the `-c` or `--coverage` flag to specify target coverage depth (e.g., 100x), which determines the total number of reads generated proportional to the reference length.
- **Read Parameters**: Allows independent specification of read length (`-l`), mate pair inner fragment size (`-m`), and standard deviation (`-s`) for paired-end simulations.

## Pitfalls

- **Mismatched Read Lengths for Paired Ends**: Setting different read lengths with `-l` for single-end simulations or using mismatched `--len1`/`--len2` for paired-end data creates inconsistent read pairs that downstream aligners may reject, leading to alignment failures.
- **Insufficient Coverage for Testing**: Using very low coverage values (e.g., below 10x) produces too few reads to reliably test variant calling sensitivity, causing false negatives in pipeline validation.
- **Invalid Fragment Size**: Specifying a mean inner fragment size (`-m`) smaller than the sum of read lengths forces ART to fail or produce invalid read pairs; the inner fragment size must exceed the combined read lengths.
- **Missing Output Directory**: Running ART without specifying an output directory (`-o`) causes all output files to overwrite each other when processing multiple reference sequences, resulting in lost data.

## Examples

### Simulate paired-end Illumina reads from a reference genome

**Args:** -ss HS25 -sam -i reference.fasta -o output_prefix -l 150 -f 200 -c 100
**Explanation:** Uses the built-in Illumina HiSeq 2500 profile (`-ss HS25`) to generate paired-end reads with 150 bp length, 200 bp inner fragment, and 100x coverage from the reference FASTA, also producing a SAM alignment file for validation.

### Simulate single-end reads with reduced error rate

**Args:** -ss MSv3 -i reference.fasta -o output_prefix -l 250 -c 50 -e 0.001
**Explanation:** Generates single-end MiSeq v3 reads at 250 bp length with an artificially lowered base error rate (`-e 0.001`) for testing pipelines that require higher accuracy or when simulating low-error regions.

### Simulate paired-end reads with large insert size for mate-pair sequencing

**Args:** -ss HS10X -i reference.fasta -o output_prefix -l 100 -m 2000 -s 200 -c 30
**Explanation:** Creates Illumina HiSeq 10X paired-end reads with 100 bp reads and 2000 bp mean inner fragment size (200 bp standard deviation) at 30x coverage, suitable for testing mate-pair library processing pipelines.

### Simulate Ion Torrent amplicon reads with specific output prefix

**Args:** -ss IonTorrent -i amplicon_ref.fasta -o amplicon_sim -l 200 -f 10 -c 500
**Explanation:** Uses the Ion Torrent error profile to simulate amplicon sequencing reads with 200 bp length, 10 bp inner fragment (effectively head-to-tail amplicons), and very high 500x coverage for testing amplicon-based variant callers.

### Generate paired-end reads without a SAM file to save disk space

**Args:** -ss HS25 -i reference.fasta -o output_prefix -l 125 -f 300 -c 50
**Explanation:** Produces Illumina paired-end 125 bp reads with 300 bp fragment and 50x coverage while skipping SAM output, saving storage and I/O time when only FASTQ files are needed for downstream processing.