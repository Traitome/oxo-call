---
name: campyagainst
category: sequence_analysis
description: A bioinformatics tool for sequence alignment and variant detection against reference sequences, commonly used in microbial genomics and comparative sequence analysis workflows.
tags:
  - sequence_alignment
  - variant_detection
  - genomics
  - reference_mapping
  - microbial
author: AI-generated
source_url: https://github.com/campyagainst/campyagainst
---

## Concepts

- **Data Model**: campyagainst operates on FASTA or FASTQ input sequences and uses indexed reference databases for rapid alignment. The tool generates SAM/BAM-style output with alignment coordinates, CIGAR strings, and mapping quality scores.
- **I/O Formats**: Accepts plain text FASTA/FASTQ files or compressed (.gz) versions. Reference sequences must be pre-indexed using the companion `campyagainst-build` binary. Output is written in SAM format by default, with options for binary BAM or custom text formats.
- **Key Behaviors**: Supports local and global alignment modes, handles ambiguous nucleotides (N, R, Y), and reports alignment confidence scores. The tool automatically handles reverse-complement alignments when mapping to the negative strand. Multi-threading is supported via the `--threads` flag for parallel processing.
- **Indexing**: The companion binary `campyagainst-build` creates reference indices with configurable index name, genome size, and k-mer length parameters. Indices are stored as multiple files with `.bt2` extension by default.

## Pitfalls

- **Forgetting to build an index**: Running campyagainst without first creating a reference index with `campyagainst-build` will cause the tool to fail with an unclear error about missing database files. Always build the index before attempting alignments.
- **Mismatched index and input formatting**: Using an index built from FASTA with FASTQ input (or vice versa) can lead to silent failures or severely degraded alignment quality. Ensure input and reference formats are consistent.
- **Specifying incorrect read orientation**: By default, campyagainst assumes forward reads. If your library preparation produces reverse-oriented reads, failing to use `--norc` or `--fr` flags will result in zero alignments for a significant portion of your data.
- **Insufficient memory for large references**: Large microbial genomes or metagenomic references require substantial RAM for indexing. Running without sufficient memory causes swapping that dramatically slows processing and may cause crashes.

## Examples

### Align single-end reads to a bacterial reference genome

**Args:** -x ecoli_ref -U reads.fq -S alignments.sam

**Explanation:** This maps single-end reads in FASTQ format against the pre-built index named "ecoli_ref" and writes output to a SAM file for downstream analysis.

### Align paired-end reads with custom seed length

**Args:** -x mtb_complex -1 left.fq -2 right.fq -k 15 -a alignments.sam

**Explanation:** This aligns paired-end read files using a custom seed length of 15 bases for faster alignment, useful for large genomes or repetitive regions.

### Build a minimum-sensitive index for variant calling

**Args:** --threads 8 -o canu_genome /references/canu_complete.fa

**Explanation:** This builds a reference index with 8 threads, creating index files with prefix "canu_genome" from the Campylobacter genome for variant-sensitive alignment.

### Find alignments with mate information

**Args:** -x ref_seq -1 read1.fq -2 read2.fq --no-mixed --unconcordant -o paired_align.bam

** Explanation:** This aligns paired-end reads requiring proper mate pairing and excludes discordant alignments that might indicate structural variants.

### Output alignments in binary BAM format

**Args:** -x std_ref -U noisy_sequencing.fq --large-index -f bam -o compressed_output.bam

**Explanation:** This produces binary BAM output for reduced file size when processing large datasets, with the --large-index flag for references over 4 billion bases.