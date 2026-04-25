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
**Explanation:** configureStrelkaGermlineWorkflow.py companion script; --bam sorted.bam input BAM; --referenceFasta reference.fa reference genome; --runDir strelka_germline output directory; && python strelka_germline/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads; two-step workflow

### configure and run Strelka2 somatic variant calling (configureStrelkaSomaticWorkflow.py)
**Args:** `configureStrelkaSomaticWorkflow.py --normalBam normal.bam --tumourBam tumor.bam --referenceFasta reference.fa --runDir strelka_somatic && python strelka_somatic/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaSomaticWorkflow.py companion script; --normalBam normal.bam normal sample BAM; --tumourBam tumor.bam tumor sample BAM; --referenceFasta reference.fa reference genome; --runDir strelka_somatic output directory; && python strelka_somatic/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads

### run Strelka2 germline on WES data with target regions (configureStrelkaGermlineWorkflow.py)
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --exome --callRegions targets.bed.gz --runDir strelka_wes && python strelka_wes/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaGermlineWorkflow.py companion script; --bam sorted.bam input BAM; --referenceFasta reference.fa reference genome; --exome adjusts for WES/targeted sequencing; --callRegions targets.bed.gz bgzipped target BED; --runDir strelka_wes output directory; && python strelka_wes/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads

### run Strelka2 somatic with Manta indel candidates (configureStrelkaSomaticWorkflow.py)
**Args:** `configureStrelkaSomaticWorkflow.py --normalBam normal.bam --tumourBam tumor.bam --referenceFasta reference.fa --indelCandidates manta_results/results/variants/candidateSmallIndels.vcf.gz --runDir strelka_with_manta && python strelka_with_manta/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaSomaticWorkflow.py companion script; --normalBam normal.bam normal sample BAM; --tumourBam tumor.bam tumor sample BAM; --referenceFasta reference.fa reference genome; --indelCandidates manta_results/results/variants/candidateSmallIndels.vcf.gz Manta indel candidates; --runDir strelka_with_manta output directory; && python strelka_with_manta/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads; Manta must be run first

### run Strelka2 germline with haploid ploidy for chrX
**Args:** `configureStrelkaGermlineWorkflow.py --bam male_sample.bam --referenceFasta reference.fa --ploidy 1 --callRegions chrX.bed.gz --runDir strelka_chrX && python strelka_chrX/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaGermlineWorkflow.py companion script; --bam male_sample.bam input BAM; --referenceFasta reference.fa reference genome; --ploidy 1 haploid setting; --callRegions chrX.bed.gz bgzipped BED; --runDir strelka_chrX output directory; && python strelka_chrX/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads; for haploid regions (chrX in males, mitochondria)

### run Strelka2 with callable regions output
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --outputCallableRegions --runDir strelka_callable && python strelka_callable/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaGermlineWorkflow.py companion script; --bam sorted.bam input BAM; --referenceFasta reference.fa reference genome; --outputCallableRegions outputs BED of callable regions; --runDir strelka_callable output directory; && python strelka_callable/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads; for coverage analysis

### run Strelka2 somatic for RNA-seq data
**Args:** `configureStrelkaSomaticWorkflow.py --normalBam normal.bam --tumourBam tumor.bam --referenceFasta reference.fa --rna --runDir strelka_rna && python strelka_rna/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaSomaticWorkflow.py companion script; --normalBam normal.bam normal sample BAM; --tumourBam tumor.bam tumor sample BAM; --referenceFasta reference.fa reference genome; --rna RNA-seq somatic mode; --runDir strelka_rna output directory; && python strelka_rna/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads

### run Strelka2 for targeted/amplicon sequencing
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --targeted --callRegions amplicons.bed.gz --runDir strelka_amplicon && python strelka_amplicon/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaGermlineWorkflow.py companion script; --bam sorted.bam input BAM; --referenceFasta reference.fa reference genome; --targeted amplicon/targeted sequencing mode; --callRegions amplicons.bed.gz bgzipped target BED; --runDir strelka_amplicon output directory; && python strelka_amplicon/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads; without WES assumptions

### run Strelka2 germline with continuous value format
**Args:** `configureStrelkaGermlineWorkflow.py --bam sorted.bam --referenceFasta reference.fa --callContinuousVF --runDir strelka_continuous && python strelka_continuous/runWorkflow.py -m local -j 8`
**Explanation:** configureStrelkaGermlineWorkflow.py companion script; --bam sorted.bam input BAM; --referenceFasta reference.fa reference genome; --callContinuousVF outputs continuous value format; --runDir strelka_continuous output directory; && python strelka_continuous/runWorkflow.py execute workflow; -m local local machine mode; -j 8 threads; for downstream analysis
