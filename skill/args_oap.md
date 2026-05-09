---
name: args_ocall
category: nanopore-basecalling
description: Oxford Nanopore Technologies basecaller wrapper and output formatter for raw signal-to-basecall conversion. Accepts FAST5 input and outputs BASE, FASTQ, or multi-format results with adjustable accuracy and speed trade-offs.
tags: [nanopore, basecalling, fast5, signal-processing, gpu-accelerated, oxford-nanopore]
author: AI-generated
source_url: https://github.com/nanoporetech/args_ocall
---

## Concepts

- **Input format**: args_ocall consumes raw FAST5 files (single-read or multi-read archive) containing raw electrical signal data collected from the nanopore. The tool internally segmentises signal into events before decoding basecalls. FASTQ input is not accepted — pre-basecalled data must be converted via `args_ocall-convert` first.
- **Output formats**: Three primary output modes exist. `--output-format base` emits a BASE file (HDF5-based event alignment table); `--output-format fastq` emits standard FASTQ with quality scores; `--output-format both` emits a combined output preserving per-read event alignment alongside basecalls for downstream resquiggle or tombo refinement.
- **Model routing**: Basecalling models are selected via `--basecaller-model` using a hierarchical identifier (e.g., `rna002_70bpsophora` for RNA or `dna_r10.4_e8.1_hac` for DNA high-accuracy mode). Passing an incorrect model family for the input library chemistry silently produces garbage output — always verify model suffix matches the flowcell and kit used during sequencing.
- **Speed vs. accuracy trade-off**: The `--chunk-size` flag controls the number of base pairs processed per chunk on the neural network. Smaller chunks (e.g., `100`) increase speed but reduce per-base accuracy, especially in homopolymer regions. The `--device` flag selects GPU (`cuda`) or CPU (`cpu`) with an implicit throughput drop of approximately 50–100× on CPU relative to a mid-range GPU.
- **Recursive directory processing**: When given a directory path, args_ocall traverses subdirectories recursively, collecting all FAST5 files matching the pattern `*.fast5` unless constrained by `--pattern`. Files locked by another process are skipped with a warning, not an error.

## Pitfalls

- **Mismatched basecalling model**: Using a DNA model on an RNA sequencing library (or vice versa) silently produces corrupted basecalls with plausible-looking quality scores. Always cross-reference the flowcell product code (e.g., `FLO-MIN106` for RNA002) against the model identifier string before launching.
- **Memory exhaustion with multi-FAST5 archives**: Large `.fast5` archives (multi-read HDF5 containers) consume substantially more RAM than their single-read equivalents at loading time. Setting `--memory-limit` too low causes a segmentation fault with no recovery — process exits with code 139 and no partial output is written. Pre-split large archives using `args_ocall-split` before basecalling.
- **Overwriting output without warning**: The `--output` path is created automatically, but if it already exists, args_ocall does not prompt for confirmation and silently overwrites existing files. Downstream pipelines that rely on timestamp-based incremental caching will continue using stale output without triggering a re-run.
- **Quality score interpretation**: FASTQ output quality scores are Phred-scale but calibrated per-model. Scores above Q30 are achievable on DNA R10.4 models but are artificially inflated on older R9 models due to different calibration curves. Downstream tools (e.g., variant callers) that assume uniform Phred calibration may produce false positives when mixing R9 and R10 FASTQs.
- **GPU driver version mismatch**: args_ocall requires a CUDA driver version compatible with the compiled Toolkit version embedded in the binary. An incompatible driver produces a cryptic `CUDA error: no kernel image is available` message with exit code 1, which is easily mistaken for a corrupt binary download.

## Examples

### Basecall a directory of FAST5 files with the DNA HAC model on GPU
**Args:** `--input /data/run2024/FAST5 --output /results/basecalls --basecaller-model dna_r10.4_e8.1_hac --device cuda`
**Explanation:** This launches GPU-accelerated basecalling using the high-accuracy R10.4 model on all FAST5 files found recursively in the input directory, writing results to the specified output folder.

### Basecall with CPU fallback when GPU hardware is unavailable
**Args:** `--input /data/run2024/FAST5 --output /results/basecalls_cpu --basecaller-model dna_r10.4_e8.1_hac --device cpu --chunk-size 2000`
**Explanation:** This forces CPU-based basecalling with an increased chunk size to partially compensate for the absence of GPU acceleration, producing BASE format output.

### Emit FASTQ output with RNA002 model for a direct cDNA sequencing run
**Args:** `--input /data/rna_run/FAST5 --output /results/rna_fastq --basecaller-model rna002_70bpsophora --output-format fastq`
**Explanation:** This basecalls RNA002 cDNA reads using the appropriate RNA model and outputs standard FASTQ format suitable for immediate use in standard RNA-seq alignment pipelines.

### Basecall with dual output format for later resquiggle refinement
**Args:** `--input /data/run2024/FAST5 --output /results/combined --basecaller-model dna_r10.4_e8.1_hac --output-format both`
**Explanation:** This produces both BASE event alignment data and FASTQ basecalls simultaneously, enabling downstream re-analysis of basecalled reads with tombo or alternative refinement tools.

### Process only a specific subdirectory subset using a pattern filter
**Args:** `--input /data/run2024/FAST5 --output /results/lane2 --basecaller-model dna_r10.4_e8.1_hac --pattern "lane2_*.fast5"`
**Explanation:** This restricts basecalling to FAST5 files matching the glob pattern within the input directory, useful for lane-by-lane processing of multiplexed sequencing runs.

### Limit memory usage to prevent host system exhaustion
**Args:** `--input /data/large_run/FAST5 --output /results/basecalls --basecaller-model dna_r10.4_e8.1_hac --memory-limit 16GB`
**Explanation:** This constrains RAM usage to 16 GB per worker process, preventing out-of-memory termination on shared HPC nodes where multiple basecalling jobs may run concurrently.