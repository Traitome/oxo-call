---
name: kb
category: single-cell
description: "kallisto|bustools workflow for fast single-cell RNA-seq preprocessing and count matrix generation"
tags: [single-cell, scrna-seq, kallisto, bustools, barcode, umi, 10x-genomics, kb-python]
author: oxo-call built-in
source_url: "https://www.kallistobus.tools/"
---

## Concepts
- kb-python (kb) wraps kallisto and bustools for ultra-fast scRNA-seq preprocessing.
- Main commands: kb ref for reference/index building; kb count for FASTQ to count matrix; kb extract for read extraction.
- Use -x to specify technology: 10xv3, 10xv2, 10xv1, INDROPS, CELSEQ, CELSEQ2, SMARTSEQ2, BULK, etc.
- kb count takes FASTQ files as positional arguments (not -1/-2 flags); order depends on technology.
- Output: cells_x_genes.mtx (sparse matrix), cells_x_genes.barcodes.txt, cells_x_genes.genes.txt.
- Use --workflow lamanno for RNA velocity output (spliced/unspliced matrices).
- Use --h5ad to output AnnData format directly for Scanpy; --loom for loom format; --cellranger for Cell Ranger compatible output.
- kb ref --workflow nac builds a reference for nascent/ambiguous/complete RNA quantification (single-nuclei).
- kb ref --workflow kite builds index for feature barcoding (CITE-seq, cell hashing, etc.).
- Use kb --list to see all supported technologies with barcode/UMI positions.
- Memory control: -m sets max memory (e.g., -m 32G); --tmp sets temporary directory.
- Thread control: -t sets number of threads (default: 8).
- Multimapping: --mm includes reads pseudoaligning to multiple genes.
- TCC matrix: --tcc outputs transcript-compatibility counts instead of gene counts.

## Pitfalls
- The -x technology flag must match the library preparation protocol exactly; use kb --list to verify.
- FASTQ file order matters: for 10xv3, R1 (barcode+UMI) comes first, then R2 (cDNA) as positional arguments.
- kb bundles its own kallisto and bustools binaries; no separate installation needed.
- The reference must be built with kb ref matching the desired workflow (standard vs lamanno vs nac vs kite).
- Without --h5ad, kb outputs sparse matrix format — convert manually for Scanpy/Seurat compatibility.
- --workflow nac requires -c1 and -c2 files (mature and nascent transcript-to-capture mappings).
- --workflow kite requires a feature barcode TSV file as the last argument to kb ref.
- Memory issues: use -m to limit memory; --tmp to specify fast local disk for temporary files.
- --cellranger output format requires --workflow nac for spliced/unspliced subdirectories.
- Batch processing: use --batch-barcodes with a batch file for multiple samples.

## Examples

### build kb reference from genome and GTF
**Args:** `ref -i index.idx -g t2g.txt -f1 cdna.fasta genome.fa genes.gtf`
**Explanation:** kb ref subcommand; -i index.idx output kallisto index; -g t2g.txt transcript-to-gene mapping; -f1 cdna.fasta output cDNA FASTA; genome.fa genes.gtf input genome and annotation

### process 10x Chromium v3 scRNA-seq FASTQ files
**Args:** `count -i index.idx -g t2g.txt -x 10xv3 -o output_dir/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 single-cell technology; -o output_dir/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### process scRNA-seq with RNA velocity output
**Args:** `count -i spliced_unspliced.idx -g t2g.txt -x 10xv3 --workflow lamanno -o velocity_output/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i spliced_unspliced.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 technology; --workflow lamanno RNA velocity; -o velocity_output/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### process 10x Chromium v3 and output AnnData for Scanpy
**Args:** `count -i index.idx -g t2g.txt -x 10xv3 --h5ad -o output_dir/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 technology; --h5ad output AnnData format; -o output_dir/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### download pre-built reference instead of building locally
**Args:** `ref -d mouse -i index.idx -g t2g.txt`
**Explanation:** kb ref subcommand; -d mouse download pre-built mouse index; -i index.idx output index; -g t2g.txt transcript-to-gene mapping

### build reference for nascent/ambiguous/complete (NAC) RNA quantification
**Args:** `ref --workflow nac -i index.idx -g t2g.txt -f1 mature.fa -f2 nascent.fa -c1 mature.t2c.txt -c2 nascent.t2c.txt genome.fa genes.gtf`
**Explanation:** kb ref subcommand; --workflow nac for single-nuclei; -i index.idx output index; -g t2g.txt transcript-to-gene mapping; -f1 mature.fa mature FASTA; -f2 nascent.fa nascent FASTA; -c1 mature.t2c.txt mature capture mapping; -c2 nascent.t2c.txt nascent capture mapping; genome.fa genes.gtf input genome and annotation

### process single-nuclei RNA-seq with NAC workflow
**Args:** `count -i nac_index.idx -g t2g.txt -c1 mature.t2c.txt -c2 nascent.t2c.txt -x 10xv3 --workflow nac --h5ad -o nac_output/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i nac_index.idx input index; -g t2g.txt transcript-to-gene mapping; -c1 mature.t2c.txt mature capture mapping; -c2 nascent.t2c.txt nascent capture mapping; -x 10xv3 technology; --workflow nac single-nuclei; --h5ad output AnnData; -o nac_output/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### build feature barcode index for CITE-seq
**Args:** `ref --workflow kite -i fb_index.idx -g f2g.txt -f1 features.fa feature_barcodes.txt`
**Explanation:** kb ref subcommand; --workflow kite for feature barcoding; -i fb_index.idx output index; -g f2g.txt feature-to-gene mapping; -f1 features.fa features FASTA; feature_barcodes.txt barcode list

### process CITE-seq data with feature barcoding
**Args:** `count -i fb_index.idx -g f2g.txt -x 10xv3 --workflow kite --h5ad -o cite_output/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i fb_index.idx input index; -g f2g.txt feature-to-gene mapping; -x 10xv3 technology; --workflow kite feature barcoding; --h5ad output AnnData; -o cite_output/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### output Cell Ranger compatible format
**Args:** `count -i index.idx -g t2g.txt -x 10xv3 --cellranger -o cellranger_output/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 technology; --cellranger Cell Ranger compatible output; -o cellranger_output/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### include multimapped reads in quantification
**Args:** `count -i index.idx -g t2g.txt -x 10xv3 --mm --h5ad -o mm_output/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 technology; --mm include multimapped reads; --h5ad output AnnData; -o mm_output/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### generate TCC matrix instead of gene counts
**Args:** `count -i index.idx -g t2g.txt -x 10xv3 --tcc --h5ad -o tcc_output/ -t 16 R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 technology; --tcc transcript-compatibility counts; --h5ad output AnnData; -o tcc_output/ output directory; -t 16 threads; R1.fastq.gz R2.fastq.gz input reads

### list all supported single-cell technologies
**Args:** `--list`
**Explanation:** kb command; --list displays supported technologies

### extract reads pseudoaligned to specific genes
**Args:** `extract -i index.idx -g t2g.txt -ts GeneA GeneB -o extracted_reads/ -t 16 reads.fastq.gz`
**Explanation:** kb extract subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -ts GeneA GeneB target genes; -o extracted_reads/ output directory; -t 16 threads; reads.fastq.gz input reads

### run with memory and thread limits
**Args:** `count -i index.idx -g t2g.txt -x 10xv3 -m 32G --tmp /scratch/tmp -t 8 --h5ad -o output/ R1.fastq.gz R2.fastq.gz`
**Explanation:** kb count subcommand; -i index.idx input index; -g t2g.txt transcript-to-gene mapping; -x 10xv3 technology; -m 32G memory limit; --tmp /scratch/tmp temp directory; -t 8 threads; --h5ad output AnnData; -o output/ output directory; R1.fastq.gz R2.fastq.gz input reads
