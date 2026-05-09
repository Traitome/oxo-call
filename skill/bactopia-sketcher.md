---
name: bactopia-sketcher
category: genomics/genome-sketching
description: Creates MinHash sketches from bacterial genome sequences for fast pairwise comparison, clustering, and phylogenetic analysis. Outputs binary sketch files compatible with the mash algorithm.
tags:
  - minhash
  - genome-sketching
  - bacterial-wgs
  - mash
  - k-mer
  - fastq
  - assembly
  - ANI
author: AI-generated
source_url: https://github.com/bactopia/bactopia
---

## Concepts

- **MinHash Sketching:** bactopia-sketcher extracts k-mers from input sequences (FASTQ or FASTA) and uses a hashing function to select a representative subset, creating a compact "sketch" that captures the genomic identity while enabling fast distance calculations.
- **Input Flexibility:** The tool accepts single-end reads, paired-end reads, assembled genomes (FASTA), or pre-computed read sets. Input is specified via `--reads` (or `--fasta` for assemblies) and supports glob patterns for multiple samples.
- **Sketch Parameters:** The core parameters are k-mer size (`-k`), sketch size (`-s`), and hash seed (`--seed`). Default k=21 and s=1000 are suitable for bacterial genomes; larger sketches increase sensitivity but also file size and compute time.
- **Output Format:** Generated sketches use the `.msh` extension and follow the mash sketch format, which can be compared directly against other sketches using `bactopia-sketcher dist` or external mash tools.
- **Companion Binaries:** Use `bactopia-sketcher dist` to compute pairwise distances between sketches, and `bactopia-sketcher build` (or mash) to build sketch databases for fast queries.

## Pitfalls

- **Incompatible k-mer sizes:** Using a k-mer size that is too small (e.g., k=11) introduces noise from repetitive sequences, while k>31 may exclude meaningful variation in smaller bacterial genomes. Always match k across compared sketches.
- **Mixing read and assembly sketches:** Attempting to compare sketches generated from raw reads against those generated from assemblies will yield inflated distances because read sketches include sequencing error while assembly sketches are consensus-based.
- **Insufficient sketch size for similar genomes:** With default s=1000, very closely related strains (ANI > 99%) may appear more divergent than they truly are. Increase sketch size (e.g., -s 5000) when differentiating highly similar isolates.
- **Missing read handling:** If any input FASTQ file is missing or empty, the tool may fail silently or produce a sketch with zero coverage. Always validate input files exist before running.
- **Hash seed inconsistency:** Using different hash seeds (--seed) across batches produces incomparable sketches. Record or standardize the seed value in your workflow to ensure reproducibility.

## Examples

### Generate a MinHash sketch from a single-end FASTQ file

**Args:** `--reads sample_R1.fastq.gz -o sample.msh`
**Explanation:** This creates a sketch from raw single-end reads using default k=21 and s=1000, outputting a mash-compatible sketch file.

### Generate a sketch from paired-end reads

**Args:** `--reads "sample_{1,2}.fastq.gz" -o sample.msh`
**Explanation:** Using brace expansion, both read files are combined to build a single sketch representing the full genomic content of the isolate.

### Generate a sketch from an assembled genome

**Args:** `--fasta sample_assembly.fasta -o sample.msh`
**Explanation:** Providing a FASTA file bypasses k-mer extraction from reads and directly sketches the assembled contigs, resulting in lower error rates.

### Adjust k-mer and sketch size for high-resolution comparison

**Args:** `--fasta isolate.fasta -k 25 -s 5000 -o isolate_ms25.msh`
**Explanation:** Increasing k to 25 and sketch size to 5000 provides finer resolution for distinguishing nearly identical strains at the cost of larger output files.

### Compute pairwise distances between two sketch files

**Args:** `--dist sketch1.msh sketch2.msh`
**Explanation:** Using the companion subcommand, this calculates the Mash distance (equivalent to evolutionary distance) between the two sketches, reporting similarity as 1 - distance.