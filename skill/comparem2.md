---
name: comparem2
category: Metagenomics Assembly Analysis
description: A tool for comparing metagenomic assemblies to identify unique and shared sequences, perform dereplication, and analyze assembly quality across multiple samples.
tags: [metagenomics, assembly, comparison, dereplication, sequence-analysis]
author: AI-generated
source_url: https://github.com/comparem2/comparem2
---

## Concepts

- **Input Format**: comparem2 accepts FASTA and FASTQ files as primary inputs, where each sequence entry must have a unique identifier. The tool parses these files to extract sequences for downstream comparison operations.
- **Sequence Comparison Algorithm**: The tool uses alignment-free methods (k-mer based hashing) to efficiently compare large sets of sequences without performing computationally expensive pairwise alignments, making it suitable for whole-genome or metagenome scale comparisons.
- **Output Types**: Results are produced in multiple formats including FASTA files of unique sequences, tabular summary reports showing shared/unique sequence counts per input, and optionally alignment coordinates for detailed analysis.
- **Reference-based Mode**: Sequences can be compared against a reference database to identify which query sequences match existing references, useful for annotation and contamination screening.

## Pitfalls

- **Non-unique Sequence Headers**: If input FASTA files contain duplicate sequence identifiers, comparem2 may silently skip sequences or produce incorrect counts. Always ensure each sequence has a unique header before running the tool.
- **Missing Sequence Index**: Running comparison without pre-building the required index causes the tool to fail or produce empty results. Ensure all required index files are generated before running comparison commands.
- **Incompatible Sequence Types**: Mixing nucleotide and protein sequences in the same comparison run will produce meaningless results since the k-mer hashing schemes are incompatible. Keep nucleotide and protein sequences in separate comparison jobs.

## Examples

### Compare two metagenomic assemblies to find unique sequences
**Args:** compare --query assembly1.fasta --reference assembly2.fasta --unique-out unique_to_assembly1.fasta
**Explanation:** This identifies sequences present in assembly1 but absent from assembly2 by removing all k-mers that also appear in the reference assembly.

### Find sequences shared between multiple assemblies
**Args:** compare --query combined.fasta --reference all_samples.fasta --shared-out shared_seqs.fasta
**Explanation:** Extracts sequences that have matching k-mers in the reference set, representing conserved content across samples.

### Dereplicate redundant sequences from a single assembly
**Args:** dereplicate --input raw_assembly.fasta --output dedup_assembly.fasta --min-cov 2
**Explanation:** Removes duplicate or near-identical sequences keeping only unique representatives, with min-cov specifying minimum coverage threshold.

### Compare query against a reference database
**Args:** annotate --query reads.fasta --ref-db reference_db.fasta --out annotations.tsv
**Explanation:** Maps each query sequence against the reference database and outputs alignment/identity results in tabular format.

### Filter sequences by minimum length before comparison
**Args:** filter --input large_dataset.fasta --output filtered.fasta --min-len 500
**Explanation:** Removes sequences shorter than 500 bp prior to comparison, reducing computational load and improving result quality.

### Build a custom reference database for multiple samples
**Args:** build-db --inputs sample1.fasta sample2.fasta sample3.fasta --output combined_db.fasta
**Explanation:** Combines multiple input files into a single reference database for batch comparison operations.

### Generate summary statistics for comparison results
**Args:** stats --result-dir ./comparison_results --output summary.tsv
**Explanation:** Parses previous comparison output files and generates tabular summary including sequence counts, sizes, and overlap percentages.