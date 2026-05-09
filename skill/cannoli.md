---
name: cannoli
category: Functional Annotation
description: A bioinformatics tool for automated functional annotation of protein sequences, integrating multiple annotation sources including UniProt, Pfam, InterPro, and other databases to predict gene product functions, domains, and pathways.
tags:
  - protein-annotation
  - functional-annotation
  - sequence-analysis
  - bioinformatics
  - genomics
  - protein-function
author: AI-generated
source_url: https://github.com/yigittaskin/cannoli
---

## Concepts

- Cannoli takes protein sequences in FASTA format as primary input and outputs functional annotations in multiple formats including TSV, JSON, GFF3, and RDF. The tool supports batch processing of thousands of sequences in a single run.
- The tool integrates multiple annotation databases (UniProtKB, Pfam, InterPro, CDD, COGs) and uses hierarchical classification to assign Gene Ontology terms, enzyme commission numbers, and protein family memberships to input sequences.
- Cannoli provides both high-throughput command-line workflows and programmable API access for integration into larger bioinformatics pipelines. Output can be filtered by confidence scores, annotation source, or specific feature types.

## Pitfalls

- Running cannoli without specifying an output format defaults to TSV, which may not be compatible with downstream visualization tools that expect GFF3 or JSON format. Always verify your output format matches the requirements of subsequent analysis steps.
- Default annotation depth may miss lineage-specific annotations or obscure protein families. Setting thresholds too high (e.g., --evalue 1e-20) can cause false negatives for divergent sequences, while too low thresholds introduce noise from non-specific matches.
- Memory consumption scales linearly with input sequence count; attempting to annotate very large datasets (>100,000 sequences) without the --chunk option can cause out-of-memory errors on systems with limited RAM.

## Examples

### Annotate a single protein sequence in FASTA format
**Args:** annotate --input proteins.fasta --output annotation.tsv
**Explanation:** This runs functional annotation on a FASTA file containing one or more protein sequences and saves results to a tab-separated file.

### Output annotations in GFF3 format for genome browsers
**Args:** annotate --input proteins.fasta --format gff3 --output annotation.gff3
**Explanation:** Generates GFF3 formatted output which can be directly loaded into IGV, JBrowse, or UCSC Genome Browser for visualization.

### Run annotation with strict confidence threshold
**Args:** annotate --input proteins.fasta --threshold 0.9 --output high_confidence.tsv
**Explanation:** Only includes annotations with confidence score >= 0.9, filtering out low-confidence predictions to reduce false positives.

### Process large dataset in chunks to avoid memory issues
**Args:** annotate --input large_proteins.fasta --chunk 10000 --output annotations/
**Explanation:** Splits the input into chunks of 10,000 sequences each, processing sequentially to prevent memory exhaustion on large files.

### Retrieve only GO terms from annotation output
**Args:** annotate --input proteins.fasta --filter go --output go_terms.tsv
**Explanation:** Extracts only Gene Ontology annotations from the full results, useful for enrichment analysis pipelines.

### Use multiple annotation databases simultaneously
**Args:** annotate --input proteins.fasta --dbs uniprot pfam interpro --output multi_source.tsv
**Explanation:** Runs annotation against UniProt, Pfam, and InterPro databases in parallel and merges results for comprehensive coverage.