---
name: bracken
category: metagenomics
description: Bayesian re-estimation of abundance from Kraken2 reports for accurate species-level quantification
tags: [metagenomics, taxonomy, abundance, bracken, microbiome, species, kraken2, krakenuniq]
author: oxo-call built-in
source_url: "https://github.com/jenniferlu717/Bracken"
---

## Concepts

- Bracken re-estimates species-level abundances from Kraken2 output using a Bayesian framework for better accuracy.
- Bracken requires the same database used for Kraken2 AND a Bracken distribution file (database{READ_LEN}mers.kmer_distrib).
- bracken-build generates the kmer distribution file from a Kraken database; supports Kraken1, Kraken2, and KrakenUniq databases.
- Key parameters: -l SPECIES (taxonomic level: K=kingdom, P=phylum, C=class, O=order, F=family, G=genus, S=species, S1=subspecies).
- Use -r to specify the read length used for the Bracken database (must match kmer_distrib file: database150mers.kmer_distr for -r 150).
- Use -t to set minimum reads threshold — taxa with fewer reads are excluded from re-estimation (default: 0, recommend: 10).
- Output: Bracken file (.bracken) with adjusted read counts AND an updated Kraken report file (.kreport_bracken).
- combine_bracken_outputs merges Bracken results from multiple samples into one abundance table.
- bracken-build requires KMER_LEN (-k) matching the Kraken database (default 35 for Kraken2, 31 for Kraken1/Uniq).
- Supported Kraken types: kraken2 (default), kraken, krakenuniq — set with -y in bracken-build.

## Pitfalls

- CRITICAL: bracken has NO subcommands. ARGS starts directly with flags (e.g., -d, -i, -o, -l, -r, -t). Do NOT put a subcommand like 'estimate' or 'run' before flags.
- The Bracken distribution file read length must match the -r parameter — using database100mers.kmer_distr with -r 150 will fail.
- Bracken requires the Kraken report file (--report from Kraken2), NOT the per-read classification output.
- Kraken report must be in DEFAULT format, NOT MPA format — MPA reports are not supported by Bracken.
- Using wrong -l (taxonomic level) collapses counts at wrong level — use S for species-level microbiome analysis.
- bracken-build must be run separately for each read length you intend to use — 75bp, 100bp, 150bp each need their own kmer_distrib file.
- Multiple Bracken databases for different read lengths CANNOT be stored in the same folder — files will overwrite each other.
- Low -t (minimum threshold) includes many low-confidence taxa; too high -t excludes rare species.
- Bracken is a post-processing step — it requires Kraken2/KrakenUniq classification first.

## Examples

### build Bracken database from Kraken2 database for 150bp reads
**Args:** `bracken-build -d /path/to/kraken2_db -k 35 -l 150 -t 8 -y kraken2`
**Explanation:** bracken-build companion binary; -k 35 kmer length (Kraken2 default); -l 150 read length; -t 8 threads; -y kraken2 database type

### build Bracken database for KrakenUniq database
**Args:** `bracken-build -d /path/to/krakenuniq_db -k 31 -l 100 -t 8 -y krakenuniq`
**Explanation:** -k 31 for KrakenUniq default; -y krakenuniq specifies KrakenUniq database type; produces database100mers.kmer_distr

### run Bracken on a Kraken2 report for species-level abundance estimation
**Args:** `-d /path/to/kraken2_db -i kraken_report.txt -o bracken_output.bracken -w kraken_report_bracken.txt -l S -r 150 -t 10`
**Explanation:** -d database path; -i Kraken2 report; -o Bracken output; -w updated Kraken report; -l S species level; -r 150 bp read length; -t 10 minimum reads

### run Bracken for genus-level abundance estimation
**Args:** `-d /path/to/kraken2_db -i kraken_report.txt -o bracken_genus.bracken -l G -r 150 -t 5`
**Explanation:** -l G genus level; lower -t 5 to include more genera; produces genus-level abundance estimates

### combine Bracken results from multiple samples into one table
**Args:** `combine_bracken_outputs --files sample1.bracken sample2.bracken sample3.bracken --names s1,s2,s3 --output combined_abundance.txt`
**Explanation:** combine_bracken_outputs companion binary; --files Bracken outputs; --names column headers (comma-separated); --output combined TSV

### run Bracken on short reads (75 bp)
**Args:** `-d /path/to/kraken2_db -i kraken_report.txt -o bracken_75bp.bracken -l S -r 75 -t 10`
**Explanation:** -r 75 specifies 75 bp read length; database75mers.kmer_distrib must exist (created by bracken-build -l 75)

### run Bracken for family-level analysis with higher threshold
**Args:** `-d /path/to/kraken2_db -i kraken_report.txt -o bracken_family.bracken -l F -r 150 -t 50`
**Explanation:** -l F family level; -t 50 higher threshold to include only well-represented families
