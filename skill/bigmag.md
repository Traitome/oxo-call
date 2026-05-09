---
name: bigmag
category: sequence_search
description: A tool for fast sequence similarity searches against an indexed database, commonly used in genomic and metagenomic analysis pipelines.
tags: [alignment, sequence-search, database, fast-search, genomics]
author: AI-generated
source_url: https://github.com/bigmag-suite/bigmag
---

## Concepts

- **Indexed Database Format**: bigmag operates on pre-built indexed databases created by the `bigmag-build` companion tool. The index enables fast k-mer based lookup rather than exhaustive pairwise alignment, allowing searches against large reference collections in O(n) or O(log n) time per query.
- **Input Formats**: Accepts FASTA, FASTQ, and plain text sequences as queries. Output formats include tabular alignments (SAM-like), BLAST-style reports, and JSON for downstream bioinformatics pipelines.
- **Scoring and Reporting**: Uses a substitution matrix (default: BLOSUM62-equivalent) with configurable gap penalties. Reports alignable sequences with alignment coordinates, eV E-values, and bit scores sorted by significance.
- **Parallel Execution**: Supports multi-threaded querying via `-t` flag, with thread count defaulting to available CPU cores if unspecified. Threading reduces wall-clock time for bulk query sets.

## Pitfalls

- **Querying an Unindexed FASTA File**: Running bigmag directly on a plain FASTA reference file without first building an index with bigmag-build results in a failure or extremely slow performance because the tool expects an indexed database structure. Always preprocess the reference with `bigmag-build - reference.fna -D database`.
- **Mismatched Sequence Types**: Searching nucleotide sequences against a protein-indexed database (or vice versa) produces meaningless results or errors. Ensure the database and query alphabets match, or use appropriate translation options if available.
- **Insufficient Memory for Large Indexes**: Loading a massive index into RAM without specifying `--memory-limited` mode causes out-of-memory errors on systems with constrained RAM. Use `-L` to employ a memory-mapped fallback that trades speed for reduced RAM usage.
- **Overwriting Existing Databases Unintentionally**: Running bigmag-build with an output path that already contains a database overwrites it silently. Always verify the target directory is empty or use a unique database name to prevent accidental data loss.

## Examples

### Search a single nucleotide query against a pre-built nucleotide database
**Args:** `query.fq -d nt_database -o results.sam -t 4`
**Explanation:** Searches nucleotide reads from query.fq against the nt_database using 4 threads, outputting alignments in SAM format.

### Build a custom index from a set of gene sequences
**Args:** `genes.faa -D gene_index -a protein -v`
**Explanation:** Creates a protein-mode indexed database from gene amino acid sequences with verbose logging during the build process.

### Limit results to high-confidence alignments only
**Args:** `reads.fq -d ref_db -o hits.txt -e 1e-5 -a`
**Explanation:** Returns only alignments with E-value ≤ 1e-5, filtering out marginal hits to reduce downstream analysis noise.

### Run memory-efficient search on a large database with limited RAM
**Args:** `queries.fna -d large_db -o out.tsv -L -t 2`
**Explanation:** Uses memory-mapped mode to search queries against a large reference while staying within constrained memory availability.

### Output alignments in JSON format for programmatic parsing
**Args:** `input.fa -d db -o results.json --outfmt json`
**Explanation:** Writes alignments in JSON instead of the default tabular format, enabling easier parsing in Python or JavaScript pipelines.

### Build a combined nucleotide database from multiple FASTA files
**Args:** `seq1.fna seq2.fna seq3.fna -D combined_nt -a dna`
**Explanation:** Aggregates multiple nucleotide FASTA files into a single searchable index withDNA alphabet mode for mixed-source references.