---
name: strelka2
category: variant-calling
description: Fast and accurate small variant caller for germline and somatic SNVs and indels
tags: [variant-calling, somatic, germline, snv, indel, vcf, tumor-normal, illumina]
author: oxo-call built-in
source_url: "https://github.com/Illumina/strelka"
---

## Concepts

- Strelka2 has two modes: germline (configureStrelkaGermlineWorkflow.py) and somatic (configureStrelkaSomaticWorkflow.py).
- Strelka2 uses a two-step workflow: (1) configure the run (creates a runWorkflow.py); (2) execute: python runWorkflow.py -m local -j N.
- Somatic mode requires tumor BAM and normal BAM as separate arguments (--tumourBam and --normalBam).
- Strelka2 outputs to a directory (--runDir); results are in results/variants/ containing VCF files.
- For somatic mode, results include somatic.snvs.vcf.gz and somatic.indels.vcf.gz.
- For germline mode, results include variants.vcf.gz with genotyped variants.
- Use --exome flag for targeted sequencing/WES data; --rna for RNA-seq somatic variant calling.
- Strelka2 requires Manta SV results for improved indel calling — run Manta first and use --indelCandidates.

## Pitfalls

- Strelka2 requires a two-step workflow — running only the configure step does not call variants.
- Input BAMs must be coordinate-sorted and indexed (.bai files required).
- The reference FASTA must be indexed with samtools faidx.
- For WES data, use --exome and provide --callRegions with the target BED file (bgzipped and tabix-indexed).
- --runDir must not exist before configuration — Strelka2 creates it fresh.
- The --callRegions BED file must be bgzipped (.bed.gz) and indexed with tabix.

## Examples

### configure and run Strelka2 germline variant calling (configureStrelkaGermlineWorkflow.py)
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --runDir strelka_germline && python strelka_germline/runWorkflow.py -m local -j 8`
**Explanation:** configure creates runDir; then execute with -m local for local machine execution and -j 8 threads

### configure and run Strelka2 somatic variant calling (configureStrelkaSomaticWorkflow.py)
**Args:** `configureStrelkaSomaticWorkflow.py --normalBam normal.bam --tumourBam tumor.bam --referenceFasta reference.fa --runDir strelka_somatic && python strelka_somatic/runWorkflow.py -m local -j 8`
**Explanation:** --normalBam and --tumourBam specify the matched normal and tumor BAM files

### run Strelka2 germline on WES data with target regions (configureStrelkaGermlineWorkflow.py)
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --exome --callRegions targets.bed.gz --runDir strelka_wes && python strelka_wes/runWorkflow.py -m local -j 8`
**Explanation:** --exome adjusts for targeted sequencing; --callRegions restricts calling to target BED regions (bgzipped + tabix)

### run Strelka2 somatic with Manta indel candidates (configureStrelkaSomaticWorkflow.py)
**Args:** `configureStrelkaSomaticWorkflow.py --normalBam normal.bam --tumourBam tumor.bam --referenceFasta reference.fa --indelCandidates manta_results/results/variants/candidateSmallIndels.vcf.gz --runDir strelka_with_manta && python strelka_with_manta/runWorkflow.py -m local -j 8`
**Explanation:** --indelCandidates from Manta improves indel calling accuracy; Manta must be run first
