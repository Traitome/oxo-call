---
name: cnasim
category: Genomics / Simulation
description: A tool for simulating Copy Number Alteration (CNA) data in genomic sequences. Generates synthetic tumor and normal samples with controlled copy number states, purity levels, and subclonal populations for benchmarking CNV detection pipelines.
tags:
  - copy-number-variation
  - cnv
  - simulation
  - genomics
  - benchmarking
  - synthetic-data
  - tumor-analysis
author: AI-generated
source_url: https://github.com/cnasim/cnasim
---

## Concepts

- **Copy Number States**: cnasim models genomic segments with discrete copy number states (0, 1, 2, 3, 4+ copies). Each state represents the total number of chromosome copies in a genomic interval, used to simulate deletions, amplifications, and neutral regions.
- **Input Reference Format**: The tool accepts a BED-style definition file specifying target genomic regions (chromosome, start, end) where simulated CNAs will be inserted. Regions not defined in the input retain the default copy number (typically 2 for diploid genomes).
- **Output Formats**: cnasim generates outputs in multiple formats including BED (copy number per region), VCF (structural variation calls), and FASTA (simulated sequences with inserted alterations). The output format is controlled by the `--out-format` flag.
- **Tumor Purity and Ploidy**: The tool simulates mixed tumor samples with configurable purity (tumor cell fraction 0.0-1.0) and baseline tumor ploidy (average copy number). These parameters affect the observed copy number in the output.
- **Subclonal Simulation**: cnasim supports generating subclones with distinct copy number profiles. Multiple subclonal populations can be defined with relative frequencies, enabling testing of heterogeneous tumor sample analysis.

## Pitfalls

- **Overlapping Regions**: Defining overlapping regions in the input BED file causes undefined behavior—the tool may merge, override, or silently skip overlapping segments. Always ensure non-overlapping genomic intervals in your input definition.
- **InvalidPurity Values**: Setting `--purity` outside the 0.0-1.0 range causes the tool to fail with an error. Values like 1.5 or -0.1 are rejected; specify proper fractions between zero and one.
- **Integer Copy Numbers Only**: The tool requires integer copy number values in the input definition. Fractional or negative copy numbers will be rejected, causing the simulation to abort. Use rounded integers for all copy number states.
- **Genome Build Mismatch**: Failing to match the reference genome build (hg19, hg38, etc.) between your input definition and the `--genome` parameter produces incorrect coordinate mappings. Verify genome build consistency before running simulations.
- **Memory with Large Regions**: Simulating very large numbers of regions (>100,000) or whole-chromosome simulations can exhaust available memory. Process chromosomes in batches or use the `--chunk-size` parameter to limit memory consumption.

## Examples

### Simulate a simple diploid genome with one amplified region
**Args:** `-- genome hg38 --regions amph.txt --out output_cna.bed`
**Explanation:** This runs a basic simulation using hg38 reference, reading region definitions from amph.txt, and writing copy number results to output_cna.bed in BED format.

### Generate a tumor sample with 70% purity
**Args:** `-- genome hg19 --regions tumor_regions.bed --purity 0.7 --ploidy 2.5 --out tumor_sim.vcf`
**Explanation:** Simulates a tumor sample with 70% purity and 2.5 baseline ploidy using hg19 reference, outputting structural variation calls in VCF format.

### Create multi-subclonal tumor population
**Args:** `-- genome hg38 -- subclones.txt --out subclonal_out.bed`
**Explanation:** Generates a heterogeneous tumor with multiple subclones defined in subclones.txt file, each with distinct copy number profiles and frequencies.

### Output simulated sequences in FASTA format
**Args:** `-- genome hg38 --regions example.bed --out-format fasta --out simulated_seqs.fa`
**Explanation:** Produces nucleotide sequences reflecting the simulated copy number alterations, useful for testing read simulators downstream.

### Limit memory usage with chunked processing
**Args:** `-- genome hg38 --regions large_set.bed --chunk-size 10000 --out chunked_output.bed`
**Explanation:** Processes large region sets in chunks of 10,000 to prevent memory exhaustion, writing incremental results to the output file.

### Specify diploid baseline without alterations
**Args:** `-- genome hg38 --out diploid_baseline.bed`
**Explanation:** Runs a baseline simulation with default diploid copy number (2n) across the entire genome, useful as a control for comparison studies.