---
name: diamond
category: metagenomics
description: Ultra-fast protein and translated DNA database search tool — 500-20000x faster than BLAST with comparable sensitivity
tags: [blast, protein, database-search, metagenomics, annotation, alignment, nr, uniprot]
author: oxo-call built-in
source_url: "https://github.com/bbuchfink/diamond"
---

## Concepts

- DIAMOND has two main modes: diamond blastp (protein vs protein DB) and diamond blastx (DNA vs protein DB).
- Build a DIAMOND database from FASTA: diamond makedb --in proteins.faa -d database_name.
- Use -q for query, -d for database, -o for output; output format 6 is tabular (like BLAST -outfmt 6).
- Use --threads N for parallelization; --more-sensitive or --sensitive for higher sensitivity (slower).
- Output format 6 columns: qseqid sseqid pident length mismatch gapopen qstart qend sstart send evalue bitscore.
- Use --evalue to set E-value cutoff (default 0.001); --id for minimum percent identity filter.
- DIAMOND supports --outfmt 6 with custom fields: --outfmt 6 qseqid sseqid pident length evalue bitscore stitle.
- Use --top N to report top N hits; --max-target-seqs N to limit number of hits per query.

## Pitfalls

- DIAMOND database must be built with 'diamond makedb' — cannot use BLAST databases directly.
- For metagenomics, use blastx mode (translated search) for protein function annotation of DNA reads.
- Without --more-sensitive, DIAMOND may miss some hits compared to BLAST — use for higher-accuracy searches.
- --max-target-seqs 1 only keeps the best hit per query; use higher values for multi-hit analysis.
- The -d database argument does NOT include the .dmnd extension — just the base path.
- DIAMOND's default memory usage can be high for large databases — use -b (block size) to reduce RAM.

## Examples

### build a DIAMOND protein database from a FASTA file
**Args:** `makedb --in nr.faa -d nr_diamond --threads 8`
**Explanation:** --in protein FASTA; -d output database prefix; creates nr_diamond.dmnd

### search protein sequences against a DIAMOND database (blastp)
**Args:** `blastp -q proteins.faa -d nr_diamond -o blastp_results.tsv --outfmt 6 --threads 8 --evalue 1e-5`
**Explanation:** -q query proteins; -d database; -o output; --outfmt 6 tabular; --evalue E-value cutoff

### search DNA reads against protein database using blastx (translated search)
**Args:** `blastx -q reads.fastq.gz -d nr_diamond -o blastx_results.tsv --outfmt 6 --threads 16 --evalue 1e-5 --max-target-seqs 1`
**Explanation:** blastx translates DNA to protein in all 6 frames; --max-target-seqs 1 keeps best hit per read

### sensitive mode search with custom output fields
**Args:** `blastp -q proteins.faa -d uniprot_diamond -o detailed_results.tsv --outfmt '6 qseqid sseqid pident length evalue bitscore stitle' --more-sensitive --threads 8`
**Explanation:** --more-sensitive for higher accuracy; custom --outfmt includes stitle (subject description)

### search with taxonomy-aware output for functional annotation
**Args:** `blastx -q metagenome.faa -d nr_diamond --taxonmap prot.accession2taxid.gz --taxonnodes nodes.dmp -o results_tax.tsv --outfmt '6 qseqid sseqid pident evalue bitscore staxids sscinames' --threads 16`
**Explanation:** --taxonmap and --taxonnodes enable taxonomy annotation; staxids/sscinames in output
