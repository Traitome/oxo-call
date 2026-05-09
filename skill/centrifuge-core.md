---
name: centrifuge-core
category: Taxonomic Classification
description: Fast taxonomic classification tool for metagenomic sequencing reads using an FM-index based k-mer database. Classifies DNA sequences against custom databases of bacterial, viral, and archaeal genomes.
tags: metagenomics, taxonomic-classification, genomics, kmers, fastq, fasta
author: AI-generated
source_url: https://github.com/DaehwanKimLab/centrifuge
---

## Concepts

- **Input formats**: Accepts FASTA and FASTQ files (compressed with .gz extension is supported), including single-end and paired-end reads. Input can be piped via stdin using the `-` argument.
- **Output modes**: Two primary output formats exist — a tabular `hits` file showing per-read classification details with sequence ID, taxID, score, and length; and a `report` format that summarizes read counts per taxonomic rank (species, genus, family, etc.) in a format compatible with Krona visualization tools.
- **Database indexing**: Classification requires a pre-built database created by the companion `centrifuge-build` tool. Databases contain FM-index structures and are organized by taxonomic IDs. Multiple custom databases can be concatenated for broader coverage.
- **Classification thresholds**: The `-N` (minimum score) and `-L` (minimum alignment length) parameters control sensitivity. Lower values increase recall but reduce precision by including more ambiguous classifications.
- **Paired-end mode**: The `-1` and `-2` flags process paired-end reads together, allowing the classifier to use read pairing information for improved accuracy when both reads map to the same or related taxa.

## Pitfalls

- **Using an unindexed database**: Attempting classification without first building a database with `centrifuge-build` produces empty results. The error message may be silent, returning zero classifications without explicit failure.
- **Confusing report and hits output**: Specifying `--report` without redirecting to a file prints to stdout and may mix with stderr, making downstream parsing difficult. Always use `-S` (output-path) and `-U` (unmapped-output-path) for proper file handling.
- **Ignoring scoring thresholds for noisy data**: For low-quality or highly contaminated metagenomes, default thresholds may classify reads to only the closest match rather than the true origin, causing false positives in taxonomic abundance estimates.
- **Memory exhaustion with large databases**: Databases containing many reference genomes can exceed available RAM, causing the process to be killed. Use the `--packed` database option or reduce database scope for limited-memory systems.
- **Mismatched read orientations in paired-end mode**: Supplying forward reads in `-2` and reverse reads in `-1` produces nonsensical classifications, as Centrifuge expects reads in their original orientation matching the library preparation.

## Examples

### Classify single-end metagenomic reads against a bacterial database
**Args:** -x nt -U classified_hits.tsv input_reads.fq
**Explanation:** Uses the pre-built nt database (`-x nt`) to classify reads and outputs classified hits to the specified file, allowing downstream taxonomic analysis.

### Generate Krona-compatible taxonomic abundance report
**Args:** -x nt -1 reads_1.fq -2 reads_2.fq --report krona_report.tsv
**Explanation:** Processes paired-end reads and generates a summary report with taxonomic abundances that can be directly visualized using Krona tools for interactive pie charts.

### Filter classifications by minimum alignment length
**Args:** -x custom_db -L 50 -N 150 -S high_confidence_hits.tsv input.fq
**Explanation:** Uses a 50bp minimum alignment length (`-L`) and minimum score of 150 to retain only high-confidence classifications, reducing false positives from short or ambiguous matches.

### Classify reads from a custom viral database
**Args:** -x viral_refseq -S viral_hits.tsv -U unmapped.fq input.fa.gz
**Explanation:** Uses a custom viral reference database (`-x viral_refseq`) and separates successfully classified reads from those that did not match any viral reference.

### Paired-end classification with tab-separated output for scripting
**Args:** -x nt -1 R1.fq.gz -2 R2.fq.gz --tab-table -S table_output.tsv
**Explanation:** Outputs classifications in a simple tab-delimited format suitable for parsing by scripts, with one classification per line for each read pair.