---
name: chronumental
category: ancient-dna-analysis
description: A tool for simulating and modeling ancient DNA damage patterns, particularly deamination events (C→T and G→A transitions) that accumulate over time in ancient DNA sequences. Used for generating realistic simulated ancient DNA data for pipeline testing and method validation.
tags:
  - ancient-dna
  - damage-simulation
  - deamination
  - population-genetics
  - bioinformatics
  - dna-aging
author: AI-generated
source_url: https://github.com/m峨眉山/chronumental
---

## Concepts

- **Damage Model**: chronumental simulates cytosine (C) → thymine (T) deamination and guanine (G) → adenine (A) transitions, the predominant damage types in ancient DNA. The model calculates expected damage frequencies based on the formula: P(C→T) = 1 - e^(-μt), where μ is the damage rate per unit time and t is the sample age in years.

- **Input Formats**: The tool accepts reference sequences in FASTA format for damage simulation. When using the companion binary chronumental-build, input reference genomes must be pre-processed into indexed database files (.cmm files) for efficient lookup during simulation runs. VCF files can be used to specify known variant positions that should be protected from damage modification.

- **Temporal Decay**: Damage patterns follow an exponential decay model where damage accumulates log-linearly with time, allowing simulation of samples ranging from hundreds to tens of thousands of years old. The --age parameter directly controls the temporal extent, and damage probabilities are scaled accordingly.

- **Output I/O**: Simulated damage is written to standard output in FASTQ format, preserving base qualities and adding damage-specific quality modifiers. The --output flag writes to specified files, and --sam tags can be added to generate SAM-format alignments showing damage positions as NM and MD tags.

## Pitfalls

- **Using modern DNA as input without damage**: Attempting to simulate ancient DNA damage using fresh, undegraded modern sequences as input will produce unrealistic damage patterns because the damage model assumes starting from unmodified cytosines/guanines. Always verify input sequences represent the original (pre-degradation) state.

- **Incorrect age parameters for the biological context**: Setting --age to values far outside the expected range for your sample type (e.g., 100 years for a paleontological sample, or 50,000 years for a forensic case) produces physically impossible damage ratios that invalidate downstream analyses.

- **Overwriting companion database files without backup**: Running chronumental-build on existing .cmm database files will silently overwrite them. The original reference data cannot be recovered unless backed up beforehand, causing data loss for downstream chronumental runs.

- **Confusing damage rates across species boundaries**: Using default damage rates (μ parameters) calibrated for human DNA when simulating damage in bacterial, plant, or other non-human genomes produces inaccurate damage patterns because deamination rates vary significantly across organisms due to DNA packaging and repair mechanisms differences.

## Examples

### Simulate ancient DNA damage on a single reference sequence with default parameters

**Args:** --reference ancient_genome.fa --age 10000

**Explanation:** This command reads the FASTA reference, applies 10,000 years of accumulated C→T and G→A damage using the default damage rate constant, and outputs the damaged sequences in FASTQ format to stdout.

### Build a compressed reference database for faster subsequent runs

**Args:** chronumental-build --reference human_grch38.fa --output humandb.cmm

**Explanation:** This companion binary preprocesses and indexes the human reference genome into a compressed .cmm database file, enabling much faster damage simulation in later runs when the --database flag is used instead of --reference.

### Simulate damage with a custom damage rate for non-human specimens

**Args:** --reference bacterial_seq.fa --age 5000 --damage-rate 0.00015

**Explanation:** This applies a custom damage rate constant (0.00015 per year) appropriate for bacterial DNA, overwriting the default human-calibrated rate. The tool scales damage probabilities accordingly and outputs bacterial-appropriate ancient DNA damage patterns.

### Output damaged sequences to a file instead of stdout

**Args:** --reference neandertal.fa --age 40000 --output damaged_output.fq

**Explanation:** Instead of streaming to stdout, this writes all damaged FASTQ sequences to the specified output file, which can then be used directly as input for downstream ancient DNA analysis pipelines.

### Generate damage patterns with VCF-protected variant sites

**Args:** --reference ancient.fa --age 7500 --variants known_snps.vcf --protect

**Explanation:** This imports a VCF file of known variant positions and applies the --protect flag to prevent simulated damage from occurring at those sites, preserving the original alleles for benchmarking analysis against real ancient DNA datasets.