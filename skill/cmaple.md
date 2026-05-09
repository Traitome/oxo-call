---
name: cmaple
category: Bioinformatics Analysis
description: A command-line tool for comparative analysis of genomic or transcriptomic data, supporting sequence mapping, expression quantification, and differential analysis workflows.
tags: [genomics, transcriptomics, sequence analysis, differential expression, mapping]
author: AI-generated
source_url: https://github.com/cmaple/cmaple
---

## Concepts

- **Input formats**: cmaple accepts FASTQ, FASTA, and BAM/SAM files for input sequences, with support for gzipped (.gz) compression. Paired-end reads should be specified with paired _1 and _2 suffixes or explicit -1/-2 flags.
- **Output formats**: Primary outputs include sorted BAM files, tab-delimited count matrices, and JSON summary reports. Use the -o flag to specify output directory; default is current working directory.
- **Indexing**: For reference genomes, run `cmaple-build` prior to alignment. The index consists of three files (.1.ebt, .2.ebt, .3.ebt) stored in the reference directory. Indexing is required only once per reference.
- **Parallelization**: cmaple uses thread-based parallelism. Set thread count with -t flag; defaults to half of available CPU cores. Increase threads for large datasets to improve throughput.

## Pitfalls

- **Mismatched reference index**: Using a cmaple-build index from a different reference genome version causes misaligned reads and inflated multimapping rates. Always rebuild index when changing reference versions.
- **Ignoring quality trimming**: Passing untrimmed FASTQ files with low-quality bases leads to spurious variants or inflated expression counts. Apply quality filtering before alignment.
- **Insufficient disk space**: Output BAM files can exceed input size by 10x. Verify available disk space before running; cmaple will fail mid-process if disk fills.
- **Mixed paired/single end specifications**: Specifying -p flag without proper -1/-2 file pairing results in pairing all reads incorrectly. Ensure mate pairs are correctly ordered in input.

## Examples

### Build index from reference genome

**Args:** `ref.fa -d ref_index/`

**Explanation:** Creates searchable index files from reference FASTA for subsequent read mapping; index directory must exist before running.

### Align single-end reads to indexed reference

**Args:** `-i ref_index/ -q reads.fq -o output.bam -t 8`

**Args:** `-i ref_index/ -q reads.fq -o output.bam -t 8`
**Explanation:** Maps single-end FASTQ reads using 8 threads, producing sorted BAM output for variant calling downstream.

### Align paired-end reads with mate pair information

**Args:** `-i ref_index/ -1 read1.fq -2 read2.fq -o paired_output.bam -t 12`

**Explanation:** Processes paired-end library with explicit mate pairing, improving alignment accuracy around indels and structural variants.

### Generate expression count matrix from alignments

**Args:** `-i alignments.bam -g genes.gtf -c counts.tsv`

**Explanation:** Quantifies reads overlapping gene features to produce count table for differential expression analysis.

### Filter alignments by mapping quality

**Args:** `-i alignments.bam -q 30 -o filtered.bam`

**Explanation:** Retains only high-confidence alignments with MAPQ≥30, reducing false positives in downstream variant analysis.

### Run paired-end with automatic gzip detection

**Args:** `-i ref_index/ --fastq-list input_manifest.txt -o results/ -t 16`

**Explanation:** Processes multiple FASTQ pairs listed in manifest file, supporting batch processing for large RNA-seq projects.