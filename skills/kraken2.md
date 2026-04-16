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
- CRITICAL: kraken2 has NO subcommands. ARGS starts directly with flags (e.g., --db, --paired, --threads, --confidence). Do NOT put a subcommand like 'classify' or 'analyze' before flags.
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
**Explanation:** --paired for PE; --output per-read classifications; --report summary for Bracken

### classify reads with confidence threshold and save unclassified reads
**Args:** `--db /path/to/kraken2_db --paired --confidence 0.1 --threads 8 --output kraken_out.txt --report kraken_report.txt --unclassified-out unclassified#.fastq R1.fastq.gz R2.fastq.gz`
**Explanation:** --confidence 0.1 reduces false positives; --unclassified-out saves unclassified reads (# → _1/_2 for PE)

### classify single-end reads and generate report
**Args:** `--db /path/to/kraken2_db --threads 8 --output kraken_out.txt --report kraken_report.txt reads.fastq.gz`
**Explanation:** single-end classification; omit --paired for single-end reads

### classify reads and extract classified reads for downstream analysis
**Args:** `--db /path/to/kraken2_db --paired --threads 8 --output kraken_out.txt --report kraken_report.txt --classified-out classified#.fastq R1.fastq.gz R2.fastq.gz`
**Explanation:** --classified-out saves reads that were classified; useful for host decontamination (keep unclassified)

### download and build standard Kraken2 database
**Args:** `kraken2-build --standard --db /path/to/kraken2_db --threads 8`
**Explanation:** --standard downloads and builds the default database; requires ~100 GB disk space and significant time

### build custom database with specific libraries
**Args:** `kraken2-build --download-taxonomy --db custom_db && kraken2-build --download-library bacteria --db custom_db && kraken2-build --download-library viral --db custom_db && kraken2-build --build --db custom_db --threads 8`
**Explanation:** multi-step process: download taxonomy, add libraries (bacteria, viral), then build; customize for specific needs

### classify with memory mapping for low-RAM systems
**Args:** `--db /path/to/kraken2_db --memory-mapping --paired --threads 8 --output kraken_out.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** --memory-mapping avoids loading DB into RAM; slower but works on memory-constrained systems

### quick classification for preliminary analysis
**Args:** `--db /path/to/kraken2_db --quick --paired --threads 8 --output kraken_out.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** --quick uses first hit only; much faster but less accurate; suitable for quick checks, not final analysis

### classify with minimum hit groups for stringency
**Args:** `--db /path/to/kraken2_db --paired --minimum-hit-groups 3 --confidence 0.1 --threads 8 --output kraken_out.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** --minimum-hit-groups 3 requires 3+ hit groups; increases specificity; useful for reducing false positives

### output report in MetaPhlAn format
**Args:** `--db /path/to/kraken2_db --paired --threads 8 --report kraken_mpa.txt --use-mpa-style R1.fastq.gz R2.fastq.gz`
**Explanation:** --use-mpa-style outputs MetaPhlAn-compatible format; useful for pipelines expecting mpa-style reports

### classify with minimum base quality filtering
**Args:** `--db /path/to/kraken2_db --paired --minimum-base-quality 20 --threads 8 --output kraken_out.txt --report kraken_report.txt R1.fastq.gz R2.fastq.gz`
**Explanation:** --minimum-base-quality 20 filters low-quality bases; only effective with FASTQ input; improves accuracy

### build protein database for translated search
**Args:** `kraken2-build --download-taxonomy --db protein_db && kraken2-build --download-library nr --db protein_db && kraken2-build --build --db protein_db --protein --threads 8`
**Explanation:** --protein builds amino acid database; enables translated search for metagenomic protein classification

### classify single-end reads with confidence threshold
**Args:** `--db /path/to/kraken2_db --confidence 0.05 --threads 8 --output kraken_out.txt --report kraken_report.txt reads.fastq.gz`
**Explanation:** --confidence 0.05 for single-end data; lower threshold compensates for reduced information content

### add custom sequences to existing database
**Args:** `kraken2-build --add-to-library custom_sequences.fa --db custom_db && kraken2-build --build --db custom_db --threads 8`
**Explanation:** --add-to-library adds custom FASTA; rebuild required; useful for adding novel genomes or MAGs
