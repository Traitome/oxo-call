---
name: cobs
category: Sequence Search and Classification
description: COBS is a bioinformatics tool for building and searching compact FM-indexes of nucleotide or protein sequences. It enables efficient taxonomic classification and sequence similarity searching across large sequence databases.
tags:
  - sequence-search
  - fm-index
  - taxonomic-classification
  - bioinformatics
  - dna-analysis
  - protein-analysis
  - index-based-search
author: AI-generated
source_url: https://github.com/bioinformatics上游/cobs
---

## Concepts

- COBS constructs an FM-index from input sequences using the classic or space-saving tradeoff modes. The index stores k-mer fingerprints in a compact structure that enables disk-based queries against large sequence collections without loading entire sequences into memory.

- Sequence alphabet detection (DNA/RNA vs protein) must match between the build and search operations. Building with --dna and searching with --protein, or vice versa, produces zero matches because k-mer tokenization differs fundamentally between alphabets.

- The k-mer size (-k) is a critical parameter affecting both index size and search sensitivity. Larger k-mers produce smaller indexes but reduce sensitivity for divergent sequences; smaller k-mers increase sensitivity but expand index size and potential false positive rates.

- Search operations return sequence identifiers ranked by k-mer overlap scores, which reflect the proportion of shared k-mers between the query and indexed sequences. The --threshold (-t) flag filters results to include only matches above a specified minimum score.

- COBS writes output in multiple formats controlled by --output-format. Tab-delimited output is human-readable for manual inspection, while JSON output is structured for programmatic parsing in automated pipelines.

## Pitfalls

- Building an index with the wrong alphabet flag silently corrupts search results. A protein index built without --protein will match DNA queries but report meaningless k-mer overlaps that do not reflect biological similarity.

- Query sequences shorter than the k-mer size used during index construction cannot match any indexed k-mers, resulting in empty result files that appear to indicate no matches despite valid input sequences.

- Attempting to search against a directory rather than an index file produces cryptic error messages. Always verify the index file path exists and is readable before running search operations.

- Overwriting existing index files without --force flag causes build operations to abort, and subsequent searches may run against stale index data without warning.

- Insufficient disk space during index building produces truncated index files that appear to build successfully but corrupt search operations, leading to inconsistent or missing results.

## Examples

### Build a DNA sequence index from a FASTA file

**Args:** classic-Tf --dna -k 31 /path/to/genomes.fa /path/to/dna_index

**Explanation:** This constructs a DNA FM-index using a 31-kmer size from genome sequences in the specified FASTA file, optimizing the index for typical bacterial genome searches.

### Build a protein sequence index from a FASTA file

**Args:** classic-Tf --protein -k 17 /path/to/proteins.fa /path/to/protein_index

**Explanation:** This creates a protein FM-index with a 17-kmer size, which is appropriate for protein sequence similarity searches where amino acid sequences require smaller k-mer values.

### Search a DNA index using a query sequence file

**Args:** search --dna /path/to/dna_index /path/to/query.fa

**Explanation:** This queries the DNA index with nucleotide sequences from the query file, returning all indexed sequences that share k-mers with the input queries.

### Search with verbose output and score threshold filtering

**Args:** search -t 0.5 -v /path/to/dna_index /path/to/query.fa

**Explanation:** This filters search results to include only matches with similarity scores of at least 0.5 while displaying verbose diagnostic information about the search process.

### Output search results in JSON format for pipeline integration

**Args:** search --json -o results.json /path/to/dna_index /path/to/query.fa

**Explanation:** This exports search results to a JSON file, which is ideal for automated downstream analysis in workflows or web applications.