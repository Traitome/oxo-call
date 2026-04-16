---
name: survivor
category: variant-calling
description: SV simulation, merging, and comparison of structural variant calls across multiple callers and samples
tags: [sv, structural-variants, merging, vcf, long-read, benchmarking, consensus]
author: oxo-call built-in
source_url: "https://github.com/fritzsedlazeck/SURVIVOR/wiki"
---

## Concepts

- SURVIVOR merge combines SV calls from multiple VCF files using configurable distance, type agreement, and caller support thresholds.
- The merge command takes a plain text file listing VCF paths (one per line), not the VCFs directly on the command line.
- Merging parameters: max_distance (bp), min_callers_support, same_type_required (1/0), same_strand_required (1/0), estimate_SV_distance (1/0), min_sv_size.
- SURVIVOR stats provides summary statistics on SV sizes, types, and genotype distributions in a VCF.
- SURVIVOR filter removes SVs by size, type, allele frequency, or genotype quality to create a high-confidence call set.
- SURVIVOR simSV simulates structural variants on a reference genome for benchmarking purposes.
- SURVIVOR eval compares SV calls against a truth set for benchmarking accuracy.
- SURVIVOR scanreads extracts error profiles from aligned reads for realistic simulation.
- SURVIVOR simreads simulates long reads (PacBio/ONT) with realistic error profiles.

## Pitfalls

- The VCF list file for SURVIVOR merge must contain full or relative paths and have no trailing spaces or blank lines.
- All input VCFs must be uncompressed (not bgzipped); bgzipped VCFs must be decompressed with bgzip -d first.
- Setting same_type_required=0 allows merging of different SV types (DEL with INS), which is almost always incorrect — use 1.
- SURVIVOR merge distance (parameter 1) is the maximum breakpoint distance in bp; use 500-1000 for ONT, 50-100 for short-read callers.
- SURVIVOR does not sort output VCF; sort with bcftools sort before passing to downstream tools.
- min_callers_support=1 includes SVs from any single caller; for high-confidence consensus use 2 or more callers.

## Examples

### merge SV VCFs from multiple callers requiring support from at least 2 callers
**Args:** `merge vcf_list.txt 500 2 1 1 0 50 merged_svs.vcf`
**Explanation:** args: vcf_list distance=500bp min_callers=2 same_type=1 same_strand=1 estimate_dist=0 min_size=50; output is merged_svs.vcf

### merge SV calls from a single caller across multiple samples
**Args:** `merge sample_vcfs.txt 1000 1 1 1 0 50 cohort_svs.vcf`
**Explanation:** distance=1000 for loose merging across samples; min_callers=1 includes private SVs; same_type=1 preserves type

### get summary statistics for SVs in a VCF
**Args:** `stats -i calls.vcf -o sv_stats.txt`
**Explanation:** outputs counts per SV type, size distributions, and genotype quality summaries

### filter SVs to a high-confidence set by size and minimum quality
**Args:** `filter -i calls.vcf -o filtered.vcf -s 50 -e 100000 -f 0`
**Explanation:** -s 50 minimum SV size; -e 100000 maximum SV size; -f 0 minimum allele frequency (0 = no AF filter)

### simulate structural variants on a reference genome for benchmarking
**Args:** `simSV reference.fasta parameter_file.txt 0 0 simulated`
**Explanation:** generates simulated SVs from a parameter file; outputs simulated_insertions.fa, simulated_SVs.vcf, and a modified FASTA

### create a VCF list file and merge three caller outputs
**Args:** `ls sniffles.vcf pbsv.vcf cutesv.vcf > vcf_list.txt && merge vcf_list.txt 500 2 1 1 0 50 consensus_svs.vcf`
**Explanation:** creates vcf_list.txt with three caller VCF paths; merge requires at least 2 callers to agree (min_callers=2)

### convert SURVIVOR merged VCF to sorted VCF
**Args:** `bcftools sort merged_svs.vcf -Oz -o merged_svs.sorted.vcf.gz && bcftools index merged_svs.sorted.vcf.gz`
**Explanation:** SURVIVOR output is not sorted; use bcftools sort before downstream analysis

### filter SVs by type (only deletions)
**Args:** `filter -i calls.vcf -o deletions_only.vcf -s 50 -e 100000 -t DEL`
**Explanation:** -t DEL filters to deletions only; combine with size filters for specific DEL size ranges

### generate parameter file for SV simulation
**Args:** `simSV parameter_file.txt reference.fasta`
**Explanation:** creates parameter file template for SV simulation; edit parameters before running full simulation

### evaluate SV calls against simulated truth set
**Args:** `eval truth.vcf calls.vcf 500 0.5 0.5 output.txt`
**Explanation:** evaluates call accuracy against simulated truth; 500bp distance, 0.5 size/seq similarity thresholds

### scan reads for error profiles prior to simulation
**Args:** `scanreads aligned.bam error_profile.txt`
**Explanation:** analyzes aligned reads to generate error profiles for realistic read simulation

### simulate long reads with error profiles
**Args:** `simreads reference.fasta error_profile.txt 10000 10 reads.fasta`
**Explanation:** simulates 10000 reads with 10x coverage using error profile; outputs FASTA of simulated reads
