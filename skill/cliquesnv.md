---
name: cliquesnv
category: Variant Analysis
description: A tool for analyzing viral quasispecies from next-generation sequencing data, performing haplotype reconstruction, variant calling, and population structure analysis.
tags: viral, quasispecies, haplotype, assembly, variant-calling, NGS, bioinformatics
author: AI-generated
source_url: https://github.com/cobilab/cliquesnv
---

## Concepts

- cliquesnv works with FASTQ/FASTA input files containing NGS reads from viral samples (e.g., HIV, HCV, influenza) and performs local assembly to reconstruct viral haplotypes present in the sample population.
- The tool supports a reference-based mode where you provide a reference genome (via `.ref` file or FASTA) and a reads file; it aligns reads, identifies variants, and outputs haplotype sequences in FASTA format plus variant calls in tabular format.
- Output formats include: FASTA files with reconstructed haplotype sequences, CSV/TSV variant call tables with position, allele, frequency and coverage metrics, and optional population structure summary statistics.
- Companion binaries cliquesnv-build and cliquesnv-ref are used to prepare reference indexes rather than being part of the main analysis pipeline; the main cliquesnv command handles assembly and variant calling directly.
- Key parameters include `-min-cov` for minimum read coverage threshold, `-min-freq` for minimum allele frequency to call a variant, and `-min-reads` for minimum supporting reads per allele.

## Pitfalls

- Using a reference genome with poor annotations or wrong strain background causes misalignment and produces false variants; always verify your reference matches the viral subtype in your sample.
- Setting `-min-freq` too low (e.g., below 0.01) introduces sequencing error as true variants, while setting it too high (e.g., above 0.10) misses low-frequency quasispecies that may be clinically relevant.
- Feeding low-quality reads without filtering adaptor contamination or host reads (e.g., human ribosomal RNA) leads to spurious haplotypes and inflated variant counts; preprocess with tools like Trimmomatic or bowtie2 before running cliquesnv.
- Forgetting to specify output directory (`-out-dir`) causes results to overwrite previous runs or be written to the current working directory, creating confusion in batch analyses.
- Mixing read files from different samples in a single run without proper tagging produces confounded haplotypes that represent chimeras rather than true quasispecies; run separately or use sample-specific flags.

## Examples

### Assemble viral quasispecies from Illumina reads against a reference genome

**Args:** `-ref reference.fasta -reads sample_reads.fastq -out-dir output/`
**Explanation:** Aligns Illumina FASTQ reads to the provided reference genome and reconstructs quasispecies haplotypes, writing FASTA sequences to the specified output directory.

### Call variants with a minimum allele frequency threshold of 5%

**Args:** `-ref reference.fasta -reads sample_reads.fastq -min-freq 0.05 -out-dir output/`
**Explanation:** Performs variant calling while only reporting alleles present at at least 5% frequency in the viral population, reducing noise from sequencing errors.

### Generate variant table with minimum coverage of 10 reads per position

**Args:** `-ref reference.fasta -reads sample_reads.fastq -min-cov 10 -out-file variants.tsv`
**Explanation:** Outputs a variant call table where each reported variant is supported by at least 10 reads at that genomic position, ensuring statistical reliability.

### Run haploassembly without a reference genome using de novo mode

**Args:** `-reads sample_reads.fastq -denovo -out-dir denovo_output/`
**Explanation:** Performs de novo haplotype reconstruction without a reference, useful for discovering novel viral variants or when no suitable reference exists.

### Filter low-quality bases before assembly using quality threshold

**Args:** `-ref reference.fasta -reads sample_reads.fastq -q-filter 20 -out-dir filtered_output/`
**Explanation:** Trims or masks bases with Phred quality scores below 20 before assembly, improving haplotype accuracy in high-error-rate sequencing runs.

### Specify maximum number of haplotypes to reconstruct

**Args:** `-ref reference.fasta -reads sample_reads.fastq -max-haplotypes 20 -out-dir output/`
**Explanation:** Limits reconstruction to the top 20 most abundant haplotypes by frequency, useful for focusing on major quasispecies in highly diverse populations.

### Output population structure statistics for the quasispecies

**Args:** `-ref reference.fasta -reads sample_reads.fastq -stats -out-dir output/`
**Explanation:** Generates summary statistics including haplotype diversity, entropy, and frequency distribution across the viral population.