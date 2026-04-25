---
name: pbfusion
category: rna-seq
description: "PacBio's RNA fusion gene detector for long-read IsoSeq data"
tags: [rna-seq, fusion, pacbio, isoseq, long-read, transcript, cancer]
author: oxo-call built-in
source_url: "https://github.com/PacificBiosciences/pbfusion"
---

## Concepts
- pbfusion detects gene fusions from PacBio IsoSeq full-length RNA-seq data.
- Input: aligned IsoSeq BAM (aligned with pbmm2 using ISOSEQ preset) and a GTF annotation.
- pbfusion uses full-length read coverage to detect fusion breakpoints with base-pair resolution.
- Requires sorted, indexed, aligned IsoSeq BAM from pbmm2 --preset ISOSEQ.
- Output: fusions.tsv with fusion gene pairs, breakpoints, and supporting read counts.
- Long-read fusions are more precise than short-read fusions (full spanning reads).
- Use --threads for parallel processing; --output-prefix for results.
- pbfusion gff-cache creates binary cached GTF for faster repeated analysis.
- --min-coverage filters breakpoints by read support (default 2); increase for high confidence.
- --min-mean-identity (default 0.93) and --min-mean-mapq (default 10) filter low-quality alignments.
- --max-readthrough (default 100000) filters readthrough transcripts; reduce for stricter filtering.
- --allow-immune and --allow-mito permit fusions involving immunological/mitochondrial genes (often false positives).

## Pitfalls
- pbfusion requires IsoSeq data, not regular RNA-seq — it needs full-length transcripts.
- Input BAM must be aligned with IsoSeq-specific parameters (pbmm2 --preset ISOSEQ).
- pbfusion requires a Gencode or Ensembl GTF annotation file.
- Low-coverage fusions may be missed — deeper IsoSeq sequencing improves sensitivity.
- pbfusion gff-cache creates binary GTF; cache file must be regenerated when GTF changes.
- --min-coverage 2 filters singletons; increase to 3-5 for high-confidence fusion detection.
- --max-readthrough 100000 may include readthrough transcripts; reduce to 10000 for stricter filtering.
- --allow-immune and --allow-mito are disabled by default; enable only if studying immune/mitochondrial fusions.
- --min-fusion-quality MEDIUM (default) filters LOW quality calls; use LOW to include all calls.

## Examples

### detect gene fusions from PacBio IsoSeq aligned data
**Args:** `--bam isoseq_aligned.bam --gtf genes.gtf --output-dir fusion_output/ --threads 8`
**Explanation:** pbfusion command; --bam isoseq_aligned.bam aligned IsoSeq BAM input; --gtf genes.gtf gene annotation; --output-dir fusion_output/ results directory; --threads 8 parallel processing

### detect fusions with minimum supporting reads
**Args:** `--bam isoseq_aligned.bam --gtf genes.gtf --output-dir fusion_output/ --min-support 3 --threads 8`
**Explanation:** pbfusion command; --bam isoseq_aligned.bam aligned BAM; --gtf genes.gtf gene annotation; --output-dir fusion_output/ results directory; --min-support 3 requires at least 3 reads supporting each fusion; --threads 8 parallel processing; reduces false positives

### cache GTF annotation for faster analysis
**Args:** `gff-cache --gtf genes.gtf --output genes.gtf.bin`
**Explanation:** pbfusion gff-cache subcommand; --gtf genes.gtf input GTF annotation; --output genes.gtf.bin output binary GTF; speeds up repeated pbfusion discover runs

### detect fusions with cached GTF
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf.bin --output-prefix fusion_out --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf.bin cached binary GTF; --output-prefix fusion_out output prefix; --threads 8 parallel processing; use cached .bin GTF; much faster than parsing raw GTF each time

### high-confidence fusion detection with strict filters
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix strict --min-coverage 5 --min-mean-identity 0.95 --min-mean-mapq 20 --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix strict output prefix; --min-coverage 5 minimum reads; --min-mean-identity 0.95 identity threshold; --min-mean-mapq 20 mapping quality; --threads 8 parallel processing; strict filters: 5+ reads, 95% identity, MAPQ 20; high-confidence calls only

### include immunological gene fusions
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix immune --allow-immune --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix immune output prefix; --allow-immune permits fusions involving immunological genes; --threads 8 parallel processing; disabled by default

### reduce readthrough transcript detection
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix strict_rt --max-readthrough 10000 --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix strict_rt output prefix; --max-readthrough 10000 filters readthrough transcripts within 10kb; --threads 8 parallel processing; stricter fusion detection

### include all fusion quality levels
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix all --min-fusion-quality LOW --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix all output prefix; --min-fusion-quality LOW includes LOW quality calls; --threads 8 parallel processing; for sensitive detection

### filter complex fusion events
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix simple --max-genes-in-event 2 --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix simple output prefix; --max-genes-in-event 2 marks multi-gene events as low quality; --threads 8 parallel processing; reduces false positives

### detect fusions from multiple IsoSeq samples
**Args:** `discover --bam sample1.bam,sample2.bam,sample3.bam --gtf genes.gtf --output-prefix multi_sample --threads 8`
**Explanation:** pbfusion discover subcommand; --bam sample1.bam,sample2.bam,sample3.bam comma-separated BAM inputs; --gtf genes.gtf input GTF; --output-prefix multi_sample output prefix; --threads 8 parallel processing; multi-sample fusion detection; identifies recurrent fusions across samples

### include mitochondrial gene fusions
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix mito --allow-mito --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix mito output prefix; --allow-mito permits fusions involving mitochondrial genes; --threads 8 parallel processing; disabled by default as often false positives

### detect fusions with custom minimum fusion quality
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix medium --min-fusion-quality MEDIUM --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix medium output prefix; --min-fusion-quality MEDIUM (default) balances sensitivity and specificity; --threads 8 parallel processing; LOW for sensitive, HIGH for specific

### analyze fusion output TSV file
**Args:** `awk -F'\t' 'NR>1 && $6>=3 {print $1"-"$2, $6}' fusions.tsv | sort | uniq -c`
**Explanation:** awk command; -F'\t' tab delimiter; 'NR>1 && $6>=3 {print $1"-"$2, $6}' filter by read support column 6; fusions.tsv input file; sort | uniq -c count occurrences; parse fusions.tsv output; useful for downstream analysis

### detect fusions with relaxed identity threshold for noisy data
**Args:** `discover --bam isoseq_aligned.bam --gtf genes.gtf --output-prefix relaxed --min-mean-identity 0.90 --threads 8`
**Explanation:** pbfusion discover subcommand; --bam isoseq_aligned.bam input BAM; --gtf genes.gtf input GTF; --output-prefix relaxed output prefix; --min-mean-identity 0.90 lowers identity threshold; --threads 8 parallel processing; useful for lower-quality IsoSeq data or early chemistry versions
