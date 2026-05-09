---
name: bioepic
category: epigenetics/epitranscriptomics
description: A tool for detecting and quantifying epitranscriptomic modifications (m6A, m1A, ψ, etc.) from nanopore sequencing direct RNA data. Uses signal-level features and machine learning models to predict modification sites and stoichiometry across transcriptomes.
tags:
  - nanopore
  - epitranscriptomics
  - RNA modifications
  - direct RNA sequencing
  - m6A detection
  - m1A detection
  - pseudouridine
  - basecalling
  - signal analysis
  - TOML configuration
author: AI-generated
source_url: https://github.com/bioepic/bioepic
---

## Concepts

- **Direct RNA Sequencing Data Model**: Bioepic operates on FAST5 files (multi-read) or pod5 files containing raw electrical signal traces from nanopore runs. Each read has a sampled signal trace indexed by channel and time, which bioepic resquiggles against a reference or consensus transcript to obtain alignment positions before modification calling.

- **TOML Configuration and Caller Modes**: The main configuration file (`config.toml`) specifies caller mode (`standard`, `adaptive`, `denovo`), reference genome/transcriptome path, basecall model tag, k-mer model path, and modification priors. Command variants like `bioepic detect`, `bioepic quantify`, and `bioepic call` consume this file; overriding individual flags is possible but the TOML takes precedence if not explicitly overwritten.

- **Output Formats and Aggregation**: Bioepic produces per-site JSON/TSV files with log-odds scores, per-read BED files with modification probabilities, and a summary CSV aggregating coverage and stoichiometry per gene/transcript. The `bioepic aggregate` companion subcommand merges multiple samples when provided with a manifest file listing input paths, enabling cross-sample comparison and differential modification analysis.

- **Machine Learning Models and Training**: Bioepic includes pre-trained models for human, mouse, and yeast epitranscriptomes but also supports custom model training via `bioepic train` using a TOML manifest describing labeled FAST5 reads, feature set, and hyperparameter search space. Models are stored as `.bioepic` checkpoint files and referenced by name in the detection config.

## Pitfalls

- **Mismatched Basecall and K-mer Model Versions**: Using a Guppy or Dorado basecall model version that does not match the k-mer model bundled with bioepic causes systematic errors in signal-to-base alignment, producing inflated false positive rates for modification calls. Always verify that `basecall_model` and `kmer_model` in `config.toml` originate from the same toolkit release.

- **Insufficient Read Coverage Per Transcript**: Bioepic requires a minimum per-transcript read depth (default 20 reads) to call modifications reliably; below this threshold, log-odds scores become noisy and sites may be spuriously filtered or called with high uncertainty. Users working with low-input samples must explicitly lower `min_reads` in the TOML and interpret results cautiously, preferably filtering by `min_log_odds` > 3.0.

- **Reference Transcriptome Annotation Errors**: If the reference genome or transcriptome FASTA contains unmapped contigs, duplicate IDs, or lacks critical isoforms, bioepic will silently map reads to the wrong transcript, leading to misassigned modification sites in downstream BED output. Always validate the input reference with `bioepic validate-ref` before running detection.

- **Memory Configuration on High-Channel Runs**: A standard laptop configuration allocating less than 8 GB RAM will cause bioepic to thrash when processing FAST5 files with many channels (> 512) and long read lengths (> 100 kb). Explicitly set `max_ram_gb` and `num_workers` in the TOML to match available hardware, or subsample reads using the `subsample_reads` parameter.

- **Ignoring Strandedness of Input RNA**: If RNA was prepared with an unstranded library kit but the config specifies ` stranded = true`, bioepic will reverse-complement signals incorrectly, flipping modification motifs on the negative strand and corrupting gene-level stoichiometry calls. Verify library preparation kit strandedness and align the config setting accordingly.

## Examples

### Run standard modification detection on a pod5 directory with a reference genome
**Args:** `detect --input-dir ./pod5_runs/ERR123456 --genome hg38.fa --config config.toml --output results/`
**Explanation:** This invokes the standard detect pipeline which resquiggles raw signals, aligns to the provided genome, and outputs per-site modification TSV files to the specified directory.

### Quantify stoichiometry for m6A sites across multiple samples using a manifest
**Args:** `quantify --manifest sample_manifest.tsv --mod-type m6A --model human_m6A_v2.bioepic --output stoichiometry.tsv`
**Explanation:** The quantify subcommand reads a manifest of sample paths, aggregates m6A log-odds scores per site, and computes fractional modification (stoichiometry) per gene per sample.

### Train a custom caller for yeast pseudouridine using labeled FAST5 reads
**Args:** `train --fast5-dir ./labeled_reads/ --labels yeast_psi_sites.bed --features canonical --epochs 50 --output yeast_psi_model.bioepic`
**Explanation:** The train subcommand fine-tunes a bioepic model on a labeled dataset of pseudouridine sites, using canonical signal features for 50 training epochs and saving the resulting checkpoint.

### Call modifications denovo without a reference using adaptive mode
**Args:** `call --mode denovo --input-dir ./fast5_dir/ --basecall-model "dna_r10.4.1_e8.1_hac@latest" --adaptive-threshold 0.05 --output denovo_calls/`
**Explanation:** The denovo mode discovers modification sites without a reference by clustering signal anomalies, using an adaptive p-value threshold of 0.05 to accept candidate sites.

### Validate a reference transcriptome before running detection
**Args:** `validate-ref --genome hg38.fa --transcriptome gencode_v45.tx --check-duplicates --check-strandedness`
**Explanation:** This companion command audits the reference files for duplicate IDs, strandedness mismatches, and missing isoform entries, printing warnings before the main detection run.

### Run detection with explicit RAM and worker overrides on a high-capacity node
**Args:** `detect --input-dir ./run1/ --genome hg38.fa --config base.toml --max-ram-gb 32 --num-workers 16 --output ./out/`
**Explanation:** Explicit flags override the TOML config, allocating 32 GB RAM and 16 parallel workers for a large nanopore run processed on a multi-core server node.