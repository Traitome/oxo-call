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

## Pitfalls

- MultiQC searches recursively — run from the project root or specify the correct parent directory.
- If no tools are detected, check that the log/output files have the expected names (e.g., *_fastqc.zip for FastQC).
- Without -f, MultiQC will not overwrite an existing report — always use -f when re-running.
- MultiQC does not re-run QC tools — it only aggregates existing output files.
- Some tools produce output only when run with specific flags (e.g., samtools flagstat must be redirected to a file).
- Large projects with hundreds of samples may need --export to reduce report size.

## Examples

### aggregate all QC results from the current directory into a single report
**Args:** `. -o multiqc_report/ -f`
**Explanation:** . scans current directory recursively; -o sets output directory; -f overwrites existing report

### aggregate QC results from a specific results directory
**Args:** `/path/to/results/ -o /path/to/qc_summary/ -n project_qc_report -f`
**Explanation:** -n renames the report file; -o specifies output directory; -f forces overwrite

### run multiqc ignoring a specific subdirectory
**Args:** `/results/ --ignore /results/old_run/ -o multiqc_output/ -f`
**Explanation:** --ignore excludes specified paths from the search

### generate a multiqc report with flat (non-interactive) output suitable for PDF
**Args:** `. --flat -o flat_report/ -f`
**Explanation:** --flat generates non-interactive HTML plots, useful for PDF generation or static reports

### run multiqc on only FastQC and Trimmomatic outputs
**Args:** `fastqc_results/ trimmomatic_logs/ -o summary_qc/ -f`
**Explanation:** pass multiple directories as input; MultiQC aggregates from all of them
