---
name: amrfior
category: Antimicrobial Resistance Detection
description: A bioinformatics tool for detecting and analyzing antimicrobial resistance (AMR) genes in bacterial sequences. amrfior scans input sequences against curated AMR gene databases to identify known resistance determinants and predict phenotypic resistance profiles.
tags:
  - amr
  - antimicrobial
  - resistance
  - gene-detection
  - microbiology
  - pathogen-analysis
author: AI-generated
source_url: https://github.com/username/amrfior
---

## Concepts

- **Input formats**: amrfior accepts FASTA (single or multi-sequence) and FASTQ files containing nucleotide sequences from bacterial genomes, contigs, or reads. Sequences must be in standard FASTA/FASTQ format with valid headers.
- **Database matching**: The tool performs local alignment or exact matching of input sequences against a built-in or user-provided AMR gene database. Each gene in the database has associated metadata including antibiotic class, resistance mechanism, and detection confidence thresholds.
- **Output formats**: Results are provided in text summary, TAB-delimited tables, or JSON for downstream integration. The output includes gene name, coverage, identity percentage, antibiotic class, and resistance mechanism.
- **Confidence scoring**: Each detection is assigned a confidence score based on sequence coverage, identity percentage, and database entry quality. Thresholds can be adjusted to balance sensitivity and specificity.

## Pitfalls

- **Ignoring coverage and identity thresholds**: Setting coverage below 80% or identity below 90% may produce false positives from partial or divergent homologs that lack actual resistance function, leading to incorrect resistance phenotype predictions.
- **Using outdated AMR databases**: Running analyses with default databases that have not been updated may miss recently emerging resistance genes (e.g., mcr-1, colistin resistance) or include obsolete gene annotations, reducing detection accuracy.
- **Not specifying species background**: Failing to provide species context can cause false positives from intrinsic genes that are native to certain species (e.g., chromosomal beta-lactamases in Enterobacter) rather than acquired resistance.
- **Overlooking frame-shift mutations**: Some resistance genes require specific mutations to confer resistance; detecting only the gene backbone without checking critical mutations may misreport susceptibility.

## Examples

### Detect AMR genes in a bacterial genome assembly
**Args:** -i assembly.fasta --db amr_db.fasta --out results.txt
**Explanation:** Scans a complete bacterial genome assembly against the AMR gene database and saves gene detections to a text file.

### Run with custom database and JSON output
**Args:** -i contigs.fa --db custom_amr.fasta --json --out results.json
**Explanation:** Uses a user-curated AMR database and outputs results in JSON format for programmatic downstream analysis.

### Adjust detection stringency for lenient analysis
**Args:** -i reads.fastq --db amr_db.fasta --min-coverage 70 --min-identity 80 --out lenient.txt
**Explanation:** Loweres detection thresholds to catch divergent resistance genes in highly variable sequences, useful for novel variants.

### Filter output by antibiotic class
**Args:** -i input.fa --db amr_db.fasta --class "fluoroquinolones" --out fluoroquinolone_genes.txt
**Explanation:** Restricts output to only genes conferring fluoroquinolone resistance, filtering out unrelated detections.

### Generate summary report with verbose output
**Args:** -i genome.fa --db amr_db.fasta --verbose --summary --out report.txt
**Explanation:** Produces a detailed report including database match coordinates, alignment details, and per-gene confidence metrics.