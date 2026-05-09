---
name: camisim2
category: Read Simulation / Metagenomics
description: A microbial community read simulator that generates synthetic metagenomic sequencing reads from taxonomic profiles and reference genomes, supporting multiple sequencing platforms with configurable error models.
tags:
  - metagenomics
  - read-simulation
  - microbial-community
  - benchmarking
  - synthetic-data
  - taxonomic-profiling
author: AI-generated
source_url: https://github.com/voeneved/camisim2
---

## Concepts

- **Community profile input format**: Camisim2 requires a taxonomic abundance table (typically in YAML or TSV format) where each row defines a taxonomic identifier (species/strain level) paired with its relative abundance. Abundances are normalized internally, so only relative ratios matter; however, the file must contain valid NCBI taxonomy IDs or matching identifiers for the reference database being used.

- **Reference genome sourcing**: The simulator locates reference genomes by matching taxonomy IDs to a pre-built genome index. The `camisim2-build` companion binary constructs this index from a collection of FASTA files organized in a directory structure mirroring NCBI taxonomy. Missing genomes for high-abundance taxa cause simulation to fail or substitute placeholder sequences, directly impacting read accuracy downstream.

- **Sequencing platform error models**: Camisim2 applies platform-specific error profiles during read generation. Illumina uses position-dependent base-calling errors and GC-biased dropout models. PacBio and Oxford Nanopore simulations incorporate indel error rates that scale with homopolymer context and signal complexity. Selecting the wrong model results in unrealistic quality score distributions that will mislead downstream quality-control expectations.

- **Fragment length and read length parameters**: For paired-end Illumina reads, the fragment length distribution is modeled as a normal or uniform distribution specified by mean and standard deviation. Reads are then drawn from both ends of each fragment with a configurable insert size. If the specified read length exceeds the fragment length, read pairs will overlap and may require adapter trimming in downstream processing.

## Pitfalls

- **Mismatched taxonomy IDs in abundance table**: Using taxonomy IDs that do not exist in the reference genome index causes the simulator to skip those entries silently, reducing total read output and skewing the effective community composition. This results in benchmarking datasets that do not reflect the intended diversity, leading to false conclusions about pipeline sensitivity.

- **Incorrect platform parameter for error model**: Specifying `--platform illumina` when simulating Nanopore reads produces quality scores that never drop below Q30, which is biologically implausible. Downstream tools trained on realistic error profiles will misinterpret these artificial high-quality reads as indicating overly clean sequencing.

- **Insufficient disk space for output**: Simulating large communities with deep coverage generates FASTQ files that can exceed hundreds of gigabytes. The tool does not check available disk space before writing; if the filesystem fills during write, partial output files are produced without error reporting, and downstream analysis runs on truncated data.

- **Overlapping paired-end reads without adapter trimming awareness**: Setting read length within 20bp of mean fragment length produces heavily overlapping pairs. If downstream pipelines do not perform adapter trimming (or trimming is disabled), overlap Removal tools may produce chimeric mappings that inflate apparent structural variation metrics.

- **Duplicate reference genomes in index building**: If `camisim2-build` encounters multiple FASTA files with identical taxonomy IDs during indexing, the behavior is undefined and may cause silent file overwrites. Subsequent simulations using those taxa will randomly select among duplicates, introducing nondeterministic read content that prevents reproducible benchmarking.

## Examples

### Simulate Illumina paired-end reads from a taxonomic profile with default parameters
**Args:** `-c community_profile.tsv -i genomes/ --platform illumina -o output_dir/ -n 1000000`
**Explanation:** The `-c` flag specifies the community abundance table, `-i` points to the pre-built genome index, `-n` sets the target number of read pairs to generate, and output FASTQ files are written to `-o`.

### Simulate Oxford Nanopore single-molecule reads with long read lengths
**Args:** `-c community_profile.tsv -i genomes/ --platform ont -o output_dir/ -n 500000 --read-length 10000 --mean-accuracy 0.88`
**Explanation:** Setting `--platform ont` activates the Nanopore error model with indel-prone homopolymer errors, `--read-length 10000` generates long reads typical of R9.4 flow cells, and `--mean-accuracy 0.88` tunes the baseline quality baseline.

### Build a reference genome index from a directory of FASTA files
**Args:** `camisim2-build -d reference_genomes/ -t taxmapping.tsv -o genome_index/`
**Explanation:** The companion binary `camisim2-build` creates the searchable index, `-d` specifies the FASTA input directory, `-t` provides a taxonomy-to-file mapping table, and `-o` designates the index output directory for later simulation runs.

### Simulate paired-end reads with custom fragment length distribution
**Args:** `-c community_profile.tsv -i genomes/ --platform illumina -o output_dir/ -n 2000000 --read-length 150 --fragment-mean 350 --fragment-std 25`
**Explanation:** Overriding default fragment parameters with `--fragment-mean 350` and `--fragment-std 25` produces inserts centered at 350bp with 25bp standard deviation, appropriate for MiSeq v3 chemistry, while `--read-length 150` sets each read to 150bp.

### Simulate reads with GC-biased error model for high-GC content organisms
**Args:** `-c community_profile.tsv -i genomes/ --platform illumina -o output_dir/ -n 500000 --gc-bias --gc-window 500`
**Explanation:** Enabling `--gc-bias` activates GC-dependent coverage dropout, where high-GC regions suffer coverage reduction during simulation, and `--gc-window 500` sets the smoothing window for GC calculations, mimicking real Illumina sequencer artifacts.

### Simulate PacBio HiFi reads with reduced error rate
**Args:** `-c community_profile.tsv -i genomes/ --platform pacbio-hifi -o output_dir/ -n 100000 --read-length 15000 --circular`
**Explanation:** Setting `--platform pacbio-hifi` switches to high-fidelity mode with Q30+ base qualities, `--circular` treats genomes as circular for polymerase read generation, and `--read-length 15000` sets the CCS read length target.