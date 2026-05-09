---
name: ccmetagen
category: Taxonomic classification
description: A k-mer based tool for taxonomic classification of metagenomic sequencing reads. Assigns taxonomic labels to short reads using oligonucleotide frequencies and a reference database.
tags: [metagenomics, taxonomy, classification, k-mer, microbial]
author: AI-generated
source_url: https://github.com/GenomicaMicrobiana/ccmetagen
---

## Concepts

- **Input Format**: ccmetagen accepts FASTQ files (single-end or paired-end) containing short sequencing reads. Quality scores are used for optional read filtering before classification.
- **K-mer Classification Engine**: The tool uses oligonucleotide (k-mer) frequency signatures to classify reads taxonomically. Default k-mer size is 4 (tetranucleotides), which provides optimal discrimination for microbial genomes.
- **Reference Database**: Classification requires a pre-built database of known organism k-mer profiles. Databases are created using the companion tool `ccmetagen-build` from FASTA genome files.
- **Output Formats**: Results are delivered in TXT, CSV, or JSON format, containing read ID, assigned taxonomy (kingdom to species levels), and confidence scores.
- **Confidence Thresholding**: Reads below the minimum confidence score are labeled as "unclassified" rather than assigned to a taxonomy, reducing false positive rates.

## Pitfalls

- **Missing Database**: Running ccmetagen without specifying a database (`-d` flag) causes immediate failure with no taxonomic assignments. Always verify database path before execution.
- **Incorrect K-mer Size Mismatch**: Using a database built with a different k-mer size than specified at runtime produces meaningless or all-unclassified results. Ensure `-k` value matches the database creation parameter.
- **Oversized Input Files Without Memory Adjustment**: Large FASTQ files may cause memory exhaustion if system RAM is insufficient. Use the `-mem` flag to limit memory usage, though this may increase runtime.
- **Paired-End Files Mismatched**: Providing mismatched R1/R2 files (different read counts or order) leads to classification errors or crashes. Verify read pairing before execution.
- **Ignoring Unclassified Reads**: Treating all reads as successfully classified inflates diversity estimates. Always review the "unclassified" proportion in output to assess classification quality.

## Examples

### Classify a single-end FASTQ file using a reference database
**Args:** -i reads.fastq -d refdb.kmer -o classification.txt -t 8
**Explanation:** Classifies single-end reads from reads.fastq against the reference database, writing results to classification.txt with 8 threads for parallel processing.

### Classify paired-end reads with custom k-mer size
**Args:** -i1 sample_R1.fastq -i2 sample_R2.fastq -d refdb.kmer -k 6 -o paired_results.txt
**Explanation:** Uses paired-end read classification with 6-mers (hexamers) instead of the default 4-mers, providing finer taxonomic resolution for closely related species.

### Run classification with quality filtering and output statistics
**Args:** -i reads.fastq -d refdb.kmer -o output.txt -q 20 -s stats.json
**Explanation:** Filters reads with quality scores below 20 before classification and outputs detailed statistics including classification rates to stats.json.

### Limit memory usage for large datasets
**Args:** -i huge_dataset.fastq -d refdb.kmer -o results.txt -mem 4G -t 4
**Explanation:** Limits ccmetagen to 4GB RAM and uses 4 threads when processing a large dataset to prevent system resource exhaustion.

### Output results in CSV format with species-level assignments
**Args:** -i reads.fastq -d refdb.kmer -o results.csv -f csv -l species
**Explanation:** Outputs classification results in CSV format with species-level taxonomy labels, facilitating downstream diversity analysis in spreadsheet software.