---
name: checkm2
category: metagenomics
description: Assessment of metagenome-assembled genome (MAG) and isolate genome quality using machine learning
tags: [metagenomics, mag, quality, completeness, contamination, genome, binning, MIMAG, diamond, prodigal]
author: oxo-call built-in
source_url: "https://github.com/chklovski/CheckM2"
---

## Concepts

- CheckM2 assesses genome bin quality (completeness and contamination) using machine learning models trained on protein signatures.
- CheckM2 is the successor to CheckM1 — it's faster and doesn't require lineage-specific marker genes.
- Use 'checkm2 predict' to run quality assessment on a directory of genome FASTA files.
- Use --input to specify the directory of bins; --output-directory for results; --threads for parallelism.
- Output includes quality_report.tsv with Completeness (%), Contamination (%), and Genome_Size columns.
- High-quality MAG: ≥90% completeness, ≤5% contamination (MIMAG standards).
- Medium-quality MAG: ≥50% completeness, ≤10% contamination.
- Use --database_path to specify the CheckM2 database directory (download with 'checkm2 database --download').
- Two prediction models: general (gradient boost) and specific (neural network); --allmodels runs both.
- Uses Prodigal for gene prediction and DIAMOND for protein annotation internally.
- CHECKM2DB environment variable can specify database location instead of --database_path.
- --lowmem mode reduces RAM usage for large datasets at the cost of slower runtime.

## Pitfalls

- CheckM2 requires the database to be downloaded separately — run 'checkm2 database --download' first.
- Input bins must be in a single directory — CheckM2 processes all FASTA files in --input directory.
- File extensions must be .fasta, .fa, or .fna — other extensions are not recognized unless --extension is set.
- CheckM2 uses protein coding predictions — very fragmented assemblies with few ORFs give inaccurate results.
- CheckM2 output directory must not already exist — use a fresh output directory.
- For large datasets, use --threads to speed up the protein prediction step.
- CheckM2 has subcommands (predict, database, testrun); 'predict' is the main analysis command.
- Default file extension is .fna, not .fasta or .fa — explicitly set -x if using different extensions.
- --resume reuses existing Prodigal and DIAMOND results; useful for interrupted runs but may use stale data.
- CHECKM2DB environment variable overrides --database_path; check both if database issues occur.

## Examples

### assess quality of all MAG bins in a directory
**Args:** `predict --input bins_directory/ --output-directory checkm2_results/ --threads 16`
**Explanation:** checkm2 predict subcommand; --input bins_directory/ directory with FASTA bins; --output-directory checkm2_results/ output directory; --threads 16 parallel processing; outputs quality_report.tsv

### assess genome quality with custom database path
**Args:** `predict --input bins_directory/ --output-directory checkm2_output/ --threads 16 --database_path /path/to/checkm2_database/`
**Explanation:** checkm2 predict subcommand; --input bins_directory/ directory with FASTA bins; --output-directory checkm2_output/ output directory; --threads 16 parallel processing; --database_path /path/to/checkm2_database/ specifies downloaded CheckM2 database; required if not in default location

### assess quality and produce detailed outputs including protein predictions
**Args:** `predict --input bins_directory/ --output-directory checkm2_results/ --threads 16 --allmodels`
**Explanation:** checkm2 predict subcommand; --input bins_directory/ directory with FASTA bins; --output-directory checkm2_results/ output directory; --threads 16 parallel processing; --allmodels runs all CheckM2 quality models; provides more comprehensive quality estimates

### download the CheckM2 database
**Args:** `database --download --path /path/to/databases/`
**Explanation:** checkm2 database subcommand; --download flag; --path /path/to/databases/ output location; downloads the CheckM2 DIAMOND database; must be run before first use

### run checkm2 in low memory mode
**Args:** `predict --input bins_directory/ --output-directory checkm2_results/ --threads 16 --lowmem`
**Explanation:** checkm2 predict subcommand; --input bins_directory/ directory with FASTA bins; --output-directory checkm2_results/ output directory; --threads 16 parallel processing; --lowmem reduces DIAMOND blocksize to decrease RAM usage; useful for large datasets or memory-constrained systems

### use specific prediction model only
**Args:** `predict --input bins_directory/ --output-directory checkm2_results/ --threads 16 --specific`
**Explanation:** checkm2 predict subcommand; --input bins_directory/ directory with FASTA bins; --output-directory checkm2_results/ output directory; --threads 16 parallel processing; --specific forces neural network model; use when bins are from known lineages for potentially better accuracy

### check current database location
**Args:** `database --current`
**Explanation:** checkm2 database subcommand; --current flag; prints the currently configured database path; useful for troubleshooting database issues

### set database location without downloading
**Args:** `database --setdblocation /path/to/checkm2_database.dmnd`
**Explanation:** checkm2 database subcommand; --setdblocation flag; /path/to/checkm2_database.dmnd existing database file; points CheckM2 to an existing database file; alternative to --download for shared installations

### run test to verify installation
**Args:** `testrun --threads 8`
**Explanation:** checkm2 testrun subcommand; --threads 8 parallel processing; runs CheckM2 on internal test genomes to verify installation works correctly; recommended after first install

### process protein files instead of nucleotide
**Args:** `predict --input protein_directory/ --output-directory checkm2_protein_results/ --threads 16 --genes`
**Explanation:** checkm2 predict subcommand; --input protein_directory/ directory with protein files; --output-directory checkm2_protein_results/ output directory; --threads 16 parallel processing; --genes treats input as protein files instead of nucleotide; skips Prodigal gene prediction step
