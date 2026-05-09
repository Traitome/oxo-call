---
name: captus
category: Genomics/Assembly
description: A tool for extracting, filtering, and manipulating genomic assemblies from various formats. Used to retrieve specific sequences, filter assemblies by length or taxonomy, and convert between assembly formats.
tags: [assembly, extraction, genomics, fasta, contigs, filtering]
author: AI-generated
source_url: https://github.com/gtorilab/captus
---

## Concepts

- **Input formats:** captus accepts FASTA, FASTQ, and potentially SAM/BAM files containing assembled contigs or sequences. The tool can process multiple input files in batch mode.
- **Output formats:** Extracted sequences can be written to FASTA format, with optional filtering applied based on sequence length, coverage, or taxonomy annotations.
- **Filtering capabilities:** Sequences can be filtered by minimum/maximum length thresholds, by taxonomic assignment (if provided in sequence headers), or by coverage values embedded in the sequence data.
- **Index-free operation:** captus operates directly on sequence files without requiring genome indices, making it efficient for quick extraction tasks without preprocessing overhead.

## Pitfalls

- **Missing sequence headers:** If input FASTA files lack descriptive headers with taxonomic or length information, filtering by taxonomy or length criteria will fail silently and return no matches.
- **Case sensitivity in filters:** Taxonomy filters are case-sensitive; using "bacteria" instead of "Bacteria" will result in zero matches when the annotation uses title case.
- **Memory with large files:** Processing very large assemblies (multi-GB FASTA files) without streaming flags can cause memory exhaustion; use chunked processing for genome-scale datasets.
- **Invalid length thresholds:** Setting minimum length greater than maximum length produces empty output without warning; verify threshold ordering before execution.

## Examples

### Extract all sequences longer than 500 bp
**Args:** `--min_length 500 --input assembly.fasta --output long_contigs.fasta`
**Explanation:** This extracts sequences from assembly.fasta that meet the minimum length requirement, writing them to the output file.

### Filter sequences by taxonomic annotation
**Args:** `--filter_taxa "Escherichia" --input metagenome_assembly.fasta --output e coli_seqs.fasta`
**Explanation:** This selects only sequences whose headers contain "Escherichia" in the annotation, useful for target organism extraction.

### Convert FASTQ to FASTA format
**Args:** `--convert fastq2fasta --input reads.fq --output reads.fa`
**Explanation:** This converts nucleotide sequences from FASTQ to FASTA format, stripping quality scores while preserving sequence data.

### Extract sequences within a length range
**Args:** `--min_length 1000 --max_length 5000 --input contigs.fasta --output medium_contigs.fasta`
**Explanation:** This filters to include only sequences with length between 1000 and 5000 base pairs, excluding both shorter and longer sequences.

### Batch extract from multiple files
**Args:** `--input_dir ./assemblies/ --output_dir ./filtered/ --min_length 200`
**Explanation:** This processes all sequence files in the input directory, applying length filtering and writing results to corresponding files in the output directory.

### Remove duplicate sequences
**Args:** `--dedupe --input assembly.fasta --output deduped.fasta`
**Explanation:** This identifies and removes duplicate or near-identical sequences based on sequence identity threshold.

### Extract sequences by exact header match
**Args:** `--header "contig_42" --input assembly.fasta --output target.fasta`
**Explanation:** This extracts the single sequence with exact header match "contig_42", useful for retrieving specific known contigs.