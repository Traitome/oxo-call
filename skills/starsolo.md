---
name: starsolo
category: single-cell
description: STARsolo — single-cell RNA-seq processing module within STAR aligner, a Cellranger-compatible alternative
tags: [single-cell, scrna-seq, alignment, barcode, umi, star, 10x-genomics]
author: oxo-call built-in
source_url: "https://github.com/alexdobin/STAR/blob/master/docs/STARsolo.md"
---

## Concepts
- STARsolo is built into STAR v2.7+; it processes scRNA-seq with barcode/UMI demultiplexing during alignment.
- Use --soloType to specify library type: CB_UMI_Simple (10x v1/v2/v3), CB_UMI_Complex (Drop-seq), or SmartSeq.
- Key parameters: --soloCBwhitelist (barcode whitelist), --soloCBstart/End, --soloUMIstart/End for position specification.
- 10x Chromium v3: --soloCBlen 16 --soloUMIlen 12; v2: --soloCBlen 16 --soloUMIlen 10.
- Output: Solo.out/Gene/filtered/ (filtered matrix), Solo.out/Gene/raw/ (all cells), similar to Cell Ranger output.
- Use --soloFeatures Gene (gene counts), Velocyto (for RNA velocity), GeneFull (for pre-mRNA counts).
- STARsolo is ~5x faster than Cell Ranger and produces comparable results.
- Use --genomeDir with a STAR genome index (same as regular STAR alignment).
- --soloCellFilter controls cell filtering: CellRanger2.2, EmptyDrops_CR, TopCells, or None.
- --soloUMIdedup controls UMI deduplication method: 1MM_All, 1MM_Directional, Exact.
- --soloFeatures GeneFull includes intronic reads; useful for single-nucleus RNA-seq.
- --soloBarcodeReadLength 0 allows variable barcode read lengths.

## Pitfalls
- STARsolo requires the correct barcode whitelist file matching the 10x Chromium version used.
- --soloType must match the library preparation protocol — using wrong type gives incorrect demultiplexing.
- For 10x v3, R1 contains barcode+UMI (28 bp), R2 contains cDNA — pass R1 as --readFilesIn barcode_R1 cDNA_R2 order.
- STAR genome index must include GTF (--sjdbGTFfile) — essential for gene-level counting.
- Without --outSAMtype BAM SortedByCoordinate, output is unsorted SAM — specify for downstream tools.
- STARsolo output matrices use Ensembl IDs by default — check if your downstream tools expect gene symbols.
- --soloCellFilter CellRanger2.2 is default; use EmptyDrops_CR for Cell Ranger 3.x compatibility.
- --soloUMIdedup 1MM_Directional is default; 1MM_All is more permissive, Exact is strictest.
- --soloFeatures GeneFull is needed for single-nucleus RNA-seq (includes intronic reads).
- --soloBarcodeReadLength 0 is needed when barcode and UMI are in separate reads.

## Examples

### process 10x Chromium v3 scRNA-seq with STARsolo
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBstart 1 --soloCBlen 16 --soloUMIstart 17 --soloUMIlen 12 --genomeDir /path/to/star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outSAMattributes NH HI nM AS CR UR CB UB GX GN sS sQ sM --runThreadN 16 --outFileNamePrefix sample_starsolo/`
**Explanation:** R2 (cDNA) before R1 (barcode); --soloCBwhitelist barcode whitelist; 16bp CB + 12bp UMI for 10x v3

### process 10x Chromium v2 scRNA-seq with STARsolo
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 737K-august-2016.txt --soloCBlen 16 --soloUMIlen 10 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix sample_v2/`
**Explanation:** v2 whitelist and 10bp UMI; same R2-before-R1 ordering

### run STARsolo with RNA velocity output
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloFeatures Gene Velocyto --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix velocity_sample/`
**Explanation:** --soloFeatures Velocyto adds spliced/unspliced/ambiguous count matrices for RNA velocity analysis

### run STARsolo for single-nucleus RNA-seq
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloFeatures GeneFull --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix snRNA_sample/`
**Explanation:** --soloFeatures GeneFull includes intronic reads; essential for single-nucleus RNA-seq

### use EmptyDrops cell filtering (Cell Ranger 3.x compatible)
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloCellFilter EmptyDrops_CR --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix emptydrops_sample/`
**Explanation:** --soloCellFilter EmptyDrops_CR uses EmptyDrops algorithm; compatible with Cell Ranger 3.x

### strict UMI deduplication (Exact match only)
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloUMIdedup Exact --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix exact_umi_sample/`
**Explanation:** --soloUMIdedup Exact requires exact UMI matches; strictest deduplication

### output top N cells by UMI count
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloCellFilter TopCells 5000 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix top5k_sample/`
**Explanation:** --soloCellFilter TopCells 5000 outputs only top 5000 cells by UMI count

### no cell filtering (output all barcodes)
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloCellFilter None --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix nofilter_sample/`
**Explanation:** --soloCellFilter None disables cell filtering; outputs all barcodes in whitelist

### allow variable barcode read length
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloBarcodeReadLength 0 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix varlen_sample/`
**Explanation:** --soloBarcodeReadLength 0 allows variable barcode read lengths; for non-standard protocols
