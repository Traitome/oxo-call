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
- --by-strand generates separate consensus for each strand; useful for strand-specific analysis.
- --hd-finder enables heteroduplex detection and splitting; for samples with heteroduplex artifacts.
- --all-kinetics calculates kinetics for all ZMWs, not just HiFi reads; larger output but more data.
- --top-passes limits number of subreads used per ZMW (default 60); reduces runtime for high-pass libraries.
- --min-snr filters ZMWs by signal-to-noise ratio (default 2.5); lower values include more ZMWs.

## Pitfalls
- ccs requires PacBio-format subreads BAM — regular FASTQ input is not supported.
- Low-pass libraries (few SMRTbell passes) produce fewer HiFi reads — need ≥3 passes for Q20.
- --min-passes 3 ensures at least 3 full passes around the SMRTbell insert.
- ccs is CPU-intensive — use all available cores (-j) for faster processing.
- ccs output quality is influenced by SMRTbell library length and complexity.
- The --chunk option is for distributed/cluster processing — requires post-merge step.
- --by-strand doubles output size (separate files per strand); ensure sufficient disk space.
- --all-kinetics generates much larger output; use --hifi-kinetics for smaller methylation-ready files.
- --skip-polish outputs draft consensus only; faster but lower accuracy, not recommended for HiFi.
- --subread-fallback uses representative subread if polishing fails; may include lower quality reads.

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

### generate strand-specific HiFi reads
**Args:** `subreads.bam hifi_by_strand.bam -j 32 --min-rq 0.99 --by-strand`
**Explanation:** --by-strand generates separate consensus for forward and reverse strands

### detect and split heteroduplex artifacts
**Args:** `subreads.bam hifi_split.bam -j 32 --min-rq 0.99 --hd-finder`
**Explanation:** --hd-finder identifies heteroduplex molecules and splits them; for amplicon data

### output all ZMWs including low quality
**Args:** `subreads.bam all_reads.bam -j 32 --min-rq 0.99 --all`
**Explanation:** --all outputs all ZMWs, not just HiFi; includes failed and below-threshold reads

### use subread fallback for failed polishing
**Args:** `subreads.bam hifi_with_fallback.bam -j 32 --min-rq 0.99 --subread-fallback`
**Explanation:** --subread-fallback emits representative subread if polishing fails; maximizes yield

### calculate kinetics for all ZMWs
**Args:** `subreads.bam hifi_all_kinetics.bam -j 32 --min-rq 0.99 --all-kinetics`
**Explanation:** --all-kinetics adds IPD/PW for all ZMWs; larger output but complete kinetics data

### limit top passes for faster processing
**Args:** `subreads.bam hifi_fast.bam -j 32 --min-rq 0.99 --top-passes 30`
**Explanation:** --top-passes 30 uses only top 30 subreads per ZMW; faster for high-pass libraries

### generate report files for QC
**Args:** `subreads.bam hifi_reads.bam -j 32 --min-rq 0.99 --report-file ccs_report.txt --report-json ccs_report.json`
**Explanation:** --report-file and --report-json generate detailed QC reports in text and JSON formats
