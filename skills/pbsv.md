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
- pbsv call --ccs is required for CCS/HiFi reads; omit for CLR/subreads.
- --min-sv-length sets minimum SV size (default 20bp); increase to filter small variants.
- --max-sv-length sets maximum SV size; default 100kb for deletions, 10kb for insertions.
- --region allows per-chromosome processing for parallelization.
- .svsig.gz can be indexed with tabix for random access during pbsv call -r.

## Pitfalls
- PBSV requires sorted, indexed, haplotagged BAM files — run pbmm2 or minimap2 and samtools sort/index first.
- The .svsig.gz file from pbsv discover is required for pbsv call — cannot skip the discover step.
- PBSV is designed for PacBio reads; for ONT SVs, use Sniffles2 or cuteSV instead.
- Without --tandem-repeats annotation, SV breakpoints in repeat regions may be imprecise.
- pbsv call requires all samples' .svsig.gz files at once for joint genotyping across a cohort.
- pbsv call --ccs is required for HiFi reads; omitting causes incorrect SV calls.
- --min-sv-length 20 (default) includes small indels; increase to 50+ for true SVs only.
- --max-sv-length limits insertion size (default 15kb); increase for large insertions.
- Deletions >100kb are called as translocations; adjust expectations for large deletions.
- Index .svsig.gz with tabix for efficient random access with pbsv call -r.

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

### call SVs from HiFi reads with --ccs flag
**Args:** `call --ccs --hifi reference.fa sample.svsig.gz output.vcf`
**Explanation:** --ccs is required for HiFi/CCS reads; omit for CLR/subreads

### discover SVs per chromosome for parallel processing
**Args:** `discover --hifi --region chr1 sorted.bam sample.chr1.svsig.gz`
**Explanation:** --region processes single chromosome; run in parallel for each chr

### index svsig.gz for random access
**Args:** `tabix -c '#' -s 3 -b 4 -e 4 sample.svsig.gz`
**Explanation:** index .svsig.gz for efficient random access with pbsv call -r

### call SVs with indexed svsig and region
**Args:** `call --ccs --hifi -r chr1:1000000-2000000 reference.fa sample.svsig.gz output.vcf`
**Explanation:** -r specifies region; uses indexed .svsig.gz for efficient access

### filter small variants with minimum SV length
**Args:** `call --ccs --hifi --min-sv-length 50 reference.fa sample.svsig.gz output.vcf`
**Explanation:** --min-sv-length 50 filters variants <50bp; for true SVs only

### increase maximum insertion size for large insertions
**Args:** `call --ccs --hifi --max-sv-length 50k reference.fa sample.svsig.gz output.vcf`
**Explanation:** --max-sv-length 50k increases insertion limit from 15kb to 50kb

### call SVs with multiple threads
**Args:** `call --ccs --hifi -j 8 reference.fa sample.svsig.gz output.vcf`
**Explanation:** -j 8 uses 8 threads for faster genotyping
