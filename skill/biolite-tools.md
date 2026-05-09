---
name: biolite-tools
category: Bioinformatics Utilities
description: A collection of bioinformatics command-line tools for sequence analysis, format conversion, and genomic data processing. Includes companion binaries for index building and specialized data manipulation tasks.
tags: [bioinformatics, sequence-analysis, genomics, data-processing, ngs]
author: AI-generated
source_url: https://github.com/biolite-tools/biolite-tools
---

## Concepts

- **Data Model**: biolite-tools processes standard bioinformatics file formats including FASTA, FASTQ, SAM, BAM, BED, and VCF. The tools operate on both plain text and compressed (gzip) input files, automatically detecting format from file extensions.
- **Index Building**: For sequence alignment and k-mer operations, use the companion binary `biolite-tools-build` to create proper index structures before running the main analysis tools. Index files are stored with `.bt2` extension by default.
- **Streaming and Pipelines**: All biolite-tools binaries support stdin/stdout for pipeline integration. Use `-` as the input argument to read from standard input, enabling seamless piping between tools in chained workflows.
- **Output Naming**: Output filenames are automatically generated based on input files unless explicitly specified with the `-o` flag. The tool appends appropriate suffixes (e.g., `.filtered`, `.sorted`) to input filenames.

## Pitfalls

- **Forgetting Index Files**: Running analysis tools without pre-built index files causes failures with cryptic "index not found" errors. Always run `biolite-tools-build` first for alignment or k-mer based operations.
- **Incompatible Format Combinations**: Attempting to process SAM format with tools expecting BAM (or vice versa) without explicit format conversion will produce incorrect results or errors. Use dedicated converter tools first.
- **Ignoring Thread Settings**: Default single-threaded execution can be extremely slow on large datasets. Use the `-p` flag to specify thread counts based on available CPU cores for significant speedup.
- **Overwriting Outputs**: By default, tools will overwrite existing output files without warning. Use the `-k` flag to keep existing files and prevent accidental data loss.

## Examples

### Build index for reference sequence
**Args:** `build -t 8 reference.fasta genome_index`
**Explanation:** Creates a searchable index file using 8 threads for faster subsequent alignment operations against the reference genome.

### Filter sequences by quality score
**Args:** `filter -q 30 -o quality_filtered.fasta input.fasta`
**Explanation:** Removes sequences with any base having quality score below 30, outputting to a explicitly named file.

### Convert FASTQ to FASTA format
**Args:** `convert -i input.fastq -o output.fasta -f fasta`
**Explanation:** Converts sequence format from FASTQ to FASTA, stripping quality information while preserving sequence data.

### Sort alignments by genomic position
**Args:** `sort -o sorted.bam unsorted.bam`
**Explanation:** Sorts BAM alignments by genomic coordinate, producing a file ready for downstream analysis tools requiring sorted input.

### Extract specific genomic regions
**Args:** `extract -r chr1:1000000-2000000 -o region.bed input.bed`
**Explanation:** Pulls only BED features overlapping chromosome 1 positions 1Mb to 2Mb for targeted analysis.

### Count k-mer frequencies
**Args:** `kmer -k 21 -o kmers.txt input.fasta`
**Explanation:** Calculates 21-mer occurrence frequencies across all sequences in the input FASTA file.

### Merge multiple alignment files
**Args:** `merge -o merged.bam sample1.bam sample2.bam sample3.bam`
**Explanation:** Combines three BAM files into a single sorted output, handling duplicate markers appropriately.

### Calculate read depth statistics
**Args:** `depth -b 50 -p 4 -o stats.txt input.bam`
**Explanation:** Computes per-base depth using 50bp bins across 4 threads, outputting summary statistics to a text file.