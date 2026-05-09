---
name: badread
category: Read Simulation
description: A tool to simulate error-prone long reads (Oxford Nanopore, PacBio) from a reference genome for testing bioinformatics pipelines.
tags: [long-reads, simulation, nanopore, pacbio, error-model, testing, assembly]
author: AI-generated
source_url: https://github.com/rrwick/Badread
---

## Concepts

- **Input**: badread takes a reference genome in FASTA format and generates simulated long reads with realistic errors. The reference can be a single sequence or a multi-contig assembly.
- **Output**: Reads are written to stdout in FASTQ format, including quality scores that reflect the simulated error rates. Use shell redirection to save to a file.
- **Error modeling**: badread simulates three error types: substitutions (mismatches), insertions (extra bases), and deletions (missing bases). The rates are controlled by per-platform preset parameters or custom values.
- **Platform presets**: Use `--quantity` to specify total bases (e.g., `1G`) or coverage (e.g., `30x`). Each platform preset (nanopore, pacbio, solid) has default error profiles optimized from real data.
- **Chimeric reads**: Enable chimeric read generation with `--chimericReads` to simulate fusion events common in some long-read datasets.

## Pitfalls

- **Omitting input reference**: Running badread without a reference FASTA file will cause it to read from stdin, which may lead to unexpected behavior if you intend to use a file. Always provide a reference explicitly.
- **Incorrect quantity format**: Specifying quantity without units (e.g., `--quantity 1000` instead of `--quantity 1k` or `--quantity 1000bp`) causes ambiguous interpretation and may generate far fewer bases than intended.
- **Overly high error rates**: Setting custom error rates that are too high (e.g., `--max_errors 0.5`) produces reads with little usable sequence, rendering downstream analysis results meaningless.
- **Forgetting read length limits**: Default max read length is 100000 bp. For simulating very long Nanopore reads (10kb+), specify appropriate `--max_length` values or the reads will be artificially truncated.
- **Ignoring seed for reproducibility**: Without `--seed`, each run produces different random reads. Always set a seed when you need reproducible results for testing or debugging.

## Examples

### Generate 30x coverage Nanopore reads from a reference
**Args:** `reference.fasta --quantity 30x --technology nanopore`
**Explanation:** This generates simulated Nanopore reads at 30-fold coverage from the reference genome, automatically applying preset error rates typical of Oxford Nanopore R9.4 chemistry.

### Output 1 gigabase of PacBio reads to a FASTQ file
**Args:** `ref.fa --quantity 1G --technology pacbio --out reads.fq`
**Explanation:** This creates one gigabase of simulated PacBio HiFi reads and writes them directly to reads.fq rather than stdout, using PacBio-specific error profiles.

### Create 10000 random reads with custom length distribution
**Args:** `genome.fasta --quantity 10000 --min_length 5000 --max_length 15000 --length_dist gaussian --out random_reads.fq`
**Explanation:** This generates exactly 10000 reads with lengths normally distributed around the midpoint of 5000-15000 bp, useful for testing assembly with specific read length requirements.

### Enable chimeric reads to test fusion detection
**Args:** `assembly.fa --quantity 10x --chimericReads 0.01 --technology nanopore`
**Explanation:** This produces Nanopore reads with 1% chimerism, inserting small amounts of sequence from distant genomic regions into single reads to test fusion detection or alignment tools.

### Set a random seed for reproducible test data
**Args:** `ref.fasta --quantity 5x --seed 42 --technology nanopore --out reproducible_reads.fq`
**Explanation:** Using seed 42 ensures identical read generation across runs, which is essential when comparing tool performance or debugging pipelines with the same test dataset.

### Use custom error rates instead of presets
**Args:** `test.fa --quantity 1x --technology custom --suberror 0.05 --inserror 0.02 --delerror 0.02`
**Explanation:** This manually sets 5% substitution, 2% insertion, and 2% deletion error rates, allowing testing of tools under specific error conditions not matching standard platform presets.

### Simulate a subset using a seed with read start positions
**Args:** `ecoli.fa --quantity 10x --seed 1234 --technology nanopore > test_reads.fq`
**Explanation:** Combined with a seed, this outputs to stdout which can be redirected to create a fixed test set for benchmarking multiple assemblers on identical input data.