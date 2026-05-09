---
name: ancestry_hmm
category: Population Genomics
description: A hidden Markov model tool for inferring ancestral backgrounds in bacterial genomes. It models the ancestry of each genomic position based on sequence data and population-specific allele frequencies.
tags: ancestry, HMM, population genetics, bacterial genomics, genotype inference, ancestral backgrounds
author: AI-generated
source_url: https://www.sanger.ac.uk/science/tools/ancestry-hmm
---

## Concepts

- **HMM-based ancestry inference**: ancestry_hmm uses a hidden Markov model where each state represents a different ancestral background. The model calculates the probability that each genomic position originates from each reference population based on observed alleles and population-specific allele frequency parameters.
- **Input requirements**: The tool requires three primary input files: (1) a sequence data file in FASTA or simple format containing the query sequences to be analyzed, (2) a positions file specifying the genomic coordinates to analyze, and (3) a parameters file containing population-specific allele frequencies for each position.
- **Output format**: The tool produces ancestry probability estimates for each position, indicating the likelihood that each base originates from each specified ancestral background. Output can be generated as probability matrices or discrete ancestry calls.
- **Multi-population support**: ancestry_hmm can model ancestry from multiple reference populations simultaneously, making it suitable for analyzing bacterial isolates from complex population structures.

## Pitfalls

- **Mismatched position files**: Using a positions file that does not correspond to the exact genomic coordinates in the sequence data will cause the model to analyze incorrect regions, producing meaningless ancestry probabilities.
- **Incorrect population count specification**: Failing to specify the correct number of populations (`-k` flag) that match the parameter file will lead to parameter loading errors or severely incorrect ancestry estimates.
- **Improper parameter file formatting**: The parameters file requires a specific three-column format (position, allele for population 1, allele for population 2). Using incorrect delimiters or column orders will cause the tool to fail or produce silent errors.
- **Memory constraints with large genomes**: Analyzing whole bacterial chromosomes at high resolution can generate large output files and consume significant memory; consider using strategic subsampling of positions for initial exploratory analysis.

## Examples

### Run ancestry inference with all required inputs
**Args:** `-o output_probs.txt -k 2 -p param.txt -s sequences.fa -l positions.txt`
**Explanation:** This runs ancestry_hmm with two reference populations, specifying the parameter file, sequence file, and positions file, outputting probability estimates to the specified output file.

### Generate discrete ancestry calls instead of probabilities
**Args:** `-o ancestry_calls.txt -k 2 -p param.txt -s sequences.fa -l positions.txt -c`
**Explanation:** The `-c` flag outputs discrete ancestry calls (the most likely ancestry for each position) rather than full probability matrices, producing easier-to-interpret categorical assignments.

### Adjust the transition probability for ancestry changes
**Args:** `-o output.txt -k 2 -p param.txt -s sequences.fa -l positions.txt -t 0.001`
**Explanation:** The `-t` flag sets the transition probability (the probability of switching between ancestral backgrounds), controlling how fragmented the ancestry calls appear; lower values produce fewer, longer ancestry segments.

### Run with three ancestral backgrounds
**Args:** `-o tripop_output.txt -k 3 -p tripop_param.txt -s sequences.fa -l positions.txt`
**Explanation:** Using `-k 3` specifies three ancestral populations, requiring a parameter file with allele frequencies for all three populations at each position; the output will contain probabilities for all three backgrounds.

### Generate output in binary format for downstream processing
**Args:** `-o binary_output.bin -k 2 -p param.txt -s sequences.fa -l positions.txt -b`
**Explanation:** The `-b` flag outputs ancestry probabilities in binary format rather than text, producing smaller output files that can be processed more efficiently by custom downstream scripts.