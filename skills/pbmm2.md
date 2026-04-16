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
- Use --preset to specify read type: HIFI (CCS/HiFi), SUBREAD (CLR), ISOSEQ (IsoSeq transcriptome), UNROLLED.
- pbmm2 align: aligns reads to reference; outputs sorted BAM with PacBio-compatible tags.
- pbmm2 index: creates a reference index (optional but recommended for repeated use).
- Use --sort for sorted output; --preset HIFI for CCS/HiFi reads (most common).
- pbmm2 output BAM is compatible with PBSV, DeepVariant PACBIO model, and other PacBio tools.
- Use -j N for alignment threads; --sort-threads for sort threads.
- --secondary outputs secondary alignments; --max-secondary-alns limits number retained.
- --min-gap-comp-id-perc sets minimum gap-compressed identity (default 70%); increase for stricter mapping.
- --strip removes kinetic and QV tags; reduces file size but prevents downstream polishing.
- --bam-index generates BAI or CSI index for sorted output (default BAI).
- --rg adds read group information; essential for multi-sample variant calling.

## Pitfalls
- pbmm2 is designed for PacBio data — for ONT use minimap2 directly.
- --preset HIFI is for CCS reads; --preset SUBREAD for raw CLR reads — using wrong preset reduces mapping rate.
- pbmm2 natively handles PacBio BAM input without conversion to FASTQ.
- Without --sort, pbmm2 outputs unsorted BAM — use --sort for indexed BAM.
- For downstream PBSV, pbmm2 with --sort --preset HIFI is the standard alignment step.
- --strip removes tags needed for ccs polishing; only use if you don't need downstream polishing.
- --secondary increases output size; use --max-secondary-alns to limit secondary alignments.
- --preset ISOSEQ uses splice-aware settings; do not use for genomic DNA alignment.
- --min-gap-comp-id-perc 70 (default) is permissive; increase to 80-90 for unique mappings only.
- --bam-index CSI is needed for references with chromosomes >512 Mbp; default BAI has size limits.

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

### align with read group information for variant calling
**Args:** `align --preset HIFI --sort -j 16 --rg '@RG\tID:sample1\tSM:sample1\tPL:PACBIO' reference.fa reads.bam aligned.bam`
**Explanation:** --rg adds read group; essential for GATK and other tools requiring RG headers

### output secondary alignments for structural variant detection
**Args:** `align --preset HIFI --sort -j 16 --secondary --max-secondary-alns 10 reference.fa reads.bam aligned.bam`
**Explanation:** --secondary outputs secondary alignments; --max-secondary-alns 10 limits to top 10

### increase mapping stringency for unique mappings
**Args:** `align --preset HIFI --sort -j 16 --min-gap-comp-id-perc 85 reference.fa reads.bam aligned_strict.bam`
**Explanation:** --min-gap-comp-id-perc 85 requires 85% identity; filters out lower quality mappings

### strip kinetic tags to reduce file size
**Args:** `align --preset HIFI --sort -j 16 --strip reference.fa reads.bam aligned_stripped.bam`
**Explanation:** --strip removes kinetic/QV tags; smaller files but cannot be polished later

### generate CSI index for large references
**Args:** `align --preset HIFI --sort -j 16 --bam-index CSI reference.fa reads.bam aligned.bam`
**Explanation:** --bam-index CSI creates CSI index; required for references with chromosomes >512 Mbp

### align FASTQ input with sample name
**Args:** `align --preset HIFI --sort -j 16 --sample MySample reference.fa reads.fastq aligned.bam`
**Explanation:** --sample adds sample name to read groups; for FASTQ input without existing RG info

### use pre-built index for faster alignment
**Args:** `align -d reference.mmi --preset HIFI --sort -j 16 reads.bam aligned.bam`
**Explanation:** -d reference.mmi uses pre-built index; faster for repeated alignments to same reference
