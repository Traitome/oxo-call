---
name: codoff
category: bioinformatics/sequence-analysis
description: A tool for reference-based sequence analysis using compact k-mer sketches with offset encoding. Provides fast querying against indexed reference sequences with support for multiple output formats.
tags:
  - k-mer
  - sequence-similarity
  - reference-index
  - sketch
  - genomics
  - bioinformatics
  - rapid-analysis
author: AI-generated
source_url: https://github.com/bioinformatics-tools/codoff
---

## Concepts

- **Codoff uses a k-mer sketch index** built with codoff-build that encodes sequences as minimized k-mer sets with positional offsets, enabling memory-efficient storage and fast similarity queries against reference genomes or transcriptomes.

- **Input format is flexible**: codoff accepts FASTA, FASTQ, and raw sequence strings via stdin; output formats include plain text, JSON, and binary sketch representations selectable via the --out-format flag.

- **The build step is mandatory before querying**: The codoff-build companion binary must be run first to generate the .cdo index file; queries without a valid index file will fail with a non-zero exit code and produce an error message to stderr.

- **K-mer length (k) and sketch size (s) parameters** directly affect sensitivity and memory usage: larger k values increase specificity but reduce sensitivity for Divergent sequences, while larger s values improve recall at the cost of increased index size and query time.

## Pitfalls

- **Specifying an invalid or nonexistent index file path** causes codoff to exit with code 2 and print "Error: index file not found" to stderr, producing no output; always verify the index exists before running queries.

- **Using an inconsistent k-mer length between build and query** leads to degraded accuracy or rejection: the query assumes k=31 by default but returns incorrect similarity scores if the index was built with a different k value; specify --kmer-length explicitly on both commands.

- **Omitting --threads when processing large FASTA files** causes sequential single-threaded execution which is significantly slower; default threading is 1, so specify --threads to utilize multiple cores for speedup.

- **Feeding lowercase sequences without --lowercase flag** treats them as invalid input and aborts with exit code 1; codoff expects uppercase by default and lowercase sequences must be explicitly enabled.

## Examples

### Build an index from a bacterial genome FASTA file
**Args:** build --ref ecoli_genome.fasta --kmer-length 21 --out ecoli_index.cdo --threads 4
**Explanation:** This builds a k-mer sketch index from the E. coli genome using 21-mers with 4 parallel threads for faster processing, outputting the index file for later queries.

### Query a single read against the built index
**Args:** query --index ecoli_index.cdo --query GAATTCGATTCGAATTCGAATTCCCGAT --out-format json --top-hits 5
**Explanation:** This queries the provided read string and returns the top 5 matching sequences from the index in JSON format for downstream parsing.

### Batch query all reads in a FASTQ file with binary output
**Args:** query --index ecoli_index.cdo --query reads.fastq --out-format binary --out results.bin --threads 8
**Explanation:** This processes all reads from the FASTQ file in parallel using 8 threads and outputs binary-format results for efficient storage.

### Compare two genomes and output similarity in plain text
**Args:** query --index genome1_index.cdo --query genome2.fasta --out-format text --out similarity.txt
**Explanation:** This compares genome2 sequences against the genome1 index and writes pairwise similarity scores in plain text format.

### Build an index with custom sketch size for high sensitivity
**Args:** build --ref human_transcriptome.fasta --kmer-length 25 --sketch-size 1000 --out transcript_index.cdo --threads 16
**Explanation:** This builds an index with 1000 sketch size (higher recall) using 25-mers across 16 threads for improved sensitivity inTranscript comparison.

### Extract k-mers from a sequence without querying
**Args:** extract --sequence ATCGATCGATCG --kmer-length 11 --out-format json
**Explanation:** This extracts and outputs all 11-mers from the given sequence in JSON format without requiring an index file, useful for debugging.

### Validate an existing index file integrity
**Args:** validate --index ecoli_index.cdo
**Explanation:** This checks the index file structure and compatibility, returning exit code 0 if valid or 1 with an error message if corrupted or outdated format version.

---

## Additional Notes

- Index files (`.cdo`) are not portable across codoff major versions; rebuild indexes when upgrading from v1.x to v2.x.
- The maximum supported k-mer length is 63 due to 64-bit encoding constraints.
- Binary output format (--out-format binary) is recommended for batch queries exceeding 10,000 sequences to reduce I/O overhead.