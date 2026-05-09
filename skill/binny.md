---
name: binny
category: bioinformatics/data-format
description: A binary bioinformatics data format handler for efficient storage, compression, and random-access manipulation of genomic data. Converts between standard text formats (FASTA, FASTQ, VCF, BAM) and optimized binary representations.
tags: binary-format, bioinformatics, genomics, data-conversion, compression, fasta, fastq, vcf, indexed-access
author: AI-generated
source_url: https://github.com/fieldhype/binny
---

## Concepts

- **Binary packing and unpacking**: binny converts text-based bioinformatics formats (FASTA, FASTQ, VCF, GTF) into compressed binary files (.binny extension) using optional compression levels (0-9), reducing storage by 60-80% while enabling faster random access to specific genomic regions.
- **Indexed random access**: The `index` subcommand creates a .binny.idx file that enables O(1) lookup of sequences by name or genomic coordinates without scanning the entire file, critical for large datasets like whole-genome BAM or VCF files.
- **Format auto-detection**: When input format is ambiguous, binny attempts auto-detection by examining file magic bytes and structure; however, explicit format specification via `--format` is recommended for reliability.
- **Integrity verification**: The `verify` subcommand checks file integrity via checksums and reports any corruption before destructive operations, preventing data loss from corrupted binary archives.

## Pitfalls

- **Specifying mismatched input format**: Passing `--format fasta` when the input is FASTQ causes silent data truncation or corruption because FASTQ quality scores are interpreted as sequence data, leading to downstream analysis failures.
- **Skipping index creation before random access**: Attempting coordinate-based retrieval (`binny view --chromosome chr1:1000000-2000000`) on unpacked files without a pre-built index results in full-file scans that can take hours on terabyte-scale files.
- **Deleting original files before verification**: Removing source text files after packing without running `binny verify` first risks permanent data loss if the binary archive was corrupted during the packing process.
- **Using incompatible compression levels**: Setting `--compression 0` for maximum speed on large files consumes disproportionate memory during decompression, while `--compression 9` on small files wastes CPU without meaningful space savings.

## Examples

### Convert a FASTA file to compressed binary format

**Args:** `pack --input sequences.fasta --output sequences.binny --compression 6`
**Explanation:** This converts a text FASTA file to a binary .binny file with moderate compression (level 6), reducing storage by ~70% while maintaining fast decompression for downstream tools.

### View the first 10 sequences from a binary file

**Args:** `view --input sequences.binny --head 10 --format text`
**Explanation:** Extracts and displays the first 10 sequences in human-readable FASTA format without unpacking the entire file, enabling quick data inspection of large binary archives.

### Retrieve a specific genomic region from an indexed binary VCF

**Args:** `view --input variants.binny --region chr1:50000000-51000000 --format vcf`
**Explanation:** Uses the pre-built index to directly retrieve only variants within the specified chromosome 1 region (~1Mb range), avoiding full-file scans that would take minutes on multi-gigabyte VCF files.

### Create an index for fast random access

**Args:** `index --input sequences.binny --output sequences.binny.idx`
**Explanation:** Builds a .binny.idx file containing offsets and sequence name mappings, enabling O(1) sequence lookups and coordinate-based retrieval in subsequent operations.

### Verify integrity of a binary archive before deleting originals

**Args:** `verify --input sequences.binny --checksum sha256`
**Explanation:** Computes and reports SHA-256 checksums for all packed records, ensuring no corruption occurred during storage or transfer before safe deletion of source files.

### Convert binary archive back to text FASTQ format

**Args:** `unpack --input sequences.binny --output sequences.fastq --format fastq`
**Explanation:** Decompresses and converts the binary archive back to standard FASTQ format with quality scores intact, enabling compatibility with tools that lack binny support.

### Get statistics on a binary file without unpacking

**Args:** `stats --input sequences.binny --detailed`
**Explanation:** Reports metadata including total sequences, total bases, compression ratio, and any integrity warnings without decompressing the file, useful for quick inventory of archived data.