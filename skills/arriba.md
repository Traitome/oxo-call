---
name: arriba
category: rna-seq
description: Fast and accurate gene fusion detection from RNA-seq data using STAR alignments
tags: [rna-seq, fusion, gene-fusion, cancer, star, transcript, structural-variant]
author: oxo-call built-in
source_url: "https://github.com/suhrig/arriba"
---

## Concepts

- Arriba detects gene fusions from RNA-seq data aligned with STAR; it requires STAR to be run with specific flags.
- STAR must be run with: --chimSegmentMin 10 --chimOutType WithinBAM --chimJunctionOverhangMin 10
- Arriba input: STAR-aligned BAM (-x), genome FASTA (-g), gene annotation GTF (-a).
- Use -b for a blacklist file (reduces false positives from common artifacts).
- Output: <prefix>.tsv (fusions table), <prefix>.discarded.tsv (filtered fusions).
- Use draw_fusions.R (bundled) for visualization of detected fusions.
- Arriba works best with paired-end, unstranded or reverse-strand library RNA-seq.
- Run Arriba after STAR alignment — it reads chimeric alignments embedded in the BAM.

## Pitfalls

- STAR must be run with chimeric alignment flags — regular STAR BAM without these flags produces no fusions.
- Arriba requires the gene annotation GTF to match the genome FASTA used for STAR index.
- Without the blacklist file (-b), many false positives from paralogs and repetitive regions appear.
- STAR genome index for Arriba needs --genomeSAindexNbases 14 for small genomes.
- Arriba works on single BAM — one run per sample.

## Examples

### run STAR with chimeric output for Arriba fusion detection
**Args:** `--runMode alignReads --genomeDir /star_index/ --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --chimSegmentMin 10 --chimOutType WithinBAM --chimJunctionOverhangMin 10 --chimScoreDropMax 30 --peOverlapNbasesMin 12`
**Explanation:** required STAR chimeric parameters for Arriba; these flags enable chimeric read detection

### detect gene fusions with Arriba
**Args:** `-x sample/Aligned.sortedByCoord.out.bam -o fusions.tsv -O discarded_fusions.tsv -g genome.fa -a genes.gtf -b blacklist_hg38_GRCh38_v2.4.0.tsv.gz`
**Explanation:** -x STAR BAM; -g genome FASTA; -a GTF; -b blacklist; -o fusions output; -O discarded fusions

### visualize detected fusions with Arriba draw_fusions
**Args:** `draw_fusions.R --fusions=fusions.tsv --alignments=sample/Aligned.sortedByCoord.out.bam --genome=genome.fa --annotation=genes.gtf --output=fusion_plots.pdf`
**Explanation:** R script for fusion visualization; outputs PDF with structural diagrams for each fusion
