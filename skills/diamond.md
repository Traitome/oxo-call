---
name: diamond
category: metagenomics
description: Ultra-fast protein and translated DNA database search tool — 500-20000x faster than BLAST with comparable sensitivity
tags: [blast, protein, database-search, metagenomics, annotation, alignment, nr, uniprot, blastp, blastx, clustering, linclust]
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
- Sensitivity modes (fastest to most sensitive): --faster, --fast, default, --sensitive, --more-sensitive, --very-sensitive, --ultra-sensitive.
- Clustering modes: cluster (greedy clustering), linclust (linear time clustering for very large datasets).
- --block-size (-b) controls memory usage; default is 2 (GB), increase for faster performance on high-memory systems.
- --index-chunks (-c) controls index splitting; default is 4, reduce for lower memory usage.
- --outfmt 100 outputs DAA format for later conversion; --outfmt 101 outputs SAM format.

## Pitfalls

- diamond ARGS must start with a subcommand (makedb, blastp, blastx, cluster, linclust, realign, recluster, reassign, view, merge-daa, getseq, dbinfo, makeidx) — never with flags like -q, -d, -o. The subcommand ALWAYS comes first.
- DIAMOND database must be built with 'diamond makedb' — cannot use BLAST databases directly.
- For metagenomics, use blastx mode (translated search) for protein function annotation of DNA reads.
- Without --more-sensitive, DIAMOND may miss some hits compared to BLAST — use for higher-accuracy searches.
- --max-target-seqs 1 only keeps the best hit per query; use higher values for multi-hit analysis.
- The -d database argument does NOT include the .dmnd extension — just the base path.
- DIAMOND's default memory usage can be high for large databases — use -b (block size) to reduce RAM.
- --top overrides --max-target-seqs; use --top 10 for top 10% of best score hits.
- Default --evalue 0.001 may be too stringent for distant homologs; increase to 1 or 10 for remote homology.
- --ultra-sensitive provides BLAST-like sensitivity but is much slower; use only when necessary.
- For very large query files, use --block-size to control memory; each block is loaded into RAM.
- DAA format (--outfmt 100) is DIAMOND-specific; convert to other formats with diamond view.

## Examples

### build a DIAMOND protein database with makedb
**Args:** `makedb --in nr.faa -d nr_diamond --threads 8`
**Explanation:** --in protein FASTA; -d output database prefix; creates nr_diamond.dmnd

### search protein sequences against a DIAMOND database with blastp
**Args:** `blastp -q proteins.faa -d nr_diamond -o blastp_results.tsv --outfmt 6 --threads 8 --evalue 1e-5`
**Explanation:** -q query proteins; -d database; -o output; --outfmt 6 tabular; --evalue E-value cutoff

### search DNA reads against protein database with blastx
**Args:** `blastx -q reads.fastq.gz -d nr_diamond -o blastx_results.tsv --outfmt 6 --threads 16 --evalue 1e-5 --max-target-seqs 1`
**Explanation:** blastx translates DNA to protein in all 6 frames; --max-target-seqs 1 keeps best hit per read

### sensitive blastp search with custom output fields
**Args:** `blastp -q proteins.faa -d uniprot_diamond -o detailed_results.tsv --outfmt '6 qseqid sseqid pident length evalue bitscore stitle' --more-sensitive --threads 8`
**Explanation:** --more-sensitive for higher accuracy; custom --outfmt includes stitle (subject description)

### taxonomy-aware blastx search for functional annotation
**Args:** `blastx -q metagenome.faa -d nr_diamond --taxonmap prot.accession2taxid.gz --taxonnodes nodes.dmp -o results_tax.tsv --outfmt '6 qseqid sseqid pident evalue bitscore staxids sscinames' --threads 16`
**Explanation:** --taxonmap and --taxonnodes enable taxonomy annotation; staxids/sscinames in output

### ultra-sensitive search for distant homologs
**Args:** `blastp -q proteins.faa -d nr_diamond -o ultra_sensitive.tsv --outfmt 6 --ultra-sensitive --threads 16`
**Explanation:** --ultra-sensitive provides BLAST-like sensitivity for detecting distant homologs; much slower than default

### memory-optimized search for large databases
**Args:** `blastp -q proteins.faa -d nr_diamond -o results.tsv --outfmt 6 --threads 8 --block-size 1 --index-chunks 8`
**Explanation:** --block-size 1 limits RAM to ~1GB; --index-chunks 8 reduces memory further at cost of speed

### cluster protein sequences with CD-HIT-like algorithm
**Args:** `cluster -d proteins.faa -o clusters.tsv --approx-id 50 --threads 16`
**Explanation:** diamond cluster for protein clustering; --approx-id 50 for 50% identity threshold (like CD-HIT)

### linear time clustering for very large datasets
**Args:** `linclust -d proteins.faa -o linclusters.tsv --approx-id 50 --threads 16`
**Explanation:** linclust is faster than cluster for very large datasets; uses linear time algorithm

### output in SAM format for downstream analysis
**Args:** `blastx -q reads.fastq -d nr_diamond -o aligned.sam --outfmt 101 --threads 16`
**Explanation:** --outfmt 101 outputs SAM format; compatible with samtools and other SAM-processing tools

### save results in DAA format for later conversion
**Args:** `blastp -q proteins.faa -d nr_diamond -o results.daa --outfmt 100 --threads 16`
**Explanation:** --outfmt 100 outputs binary DAA format; space-efficient and can be converted later with diamond view

### convert DAA to BLAST tabular format
**Args:** `view -a results.daa -o results.tsv --outfmt 6`
**Explanation:** diamond view converts DAA files to other formats; useful for post-processing archived results

### get database information
**Args:** `dbinfo -d nr_diamond`
**Explanation:** dbinfo prints database statistics including number of sequences, letters, and taxonomy info
