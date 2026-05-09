---
name: clark
category: metagenomics
description: A taxonomic classification tool for metagenomic sequences using discriminative k-mers from a reference database.
tags: [classification, metagenomics, taxonomy, k-mers, binning, microbiota]
author: AI-generated
source_url: https://github.com/phyloMe/CLARK
---

## Concepts

- **Discriminative k-mer classification**: Clark uses k-mer frequency profiles to classify sequences, requiring a pre-built database (created with companion binary `clark-build`) containing reference genomes with their discriminative k-mers.
- **Input format flexibility**: Accepts FASTA, FASTQ, and plain multi-line sequence files as input. Single-end and paired-end read files are supported, with the `-b` option specifying paired-end mode.
- **Classification modes**: Offers three reporting modes—`default` (best hit), `extened` (top matches with confidence), and `full` (all matching references above threshold).
- **Output streams**: Results are written to a tab-delimited file specified with `-o`, and a summary to stdout for quick quality checks. The `-r` flag controls the detailed results output.
- **Sensitivity settings**: The `--d` flag adjusts the discriminative power threshold (default 1.0), allowing trade-offs between specificity and sensitivity based on sequence similarity.

## Pitfalls

- **Missing or incompatible database**: Running classification without a valid database (using `-d` option) will cause immediate failure. The database must be built with `clark-build` and match the reference sequences for your organism of interest.
- **Misconfigured k-mer size**: Using a k-mer size (set at build time with `clark-build` via `-k`) incompatible with the input sequences leads to poor classification accuracy or zero hits.
- **Conflicting input specifications**: Providing both positional input file and `-l` list file simultaneously creates ambiguity and the tool will fail to resolve which input source to process.
- **Insufficient memory for large datasets**: Running with default memory settings on large metagenomes without adjusting `-m` can cause excessive disk swapping or premature termination.
- **Threshold too strict**: Setting the threshold (`--t`) too high may eliminate valid low-confidence matches, resulting in unclassified sequences even when matches exist.

## Examples

### Classify a single metagenomic read file against a database
**Args:** `-d /path/to/database -r results.txt -o output.txt input.fq`
**Explanation:** Runs classification on input FASTQ file using the specified database and writes both detailed results and summary output files.

### Classify paired-end reads from two files
**Args:** `-d /path/to/database -b forward_reads.fq -o output.txt reverse_reads.fq`
**Explanation:** Treats input as paired-end reads and uses both forward and reverse files to improve classification accuracy through read-pair consistency.

### Classify with custom threshold for more sensitive results
**Args:** `-d /path/to/database -o output.txt --t 0.5 input.fq`
**Explanation:** Lowers the threshold to 0.5 instead of default 1.0, reporting matches that would otherwise be filtered as low confidence.

### Classify multiple files using a list file
**Args:** `-d /path/todatabase -l filelist.txt -o summary.tsv`
**Explanation:** Processes multiple input files specified in `filelist.txt` (one file path per line) and writes a combined summary to the output file.

### Use extended reporting mode for detailed confidence scores
**Args:** `-d /path/to/database -o output.txt --mode extended input.fa`
**Explanation:** Requests extended reporting mode which outputs the top matches with their confidence scores, useful for analyzing ambiguous classifications.