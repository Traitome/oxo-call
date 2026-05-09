---
name: astalavista
category: sequence-analysis
description: Finds EST (Expressed Sequence Tags) fragments belonging to gene families through local alignment against protein sequences. Identifies conserved regions and classifies ESTs based on alignment to known gene family members.
tags: EST, gene-family, local-alignment, sequence-analysis, bioinformatics
author: AI-generated
source_url: https://bioinformatics.org/astalavista
---

## Concepts

- Astalavista performs local alignment (using BLAST-like algorithm) of query EST sequences against a set of protein sequences representing known gene family members, identifying which ESTs belong to specific protein families.
- The tool requires two primary input files: a protein database file (in FASTA format) containing reference gene family members, and a query EST file (in FASTA format) to be classified.
- Results can be filtered by setting an e-value threshold (using the `-e` flag) to control alignment significance — lower values (e.g., 1e-10) return only high-confidence matches while higher values (e.g., 1e-3) include more tentative alignments.
- Output formats include plain text alignment results, tabular format for parsing, and XML for integration with other analysis pipelines — control this with the `-o` and `-m` flags.
- A companion binary `astalavista-build` must be used to format the protein reference database before running alignments (similar to BLAST database formatting).

## Pitfalls

- Running `astalavista` without first building the protein database with `astalavista-build` will cause the tool to fail with a database format error, wasting computation time and requiring a restart.
- Setting the e-value threshold too high (e.g., greater than 1e-3) will include alignments with low statistical significance, leading to false-positive EST classifications in downstream analysis.
- Using low-quality or contaminated EST sequences as input will produce unreliable alignments, causing incorrect gene family assignments and potentially invalid research conclusions.
- Forgetting to specify the output file with `-o` will overwrite previous results or write to stdout in an unstructured format, making automated parsing difficult.
- Mismatching the database format version between `astalavista-build` and `astalavista` can cause silent failures where no alignments are returned without clear error messages.

## Examples

### Basic EST classification against a protein family database
**Args:** -d familydb.fasta -i query_est.fasta -o results.txt
**Explanation:** Uses the pre-built protein database `familydb.fasta` to classify EST sequences in `query_est.fasta`, writing aligned results to `results.txt`.

### Strict alignment filtering with low e-value threshold
**Args:** -d familydb.fasta -i query_est.fasta -e 1e-10 -o strict_results.txt
**Explanation:** Only reports alignments with e-values better than 1e-10, ensuring only high-confidence EST-to-protein matches are included in the output.

### Output in tabular format for automated parsing
**Args:** -d familydb.fasta -i query_est.fasta -m tab -o tabular_results.tsv
**Explanation:** Produces tab-delimited output format suitable for automated downstream analysis or import into spreadsheet applications.

### Building a protein database before alignment
**Args:** -d FamilyProteins.fasta -o familydb.fasta
**Explanation:** Uses `astalavista-build` to format raw protein sequences (`FamilyProteins.fasta`) into a searchable database (`familydb.fasta`) for use with subsequent alignment commands.

### Moderate filtering with e-value 0.001
**Args:** -d familydb.fasta -i query_est.fasta -e 0.001 -o moderate_results.txt
**Explanation:** Applies a standard e-value threshold of 0.001 (1e-3), balancing sensitivity and specificity for typical gene family classification tasks.

### Generating XML output for pipeline integration
**Args:** -d familydb.fasta -i query_est.fasta -m xml -o xml_results.xml
**Explanation:** Outputs alignment results in XML format, enabling integration with bioinformatics pipelines or conversion to other data formats.