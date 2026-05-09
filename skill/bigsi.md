---
name: bigsi
category: k-mer analysis / metagenomics
description: Fast approximate membership querying for k-mers across multiple sequencing datasets using Bloom filter indexing. Enables rapid search of which datasets contain specific k-mers or k-mer patterns, commonly used in metagenomic assembly-free analyses.
tags: [k-mer, bloom-filter, metagenomics, approximate-membership, index, approximate-search]
author: AI-generated
source_url: https://github.com/IFICL/bigsi
---

## Concepts

- BigSI builds a compressed Bloom filter index from multiple FASTA/FASTQ files, creating a data structure that can query whether specific k-mers appear in each indexed dataset. The index stores presence information rather than sequence identity, enabling memory-efficient storage of millions of datasets.

- The query operation returns a bit vector indicating which datasets contain each input k-mer. For batch queries, it produces a matrix where rows correspond to query k-mers and columns to indexed datasets, enabling rapid screening of metagenomic samples.

- The index supports configurable false positive rates (typically 0.01 to 0.001) that trade specificity for memory. Lower false positive rates require more memory but produce fewer spurious matches, critical when distinguishing closely related sequences.

- BigSI uses a disk-based storage model that allows querying large indexes without loading them entirely into RAM, making it scalable to TB-scale metagenomic databases while maintaining reasonable query speeds.

## Pitfalls

- Using mismatched k-mer sizes between index construction and querying returns zero matches. The k-mer size must be consistent across building and querying, and using an incorrect size silently produces empty results.

- Setting the false positive rate too high (e.g., >0.05) introduces numerous spurious dataset associations, leading to incorrect conclusions about sample membership in metagenomic analyses.

- Querying reads containing ambiguous nucleotide characters (N) produces unreliable results because Bloom filters cannot represent ambiguity states—these k-mers may be falsely reported as present or absent.

- Building an index from FASTQ files with low-quality base calls that haven't been quality trimmed inflates the index size with noisy k-mers, reducing specificity and increasing memory usage.

- Attempting to query a k-mer shorter than the index k-mer size is impossible; the query must use k-mers of exactly the same length as those indexed.

## Examples

### Build a BigSI index from a directory of FASTQ files
**Args:** build -k 20 -f 0.01 /path/to/fastq/dir /path/to/index.bigsi
**Explanation:** Creates a Bloom filter index with 20-mers from all FASTQ files in the directory, using a 1% false positive rate to balance memory usage against spurious matches.

### Query a single k-mer against an existing index
**Args:** query AGCTAGCTAGCTAGCTAGC /path/to/index.bigsi
**Explanation:** Queries whether the 20-mer "AGCTAGCTAGCTAGCTAGC" is present in any dataset indexed, returning a bit vector of matching datasets.

### Query a FASTA file of k-mers and output results to a file
**Args:** query -o results.txt /path/to/queries.fasta /path/to/index.bigsi
**Explanation:** Batch queries all k-mers in the FASTA file against the index and writes the presence matrix to results.txt for downstream analysis.

### Build an index with a lower false positive rate for higher precision
**Args:** build -k 25 -f 0.001 /path/to/filtered/dir /path/to/precise.bigsi
**Explanation:** Constructs a 25-mer index with only 0.1% false positive rate, using more memory but producing fewer spurious dataset associations for high-precision metagenomics.

### View index metadata and statistics
**Args:** info /path/to/index.bigsi
**Explanation:** Displays index statistics including k-mer size, false positive rate, number of datasets indexed, total k-mers stored, and memory footprint.

### Query multiple k-mers from stdin input
**Args:** query - /path/to/index