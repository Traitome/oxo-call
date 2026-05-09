---
name: atol-qc-raw-ont
category: sequencing-qc
description: Quality control report generator for raw Oxford Nanopore Technologies (ONT) sequencing runs. Reads POD5 or FAST5 signal files and produces per-channel and per-read statistics, coverage heatmaps, and a summary HTML report with quality alerts.
tags:
  - nanopore
  - ont
  - quality-control
  - pod5
  - fast5
  - signal-level-qc
  - sequencing-metrics
author: AI-generated
source_url: https://github.com/nanoporetech/atol
---

## Concepts

- **POD5 and FAST5 signal formats** are the primary input formats. The tool reads raw electrical signal data (A/D units per channel over time) directly from POD5 files (preferred, current ONT format) or legacy FAST5 files. It does NOT require basecalled FASTQ as input, making it suitable for pre-basecalling QC.
- **Per-channel statistics** are aggregated across all active channels in the flow cell. Each channel has an individual sampling quality profile, and the report flags channels with abnormal signal drift, low read counts, or dead-channel status. This informs whether a flow cell was loaded evenly.
- **Read-level quality thresholds** are configurable independently of basecall quality. The tool evaluates raw signal quality signals (e.g., sampling rate stability, ADC clipping events, mux phase) which are independent of basecall Q-scores. These raw metrics are the earliest indicator of run health before demultiplexing or alignment.
- **Report output is multi-format**: The tool generates a machine-readable `qc_summary.json`, a tab-delimited `channel_stats.tsv`, and an `index.html` dashboard. Downstream pipelines can consume the JSON for pass/fail gates in automated workflows.
- **MUX scan awareness**: ONT runs perform periodic MUX (multiplexing) scans to re-balance channel signal. The tool auto-detects MUX scan boundaries and excludes those regions from quality metrics, preventing artificial dilution of per-read statistics.

## Pitfalls

- **Passing a directory instead of the correct file format glob** causes the tool to fail silently or produce empty reports. You must specify the exact file format with `--input-format pod5` or `--input-format fast5`. A bare directory path is interpreted as a glob pattern, not recursively scanned.
- **Ignoring the `--min-channel-occupancy` flag when the flow cell is sparsely loaded** results in misleading per-channel histograms. Channels with zero or very few reads skew average quality metrics, but the HTML report still renders them as active, leading to incorrect conclusions about run uniformity.
- **Using a pre-existing output directory without `--force`** causes the tool to refuse overwriting existing report files, silently exiting with code 1. Any automated pipeline that re-runs QC on the same run directory will produce a stale report unless `--force` is specified.
- **Misinterpreting raw signal Q-score as basecall accuracy** is a conceptual error the report explicitly warns against, but users frequently do this. The tool reports raw sampling quality (sampling rate variance, event clipping rate), NOT basecall Q-score. These metrics are uncorrelated with FASTQ read quality and should only drive flow cell loading decisions.
- **Running without specifying `--kit`** discards barcode-aware read filtering. In multiplexed runs, barcoded reads are treated identically to non-barcoded reads, inflating per-channel read counts and skewing barcode balance metrics in the report.

## Examples

### Generate a basic QC report from a POD5 run directory
**Args:** `--input /data/run20240101/pod5 --output /data/run20240101/qc_report --input-format pod5`
**Explanation:** This points the tool at the POD5 run directory, sets the output location, and declares the input format so the tool skips format auto-detection and begins processing immediately.

### Generate a QC report and overwrite an existing report directory
**Args:** `--input /data/run20240101/pod5 --output /data/run20240101/qc_report --input-format pod5 --force`
**Explanation:** Using `--force` allows the tool to overwrite previously generated report files in the output directory without prompting or failing, enabling reproducible pipeline re-runs.

### Limit the report to only channels with at least 50 reads
**Args:** `--input /data/run20240101/pod5 --output /data/run20240101/qc_report --input-format pod5 --min-channel-occupancy 50`
**Explanation:** Setting `--min-channel-occupancy 50` excludes sparsely occupied channels from summary statistics, preventing them from diluting average metrics and producing a more accurate flow cell health assessment.

### Include barcode-aware metrics by specifying the sequencing kit
**Args:** `--input /data/run20240101/pod5 --output /data/run20240101/qc_report --input-format pod5 --kit SQK-NBD114-24"`
**Explanation:** Specifying `--kit` enables barcode-aware read grouping in the report, so barcode balance charts and per-sample read counts are populated rather than lumping all reads into a single pool.

### Run in verbose mode to see per-file progress during a large run
**Args:** `--input /data/run20240101/pod5 --output /data/run20240101/qc_report --input-format pod5 --verbose`
**Explanation:** Verbose mode streams a line-by-line progress log to stderr for each POD5 file processed, which is useful for monitoring long runs interactively via `tail -f` or in a job scheduler log file.

### Skip MUX scan regions and produce only the JSON summary
**Args:** `--input /data/run20240101/pod5 --output /data/run20240101/qc_report --input-format pod5 --skip-mux-scan --output-format json-only`
**Explanation:** This combination skips MUX scan exclusion logic and restricts output to the machine-readable `qc_summary.json` file, suitable for programmatic pass/fail gating in automated pipeline workflows.