---
name: cassiopee
category: genomics
description: A bioinformatics tool for processing and analyzing genomic sequencing data, designed for variant calling and sequence alignment workflows.
tags:
- genomics
- variant-calling
- sequence-analysis
- dna-sequencing
- bioinformatics
author: AI-generated
source_url: https://github.com/cassiopee/cassiopee
---

## Concepts

- **Input Formats:** Cassiopee accepts FASTQ, FASTA, and BAM/SAM files as primary inputs, supporting both single-end and paired-end sequencing reads.
- **Output Data Model:** The tool generates VCF (Variant Call Format) files containing detected genetic variants, along with optional alignment summaries in BED format.
- **Alignment Engine:** Cassiopee uses a custom hash-based alignment algorithm optimized for speed with whole-genome sequencing data, enabling efficient gapped Alignment for INDEL detection.
- **Variant Calling Pipeline:** The tool implements a ensemble approach combining multiple statistical models to call SNVs and small INDELs, with built-in quality filtering thresholds.

## Pitfalls

- **Missing Reference Genome:** Running cassiopee without specifying a reference genome using the `--ref` flag will cause alignment failures, as the tool requires a indexed reference for read mapping.
- **Incompatible File Encoding:** Providing input FASTQ files with non-standard line endings (Windows-style CRLF) can cause parsing errors and lead to truncated output or silent data loss.
- **Memory Exhaustion with Large Files:** Specifying an overly large number of threads via `--threads` on systems with limited RAM can cause the process to crash, particularly when processing whole-genome BAM files.

## Examples

### Align sequencing reads to a reference genome
**Args:** `--ref grch38.fa --input reads.fq --output alignment.bam --algorithm hash`
**Explanation:** Aligns input FASTQ reads to the GRCh38 reference genome using the hash-based algorithm, producing a sorted BAM output file.

### Call variants from an alignment file
**Args:** `--input alignment.bam --ref grch38.fa --output variants.vcf --min-quality 30`
**Explanation:** Performs variant calling on the provided alignment, filtering out calls with quality scores below 30 and writing results to a VCF file.

### Run variant calling with paired-end data
**Args:** `--input reads1.fq reads2.fq --ref grch38.fa --output results.vcf --library paired --min-depth 10`
**Explanation:** Processes paired-end sequencing data, requiring a minimum read depth of 10 for variant calling.

### Generate a summary report of called variants
**Args:** `--input variants.vcf --report-summary --output summary.txt --format bed`
**Explanation:** Converts variant calls into a BED format summary report for downstream genomic region analysis.

### Adjust filtering thresholds for high-confidence calls
**Args:** `--input alignment.bam --ref grch38.fa --output strict.vcf --filter-snps --min-allele-freq 0.95`
**Explanation:** Applies stringent filtering requiring allele frequencies of 95% or higher, suitable for identifying fixed genetic differences.

### Process multiple samples in batch mode
**Args:** --batch sample_manifest.txt --ref grch38.fa --output-dir results/ --threads 8
**Explanation:** Processes multiple samples listed in a manifest file using 8 concurrent threads, writing all outputs to a specified directory.