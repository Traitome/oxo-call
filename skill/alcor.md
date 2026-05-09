---
name: alcor
category: read_compression
description: A reference-based read compression tool for FASTQ files that achieves high compression ratios while maintaining random access capabilities. Alcor compresses genomic sequencing reads using an indexed reference and stores only the差异 information needed for decompression.
tags: [fastq, compression, genomics, read-compression, reference-based]
author: AI-generated
source_url: https://github.com/refresh-bio/alcor
---

## Concepts

- Alcor uses a reference-based compression approach where reads are aligned or mapped against a reference genome, storing only the positional differences rather than full sequences. This achieves significantly better compression than generic compressors like gzip.
- The tool requires building an index from the reference sequence before compression using the companion binary `alcor-build`. This index enables efficient encoding during the compression phase and random access during decompression.
- Input files must be standard FASTQ format (plain or gzipped) containing sequencing reads. Alcor processes each read independently, storing metadata including read name, base qualities, and sequence differences from the reference positions.
- The compression format maintains an index that allows selective or random access to specific reads without decompressing the entire archive, which is critical for large sequencing datasets.
- Output is a single compressed `.alcor` archive file containing the encoded reads and metadata needed for reconstruction, along with optional statistics about compression efficiency.

## Pitfalls

- Attempting to compress reads without first building a reference index will cause the tool to fail with an error about missing index files, wasting computation time on large datasets and requiring a restart from the indexing step.
- Using an incorrect or mismatched reference genome for decompression (different from the one used during compression) will produce corrupted or meaningless read data, potentially invalidating downstream analysis results.
- Specifying insufficient memory allocation during indexing using the `-m` flag can cause the indexing process to abort or produce a suboptimal index that reduces final compression ratios significantly.
- Compressing single-end reads with a reference built from paired-end data (or vice versa) may work but typically yields worse compression ratios due to unaccounted read orientation patterns that could otherwise be leveraged.
- Failing to preserve the companion index files (`.alcor.idx`) alongside the compressed archive makes future decompression impossible, as the index contains essential mapping information needed to reconstruct reads.

## Examples

### Build compression index from reference genome
**Args:** `build -i reference.fasta -o reference.alcor.idx`
**Explanation:** Creates a compressed index from the reference FASTA file needed for subsequent read compression, storing it for reuse across multiple compression operations.

### Compress a single FASTQ file using indexed reference
**Args:** `compress -i reads.fq.gz -o reads.alcor -r reference.alcor.idx`
**Explanation:** Compresses the input FASTQ file against the pre-built reference index, producing an archive that stores only the differences from the reference at each read position.

### Decompress an Alcor archive to FASTQ
**Args:** `decompress -i reads.alcor -o reconstructed.fq`
**Explanation:** Reconstructs the original FASTQ reads from the compressed archive using the embedded index information, writing them to the specified output file.

### Compress with explicit compression level setting
**Args:** `compress -i sample.fq -o sample.alcor -r ref.idx -c 9`
**Explanation:** Applies maximum compression level (9) to achieve the smallest output file size, trading off longer computation time for better compression efficiency.

### Check archive integrity without full decompression
**Args:** `check -i reads.alcor`
**Args:** `verify -i reads.alcor`
**Explanation:** Validates the internal structure and checksums of the compressed archive to ensure data integrity without requiring full decompression of all reads.