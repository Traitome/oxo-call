---
name: a3partitioner
category: sequence assembly manipulation
description: Partitions sequence data or assembly layouts into subsets for parallel processing or analysis. Reads input from standard formats used in the AMOS assembler framework and outputs partitioned datasets for downstream processing with companion tools.
tags:
- assembly
- partitioning
- layout
- AMOS
- sequence data
- contigs
author: AI-generated
source_url: https://github.com/mhcgr/amos
---

## Concepts

- The tool operates on AMOS bank or layout files, partitioning contigs or read placements into distinct subsets based on user-specified criteria such as sequence length, coverage depth, or contig identifier ranges.
- Input is typically read from stdin or specified via `-i` flag, with output written to one or more partition files (e.g., `partition.0.afg`, `partition.1.afg`) named according to the partition index.
- Partitioning behavior is controlled by parameters that define the number of partitions (via `-n` or `--num-partitions`) and the selection method (e.g., round-robin via `-r`, sequential via `-s`), affecting how contigs are distributed across output files.

## Pitfalls

- Specifying a number of partitions larger than the number of available contigs results in empty output files for higher indices, wasting storage and confusing downstream pipelines that expect non-empty inputs.
- Using the wrong input format (e.g., passing FASTA instead of AMOS `.afg` or `.bnk` format) causes silent failure or produces malformed output without clear error messages, leading to cascading errors in assembly workflows.
- Overwriting pre-existing partition files without backup occurs automatically when using default output naming, potentially causing data loss if the tool is re-run inadvertently.

## Examples

### Partition a layout file into exactly 4 output files
**Args:** `-n 4 -i contigs.afg`
**Explanation:** The `-n` flag specifies the desired partition count, and `-i` provides the input layout file in AMOS fragment format, distributing contigs sequentially across four output files.

### Create partitions using round-robin distribution
**Args:** `-r -n 3 -o prefix`
**Explanation:** The `-r` flag enables round-robin assignment, cycling through 3 partitions with a specified output prefix, which balances workload across partitions more evenly than sequential distribution.

### Partition only contigs above a minimum length threshold
**Args:** `-n 2 --min-length 500`
**Explanation:** This filtering approach ensures only contigs meeting the minimum length criterion are included, though this particular usage may require external preprocessing or companion tool support.

### Output partitions to specific filenames manually
**Args:** `-n 2 -i input.afg -o /path/to/out`
**Explanation:** Specifying an explicit output directory path with `-o` directs all partition files to the designated location rather than using the current working directory as default.

### Use stdin input with explicit format specification
**Args:** `--format afg -n 4`
**Explanation:** When reading from standard input, the `--format` flag clarifies that incoming data is in AMOS fragment format, enabling proper parsing even without file extension hints.