---
name: kallisto
category: rna-seq
description: Ultrafast pseudoalignment RNA-seq transcript quantification tool
tags: [rna-seq, quantification, pseudoalignment, transcript, expression, tpm, counts]
author: oxo-call built-in
source_url: "https://pachterlab.github.io/kallisto/"
---

## Concepts
- Kallisto uses pseudoalignment to quantify RNA-seq data; it is 20x faster than alignment-based tools with comparable accuracy.
- Two-step workflow: (1) kallisto index -i index.idx transcriptome.fa; (2) kallisto quant -i index.idx -o outdir R1.fq R2.fq.
- Kallisto requires transcriptome FASTA (cDNA), not genome FASTA.
- Output includes abundance.tsv (TPM and estimated counts), run_info.json (run statistics), and abundance.h5 (for sleuth).
- Use --bootstrap-samples 100 (-b 100) for bootstrap variance estimation; required for sleuth differential expression.
- Strandedness: --rf-stranded for reverse-strand (dUTP), --fr-stranded for forward-strand; default is unstranded.
- kb-python (kallisto|bustools) is the modern single-cell extension for scRNA-seq processing.
- Index options: -k sets k-mer size (default 31, odd only, max 63); --aa for amino acid sequences; --d-list for masking sequences.
- Quant options: --pseudobam outputs pseudoalignments as BAM; --genomebam projects to genome coordinates (requires -g GTF).
- Single-end mode: requires --single, -l (mean fragment length), and -s (standard deviation).
- BUS format: kallisto bus generates BUS files for single-cell data with -x for technology (10xv3, 10xv2, etc.).
- quant-tcc: quantifies from transcript-compatibility counts for long reads or pre-computed data.
- Thread control: -t/--threads for parallel processing; beneficial for index building and quantification.

## Pitfalls
- Kallisto ARGS must start with a subcommand (index, quant, quant-tcc, bus, h5dump, inspect, version, cite) — never with flags like -i, -o, -b. The subcommand ALWAYS comes first.
- Kallisto requires transcriptome cDNA FASTA, NOT genome FASTA — indexing the genome produces wrong results.
- Without --bootstrap-samples, downstream differential expression with sleuth cannot estimate variance.
- For paired-end, both FASTQ files are passed as positional arguments (no -1/-2 flags) after -o output_dir.
- kallisto does not output BAM by default — use kallisto quant --pseudobam for BAM output (if needed).
- Forgetting strandedness option for stranded libraries leads to approximately half the reads assigned per strand.
- kallisto quant outputs to a directory — make sure the output directory does not already exist or use a fresh path.
- --genomebam requires both -g GTF and optionally -c chromosomes file for proper genome coordinate projection.
- k-mer size (-k) must be odd and ≤63; even values or values >63 will cause errors.
- Single-end quantification requires explicit --single, -l, and -s parameters; missing any will fail.
- The --d-list option masks sequences from quantification; useful for removing rRNA or other contaminants.
- For single-cell data, use kb-python (kb count) instead of direct kallisto bus for easier workflow.

## Examples

### build a kallisto index from a transcriptome FASTA
**Args:** `index -i transcriptome.idx transcriptome.fa`
**Explanation:** kallisto index subcommand; -i transcriptome.idx output index file; transcriptome.fa input transcriptome FASTA

### quantify paired-end RNA-seq reads
**Args:** `quant -i transcriptome.idx -o sample_output -b 100 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** kallisto quant subcommand; -i transcriptome.idx input index; -o sample_output output directory; -b 100 bootstrap samples; --threads 8 threads; R1.fastq.gz R2.fastq.gz paired-end reads

### quantify single-end RNA-seq reads with fragment length parameters
**Args:** `quant -i transcriptome.idx -o sample_output --single -l 200 -s 20 -b 100 --threads 8 reads.fastq.gz`
**Explanation:** kallisto quant subcommand; -i transcriptome.idx input index; -o sample_output output directory; --single single-end mode; -l 200 mean fragment length; -s 20 fragment length SD; -b 100 bootstrap samples; --threads 8 threads; reads.fastq.gz input reads

### quantify strand-specific reverse-strand paired-end RNA-seq
**Args:** `quant -i transcriptome.idx -o sample_output --rf-stranded -b 100 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** kallisto quant subcommand; -i transcriptome.idx input index; -o sample_output output directory; --rf-stranded reverse-strand dUTP library; -b 100 bootstrap samples; --threads 8 threads; R1.fastq.gz R2.fastq.gz paired-end reads

### quantify multiple samples in batch
**Args:** `quant -i transcriptome.idx -o sample1_out -b 50 --threads 4 sample1_R1.fq.gz sample1_R2.fq.gz`
**Explanation:** kallisto quant subcommand; -i transcriptome.idx input index; -o sample1_out output directory; -b 50 bootstrap samples; --threads 4 threads; sample1_R1.fq.gz sample1_R2.fq.gz paired-end reads

### build index with custom k-mer size
**Args:** `index -k 21 -i transcriptome.idx transcriptome.fa`
**Explanation:** kallisto index subcommand; -k 21 custom k-mer size; -i transcriptome.idx output index file; transcriptome.fa input transcriptome FASTA

### build index masking rRNA sequences
**Args:** `index -i transcriptome.idx --d-list rRNA.fa transcriptome.fa`
**Explanation:** kallisto index subcommand; -i transcriptome.idx output index; --d-list rRNA.fa mask rRNA sequences; transcriptome.fa input transcriptome FASTA

### generate pseudoalignments as BAM file
**Args:** `quant -i transcriptome.idx -o sample_output --pseudobam -b 100 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** kallisto quant subcommand; -i transcriptome.idx input index; -o sample_output output directory; --pseudobam output BAM; -b 100 bootstrap samples; --threads 8 threads; R1.fastq.gz R2.fastq.gz paired-end reads

### project pseudoalignments to genome coordinates
**Args:** `quant -i transcriptome.idx -o sample_output --genomebam -g annotation.gtf -c chromosomes.txt -b 100 R1.fastq.gz R2.fastq.gz`
**Explanation:** kallisto quant subcommand; -i transcriptome.idx input index; -o sample_output output directory; --genomebam project to genome coordinates; -g annotation.gtf GTF file; -c chromosomes.txt chromosome sizes; -b 100 bootstrap samples; R1.fastq.gz R2.fastq.gz paired-end reads

### generate BUS file for single-cell data
**Args:** `bus -i transcriptome.idx -o bus_output -x 10xv3 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** kallisto bus subcommand; -i transcriptome.idx input index; -o bus_output output directory; -x 10xv3 single-cell technology; --threads 8 threads; R1.fastq.gz R2.fastq.gz input reads

### list supported single-cell technologies
**Args:** `bus --list`
**Explanation:** kallisto bus subcommand; --list displays supported technologies

### convert HDF5 output to plaintext
**Args:** `h5dump abundance.h5 > abundance_dump.tsv`
**Explanation:** kallisto h5dump subcommand; abundance.h5 input HDF5; output to abundance_dump.tsv

### inspect index file information
**Args:** `inspect transcriptome.idx`
**Explanation:** kallisto inspect subcommand; transcriptome.idx input index file
