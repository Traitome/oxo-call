---
name: centrifuge
category: taxonomic_classification
description: Fast and accurate taxonomic classification tool for metagenomic read assignment using hierarchical graph-based indexing. Classifies DNA sequencing reads against a database of reference genomes or custom sequences, reporting taxonomic lineage and classification scores.
tags: [metagenomics, taxonomic-classification, read-classification, microbial-detection, taxonomy]
author: AI-generated
source_url: https://github.com/DaehwanKimLab/centrifuge
---

## Concepts

- Centrifuge indexes reference genomes using a compact graph representation (P-graph) that enables fast traversal and hierarchical classification. The index consists of multiple files with `.cf`, `.cfi`, `.cfq` extensions, created by the `centrifuge-build` companion binary.

- Classification results are output in multiple formats: tabular (default), BLAST-like tabular (`--blast-tab`), SAM (`-s`), and an optional summary report (`-S` or `--report-file`). The output includes read ID, taxID, score, taxname, and taxonomic lineage in a semicolon-separated hierarchy.

- The tool classifies both single-end (`-1`) and paired-end reads (`-1` / `-2` or `-U` for unpaired). Paired-end mode considers both reads for a single classification decision, improving accuracy for short or ambiguous reads.

- Taxonomic classification uses a minimum score threshold (`--min-score`, default 300) and minimum read length (`--min-read-len`, default 70) to filter low-quality assignments. Increasing `--min-score` reduces false positives at the cost of leaving more reads unclassified.

- Custom taxonomy files (in NCBI taxonomy dump format) can be used alongside reference sequences to build custom classification databases. The taxonomy defines the hierarchical relationships used for per-level abundance summarization.

## Pitfalls

- **Forgetting to build a custom index**: Attempting to classify reads with a database name that doesn't exist will cause a silent failure or confusing file-not-found error. You must first run `centrifuge-build` to create the index files from your reference genomes.

- **Using the wrong input flag for paired-end data**: Using `-1` only for paired-end reads causes centrifuge to treat the second file as unpaired, producing incorrect classifications. Always provide both `-1` and `-2` for true paired-end mode, or concatenate both files and use `-U`.

- **Not specifying output format before redirecting**: The output flag `-o` writes classification results, not the summary. If you want both classification and summary, you must specify both `-o` and `-S` explicitly, otherwise summary goes to stdout.

- **Ignoring unclassified reads in output**: By default, unclassified reads appear in the output with taxID 0. If you need only classified reads, filter with tools like `awk '$2 > 0'` or use the `--report-only` option for summary statistics only.

- **Building an index without taxonomy information**: Passing FASTA sequences to `centrifuge-build` without a valid taxonomy file results in a flat classification hierarchy (no lineage information). For meaningful taxonomic summaries, ensure the taxonomy file is properly formatted and passed via the `-t` flag.

## Examples

### Classify single-end metagenomic reads against a pre-built index

**Args:** `-x nt -1 sample_R1.fastq.gz -S -o classification.tsv`

**Explanation:** Uses the pre-built nt index to classify single-end reads in FASTQ format, outputting both detailed classifications to a file and a summary report to stdout.

### Classify paired-end reads and generate a taxonomic abundance summary

**Args:** `-x database -1 read1.fq -2 read2.fq --report-file report.txt -o results.tsv`

**Explanation:** Classifies paired-end reads by considering both mate information, produces a summary table in `report.txt` suitable for downstream metagenomic analysis, and writes detailed results to `results.tsv`.

### Build a custom index from FASTA genomes with NCBI taxonomy

**Args:** `--verbose -p 8 -t taxonomy.dmp --seed 42 input_genomes.fasta custom_db`

**Explanation:** Creates a custom classification database using 8 threads, taxonomic relationships from the taxonomy dump file for hierarchical classification, and a random seed for reproducibility.

### Increase stringency to reduce false positive classifications

**Args:** `-x nt -1 reads.fq --min-score 500 --min-read-len 100 -o strict_results.tsv`

**Explanation:** Raises the score threshold to 500 and minimum read length to 100bp, producing more reliable classifications at the cost of potentially leaving more reads unclassified.

### Output classifications in BLAST-tab format for easy parsing

**Args:** `-x nt -1 sample.fq --blast-tab -o blast_results.tsv`

**Explanation:** Outputs classification results in a BLAST-like tabular format (12 columns) that integrates easily with other bioinformatics tools expecting standard BLAST output parsing conventions.

### Filter output to keep only classified reads at genus level or better

**Args:** `-x db -1 reads.fq -S | awk -F'\t' '$2 > 0 && $6 >= 1000'`

**Explanation:** Suppresses the summary and processes the output to retain only reads with valid taxIDs (greater than 0) and classification scores of at least 1000, which typically correspond to genus-level or better assignments.

### Run centrifuge on a compressed FASTQ file without explicit decompression

**Args:** `-x nt -1 sample.fastq.gz -o results.tsv`

**Explanation:** Directly reads gzipped FASTQ input without manual decompression, leveraging centrifuge's built-in support for compressed input files to save disk space and preprocessing time.