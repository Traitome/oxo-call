---
name: artic
category: Bioinformatics / Nanopore Sequencing Pipeline
description: A bioinformatics pipeline and collection of tools for targeted nanopore sequencing of viruses, designed for field deployment during outbreak response. Provides end-to-end processing from raw FASTQ files to consensus sequences and variant calls.
tags: [nanopore, viral-genomics, amplicon-sequencing, field-bioinformatics, outbreak-response, covid-19, ebola]
author: AI-generated
source_url: https://artic.readthedocs.io/en/latest/
---

## Concepts

- ARTIC is an amplicon-based sequencing pipeline that uses multiplexed primer schemes to amplify viral genomic regions, then sequences those amplicons on Oxford Nanopore Technology (ONT) devices. The pipeline processes raw nanopore reads through demultiplexing, primer trimming, alignment, and consensus generation stages.

- The pipeline uses **Medaka** for neural network-based consensus calling and variant detection, producing high-accuracy consensus sequences from noisy nanopore reads. Medaka models are scheme-specific and must match the primer scheme version used for sequencing.

- ARTIC projects use a structured directory layout with a `artic.json` configuration file that defines sample sheets, sequencing kit, flow cell ID, and other run metadata. This configuration drives automated processing through the `artic` command without manual flag specification per sample.

- The pipeline supports **multiple workflows**: `collect` retrieves FASTQ files from sequencing positions, `demux` assigns reads to samples, `trim` removes primer sequences, `align` maps reads to a reference, `filter` removes low-quality or chimerical reads, `track` monitors variants in real-time, `call` performs SNV/indel detection, and `polish` generates the final consensus sequence.

- Each viral scheme (e.g., SARS-CoV-2 v1, v3, v4; Ebola; influenza) defines primer coordinates and a reference genome. Using mismatched scheme versions between sequencing and analysis causes primer trimming errors and downstream artifacts in consensus sequences.

## Pitfalls

- Using a Medaka model that does not match the basecaller version or flow cell type used during sequencing leads to degraded consensus accuracy and false variant calls in low-complexity regions.

- Skipping the `filter` step or using permissive alignment thresholds allows chimerical reads and low-quality alignments to propagate into variant calls, producing false positive SNVs especially in mixed infection scenarios.

- Running the pipeline on insufficient compute resources without adjusting thread counts causes memory exhaustion or disk space errors during parallel alignment steps, potentially corrupting intermediate BAM files.

- Applying a primer scheme version different from the one used during library preparation results in off-target primer trimming, creating artificial deletions at primer binding sites in the consensus sequence.

- Failing to validate basecalling and demultiplexing metrics before running expensive alignment and variant calling steps wastes computation and may require reprocessing entire runs.

## Examples

### View available ARTIC commands and global options
**Args:** `help`
**Explanation:** Displays all top-level subcommands including `collect`, `demux`, `trim`, `align`, `filter`, `call`, `polish`, and `report`, along with their descriptions and global flags.

### Collect FASTQ files from a sequencing run directory
**Args:** `collect --run /data/run_2024_01_15 --sample-sheet samples.csv`
**Explanation:** Aggregates all FASTQ files from the specified run directory, organizing them by sample according to the provided sample sheet for downstream demultiplexing.

### Demultiplex reads by sample barcode
**Args:** `demux --fastq-directory /data/fastq_pass --output demux_output/ --kit SQK-RBK004`
**Explanation:** Sorts nanopore reads into sample-specific FASTQ files based on barcodes defined in the sequencing kit, outputting to the specified directory for primer trimming.

### Trim primer sequences from demultiplexed reads
**Args:** `trim --scheme SARS-CoV-2/v3 --min-length 400 --max-length 700`
**Explanation:** Removes primer sequences using the specified primer scheme and filters reads by amplicon length bounds, preparing clean reads for alignment and reducing artificial variants.

### Generate consensus sequence from aligned reads
**Args:** `polish --aligned-reads alignment.bam --scheme SARS-CoV-2/v3 --model r941_min_hac_snps`
**Explanation:** Uses Medaka to call a consensus sequence from the filtered BAM file, applying the scheme-specific primer coordinates and model for accurate variant detection.