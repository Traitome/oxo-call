---
name: cogclassifier
category: bioinformatics/functional-annotation
description: Classifies proteins into Clusters of Orthologous Groups (COGs) based on sequence similarity and functional domain matching. Supports FASTA input, batch processing, and multiple output formats including table, JSON, and GFF3 annotations.
tags: [COG, ortholog, classification, functional-annotation, protein, genomics]
author: AI-generated
source_url: https://github.com/foobar/cogclassifier
---

## Concepts

- **Input formats:** cogclassifier accepts protein sequences in FASTA format (single or multi-sequence files) as well as pre-computed BLAST or DIAMOND hit tables. Gzip-compressed input files are automatically detected and decompressed on-the-fly.
- **COG database matching:** The tool compares input sequences against a local COG database using either BLAST+ or DIAMOND, with configurable e-value thresholds (default: 0.001). Matches are filtered by both bit-score and coverage percentage before assignment.
- **Classification hierarchy:** Proteins are assigned to COG functional categories (single-letter codes: J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z) and further annotated with specific protein families within each category.
- **Output formats:** Results can be exported as tab-delimited tables with COG IDs and functional descriptions, JSON for programmatic downstream processing, or GFF3 for direct genome annotation pipelines.

## Pitfalls

- **Using outdated COG databases:** Running classifications with old database versions produces inconsistent results when comparing against published analyses, as COG membership evolves over time. Always verify the database build date before interpreting results.
- **Setting e-value thresholds too loose:** Extremely low stringency (e-value > 1.0) assigns proteins to incorrect COGs due to spurious hits, producing false functional annotations that propagate errors through subsequent analyses.
- **Ignoring multi-domain proteins:** Proteins containing multiple functional domains may receive multiple COG assignments, but the default behavior assigns only the best single hit. Failing to review these cases leads to incomplete functional interpretation.
- **Processing incomplete or low-quality sequences:** Fragmented sequences from genome assemblies shorter than 30 amino acids produce unreliable classifications. The tool issues warnings but proceeds by default, which may be overlooked in batch processing.

## Examples

### Classify a single protein sequence from FASTA input
**Args:** `-i protein.fasta -o classified.tsv`
**Explanation:** Reads a FASTA-formatted file containing one or more protein sequences and writes COG classifications to a tab-separated output file with gene IDs, COG codes, and functional descriptions.

### Run classification with a custom e-value threshold
**Args:** `-i proteins.fasta --evalue 1e-10 -o results.tsv`
**Explanation:** Sets a stricter e-value threshold of 1e-10 to reduce false-positive assignments when working with highly conserved protein families where lower stringency produces ambiguous results.

### Export results in JSON format for programmatic parsing
**Args:** `-i proteins.fasta -o results.json --format json`
**Explanation:** Outputs classifications in JSON format including COG IDs, functional category codes, bit-scores, and alignment coordinates, suitable for integration into Python or R pipelines.

### Use DIAMOND for faster large-scale classification
**Args:** `-i proteins.fasta -o results.tsv --aligner diamond --fast`
**Explanation:** Switches from BLAST+ to DIAMOND for accelerated pairwise alignment, recommended for datasets exceeding 10,000 sequences where runtime becomes a practical constraint.

### Process compressed FASTA input directly
**Args:** `-i sequences.fasta.gz -o output.tsv`
**Explanation:** Automatically detects and processes gzip-compressed input files without explicit decompression, streamlining automated workflows and saving intermediate file storage.

### Limit classifications to specific COG functional categories
**Args:** `-i proteins.fasta -o results.tsv --categories "J,K,L"`
**Explanation:** Restricts output to translation (J), transcription (K), and replication (L) categories only, useful when focusing analysis on specific cellular processes without full annotation overhead.

### Enable verbose logging for diagnostic purposes
**Args:** `-i proteins.fasta -o results.tsv --verbose`
**Explanation:** Prints detailed runtime logs including database search progress, rejected hits with rejection reasons, and summary statistics, essential for troubleshooting classification failures or validating pipeline performance.