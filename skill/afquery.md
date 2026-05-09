---
name: afquery
category: Genome Assembly / Database Queries
description: A command-line tool for querying AMOS format (.af) databases to extract reads, contigs, scaffolds, and other assembly elements based on flexible criteria.
tags:
  - assembly
  - database
  - query
  - amos
  - bioinformatics
  - extraction
author: AI-generated
source_url: https://github.com/mjsthompson/amos
---

## Concepts

- **AMOS Format Database**: The `.af` database is a binary or text-based repository that stores assembly elements including reads, contigs, scaffolds, and metadata. You must specify the database path using the `-d` flag.
- **Query Types**: afquery supports querying by element type (`-r` for reads, `-c` for contigs, `-s` for scaffolds), by ID or ID range, and by quality/coverage thresholds. Multiple query types can be combined.
- **Output Modes**: Results can be exported in multiple formats: plain text (`-o`), binary export, or specific formats like Fasta for sequences. The output mode is controlled via the `-o` flag and associated format specifiers.
- **Filtering and Constraints**: You can filter results using coverage (`-C`), read quality (`-q`), or read length (`-l`) thresholds. These filters are applied before output and reduce the result set.
- **Companion Tools**: afquery works alongside other AMOS tools like `afbuild` (database creation), `afdump` (full database export), and `makecons` (consensus generation).

## Pitfalls

- **Missing Database Flag**: Forgetting the `-d` flag causes afquery to look for a default database and fail with an unclear error message. Always explicitly specify the database path.
- **Confusing Output Formats**: Specifying an output format without the correct suffix or flag can produce empty or corrupted output. The output format must match the specified `-o` mode.
- **No Results Without Matching Criteria**: If your query criteria (e.g., coverage threshold too high) don't match any elements, afquery exits silently with no output. This is not an error but can be confusing if your threshold is unrealistic.
- **Incompatible Binary Databases**: Opening a database created with a different version of AMOS tools can cause read errors. Ensure the database was built with a compatible `afbuild` version.

## Examples

### Extract all reads from a database
**Args:** `-d assembly.af -r`
**Explanation:** The `-r` flag selects all read elements from the database specified with `-d`, outputting them in the default format.

### Query contigs within a specific ID range
**Args:** `-d assembly.af -c -i 100-500`
**Explanation:** Queries contigs (`-c`) whose IDs fall between 100 and 500, using the `-i` flag for ID range specification.

### Export reads in FASTA format with minimum length
**Args:** `-d assembly.af -r -o fasta -l 500`
**Explanation:** Outputs reads (`-r`) as FASTA sequences, filtering to only include reads with length at least 500bp using `-l`.

### Get scaffolds above a coverage threshold
**Args:** `-d assembly.af -s -C 10`
**Explanation:** Selects scaffolds (`-s`) with coverage of 10x or higher using the coverage filter `-C`.

### Extract a specific read by ID
**Args:** `-d assembly.af -r -i 12345`
**Explanation:** Retrieves a single read with exact ID 12345 from the database, useful for targeted verification.

### Output contigs to a file in text format
**Args:** `-d assembly.af -c -o text -O output_contigs.txt`
**Explanation:** Exports contigs (`-c`) in text format to a specific output file using `-O`, rather than writing to standard output.

### Query reads with high quality scores
**Args:** `-d assembly.af -r -q 30`
**Explanation:** Filters reads to only those with average quality scores of 30 or higher using the `-q` quality threshold flag.

### Combine multiple filters for reads
**Args:** `-d assembly.af -r -l 1000 -q 20 -C 5`
**Explanation:** Applies multiple filters simultaneously: reads must be at least 1000bp long (`-l`), have quality 20+ (`-q`), and coverage 5x or more (`-C`).