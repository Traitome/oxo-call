---
name: bloomfiltertrie
category: Sequence Indexing / Probabilistic Data Structure
description: A mutable, space-efficient Bloom-filter-enhanced trie for indexing and querying large k-mer sets. Supports dynamic insertion, membership queries, and graph-based traversal. Commonly used in bioinformatics for de Bruijn graph indexing and large-scale sequence comparison tasks.
tags:
  - k-mer indexing
  - probabilistic data structure
  - de-bruijn-graph
  - sequence-search
  - bloom-filter
  - trie
  - genomics
  - bioinformatics
author: AI-Generated
source_url: https://github.com/gt1/bloomfiltertrie
---

## Concepts

- **Bloom-filter-augmented trie structure**: Each trie node maintains a Bloom filter encoding the set of k-mers descending through that node, allowing probabilistic membership pruning during traversal. The false-positive rate (configured at build time) trades off memory usage against query specificity.
- **Two companion binaries**: `bloomfiltertrie-build` constructs the index from one or more FASTA/FASTQ input files, while `bloomfiltertrie-query` answers membership queries against the built index. Using the wrong binary for the wrong operation will produce errors or meaningless output.
- **Input format**: All sequence inputs must be FASTA or FASTQ (plain or gzip-compressed). Lines that do not conform to these formats are silently skipped, which can lead to silently missing data if the input file contains unexpected content.
- **K-mer length is fixed at build time**: The `-k` flag sets k-mer size and must be consistent across all input files for a single index. Mixing different k values produces a corrupted or inconsistent index that may crash or give incorrect results at query time.
- **Mutable index**: Unlike immutable k-mer sketches, `bloomfiltertrie-build` supports incremental addition via `--mode add` without rebuilding from scratch, making it suitable for streaming or batched indexing pipelines.

## Pitfalls

- **Incorrect k-mer length**: Setting a k-mer length that is longer than the shortest sequence in the input causes those sequences to be silently omitted from the index, because no k-mers can be extracted. Always verify that the shortest input read is longer than the specified `-k` value.
- **Forgetting to specify the index file**: `bloomfiltertrie-query` requires an explicit `--index` path. Running it without this flag will fail with a usage error, not with a silent default, potentially breaking pipelines that assume a default path.
- **Conflicting build modes on an existing index**: Using `--mode add` on a freshly created index file (as opposed to resuming an existing one) is legal but using `--mode build` on a non-empty index path will overwrite it silently, destroying any previously accumulated data without confirmation.
- **Ignoring false-positive rates during query interpretation**: Bloom filter nodes can return false positives at deeper trie levels. Query results marked as "found" may not actually represent genuine k-mer presence; downstream tools that consume `bloomfiltertrie-query` output must account for this probabilistic nature.
- **Mixing compressed and uncompressed input files without declaring the flag**: `--gzip` must be explicitly provided for gzip-compressed input. Omitting it while passing `.gz` files causes the build or query to read garbage, leading to an index full of spurious k-mers or zero query hits.

## Examples

### Build a BloomFilterTrie index from a single FASTA file with a 25-mer length
**Args:** `bloomfiltertrie-build -k 25 --index out.bft references.fasta`
**Explanation:** This constructs a BloomFilterTrie named `out.bft` using all 25-mers from `references.fasta`, storing node-level Bloom filters with default false-positive rate.

### Build an index from multiple input files with gzip compression and reduced false-positive rate
**Args:** `bloomfiltertrie-build -k 31 --fpr 0.001 --gzip --index out.bft sample1.fq.gz sample2.fq.gz`
**Explanation:** This indexes two gzip-compressed FASTQ files using 31-mers and an aggressive false-positive rate of 0.1%, producing a larger but more specific index.

### Query a single k-mer against an existing index
**Args:** `bloomfiltertrie-query --index out.bft AAGGCTTGAC`
**Explanation:** This checks whether the exact 10-character string `AAGGCTTGAC` is found in the index, returning a presence/absence result accounting for trie-level Bloom filter pruning.

### Query all k-mers from a FASTQ file using batch mode
**Args:** `bloomfiltertrie-query --index out.bft --batch queries.fasta`
**Explanation:** This reads every sequence from `queries.fasta`, extracts k-mers of the length configured at build time, and reports membership for each, suitable for large-scale screening pipelines.

### Incrementally add new sequences to an existing index using add mode
**Args:** `bloomfiltertrie-build --mode add --index out.bft new_reads.fasta`
**Explanation:** This opens the existing `out.bft` index and adds all k-mers from `new_reads.fasta` without rebuilding, preserving previously indexed data and updating affected Bloom filter nodes.

### Query with verbose output showing traversal depth
**Args:** `bloomfiltertrie-query --index out.bft --verbose --batch query_set.fq`
**Explanation:** The `--verbose` flag emits per-node traversal information, which helps diagnose unexpected false-positive hits or identify trie branches with high Bloom filter collision rates.

### Build with explicit number of hash functions to control memory
**Args:** `bloomfiltertrie-build -k 27 -f 7 --index large.bft --gzip --reads collection.fa.gz`
**Explanation:** Specifying `-f 7` explicitly sets the number of Bloom filter hash functions, overriding the default calculation. Combined with `--gzip`, this builds a memory-tuned index from a large compressed FASTA collection.

### Print the index size and statistics after building
**Args:** `bloomfiltertrie-build -k 25 --index out.bft --stats in.fa`
**Explanation:** The `--stats` flag outputs index statistics (node count, Bloom filter bits per node, estimated false-positive rate) to stderr after construction, which is useful for monitoring and reproducibility.