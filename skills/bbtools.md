---
name: bbtools
category: sequence-processing
description: BBTools suite — BBDuk (adapter/quality trimming), BBMap (alignment), BBMerge (paired-end merging), reformat.sh, and related tools for FASTQ/FASTA processing
tags: [bbtools, bbduk, bbmap, bbmerge, reformat, adapter-trimming, decontamination, fastq, fasta, ngs]
author: oxo-call built-in
source_url: "https://jgi.doe.gov/data-and-tools/software-tools/bbtools/"
---

## Concepts
- BBTools is a JVM-based suite installed in a directory (e.g. `~/bbtools/`); each tool is a shell script (`bbduk.sh`, `bbmap.sh`, etc.) that wraps `java -jar BBTools.jar`.
- Key tools: `bbduk.sh` (adapter trimming and quality filtering), `bbmap.sh` (alignment to a reference), `bbmerge.sh` (paired-end fragment merging), `reformat.sh` (format conversion and subsampling), `dedupe.sh` (duplicate removal), `splitnextera.sh`, `bbsplit.sh` (splitting reads by genome).
- BBDuk adapters file: `~/bbtools/resources/adapters.fa` — bundled with BBTools for Illumina adapters; use `ref=adapters.fa` to trim all common adapters.
- Memory allocation: set `jvm` flag or use the `-Xmx` Java heap flag; `bbduk.sh -Xmx4g` allocates 4 GB heap; default is ~85% of available RAM (auto-detection).
- BBMap index (`ref=`) is built on first run and cached in a `ref/` directory in the CWD; subsequent runs reuse the index.
- `in=` and `in2=` for paired-end reads; `out=`, `out2=` for paired output; `outs=` for singletons.
- `threads=` or `t=` controls CPU usage; default is all available cores.
- BBMerge combines paired reads that overlap into single merged reads; useful before assembly or for amplicon analysis.
- `reformat.sh` is a versatile conversion tool: FASTQ↔FASTA, interleaved↔paired, subsample, compress/decompress, rename reads.
- Decontamination: `bbduk.sh ref=contaminants.fa` filters out reads matching the reference (host removal, PhiX spike removal).

## Pitfalls
- BBTools requires Java 8 or later; ensure `JAVA_HOME` points to a compatible JVM or load the java module on HPC.
- `bbmap.sh` `nodisk=t` stores the reference index in RAM only; fine for short references but will fail with OutOfMemoryError for large genomes without sufficient heap.
- `k=` (kmer length) in BBDuk affects sensitivity: shorter k (e.g. k=21) catches more adapter contamination but increases false positives; default k=23 is a good balance.
- BBDuk `trimq=` and `qtrim=rl` (trim both ends) are separate from `maq=` (minimum average quality for read discard); use both together for comprehensive QC.
- BBMap output SAM/BAM: by default, outputs SAM; add `bamscript=bs.sh; sh bs.sh` or pipe to `samtools` to get a sorted BAM.
- The `ref/` index directory is created in the working directory by default; set `path=` to change the index location.
- Running multiple BBTools jobs in the same directory without specifying `path=` causes index conflicts; always specify unique paths.

## Examples

### trim adapters and quality-filter with BBDuk
**Args:** `bbduk.sh in=R1.fastq.gz in2=R2.fastq.gz out=R1_trimmed.fastq.gz out2=R2_trimmed.fastq.gz ref=adapters.fa ktrim=r k=23 mink=11 hdist=1 tpe tbo qtrim=r trimq=20 minlen=50`
**Explanation:** ktrim=r trims adapters on the right; tpe trims adapter pairs equally; tbo trims by overlap; qtrim=r trims low-quality 3' bases below Q20; minlen=50 discards short reads

### remove PhiX contamination from a FASTQ
**Args:** `bbduk.sh in=R1.fastq.gz in2=R2.fastq.gz out=clean_R1.fastq.gz out2=clean_R2.fastq.gz ref=phix174_ill.ref.fa.gz k=31 hdist=1`
**Explanation:** filters out reads matching PhiX174; k=31 kmer matching; hdist=1 allows 1 mismatch; bundled PhiX reference in BBTools resources/

### align reads to a reference genome
**Args:** `bbmap.sh in=reads.fastq.gz ref=genome.fa out=aligned.sam`
**Explanation:** builds index in ref/ on first run; aligns reads.fastq.gz; outputs SAM; add threads=16 for parallelism

### merge overlapping paired-end reads
**Args:** `bbmerge.sh in=R1.fastq.gz in2=R2.fastq.gz out=merged.fastq.gz outu=unmerged_R1.fastq.gz outu2=unmerged_R2.fastq.gz`
**Explanation:** merges overlapping read pairs into single longer reads; unmerged reads go to outu/outu2; useful for amplicon analysis

### subsample a FASTQ file to a specific number of reads
**Args:** `reformat.sh in=large.fastq.gz out=subset.fastq.gz samplereadstarget=1000000`
**Explanation:** randomly subsamples to 1M reads; deterministic seed ensures reproducibility; works with gzipped input/output

### convert FASTQ to FASTA
**Args:** `reformat.sh in=reads.fastq.gz out=reads.fa`
**Explanation:** converts FASTQ to FASTA by stripping quality scores; reformat.sh auto-detects formats from file extensions

### remove host reads before metagenomics analysis
**Args:** `bbmap.sh in=sample.fastq.gz ref=human_genome.fa outm=host_reads.fastq.gz outu=non_host_reads.fastq.gz nodisk=t`
**Explanation:** outm captures reads mapping to human genome; outu is the cleaned (non-host) output; nodisk=t keeps index in RAM for smaller references

### get detailed statistics for a FASTQ file
**Args:** `reformat.sh in=reads.fastq.gz`
**Explanation:** with no out= specified, reports read counts, length distribution, GC content, and quality score statistics without producing output files
