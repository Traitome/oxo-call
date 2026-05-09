---
name: cmash
category: genomics/phyloinformatics
description: Core-genome MASH (CMash) estimates average nucleotide identity (ANI) and related measures between microbial genomes using MinHash sketches. It is optimized for core-genome comparisons rather than pairwise whole-genome ANI, providing faster computation with statistically rigorous confidence intervals.
tags: [ANI, microbial, genomics, MinHash, phylogeny, genome-comparison, core-genome]
author: AI-generated
source_url: https://github.com/Marbl-Lab/cmash
---

## Concepts

- **MinHash Sketch Data Model**: CMash represents genomes as MinHash sketches—fixed-size compressed representations of k-mer sets. Smaller sketch sizes (e.g., -k 21 -s 1000) provide faster computation but wider confidence intervals; larger sketches (e.g., -s 10000) give narrower intervals but require more memory and compute time.
- **Core-Genome ANI vs. MASH ANI**: CMash computes "core-genome ANI" by restricting comparisons to k-mers present in both genomes, making it more robust to accessory genome content and horizontally transferred elements than standard MASH ANI which uses all shared k-mers.
- **Input Format Flexibility**: CMash accepts FASTA, FASTQ, and GenBank (.gbk, .gbff) files as input. For FASTQ files, quality scores are ignored. Input can be specified as file paths or URLs ending in .fasta/.fastq/.fa/.fq/.gz.
- **Database Workflow**: The typical workflow involves two steps: (1) build a database of reference genomes using the companion binary, then (2) evaluate query genomes against that database to obtain ANI estimates with Jaccard dissimilarity, confidence intervals, and p-values.
- **Statistical Output**: For each query-reference pair, CMash reports the estimated ANI (as fraction 0-1 or percentage), lower and upper bounds of the confidence interval (defaults to 95%), and a p-value testing whether the observed overlap differs from random expectation.

## Pitfalls

- **Confusing cmash with MASH**: CMash is a distinct tool from MASH (the original MASH tool). They use different algorithms, output formats, and have incompatible sketch files (.cmash.msh vs .msh). Do not try to use MASH sketches with CMash or vice versa.
- **Insufficient Sketch Size for Closely Related Genomes**: When comparing very closely related strains (e.g., >99% ANI), using default sketch parameters (-s 1000) may produce excessively wide confidence intervals. Increase -s to at least 5000 for fine-grained strain-level discrimination.
- **Reference Database Not Pre-built**: Running `cmash-evaluate` without first building a reference database will fail. Always run `cmash-build` to create the .cmash.msh database before evaluation.
- **Inconsistent k-mer Size Across Steps**: The k-mer size (-k) must be identical when building the database and evaluating queries. Using different k-mer sizes will produce meaningless or NaN ANI values.
- **Memory Usage with Large Databases**: Building a database from thousands of reference genomes with large sketch sizes can require tens of gigabytes of RAM. Monitor memory usage and consider splitting large databases into chunks if memory is limited.

## Examples

### Build a sketch database from a directory of reference FASTA files
**Args:** `-k 21 -s 5000 input_dir/*.fasta database.cmash.msh`
**Explanation:** Creates a pre-computed sketch database from all .fasta files in the directory using k-mer size 21 and 5000 sketches per genome, enabling fast subsequent queries.

### Build a database from GenBank files
**Args:** `-k 31 -s 2000 input_dir/*.gbk ref_db.cmash.msh`
**Explanation:** Uses GenBank (.gbk) files which contain annotated genome sequences; k-mer size 31 is standard for bacterial genomes, and 2000 sketches balances speed and accuracy.

### Evaluate a query genome against a pre-built database
**Args:** `-k 21 -s 5000 query.fasta ref_database.cmash.msh output.tsv`
**Explanation:** Computes ANI between the query genome and all reference genomes in the database, outputting results to a tab-separated file with ANI estimates and confidence intervals.

### Adjust confidence interval width by changing sketch size
**Args:** `-k 21 -s 10000 query.fasta ref_database.cmash.msh output.tsv`
**Explanation:** Increasing sketch size from default 5000 to 10000 produces narrower confidence intervals for more precise ANI estimates, at the cost of slower computation.

### Limit output to top hits only using a p-value threshold
**Args:** `-k 21 -s 5000 --pvalue 0.05 query.fasta ref_database.cmash.msv output.tsv`
**Explanation:** Filters results to only include comparisons where the p-value is less than 0.05, removing statistically insignificant matches from the output.

### Use a custom output prefix for multiple queries
**Args:** `-k 21 -s 3000 --output-prefix pairwise_results query_dir/*.fasta pairwise_ref.cmash.msh`
**Explanation:** Evaluates multiple query files in batch against a reference database, producing individual output files prefixed with "pairwise_results" for each query.

### Run with URLs as input instead of local files
**Args:** `-k 21 -s 2000 https://example.com/genomes/*.fasta remote_db.cmash.msh`
**Explanation:** CMash can fetch FASTA files directly from HTTP/HTTPS URLs, useful for building databases from online genome repositories without manual download.