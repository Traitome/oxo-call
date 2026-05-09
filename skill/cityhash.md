---
name: cityhash
category: Hash Functions
description: A fast, non-cryptographic hash function family (cityhash64 and cityhash128) developed by Google for hashing strings and binary data. Produces 64-bit or 128-bit hash values useful for hash tables, checksums, and fingerprinting data in bioinformatics pipelines.
tags:
- hash
- fingerprinting
- checksum
- string-matching
- non-cryptographic
author: AI-generated
source_url: https://github.com/google/cityhash
---

## Concepts

- **Hash Output Sizes**: cityhash provides two main variants — `cityhash64` produces a 64-bit (16 hex character) hash, while `cityhash128` produces a 128-bit (32 hex character) hash; the larger variant reduces collision risk for large datasets.
- **Input Handling**: The tool reads from standard input (stdin) by default, accepting strings or piped data, and outputs the hash value in hexadecimal format to stdout.
- **Seed Values**: Both variants support an optional seed parameter (first argument for cityhash64, first two for cityhash128) to generate different hash outputs for the same input — useful for creating multiple independent hash families from one dataset.
- **Speed Optimization**: CityHash is optimized for modern CPUs with AES-NI instructions, making it significantly faster than cryptographic hash functions like SHA-256 for high-throughput bioinformatics data processing.

## Pitfalls

- **Confusing Input Sources**: Using both stdin and a seed argument simultaneously can lead to unexpected behavior — the seed takes precedence and stdin data is ignored depending on implementation.
- **Collision in 64-bit Mode**: For datasets with billions of unique strings (e.g., k-mer spectra), 64-bit hashes have a non-trivial collision probability; use cityhash128 for bioinformatics analyses where false positives are costly.
- **No Built-in Verification**: CityHash is not a cryptographic hash — it provides no integrity verification or resistance to adversarial manipulation; do not use for security-critical checksums.
- **Whitespace Stripping**: Trailing newlines in stdin are often stripped before hashing, which can cause subtle bugs when comparing hashes of file contents versus strings with embedded newlines.

## Examples

### Hash a simple string using the 64-bit variant
**Args:** `64 "my-sequence"`
**Explanation:** Computes a 64-bit hash (8 bytes in hex) of the quoted string input, useful for quick fingerprinting of sequence identifiers in pipelines.

### Hash sequence data piped from another command
**Args:** `64`
**Explanation:** Reads raw sequence data from stdin and outputs the 64-bit hash, enabling chain composition with tools like `grep` or `awk` in shell pipelines.

### Generate a 128-bit hash for a FASTA header
**Args:** `128 " >gene_001|ATCGATCG"`
**Explanation:** Produces a 128-bit hash (32 hex characters) which reduces collision risk when creating unique keys for large genomic feature databases.

### Use a seed value to generate independent hash variants
**Args:** `64 42 "ATGCATGC"`
**Explanation:** Applies seed value 42 to produce a different hash output than the unseeded version, allowing parallel independent analyses in distributed workflows.

### Hash gzip-compressed input data directly
**Args:** `128`
**Explanation:** When piped through `zcat file.gz | cityhash128`, the tool hashes the decompressed content, enabling integrity checks on compressed archives without explicit decompression steps.

### Compare two sequence files for equality using hash fingerprints
**Args:** `64`
**Explanation:** Generating hashes of both files and comparing the outputs quickly identifies whether files are identical without performing a full byte-by-byte diff, useful for validating pipeline outputs.