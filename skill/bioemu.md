---
name: bioemu
category: sequence-simulation
description: A tool for generating synthetic biological sequences and data for testing, benchmarking, and simulation purposes. Supports DNA, RNA, and protein sequences with configurable evolutionary models and error profiles.
tags: [simulation, synthetic-data, sequence-generation, testing, benchmarking]
author: AI-generated
source_url: https://github.com/bioemu/bioemu
---

## Concepts

- **Input/Output Formats**: bioemu reads plain-text configuration files (JSON or YAML) describing the desired simulation parameters and outputs standard biological formats including FASTA, FASTQ, and multi-FASTA archives. Output format is specified via the `--format` flag.
- **Sequence Models**: The tool supports three primary sequence generation models: (1) `random` generates i.i.d. sequences with uniform or custom base frequencies, (2) `markov` generates sequences using a k-th order Markov chain trained on a reference sequence, and (3) `evolutionary` applies an evolutionary model (e.g., Jukes-Cantor, Kimura 2-parameter) to a seed sequence over a specified number of generations.
- **Error Injection**: Sequencing error profiles can be injected via `--error-model`, which accepts profiles like `illumina-novaseq`, `pacbio-hifi`, or a custom JSON file specifying per-base error rates. Error rates are applied after sequence generation when using `--add-errors`.
- **Metadata Annotations**: Generated sequences automatically receive placeholder annotations including unique sequence identifiers (`seq_ID`), simulated quality scores for FASTQ output (Phred-scaled), and metadata fields for source organism context (`--organism-tag`).
- **Companion Binaries**: bioemu ships with `bioemu-ref` for converting reference GenBank/EMBL files into simulation-ready Markov models, and `bioemu-eval` for comparing synthetic outputs against reference datasets using metrics like Shannon entropy and k-mer frequency correlation.

## Pitfalls

- **Mismatched Base Frequencies**: Specifying `--base-freq` with values that do not sum to 1.0 causes the tool to silently rescale the frequencies, producing unintended sequence composition bias. Always ensure frequencies sum to exactly 1.0 or omit the flag to use uniform defaults.
- **Overwriting Existing Output**: Using `--output` with a filename that already exists will overwrite it without warning or confirmation, resulting in permanent data loss of previous files. Always verify the output path or use `--output-mode append` to preserve existing data.
- **Insufficient Evolutionary Steps**: Setting `--generations` below 10 when using `evolutionary` model produces sequences that are too similar to the seed, causing downstream tools to misclassify them as duplicates rather than independently evolved sequences. Use at least 50 generations for realistic divergence.
- **FASTQ Format Misconfiguration**: Requesting FASTQ output (`--format fastq`) without specifying an `--error-model` results in uniform quality scores of Q30, which may not reflect the actual error distribution of the sequencing platform being emulated. Pair `--format fastq` with `--error-model` for realistic quality profiles.
- **Large Sequence Counts Without Chunking**: Generating more than 10,000 sequences (`--count`) in a single run without `--chunk-size` causes excessive memory usage and slow I/O. Use `--chunk-size 1000` to process sequences in batches for runs exceeding 5,000 sequences.

## Examples

### Generate a simple random DNA sequence set
**Args:** `generate --count 100 --length 500 --model random --format fasta --output random_seqs.fasta`
**Explanation:** Creates 100 random DNA sequences of 500 nucleotides each using uniform base frequencies, outputting in FASTA format for use in basic pipeline testing.

### Generate FASTQ reads with Illumina error profile
**Args:** `generate --count 5000 --length 150 --model evolutionary --seed-seq ACGTGA --generations 200 --error-model illumina-novaseq --format fastq --output illumina_reads.fq`
**Explanation:** Produces 5,000 evolutionary-derived 150bp reads with realistic Illumina Novaseq error rates and quality scores, suitable for validating read-alignment pipelines.

### Build a Markov model from a reference FASTA file
**Args:** `bioemu-ref --input ecoli_genome.fa --order 3 --output ecoli_markov.json`
**Explanation:** Trains a 3rd-order Markov chain on the E. coli genome FASTA file, outputting a JSON model file that can be passed to `generate` via the `--markov-model` flag.

### Generate protein sequences with custom amino acid frequencies
**Args:** `generate --count 200 --length 300 --model random --base-freq 0.3 0.25 0.2 0.15 0.1 --alphabet protein --format fasta --output custom_proteins.fa`
**Explanation:** Creates 200 random protein sequences of 300 amino acids using custom residue frequencies (0.3, 0.25, 0.2, 0.15, 0.1), outputting in FASTA format for homology modeling validation.

### Evaluate similarity between synthetic and reference datasets
**Args:** `bioemu-eval --synthetic test_seqs.fasta --reference real_seqs.fasta --metrics entropy kmer --kmer-size 4 --output eval_report.json`
**Explanation:** Computes Shannon entropy and 4-mer frequency correlation between synthetic and real sequence sets, writing a JSON report to evaluate the fidelity of the simulation.

### Generate sequences in chunks to manage memory
**Args:** `generate --count 20000 --length 1000 --model markov --markov-model human_markov.json --chunk-size 2000 --output large_sim.fasta`
**Explanation:** Produces 20,000 sequences from a Markov model in batches of 2,000 to avoid memory exhaustion, suitable for large-scale benchmarking studies.

### Simulate paired-end reads with variable insert sizes
**Args:** `generate --count 10000 --length 100 --model evolutionary --seed-seq TACGTAC --generations 150 --paired --insert-size 350 --std-dev 50 --format fastq --output paired_seqs.R1.fq paired_seqs.R2.fq`
**Explanation:** Generates 10,000 paired-end read pairs with mean insert size of 350bp and 50bp standard deviation, outputting mate files in FASTQ format for paired-read alignment testing.

### Append new sequences to an existing output file
**Args:** `generate --count 50 --length 200 --model random --output more_seqs.fasta --output-mode append`
**Explanation:** Appends 50 additional random sequences to the end of `more_seqs.fasta` without overwriting, allowing incremental dataset building.