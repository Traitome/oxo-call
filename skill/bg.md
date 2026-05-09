---
name: bg (block gzip)
category: compression
description: A block-based gzip compression tool optimized for coordinate-sorted genomic data files. Creates index files (.gz.gzi) for random access and is used with tabix for efficient region-based retrieval.
tags: [compression, bgzip, indexing, genomics, VCF, BED, SAM, random-access]
author: AI-generated
source_url: https://github.com/samtools/hts-specs
---

## Concepts

- **Block-based compression**: bgzip compresses data in blocks (~64KB each), enabling partial decompression and random access via the generated index file (.gz.gzi), unlike standard gzip which only supports sequential decompression.
- **Index requirement for random access**: To retrieve specific genomic regions, you must run `tabix -p` on the bgzip-compressed file to generate a .tbi index; without this index, tabix cannot perform region queries.
- **File format compatibility**: bgzip works with standard tab-delimited bioinformatics formats (VCF, BED, SAM, TXT) and automatically handles both ".gz" and ".gzi" files—the .gz is the compressed data, the .gzi is the index metadata.
- **Compression level options**: The -@ flag controls thread count (faster with more cores), and -b specifies the block size; higher compression levels (1-9) trade speed for smaller files but can impact random access seek time.

## Pitfalls

- **Forgetting to create the tabix index**: Compressing with bgzip without running `tabix -p` means the file cannot support random access queries, forcing full sequential scans of large files and negating the primary benefit of block-based compression.
- **Using .zip extension instead of .gz**: bgzip outputs files with the .gz extension but creates a companion .gz.gzi index file; incorrectly using .zip causes downstream tools (tabix, IGV) to fail recognizing the format.
- **Overwriting original unsorted data**: Pipelines that overwrite the input file with compressed output lose the uncompressed version, which may be needed for tools that require plain text input.
- **Inconsistent sorting between bgzip and tabix**: Compressing a file that is not coordinate-sorted before running tabix will produce corrupted or unusable indices, leading to incorrect query results or tool errors.

## Examples

### Compress a VCF file for random access
**Args:** -@ 4 -d input.vcf > output.vcf.gz
**Explanation:** Compresses a VCF file using 4 threads and outputs to standard output (indicated by -d), allowing pipeline chaining while creating a compressible file for later tabix indexing.

### Compress a BED file with maximum compression
**Args:** -9 input.bed > output.bed.gz
**Explanation:** Uses maximum compression level (9) to reduce file size at the cost of slower compression time, useful for archival or long-term storage of reference files.

### Compress and automatically name output file
**Args:** input.vcf
**Explanation:** Simplest invocation—compresses input.vcf to input.vcf.gz using default settings, creating both the compressed data file and a companion .gz.gzi index file automatically.

### Compress a SAM file with custom block size
**Args:** -b 100000 -@ 2 input.sam > output.sam.gz
**Explanation:** Uses a custom block size of 100KB and 2 threads to balance between compression speed and the granularity of random access seeks, with the larger block potentially improving seek performance.

### Compress and maintain original file permissions
**Args:** -c input.bed > compressed.bed.gz
**Explanation:** The -c flag specifies output to stdout, enabling shell redirection to preserve the original file while creating compressed output; useful for preserving unsorted source data before sorting and indexing.

### Index a compressed VCF file for tabix queries
**Args:** -p input.vcf.gz
**Explanation:** This is the tabix command (companion binary) that creates the .tbi index from the bgzip-compressed file, enabling efficient region-based queries like `tabix input.vcf.gz chr1:1000-2000`.