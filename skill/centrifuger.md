---
name: centrifuger
category: taxonomic-classification
description: A fast and memory-efficient centrifugal algorithm-based tool for taxonomic classification of DNA sequences from metagenomic samples. Classifies query reads against a database of bacterial, archaeal, and viral genomes using a compressed index and reports taxonomy assignments.
tags: [metagenomics, taxonomic-classification, k-mer-index, DNA-sequencing, microbiome-analysis]
author: AI-generated
source_url: https://github.com/DaehwanKimLab/centrifuer
---

## Concepts

- **Index Architecture**: Centrifuer uses a pre-built compressed index (`.cf` files) containing k-mer signatures mapped to NCBI taxonomy IDs. The index is created from complete genomes in RefSeq and must be built or downloaded before classification runs. Without a valid index, classification will fail with a file-not-found error.

- **Input/Output Formats**: Input accepts FASTQ or FASTA files (single or paired-end via `-1/-2` flags) with standard quality encoding. Output defaults to a tab-delimited format reporting query name, taxonomy ID, hit length, and abundance; alternate output modes include SAM-style alignment or a summary table of taxonomic abundance across samples.

- **Classification Scoring**: Each read is classified by finding k-mer matches in the index, resolving ambiguous hits via a weighted scoring scheme that prefers lower common ancestor scores, and reporting both the best hit and optionally all hits above a minimum score threshold. The `--min-sum-len` and `-F` (filter flags) parameters control which matches are retained.

- **Threading and Performance**: Multi-threaded execution is controlled via `-p/--threads` and scales linearly for independent read batches. Memory footprint depends on index size; large indexes (bacteria+virus) may require 32–64 GB RAM. Using the `-S` (output all hits) flag significantly increases I/O and memory usage.

## Pitfalls

- **Missing or Incompatible Index**: Running `centrifuger` without providing a valid index path (`-x`) results in immediate failure. Pre-built indexes must match the exact Centrifuer version; using an index built with a different version causes silent errors or incorrect classifications. Always verify index integrity with `centrifuer-inspect` before large runs.

- **Improper Paired-End Read Handling**: Passing both `-1` and `-2` for paired-end files without ensuring reads are properly ordered and interleaved correctly produces fragmented or missing classifications. If mate pairs are out of sync, use `centrifuer` in single-end mode or preprocess with `fastq-pair` to synchronize input files.

- **Ignoring Classification Quality Thresholds**: Without setting `--un`, `--al`, or `-F` filters, low-complexity or ambiguous reads receive spurious taxonomy assignments that inflate false positives in downstream analysis. Always set appropriate `--min-sum-len` values and review the `ambig` column in output to identify and handle ambiguous classifications.

- **Output File Overwrites**: Redirecting stdout to a file (`> output.tsv`) without checking for existing files causes silent data loss when runs are repeated. Always use explicit `-S/--out` arguments with unique filenames or enable append mode explicitly; Centrifuer does not prompt before overwrite.

## Examples

### Classify single-end FASTQ reads against a pre-built index
**Args:** `-x humann_db/cfx humann_sample_R1.fastq.gz -S classification_output.tsv`
**Explanation:** This runs basic single-end classification using the `humann_db/cfx` index, writing results to `classification_output.tsv`. Omitting `-F` retains all hits above default thresholds.

### Classify paired-end reads with multiple threads
**Args:** `-x refseq_cfx -1 left_reads.fastq.gz -2 right_reads.fastq.gz -p 8 -S paired_output.tsv`
**Explanation:** Enables parallel processing across 8 threads for paired-end input, substantially reducing runtime on multi-core systems. Each thread processes independent read batches independently.

### Filter output to high-confidence classifications only
**Args:** `-x cfx_index --min-sum-len 50 -F 0 -S filtered.tsv input.fastq.gz`
**Explanation:** Sets a minimum k-mer match length of 50 and `-F 0` to discard reads with zero classifications, producing a cleaner output file with fewer ambiguous or spurious hits.

### Output all taxonomy hits per read for downstream analysis
**Args:** `-x cfx -a -S all_hits.tsv sample.fastq.gz`
**Explanation:** The `-a` flag reports all hits above threshold for each read rather than just the best hit, which is required for abundance estimation or strain-level analysis workflows.

### Generate SAM-style output for genome alignment visualization
**Args:** `-x cfx_index -u unmapped.fastq -U aligned.sam input.fastq.gz`
**Explanation:** Writes aligned reads in SAM format to `aligned.sam` and unmapped reads to `unmapped.fastq`, enabling direct use in alignment viewers like IGV for manual inspection of classification results.