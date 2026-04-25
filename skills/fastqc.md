---
name: fastqc
category: qc
description: Quality control analysis tool for high-throughput sequencing data producing HTML and zip reports
tags: [qc, quality-control, fastq, ngs, illumina, report, sequencing, bam, sam]
author: oxo-call built-in
source_url: "https://www.bioinformatics.babraham.ac.uk/projects/fastqc/"
---

## Concepts

- FastQC generates HTML and ZIP reports per FASTQ file; reports include per-base quality, sequence quality distribution, GC content, and adapter content.
- Use -o to specify output directory; FastQC creates <input_name>_fastqc.html and <input_name>_fastqc.zip in that directory.
- Use -t N for parallel processing of multiple files (one thread per file, not internal threading).
- FastQC accepts FASTQ (.fastq, .fq), gzipped FASTQ (.fastq.gz, .fq.gz), BAM, SAM, and BFAST formats.
- The --noextract flag keeps the zip file without extracting it (useful when only the HTML is needed).
- Key modules: Per Base Sequence Quality, Per Sequence Quality, Per Base N Content, Sequence Duplication, Adapter Content.
- FastQC works in batch mode — pass multiple files at once: fastqc file1.fq file2.fq -o qc_results/
- --nogroup disables base grouping for reads >50bp; shows data for every base but may crash on very long reads.
- --memory sets base memory per file (default 512MB); increase for files with very long sequences.
- --svg saves graphs in SVG format instead of PNG for higher quality vector graphics.
- -k/--kmers sets Kmer length for Kmer content module (2-10bp, default 7).
- -c/--contaminants specifies a file with contaminant sequences to screen overrepresented sequences.
- --dup_length sets truncation length for duplicate detection (default 50bp); useful for long reads with UMIs.

## Pitfalls

- fastqc has NO subcommands. ARGS starts directly with input files or flags (e.g., -t, -o, --noextract). Do NOT put a subcommand like 'check' or 'analyze' before flags.
- FastQC output goes to the same directory as the input by default — always use -o to control output location.
- FastQC is single-threaded per file; -t only helps when processing multiple files simultaneously.
- FastQC adapts thresholds for short reads but may give misleading WARN/FAIL on amplicon or very short-read data — interpret in context.
- FastQC does not trim reads — it only evaluates quality and generates reports.
- For BAM input, FastQC works on the sequences as stored — ensure BAM is not empty.
- Using --extract together with -o creates subdirectories — check that the output path has write permission.
- --nogroup can crash on very long reads (e.g., PacBio/Nanopore) and create huge plots — use with caution.
- Each thread allocates 512MB memory by default; ensure sufficient RAM when using high -t values.
- Kmer module is disabled by default in recent versions; enable with appropriate -k value if needed.
- --casava flag only works with raw Casava output files with specific naming conventions.

## Examples

### run quality control on a single FASTQ file
**Args:** `reads.fastq.gz -o qc_results/`
**Explanation:** reads.fastq.gz input file; -o qc_results/ output directory; generates reads_fastqc.html and reads_fastqc.zip in qc_results/; create the output directory first

### run quality control on paired-end FASTQ files using 4 threads
**Args:** `-t 4 -o qc_results/ R1.fastq.gz R2.fastq.gz`
**Explanation:** -t 4 processes both files in parallel (2 files × 2 threads for I/O); -o qc_results/ output directory; R1.fastq.gz R2.fastq.gz input files; output goes to qc_results/

### run quality control on multiple samples and keep zip files without extracting
**Args:** `--noextract -t 8 -o qc_output/ sample1_R1.fastq.gz sample1_R2.fastq.gz sample2_R1.fastq.gz sample2_R2.fastq.gz`
**Explanation:** --noextract keeps only HTML and zip without extracting; -t 8 processes up to 8 files in parallel; -o qc_output/ output directory; sample1_R1.fastq.gz sample1_R2.fastq.gz sample2_R1.fastq.gz sample2_R2.fastq.gz input files

### run fastqc on a BAM file
**Args:** `-t 4 -o qc_results/ aligned.bam`
**Explanation:** -t 4 threads; -o qc_results/ output directory; aligned.bam input BAM file; FastQC can process BAM files; useful for QC of already-aligned data

### run fastqc with custom adapter sequences and format specification
**Args:** `-f fastq -a adapters.txt -t 4 -o qc_results/ reads.fastq.gz`
**Explanation:** -f fastq specifies format; -a adapters.txt provides custom adapter sequences for adapter content module; -t 4 threads; -o qc_results/ output directory; reads.fastq.gz input file

### run fastqc with SVG output for publication-quality graphics
**Args:** `--svg -t 4 -o qc_results/ sample1.fastq.gz sample2.fastq.gz`
**Explanation:** --svg generates SVG format graphs instead of PNG; -t 4 threads; -o qc_results/ output directory; sample1.fastq.gz sample2.fastq.gz input files; better for publications and presentations

### run fastqc on long reads with increased memory
**Args:** `--memory 1024 -t 2 -o qc_results/ long_reads.fastq.gz`
**Explanation:** --memory 1024 allocates 1GB per file; -t 2 threads; -o qc_results/ output directory; long_reads.fastq.gz input file; necessary for files with very long sequences (e.g., PacBio)

### run fastqc with custom contaminant screening
**Args:** `-c contaminants.txt -t 4 -o qc_results/ reads.fastq.gz`
**Explanation:** -c contaminants.txt specifies contaminant file (name[tab]sequence format); -t 4 threads; -o qc_results/ output directory; reads.fastq.gz input file; screens overrepresented sequences

### run fastqc with specific Kmer length for Kmer content analysis
**Args:** `-k 5 -t 4 -o qc_results/ reads.fastq.gz`
**Explanation:** -k 5 sets Kmer length to 5bp for Kmer content module; -t 4 threads; -o qc_results/ output directory; reads.fastq.gz input file; valid range is 2-10bp

### run fastqc on Casava raw output files
**Args:** `--casava -t 4 -o qc_results/ sample_L001_R1_001.fastq.gz sample_L001_R2_001.fastq.gz`
**Explanation:** --casava groups files from same sample and excludes filtered reads; -t 4 threads; -o qc_results/ output directory; sample_L001_R1_001.fastq.gz sample_L001_R2_001.fastq.gz input files; requires Casava naming convention

### run fastqc with custom duplication detection length
**Args:** `--dup_length 75 -t 4 -o qc_results/ reads_with_umis.fastq.gz`
**Explanation:** --dup_length 75 truncates sequences to 75bp for duplicate detection; -t 4 threads; -o qc_results/ output directory; reads_with_umis.fastq.gz input file; useful for long reads with UMIs
