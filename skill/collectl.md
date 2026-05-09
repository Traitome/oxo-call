---
name: collectl
category: System Monitoring
description: A lightweight systems performance monitoring tool that captures and records detailed data about CPU, memory, disk, network, and processes in real-time. Useful for profiling computational workflows, debugging performance bottlenecks, and analyzing resource utilization during bioinformatics analyses.
tags:
  - system monitoring
  - performance profiling
  - resource tracking
  - linux metrics
  - bioinformatics infrastructure
author: AI-generated
source_url: https://collectl.sourceforge.net/
---

## Concepts

- **Data Collection Modes**: collectl operates in two primary modes—*record* mode (capturing system data to files for later analysis) and *playback* mode (reading previously recorded files and outputting formatted reports). Record mode uses the `-s` flag to specify which subsystems to monitor (e.g., `-s cmnd` for CPU, memory, network, disk).
- **Subsystem Flags**: System resources are represented by single-character flags: `c` (CPU), `m` (memory), `d` (disk), `n` (network), `j` (interrupts), `s` (socket statistics), `t` (TCP statistics), `f` (filesystem), `y` (VM statistics). Combining flags like `-s cmd` captures CPU, memory, and disk I/O simultaneously.
- **Output File Formats**: When recording, collectl creates raw binary files (default) or can output in plain text format using the `-f` flag with a directory path. During playback, the `-P` flag generates formatted reports (summary, detail, or verbose), while `-csv` or `-json` exports data in machine-readable formats for downstream analysis.
- **Interval and Duration**: The `-i` flag sets the sampling interval in seconds (default is 60 for record mode, 1 for interactive display). The `-m` flag in record mode switches to daemon-like operation with specified duration or indefinite collection until explicitly stopped.

## Pitfalls

- **Default Sampling Interval Too Coarse**: The default 60-second interval in record mode may miss short-lived performance spikes. Using `-i 1` or `-i 5` provides finer resolution but drastically increases storage requirements—collectl can generate hundreds of MBs per hour when monitoring all subsystems at 1-second intervals.
- **Playacting With Large Data Files Without Filtering**: Passing large recorded files to playback without subsystem filters (`-s c`) forces collectl to parse the entire dataset, consuming excessive memory and time. Always specify the subsystem of interest during playback to reduce processing overhead.
- **Running as Root Without Permission Issues**: Executing collectl with root privileges can create files owned by root in shared directories, making them inaccessible to regular users for playback. Use appropriate file permissions (`chmod`) or run collectl under a dedicated service account to avoid permission denied errors during analysis.
- **Confusing Interactive Display with Record Mode**: In interactive mode, collectl continuously prints formatted output to the terminal—this does NOT save data for later use. Users expecting to review data later must explicitly use record mode flags (`-o`, `-f`) to write output files.

## Examples

### Real-time CPU and memory monitoring
**Args:** `-s cm -i 2`
**Explanation:** Monitors CPU (`c`) and memory (`m`) subsystems every 2 seconds, displaying output in real-time to stdout—useful for observing resource trends during active processes without persisting data.

### Recording disk and network I/O to a file
**Args:** `-s dn -i 10 -o /data/performance -f run1`
**Explanation:** Records disk (`d`) and network (`n`) metrics to files in `/data/performance/` every 10 seconds, creating files with the prefix `run1` for later playback analysis.

### Playback and export CPU statistics as CSV
**Args:** `-P -s c -f /data/performance/run1 -csv`
**Explanation:** Plays back a previously recorded dataset from `/data/performance/run1`, filters to only CPU (`c`) statistics, and outputs results in CSV format for import into spreadsheet tools or R/Python scripts.

### Generate a summary report of all subsystems
**Args:** `-P -s cmnd -f /data/performance/run1`
**Explanation:** Generates a formatted summary report from recorded data containing CPU, memory, network, and disk statistics—useful for quick performance overviews after a bioinformatics pipeline run.

### Monitor a specific process by PID
**Args:** `-s p -i 5 -f /data/performance/proc_monitor`
**Explanation:** Monitors process-specific statistics (`p`) including CPU, memory, and I/O for all processes every 5 seconds, writing output files to the specified directory for targeted process profiling.

### Collect in daemon mode for continuous background monitoring
**Args:** `-s cmnd -i 30 -m -o /data/performance`
**Explanation:** Runs collectl as a background daemon recording CPU, memory, network, and disk data every 30 seconds to `/data/performance/`, continuing until the process is explicitly terminated—ideal for long-running workflows.