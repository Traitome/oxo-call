---
name: fastq-screen
category: qc
description: Rapid contamination and composition screening of FASTQ reads against multiple reference databases
tags: [qc, contamination, screening, fastq, bowtie2, bwa, bismark, multiqc, ngs, bisulfite]
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
- --filter extracts reads mapping to specific genomes using a binary code (0=unmapped, 1=unique, 2=multi); --pass sets minimum genomes to pass.
- --tag labels each read header with alignment status (0=unmapped, 1=unique, 2=multi) for all screened genomes.
- --nohits extracts reads that did not map to any reference genome (equivalent to --tag --filter 0000).
- --inverse inverts --filter results, useful for excluding reads mapping to contaminants.
- --top extracts reads from the top of the file (faster but may introduce bias); format is --top reads[,skip_lines].
- --get_genomes downloads pre-indexed Bowtie2 genomes for common species (or Bismark indices with --bisulfite).

## Pitfalls

- FastQ Screen requires bowtie2 (or bowtie/bwa) in PATH and indexed databases; a missing aligner causes a fatal error at startup.
- Config file paths to bowtie2 indexes must be absolute or resolvable from the working directory; relative paths often break.
- The --subset default of 100000 reads may miss rare contaminants; use --subset 0 for comprehensive screening.
- FastQ Screen does not trim reads before screening; adapter-heavy reads will falsely show up as unmapped, reducing sensitivity.
- Using paired-end input with --paired requires both R1 and R2; providing only R1 with --paired enabled causes an error.
- Running on CRAM input requires samtools in PATH; FastQ Screen calls samtools view internally for CRAM support.
- --filter requires understanding binary codes: 0=unmapped, 1=unique, 2=multi; number of digits must match number of genomes in config.
- --tag processes the entire file by default; use --subset to limit processing time when tagging is not needed for all reads.
- --top is faster than --subset but may introduce bias from non-uniform read distribution; use --subset for representative sampling.
- --bisulfite requires Bismark indices; standard Bowtie2 indices will not work for bisulfite-converted libraries.

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

### extract reads that did not map to any reference genome
**Args:** `--conf fastq_screen.conf --nohits --outdir results/ --threads 8 sample.fastq.gz`
**Explanation:** --nohits extracts reads unmapped to all genomes; useful for identifying novel sequences or contamination

### filter reads mapping uniquely to human genome (assuming human is first in config)
**Args:** `--conf fastq_screen.conf --filter 1000 --outdir results/ --threads 8 sample.fastq.gz`
**Explanation:** --filter 1000 keeps reads uniquely mapping to first genome (1=unique, 0=unmapped for others); removes human contamination

### tag reads with alignment status for all genomes
**Args:** `--conf fastq_screen.conf --tag --subset 0 --outdir results/ --threads 8 sample.fastq.gz`
**Explanation:** --tag adds alignment codes to read headers; --subset 0 processes all reads; output can be used for downstream filtering

### filter with OR logic (pass if maps to any genome)
**Args:** `--conf fastq_screen.conf --filter 1000 --pass 1 --outdir results/ --threads 8 sample.fastq.gz`
**Explanation:** --pass 1 means read passes if it maps to at least 1 genome; acts as OR operator for multi-genome screening

### invert filter to exclude contaminant reads
**Args:** `--conf fastq_screen.conf --filter 1000 --inverse --outdir results/ --threads 8 sample.fastq.gz`
**Explanation:** --inverse inverts filter; keeps reads NOT mapping to first genome; useful for removing known contaminants

### download pre-indexed reference genomes
**Args:** `--get_genomes`
**Explanation:** downloads pre-indexed Bowtie2 genomes for common species; run once to set up screening databases

### screen bisulfite-converted libraries with Bismark
**Args:** `--conf bisulfite_screen.conf --bisulfite --paired --outdir results/ --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** --bisulfite uses Bismark aligner for bisulfite libraries; requires Bismark-indexed databases in config
