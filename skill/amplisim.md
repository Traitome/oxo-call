---
name: amplisim
category: Amplicon Sequencing Simulation
description: A tool for simulating amplicon sequences from reference genomes using primer definitions and target regions. Generates synthetic amplicon datasets for benchmarking and testing amplicon-based bioinformatics pipelines.
tags:
  - amplicon-simulation
  - in-silico-pcr
  - synthetic-data
  - metagenomics
  - target-sequencing
  - bioinformatics
  - sequencing-simulation
author: AI-generated
source_url: https://github.com/oxo-software/amplisim
---

## Concepts

- Amplisim takes a reference genome (FASTA) and primer definitions to in-silico amplify target regions, producing realistic amplicon sequences with configurable error profiles that mimic actual sequencing data.
- The tool requires three core inputs: a reference genome file, a primer file (TSV/CSV with forward/reverse primer sequences), and a target loci definition file specifying chromosomal coordinates for amplification.
- Output is generated in standard bioinformatics formats (FASTA for sequences, FASTQ when quality simulation is enabled), with per-base quality scores derived from configurable Phred score models.
- Amplisim supports three simulation modes: singleplex (one primer pair), multiplex (multiple primer pairs), and variable-coverage (non-uniform read depth across targets).
- The companion binary `amplisim-build` creates indexed reference databases for faster subsequent simulations, and `amplisim-stats` generates summary reports of simulated datasets.

## Pitfalls

- Specifying primers with ambiguous bases (e.g., "R" for A/G) without enabling the ambiguity resolution flag causes incomplete matches and missing amplicons, resulting in gaps in downstream analysis datasets.
- Using a reference genome that does not match the coordinate system in the target loci file produces empty output files because chromosomal coordinates cannot be mapped correctly, wasting computation time.
- Setting identical forward and reverse primer sequences for paired-end simulation when the target region is smaller than the primer length causes self-complementarity issues, generating malformed or chimeric amplicon sequences.
- Forgetting to enable the reverse-complement flag when primer sequences are provided in the 5'-to-3' orientation on only one strand results in only forward-strand amplification, missing approximately half of the expected amplicons.
- Specifying extremely high coverage values without adjusting the maximum memory allocation causes out-of-memory errors on systems with limited RAM, terminating the simulation before completion.

## Examples

### Simulate amplicons from a bacterial genome with a single primer pair
**Args:** `reference.fasta --primers primers.tsv --loci regions.bed --output simulated_amplicons.fasta`
**Explanation:** This runs a basic singleplex simulation using the reference genome, primer definitions, and target chromosomal coordinates to generate a FASTA file containing all matching amplicon sequences.

### Build an indexed reference database for faster repeated simulations
**Args:** `amplisim-build reference.fasta --index-dir ./amplisim_db --threads 4`
**Explanation:** The companion binary `amplisim-build` creates a compressed index of the reference genome, enabling subsequent simulation runs to skip the expensive genome parsing step.

### Simulate multiplex amplicons with variable coverage depth
**Args:** `reference.fasta --primers multiplex_primers.tsv --loci targets.bed --coverage-profile coverage.tsv --output multiplex_output.fasta --format fastq`
**Explanation:** This multiplex mode uses a coverage profile to assign non-uniform read depths to different primer pairs, generating FASTQ output with quality scores that reflect realistic heterogeneous sequencing runs.

### Generate simulated amplicons with PacBio-style error model
**Args:** `reference.fasta --primers primer_pairs.tsv --loci genomic_targets.bed --error-model pacbio --coverage 50 --output pacbio_amplicons.fastq`
**Explanation:** The pacbio error model introduces insertions, deletions, and substitutions at rates appropriate for PacBio HiFi reads, producing synthetic amplicon data suitable for testing long-read bioinformatics pipelines.

### Simulate paired-end amplicons with quality score calibration
**Args:** `reference.fasta --primers primers.csv --loci amplicon_coords.bed --paired-end --read-length 250 --qual-offset 33 --output paired_amplicons.fastq`
**Explanation:** This configuration generates paired-end reads with specified length and standard Illumina quality score offset (Sanger/Phred+33 encoding), enabling integration with short-read pipelines expecting standard quality encodings.

### Generate a summary statistics report of simulated data
**Args:** `amplisim-stats simulated_amplicons.fasta --report summary_report.txt --metrics read_count,length_dist,gc_content`
**Explanation:** The companion binary `amplisim-stats` analyzes the simulated dataset to produce a summary report including read counts, length distributions, and GC content statistics for pipeline validation.

### Simulate amplicons with ambiguity resolution enabled
**Args:** `reference.fasta --primers ambiguous_primers.tsv --loci target_regions.bed --resolve-ambiguities --output resolved_amplicons.fasta`
**Explanation:** Enabling ambiguity resolution allows IUPAC degenerate base codes in primer sequences to match multiple possibilities, ensuring complete amplicon coverage when working with primers containing mixed-base positions.

### Simulate metagenomic amplicons from multiple reference genomes
**Args:** `genome1.fasta genome2.fasta genome3.fasta --primers metagenomic_primers.tsv --loci v34_regions.bed --output mock_metagenome.fastq --proportional-abundance species_abundance.tsv`
**Explanation:** This mode processes multiple reference genomes simultaneously, generating synthetic 16S/18S amplicons with species abundance proportional to the specified abundance file, suitable for testing metagenomic profiling tools.