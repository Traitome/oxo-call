---
name: cmph
category: Hash Functions / Data Structures
description: Generates minimal perfect hash functions for fast, collision-free hash table lookups in bioinformatics applications such as k-mer indexing and sequence membership testing.
tags: [hash, perfect-hash, data-structure, k-mer, indexing, cmph, library]
author: AI-generated
source_url: https://github.com/avarab/cmph
---

## Concepts

- **Minimal Perfect Hashing**: cmph generates a hash function that maps n keys to n consecutive integers (0 to n-1) with zero collisions, enabling O(1) worst-case lookup time withouthash table overhead.
- **Input Format**: Keys are provided as a text file with one key per line (e.g., k-mers, sequence identifiers); binary keyfiles are supported for large datasets.
- **Hash Function Algorithms**: cmph supports multiple algorithms including CHM (Composed Hashing Method), BDZ (Biggest Deviation Zigzag), and BMZ (Biggest Mod Zigzag), selectable via flags; BDZ is generally fastest for random keys.
- **Packed Format**: Generated hash structures can be packed into binary files for fast loading and memory-efficient deployment in production pipelines.
- **Companion Binaries**: The main `cmph` command is a wrapper; actual operations use `cmph-build` (creation) and `cmph-search` (lookup); these are the primary commands users invoke.

## Pitfalls

- **Non-Unique Keys**: Duplicate keys in the input file cause hash generation to fail or produce invalid hashes; always deduplicate input keyfiles before running cmph-build.
- **Memory Consumption for Large Datasets**: Building perfect hashes for millions of keys requires significant RAM; for very large k-mer sets (billions), consider chunking or using disk-backed construction.
- **Algorithm Mismatch**: Using the wrong algorithm for key distribution (e.g., CHM for highly clustered keys) results in slower lookups; test BDZ and BMZ to compare performance.
- **File Permission Issues**: cmph-build writes output files; ensure write permissions exist in the target directory or the command fails silently.
- **Hash Version Incompatibility**: Packed hash files from older cmph versions may not load with newer cmph-search binaries; regenerate hashes when upgrading the tool.

## Examples

### Building a minimal perfect hash from a k-mer file
**Args:** `cmph-build -v input_kmers.txt hash_data.cmph`
**Explanation:** Creates a minimal perfect hash from a file containing one k-mer per line, outputting to a packed hash file for fast lookups.

### Creating a hash with the BDZ algorithm for optimal speed
**Args:** `cmph-build -v -a bdz input_sequences.txt output.cmph`
**Explanation:** Uses the BDZ algorithm (recommended for random key distributions) to build a faster hash structure than default settings.

### Searching for a key using a generated hash file
**Args:** `cmph-search hash_data.cmph`
**Explanation:** Launches an interactive search session where you can input keys to test membership against the pre-built hash structure.

### Creating an indexed hash for batch search operations
**Args:** `cmph-build -v -i input_ids.txt batch_hash.cmph`
**Explanation:** Builds an indexed hash that supports efficient batch query processing, useful for membership testing across large query sets.

### Rebuilding a hash with verbose output for debugging
**Args:** `cmph-build -v -d debug_info.txt input_keys.txt rebuilt.cmph`
**Explanation:** Generates detailed debug information during hash construction, helping diagnose performance issues or failures with large keyfiles.