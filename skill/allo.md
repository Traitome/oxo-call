---
name: allo
category: Sequence Alignment
description: allo is a fast exact-match read alignment tool that uses efficient indexing for mapping short reads to a reference genome. It constructs a suffix array-based index for rapid exact lookups and outputs alignments in SAM format.
tags: [alignment, read-mapping, suffix-array, genomics, fastq, fasta, sam]
author: AI-generated
source_url: https://github.com/EXAMPLE/allo
---

## Concepts

- **Index Construction**: allo builds an index from a reference FASTA file using the companion tool `allo-build`, which creates a suffix array structure for O(m) exact-match lookup of reads of length m.
- **Input Formats**: allo accepts single-end or paired-end reads in FASTA or FASTQ format. Reads must be provided via stdin or a specified input file; the format is auto-detected based on file contents.
- **Output Format**: Alignments are output in SAM (Sequence Alignment/Map) format by default, with one line per read containing the query name, flag, reference name, position, mapping quality, CIGAR string, and optional tags.
- **Exact-Match Only**: allo performs exact-match alignment only—no mismatches, insertions, or deletions are allowed. Reads that do not match exactly at any position are reported as unmapped.
- **Multi-mapping Handling**: The `-k` parameter controls the maximum number of reported alignments per read. Without `-k`, allo reports the first valid alignment only.

## Pitfalls

- **Using Exact-Match for SNP-Tolerant Alignment**: Specifying reads with known SNPs or sequencing errors will result in all those reads being reported as unmapped, since allo does not allow mismatches or indels.
- **Forgetting to Build an Index**: Running `allo` without first constructing an index with `allo-build` will cause the tool to fail with an error about a missing index file; always run `allo-build` on the reference first.
- **Specifying the Wrong Number of Threads**: Using `-t 0` or a negative value will cause allo to fail; thread counts must be positive integers.
- **Assuming Mixed Case Insensitivity**: Reference sequences in the FASTA file must be consistently cased—mixing upper and lower case may cause alignment failures or missed matches.
- **Ignoring Unmapped Reads in Downstream Analysis**: Unmapped reads are reported in SAM with flag 4 set; pipelines that filter by mapping quality alone will incorrectly include these as "valid" unmapped reads.

## Examples

### Build an index from a reference genome FASTA file
**Args:** `ref.fa allo-index`
**Explanation:** This runs the companion tool `allo-build` to construct a suffix array index from the reference FASTA file `ref.fa`, producing an index file named `allo-index` for use by `allo`.

### Align single-end reads to an indexed reference
**Args:** `-x allo-index -U reads.fq -S`
**Explanation:** This aligns single-end reads from `reads.fq` against the indexed reference `allo-index`, outputting in SAM format to stdout. The `-S` flag explicitly specifies single-end input mode.

### Align paired-end reads to an indexed reference
**Args:** `-x allo-index -1 reads1.fq -2 reads2.fq`
**Explanation:** This aligns paired-end read pairs from `reads1.fq` and `reads2.fq` to the reference, producing paired-end alignments with proper mate information in the SAM output.

### Limit to 3 reported alignments per read
**Args:** `-x allo-index -U reads.fq -k 3`
**Explanation:** This outputs up to 3 alignments per read when multiple exact-match positions exist, allowing detection of multi-mapping reads while limiting output size.

### Use 8 threads for parallel alignment
**Args:** `-x allo-index -U reads.fq -t 8`
**Explanation:** This runs alignment with 8 parallel threads to improve throughput on multi-core systems, reducing runtime for large read sets.

### Output alignments to a named SAM file
**Args:** `-x allo-index -U reads.fq -o alignments.sam`
**Explanation:** This writes the SAM-formatted alignments to `alignments.sam` instead of stdout, allowing redirection for downstream processing or piping to tools like `samtools`.

### Suppress header lines in SAM output
**Args:** `-x allo-index -U reads.fq --noheader`
**Explanation:** This outputs alignment lines only without the SAM header section (starting with `@`), useful when piping directly to tools that expect alignment records only.