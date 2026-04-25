---
name: chromap
category: epigenomics
description: Ultrafast short-read aligner for single-cell ATAC-seq and ChIP-seq with built-in barcode processing
tags: [atac-seq, chip-seq, alignment, single-cell, barcode, epigenomics, fast, scatac, hic, Tn5]
author: oxo-call built-in
source_url: "https://github.com/haowenz/chromap"
---

## Concepts

- Chromap is an ultrafast aligner specifically designed for ATAC-seq, ChIP-seq, and scATAC-seq data.
- Use --preset atac for ATAC-seq; --preset chip for ChIP-seq; --preset hic for Hi-C data.
- Build index: chromap -i -r genome.fa -o genome.index
- Align: chromap --preset atac -x genome.index -r genome.fa -1 R1.fq -2 R2.fq -o output.bed
- For scATAC-seq with barcodes, use -b barcode.fq and --barcode-whitelist whitelist.txt.
- Chromap outputs in BED, SAM, or pairs format; use -q for minimum MAPQ filter.
- Chromap is 10-100x faster than BWA+Picard for ATAC-seq peak calling workflows.
- --Tn5-shift performs Tn5 transposase shift (+4bp forward, -5bp reverse) for ATAC-seq peak calling.
- --trim-adapters removes sequencing adapters from 3' ends automatically.
- --barcode-translate converts barcodes to different sequences during output for multiplexing.
- --read-format specifies barcode position in reads (e.g., "r1:0:-1,bc:0:15" for 10x format).
- --low-mem mode reduces memory usage for large reference genomes.
- Three deduplication modes: bulk-level (--remove-pcr-duplicates), cell-level, or no dedup.

## Pitfalls

- chromap has NO subcommands. ARGS starts directly with flags (e.g., -i, --preset, -x, -r, -1, -2). Do NOT put a subcommand like 'align' or 'index' before flags. Use -i flag (not a subcommand) for index building mode.
- Chromap requires its own index (not BWA or Bowtie2 indices) — build with chromap -i.
- For ATAC-seq, use --preset atac to handle fragment length distribution properly.
- Chromap output is fragments/BED by default, not BAM — convert if BAM is required.
- For scATAC-seq, the barcode FASTQ (-b) must be correctly specified with position.
- Chromap deduplicates by default — use --no-remove-pcr-duplicates to disable.
- --Tn5-shift only works with BED/TagAlign output, NOT with --SAM output format.
- --barcode-whitelist file should be uncompressed or use process substitution <(zcat file.gz).
- Default MAPQ threshold is 30; lower with -q if you need more sensitive alignments.
- --read-format syntax is critical: "r1:0:-1,bc:0:15" means barcode is first 16bp of read1.

## Examples

### build Chromap genome index
**Args:** `-i -r genome.fa -o genome.index`
**Explanation:** -i index mode; -r genome.fa reference FASTA input; -o genome.index index output file

### align paired-end ATAC-seq reads with Chromap
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -o fragments.bed -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; -o fragments.bed output BED; -t 16 threads

### process single-cell ATAC-seq with barcodes
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -b barcode.fastq.gz --barcode-whitelist whitelist.txt -o scatac_fragments.bed -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; -b barcode.fastq.gz barcode FASTQ; --barcode-whitelist whitelist.txt valid barcode list; -o scatac_fragments.bed output BED; -t 16 threads; outputs fragments per cell

### align ChIP-seq reads with Chromap
**Args:** `--preset chip -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -o chip_aligned.bed -t 16`
**Explanation:** --preset chip ChIP-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; -o chip_aligned.bed output BED; -t 16 threads

### align with Tn5 shift for ATAC-seq peak calling
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz --Tn5-shift -o fragments_shifted.bed -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; --Tn5-shift adjusts coordinates (+4bp forward, -5bp reverse) for Tn5 integration sites; -o fragments_shifted.bed output BED; -t 16 threads

### output in SAM format for downstream analysis
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz --SAM -o aligned.sam -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; --SAM outputs SAM format instead of default BED; -o aligned.sam output SAM; -t 16 threads; note --Tn5-shift does NOT work with --SAM

### align Hi-C reads with pairs format output
**Args:** `--preset hic -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz --pairs -o hic_pairs.pairs -t 16`
**Explanation:** --preset hic Hi-C preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; --pairs outputs 4DN pairs format for Hi-C analysis tools; -o hic_pairs.pairs output pairs; -t 16 threads

### trim adapters and remove duplicates for clean ATAC-seq
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz --trim-adapters --remove-pcr-duplicates -o clean_fragments.bed -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; --trim-adapters removes 3' adapters; --remove-pcr-duplicates removes PCR duplicates at bulk level; -o clean_fragments.bed output BED; -t 16 threads

### low memory mode for large genomes
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz --low-mem -o fragments.bed -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; --low-mem reduces memory footprint; -o fragments.bed output BED; -t 16 threads; useful for large reference genomes or memory-constrained systems

### scATAC-seq with cell-level deduplication
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -b barcode.fastq.gz --barcode-whitelist whitelist.txt --remove-pcr-duplicates-at-cell-level -o sc_fragments.bed -t 16`
**Explanation:** --preset atac ATAC-seq preset; -x genome.index Chromap index; -r genome.fa reference FASTA; -1 R1.fastq.gz -2 R2.fastq.gz paired reads input; -b barcode.fastq.gz barcode FASTQ; --barcode-whitelist whitelist.txt valid barcode list; --remove-pcr-duplicates-at-cell-level deduplicates per cell barcode; -o sc_fragments.bed output BED; -t 16 threads; more stringent than bulk-level
