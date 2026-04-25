---
name: multiqc
category: qc
description: Aggregates bioinformatics QC results from many tools and samples into a single interactive HTML report
tags: [qc, report, aggregation, ngs, fastqc, trimming, alignment, pipeline]
author: oxo-call built-in
source_url: "https://multiqc.info/"
---

## Concepts
- MultiQC scans a directory recursively for outputs from FastQC, Trimmomatic, STAR, HISAT2, Salmon, featureCounts, samtools, GATK, and 100+ other tools.
- Run MultiQC in or pointing to the directory containing QC output files: multiqc /path/to/results/
- Use -o to specify the output directory for multiqc_report.html and multiqc_data/; use -n to rename the report.
- Use --ignore to exclude directories/patterns; --ignore-samples to exclude specific samples by name.
- MultiQC auto-detects tools — just point it at the directory. No need to specify tools manually in most cases.
- Use -f to overwrite existing reports; -p to also generate PDF; --flat to create flat (non-interactive) HTML.
- The multiqc_data/ directory contains parsed TSV files with all the data — useful for downstream programmatic analysis.
- Use -m/--module to run only specific modules; -e/--exclude to skip specific modules.
- --cl-config allows inline YAML config overrides for customizing report behavior.
- --sample-names provides a TSV file for sample renaming in the report.
- --replace-names allows renaming samples based on a TSV file with old/new name pairs.
- --data-format specifies output format (tsv, csv, json, yaml) for the data directory.
- --no-report generates only data files without the HTML report for programmatic use.

## Pitfalls
- MultiQC has NO subcommands. ARGS starts directly with input paths or flags (e.g., -o, -n, -f, --ignore). Do NOT put a subcommand like 'report' or 'aggregate' before input paths.
- MultiQC searches recursively — run from the project root or specify the correct parent directory.
- If no tools are detected, check that the log/output files have the expected names (e.g., *_fastqc.zip for FastQC).
- Without -f, MultiQC will not overwrite an existing report — always use -f when re-running.
- MultiQC does not re-run QC tools — it only aggregates existing output files.
- Some tools produce output only when run with specific flags (e.g., samtools flagstat must be redirected to a file).
- Large projects with hundreds of samples may need --export to reduce report size.
- --module and --exclude are mutually exclusive; use one or the other, not both.
- Sample renaming with --replace-names requires exact matches; verify TSV format (old_name\tnew_name).
- --cl-config YAML syntax errors can cause silent failures; validate config before running.
- PDF generation requires Pandoc installation; check with 'pandoc --version' first.
- --ignore patterns use glob syntax; test patterns with 'ls' before running MultiQC.

## Examples

### aggregate all QC results from the current directory into a single report
**Args:** `. -o multiqc_report/ -f`
**Explanation:** multiqc command; . scans current directory recursively; -o multiqc_report/ output directory; -f overwrites existing report

### aggregate QC results from a specific results directory
**Args:** `/path/to/results/ -o /path/to/qc_summary/ -n project_qc_report -f`
**Explanation:** multiqc command; /path/to/results/ input path; -o /path/to/qc_summary/ output directory; -n project_qc_report renames report file; -f forces overwrite

### run multiqc ignoring a specific subdirectory
**Args:** `/results/ --ignore /results/old_run/ -o multiqc_output/ -f`
**Explanation:** multiqc command; /results/ input path; --ignore /results/old_run/ excludes specified paths; -o multiqc_output/ output directory; -f overwrites

### generate a multiqc report with flat (non-interactive) output suitable for PDF
**Args:** `. --flat -o flat_report/ -f`
**Explanation:** multiqc command; . current directory input; --flat generates non-interactive HTML; -o flat_report/ output directory; -f overwrites

### run multiqc on only FastQC and Trimmomatic outputs
**Args:** `fastqc_results/ trimmomatic_logs/ -o summary_qc/ -f`
**Explanation:** multiqc command; fastqc_results/ trimmomatic_logs/ input directories; -o summary_qc/ output directory; -f overwrites

### run only specific modules (FastQC and STAR)
**Args:** `results/ -m fastqc -m star -o qc_report/ -f`
**Explanation:** multiqc command; results/ input directory; -m fastqc -m star runs only these modules; -o qc_report/ output directory; -f overwrites

### exclude specific modules from the report
**Args:** `results/ -e cutadapt -e fastqc -o qc_report/ -f`
**Explanation:** multiqc command; results/ input directory; -e cutadapt -e fastqc excludes these modules; -o qc_report/ output directory; -f overwrites

### rename samples using a TSV file
**Args:** `results/ --sample-names sample_names.tsv -o renamed_report/ -f`
**Explanation:** multiqc command; results/ input directory; --sample-names sample_names.tsv provides alternative display names; -o renamed_report/ output directory; -f overwrites

### replace sample names with new names
**Args:** `results/ --replace-names rename_map.tsv -o renamed_report/ -f`
**Explanation:** multiqc command; results/ input directory; --replace-names rename_map.tsv renames samples; -o renamed_report/ output directory; -f overwrites

### export data in JSON format for downstream analysis
**Args:** `results/ --data-format json --no-report -o data_only/ -f`
**Explanation:** multiqc command; results/ input directory; --data-format json outputs JSON; --no-report skips HTML; -o data_only/ output directory; -f overwrites

### apply inline config to customize thresholds
**Args:** `results/ --cl-config "qualimap_config: { general_stats_coverage: [10, 20, 50] }" -o custom_report/ -f`
**Explanation:** multiqc command; results/ input directory; --cl-config YAML inline config overrides; -o custom_report/ output directory; -f overwrites

### generate PDF report with simple template
**Args:** `results/ --pdf -t simple -o pdf_report/ -f`
**Explanation:** multiqc command; results/ input directory; --pdf creates PDF; -t simple uses simple template; -o pdf_report/ output directory; -f overwrites

### ignore specific samples by name pattern
**Args:** `results/ --ignore-samples "*control*" --ignore-samples "*blank*" -o filtered_report/ -f`
**Explanation:** multiqc command; results/ input directory; --ignore-samples "*control*" --ignore-samples "*blank*" excludes samples matching glob patterns; -o filtered_report/ output directory; -f overwrites
