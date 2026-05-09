---
name: bpp-popgen
category: phylogenetic-analysis
description: Bio++ population genetics simulator for generating genetic data samples under various population models including coalescent, Wright-Fisher, and structured populations. Supports simulation of genetic variation with configurable mutation rates, recombination, selection, and demographic scenarios.
tags:
- population-genetics
- simulation
- coalescent
- bioinformatics
- genetic-variation
- evolutionary-models
author: AI-generated
source_url: https://github.com/BioPP/bpp-popgen
---

## Concepts

- **Simulation Models**: bpp-popgen simulates genetic data under several population genetic models including the standard coalescent, Wright-Fisher diffusion, and structured island models. Users define the demographic model and its parameters in an input configuration file.
- **Input/Output Formats**: The tool takes a parameter file (typically in a key=value format) specifying the simulation model, sample sizes, mutation rate, recombination rate, and output preferences. It generates output genetic data in common formats such as FASTA, PHYLIP, Nexus, or binary formats for downstream phylogenetic or population genetic analyses.
- **Key Parameters**: Essential parameters include effective population size (Ne), mutation rate per site per generation (theta), recombination rate, sample size (number of sequences), sequence length, and the output format. These can be specified directly in the parameter file or via command-line overrides.
- **Companion Tools**: The bpp-popgen package includes ancillary programs such as bpp-popgen-check for validating parameter files before running simulations, helping catch configuration errors early in the workflow.

## Pitfalls

- **Incorrect Parameter File Syntax**: Using malformed parameter files with missing required fields (e.g., omitting sample size or mutation rate) causes the simulation to fail silently or produce an error. Always validate parameter files with bpp-popgen-check or a similar validation tool before running large simulations.
- **Confusing Sequence Length Units**: Specifying sequence length in base pairs versus number of sites can lead to unexpectedly small or large output datasets. Verify the expected output size against your intended analysis before running simulations that produce large datasets.
- **Insufficient Effective Population Size**: Setting Ne too low during simulation can introduce excessive genetic drift, causing unrealistically high divergence in the simulated sample. Ensure Ne reflects the biological context of your study organism.

## Examples

### Simulate a basic coalescent sample with default parameters
**Args:** --param param.txt
**Explanation:** Runs a coalescent simulation using parameters defined in param.txt, which should include population size, mutation rate, sample size, and sequence length.

### Override the sample size specified in the parameter file
**Args:** --param param.txt nsam=50
**Explanation:** Runs the simulation with 50 sequences instead of the value originally set in the parameter file, allowing quick exploration of different sample sizes without editing the file.

### Simulate with a specific mutation rate theta
**Args:** --param param.txt theta=0.01
**Explanation:** Overrides the mutation rate parameter to 0.01 (per site per generation), enabling simulation of genetic variation under a different mutation intensity than originally configured.

### Output simulated sequences in FASTA format
**Args:** --param param.txt output=simulated.fasta output_format=Fasta
**Explanation:** Writes the simulated genetic alignment to simulated.fasta in FASTA format, compatible with most downstream sequence analysis tools and viewers.

### Simulate with a predefined random seed for reproducibility
**Args:** --param param.txt seed=12345
**Explanation:** Sets the random number generator seed to 12345, ensuring the exact same simulated dataset is produced when the same parameters and seed are used again.

### Run simulation and redirect standard error to a log file
**Args:** --param param.txt 2> simulation_errors.log
**Explanation:** Captures error messages and warnings from the simulation to simulation_errors.log for later review, useful when debugging simulation failures or unexpected results.