---
name: bold-identification
category: Bioinformatics - Species Identification
description: A DNA barcode identification tool that matches query sequences against the BOLD (Barcode of Life Data Systems) reference database to identify species based on COI and other DNA barcode markers.
tags: [dna-barcode, species-identification, COI, molecular-taxonomy, bold, bioinformatics]
author: AI-generated
source_url: https://www.boldsystems.org/
---

## Concepts

- **DNA Barcode Matching**: The tool compares query DNA sequences (typically COI gene fragments) against reference sequences in the BOLD database using sequence similarity algorithms to assign taxonomic identifications.
- **Input Formats**: Accepts FASTA-formatted DNA sequences as primary input; sequences should contain the cytochrome oxidase I (COI) gene region for optimal identification accuracy.
- **Confidence Scoring**: Returns identification results with percentage similarity scores and E-values, indicating the reliability of theSpecies assignment based on database matches.
- **Database Connectivity**: Requires internet connectivity to query the BOLD remote database; offline mode uses locally cached reference datasets when available.
- **Output Options**: Supports multiple output formats including TSV, CSV, and JSON for downstream bioinformatics analysis pipelines.

## Pitfalls

- **Insufficient Sequence Length**: Sequences shorter than 500bp often produce unreliable identifications due to inadequate discriminatory power in the COI region.
- **Non-Standard Marker Genes**: Using sequences from genes other than COI (such as rbcL or matK for plants) will result in failed or incorrect identifications since the BOLD database is primarily built on COI.
- **Database Version Mismatch**: Using an outdated local database cache can lead to missing newer species entries, causing false negative identifications for recently described species.
- **Ambiguous Results**: Sequences with multiple equally similar matches (low sequence divergence) produce ambiguous identifications that require manual expert verification.
- **Missing Reference Data**: Querying sequences from geographic regions or taxonomic groups poorly represented in BOLD will yield low-confidence or no matches regardless of sequence quality.

## Examples

### Identify species from a FASTA file containing COI sequences
**Args:** `-i sequences.fasta -o identification_results.tsv`
**Explanation:** This runs identification on all sequences in the input FASTA file and writes results to a tab-separated file for easy parsing.

### Identify species with verbose output showing all database matches
**Args:** `-i query.fasta --Verbose --maxHits 10`
**Explanation:** Returns the top 10 database matches for each query with detailed alignment information for verification.

### Output results in JSON format for pipeline integration
**Args:** `-i sequences.fasta -o results.json --format json`
**Explanation:** Outputs machine-readable JSON format suitable for automated pipelines and downstream bioinformatics processing.

### Specify a particular marker gene for identification
**Args:** `-i input.fasta -m COI --similarity 98`
**Explanation:** Sets the marker gene to COI and only accepts identifications with 98% or higher similarity to reference sequences.

### Use a specific taxonomy file to restrict identification
**Args:** `-i sequence.fasta --taxonomyFile species_list.txt --outputAll`
**Explanation:** Restricts identifications to species in the provided taxonomy file and outputs all results including non-matches.

### Generate a summary report of all identifications
**Args:** `-i sequences.fasta --summary --outfile report.txt`
**Explanation:** Creates a compiled summary report containing statistics on identification success rates across all input sequences.

### Process sequences with specific database parameters
**Args:** `-i input.fasta -d bold_ref --minScore 500 --eValue 1e-10`
**Explanation:** Filters results to only include matches with a minimum score of 500 and E-value better than 1e-10 for increased stringency.

### Identify using a custom FASTA file with sequence names as species hypotheses
**Args:** `-i queries.fasta -v --batchSize 100`
**Explanation:** Processes sequences in batches of 100 to manage memory usage for large datasets while verifying query names.

### Run identification requiring exact species-level matches
**Args:** `-i dataset.fasta --strict --speciesLevel --output match_results.tsv`
**Explanation:** Only returns species-level identifications (rejects genus-level matches) with strict criteria for maximum accuracy.