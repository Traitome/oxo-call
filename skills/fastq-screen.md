---
name: fastq-screen
category: qc
description: Rapid contamination and composition screening of FASTQ reads against multiple reference databases
tags: [qc, contamination, screening, fastq, bowtie2, multiqc, ngs]
author: oxo-call built-in
source_url: "https://www.bioinformatics.babraham.ac.uk/projects/fastq_screen/"
---

## Concepts

- FastQ Screen aligns a subset of reads to multiple user-defined databases in parallel and reports the fraction mapping to each.
- Databases are configured in a text config file with genome name, path to bowtie2 index, and aligner; multiple genomes can be screened simultaneously.
- By default FastQ Screen samples 100,000 reads (--subset); use --subset 0 to screen all reads for thorough contamination assessment.
- Output includes a PNG plot and a TXT table; the TXT is MultiQC-compatible and can be aggregated across samples automatically.
- Common databases to include: human, mouse, PhiX, Mycoplasma, E. coli, rRNA, adapter sequences, and sample-specific organisms.
- --aligner can be set to bowtie2 (default), bowtie, or bwa depending on what is installed and configured.

## Pitfalls

- FastQ Screen requires bowtie2 (or bowtie/bwa) in PATH and indexed databases; a missing aligner causes a fatal error at startup.
- Config file paths to bowtie2 indexes must be absolute or resolvable from the working directory; relative paths often break.
- The --subset default of 100000 reads may miss rare contaminants; use --subset 0 for comprehensive screening.
- FastQ Screen does not trim reads before screening; adapter-heavy reads will falsely show up as unmapped, reducing sensitivity.
- Using paired-end input with --paired requires both R1 and R2; providing only R1 with --paired enabled causes an error.
- Running on CRAM input requires samtools in PATH; FastQ Screen calls samtools view internally for CRAM support.

## Examples

### screen a FASTQ file against default databases
**Args:** `--conf fastq_screen.conf --outdir results/ --threads 8 sample_R1.fastq.gz`
**Explanation:** --conf points to the database config file; --outdir sets output directory; --threads for parallel alignment

### screen all reads (no subsampling) for thorough contamination check
**Args:** `--conf fastq_screen.conf --subset 0 --outdir results/ --threads 8 sample_R1.fastq.gz`
**Explanation:** --subset 0 disables subsampling and screens every read; slower but detects rare contaminants

### screen paired-end reads and report bisulfite alignment stats
**Args:** `--conf fastq_screen.conf --aligner bismark --paired --outdir results/ --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** --aligner bismark for bisulfite-treated libraries; --paired enables paired-end mode

### screen reads and get only the table output without generating plots
**Args:** `--conf fastq_screen.conf --no_html --outdir results/ --threads 8 sample.fastq.gz`
**Explanation:** --no_html suppresses HTML/PNG generation; faster when only the TXT table is needed

### add a custom database to the config and screen for mycoplasma contamination
**Args:** `--conf custom_screen.conf --outdir results/ --threads 8 sample_R1.fastq.gz`
**Explanation:** custom_screen.conf should include a DATABASE line for Mycoplasma bowtie2 index; same command, custom config

### screen multiple samples in a loop and collect MultiQC report
**Args:** `for f in *.fastq.gz; do fastq_screen --conf fastq_screen.conf --outdir screen_results/ --threads 4 $f; done && multiqc screen_results/ -o multiqc_report/`
**Explanation:** screen each sample then aggregate all _screen.txt files with MultiQC for a combined contamination report
