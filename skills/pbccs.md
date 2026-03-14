---
name: pbccs
category: long-read
description: PacBio CCS (Circular Consensus Sequencing) for generating high-fidelity HiFi reads from SMRT sequencing
tags: [pacbio, hifi, ccs, long-read, basecalling, consensus, smrt]
author: oxo-call built-in
source_url: "https://github.com/PacificBiosciences/ccs"
---

## Concepts

- ccs (formerly PacBio CCS) generates HiFi reads from PacBio SMRT subreads by circular consensus sequencing.
- Input: PacBio subreads BAM (from SMRT Cell); output: HiFi BAM with Q≥20 (99% accuracy) reads.
- Use --min-rq to set minimum read quality (default 0.99 = Q20); --min-passes for minimum SMRTbell passes.
- Use --chunk N/M for distributed computing (chunk N of M total).
- Use -j N for multi-threading; output BAM contains only HiFi-quality reads by default.
- ccs integrates both primary consensus generation and kinetics calculation for methylation calling.
- The --all flag outputs all reads including below-quality-threshold reads.
- For methylation: use --hifi-kinetics to include base modification information.

## Pitfalls

- ccs requires PacBio-format subreads BAM — regular FASTQ input is not supported.
- Low-pass libraries (few SMRTbell passes) produce fewer HiFi reads — need ≥3 passes for Q20.
- --min-passes 3 ensures at least 3 full passes around the SMRTbell insert.
- ccs is CPU-intensive — use all available cores (-j) for faster processing.
- ccs output quality is influenced by SMRTbell library length and complexity.
- The --chunk option is for distributed/cluster processing — requires post-merge step.

## Examples

### generate HiFi reads from PacBio subreads BAM
**Args:** `subreads.bam hifi_reads.bam -j 32 --min-rq 0.99`
**Explanation:** input subreads BAM; output HiFi BAM; -j 32 threads; --min-rq 0.99 minimum Q20 quality

### generate HiFi reads with minimum 3 SMRTbell passes
**Args:** `subreads.bam hifi_reads.bam -j 32 --min-rq 0.99 --min-passes 3`
**Explanation:** --min-passes 3 ensures 3 complete circular passes; improves consensus accuracy

### generate HiFi reads with kinetics for methylation calling
**Args:** `subreads.bam hifi_reads_kinetics.bam -j 32 --min-rq 0.99 --hifi-kinetics`
**Explanation:** --hifi-kinetics adds IPD and pulse width tags for subsequent methylation analysis

### generate HiFi reads using chunked processing for distributed computing
**Args:** `subreads.bam chunk1.bam -j 16 --min-rq 0.99 --chunk 1/4`
**Explanation:** --chunk 1/4 processes first quarter; run chunks 1-4 in parallel, then merge with pbmerge
