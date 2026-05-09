---
name: allelecodes
category: sequence_analysis
description: A tool for extracting and managing allele codes from DNA sequences, commonly used in bacterial genomics for Multi-Locus Sequence Typing (MLST) analysis. Converts sequence data to allelic designations and supports allele profile matching against known databases.
tags: [mlst, allele-codes, sequence-typing, bacterial-genomics, genotyping]
author: AI-generated
source_url: https://github.com/kflabba/allelecodes
---

## Concepts

- **Input Format**: Accepts FASTA or FASTQ format sequence files containing DNA fragments from bacterial isolates. Each sequence should represent a gene locus used for MLST (e.g., arcC, aroE, asd, etc.). The tool parses sequence headers to identify isolate and locus information.
  
- **Allele Matching**: Compares input sequences against a reference allele database to assign allele numbers. Sequences are considered exact matches only if they are 100% identical to a reference allele; partial matches or novel sequences receive new allele designations or are flagged as unknown.

- **Output Types**: Generates tab-delimited tables with isolate IDs, locus names, and assigned allele numbers. Supports verbose output including sequence details, alignment scores, and allele definitions. Can export allele profiles in standard MLST formats (e.g., CSV, JSON, or legacy text formats).

- **Database Management**: Uses an allele database file (typically named ` alleles.txt` or similar) containing known allele sequences with their designated numbers. The database must be indexed or built before running analyses. Users can maintain multiple gene-specific databases.

- **Companion Binaries**: The toolset includes `allelecodes-build` for creating or updating allele databases from FASTA inputs. Running `allelecodes-build` is required before querying new gene loci or adding novel alleles to the reference database.

## Pitfalls

- **Missing Database File**: Running `allelecodes` without a valid allele database produces empty or errored output. The tool will fail to match any sequences if the database path is incorrect, not specified, or the database file lacks entries for the queried loci.

- **Sequence Quality Issues**: Low-quality sequences with ambiguous bases (N characters) or containing insertion/deletion mutations may fail matching or be incorrectly assigned. Using unfiltered raw reads instead of assembled contigs results in high rates of failed allele calls.

- **Locus Name Mismatches**: Sequence headers must contain exact locus names matching the database keys (e.g., ` isolate1_arcC` vs. `arcC`). Headers with typos, missing locus identifiers, or inconsistent naming cause zero matches even with correct sequences.

- **Database Version Incompatibility**: Using an outdated allele database with the current tool version produces inconsistent allele numbers across analyses. Allele designations can change between database releases as new alleles are discovered and assigned.

- **Memory Limits with Large Datasets**: Processing thousands of sequences against databases with hundreds of alleles per locus consumes significant memory. Large batch jobs may fail on systems with insufficient RAM without adjusting memory allocation settings.

## Examples

### Extract allele codes from a single gene FASTA file
**Args:** `-i genes.fasta -d alleles.txt -o results.txt`
**Explanation:** This runs allele code extraction on sequences in `genes.fasta` using the allele database `alleles.txt` and writes matching allele numbers to `results.txt`.

### Query multiple genes with verbose output
**Args:** `-i all_genes.fasta -d alleles.txt --verbose`
**Explanation:** Runs matching on a multi-gene FASTA file with verbose output enabled, showing detailed allele information for each sequence including alignment scores and source database entries.

### Build a new allele database from reference sequences
**Args:** `build -i ref_sequences.fasta -o new_alleles.txt -l arcC aroE asd`
**Explanation:** Uses the `build` companion binary to create a new allele database named `new_alleles.txt` from reference sequences, specifying three locus names as database keys.

### Export results in JSON format
**Args:** `-i sequences.fasta -d database.txt --json -o profile.json`
**Explanation:** Outputs allele assignments in JSON format instead of the default tabular format, useful for integration with downstream pipelines or databases.

### Append novel alleles to existing database
**Args:** `build -i novel_alleles.fasta -d existing_alleles.txt -a -o updated.txt`
**Explanation:** Uses the `build` binary with the append flag (`-a`) to add newly discovered alleles to an existing database, preserving all previous entries while adding novel sequence designations.

### Run with custom minimum sequence length filter
**Args:** `-i input.fasta -d alleles.txt --min-length 400`
**Explanation:** Filters out sequences shorter than 400 bp before matching, useful when working with fragmented assemblies where short contigs may produce unreliable allele calls.

### Batch process a directory of isolate files
**Args:** `-i ./isolates/ -d db.txt -o batch_results.txt`
**Explanation:** Processes all sequence files in a directory against the database, outputting combined allele profiles for multiple isolates in a single results file.