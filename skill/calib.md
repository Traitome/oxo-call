---
name: calib
category: Read Simulation / Bioinformatics
description: A long-read simulator for PacBio and Oxford Nanopore sequencing data that generates realistic synthetic reads with calibrated error profiles from a reference genome.
tags: [read-simulation, long-reads, pacbio, nanopore, synthetic-data, error-model, training-data]
author: AI-generated
source_url: https://github.com/pedroheader/calib
---

## Concepts

- **Input Format**: calib takes a reference genome in FASTA format and generates synthetic long reads by sampling substrings along the reference, simulating the error profiles typical of PacBio or Oxford Nanopore technologies.
- **Error Model**: The tool models insertion, deletion, and substitution errors with position-dependent error rates, allowing users to specify error rate parameters (e.g., `--lambda`) to match real-world basecalling accuracy.
- **Output Format**: Simulated reads are emitted in FASTQ format, preserving quality scores that reflect the simulated per-base error probabilities.
- **Read Length Distribution**: Users can control the mean and standard deviation of read lengths using flags like `--num-reads` and `--mean-read-length` to match specific sequencing run characteristics.
- **Technology-Specific Profiles**: Separate error profiles exist for PacBio CLR and Nanopore reads, selectable via the `--technology` flag to ensure biologically accurate simulations.

## Pitfalls

- **Incorrect Reference Indexing**: Using a fragmented or poorly assembled reference genome results in reads that do not represent realistic genomic coverage, leading to biased downstream analysis.
- **Wrong Technology Parameter**: Specifying the wrong technology type produces unrealistic error profiles; for instance, using Nanopore parameters on PacBio data yields incorrect indel-to-substitution ratios.
- **Insufficient Read Count for Training Data**: Generating too few reads produces sparse training datasets that fail to capture full sequence diversity, degrading machine learning model performance in assemblers or basecallers.
- **Memory Overflow with Large References**: Setting `--num-reads` extremely high against a small reference causes excessive read overlap, inflating file sizes and causing downstream tools to crash due to memory exhaustion.
- **Quality Score Saturation**: Using error rates lower than realistic values results in quality scores that exceed Phred+33 range limits, causing parsing errors in tools expecting standard FASTQ encoding.

## Examples

### Generate 1000 simulated PacBio CLR reads from a reference genome

**Args:** `--reference ref.fa --technology pacbio --num-reads 1000 --mean-read-length 5000 --output-simreads reads.fq`

**Explanation:** This samples 1000 reads averaging 5000 bases each from the reference using PacBio CLR error characteristics, outputting to FASTQ for downstream training.

### Simulate Nanopore reads with a specified error rate

**Args:** `--reference ref.fa --technology nanopore --num-reads 5000 --lambda 0.15 --output-simreads nanopore_reads.fq`

**Explanation:** Uses a 15% error rate lambda to generate 5000 Nanopore-style reads, producing realistic high-error long reads for testing basecallers.

### Generate reads with custom length distribution

**Args:** `--reference ref.fa --technology pacbio --num-reads 2000 --mean-read-length 10000 --sd-read-length 3000 --output-simreads long_reads.fq`

**Explanation:** Produces 2000 reads with mean length 10kb and standard deviation 3kb, simulating long-insert fragments typical of PacBio HiFi libraries.

### Create a small test dataset for quick validation

**Args:** `--reference small_ref.fa --technology pacbio --num-reads 100 --mean-read-length 1000 --output-simreads test.fq`

**Explanation:** Generates 100 short reads from a small reference for rapid pipeline testing without excessive file I/O overhead.

### Generate reads with forced quality score range

**Args:** `--reference ref.fa --technology nanopore --num-reads 1500 --mean-read-length 8000 --min-qscore 2 --max-qscore 40 --output-simreads qc_filtered.fq`

**Explanation:** Outputs reads with quality scores bounded between Phred 2 and 40, ensuring compatibility with downstream tools expecting standard ranges.