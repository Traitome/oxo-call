---
name: canu
category: assembly
description: High-quality de novo assembler for long reads (ONT and PacBio) with built-in error correction
tags: [assembly, long-read, nanopore, pacbio, de-novo, hifi, error-correction]
author: oxo-call built-in
source_url: "https://canu.readthedocs.io/"
---

## Concepts

- Canu performs error correction, trimming, and assembly for long reads; handles PacBio CLR, ONT, and HiFi.
- Key parameters: -p (output prefix), -d (output directory), genomeSize (estimated genome size).
- Use -pacbio for PacBio CLR reads; -nanopore for ONT raw reads; -pacbio-hifi for CCS/HiFi reads.
- Canu runs three stages: correction, trimming, assembly — all three by default.
- For HiFi reads, Canu skips error correction (already accurate enough): canu -pacbio-hifi reads.fastq.gz.
- Output: <prefix>.contigs.fasta, <prefix>.unassembled.fasta, <prefix>.report.
- Canu uses SLURM/PBS or runs locally; for local use, specify maxMemory and maxThreads.
- Canu is slower than Flye but may produce more contiguous assemblies for some datasets.

## Pitfalls

- genomeSize is required — use k, m, g suffixes (e.g., 5m for 5 Mb, 3g for 3 Gb).
- Canu requires significant RAM for large genomes — human genome needs ~256 GB RAM.
- Without maxMemory and maxThreads, Canu may try to use all available resources on shared systems.
- For HiFi reads, use -pacbio-hifi not -pacbio — correction stage is unnecessary for HiFi data.
- Canu can be slow for large datasets — Flye is generally faster with comparable quality.
- The output directory (-d) should not already exist with incomplete Canu runs — use -d new_directory.

## Examples

### assemble bacterial genome from ONT reads
**Args:** `-p ecoli_assembly -d canu_ecoli/ genomeSize=5m -nanopore reads.fastq.gz maxMemory=16g maxThreads=8`
**Explanation:** -p prefix; -d output dir; genomeSize required; -nanopore for ONT; maxMemory/maxThreads for local

### assemble genome from PacBio HiFi reads
**Args:** `-p hifi_assembly -d canu_hifi/ genomeSize=3g -pacbio-hifi hifi_reads.fastq.gz maxMemory=64g maxThreads=32`
**Explanation:** -pacbio-hifi skips error correction; genomeSize=3g for human-sized genome

### assemble metagenome from ONT reads
**Args:** `-p metagenome -d canu_meta/ genomeSize=100m -nanopore meta_reads.fastq.gz maxMemory=128g maxThreads=32 useGrid=false`
**Explanation:** useGrid=false for local run; genomeSize estimate for metagenome; may need higher memory

### run only the assembly stage (skip correction and trimming)
**Args:** `-p assembly_only -d canu_assembly_only/ -assemble genomeSize=5m -nanopore-corrected corrected_reads.fasta maxMemory=16g maxThreads=8`
**Explanation:** -assemble runs only assembly stage; -nanopore-corrected for pre-corrected reads
