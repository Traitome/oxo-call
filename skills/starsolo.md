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
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt barcode whitelist; --soloCBstart 1 --soloCBlen 16 barcode position; --soloUMIstart 17 --soloUMIlen 12 UMI position for v3; --genomeDir /path/to/star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz cDNA before barcode; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --outSAMattributes NH HI nM AS CR UR CB UB GX GN sS sQ sM attributes; --runThreadN 16 threads; --outFileNamePrefix sample_starsolo/ output prefix

### process 10x Chromium v2 scRNA-seq with STARsolo
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 737K-august-2016.txt --soloCBlen 16 --soloUMIlen 10 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix sample_v2/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 737K-august-2016.txt v2 whitelist; --soloCBlen 16 --soloUMIlen 10 v2 UMI length; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz cDNA before barcode; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix sample_v2/ output prefix

### run STARsolo with RNA velocity output
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloFeatures Gene Velocyto --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix velocity_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloFeatures Gene Velocyto adds spliced/unspliced/ambiguous count matrices; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix velocity_sample/ output prefix; for RNA velocity analysis

### run STARsolo for single-nucleus RNA-seq
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloFeatures GeneFull --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix snRNA_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloFeatures GeneFull includes intronic reads; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix snRNA_sample/ output prefix; essential for single-nucleus RNA-seq

### use EmptyDrops cell filtering (Cell Ranger 3.x compatible)
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloCellFilter EmptyDrops_CR --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix emptydrops_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloCellFilter EmptyDrops_CR uses EmptyDrops algorithm; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix emptydrops_sample/ output prefix; compatible with Cell Ranger 3.x

### strict UMI deduplication (Exact match only)
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloUMIdedup Exact --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix exact_umi_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloUMIdedup Exact requires exact UMI matches; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix exact_umi_sample/ output prefix; strictest deduplication

### output top N cells by UMI count
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloCellFilter TopCells 5000 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix top5k_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloCellFilter TopCells 5000 outputs only top 5000 cells; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix top5k_sample/ output prefix

### no cell filtering (output all barcodes)
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloCellFilter None --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix nofilter_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloCellFilter None disables cell filtering; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix nofilter_sample/ output prefix; outputs all barcodes in whitelist

### allow variable barcode read length
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloBarcodeReadLength 0 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix varlen_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloBarcodeReadLength 0 allows variable barcode read lengths; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix varlen_sample/ output prefix; for non-standard protocols

### process Drop-seq data with CB_UMI_Complex
**Args:** `--soloType CB_UMI_Complex --soloCB0_wl 0 0 12 dropseq_whitelist.txt --soloCB0 0_0_12 0_12_8 --soloUMI0 12_20_8 --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix dropseq_sample/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Complex for Drop-seq; --soloCB0_wl 0 0 12 dropseq_whitelist.txt whitelist; --soloCB0 0_0_12 0_12_8 barcode positions; --soloUMI0 12_20_8 UMI position; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix dropseq_sample/ output prefix; Drop-seq uses 12bp CB + 8bp UMI

### process Smart-seq2 data without UMI
**Args:** `--soloType SmartSeq --soloFeatures Gene --genomeDir /star_genome/ --readFilesIn sample.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix smartseq_sample/`
**Explanation:** STAR command with STARsolo; --soloType SmartSeq for plate-based Smart-seq2; --soloFeatures Gene gene counts; --genomeDir /star_genome/ index; --readFilesIn sample.fastq.gz single sample input; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix smartseq_sample/ output prefix; no barcode/UMI demultiplexing

### run STARsolo with multi-sample batch processing
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --genomeDir /star_genome/ --readFilesIn sample1_R2.fq.gz,sample2_R2.fq.gz sample1_R1.fq.gz,sample2_R1.fq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix batch_samples/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --genomeDir /star_genome/ index; --readFilesIn sample1_R2.fq.gz,sample2_R2.fq.gz comma-separated cDNA inputs sample1_R1.fq.gz,sample2_R1.fq.gz comma-separated barcode inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix batch_samples/ output prefix; multiple samples in one STARsolo run

### output both Gene and GeneFull for comprehensive analysis
**Args:** `--soloType CB_UMI_Simple --soloCBwhitelist 3M-february-2018.txt --soloCBlen 16 --soloUMIlen 12 --soloFeatures Gene GeneFull --genomeDir /star_genome/ --readFilesIn R2.fastq.gz R1.fastq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --runThreadN 16 --outFileNamePrefix comprehensive/`
**Explanation:** STAR command with STARsolo; --soloType CB_UMI_Simple 10x mode; --soloCBwhitelist 3M-february-2018.txt whitelist; --soloCBlen 16 --soloUMIlen 12 v3 lengths; --soloFeatures Gene GeneFull outputs both gene-level and pre-mRNA counts; --genomeDir /star_genome/ index; --readFilesIn R2.fastq.gz R1.fastq.gz inputs; --readFilesCommand zcat for gzip; --outSAMtype BAM SortedByCoordinate sorted BAM; --runThreadN 16 threads; --outFileNamePrefix comprehensive/ output prefix; enables both standard and snRNA-seq analysis
