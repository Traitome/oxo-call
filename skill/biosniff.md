---
name: biosniff
category: Bioinformatics Data Extraction
description: A command-line tool for sniffing, extracting, and filtering data from common bioinformatics file formats. Supports streaming operations on FASTA, FASTQ, VCF, and BED files with built-in pattern matching and field extraction capabilities.
tags: [bioinformatics, data-extraction, fasta, fastq, vcf, bed, text-processing, streaming, cli]
author: AI-generated
source_url: https://github.com/biosniff/biosniff
---

## Concepts

- **Streaming Line-by-Line Processing**: biosniff processes input files sequentially line-by-line, making it memory-efficient for large genomics files (e.g., multi-GB FASTQ files). Use with piped input or file arguments.
- **Format-Specific Parsing**: Automatic detection of input format (FASTA, FASTQ, VCF, BED) based on file extension or magic bytes. FASTA/Q records span multiple lines; biosniff assembles them into complete records before output.
- **Pattern Matching with Regex**: Supports regular expression matching against sequence identifiers, quality scores, and variant columns. Patterns are applied per-record after assembly.
- **Field Extraction via Column or Position**: For structured formats (VCF, BED), specify 1-based column indices (e.g., COL5 for chromosome). For FASTQ, use --seq, --qual, or --id flags.

## Pitfalls

- **Forgetting Multi-line Record Assembly**: FASTQ/FASTA records span multiple lines; without proper handling, piping directly to other tools breaks records. biosniff handles this internally but passing raw lines to external tools loses context.
- **Using 0-based Column Indices**: VCF and BED use 1-based column indexing in standard specifications. Using 0-based indices (common in programming) extracts the wrong column, leading to missing or incorrect data.
- **Large Regex Without Anchoring**: Unanchored regex patterns (e.g., `ATCGN`) match anywhere in the sequence, causing false positives. Use anchors like `^` and `$` for exact matching or enable --exact-mode.
- **Ignoring Compressed Files**: biosniff does not auto-detect .gz/.bz2 compression. Decompress files first using `gzip -dc` or `bzcat` before piping to biosniff, or use the --compressed flag if supported.

## Examples

### Extract sequences from a FASTQ file matching a prefix
**Args:** `--format fastq --id-pattern "^SRR" input.fastq`
**Explanation:** Uses regex `^SRR` anchored at the start of the read identifier to only output records from Sequence Read Archive samples beginning with "SRR".

### Filter VCF variants by QUAL field threshold
**Args:** `--format vcf --col-qual 6 --numeric-gte 50 input.vcf`
**Explanation:** Extracts VCF records where the QUAL (quality) column (column 6) is greater than or equal to 50, removing low-confidence variant calls.

### Output only sequence lines from multi-line FASTQ
**Args:** `--format fastq --seq-only large_dataset.fastq`
**Explanation:** Uses the built-in --seq-only flag to extract only the DNA/RNA sequence lines from each FASTQ record, collapsing multi-line sequences into single lines.

### Extract BED regions on chromosome 1
**Args:** `--format bed --col-chr 1 --eq "chr1" hotspots.bed`
**Explanation:** Filters BED file to only regions in the chromosome column (column 1) that exactly match "chr1", useful for whole-genome analyses needing chromosome-specific subsets.

### Count records matching a regex without outputting
**Args:** `--format fasta --id-pattern "^[A-Z]{2,3}" --count-only sequences.fa`
**Description:** Runs in count-only mode to tally FASTA records with 2-3 letter uppercase accession prefixes without printing records, useful for quick data profiling.