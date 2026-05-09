---
name: blast
category: sequence_alignment
description: NCBI BLAST (Basic Local Alignment Search Tool) suite for comparing gene or protein sequences against a database of known sequences. Supports nucleotide-nucleotide (blastn), protein-protein (blastp), and translated searches (blastx, tblastn, tblastx).
tags: [alignment, similarity_search, sequence_analysis, ncbi, homology]
author: AI-generated
source_url: https://www.ncbi.nlm.nih.gov/books/NBK153387/
---

## Concepts

- **Query-database model**: BLAST requires a query sequence (provided via `-query` in FASTA format) and a target database (specified via `-db`). Pre-formatted BLAST databases (e.g., nr, nt, swissprot) are available from NCBI, or custom databases can be built using the companion tool `makeblastdb`.
- **Statistical significance**: The expect value (`-evalue`) threshold controls sensitivity. Lower values (e.g., 0.001) return only high-confidence matches; higher values (e.g., 10.0) include weaker hits but increase false positives. For large databases like nr, values above 0.001 often yield excessive noise.
- **Output format flexibility**: The `-outfmt` flag controls output: 0 = pairwise alignment, 5 = tabular (best for parsing), 6 = extended tabular with comment lines, 11 = JSON. Tabular formats are machine-parseable and ideal for pipelines.
- **Word size and scoring**: Small word size (`-word_size`) increases sensitivity but slows search. The scoring matrix (e.g., BLOSUM62 for protein) and gap penalties (`-gapopen`, `-gapextend`) affect alignment quality and should match the evolutionary distance expected.
- **Companion binaries**: BLAST suite includes `blastn`, `blastp`, `blastx`, `tblastn`, `tblastx` for different search types, and `makeblastdb` for building custom databases, and `blastdbcmd` for extracting sequences.

## Pitfalls

- Using default `-evalue` of 10.0 on large databases like nr retrieves many biologically meaningless matches, wasting storage and complicating downstream analysis. Results often include spurious hits with minimal similarity.
- Neglecting to specify `-outfmt` produces human-readable text that is fragile across BLAST versions and difficult to parse programmatically. Tabular format (5 or 6) should be used for automation.
- Running translated searches (blastx, tblastn, tblastx) without understanding the computational cost can consume excessive resources. These involve six-frame translation and are inherently slower than direct nucleotide or protein searches.
- Not setting `-max_target_seqs` (or `-num_descriptions`/`-num_alignments` in older versions) may truncate results for common queries, silently dropping significant hits and leading to incomplete analyses.
- Using protein scoring matrices (e.g., BLOSUM62) with nucleotide searches ignores codon-aware scoring, potentially missing conserved protein domains in genomic sequences.

## Examples

### Search a protein query against a protein database using BLASTP
**Args:** `-query seq.fasta -db swissprot -out results.tsv -outfmt 6 -evalue 0.001 -max_target_seqs 20`
**Explanation:** This runs blastp to find up to 20 protein homologs with statistical significance better than 0.001, outputting results in extended tabular format for easy parsing.

### Build a custom BLAST database from a FASTA file
**Args:** `-in input.fasta -dbtype prot -title "MyProteinDB" -parse_seqids -out mydb`
**Explanation:** Creates a searchable protein database from a FASTA file, enabling subsequent BLAST searches with `-db mydb`.

### Run a fast nucleotide search with specific word size
**Args:** `-query gene.fasta -db nt -task blastn -word_size 11 -evalue 1e-10 -outfmt 5 -out hits.tsv`
**Explanation:** Uses the faster blastn word-based algorithm with a larger word size (11) for high-identity matches, suitable for closely related sequences.

### Perform a translated search (protein query against nucleotide database)
**Args:** `-query protein.fasta -db nt -task tblastn -evalue 0.0001 -outfmt 11 -out result.json`
**Explanation:** Translates the nucleotide database in all six frames and compares the protein query, outputting results in JSON format for programmatic processing.

### Search multiple query sequences in batch mode
**Args:** `-query queries.fasta -db nr -outfmt 5 -evalue 0.01 -num_threads 8 -out batch_results.tsv`
**Explanation:** Processes multiple FASTA entries in a single run using 8 CPU threads, outputting in tabular format for high-throughput analysis.

### Extract a specific sequence from a BLAST database
**Args:** `-db nr -entry all -outfmt fasta -range 100-500`
**Explanation:** Uses blastdbcmd to extract nucleotides 100-500 from the nr database entry in FASTA format.