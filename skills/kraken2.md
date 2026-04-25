---
name: kraken2
category: metagenomics
description: Ultrafast and highly accurate taxonomic classification of sequencing reads using exact k-mer matches
tags: [metagenomics, taxonomy, classification, kraken, microbiome, ngs, k-mer]
author: oxo-call built-in
source_url: "https://github.com/DerrickWood/kraken2"
---

## Concepts
- Kraken2 classifies sequencing reads by matching k-mers against a pre-built taxonomic database.
- Use --db to specify the database path; databases include Standard, PlusPF, nt, and custom builds.
- Kraken2 outputs a report (--report) and a per-read classification file (standard output or --output).
- The report file has 6 columns: percentage, reads covered, reads directly, rank code, NCBI taxon ID, name.
- Use --paired for paired-end reads; pass R1 and R2 as positional arguments after all options.
- Use --threads N for parallel classification; --confidence 0.1 adds a confidence threshold to reduce false positives.
- Bracken uses Kraken2's report to re-estimate species-level abundances more accurately.
- Use --classified-out and --unclassified-out to save classified/unclassified reads to FASTQ files.
- kraken2-build creates databases: --download-taxonomy, --download-library, --build, --standard.
- Minimizer-based classification: Kraken2 uses minimizers of k-mers for faster, memory-efficient classification.
- --memory-mapping avoids loading database into RAM; slower but useful for memory-constrained systems.
- --quick uses first hit only for faster classification; trades accuracy for speed.
- --minimum-hit-groups sets minimum overlapping k-mers needed for classification (default: 2).
- --use-mpa-style outputs report in MetaPhlAn format for compatibility with downstream tools.
- Protein databases: kraken2-build --protein builds translated search databases for metagenomic protein classification.

## Pitfalls
- kraken2 has NO subcommands. ARGS starts directly with flags (e.g., --db, --paired, --threads, --confidence). Do NOT put a subcommand like 'classify' or 'analyze' before flags.
- The database path must contain hash.k2d, opts.k2d, and taxo.k2d files — check database integrity.
- Without --report, Kraken2 only outputs per-read assignments, not the taxonomic summary needed for Bracken.
- Kraken2 databases can be very large (>50 GB for Standard) — ensure sufficient disk space.
- For paired-end data, use --paired AND pass BOTH files — omitting --paired treats R1 and R2 as unrelated reads.
- --confidence threshold (0.0-1.0) is a fraction of k-mers that must match; higher = more specific, lower = more sensitive.
- The Kraken2 database must be kept in RAM (uses mmap) — performance degrades severely without sufficient memory.
- --memory-mapping avoids loading DB into RAM but is significantly slower; use only when RAM is insufficient.
- --quick mode is faster but less accurate; suitable for preliminary analysis, not final results.
- Database building requires significant disk space (~100 GB) and time; consider downloading pre-built databases.
- --minimum-base-quality filters low-quality bases in FASTQ; only effective with FASTQ input.
- --gzip-compressed and --bzip2-compressed auto-detected for regular files; explicit flags needed for pipes.

## Examples

### classify paired-end metagenomic reads against the standard database
**Args:** `--db /path/to/kraken2_db --paired --threads 8 --output kraken_output.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database path; --paired paired-end mode; --threads 8 threads; --output kraken_output.txt per-read classifications; --report kraken_report.txt taxonomic summary; R1.fastq.gz R2.fastq.gz input reads

### classify reads with confidence threshold and save unclassified reads
**Args:** `--db /path/to/kraken2_db --paired --confidence 0.1 --threads 8 --output kraken_out.txt --report kraken_report.txt --unclassified-out unclassified#.fastq R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --paired paired-end mode; --confidence 0.1 confidence threshold; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; --unclassified-out unclassified#.fastq unclassified reads; R1.fastq.gz R2.fastq.gz input reads

### classify single-end reads and generate report
**Args:** `--db /path/to/kraken2_db --threads 8 --output kraken_out.txt --report kraken_report.txt reads.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; reads.fastq.gz single-end input

### classify reads and extract classified reads for downstream analysis
**Args:** `--db /path/to/kraken2_db --paired --threads 8 --output kraken_out.txt --report kraken_report.txt --classified-out classified#.fastq R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --paired paired-end mode; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; --classified-out classified#.fastq classified reads; R1.fastq.gz R2.fastq.gz input reads

### download and build standard Kraken2 database
**Args:** `kraken2-build --standard --db /path/to/kraken2_db --threads 8`
**Explanation:** kraken2-build command; --standard download and build default database; --db /path/to/kraken2_db database path; --threads 8 threads

### build custom database with specific libraries
**Args:** `kraken2-build --download-taxonomy --db custom_db && kraken2-build --download-library bacteria --db custom_db && kraken2-build --download-library viral --db custom_db && kraken2-build --build --db custom_db --threads 8`
**Explanation:** kraken2-build --download-taxonomy --db custom_db download taxonomy; kraken2-build --download-library bacteria --db custom_db add bacteria library; kraken2-build --download-library viral --db custom_db add viral library; kraken2-build --build --db custom_db --threads 8 build database

### classify with memory mapping for low-RAM systems
**Args:** `--db /path/to/kraken2_db --memory-mapping --paired --threads 8 --output kraken_out.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --memory-mapping avoid loading DB into RAM; --paired paired-end mode; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; R1.fastq.gz R2.fastq.gz input reads

### quick classification for preliminary analysis
**Args:** `--db /path/to/kraken2_db --quick --paired --threads 8 --output kraken_out.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --quick quick classification; --paired paired-end mode; --threads 8 threads; --output kraken_out.txt per-read classifications; R1.fastq.gz R2.fastq.gz input reads

### classify with minimum hit groups for stringency
**Args:** `--db /path/to/kraken2_db --paired --minimum-hit-groups 3 --confidence 0.1 --threads 8 --output kraken_out.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --paired paired-end mode; --minimum-hit-groups 3 minimum hit groups; --confidence 0.1 threshold; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; R1.fastq.gz R2.fastq.gz input reads

### output report in MetaPhlAn format
**Args:** `--db /path/to/kraken2_db --paired --threads 8 --report kraken_mpa.txt --use-mpa-style R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --paired paired-end mode; --threads 8 threads; --report kraken_mpa.txt summary; --use-mpa-style MetaPhlAn format; R1.fastq.gz R2.fastq.gz input reads

### classify with minimum base quality filtering
**Args:** `--db /path/to/kraken2_db --paired --minimum-base-quality 20 --threads 8 --output kraken_out.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --paired paired-end mode; --minimum-base-quality 20 quality filter; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; R1.fastq.gz R2.fastq.gz input reads

### build protein database for translated search
**Args:** `kraken2-build --download-taxonomy --db protein_db && kraken2-build --download-library nr --db protein_db && kraken2-build --build --db protein_db --protein --threads 8`
**Explanation:** kraken2-build --download-taxonomy --db protein_db download taxonomy; kraken2-build --download-library nr --db protein_db add nr library; kraken2-build --build --db protein_db --protein --threads 8 build protein database

### classify single-end reads with confidence threshold
**Args:** `--db /path/to/kraken2_db --confidence 0.05 --threads 8 --output kraken_out.txt --report kraken_report.txt reads.fastq.gz`
**Explanation:** kraken2 command; --db /path/to/kraken2_db database; --confidence 0.05 threshold; --threads 8 threads; --output kraken_out.txt per-read classifications; --report kraken_report.txt summary; reads.fastq.gz single-end input

### add custom sequences to existing database
**Args:** `kraken2-build --add-to-library custom_sequences.fa --db custom_db && kraken2-build --build --db custom_db --threads 8`
**Explanation:** kraken2-build --add-to-library custom_sequences.fa --db custom_db add custom sequences; kraken2-build --build --db custom_db --threads 8 rebuild database
