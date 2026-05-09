---
name: bcbio_monitor
category: NGS Pipeline Monitoring
description: Command-line tool for monitoring bcbio distributed sequencing analysis runs. Tracks sample-level progress, quality metrics, resource utilization, and error states in real-time by querying a run's MongoDB or SQLite backend database and shared filesystem.
tags:
  - ngs
  - pipeline
  - monitoring
  - quality-control
  - hts
  - distributed
  - bcbio
author: AI-generated
source_url: https://github.com/chapmanb/bcbio
---

## Concepts

- bcbio_monitor reads run state from a backend database (MongoDB or SQLite) that bcbio creates during initialization. Without the database file present in the run directory, the monitor outputs no useful information and exits silently with status code 0.
- The tool supports multiple output formats: plain text for terminal dashboards, JSON for programmatic parsing, and CSV for spreadsheet-based downstream analysis. Output format is controlled by the `--format` flag and defaults to human-readable text.
- Monitoring intervals are configurable via `--poll INTERVAL` where INTERVAL is specified in seconds. Short intervals (under 5 seconds) generate excessive filesystem I/O on shared storage, while intervals exceeding 300 seconds may miss rapid sample transitions and error spikes.
- Sample-level status tracking includes per-step progression through alignment, variant calling, and annotation phases. Each sample displays a phase indicator (BAM generation, variant detection, annotation, etc.), estimated time remaining, and current quality metrics like coverage and insert size distribution.
- Resource utilization reporting captures CPU, memory, and disk I/O per node when running in distributed mode with bcbio's scheduler integration. This data is stored in the run database and queried on each poll cycle.

## Pitfalls

- Attempting to monitor a run that has not been started yet produces empty output because the database schema has not been initialized. Always verify the run is actively executing with `ps aux | grep bcbio` before relying on monitor output.
- Running bcbio_monitor from a directory different than the run's base directory causes the tool to fail to locate the project configuration file `bcbio_system.yaml`, resulting in a "Project not found" error message.
- Specifying a non-existent output file path with `--outfile` without parent directory creation results in silent failure where no output is written and no error is raised, leading to downstream pipelines expecting data that does not exist.
- Using the JSON output format for large runs (over 500 samples) generates JSON payloads exceeding 10 MB per poll, which can cause memory pressure on systems with limited RAM and slow subsequent parsing.
- Monitoring runs that used bcbio's `--workdir` option to relocate temporary files requires passing the same `--workdir` flag to bcbio_monitor, otherwise the tool cannot resolve relative paths in the database and reports all samples as stuck at their current phase.
- Forgetting to stop monitoring sessions before deleting a run directory leads to zombie monitor processes that continue holding file descriptors to deleted paths, eventually exhausting available file descriptors on the system.

## Examples

### Monitoring a run with human-readable dashboard output
**Args:** `--run /data/run2019/bcbio_project --poll 10`
**Explanation:** Starts a real-time text dashboard that polls the run database every 10 seconds, displaying per-sample phase, progress bar, and elapsed time until the run completes or is interrupted.

### Exporting run status as JSON for programmatic consumption
**Args:** `--run /data/run2019/bcbio_project --format json --outfile /logs/run_status.json --poll 60`
**Explanation:** Outputs machine-readable JSON every 60 seconds to a file suitable for integration with external alerting systems or web dashboards built with tools like Grafana.

### Monitoring with per-node resource utilization reporting
**Args:** `--run /data/run2019/bcbio_project --resources --poll 30`
**Explanation:** Enables detailed per-node CPU and memory usage tracking alongside sample status, useful for identifying compute bottlenecks in distributed bcbio runs on clusters using SGE or Slurm schedulers.

### Checking a run summary without continuous polling
**Args:** `--run /data/run2019/bcbio_project --once`
**Explanation:** Queries the database exactly once and exits, displaying current sample statuses without entering a continuous monitoring loop. Ideal for inclusion in wrapper scripts or cron-based periodic checks.

### Filtering output to show only samples with errors or warnings
**Args:** `--run /data/run2019/bcbio_project --errors --poll 15`
**Explanation:** Displays only samples that have reported non-zero exit codes or quality warnings, suppressing healthy samples from the output and reducing visual noise when debugging problematic runs.

### Monitoring a run relocated to a custom work directory
**Args:** `--run /data/run2019/bcbio_project --workdir /scratch/bcbio_work --poll 10`
**Explanation:** Tells the monitor to resolve temporary file paths using the custom work directory location, matching the `--workdir` setting used during bcbio run initialization to avoid false reports of sample progress.