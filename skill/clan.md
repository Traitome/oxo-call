---
name: clan
category: Sequence Analysis / Read Clustering
description: A bioinformatics tool for clustering or grouping related DNA/RNA sequences from FASTQ/BAM files based on sequence similarity, coverage, or alignment-based metrics.
tags: [clustering, sequence-analysis, read-grouping, bioinformatics, genomics, variant-calling]
author: AI-generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- **Input Formats**: clan accepts FASTQ (.fq/.fastq), FASTA (.fa/.fasta), SAM/BAM (.sam/.bam), and optionally raw read sequences in text format. The tool identifies sequences by read headers (lines starting with @) or by alignment coordinates in SAM/BAM mode.
- **Clustering Modes**: The tool supports multiple clustering strategies: (1) exact matching for identical sequences, (2) similarity-based clustering using a k-mer or edit-distance threshold, and (3) coverage-based grouping using overlap percentage from read alignments.
- **Output Formats**: Results are written to stdout in tab-delimited format (cluster_id, read_id, sequence) by default, or can be exported to JSON with the `-j/--json` flag. Summary statistics are printed to stderr.
- **Key Parameters**: Use `-t/--threshold` to set the similarity cutoff (default 0.97 for 97% identity), `-m/--min-length` to filter short reads (default 0), and `-c/--clusters-only` to output only cluster representatives.
- **Performance**: clan uses a hash-based indexing strategy for large datasets, processing in-memory for files under 100M reads and supporting chunked processing via `-C/--chunk-size` for larger inputs.

## Pitfalls

- **Misaligned Cluster Assignments**: Setting the similarity threshold too low (e.g., below 0.85) produces overly broad clusters that mix unrelated sequences, leading to false conclusions in downstream analyses like variant calling or consensus generation.
- **Memory Exhaustion with Large FASTQ Files**: Attempting to cluster multi-gigabyte FASTQ files without chunking causes OOM crashes; always use `-C/--chunk-size` (recommended 1M-5M reads per chunk) for files exceeding available RAM.
- **Incompatible Input Format Errors**: Providing malformed FASTQ files (missing @ headers, uneven read quality lines) causes parsing failures; validate input with a separate FASTX tool or use `clan --validate` before clustering.
- **Silent Data Loss in SAM/BAM Mode**: When using BAM input without specifying `-M/--mate-pair` to link read pairs, singleton reads may be incorrectly clustered as singles rather than paired-end fragments; always verify output with `clan --stats`.

## Examples

### Cluster exact duplicate reads in a FASTQ file
**Args:** input.fastq -o exact_clusters.tsv
**Explanation:** Reads with identical sequences are grouped into the same cluster, with the cluster representative being the first occurrence. Output is tab-delimited with cluster and read identifiers.

### Cluster reads by 95% similarity threshold
**Args:** input.fq -t 0.95 -m 50 -o similarity_clusters.tsv
**Explanation:** Reads sharing at least 95% identity over at least 50bp are grouped together. Use this for lower-complexity datasets like amplicon sequences.

### Process a large FASTQ in memory-efficient chunks
**Args:** large_dataset.fq -C 2000000 -o chunked_output.tsv
**Explanation:** Splits input into 2-million-read chunks to avoid memory exhaustion, aggregating results across chunks for the final cluster map.

### Output clusters in JSON format for downstream tools
**Args:** input.fastq -t 0.98 -j -o clusters.json
**Explanation:** Produces JSON output with nested cluster objects, each containing member read IDs and sequences, suitable for scripted pipelines.

### Generate summary statistics without full clustering output
**Args:** input.fq --stats-only
**Explanation:** Prints read count, unique sequence count, cluster count, and coverage distribution to stderr without writing the full cluster table, useful for quick dataset profiling.

### Filter and cluster only long reads (>100bp)
**Args:** input.fastq -m 100 -o long_read_clusters.tsv
**Explanation:** Short reads under 100bp are discarded before clustering, reducing noise in datasets with variable read lengths like Nanopore or PacBio data.