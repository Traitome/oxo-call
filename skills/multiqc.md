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
**Explanation:** . scans current directory recursively; -o sets output directory; -f overwrites existing report

### aggregate QC results from a specific results directory
**Args:** `/path/to/results/ -o /path/to/qc_summary/ -n project_qc_report -f`
**Explanation:** input path; -n renames the report file; -o specifies output directory; -f forces overwrite

### run multiqc ignoring a specific subdirectory
**Args:** `/results/ --ignore /results/old_run/ -o multiqc_output/ -f`
**Explanation:** /results/ input path; --ignore excludes specified paths; -o output directory; -f overwrites

### generate a multiqc report with flat (non-interactive) output suitable for PDF
**Args:** `. --flat -o flat_report/ -f`
**Explanation:** . current directory input; --flat generates non-interactive HTML; -o output directory; -f overwrites; useful for PDF generation

### run multiqc on only FastQC and Trimmomatic outputs
**Args:** `fastqc_results/ trimmomatic_logs/ -o summary_qc/ -f`
**Explanation:** fastqc_results/ and trimmomatic_logs/ input directories; -o output directory; -f overwrites; MultiQC aggregates from all

### run only specific modules (FastQC and STAR)
**Args:** `results/ -m fastqc -m star -o qc_report/ -f`
**Explanation:** results/ input; -m fastqc -m star runs only these modules; -o output; -f overwrites; useful for focused reports

### exclude specific modules from the report
**Args:** `results/ -e cutadapt -e fastqc -o qc_report/ -f`
**Explanation:** results/ input; -e cutadapt -e fastqc excludes these modules; -o output; -f overwrites; useful when tools produced problematic outputs

### rename samples using a TSV file
**Args:** `results/ --sample-names sample_names.tsv -o renamed_report/ -f`
**Explanation:** results/ input; --sample-names provides alternative display names; -o output; -f overwrites; TSV format: sample_id\tdisplay_name

### replace sample names with new names
**Args:** `results/ --replace-names rename_map.tsv -o renamed_report/ -f`
**Explanation:** results/ input; --replace-names renames samples permanently; -o output; -f overwrites; TSV format: old_name\tnew_name

### export data in JSON format for downstream analysis
**Args:** `results/ --data-format json --no-report -o data_only/ -f`
**Explanation:** results/ input; --data-format json outputs JSON; --no-report skips HTML; -o output; -f overwrites; for programmatic use

### apply inline config to customize thresholds
**Args:** `results/ --cl-config "qualimap_config: { general_stats_coverage: [10, 20, 50] }" -o custom_report/ -f`
**Explanation:** results/ input; --cl-config overrides config values inline; -o output; -f overwrites; YAML syntax for quick customization

### generate PDF report with simple template
**Args:** `results/ --pdf -t simple -o pdf_report/ -f`
**Explanation:** results/ input; --pdf creates PDF; -t simple uses simple template; -o output; -f overwrites; requires Pandoc installed

### ignore specific samples by name pattern
**Args:** `results/ --ignore-samples "*control*" --ignore-samples "*blank*" -o filtered_report/ -f`
**Explanation:** results/ input; --ignore-samples excludes samples matching glob patterns; -o output; -f overwrites; useful for removing controls
