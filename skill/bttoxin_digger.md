---
name: bttoxin_digger
category: sequence-analysis
description: A bioinformatics tool for identifying and analyzing bacterial toxin genes and proteins in genomic sequences. It searches input sequences against toxin databases, performs domain analysis, and outputs predictions with confidence scores.
tags: [bacterial-toxins, genomics, sequence-analysis, gene-prediction, toxin-identification]
author: AI-generated
source_url: https://github.com/bttoxin/bttoxin_digger
---

## Concepts

- **Input Formats**: bttoxin_digger accepts FASTA and GenBank files containing nucleotide or protein sequences. For nucleotide inputs, the tool automatically performs translation toprotein space before toxin searching.
- **Toxin Database**: The tool uses an integrated toxin profile database (HMM-based) covering known bacterial toxin families including RTX toxins, cytotoxins, and enterotoxins. Use the `--db-update` flag to refresh the toxin definitions.
- **Output Modes**: Results are provided in tabular format (default), JSON (`--format json`), or GFF3 (`--format gff3`) for integration with annotation pipelines. The tabular output includes gene ID, toxin family, e-value, and bit score.
- **Sensitivity Settings**: Three detection stringency levels are available via `--sensitivity` (low, medium, high). Higher sensitivity increases recall but may produce more false positives; use `--sensitivity low` for confident predictions only.
- **Batch Processing**: Multiple input files can be processed in parallel using `--input-list` with a file containing one FASTA/GenBank path per line, enabling high-throughput toxin screening across genome collections.

## Pitfalls

- **Using Nucleotide Input Without Translation**: When providing nucleotide sequences, ensure the input format is recognized. The tool requires complete ORFs for accurate toxin detection; fragmented nucleotide sequences may yield false negatives. Always validate ORF completeness with a separate tool before running bttoxin_digger.
- **Ignoring E-value Thresholds**: The default e-value cutoff of 1e-5 may miss divergent toxin variants in novel genomes. Adjusting to `--e-value 1e-3` can increase detection but also introduces false positives in regions with low complexity; always manually review borderline predictions.
- **Forgetting Database Updates**: The toxin database ships with the tool but becomes outdated as new toxin families are characterized. Running without `--db-update` for extended periods may cause the tool to miss recently identified toxin types; check for updates before analyzing novel bacterial strains.
- **Insufficient Memory for Large Genomes**: For genome-scale inputs exceeding 10 Mb, the tool requires increased memory allocation via `--memory`. Insufficient memory causes the process to terminate mid-analysis; monitor memory usage and allocate at least 4 GB for bacterial genomes.
- **Incorrect Output Format for Downstream Tools**: Using the default tabular format when interfacing with annotation pipelines that require GFF3 can cause integration failures. Always specify `--format gff3` when piping results into genome browsers or other bioinformatics tools.

## Examples

### Identify toxin genes in a single bacterial genome FASTA file

**Args:** `--input genome.fasta --output toxins.tsv --format tabular`
**Explanation:** This runs toxin gene identification on the input genome FASTA file and writes predictions to a tab-separated file with standard output columns including gene coordinates, toxin family, and alignment metrics.

### Search protein sequences against toxin database with high sensitivity

**Args:** `--input proteins.faa --db-type protein --sensitivity high --output high_sensitivity_results.tsv --e-value 0.001`
**Explanation:** Using high sensitivity and relaxed e-value captures divergent toxin variants but generates more candidates requiring manual curation; appropriate for exploratory analysis of uncharacterized genomes.

### Export results in GFF3 format for genome annotation pipelines

**Args:** `--input contigs.fasta --output toxins.gff3 --format gff3`
**Explanation:** GFF3 output includes genomic coordinates and attributes compatible with genome browsers and annotation tools, enabling direct integration into bacterial genome annotation workflows.

### Update the internal toxin database before analysis

**Args:** `--db-update --db-path /custom/toxin/profiles`
**Explanation:** Updates or replaces the built-in toxin profiles with custom HMM definitions from the specified directory, enabling detection of specialized toxin families not included in the default database.

### Process multiple genomes from an input list file

**Args:** `--input-list genomes.txt --output-dir results/ --threads 8`
**Explanation:** Reads multiple FASTA file paths from the text file and processes them in parallel using 8 threads, outputting individual result files to the specified directory for high-throughput toxin screening.

### Run with increased memory allocation for large bacterial genomes

**Args:** `--input large_genome.fasta --output toxins_large.tsv --memory 8192`
**Explanation:** Allocates 8 GB of memory to handle large genome assemblies (exceeding 10 Mb) without termination, ensuring complete analysis of megabase-scale bacterial chromosomes.

### Filter results to show only high-confidence toxin predictions

**Args:** `--input genome.fasta --output high_conf.tsv --min-score 50 --min-coverage 0.8`
**Explanation:** Applies stringent filtering requiring bit scores above 50 and 80% coverage, outputting only high-confidence predictions and reducing manual review burden for well-characterized toxin families.