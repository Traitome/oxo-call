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
- --ploidy sets ploidy for germline calling (default 2); use 1 for haploid regions.
- --outputCallableRegions outputs BED of callable regions; useful for coverage analysis.
- --targeted is for targeted sequencing without --exome optimization.
- --callContinuousVF is for continuous value format output.

## Pitfalls
- Strelka2 requires a two-step workflow — running only the configure step does not call variants.
- Input BAMs must be coordinate-sorted and indexed (.bai files required).
- The reference FASTA must be indexed with samtools faidx.
- For WES data, use --exome and provide --callRegions with the target BED file (bgzipped and tabix-indexed).
- --runDir must not exist before configuration — Strelka2 creates it fresh.
- The --callRegions BED file must be bgzipped (.bed.gz) and indexed with tabix.
- --ploidy 1 is needed for haploid regions (chrX/Y in males, mitochondria).
- --outputCallableRegions adds runtime but provides useful coverage information.
- --rna mode is for RNA-seq; use --rna for RNA-seq somatic variant calling.
- --targeted is for amplicon/targeted sequencing without WES assumptions.

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

### run Strelka2 germline with haploid ploidy for chrX
**Args:** `configureStrelkaGermlineWorkflow.py --bam male_sample.bam --referenceFasta reference.fa --ploidy 1 --callRegions chrX.bed.gz --runDir strelka_chrX && python strelka_chrX/runWorkflow.py -m local -j 8`
**Explanation:** --ploidy 1 for haploid regions (chrX in males, mitochondria)

### run Strelka2 with callable regions output
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --outputCallableRegions --runDir strelka_callable && python strelka_callable/runWorkflow.py -m local -j 8`
**Explanation:** --outputCallableRegions outputs BED of callable regions for coverage analysis

### run Strelka2 somatic for RNA-seq data
**Args:** `configureStrelkaSomaticWorkflow.py --normalBam normal.bam --tumourBam tumor.bam --referenceFasta reference.fa --rna --runDir strelka_rna && python strelka_rna/runWorkflow.py -m local -j 8`
**Explanation:** --rna mode for RNA-seq somatic variant calling

### run Strelka2 for targeted/amplicon sequencing
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --targeted --callRegions amplicons.bed.gz --runDir strelka_amplicon && python strelka_amplicon/runWorkflow.py -m local -j 8`
**Explanation:** --targeted for targeted sequencing without WES assumptions

### run Strelka2 germline with continuous value format
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --callContinuousVF --runDir strelka_continuous && python strelka_continuous/runWorkflow.py -m local -j 8`
**Explanation:** --callContinuousVF outputs continuous value format for downstream analysis
