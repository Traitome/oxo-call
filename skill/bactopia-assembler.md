---
name: bactopia-assembler
category: assembly
description: A bacterial genome assembly pipeline that assembles short-read sequencing data into contiguous sequences using configurable assemblers (SPAdes, Shovill, or Velvet). Handles paired-end Illumina data and produces assembly statistics for quality assessment.
tags: [bacterial-genomics, genome-assembly, short-reads, spades, shovill, bioinformatics]
author: AI-generated
source_url: https://bactopia.github.io/
---

## Concepts

- **Input Requirements**: bactopia-assembler processes paired-end Illumina FASTQ files (R1 and R2 reads). Reads should be quality-trimmed before assembly for optimal results; raw reads may contain adapters and low-quality bases that degrade assembly quality.
- **Assembler Selection**: The pipeline supports multiple assemblers (SPAdes, Shovill, Velvet) controlled via the `--assembler` parameter. SPAdes is the default and generally performs well for bacterial genomes, while Shovill uses a subset-based approach that can be faster for large datasets.
- **Output Artifacts**: The tool produces assembly contigs (`contigs.fa`), a GFA file (`assembly.gfa`), and an assembly summary JSON file containing metrics like N50, genome coverage, and contig count. These files are essential for downstream analysis and quality control.
- **Coverage Calculation**: Assembly coverage is automatically calculated from read depth. High coverage (>50x) typically improves assembly quality but may require increased memory; the pipeline estimates coverage from input read files.
- **Memory Management**: The `--memory` parameter controls RAM allocation for the underlying assembler. Insufficient memory causes assembly failures, especially for high-coverage datasets or large genomes.

## Pitfalls

- **Using Untrimmed Reads**: Feeding raw, untrimmed reads to the assembler introduces adapters and low-quality bases into the assembly, resulting in fragmented contigs and inflated genome sizes due to false connections between unrelated sequences.
- **Incorrect Genome Size Estimate**: Providing an inaccurate `--genome-size` value causes SPAdes to use inappropriate k-mer lengths and coverage thresholds, leading to either over-fragmented assemblies or chimeric contigs.
- **Insufficient Memory for High-Coverage Data**: Running with default memory settings on datasets with >100x coverage often causes the assembler to crash or terminate prematurely, wasting compute resources and requiring restart with adjusted parameters.
- **Mismatched Read Files**: Submitting reads from different samples or incorrectly paired R1/R2 files produces nonsensical assemblies with broken contigs and anomalously low coverage statistics.
- **Ignoring Assembly QC Metrics**: Proceeding to downstream analysis without reviewing N50, contig count, and coverage metrics risks using poor-quality assemblies, which can skew variant calling, annotation, and phylogenetic analyses.

## Examples

### Assemble bacterial genome from paired-end reads using default settings
**Args:** `--fastqs sample_R1.fastq.gz --sample my bacterium --assembler spades`
**Explanation:** This runs SPAdes (the default assembler) on paired-end reads, producing assembly outputs with auto-detected parameters suitable for typical bacterial genomes.

### Use Shovill assembler for faster assembly of a well-characterized species
**Args:** `--fastqs R1.fq.gz --sample strain_abc --assembler shovill --genome-size 4.5m`
**Explanation:** Shovill uses a subset-based approach that can be faster than SPAdes; specifying genome size helps optimize k-mer selection and reduces compute time.

### Assemble with custom k-mer lengths and increased memory
**Args:** `--fastqs reads_R1.fastq.gz --sample custom_asm --assembler spades --kmers 21,33,55 --memory 64`
**Explanation:** Custom k-mer sizes allow tuning for specific genomes, and allocating 64GB RAM prevents out-of-memory errors on high-coverage datasets.

### Run assembly with intermediate read QC using fastp
**Args:** `--fastqs sample1_R1.fastq.gz --sample qc_test --assembler spades --skip-qc false --fastp-args "--cut_front --cut_tail"`
**Explanation:** Enabling built-in QC with custom fastp arguments removes low-quality bases from read ends before assembly, improving contig continuity for lower-quality inputs.

### Generate assembly with verbose logging for debugging
**Args:** `--fastqs R1.fq --sample debug_run --assembler spades --verbose`
**Explanation:** Verbose output captures detailed assembler logs, helpful when troubleshooting failed assemblies or investigating unexpected contig structures.