---
name: besst
category: Small RNA Sequencing Error Correction
description: A tool for error-correcting small RNA sequencing data using k-mer based approaches and bootstrap methods to improve downstream analysis accuracy for microRNA and other small RNA studies.
tags: [small-RNA, error-correction, sequencing, bioinformatics, microRNA, quality-filtering]
author: AI-Generated
source_url: https://github.com/ksahlin/BESST
---

## Concepts

- BESST processes raw FASTQ input files containing small RNA sequencing reads (typically 18-35 nucleotides) and performs error correction by comparing k-mer frequencies across reads to distinguish authentic sequence variants from sequencing noise
- The algorithm applies a bootstrap resampling approach to assess confidence in each correction, marking corrections with low confidence as ambiguous rather than forcibly modifying the sequence
- Output consists of error-corrected FASTQ files along with a report detailing the number and types of corrections applied, enabling downstream microRNA quantification tools to operate on higher-quality data
- BESST supports both single-end and paired-end small RNA sequencing data, automatically detecting read orientation for paired-end libraries before applying correction logic

## Pitfalls

- Using BESST on datasets containing reads longer than 50 nucleotides will produce unreliable corrections because the k-mer frequency models are optimized for the short length distribution typical of microRNAs, potentially introducing chimeric sequences
- Skipping the built-in quality score filtering (by setting `-q` below the recommended threshold) can allow low-confidence base calls to drive incorrect corrections, especially in regions with high sequencing error rates near adapter remnants
- Running BESST without specifying an output directory with `-o` causes corrected reads to overwrite input files, permanently losing the uncorrected data if subsequent analysis reveals overcorrection issues
- Applying default k-mer size parameters to small RNA data from different library preparation methods (such as AGO-crosslinked reads) may reduce correction accuracy because the underlying error profiles differ from standard Illumina small RNA protocols

## Examples

### Basic error correction with default parameters
**Args:** -q sample_smallRNA.fastq -o corrected_output
**Explanation:** This runs standard error correction on a single FASTQ file using default k-mer sizes and correction thresholds, writing results to the specified output directory.

### Quality-aware error correction with explicit threshold
**Args:** -q sample_smallRNA.fastq -o corrected_output -min_quality 25
**Explanation:** Setting a minimum quality threshold filters out low-confidence base calls before the correction algorithm evaluates k-mer frequencies, reducing false corrections in noisy regions.

### Paired-end small RNA sequencing correction
**Args:** -pe paired_smallRNA_reads.fastq -o corrected_paired -orient
**Explanation:** The paired-end mode automatically detects read orientation and applies correction across both ends of overlapping read pairs, improving error detection in duplex regions.

### Custom k-mer size for alternative small RNA length distributions
**Args:** -q small_rna_atac.fastq -k 6 -o corrected_atac -s 20
**Explanation:** Reducing the k-mer size accommodates shorter small RNA fragments from ATAC-seq derived nuclei acids, which may differ from standard microRNA length profiles.

### Generating correction statistics for quality assessment
**Args:** -q sample_smallRNA.fastq -o corrected_output -report stats.json
**Explanation:** Exporting correction statistics to JSON format enables integration with downstream pipelines and quality control dashboards for sequencing run evaluation.

### Multi-threaded processing for large datasets
**Args:** -q large_mirseq_dataset.fastq -o corrected_large -threads 8
**Explanation:** Parallelizing across 8 CPU threads accelerates processing of large sequencing runs without altering the underlying correction algorithm or quality thresholds.