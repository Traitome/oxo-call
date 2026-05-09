---
name: blast-legacy
category: Sequence Alignment / Similarity Search
description: Legacy BLAST (Basic Local Alignment Search Tool) executable for nucleotide and protein similarity searches using the classic BLAST 2.0 algorithm and database format. Supports offline database searches without requiring NCBI account authentication.
tags: [blast, sequence alignment, similarity search, nucleotide, protein, fasta, legacy]
author: AI-generated
source_url: https://www.ncbi.nlm.nih.gov/books/NBK153387/
---

## Concepts

- **Input formats**: Accepts FASTA, FASTQ, GenBank, and raw sequences (with -parse_seqids option) as queries. Multi-sequence queries can be provided in a single file or as multiple entries separated by headers.
- **Output formats**: Produces alignment results in multiple formats including tabular (-outfmt 6/7), XML, ASN.1, and human-readable BLAST report. Default output goes to stdout; use -out to specify a file.
- **Database types**: Works with legacy-format BLAST databases created by formatdb (legacy makeblastdb). Databases must be pre-built with matching sequence type (nucleotide or protein) and appropriate -parse_seqids setting.
- **Scoring parameters**: Uses BLOSUM62 for protein searches (default) and +4/-5 scoring for nucleotide searches. E-value threshold defaults to 10.0; lower values increase stringency.
- **Task variants**: Uses -task flag to specify preset parameters: blastn, blastn-short, dc-megablast, megablast, rmblastn, blastp, blastx, tblastn, tblastx.

## Pitfalls

- **Database format mismatch**: Using a database created with the newer makeblastdb format (BLAST 2.2.31+) with legacy BLAST results in "Error: BLAST Database [name] does not exist or is unreadable" because legacy formatdb cannot read the new database format.
- **Query sequence contamination**: Failing to mask low-complexity regions (using -dust or -seg) produces spurious high-scoring alignments to repetitive elements, inflating e-values and wasting computation time.
- **E-value misinterpretation**: Setting a very high e-value threshold (e.g., 1000) returns millions of irrelevant alignments, making results unusable for downstream analysis and consuming excessive memory.
- **Missing parse_seqids**: Searching databases created without -parse_seqids prevents retrieval of subject sequence identifiers from the results, breaking workflows that require sequence ID extraction.
- **Incorrect task preset**: Running blastp on nucleotide sequences (or vice versa) produces zero results because the algorithm expects translated query-to-subject matching.

## Examples

### Search a nucleotide query against a local nucleotide database
**Args:** -query query.fasta -db nr_nucleotide -out results.tsv -evalue 1e-10 -outfmt 6 -task blastn
**Explanation:** Runs a nucleotide BLAST search using the blastn algorithm with stringency set to e-value 1e-10, outputting tabular results for efficient parsing.

### Find protein homologs using Protein BLAST with statistical correction
**Args:** -query protein_seq.fasta -db nr_protein -out homologs.txt -evalue 0.001 -outfmt 7 -html -task blastp
**Explanation:** Searches protein sequences against the protein database with corrected statisticaloutput in HTML format for web browser viewing.

### Translated search of nucleotide query against protein database
**Args:** -query nucleotide.fasta -db nr_protein -out translated.tsv -evalue 1e-5 -outfmt 6 -task blastx
**Explanation:** Performs a six-frame translation of the input nucleotide query and searches against the protein database, identifying potential coding regions.

### Search with low-complexity filtering enabled
**Args:** -query repeats.fasta -db refseq_viral -out clean.tsv -evalue 0.01 -outfmt 6 -dust yes -task blastn
**Explanation:** Applies low-complexity masking to filter out repetitive and low-complexity regions before alignment, producing biologically meaningful results.

### Run megablast for highly similar sequences with increased word size
**Args:** -query strain.fasta -db assembly -out matches.tsv - evalue 0.1 -outfmt 6 -task megablast -word_size 16
**Explanation:** Uses the megablast algorithm optimized for very similar sequences (90%+ identity) with a larger word size (16 vs default 11) for faster execution.

### Generate ASN.1 output for programmatic parsing
**Args:** -query seq.fasta -db custom -out results.asn -evalue 1 -outfmt 1 -task blastp
**Explanation:** Outputs results in ASN.1 format for programmatic parsing by downstream bioinformatics tools that require structured binary data.