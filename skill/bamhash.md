---
name: bamhash
category: Data Integrity / BAM Processing
description: A tool for computing and verifying binary checksums of BAM/SAM alignment files. Enables detection of data corruption or accidental modification by generating per-base hash values that can be validated against original files.
tags:
- bam
- sam
- checksum
- integrity
- hash
- verification
- data-validation
- bioinformatics
author: AI-generated
source_url: https://github.com/ flavien/bamhash
---

## Concepts

- **Binary Checksum Generation**: bamhash computes cryptographic hashes at the binary level of BAM files, meaning it checksums the actual bytes stored in the compressed BAM format, not the decompressed text. This ensures detection of any corruption in the compressed data stream.
- **Companion Binary Architecture**: The toolset includes `bamhash` for computing/verifying checksums and `bamhash-build` for creating hash index databases. The hash index must be built from the original, trusted file before verification can be performed on test files.
- **Chunk-based Hashing**: Hashes are computed in configurable genomic chunks (default typically 1KB or 1MB), enabling both efficient processing of large files and localization of detected differences to specific genomic regions.
- **Multi-format Support**: Works with both BAM (binary) and SAM (text) formats, automatically detecting the input format and adapting the hashing algorithm accordingly.
- **Deterministic Output**: Uses a deterministic hashing algorithm that produces consistent results across different runs and machines, critical for reproducible data validation in collaborative workflows.

## Pitfalls

- **Mismatched Reference Hash Index**: Using a hash index built from a different file version or with different parameters (chunk size, algorithm) will cause verification to fail even if the data is intact. Always ensure the index matches the exact source file.
- **Unsorted vs Sorted Assumptions**: If a BAM file was re-sorted (e.g., from queryname to coordinate order) after the hash index was built, verification will fail because the byte positions have changed. Document the sorting order when creating the initial hash index.
- **Compressed vs Uncompressed Stream**: Hashes computed on a compressed BAM file (.bam) will not match hashes from the same data in an uncompressed SAM (.sam) format, even though the underlying sequences are identical. Use consistent file formats for comparison.
- **Ignoring Header Metadata**: Changes to read groups, RG tags, or other header metadata in the BAM will cause verification failures if these fields are included in the hash calculation. Clarify whether header changes should trigger failures in your workflow.
- **Insufficient Chunk Size for Large Files**: Setting chunk sizes too small can result in excessive memory usage and slow performance, while chunk sizes too large may reduce the precision of error localization when mismatches are detected.

## Examples

### Compute checksums for a BAM file
**Args:** -b myfile.bam -o myfile.bamhash
**Explanation:** Computes binary checksums for myfile.bam and saves the hash values to myfile.bamhash for later verification. The -b flag specifies the input BAM file and -o specifies the output hash file.

### Verify a BAM file against an existing hash
**Args:** -c myfile.bam -i reference.bamhash
**Args:** Explanation: The -c flag triggers comparison mode, verifying myfile.bam against the reference hash index in reference.bamhash. Any differences will be reported with genomic coordinates.

### Build a hash index from a trusted BAM file
**Args:** -b trusted.bam -o trusted_index.bamhash
**Explanation:** Creates a hash database from a trusted source file that can later be used to verify copies or backups. This is the first step in a data integrity workflow.

### Specify a custom chunk size for large genomes
**Args:** -b large_genome.bam -o large_index.bamhash -s 1048576
**Explanation:** Sets the chunk size to 1MB (1048576 bytes) for processing large files more efficiently. The -s flag controls the granularity of hash computation.

### Use MD5 hash algorithm instead of default
**Args:** -b input.bam -o output.bamhash -a md5
**Explanation:** Specifies MD5 hashing instead of the default algorithm. Different hash algorithms may be required for compatibility with other verification pipelines.

### Verify a compressed SAM output
**Args:** -c -i input.bamhash input.sam
**Explanation:** Verifies a SAM format file against a hash index that was built from a BAM. Format conversion differences will cause verification to fail if the index wasn't built from SAM format.

### Output detailed mismatch report
**Args:** -c -v -i ref_hash.bamhash test.bam
**Explanation:** The -v flag enables verbose output, showing detailed information about which chunks contain mismatched data. useful for debugging.