---
name: bigsdb-downloader
category: Data Retrieval / Database Tools
description: A command-line tool for downloading bacterial genome sequences, assemblies, and associated metadata from BIGSdb (Bacterial Isolate Genome Sequence Database) servers. Supports filtered queries by species, locus, date range, and MLST profiles.
tags: [bigsdb, genome, bacteria, database, sequence-download, fasta, assembly, mlst, antimicrobial-resistance]
author: AI-generated
source_url: https://github.com/ BIGSdb/bigsdb-downloader
---

## Concepts

- The tool operates in client-server mode, requiring a valid BIGSdb server URL (configured via `--server` or environment variable `BIGSDB_SERVER`) and supports both public and authenticated access. Use `--user` and `--password` for credentials or `--token` for OAuth2 tokens when accessing private datasets.

- Output formats are controlled by `--format`, supporting `fasta` (nucleotide sequences), `gbk` (GenBank), `json` (structured metadata), and `csv` (tabular isolates table). For batch downloads, `--output-dir` specifies the target directory, and `--compression` enables gzip/bzip2 compression of output files.

- Query filtering uses `--query` with BIGSdb's isolate database syntax, allowing complex selections via field names, operators, and Boolean logic (e.g., `species_id = 123 AND date >= 2020-01-01`). Additionally, `--locus` extracts specific gene loci (e.g., `--locus rpoB` for housekeeping genes), and `--mlst` filters by MLST sequence types.

- Download resuming is supported via `--resume` or `--checkpoint`, which stores progress in `.bigsdb-downloader.state` files. This is essential for large datasets or unstable connections, preventing re-download of already-retrieved records after interruption.

- Metadata enrichment can be enabled with `--include-metadata` to append isolate provenance, antimicrobial resistance phenotypes, and epidemiological data from linked BIGSdb schemes into the output, enabling downstream integrative analyses.

## Pitfalls

- Omitting `--server` defaults to the public BIGSdb server at https://pubmlst.org, but this may not contain the specific scheme or species database you need. Targeting the wrong server produces empty results or authentication errors. Always verify the server URL matches your target database's scheme name.

- Using overly broad queries without pagination (`--batch-size`) can overwhelm memory or trigger server-side timeouts on large result sets (>10,000 isolates). The server may drop the connection mid-transfer, resulting in partial downloads that appear complete but lack records. Always set `--batch-size` to a reasonable number (e.g., 500–1000) for large queries.

- Failing to specify `--format` when downloading sequence data may default to FASTA, which discards metadata and feature annotations. For downstream annotation or variant calling, GenBank (`gbk`) or JSON formats preserve critical feature locuses and qualifiers that FASTA lacks.

- Authentication credentials passed via `--password` on the command line are visible in process listings (`ps aux`). Use `--password-file` or environment variable `BIGSDB_PASSWORD` instead to avoid credential exposure on shared systems, especially in HPC environments.

- Not configuring `--output-dir` in multi-run workflows causes files to overwrite each other in the current working directory. Combined with `--prefix` omission, downstream analysis scripts may read stale or mismatched data, compromising reproducibility.

## Examples

### Download all sequences for a specific species

**Args:** `--server https://pubmlst.org/bigsdb --species "Salmonella enterica" --format fasta --output-dir ./salmonella_genomes`
**Explanation:** This connects to the public PubMLST server, filters isolates by species name, downloads nucleotide sequences in FASTA format, and saves them to a local directory. The `--species` flag performs a string match on the isolates table's species field.

### Extract specific gene loci for MLST analysis

**Args:** `--server https://pubmlst.org --locus aroD --locus hemD --locus thrA --format csv --output mlst_genes.csv`
**Explanation:** This downloads allele sequences for three MLST housekeeping genes in CSV format for downstream MLST profiling. Multiple `--locus` flags combine into a single multi-locus export, which can be parsed by mlst or srst2 tools.

### Filter isolates by date and export with metadata

**Args:** `--server https://pubmlst.org --query "date_collected >= 2022-01-01" --include-metadata --format json --output-dir ./recent_iso`
**Explanation:** This filters isolates collected after January 2022 using BIGSdb query syntax and includes full provenance metadata (host, location, isolation source) in JSON format, enabling epidemiological trend analysis.

### Batch download with checkpointing for large datasets

**Args:** `--server https://bigsdb.pasteur.fr --species "Listeria monocytogenes" --batch-size 500 --checkpoint --output-dir ./listeria_batch`
**Explanation:** This downloads Listeria genomes in batches of 500 records with automatic checkpoint saving to resume interrupted downloads. The checkpoint file allows resuming without re-fetching already-downloaded batches.

### Download using authenticated access for private schemes

**Args:** `--server https://bigsdb.internal.org --scheme Salmonella --user researcher --password-file ~/.bigsdb_cred --format gbk --output-dir ./private_assemblies`
**Explanation:** This authenticates to a private BIGSdb instance using credentials from a file (avoiding command-line exposure), downloads GenBank-formatted assemblies with full feature annotations required for in-house comparative genomics.