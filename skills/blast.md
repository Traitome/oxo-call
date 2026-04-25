---
name: blast
category: sequence-utilities
description: Basic Local Alignment Search Tool — search protein or nucleotide databases for sequence similarity
tags: [blast, alignment, database-search, nucleotide, protein, ncbi, homology, makeblastdb, blastdbcmd]
author: oxo-call built-in
source_url: "https://blast.ncbi.nlm.nih.gov/doc/blast-help/"
---

## Concepts

- BLAST+ suite includes: blastn (nucleotide vs nucleotide), blastp (protein vs protein), blastx (translated DNA vs protein), tblastn (protein vs translated DNA), tblastx (translated vs translated), psiblast (iterative position-specific), deltablast (domain-enhanced).
- Build a BLAST database with makeblastdb: makeblastdb -in sequences.fasta -dbtype nucl -out db_name.
- Use -query for query file; -db for database; -out for output; -outfmt 6 for tabular output.
- tabular format 6 columns: qseqid sseqid pident length mismatch gapopen qstart qend sstart send evalue bitscore.
- Use -evalue to set E-value threshold (default 10); -max_target_seqs to limit hits per query.
- -num_threads N for parallel search; -perc_identity for minimum percent identity filter.
- Remote BLAST against NCBI databases: add -remote flag (slower but no local database needed).
- outfmt '6 std stitle staxids' adds title and taxonomy to standard tabular output.
- blastn default task is 'megablast' (optimized for near-identical sequences); use -task blastn for traditional blastn or -task dc-megablast for cross-species megablast.
- -subject allows searching against a FASTA file directly without building a database (incompatible with -db, -taxids).
- blastdbcmd retrieves sequences from BLAST databases by accession or taxonomy (-entry, -taxids).
- -qcov_hsp_perc filters by query coverage percentage; -culling_limit removes redundant hits enveloped by better ones.
- -taxids and -negative_taxids restrict search to/exclude specific taxonomy IDs (requires taxid-indexed database or -remote).

## Pitfalls

- BLAST has NO single 'blast' command. Each tool is a separate binary: blastn, blastp, blastx, tblastn, tblastx for searching; makeblastdb for building databases; blastdbcmd for retrieval. ARGS for search tools start with flags like -query, -db, -out — never with a subcommand. For makeblastdb, ARGS start with -in, -dbtype, -out.
- BLAST database must be built with makeblastdb before searching (except -remote or -subject).
- -max_target_seqs 1 with tabular output may not always return the BEST hit — it returns the first found. Use -culling_limit 1 for best non-redundant hit.
- blastn default task is 'megablast' with word size 28 — too strict for short sequences (<100 bp) or distant homologs. Use -task blastn-short for <30 bp queries, -task blastn for traditional search, or -task dc-megablast for cross-species.
- The -db argument is the database PREFIX, not the full file name (without .nhr/.nin etc.).
- BLAST can be very slow on large databases without -num_threads; use DIAMOND for large protein databases.
- outfmt 5 (XML) is verbose but machine-readable; outfmt 6 is most commonly used for scripting.
- -subject is incompatible with -taxids, -gilist, -db_soft_mask, -db_hard_mask, and -remote.
- For blastn, the default -reward/-penalty is 2/-3 for megablast but 1/-2 for traditional blastn — changing -task also changes these defaults.

## Examples

### build a nucleotide BLAST database from a FASTA file
**Args:** `makeblastdb -in genome.fasta -dbtype nucl -out genome_db -title 'Genome Database' -parse_seqids`
**Explanation:** makeblastdb tool; -in specifies input FASTA; -dbtype nucl for nucleotide (prot for protein); -out database prefix; -title sets database title; -parse_seqids enables sequence retrieval by ID

### run blastn to find similar nucleotide sequences
**Args:** `blastn -query query.fasta -db genome_db -out blast_results.txt -outfmt 6 -evalue 1e-5 -num_threads 8`
**Explanation:** blastn tool; -query specifies query file; -db is the database; -out writes results; -outfmt 6 tabular output; -evalue 1e-5 threshold; -num_threads 8 parallel search

### search protein sequences against NR database
**Args:** `blastp -query proteins.faa -db /path/to/nr -out blastp_results.txt -outfmt '6 std stitle staxids' -evalue 1e-5 -num_threads 16 -max_target_seqs 5`
**Explanation:** blastp tool; -query specifies query file; -db nr (NCBI non-redundant); -out writes results; -outfmt '6 std stitle staxids' adds title and taxonomy; -evalue 1e-5 threshold; -num_threads 16 parallel; -max_target_seqs 5 top hits per query

### run blastx to annotate nucleotide sequences against protein database
**Args:** `blastx -query contigs.fasta -db /path/to/swissprot -out blastx_results.txt -outfmt 6 -evalue 1e-5 -num_threads 8 -max_target_seqs 1`
**Explanation:** blastx tool; -query specifies query file; blastx translates query DNA in all 6 frames; -db swissprot for curated protein annotations; -out writes results; -outfmt 6 tabular; -evalue 1e-5 threshold; -num_threads 8 parallel; -max_target_seqs 1 top hit

### perform remote BLAST search against NCBI nr database
**Args:** `blastn -query query.fasta -db nr -out remote_blast.txt -outfmt 6 -remote -max_target_seqs 10`
**Explanation:** blastn tool; -query specifies query file; -db nr database; -out writes results; -outfmt 6 tabular; -remote searches NCBI servers directly; -max_target_seqs 10 hits per query; no local database needed; slower than local search

### search against a FASTA file without building a database
**Args:** `blastn -query query.fasta -subject target.fasta -out results.txt -outfmt 6 -evalue 1e-5`
**Explanation:** blastn tool; -query specifies query file; -subject replaces -db; target.fasta input; -out writes results; -outfmt 6 tabular; -evalue 1e-5 threshold; no makeblastdb needed; incompatible with -taxids, -remote; good for quick one-off searches

### use traditional blastn instead of megablast for distant homologs
**Args:** `blastn -task blastn -query query.fasta -db genome_db -out results.txt -outfmt 6 -evalue 1e-10 -word_size 11`
**Explanation:** blastn tool; -task blastn uses traditional algorithm (default is megablast); -query specifies query; -db is database; -out writes results; -outfmt 6 tabular; -evalue 1e-10 threshold; -word_size 11 for sensitivity; use -task dc-megablast for cross-species megablast

### search short sequences (<30 bp) with blastn-short
**Args:** `blastn -task blastn-short -query primers.fasta -db genome_db -out results.txt -outfmt 6 -evalue 1000 -word_size 7`
**Explanation:** blastn tool; -task blastn-short optimized for queries <30 bp; -query specifies query; -db is database; -out writes results; -outfmt 6 tabular; -evalue 1000 higher threshold for short queries; -word_size 7 for maximum sensitivity

### retrieve sequences from a BLAST database by accession
**Args:** `blastdbcmd -db genome_db -entry NM_001234 -out retrieved.fa`
**Explanation:** blastdbcmd tool; -db specifies database; -entry specifies sequence identifier; -out writes output; use -entry_batch for multiple IDs from a file; -taxids to retrieve by taxonomy

### filter BLAST results by taxonomy
**Args:** `blastn -query query.fasta -db nt -out results.txt -outfmt 6 -taxids 9606 -evalue 1e-5 -remote`
**Explanation:** blastn tool; -query specifies query file; -db nt database; -out writes results; -outfmt 6 tabular; -taxids 9606 restricts to human (taxonomy ID); -evalue 1e-5 threshold; -remote enables remote search; -negative_taxids to exclude; requires -remote or taxid-indexed database

### filter by query coverage and percent identity
**Args:** `blastn -query query.fasta -db genome_db -out results.txt -outfmt 6 -qcov_hsp_perc 80 -perc_identity 95 -evalue 1e-5`
**Explanation:** blastn tool; -query specifies query file; -db is database; -out writes results; -outfmt 6 tabular; -qcov_hsp_perc 80 requires ≥80% query coverage; -perc_identity 95 requires ≥95% identity; -evalue 1e-5 threshold; combined filters ensure high-quality matches
