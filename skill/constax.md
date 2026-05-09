---
name: constax
category: fungal-taxonomy
description: A tool for taxonomic classification of fungal ITS sequences using constrained clustering and phylogenetic placement approaches.
tags: [fungal, ITS, taxonomy, classification, microbiome, amplicon]
author: AI-generated
source_url: https://github.com/GitLong/ConStax
---

## Concepts

- ConStax classifies fungal ITS sequences by matching query sequences against a reference database using constrained clustering, assigning taxonomy with confidence scores based on similarity thresholds and phylogenetic placement.
- The primary input format is FASTA (or FASTQ with adapter removal) containing ITS1 or ITS2 region sequences, and outputs a tab-delimited classification file with OTU IDs, taxonomy strings, and confidence values.
- The tool requires a pre-built UNITE-formatted UDB database (custom or standard) for reference matching; users can generate custom databases with the constax-udb companion tool for project-specific taxonomic frameworks.
- Confidence scoring is threshold-based (default 0.5) and determines whether a sequence is classified to a specific taxonomic rank or marked as unclassified/uncertain.
- Output files include a main result table (taxonomic assignments) and supporting files (score matrices, log files) used for downstream ecological or biodiversity analyses.

## Pitfalls

- Using untrimmed raw sequencing reads instead of cleaned ITS amplicons causes misclassification because adapter sequences and poor-quality ends introduce spurious matches in the reference database.
- Setting the confidence threshold too low (e.g., below 0.3) results in over-confident assignments to incorrect taxa, while setting it too high (e.g., above 0.9) leaves many valid sequences unclassified, skewing community composition estimates.
- Providing a mismatched or outdated UNITE database version causes systematic taxonomic errors; the database version must correspond to the fungal group under study (e.g., general FASTA release for broad surveys).
- Not specifying the correct database path with `--db` causes the tool to fail silently or use a default database, leading to unintended taxonomic assignments from an inappropriate reference set.
- Ignoring the output log warnings about low-coverage matches means legitimate sequences with ambiguous taxonomy are being forced into incorrect clades, corrupting downstream diversity metrics.

## Examples

### Classify ITS sequences using a standard UNITE database
**Args:** `-i sequences.fasta --db unite_v9.udb --conf 0.5 --output classified`
**Explanation:** Specifies the input FASTA, points to the UNITE UDB database, and requests moderate confidence threshold to balance classification rate and accuracy.

### Generate a custom UDB database from a FASTA reference set
**Args:** `-i custom_fungi.fasta --db custom_db --utax --format udb`
**Explanation:** Converts a custom FASTA reference file into a ConStax-compatible UDB database using the UTaX-style taxonomy format for project-specific classification.

### Classify with a high confidence threshold for conservative results
**Args:** `-i reads.fasta --db unite_dynamic.udb --conf 0.8 --output high_conf_results`
**Explanation:** Uses a high confidence cutoff to minimize false-positive taxonomic assignments when analyzing ecologically sensitive or rare taxa.

### Classify using custom database with verbose output for debugging
**Args:** `-i sample.fa --db my_fungi.udb --conf 0.5 --outfmt sin -v --log debug.log`
**Explanation:** Enables verbose and sin output formats to troubleshoot classification issues and track database matching details for quality control.

### Batch classify multiple files and merge results
**Args:** `-i batch_input/ --db unite_v9.udb --conf 0.5 --output batch_classified`
**Explanation:** Processes a directory of multiple FASTA files simultaneously, generating individual classification outputs that can be merged for meta-analysis across samples.