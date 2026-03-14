---
name: pbsv
category: variant-calling
description: PacBio structural variant caller for HiFi and CLR reads
tags: [structural-variant, sv, pacbio, hifi, long-read, vcf, deletion, insertion]
author: oxo-call built-in
source_url: "https://github.com/PacificBiosciences/pbsv"
---

## Concepts

- PBSV calls SVs from PacBio HiFi or CLR reads using a two-step approach: discover + call.
- Step 1: pbsv discover — scans aligned BAM for SV signatures, outputs .svsig.gz file.
- Step 2: pbsv call — genotypes SV candidates from .svsig.gz, outputs VCF.
- Input BAM must be aligned with pbmm2 (PacBio aligner) or minimap2 with --secondary=no.
- Use --hifi preset for HiFi reads; pbsv defaults work well for most PacBio HiFi data.
- Multiple samples can be called jointly: run pbsv discover per sample, then pbsv call with all .svsig.gz files.
- Use --tandem-repeats for a repeat annotation BED file to improve SV breakpoint accuracy in STRs.

## Pitfalls

- PBSV requires sorted, indexed, haplotagged BAM files — run pbmm2 or minimap2 and samtools sort/index first.
- The .svsig.gz file from pbsv discover is required for pbsv call — cannot skip the discover step.
- PBSV is designed for PacBio reads; for ONT SVs, use Sniffles2 or cuteSV instead.
- Without --tandem-repeats annotation, SV breakpoints in repeat regions may be imprecise.
- pbsv call requires all samples' .svsig.gz files at once for joint genotyping across a cohort.

## Examples

### discover SV signatures from PacBio HiFi aligned BAM
**Args:** `discover --hifi sorted.bam sample.svsig.gz`
**Explanation:** --hifi preset for CCS/HiFi data; output .svsig.gz contains SV candidate signatures

### call SVs from a single sample's signature file
**Args:** `call --hifi reference.fa sample.svsig.gz output_svs.vcf`
**Explanation:** call genotypes SVs; reference.fa required; output VCF with SV calls

### call SVs jointly from multiple samples
**Args:** `call --hifi reference.fa sample1.svsig.gz sample2.svsig.gz sample3.svsig.gz cohort_svs.vcf`
**Explanation:** provide all .svsig.gz files for joint genotyping across cohort

### discover with tandem repeat annotation for better accuracy
**Args:** `discover --hifi --tandem-repeats hg38.trf.bed sorted.bam sample.svsig.gz`
**Explanation:** --tandem-repeats BED file improves breakpoint precision in STR/VNTR regions
