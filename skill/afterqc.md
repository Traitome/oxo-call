---
name: afterqc
category: ngs-quality-control
description: Automated quality control, filtering and trimming tool for Illumina NGS data. PerformsAdapter detection and trimming, quality filtering, bubble removal, and generates comprehensive QC reports in JSON and text formats.
tags: ngs, quality-control, illumina, filtering, trimming, fastq, adapter-removal
author: AI-generated
source_url: https://github.com/OpenGene/AfterQC
---

## Concepts

- **Input Formats:** Accepts single-end (SE) or paired-end (PE) FASTQ files, supporting both gzip compressed (.fq.gz/.fastq.gz) and uncompressed (.fq/.fastq) formats. Use `-1` and `-2` flags for paired-end reads.

- **Filtering Modes:** AfterQC operates in two modes—`simple` (basic pass/fail filtering) or `professional` (advanced QC with more detailed metrics). Use `--mode simple` or `--mode professional` to specify.

- **Adapter Handling:** Automatically detects Illumina adapters by default. Custom adapter sequences can be specified using `--adapter_A` for the forward strand and `adapter_B` for the reverse strand in paired-end data.

- **Quality Thresholds:** Defines filtering criteria via `--qualified_quality_phred` (minimum Phred score, default 20), `--cut_mean_quality` (sliding window average quality), and `--length_threshold` (minimum read length, default 15bp).

- **Output Structure:** Generates filtered FASTQ files in the specified output directory, along with JSON QC reports and simple text summaries containing filtering statistics, base composition, and quality metrics.

---

## Pitfalls

- **MisSpecifying Paired-End Input:** Using only `-1` for a paired-end dataset results in only the forward reads being processed, losing the reverse read information and producing incomplete QC statistics. Always provide `-2` for paired-end libraries.

- **Ignoring Default Output Directory:** AfterQC writes results to the current working directory if `-o` is not specified, potentially overwriting existing files or filling the working directory with intermediate files. Always explicitly specify an output directory.

- **Setting Inappropriate Quality Thresholds:** Using overly strict quality thresholds (e.g., Phred 40) may remove too many reads, while too permissive settings (e.g., Phred 10) may retain low-quality data. Adjust `--qualified_quality_phred` based on the sequencing depth requirements of your downstream analysis.

- **Neglecting Bubble Removal:** In certain NGS runs, "bubbles" (PCR chimera artifacts) can artificially inflate diversity metrics. Using `--do_bubble_correction` is recommended for applications like 16S metagenomics where taxonomy inference is sensitive to spurious diversity.

- **Incorrect File Ordering for Reverse Reads:** Swapping the forward and reverse read files in `-1` and `-2` arguments will cause incorrect mate-pair detection, leading to erroneous filtering decisions and misleading Insert Size metrics in the QC report.

---

## Examples

### Run basic QC on a single-end FASTQ file
**Args:** `-1 sample_R1.fq.gz -o qc_results/`
**Explanation:** Processes a single-end FASTQ file and writes all output to the designated directory, generating filtered reads and QC statistics.

### Run QC on paired-end data with automatic adapter detection
**Args:** `-1 sample_R1.fq.gz -2 sample_R2.fq.gz -o qc_results/ --mode professional`
**Explanation:** Performs comprehensive QC on paired-end reads using professional mode, enabling detailed metrics and automatic Illumina adapter detection.

### Filter reads with custom quality and length thresholds
**Args:** `-1 sample_R1.fq.gz -o qc_results/ --qualified_quality_phred 30 --length_threshold 50 --cut_mean_quality 25`
**Explanation:** Applies stricter filtering requiring minimum Phred 30 quality, 50bp minimum length, and sliding window average quality of 25.

### Trim custom adapter sequences from paired-end data
**Args:** `-1 sample_R1.fq.gz -2 sample_R2.fq.gz -o qc_results/ --adapter_A AGATCGGAAGAGCACACGTCTGAACTCCAGTCA --adapter_B AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT`
**Explanation:** Removes specified custom adapter sequences from both forward and reverse reads, replacing default automatic detection.

### Enable bubble removal for metagenomic data
**Args:** `-1 sample_R1.fq.gz -o qc_results/ --do_bubble_correction`
**Explanation:** Activates bubble correction to remove PCR chimera artifacts that could confound taxonomic classification in metagenomic samples.

### Apply gzip compression to output files
**Args:** `-1 sample_R1.fq.gz -2 sample_R2.fq.gz -o qc_results/ --gzip`
**Explanation:** Compresses filtered output FASTQ files using gzip, reducing storage requirements while maintaining compatibility with downstream tools.

### Process uncompressed FASTQ files with verbose output
**Args:** `-1 sample_R1.fq -2 sample_R2.fq -o qc_results/ --verbose`
**Explanation:** Processes uncompressed FASTQ files and enables verbose logging for debugging or documentation purposes.

### Generate QC only without filtering reads
**Args:** `-1 sample_R1.fq.gz -o qc_results/ --only_QC`
**Explanation:** Performs quality metrics calculation and reporting without writing filtered FASTQ files, useful for assessing data quality before deciding on filtering parameters.