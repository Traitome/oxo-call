---
name: "chanjo"
category: "coverage-analysis"
description: "A coverage visualization and analysis tool for targeted sequencing data, providing interactive HTML reports and a JSON API for exploring genomic coverage metrics."
tags: ["coverage", "sequencing", "bedtools", "bam", "visualization", "targeted-sequencing", "ngs"]
author: "AI-generated"
source_url: "https://github.com/lasseignelab/chanjo"
---

## Concepts

- Chanjo operates on aligned reads (BAM/CRAM files) combined with genomic regions defined in BED format — the BED file specifies the target exons, genes, or regions of interest for coverage calculation.
- The tool calculates per-base and region-level coverage metrics, including mean depth, breadth of coverage above threshold, and base-level completeness — these metrics are stored in an SQLite database for fast querying.
- Chanjo provides two primary interfaces: a command-line tool for generating coverage reports (`chanjo report`) and a JSON HTTP API (`chanjo api start`) for programmatic access to coverage data.
- The companion binary `chanjo-build` initializes the coverage database from a BAM file and a corresponding BED file, linking alignment data to target regions.
- Coverage thresholds are configurable — the default minimum coverage depth is set to 1x, but can be adjusted via the `--min-coverage` flag to define what constitutes "covered"bases.

## Pitfalls

- Using a BED file with chromosome names that do not exactly match the chromosome naming in the BAM file header will result in zero coverage reported for all targets, because chanjo performs exact string matching on contig identifiers.
- Running `chanjo-build` multiple times on the same database without the `--overwrite` flag will preserve existing coverage data but may fail to update targets that were removed from the BED file — stale target entries will persist in reports.
- Specifying an incorrect or mismatched reference genome FASTA file results in silent failures where coverage numbers are artificially low because the alignment coordinates in the BAM file do not align to the provided reference.
- Failing to index the BAM file with `samtools index` before running chanjo will cause the tool to skip regions or report incomplete coverage due to inability to efficiently random-access the alignment data.
- Setting the coverage threshold too high (e.g., `--min-coverage 100`) when the sequencing depth is low will artificially report genomic regions as uncovered, making it appear that targets failed when they simply were not sequenced deeply enough.

## Examples

### Initialize a new coverage database from a BAM file and target BED file
**Args:** `build --bam alignments.bam --bed targets.bed --database coverage.db`
**Explanation:** The `build` subcommand creates a new SQLite database linking alignments in the BAM file to the genomic targets defined in the BED file for subsequent coverage queries.

### Generate an HTML coverage report for the current database
**Args:** `report --output coverage_report.html`
**Explanation:** The `report` subcommand reads the coverage data from the SQLite database and produces a standalone HTML file with interactive visualizations showing breadth and depth of coverage per target.

### Start the JSON API server on a specific host and port
**Args:** `api start --host 0.0.0.0 --port 8000`
**Explanation:** The `api start` subcommand launches an HTTP server that exposes coverage endpoints (e.g., `/gene/BRCA1`) in JSON format, enabling integration with other bioinformatics pipelines.

### Calculate coverage with a custom minimum depth threshold
**Args:** `build --bam alignments.bam --bed targets.bed --database coverage.db --min-coverage 10`
**Explanation:** Setting `--min-coverage 10` defines that bases with at least 10x depth are considered "covered" when computing breadth-of-coverage metrics, filtering out shallow regions.

### Filter output to specific genes by providing a comma-separated list
**Args:** `report --output report.html --genenames BRCA1,TP53,EGFR`
**Explanation:** The `--genenames` flag restricts the report to only the specified genes, producing a focused coverage summary for key targets of interest.

### Export coverage summary to JSON instead of HTML
**Args:** `json --output coverage.json --genenames KRAS`
**Explanation:** The `json` subcommand exports raw coverage data for the named gene in JSON format, useful for downstream programmatic processing.

### List all genes (or targets) stored in the database
**Args:** `ls --database coverage.db`
**Explanation:** The `ls` subcommand lists all genes or targets currently stored in the coverage database, useful for verifying which regions were included in the build step.