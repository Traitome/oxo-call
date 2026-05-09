---
name: abacat
category: Assembly Analysis / Taxonomic Profiling
description: ABACAT is a bioinformatics tool for bacterial genome assembly analysis, providing taxonomic profiling, coverage estimation, and binning of contigs. It analyzes FASTA assemblies against reference databases to classify sequences and generate assembly quality reports.
tags:
  - assembly
  - taxonomy
  - bacterial-genomics
  - coverage
  - binning
  - contig-classification
author: AI-generated
source_url: https://github.com/ale сотラボ/abacat
---

## Concepts

- **Input Requirements**: ABACAT accepts FASTA/FASTQ files containing assembled contigs or raw reads. For taxonomic assignment, it requires a pre-built reference database (created with `abacat-build`) containing known bacterial genomes or marker genes.
- **Taxonomic Profiling**: The tool assigns taxonomic labels to each contig by searching against the reference database using approximate matching algorithms, reporting the best hit above a user-defined identity threshold (default 95%).
- **Output Formats**: ABACAT produces a tab-delimited report listing each contig with its assigned taxonomy, alignment metrics (identity, coverage, e-value), and taxonomic lineage. Summary statistics are printed to stdout.
- **Database Dependency**: Accurate taxonomic assignment depends critically on the quality and comprehensiveness of the reference database. Custom databases should include representative taxa for the expected sample types.
- **Companion Binary**: `abacat-build` creates the reference database from FASTA files of known genomes, building indices for fast subsequence search during taxonomic assignment.

## Pitfalls

- **Low Identity Threshold**: Setting `--identity` too low (e.g., below 90%) causes misclassification of contigs to wrong taxa, especially in regions of conserved genes that share similarity across genera.
- **Database Contamination**: Using a reference database with poorly annotated or mislabeled sequences leads to systematic errors in taxonomic assignment, propagating incorrect classifications throughout the output.
- **Memory Limits**: Large databases or assemblies cause memory exhaustion if insufficient RAM is allocated. Monitor resource usage and consider fragmenting large assemblies into smaller batches.
- **Short Contig Filtering**: Very short contigs (below the minimum length threshold) are discarded by default, potentially removing ecologically important rare taxa present only in small fragments.
- **Database Version Mismatch**: Using an outdated reference database for taxonomic profiling yields inaccurate results when analyzing samples containing newly described species or strains absent from older builds.

## Examples

### Taxonomic profiling of a bacterial genome assembly
**Args:** `input.fasta --db ref_database --out taxonomy_report.tsv`
**Explanation:** This runs taxonomic assignment of all contigs in the input assembly against the built reference database, writing detailed per-contig assignments to the output file.

### Adjusting the minimum identity threshold for relaxed matching
**Args:** `assembly.fasta --db custom_db --identity 85 --out results.tsv`
**Explanation:** Lowering the identity threshold to 85% allows more permissive matches, useful for highly divergent strains but risking false positive classifications.

### Building a custom reference database from GenBank genomes
**Args:** `--build --fasta-dir ./bacterial_genomes/ --db custom_ref --taxonomy taxonomy_mapping.txt`
**Explanation:** This companion command creates a searchable index from FASTA files in the directory, incorporating taxonomy labels for downstream classification.

### Filtering output to genus-level classification only
**Args:** `contigs.fasta --db refdb --rank genus --out genus_report.tsv`
**Explanation:** Using the rank filter restricts output to genus-level taxonomy assignments, simplifying downstream analysis when species-level resolution is unreliable.

### Processing multiple assemblies in batch mode
**Args:** `--batch assembly_directory/ --db refdb --outdir ./results/ --threads 8`
**Explanation:** Batch mode processes all FASTA files in the directory in parallel using 8 threads, suitable for metagenomic datasets with many samples.

### Excluding unclassified contigs from the output report
**Args:** `sample.fasta --db refdb --min-identity 90 --no-unclassified --out filtered.tsv`
**Explanation:** The no-unclassified flag excludes contigs that fail to match above the identity threshold, producing a report with only confidently classified sequences.

### Analyzing coverage along taxonomic profiles
**Args:** `assembly.fasta --bam alignment.bam --db refdb --coverage --out cov_report.tsv`
**Explanation:** When provided with a BAM alignment file, abacat computes per-taxum coverage statistics, useful for identifying dominant species in metagenomic samples.