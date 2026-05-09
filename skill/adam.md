---
name: adam
category: genomics
description: ADAM is a genomics processing engine that provides tools for working with genomic data in Avro format, supporting distributed processing via Apache Spark. It offers utilities for file conversion, transformation, variant calling, and coverage analysis.
tags:
- genomics
- avro
- variant-calling
- bam
- vcf
- distributed-computing
- apache-spark
- read-processing
author: AI-generated
source_url: https://github.com/bigdatagenomics/adam
---

## Concepts

- **Avro-based data model**: ADAM stores genomic data (reads, variants, coverage, features) in Avro schemas, enabling efficient serialization and schema evolution. The native `.adam` format is a columnar representation that Spark can process in parallel.
- **Reference genome requirement**: Most ADAM operations (especially read transformations and variant calling) require a reference genome in FASTA format to be specified via `-reference` or `-r`. Without this, alignment-dependent operations will fail.
- **Input format auto-detection**: ADAM auto-detects input formats (BAM, SAM, CRAM, VCF, FASTA) based on file extensions and magic bytes, but you can explicitly specify using `-input_format` when auto-detection produces incorrect results.
- **Spark distributed processing**: ADAM runs on Apache Spark, meaning the number of executor cores and memory allocation significantly impacts performance. Use `--sparkMaster` and `--numExecutors` flags to control cluster resources.

## Pitfalls

- **Large BAM files causing OOM errors**: When converting large BAM/SAM files to ADAM format without sufficient Spark memory, executors may crash with OutOfMemoryError. Increase `--driverMemory` and `--executorMemory` to accommodate the dataset size.
- **Mismatched read groups**: If input BAM files contain multiple read groups but the output ADAM file doesn't preserve them correctly, downstream tools may fail or produce incorrect results. Always verify read group preservation with `-preserve_ReadGroups`.
- **Reference genome indexing**: ADAM requires the reference FASTA to be indexed with `.fai` (via samtools faidx). If the index is missing, operations will fail with cryptic errors. Generate the index before processing.
- **Inconsistent VCF/BCF versions**: ADAM supports VCF 4.0/4.1 specifications but may reject non-standard or malformed VCFs. Validate VCF files with `vt normalize` or GATK before using them as input.
- **Parquet compression compatibility**: ADAM writes compressed Parquet files; older readers that don't support the compression codec may fail toopen outputs. Use `-compressionCodec SNAPPY` for broader compatibility.

## Examples

### Convert a BAM file to ADAM format for distributed processing
**Args:** `--input test.bam --output reads.adam --reference hg19.fa`
**Explanation:** Converts a BAM file to ADAM's Avro-based columnar format, enabling parallel Spark processing. The reference genome is required for coordinate validation.

### Extract reads overlapping a specific genomic region
**Args:** --input test.adam --regions "chr1:10000-50000" --output subset.adam
**Explanation:** Subsets reads from the ADAM file to only those overlapping chr1 positions 10kb-50kb. This reduces data size for downstream analyses and saves cluster resources.

### Sort reads by reference position
**Args:** --input unsorted.adam --output sorted.adam --sortByPos
**Explanation:** Sorts reads by genomic coordinate within each partition. Required before variant calling or when outputting to sorted BAM format.

### Compute coverage statistics for a region
**Args:** --input reads.adam --coverage --reference hg19.fa --out coverage.txt
**Explanation:** Calculates per-base coverage across the entire dataset. Outputs coveragesummary statistics useful for assessing sequencing depth and breadth.

### Align FASTQ reads to a reference genome
**Args:** --input fastq.adam --output aligned.adam --reference hg19.fa --aligner bowtie2
**Explanation:** Performs alignment of unaligned reads in FASTQ format to the reference using Bowtie 2. Requires the reference genome for alignment seed generation.

### Convert ADAM format back to sorted BAM
**Args:** --input sorted.adam --output final.bam --forceSaveAsBAM
**Explanation:** Converts processed ADAM data back to BAM format for compatibility with standard tools like IGV or GATK. Requires reads to be coordinate-sorted.

### Join variant calls with read data
**Args:** --variants variants.adam --alignedReads aligned.adam --output joined.vcf
**Explanation:** Performs a read-backed variant callset refinement by joining variant alleles with supporting read evidence for higher accuracy.