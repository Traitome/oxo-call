---
name: biophi
category: Protein Sequence Analysis
description: A bioinformatics tool for analyzing antibody sequences, calculating humanness scores, predicting biophysical properties, and assessing developability of therapeutic proteins. Supports batch processing and generates detailed reports for protein engineering workflows.
tags:
  - antibody
  - humanization
  - protein engineering
  - biophysics
  - developability
  - therapeutic proteins
author: AI-generated
source_url: https://github.com/nick11jr/biophi
---

## Concepts

- **Input Format**: BioPhi accepts FASTA files (single or multiple sequences) and CSV files with sequence columns. Each sequence entry should have a unique identifier prefixed with '>' for FASTA format, and plain amino acid letters for CSV import.
- **Humanness Scoring**: The tool calculates a humanness score (0-100 scale) by comparing input sequences against human and mouse antibody databases. Higher scores indicate more human-like sequences, which is critical for reducing immunogenicity in therapeutic antibody development.
- **Subcommands**: BioPhi uses modular subcommands including `score` for humanness evaluation, `analyze` for biophysical property prediction (isoelectric point, hydrophobicity, stability), and `batch` for processing multiple sequences simultaneously.
- **Output Formats**: Results can be exported as CSV (default), JSON, or human-readable text reports. The CSV includes per-sequence metrics plus summary statistics; JSON provides programmatic access for pipeline integration.
- **Reference Databases**: BioPhi maintains built-in reference databases for human IgG and murine IgG sequences. Custom reference sets can be added using the companion `biophi-build` command to create custom comparison databases.

## Pitfalls

- **Non-Standard Characters**: Including non-standard amino acid symbols (like 'X', 'B', 'Z', 'J' in input sequences) will cause scoring errors or produce invalid humanness scores, as these ambiguity codes are not recognized in the reference alignment.
- **Sequence Length Limits**: Input sequences shorter than 50 residues or longer than 700 residues may produce unreliable humanness scores due to insufficient overlap with reference database patterns, leading to misleading developability assessments.
- **Database Version Mismatch**: Using outdated reference databases with new BioPhi versions causes score inflation or deflation. Always verify database versions match the BioPhi installation date to ensure consistent comparative metrics.
- **Case Sensitivity**: Entering amino acid sequences in lowercase instead of uppercase will fail parsing validation, as BioPhi expects uppercase single-letter codes. This results in empty output files with no error message displayed.

## Examples

### Calculate humanness score for a single antibody sequence
**Args:** score -i sequences.fa -o results.csv
**Explanation:** Reads a FASTA file containing an antibody variable region sequence and outputs a humanness score between 0-100 to the specified CSV file.

### Analyze biophysical properties of multiple antibody sequences
**Args:** analyze -i batch_sequences.fasta --properties pI,solubility,stability -o biophys_report.csv
**Explanation:** Processes multiple antibody sequences in batch mode, calculating isoelectric point, predicted solubility scores, and thermal stability estimates for each entry.

### Export results in JSON format for pipeline integration
**Args:** score -i monoclonal_antibody.fa -o pipeline_output.json --format json
**Explanation:** Generates JSON-formatted output containing humanness score, alignment details, and reference database match percentages for automated downstream processing.

### Build a custom reference database from proprietary antibody sequences
**Args:** build -i internal_antibodies.fasta --species rabbit -o custom_rabbit_db
**Explanation:** Uses the companion biophi-build tool to create a custom reference database from user-provided rabbit antibody sequences, enabling species-specific humanness comparisons.

### Run batch processing with parallel threads for large datasets
**Args:** batch -i large_dataset.fa -o batch_results.csv --threads 8 --chunk-size 100
**Explanation:** Processes a large FASTA file with 8 parallel threads, grouping sequences in chunks of 100 to optimize memory usage while maintaining processing speed.

### Generate human-readable report with detailed alignment visualization
**Args:** score -i test_antibody.fa -o detailed_report.txt --format txt --show-alignments
**Explanation:** Produces a text report showing the humanness score alongside visual alignment highlights showing which framework and CDR regions contribute most to the overall score.