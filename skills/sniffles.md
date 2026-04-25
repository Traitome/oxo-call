---
name: sniffles
category: variant-calling
description: Structural variant caller for long-read sequencing data from Oxford Nanopore and PacBio platforms
tags: [structural-variant, sv, long-read, nanopore, pacbio, deletion, insertion, vcf]
author: oxo-call built-in
source_url: "https://github.com/fritzsedlazeck/Sniffles"
---

## Concepts
- Sniffles2 (version 2) detects SVs from long-read (ONT/PacBio) aligned BAM files; requires sorted, indexed BAM with MD tags.
- Use minimap2 -ax map-ont or map-pb to align reads first, then sort and index with samtools.
- Sniffles2 outputs VCF by default; use --vcf to specify output filename.
- Use --snf to output a Sniffles-specific format for multi-sample population SV calling.
- Population SV calling: run Sniffles2 --snf per sample, then combine with sniffles2 --input *.snf.
- Use --minsupport N to set minimum read support (default: auto); --minsvlen to set minimum SV length.
- Sniffles2 handles mosaic SVs with --mosaic flag for detecting low-frequency SVs.
- --reference is required to output deletion sequences in VCF; provide reference FASTA.
- --tandem-repeats improves calling in repetitive regions; provide BED file with repeat annotations.
- --phase determines phase for SV calls; requires phased input alignments.
- --genotype-vcf performs force calling on known SVs; genotypes input SV set in new sample.
- --combine-pctseq controls merging distance for multi-sample calling (default 0.7).

## Pitfalls
- Sniffles2 requires MD tags in BAM — add --MD to minimap2 command: minimap2 --MD -ax map-ont ref.fa reads.fa | samtools sort.
- Input BAM must be coordinate-sorted and indexed with samtools index.
- Sniffles1 (older) has different command syntax than Sniffles2 — check version with sniffles --version.
- Without --snf, multi-sample calling requires re-running, which is less efficient than SNF-based approach.
- Very high --minsupport may miss rare somatic or mosaic events; very low values increase false positives.
- Sniffles is designed for long reads (>1kb) — it performs poorly on short Illumina reads.
- --reference is needed for deletion sequences; without it, DEL entries lack ALT sequences.
- --tandem-repeats improves accuracy in repeats; download annotations for your reference genome.
- --phase requires phased input BAM (e.g., from WhatsHap or HapCUT2); unphased BAM gives no phase info.
- --mosaic mode is more sensitive but increases false positives; validate mosaic calls carefully.
- --combine-pctseq 0.7 (default) may merge distinct SVs; decrease for stricter merging.

## Examples

### call SVs from a single Oxford Nanopore BAM file
**Args:** `--input sorted.bam --vcf output_svs.vcf --threads 8`
**Explanation:** sniffles command; --input sorted.bam sorted indexed BAM input; --vcf output_svs.vcf output VCF file; --threads 8 parallel processing

### call SVs with minimum read support of 5 and minimum SV length of 50 bp
**Args:** `--input sorted.bam --vcf output_svs.vcf --minsupport 5 --minsvlen 50 --threads 8`
**Explanation:** sniffles command; --input sorted.bam input BAM; --vcf output_svs.vcf output VCF; --minsupport 5 requires ≥5 reads; --minsvlen 50 minimum SV size to report; --threads 8 parallel processing

### generate SNF file for multi-sample population SV calling
**Args:** `--input sample1.bam --snf sample1.snf --vcf sample1.vcf --threads 8`
**Explanation:** sniffles command; --input sample1.bam input BAM; --snf sample1.snf Sniffles Network File output; --vcf sample1.vcf output VCF; --threads 8 parallel processing; creates SNF for population-level merging

### combine multiple SNF files for population-level SV calling
**Args:** `--input sample1.snf sample2.snf sample3.snf --vcf population_svs.vcf --threads 8`
**Explanation:** sniffles command; --input sample1.snf sample2.snf sample3.snf input SNF files; --vcf population_svs.vcf output VCF; --threads 8 parallel processing; pass all .snf files to merge SVs across the cohort

### call mosaic or somatic SVs with low frequency support
**Args:** `--input tumor.bam --vcf mosaic_svs.vcf --mosaic --threads 8`
**Explanation:** sniffles command; --input tumor.bam input BAM; --vcf mosaic_svs.vcf output VCF; --mosaic enables detection of low-frequency SVs (somatic mutations, mosaicism); --threads 8 parallel processing

### call SVs with reference for deletion sequences
**Args:** `--input sorted.bam --vcf output.vcf --reference genome.fa --threads 8`
**Explanation:** sniffles command; --input sorted.bam input BAM; --vcf output.vcf output VCF; --reference genome.fa reference FASTA; --threads 8 parallel processing; enables output of deletion sequences in VCF

### call SVs with tandem repeat annotations
**Args:** `--input sorted.bam --vcf output.vcf --tandem-repeats tandem_repeats.bed --threads 8`
**Explanation:** sniffles command; --input sorted.bam input BAM; --vcf output.vcf output VCF; --tandem-repeats tandem_repeats.bed BED annotation; --threads 8 parallel processing; improves SV calling in repetitive regions

### call phased SVs from phased alignments
**Args:** `--input phased.bam --vcf phased_svs.vcf --phase --threads 8`
**Explanation:** sniffles command; --input phased.bam phased input BAM; --vcf phased_svs.vcf output VCF; --phase determines phase for SV calls; --threads 8 parallel processing; requires phased input BAM with PS tag

### genotype known SVs in new sample (force calling)
**Args:** `--input sample.bam --genotype-vcf known_svs.vcf --vcf genotyped.vcf --threads 8`
**Explanation:** sniffles command; --input sample.bam input BAM; --genotype-vcf known_svs.vcf input known SVs; --vcf genotyped.vcf output VCF; --threads 8 parallel processing; genotypes input SV set for comparing SVs across samples

### combine SNF files using TSV list
**Args:** `--input snf_list.tsv --vcf population.vcf --threads 8`
**Explanation:** sniffles command; --input snf_list.tsv TSV list of SNF paths; --vcf population.vcf output VCF; --threads 8 parallel processing; snf_list.tsv contains SNF paths one per line with optional sample ID in second column

### stricter multi-sample SV merging
**Args:** `--input sample1.snf sample2.snf --vcf population.vcf --combine-pctseq 0.5 --threads 8`
**Explanation:** sniffles command; --input sample1.snf sample2.snf input SNF files; --vcf population.vcf output VCF; --combine-pctseq 0.5 requires 50% sequence similarity for merging; --threads 8 parallel processing; stricter than default 0.7

### output read names in VCF
**Args:** `--input sorted.bam --vcf output.vcf --output-rnames --threads 8`
**Explanation:** sniffles command; --input sorted.bam input BAM; --vcf output.vcf output VCF; --output-rnames includes supporting read names in VCF; --threads 8 parallel processing; useful for validation
