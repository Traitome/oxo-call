---
name: amplicontyper
category: microbial-typing
description: A bioinformatics tool for amplicon sequence-based typing (AST) of microbial pathogens. It analyzes FASTA or FASTQ amplicon sequences against a database of known alleles to determine the sequence type (ST) of bacterial isolates, commonly used for Mycobacterium tuberculosis complex and other pathogens.
tags: [amplicon, sequence-typing, microbial-typing, phylogenetics, tuberculosis, allele-profiling]
author: AI-generated
source_url: https://github.com/GenomeGraphs/amplicon_typer
---

## Concepts

- **Input Formats**: AmpliconTyper accepts FASTA (single-end) or FASTQ (with quality scores) sequence files containing PCR amplicon sequences. The input sequences must correspond to one or more defined typing loci (e.g., katG, rpoB for TB) used in the allele database.

- **Allele Database Model**: The tool uses a reference database containing known allele sequences indexed by locus name and allele number. Databases are typically distributed as FASTA files with specific headers (e.g., `>katG_1`) or as pre-built binary index files (`.adt` format) for faster loading. You must match your input sequences to the correct loci in the database.

- **Output Classification**: The tool reportssequence types (STs) by exact or near-exact matching of input sequences to database alleles. A minimum sequence identity threshold (default 97-100%) determines matches, and results include the matched allele,identity percentage, and coverage depth.

- **Companion Binaries**: AmpliconTyper includes `amplicontyper-build` for constructing custom allele databases from FASTA reference files. This companion tool must be run separately to create new typing databases before running the main analysis.

## Pitfalls

- **Loci Mismatch**: Providing input sequences from loci not present in the database results in zero matches. Always verify that your amplicon targets (e.g., katG, rpoB, hsp65) are included in the database before running analysis, otherwise all reads will be reported as "no match".

- **Low Identity Threshold**: Setting the identity threshold too low (e.g., below 95%) can cause false-positive type assignments, assigning sequences to incorrect alleles that share only partial similarity. This leads to misclassification of isolates, invalidating downstream epidemiological conclusions.

- **Database Version Drift**: Using an outdated allele database when new alleles have been discovered can result in novel alleles being incorrectly typed as the closest existing match or marked as "unknown". Always use the most current database version for accurate typing, especially for outbreak investigation.

- **Mixed Infection Failure**: The tool assumes a dominant single sequence type per sample. Samples containing mixed infections (two or more strains) will be typed as a single dominant allele, leading to incomplete or incorrect type assignment unless the `--mixed` flag is used.

## Examples

### Typing amplicon sequences using the default database

**