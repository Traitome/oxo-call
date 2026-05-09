---
name: campygstyper
category: Microbial Genomics / Strain Typing
description: A command-line tool for identifying and typing Campylobacter jejuni and C. coli isolates from genomic data using MLST (Multi-Locus Sequence Typing) and cgMLST (core genome MLST) schemes. Supports assembly-based and read-based typing with allele detection and novel allele discovery modes.
tags: [campylobacter, mlst, cgmlst, strain-typing, bacterial-typing, genomics, microbiology, epidemiology]
author: AI-generated
source_url: https://github.com/campylobacter-typing/campygstyper
---

## Concepts

- **MLST Scheme Support**: campygstyper uses PubMLST naming conventions for C. jejuni (7 housekeeping genes: aspA, glnA, gltA, glyA, pgm, tkt, uncA) and C. coli (7 housekeeping genes: aspA, glnA, gltA, glyA, pgm, tkt, uncA). The tool automatically detects the species and applies the appropriate scheme, but manual overrides are supported via the `--species` flag.

- **Input Flexibility**: The tool accepts FASTA/FASTQ assemblies for assembly-based typing, or unpaired FASTQ reads for read-based typing using de novo allele extraction. GZIP-compressed inputs are automatically detected by the `.gz` extension.

- **Allele Numbering**: Novel alleles are assigned sequential numbers starting from the maximum existing allele number in the database (e.g., if aspA-87 exists, a novel variant becomes aspA-88). This ensures compatibility with global MLST databases.

- **Output Formats**: Primary output is a tab-delimited file with sequence type (ST), allelic profile, and metadata. The `--json` flag produces machine-parseable JSON output, while `--csv` enables integration with spreadsheets or R pipelines.

## Pitfalls

- **Contaminated or Draft Assemblies**: Using highly fragmented draft assemblies can lead to missing alleles (reported as '?' or 'N'), resulting in an unknown ST instead of the correct type. This happens when the contig containing the target gene is too short or absent from the assembly.

- **Species Misidentification**: Running campygstyper without specifying `--species` and using a non-Campylobacter genome will produce meaningless or incorrect STs. The tool attempts auto-detection but may fail on highly divergent inputs, giving incorrect allelic profiles.

- **Database Desync**: Running with an outdated local database missing newer alleles can cause novel alleles to be assigned incorrect numbers, creating conflicts when sharing results with collaborators using updated databases. Always verify database version with `--version` before analysis.

- **Memory Limits on Large Read Sets**: Read-based typing with large FASTQ files (>10GB) may consume excessive memory, causing termination on resource-constrained systems. Using pre-assembled contigs is more memory-efficient for large batches.

## Examples

### Typing a Campylobacter genome assembly
**Args:** input.fasta --species jejuni
**Explanation:** Performs MLST on a pre-assembled FASTA file for C. jejuni, extracting the 7 housekeeping gene alleles and reporting the sequence type. Use this for finished genomes or high-quality assemblies.

### Typing from Illumina paired-end reads
**Args:** reads_R1.fastq.gz reads_R2.fastq.gz --output typing_results.tsv
**Explanation:** Performs read-based allele extraction using de novo assembly of the reads, then identifies the ST. The tool assembles reads for each locus independently to recover alleles from short reads. Use when no assembly is available.

### Exporting results in JSON format
**Args:** input.fasta --json --output results.json
**Explanation:** Exports the typing results as JSON, including the full allelic profile, ST, species, and quality metrics. Use this for integration with automated pipelines or web services.

### Discovering novel alleles in a novel isolate
**Args:** novel_isolates/ --find-novel --db campylobacter_db/
**Explanation:** Processes multiple files in the input directory, identifying and numbering novel alleles not present in the database. Novel alleles are logged with their DNA sequences for submission to PubMLST. Use for epidemiological outbreak investigation of unknown strains.

### Using a custom cgMLST scheme
**Args:** input.fasta --scheme cgMLST --min-coverage 95
**Explanation:** Performs core genome MLST instead of traditional 7-gene MLST, requiring at least 95% of the 534 cgMLST loci to be present. Use this for higher-resolution typing in closely related isolates.