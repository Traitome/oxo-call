---
name: manta
category: variant-calling
description: Structural variant and indel caller for short-read sequencing data
tags: [structural-variant, sv, indel, cnv, deletion, insertion, translocation, vcf]
author: oxo-call built-in
source_url: "https://github.com/Illumina/manta"
---

## Concepts
- Manta uses a two-step workflow: (1) configureManta.py to configure; (2) python runWorkflow.py -m local -j N to execute.
- Manta calls SVs ≥50bp including deletions, duplications, insertions, inversions, and translocations.
- For tumor-normal somatic SV calling, use --tumorBam and --normalBam; for germline use --bam.
- Manta outputs candidateSmallIndels.vcf.gz (used as input to Strelka2) and diploidSV.vcf.gz/somaticSV.vcf.gz.
- The output directory (--runDir) is created by configureManta.py and contains runWorkflow.py.
- Use --exome for WES/targeted sequencing; --callRegions for restricting to specific regions.
- Manta can process RNA-seq data for fusion detection with --rna flag.
- --generateEvidenceBam outputs BAM files with evidence reads supporting each SV call.
- --outputContig outputs assembled contig sequences for breakpoint regions.
- --unstranded is for RNA-seq data without strand information.
- --region specifies specific chromosomal regions to analyze (can be used multiple times).
- --memGb sets memory limit for the workflow; --scanSizeMb controls genome scanning window size.
- Candidate SVs (candidateSV.vcf.gz) include all candidates before filtering; diploidSV.vcf.gz is the final filtered output.

## Pitfalls
- Manta requires a two-step workflow — the configure step only sets up the run directory.
- Input BAMs must be coordinate-sorted and indexed.
- The reference FASTA must be indexed with samtools faidx.
- The --runDir must not already exist — Manta will not overwrite an existing run directory.
- For WES, --exome and --callRegions are both recommended for accurate SV calling.
- --callRegions BED must be bgzipped and tabix-indexed.
- --generateEvidenceBam significantly increases runtime and disk usage; only use when needed.
- Manta requires Python 2.7; ensure python2 is available in PATH before running.
- --region can be specified multiple times but each region increases memory usage.
- RNA-seq mode (--rna) is experimental and may have lower sensitivity than DNA mode.
- Tumor-only mode (--tumorBam without --normalBam) has reduced specificity; results need careful filtering.
- IMPRESE variants lack confident breakpoint assembly; use paired-end evidence for interpretation.

## Examples

### configure and run Manta germline SV calling (configureManta.py)
**Args:** `configureManta.py --bam sorted.bam --referenceFasta reference.fa --runDir manta_output && python manta_output/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --bam sorted.bam input BAM; --referenceFasta reference.fa reference FASTA; --runDir manta_output output directory; python manta_output/runWorkflow.py -m local -j 8 execute workflow with 8 threads

### run Manta somatic SV calling for tumor-normal pair (configureManta.py)
**Args:** `configureManta.py --normalBam normal.bam --tumorBam tumor.bam --referenceFasta reference.fa --runDir manta_somatic && python manta_somatic/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --normalBam normal.bam control BAM; --tumorBam tumor.bam tumor BAM; --referenceFasta reference.fa reference FASTA; --runDir manta_somatic output directory; python manta_somatic/runWorkflow.py -m local -j 8 execute workflow

### run Manta on WES data with capture regions (configureManta.py)
**Args:** `configureManta.py --bam sample.bam --referenceFasta reference.fa --exome --callRegions targets.bed.gz --runDir manta_wes && python manta_wes/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --bam sample.bam input BAM; --referenceFasta reference.fa reference FASTA; --exome WES mode; --callRegions targets.bed.gz target regions; --runDir manta_wes output directory; python manta_wes/runWorkflow.py -m local -j 8 execute

### run Manta for RNA fusion detection (configureManta.py)
**Args:** `configureManta.py --rna --bam rna_sorted.bam --referenceFasta reference.fa --runDir manta_rna && python manta_rna/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --rna RNA fusion mode; --bam rna_sorted.bam input BAM; --referenceFasta reference.fa reference FASTA; --runDir manta_rna output directory; python manta_rna/runWorkflow.py -m local -j 8 execute

### run Manta with evidence BAM generation for IGV visualization
**Args:** `configureManta.py --bam sample.bam --referenceFasta reference.fa --generateEvidenceBam --runDir manta_evidence && python manta_evidence/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --bam sample.bam input BAM; --referenceFasta reference.fa reference FASTA; --generateEvidenceBam output evidence BAMs; --runDir manta_evidence output directory; python manta_evidence/runWorkflow.py -m local -j 8 execute

### run Manta on specific genomic regions only
**Args:** `configureManta.py --bam sample.bam --referenceFasta reference.fa --region chr1:1000000-2000000 --region chr2:5000000-6000000 --runDir manta_regions && python manta_regions/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --bam sample.bam input BAM; --referenceFasta reference.fa reference FASTA; --region chr1:1000000-2000000 --region chr2:5000000-6000000 specific regions; --runDir manta_regions output directory; python manta_regions/runWorkflow.py -m local -j 8 execute

### run Manta with contig output for breakpoint assembly
**Args:** `configureManta.py --bam sample.bam --referenceFasta reference.fa --outputContig --runDir manta_contigs && python manta_contigs/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --bam sample.bam input BAM; --referenceFasta reference.fa reference FASTA; --outputContig output contigs; --runDir manta_contigs output directory; python manta_contigs/runWorkflow.py -m local -j 8 execute

### run Manta tumor-only mode (no matched normal)
**Args:** `configureManta.py --tumorBam tumor.bam --referenceFasta reference.fa --runDir manta_tumor_only && python manta_tumor_only/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --tumorBam tumor.bam tumor BAM; --referenceFasta reference.fa reference FASTA; --runDir manta_tumor_only output directory; python manta_tumor_only/runWorkflow.py -m local -j 8 execute

### run Manta with memory and scanning parameters tuned
**Args:** `configureManta.py --bam sample.bam --referenceFasta reference.fa --runDir manta_tuned && python manta_tuned/runWorkflow.py -m local -j 8 --memGb 32`
**Explanation:** configureManta.py command; --bam sample.bam input BAM; --referenceFasta reference.fa reference FASTA; --runDir manta_tuned output directory; python manta_tuned/runWorkflow.py -m local -j 8 --memGb 32 execute with 32GB memory limit

### run Manta RNA-seq with unstranded library
**Args:** `configureManta.py --rna --unstranded --bam rna.bam --referenceFasta reference.fa --runDir manta_rna_unstranded && python manta_rna_unstranded/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --rna RNA mode; --unstranded unstranded library; --bam rna.bam input BAM; --referenceFasta reference.fa reference FASTA; --runDir manta_rna_unstranded output directory; python manta_rna_unstranded/runWorkflow.py -m local -j 8 execute

### run Manta germline analysis with multiple samples
**Args:** `configureManta.py --bam sample1.bam --bam sample2.bam --bam sample3.bam --referenceFasta reference.fa --runDir manta_multi && python manta_multi/runWorkflow.py -m local -j 8`
**Explanation:** configureManta.py command; --bam sample1.bam --bam sample2.bam --bam sample3.bam multiple input BAMs; --referenceFasta reference.fa reference FASTA; --runDir manta_multi output directory; python manta_multi/runWorkflow.py -m local -j 8 execute
