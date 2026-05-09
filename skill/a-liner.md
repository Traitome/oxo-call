---
name: a-liner
category: sequence_analysis
description: A bioinformatics tool for extracting, transforming, and manipulating single-line sequence records from FASTA, FASTQ, and other standard bioinformatics file formats. Supports batch processing, filtering, and format conversion operations on nucleotide and protein sequences.
tags: [fasta, fastq, sequence-processing, text-manipulation, format-conversion, bioinformatics,-cli]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/a-liner
---

## Concepts

- **Input/Output Formats**: a-liner reads FASTA (`.fa`, `.fasta`), FASTQ (`.fq`, `.fastq`), and plain text formats (`.txt`, `.seq`). Output can be in any of these formats or converted between them using the `-of` flag. Files can be input via stdin using `-` as the filename.

- **One-Record-Per-Line Model**: Sequences are processed such that each complete sequence record (header + sequence for FASTA; header + quality for FASTQ) is treated as a single processing unit. The tool iterates through records sequentially, enabling deterministic batch operations. Record boundaries are preserved in output unless the `-j` (join) flag is used.

- **Filtering and Selection**: Records can be filtered by sequence length using `-min` and `-max`, by regex pattern matching with `-match` or `-exclude`, and by sequence identifier using `--id`. The tool supports logical AND/OR combinations via multiple filter flags.

- **Companion Binary**: a-liner-build is a companion index builder that creates lookup indices for fast random access to sequence records, useful for extracting specific records by identifier from large files without scanning the entire file.

## Pitfalls

- **Mismatched File Formats**: Providing a FASTQ file when expecting FASTA (or vice versa) will cause silent parsing errors or partial output without warning. Always verify the input format matches the inferred format from file extension, or explicitly specify using `-if fasta` or `-if fastq`.

- **Memory Limits with Unbounded Join Operations**: Using `-j` to join sequences without specifying an output buffer size can cause memory exhaustion on systems with limited RAM, particularly when processing files with thousands of large sequences. The tool will crash with a memory allocation error.

- **Overwriting Output Files**: By default, a-liner overwrites existing output files without prompting. Running a command with `-o output.fasta` on a file that already exists will silently replace it. Always use `-oa` (append) mode or verify the output path does not exist before running.

- **Incorrect Index Usage**: Creating an index with a-liner-build is required before using `--index` for retrieval; attempting to use random-access lookups without a pre-built index will fail with an error. Index files have the `.ali` extension and are not backward compatible across versions.

## Examples

### Extract sequences longer than 500 base pairs from a FASTA file
**Args:** `-i input.fasta --minlen 500 -o long_sequences.fasta`
**Explanation:** The `--minlen` filter applies a hard length cutoff, retaining only sequences with 500+ bases, which is useful for filtering out short fragments or adapters.

### Convert a FASTQ file to FASTA format
**Args:** `-i reads.fastq -of fasta -o reads.fasta`
**Explanation:** Using `-of fasta` explicitly specifies the output format, discarding quality scores (since FASTA lacks quality data), useful when quality information is not needed downstream.

### Filter sequences matching a specific regex pattern
**Args:** `-i sequences.fasta --match "^ATG" --match "TGA$" -o complete_orfs.fasta`
**Explanation:** Multiple `--match` flags are combined with AND logic, keeping only sequences that both start with ATG (start codon) and end with TGA (stop codon), useful for identifying potential complete open reading frames.

### Extract specific sequences by identifier from a large file
**Args:** `--index sequences.ali --id gene1 --id gene2 --id gene3 -o selected.fasta`
**Explanation:** Using index files (created with a-liner-build) enables O(1) random access compared to linear scanning, dramatically faster when extracting few sequences from multi-gigabyte files.

### Append filtered sequences to an existing output file
**Args:** `-i additional.fasta --minlen 100 -oa results.fasta`
**Explanation:** The `-oa` (append) flag adds filtered records to the end of the output file without overwriting, useful for building a collection incrementally across multiple commands.

### Exclude sequences containing ambiguous bases
**Args:** `-i sequences.fasta --exclude "N+" -o clean_sequences.fasta`
**Explanation:** The `--exclude` flag removes any sequence containing one or more ambiguous base calls (N), which is critical before downstream analyses like variant calling that require high-quality sequences.

### Reverse complement all sequences in a file
**Args:** `-i input.fasta -rev -o reversed.fasta`
**Explanation:** The `-rev` flag performs both reversal and complement (A↔T, G↔C) for nucleotide sequences, essential for generating the reverse strand for alignment or primer design.

### Count the number of records in a FASTQ file
**Args:** `-i reads.fastq -c`
**Explanation:** The `-c` flag outputs only the record count without performing any transformation, useful for quick inventory of sequence files in pipelines.

### Extract sequences within a specific length range
**Args:** `-i sequences.fasta --minlen 200 --maxlen 1000 -o medium_sequences.fasta`
**Explanation:** Combining `--minlen` and `--maxlen` applies both lower and upper bounds, enabling filtering of sequences by size range, which is useful for selecting appropriately-sized fragments for assembly or PCR.

### Build an index for random access to a large file
**Args:** `-i massive.fa -build massive.ali`
**Explanation:** Using a-liner-build (or the `-build` flag) creates an `.ali` index file enabling fast `--id` lookups, essential for efficient extraction from files that are too large to scan repeatedly.