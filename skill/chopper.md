---
name: chopper
category: Nucleic Acids / FASTQ Manipulation
description: A fast command-line utility for converting, filtering, and manipulating FASTQ and FASTA sequence files. Supports quality score format conversion, length-based filtering, and format transformations between FASTQ, FASTA, and their compressed variants.
tags: [fastq, fasta, conversion, quality-scores, filtering, trimming, bioinformatics]
author: AI-generated
source_url: https://github.com/samtools/htslib
---

## Concepts

- **Primary I/O Model**: chopper reads sequence data from stdin or a specified input file and writes to stdout or a specified output file, enabling seamless pipe integration with other Unix tools in bioinformatics pipelines.
- **Quality Score Conversion**: The tool can convert between Phred+33 (Sanger) and Phred+64 (Illumina 1.3-1.7) quality score encodings using the `-g` flag, which is essential when combining reads from different sequencing platforms or old/newer pipelines.
- **Format Transformation**: Files can be converted between FASTQ, FASTA, and their gzipped variants; the output format is determined by filename extensions (.fq, .fa, .fq.gz, .fa.gz) or explicit flags.
- **Length-based Filtering**: Reads can be filtered by minimum or maximum sequence length using `--minlen` and `--maxlen`, removing truncated reads or adapter contaminants before downstream analysis.

## Pitfalls

- **Mismatched Quality Score Encoding**: Using the wrong quality score format flag causes garbled quality strings in downstream tools, leading to incorrect variant calling or failed alignments because most modern mappers expect Phred+33 (Sanger) format by default.
- **Omitting Input/Output Flags in Pipeline**: When chaining chopper with other tools via Unix pipes, forgetting to specify both input (`-i`) and output (`-o`) flags can cause data to be read from or written to the wrong stream, resulting in empty or corrupt output files.
- **Incorrect File Extension Handling**: Output files without the correct extension (e.g., writing gzipped output to a `.fq` file instead of `.fq.gz`) will not be automatically compressed, causing large unexpected file sizes and potential downstream compatibility issues.
- **Filtering Without Quality Trimming**: Applying length filters before quality trimming (e.g., with `fastp` or `trimmomatic`) discards good reads that only need end-trimming, reducing the effective yield of sequencing data unnecessarily.

## Examples

### Convert a FASTQ file to FASTA format
**Args:** `-i input.fq -o output.fa`
**Explanation:** This flags explicitly specify the input FASTQ file and output FASTA file, performing the format conversion by dropping quality scores which are not needed for FASTA output.

### Convert quality scores from Illumina 1.8+ (Phred+33) to older Illumina (Phred+64)
**Args:** `-g -i input.fq -o output_illumina.fq`
**Explanation:** The `-g` flag converts quality scores from Sanger (Phred+33) to Illumina 1.3-1.7 encoding (Phred+64), required for compatibility with legacy pipelines or older analysis tools.

### Filter reads shorter than 50 bases
**Args:** `--minlen 50 -i reads.fq -o filtered.fq`
**Explanation:** Reads below the minimum length threshold are discarded, removing potential adapter remnants or highly degraded sequences from downstream analysis.

### Read from stdin and write to stdout for piping
**Args:** `-i - -o -`
**Explanation:** Using dash (`-`) for both input and output enables chopper to read from stdin and write to stdout, allowing integration into Unix pipelines like `zcat input.fq.gz | chopper -i - -o - | gzip > output.fq.gz`.

### Convert and compress output in a single step
**Args:** `-i input.fq -o output.fq.gz`
**Explanation:** The tool automatically applies gzip compression based on the `.fq.gz` output extension, creating a compressed file directly without needing an external compression step.