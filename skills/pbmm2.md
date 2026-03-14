---
name: pbmm2
category: alignment
description: "PacBio's minimap2 wrapper for aligning PacBio reads with native support for SMRT data formats"
tags: [alignment, pacbio, hifi, clr, long-read, bam, smrt, mapping]
author: oxo-call built-in
source_url: "https://github.com/PacificBiosciences/pbmm2"
---

## Concepts

- pbmm2 is a minimap2 wrapper optimized for PacBio reads; handles PacBio BAM, FASTQ, and FASTA input.
- Use --preset to specify read type: HIFI (CCS/HiFi), SUBREAD (CLR), ISOSEQ (IsoSeq transcriptome).
- pbmm2 align: aligns reads to reference; outputs sorted BAM with PacBio-compatible tags.
- pbmm2 index: creates a reference index (optional but recommended for repeated use).
- Use --sort for sorted output; --preset HIFI for CCS/HiFi reads (most common).
- pbmm2 output BAM is compatible with PBSV, DeepVariant PACBIO model, and other PacBio tools.
- Use -j N for alignment threads; --sort-threads for sort threads.

## Pitfalls

- pbmm2 is designed for PacBio data — for ONT use minimap2 directly.
- --preset HIFI is for CCS reads; --preset SUBREAD for raw CLR reads — using wrong preset reduces mapping rate.
- pbmm2 natively handles PacBio BAM input without conversion to FASTQ.
- Without --sort, pbmm2 outputs unsorted BAM — use --sort for indexed BAM.
- For downstream PBSV, pbmm2 with --sort --preset HIFI is the standard alignment step.

## Examples

### align PacBio HiFi reads to reference genome
**Args:** `align --preset HIFI --sort -j 16 --sort-threads 4 reference.fa hifi_reads.bam aligned_sorted.bam`
**Explanation:** --preset HIFI; --sort for sorted output; -j 16 alignment threads; input PacBio BAM; output sorted BAM

### align PacBio IsoSeq transcriptome reads
**Args:** `align --preset ISOSEQ --sort -j 8 reference.fa isoseq_reads.bam isoseq_aligned.bam`
**Explanation:** --preset ISOSEQ for transcript alignment with splice-aware settings

### index reference genome for repeated pbmm2 use
**Args:** `index reference.fa reference.mmi`
**Explanation:** creates minimap2 index for faster repeated alignment; pass .mmi to align -d option

### align CLR (subread) PacBio reads
**Args:** `align --preset SUBREAD --sort -j 16 reference.fa subreads.bam clr_aligned.bam`
**Explanation:** --preset SUBREAD for raw CLR reads; longer, noisier reads require different alignment parameters
