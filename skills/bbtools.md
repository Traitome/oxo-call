---
name: bbtools
category: sequence-utilities
description: BBTools suite — BBDuk (adapter/quality trimming), BBMap (alignment), BBMerge (paired-end merging), reformat.sh, and related tools for FASTQ/FASTA processing
tags: [bbtools, bbduk, bbmap, bbmerge, reformat, adapter-trimming, decontamination, fastq, fasta, ngs, java]
author: oxo-call built-in
source_url: "https://jgi.doe.gov/data-and-tools/software-tools/bbtools/"
---

## Concepts
- BBTools is a JVM-based suite with 280+ tools; each tool is a shell script (`bbduk.sh`, `bbmap.sh`, etc.) that wraps `java -jar BBTools.jar`.
- Key tools: `bbduk.sh` (adapter trimming and quality filtering), `bbmap.sh` (alignment to a reference), `bbmerge.sh` (paired-end fragment merging), `reformat.sh` (format conversion and subsampling), `dedupe.sh` (duplicate removal), `bbsplit.sh` (splitting reads by genome), `bbnorm.sh` (read normalization), `bbcms.sh` (variable-depth coverage normalization).
- BBDuk adapters file: bundled at `resources/adapters.fa` — use `ref=adapters.fa` to trim all common Illumina adapters.
- Memory allocation: `bbduk.sh -Xmx4g` allocates 4 GB Java heap; default is ~85% of available RAM (auto-detection). Always set -Xmx on shared HPC systems.
- BBMap index (`ref=`) is built on first run and cached in a `ref/` directory in the CWD; subsequent runs reuse the index.
- `in=` and `in2=` for paired-end reads; `out=`, `out2=` for paired output; `outs=` for singletons; `outu=`/`outu2=` for unmerged reads.
- `threads=` or `t=` controls CPU usage; default is all available cores.
- BBMerge combines paired reads that overlap into single merged reads; useful before assembly or for amplicon analysis.
- `reformat.sh` is a versatile conversion tool: FASTQ↔FASTA, interleaved↔paired, subsample, compress/decompress, rename reads.
- Decontamination: `bbduk.sh ref=contaminants.fa` filters out reads matching the reference (host removal, PhiX spike removal).
- BBTools uses `key=value` syntax for parameters, NOT the typical `-flag value` CLI pattern. E.g., `in=reads.fq out=clean.fq ref=adapters.fa`.
- Other useful tools: `stats.sh` (assembly/FASTQ statistics), `comparesketch.sh` / `sendsketch.sh` (sketch-based taxonomy), `callvariants.sh` (variant calling), `pileup.sh` (pooled variant calling), `samtoroc.sh` (ROC curves).
- `bbnorm.sh` normalizes read depth by kmer-based subsampling; useful before assembly of uneven-coverage data.

## Pitfalls
- BBTools uses `key=value` syntax, not `-flag value`. ARGS must follow this pattern: `bbduk.sh in=file.fq out=clean.fq ref=adapters.fa`. Never use `-in file.fq` or `--in file.fq`.
- BBTools requires Java 8 or later; ensure `JAVA_HOME` points to a compatible JVM or load the java module on HPC.
- `bbmap.sh` `nodisk=t` stores the reference index in RAM only; fine for short references but will fail with OutOfMemoryError for large genomes without sufficient heap.
- `k=` (kmer length) in BBDuk affects sensitivity: shorter k (e.g. k=21) catches more adapter contamination but increases false positives; default k=23 is a good balance.
- BBDuk `trimq=` and `qtrim=rl` (trim both ends) are separate from `maq=` (minimum average quality for read discard); use both together for comprehensive QC.
- BBMap output SAM/BAM: by default outputs SAM; add `bamscript=bs.sh; sh bs.sh` or pipe to `samtools` to get sorted BAM.
- The `ref/` index directory is created in the working directory by default; set `path=` to change the index location.
- Running multiple BBTools jobs in the same directory without specifying `path=` causes index conflicts; always specify unique paths.
- BBTools scripts typically end in `.sh` (e.g., `bbduk.sh`, not `bbduk`). Include the `.sh` suffix when invoking unless the environment provides wrapper symlinks (e.g., conda-installed BBTools).
- When installed via pixi/conda, BBTools scripts may not be in PATH — use full path: `~/.pixi/envs/bbmap/bin/bbduk.sh` or add the bin directory to PATH.

## Examples

### trim adapters and quality-filter with bbduk.sh
**Args:** `bbduk.sh in=R1.fastq.gz in2=R2.fastq.gz out=R1_trimmed.fastq.gz out2=R2_trimmed.fastq.gz ref=adapters.fa ktrim=r k=23 mink=11 hdist=1 tpe tbo qtrim=r trimq=20 minlen=50`
**Explanation:** bbduk.sh tool; in=R1.fastq.gz in2=R2.fastq.gz paired input; out=/out2= trimmed outputs; ref=adapters.fa adapter reference; ktrim=r trims adapters on the right; k=23 kmer length; mink=11 minimum kmer; hdist=1 hamming distance; tpe trims adapter pairs equally; tbo trims by overlap; qtrim=r quality trim 3' end; trimq=20 quality threshold; minlen=50 discards short reads

### remove PhiX contamination with bbduk.sh
**Args:** `bbduk.sh in=R1.fastq.gz in2=R2.fastq.gz out=clean_R1.fastq.gz out2=clean_R2.fastq.gz ref=phix174_ill.ref.fa.gz k=31 hdist=1`
**Explanation:** bbduk.sh tool; in=R1.fastq.gz in2=R2.fastq.gz paired input; out=/out2= clean outputs; ref=phix174_ill.ref.fa.gz PhiX reference; k=31 kmer matching; hdist=1 allows 1 mismatch; bundled PhiX reference in BBTools resources/

### align reads to a reference genome with bbmap.sh
**Args:** `bbmap.sh in=reads.fastq.gz ref=genome.fa out=aligned.sam threads=16`
**Explanation:** bbmap.sh alignment tool; in=reads.fastq.gz input reads; ref=genome.fa reference genome; out=aligned.sam output SAM; threads=16 parallelism; builds index in ref/ on first run

### merge overlapping paired-end reads with bbmerge.sh
**Args:** `bbmerge.sh in=R1.fastq.gz in2=R2.fastq.gz out=merged.fastq.gz outu=unmerged_R1.fastq.gz outu2=unmerged_R2.fastq.gz`
**Explanation:** bbmerge.sh tool; in=R1.fastq.gz in2=R2.fastq.gz paired input; out=merged.fastq.gz merged output; outu=/outu2= unmerged outputs; merges overlapping read pairs into single longer reads; useful for amplicon analysis

### subsample a FASTQ file with reformat.sh
**Args:** `reformat.sh in=large.fastq.gz out=subset.fastq.gz samplereadstarget=1000000`
**Explanation:** reformat.sh tool; in=large.fastq.gz input; out=subset.fastq.gz output; samplereadstarget=1000000 randomly subsamples to 1M reads; deterministic seed ensures reproducibility; works with gzipped input/output

### convert FASTQ to FASTA with reformat.sh
**Args:** `reformat.sh in=reads.fastq.gz out=reads.fa`
**Explanation:** reformat.sh tool; in=reads.fastq.gz input; out=reads.fa output FASTA; converts FASTQ to FASTA by stripping quality scores; reformat.sh auto-detects formats from file extensions

### remove host reads with bbmap.sh
**Args:** `bbmap.sh in=sample.fastq.gz ref=human_genome.fa outm=host_reads.fastq.gz outu=non_host_reads.fastq.gz nodisk=t`
**Explanation:** bbmap.sh tool; in=sample.fastq.gz input reads; ref=human_genome.fa human reference; outm=host_reads.fastq.gz mapped (host) output; outu=non_host_reads.fastq.gz unmapped (cleaned) output; nodisk=t keeps index in RAM for smaller references

### get detailed FASTQ statistics with reformat.sh
**Args:** `reformat.sh in=reads.fastq.gz`
**Explanation:** reformat.sh tool; in=reads.fastq.gz input; with no out= specified, reports read counts, length distribution, GC content, and quality score statistics without producing output files

### remove duplicate reads with dedupe.sh
**Args:** `dedupe.sh in=reads.fastq.gz out=deduped.fastq.gz`
**Explanation:** dedupe.sh tool; in=reads.fastq.gz input; out=deduped.fastq.gz output; removes exact and near-identical duplicate reads; useful before assembly or for reducing PCR duplicate bias in amplicon data

### split reads by genome of origin with bbsplit.sh
**Args:** `bbsplit.sh in=sample.fastq.gz ref=genome1.fa,genome2.fa out_genome1=reads_genome1.fastq.gz out_genome2=reads_genome2.fastq.gz`
**Explanation:** bbsplit.sh tool; in=sample.fastq.gz input reads; ref=genome1.fa,genome2.fa multiple references; out_genome1=/out_genome2= binned outputs; competitively maps reads to multiple references and bins them by best hit; reads with no match go to ambiguous output; useful for host/pathogen separation

### quality-filter and trim adapters with bbduk.sh and memory control
**Args:** `bbduk.sh -Xmx8g in=R1.fastq.gz in2=R2.fastq.gz out=R1_clean.fastq.gz out2=R2_clean.fastq.gz ref=adapters.fa ktrim=r k=23 mink=11 hdist=1 qtrim=rl trimq=20 minlen=50 threads=16`
**Explanation:** bbduk.sh tool; -Xmx8g limits Java heap to 8GB; in=/in2= paired inputs; out=/out2= clean outputs; ref=adapters.fa adapter reference; ktrim=r right trimming; k=23 kmer length; mink=11 minimum kmer; hdist=1 hamming distance; qtrim=rl quality trim both ends; trimq=20 quality threshold; minlen=50 minimum length; threads=16 parallel processing; combines adapter trimming and quality filtering

### interleave paired-end FASTQ files with reformat.sh
**Args:** `reformat.sh in=R1.fastq.gz in2=R2.fastq.gz out=interleaved.fastq.gz`
**Explanation:** reformat.sh tool; in=R1.fastq.gz in2=R2.fastq.gz paired inputs; out=interleaved.fastq.gz output; combines separate R1/R2 files into a single interleaved file; useful for tools that expect interleaved input

### generate quality and length statistics with bbduk.sh
**Args:** `bbduk.sh in=reads.fastq.gz bhist=base_hist.txt qhist=quality_hist.txt lhist=length_hist.txt`
**Explanation:** bbduk.sh tool; in=reads.fastq.gz input; bhist=base_hist.txt base composition histogram; qhist=quality_hist.txt quality histogram; lhist=length_hist.txt length distribution histogram; outputs histograms for quality analysis

### normalize read depth with bbnorm.sh
**Args:** `bbnorm.sh in=reads.fastq.gz out=normalized.fastq.gz target=100 min=5`
**Explanation:** bbnorm.sh tool; in=reads.fastq.gz input; out=normalized.fastq.gz output; target=100 normalizes to ~100x coverage; min=5 discards reads with kmer depth below 5; useful before assembly of uneven-coverage data

### get comprehensive assembly statistics with stats.sh
**Args:** `stats.sh in=contigs.fa`
**Explanation:** stats.sh tool; in=contigs.fa input assembly; reports N50, L50, total length, contig count, GC content, and other assembly metrics; works with FASTA/FASTQ
