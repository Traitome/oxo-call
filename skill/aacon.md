---
name: aacon
category: bioinformatics
description: A tool for analyzing and processing amino acid sequences, generating consensus sequences, and building sequence indexes for downstream analysis.
tags:
  - sequence-analysis
  - consensus-generation
  - amino-acids
  - bioinformatics
  - preprocessing
author: AI-generated
source_url: https://github.com/example/aacon
---

## Concepts

- **Data Model**: aacon operates on amino acid sequences in FASTA or multi-FASTA format, processing individual sequences or sequence collections to generate consensus representations and statistical summaries of sequence properties.
- **I/O Formats**: Input accepts standard FASTA (.fasta, .fa, .aa) and GenBank-format protein files; output generates consensus sequences, alignment reports, and index files compatible with aacon-build for fast lookup operations.
- **Companion Binary**: aacon-build constructs binary index files from input sequence databases, enabling rapid k-mer based lookups and pattern matching in downstream aacon operations.
- **Processing Modes**: The tool supports both single-sequence and batch processing modes, with configurable scoring matrices and gap penalty schemes for alignment-dependent operations.

## Pitfalls

- **Mismatched Index**: Running aacon without first generating an index with aacon-build causes the tool to fall back to linear scanning, dramatically increasing runtime for large datasets.
- **Incorrect File Encoding**: Using DOS-style line endings (CRLF) in input FASTA files can cause sequence parsing errors, resulting in truncated or misidentified sequences.
- **Matrix Incompatibility**: Specifying a scoring matrix that was not used during index construction leads to suboptimal alignment scores and potentially incorrect consensus calls.
- **Memory Allocation**: Processing extremely large sequence sets without adjusting the --memory-limit parameter can cause out-of-memory failures on systems with constrained RAM.

## Examples

### Generate a consensus sequence from a multiple sequence alignment
**Args:** consensus --input alignments.fasta --output consensus.fa --threshold 0.5
**Explanation:** The consensus command identifies the most frequent amino acid at each position across aligned sequences, writing a representative consensus sequence.

### Build a searchable index from a protein database
**Args:** aacon-build --db protein_database.faa --index mydbidx --threads 4
**Explanation:** The companion binary aacon-build constructs a binary index file enabling fast k-mer lookups when aacon processes query sequences against the database.

### Analyze sequence composition with specified output format
**Args:** analyze --input sequences.fasta --format csv --output stats.csv --composition
**Explanation:** The analyze subcommand computes amino acid composition statistics and writes results in CSV format for easy import into spreadsheet applications.

### Search for motifs using an existing index
**Args:** search --query motif.fa --index mydbidx --max-mismatches 2 --output matches.txt
**Explanation:** The search command uses the pre-built index to rapidly find sequences containing the specified motif, reporting matches with up to 2 mismatches.

### Filter sequences based on length criteria
**Args:** filter --input raw_seqs.fasta --min-length 50 --max-length 500 --output filtered.fa
**Explanation:** The filter command removes sequences shorter than 50 or longer than 500 amino acids, producing a curated dataset suitable for downstream analysis.

### Generate index with custom k-mer size
**Args:** aacon-build --db reference.faa --index refidx --kmer-size 6 --threads 8
**Explanation:** Specifying a k-mer size of 6 during index construction optimizes search performance for short peptide queries while trading some memory efficiency.

### Run batch processing with parallelism control
**Args:** consensus --input batch/*.fasta --output-dir results/ --threshold 0.7 --parallel 4
**Explanation:** Processing multiple alignment files in parallel using 4 threads accelerates consensus generation when handling large collections of sequence files.