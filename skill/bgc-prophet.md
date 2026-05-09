---
name: bgc-prophet
category: bioinformatics
description: A command-line tool for predicting biosynthetic gene clusters (BGCs) from genomic assemblies using hidden Markov models and machine learning classification approaches.
tags: [bgc, biosynthetic-gene-cluster, genome-analysis, natural-products, hmmer, machine-learning]
author: AI-generated
source_url: https://github.com/chg0901/bgc-prophet
---

## Concepts

- **Input data model**: bgc-prophet accepts assembled contigs in multi-FASTA format as primary input. It expects nucleotide sequences longer than 5 kb for reliable predictions, as shorter contigs lack the structural features needed for complete BGC detection. Both plain-text and gzipped FASTA files are supported as input.

- **BGC detection approach**: The tool combines profile hidden Markov models (pHMMs) built from known BGC families with a random forest classifier. It scans six-kilobase sliding windows across sequences, scoring each window against domain-specific models for NRPS, PKS, RiPP, terpenes, and other biosynthetic classes. Predictions are ranked by aggregate confidence scores between 0.0 and 1.0.

- **Output formats**: Three output modes are available—JSON (structured data), GFF3 (feature annotations), and text summary (human-readable). JSON output includes coordinate mappings, hit domains, e-values, bit scores, and per-cluster classifications for downstream automation. GFF3 output is compatible with standard genome browsers like JBrowse and Apollo.

- **Performance considerations**: Single-genome predictions run in 2–10 minutes depending on assembly size and sequence count. Parallelization is achieved through chunk-based threading controlled by the --threads flag. Large genomes exceeding 500 contigs should be pre-filtered to remove sequences shorter than 5 kb to avoid excessive false positive windows.

## Pitfalls

- **Consequence: False negatives from fragmented assemblies**: Submitting assemblies with N50 values below 10 kb causes the algorithm to miss large BGCs that span multiple contigs. Predictions become biased toward small RiPP and NRPS clusters, missing entire PKS families that require complete megasynthase domains.

- **Consequence: Overprediction due to permissive thresholds**: Running with default e-value cutoffs above 0.1 produces spurious domain hits that artificially inflate cluster counts. Users report 3–5x higher BGC counts compared to reference databases like MIBiG, requiring manual curation and reannotation effort.

- **Consequence: Output parsing failures**: The JSON schema changed between v0.2.x and v0.3.x versions, breaking automated parsers that relied on the 'cluster_id' field which was renamed to 'bgc_id'. Scripts written for older versions silently fail or produce empty result sets.

- **Consequence: Memory exhaustion on chromosome-scale assemblies**: Inputs larger than 100 Mb without the --chunk-size flag trigger out-of-memory kills on standard compute nodes. The sliding window approach scales quadratically with sequence length, requiring explicit chunk boundaries for eukaryotic or viral genomes.

- **Consequence: Missing critical cluster types**: Specifying --domain-filter without including 'terpene' or 'siderophore' categories causes the tool to silently discard entire biosynthetic families. Default filters are too restrictive for comprehensive natural product discovery in actinomycete genomes, leading to incomplete biosynthetic potential assessments.

## Examples

### Basic BGC prediction on a bacterial assembly
**Args:** input.fasta --output results.json --format json
**Explanation:** This runs a standard prediction scan on an assembled genome, outputting structured results with coordinates and domain hits in JSON format.

### Filtering predictions by confidence threshold
**Args:** assembly.fasta --output high_confidence.gff --min-score 0.75 --format gff3
**Explanation:** Setting a minimum confidence threshold of 0.75 removes low-confidence predictions, producing a GFF3 file suitable for genome browser visualization.

### Specifying BGC classes to predict
**Args:** contigs.fasta --bgc-classes nrps pks terpene --output classified.json --format json
**Explanation:** Limiting predictions to NRPS, PKS, and terpene classes reduces false positives and focuses analysis on the most chemically relevant biosynthetic families.

### Using a custom HMM reference database
**Args:** assembly.fasta --hmm-database custom-profiles.hmm --output custom_results.json --format json
**Explanation:** Providing a custom HMM database replaces default profiles, enabling detection of species-specific or novel BGC variants not covered by the built-in models.

### Parallel processing for large genomes
**Args:** large_genome.fasta --output results.json --threads 16 --chunk-size 50 --format json
**Explanation:** Enabling 16-thread parallelization with 50 Mb chunk sizes accelerates processing on large bacterial genomes while managing memory usage effectively.

### Extracting predicted BGC sequences as separate FASTA files
**Args:** assembly.fasta --extract-sequences --output bgc_sequences.fasta --format fasta
**Explanation:** This exports predicted BGC genomic regions as individual FASTA sequences for downstream analysis like phylogenetic tree construction or comparative genomics workflows.