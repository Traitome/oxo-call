---
name: bolt
category: genomics/sequence-analysis
description: A high-performance bioinformatics tool for processing genomic data, performing alignment, variant calling, or sequence analysis tasks. Designed for scalability and rapid processing of large genomic datasets.
tags: [genomics, sequence-analysis, variant-calling, alignment, bioinformatics, dna-analysis]
author: AI-generated
source_url: https://example.com/bolt-docs
---

## Concepts

- **Input Formats**: bolt accepts standard bioinformatics file formats including FASTA (.fa, .fasta), FASTQ (.fq, .fastq), SAM/BAM (.sam, .bam), and VCF (.vcf) for variant data. Files can be uncompressed or gzip-compressed.
- **Output Modes**: By default, bolt writes results to stdout in text format. Use the `--out` flag to specify an output file path. Output can be redirected to files for downstream analysis.
- **Threading and Performance**: bolt uses multiple threads for parallel processing via the `--threads` flag. More threads generally yield faster processing but increase memory consumption. The default is 1 thread.
- **Data Model**: bolt processes genomic data using a coordinate-based system where sequences are indexed starting at position 1. It supports both 0-based and 1-based coordinate conventions depending on the input format.

## Pitfalls

- **Forgetting to specify input format**: If the input file lacks a recognized extension, bolt defaults to FASTA mode. Using `--format` explicitly avoids parsing errors that can silently corrupt downstream analysis results.
- **Insufficient memory for large datasets**: Running bolt on whole-genome data without enough RAM causes process termination. Monitor memory usage and consider chunking input files or reducing `--threads` to limit footprint.
- **Ignoring strandedness flags**: Many analysis tasks are strand-specific. Omitting `--strand` or `--rf` flags when the protocol requires it produces biologically incorrect results that invalidate downstream interpretations.
- **Mismatched reference genomes**: Using a different reference build than your input data creates coordinate misalignments that are difficult to detect but lead to false positive or false negative results in variant calling.

## Examples

### Align raw sequencing reads to a reference genome
**Args:** `--reference hg38.fa --input reads.fq --output alignments.bam`
**Explanation:** This command aligns FASTQ reads to the hg38 reference genome and outputs a BAM file containing aligned sequences with coordinate information.

### Call variants from an alignment file
**Args:** `--input alignments.bam --call-variants --out variants.vcf`
**Explanation:** This command performs variant calling on an existing BAM alignment file and outputs a VCF file containing discovered variants with genotype information.

### Filter variants by quality threshold
**Args:** `--input variants.vcf --filter-qual 30 --out filtered.vcf`
**Explanation:** This command filters the input VCF file to retain only variants with quality (QUAL) scores of 30 or higher, reducing false positive calls.

### Run association test on genotype data
**Args:** `--genotypes genotypes.vcf --phenotypes phenotypes.txt --test linear`
**Explanation:** This command performs a linear regression association test between genetic markers and continuous phenotypic values, outputting summary statistics.

### Convert BAM to FASTQ format
**Args:** `--input alignments.bam --convert fastq --out reads_converted.fq`
**Explanation:** This command extracts sequences from a BAM file and converts them back to FASTQ format for re-processing with other pipelines.

### Merge multiple VCF files
**Args:** `--input sample1.vcf --merge sample2.vcf --out merged.vcf`
**Explanation:** This command combines two VCF files into a single output file, with overlapping variants handled according to merge rules (default: keep first occurrence).

### Extract reads mapping to a specific genomic region
**Args:** `--input alignments.bam --region chr1:1000000-2000000 --out region_reads.bam`
**Explanation:** This command filters a BAM file to retain only reads that overlap the specified chromosomal region (chr1, positions 1-2 million), useful for targeted analysis.

### Calculate coverage depth across the genome
**Args:** `--input alignments.bam --coverage --out coverage.txt`
**Explanation:** This command computes per-base read depth across the entire input alignment and writes coverage values to the specified output file for visualization or filtering.