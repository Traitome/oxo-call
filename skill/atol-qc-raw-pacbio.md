---
name: atol-qc-raw-pacbio
category: Bioinformatics - QC/Quality Control
description: Quality control tool for PacBio raw sequencing data (HDF5/SUBREADS format). Analyzes raw Pacific Biosciences sequencing metrics including read quality scores, length distributions, productivity metrics, and generates QC reports for raw movie data from Sequel, Sequel II, and related platforms.
tags: [pacbio, raw-data, quality-control, hdf5, sequencing, genomics, bioinformatics]
author: AI-generated
source_url: https://github.com/pacbio/atol-qc-raw-pacbio
---

## Concepts

- **Input format**: Accepts PacBio raw HDF5 movie files (.h5) and BAM formats containing unaligned subreads. The tool processes raw pulse data (AOL/XAN/SAM) and basecall information from PacBio Sequel/Sequel II instruments.
- **Quality metrics**: Computes per-read and aggregate quality metrics including average QV (quality value), read length distributions (L50/N50), polymerasome ratios, and signal-to-noise ratios across ZMW (Zero-Mode Waveguide) sites.
- **Output reports**: Generates JSON-formatted QC summaries, CSV tables of yield statistics, and optional HTML visualizations showing throughput trends, quality score histograms, and read length boxplots.
- **ZMW filtering**: Supports filtering based on ZMW loading efficiency, read quality thresholds, and single-molecule versus multi-molecule classification to exclude low-quality reads from final QC summaries.
- **Batch processing**: Can process multiple input movie files in a single run, producing aggregated statistics and comparative QC tables across runs.

## Pitfalls

- **Using old movie format versions**: Attempting to process HDF5 files from older PacBio instrument chemistries (e.g., pre-Sequel) may cause parsing errors because the data structure fields differ. Always specify the correct `--chemistry` flag or use the `--auto-detect` option.
- **Ignoring empty ZMW filtering**: Running QC without filtering empty or failed ZMWs (`--min-read-length 0`) inflates total read counts but reduces meaningful yield metrics, leading to overly optimistic QC reports.
- **Out-of-memory on large datasets**: Processing high-coverage (>100x) raw movies without specifying chunk size (`--chunk-size`) can exhaust memory. Split large files or reduce `--max-load` to prevent crashes.
- **Misinterpreting QV scores**: Confusing Phred-scaled QV values with per-base accuracy percentages. A QV of 30 means 99.9% base accuracy, not 30% accuracy.
- **Mixed instrument data**: Combining raw movies from different PacBio instrument types (e.g., Sequel and Sequel IIe) without specifying individual `--instrument-type` flags produces unreliable aggregated metrics due to differing signal processing.

## Examples

### Generate a basic QC report for a single PacBio raw movie
**Args:**