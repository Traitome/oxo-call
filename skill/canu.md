---
name: canu
category: Genome Assembly
description: A long-read genome assembler using OBT (Overlap-Based Trim) and OVL (Overlap) algorithms for assembling sequences from PacBio and Oxford Nanopore platforms. Operates in three sequential phases: correction, trim, and assemble.
tags:
  - long-read-assembly
  - pacbio
  - oxford-nanopore
  - genome-assembly
  - canu
  - haplotig-purging
author: AI-generated
source_url: https://github.com/marbl/canu
---

## Concepts

- Canu operates in three distinct sequential phases — **correction** (improves read accuracy by computing a consensus), **trimming** (removes suspect starts/ends from reads using OBT), and **assembly** (builds the final contigs using OVL overlaps). Each phase writes intermediate files, so stopping and resuming is safe if a phase is interrupted.
- Input reads must be in **FASTQ or FASTA format** (gzip or BAM accepted) and are specified via `-p` (pacbio) or `-n` (nanopore) options. All input options can be repeated multiple times to combine multiple input files into a single assembly.
- Output directory is controlled by `-d` and the assembly name by `-s`. The assembler automatically sizes grid resources (gridEngine, SLURM, PBS/Torque, or local mode) using the number of reads and genome size; forcing mismatched resource sizes via `-t` or `-M` can cause out-of-memory failures or extreme slowdowns.
- The **corrected error rate** (`corErrorRate`) and **assembled error rate** (`asmErrorRate`) parameters control sensitivity. The default `asmErrorRate=0.045` (4.5%) is tuned for long reads after correction; using raw uncorrected reads requires a higher rate (e.g., `asmErrorRate=0.16` for Nanopore 1D reads).
- **Haplotig purging** (`purmitig=1`) is available to reduce allele duplication in diploids or polyploids, but it is aggressive for haploids and will collapse genuine repeats, resulting in shorter contigs.

## Pitfalls

- Setting `-t` (threads) or `-m` (memory) too low causes excessive **swap-to-disk**, making assembly orders of magnitude slower or killing the process outright when memory is exhausted.
- Running without specifying a **genome size** (`genomeSize`) causes Canu to estimate it from input reads, which is inaccurate for repetitive or large genomes and leads to poor resource allocation and sub-optimal overlap settings.
- Using the default `corErrorRate` on **already corrected/Polished reads** (e.g., fromArrow or Medaka) removes legitimate variation, collapsing haplotypes and creating chimeric contigs; use `corErrorRate=0.001` for pre-corrected inputs.
- Specifying **both** `-pacbio` and `-nanopore` in a single run treats all reads together, which is incorrect for hybrid assemblies when platform-specific error profiles differ significantly; run separate assemblies per platform and merge with other tools instead.
- Skipping the correction phase with `stopAfter=trim` or `stopAfter=assemble` on **raw reads** (without correction) produces fragmented assemblies with hundreds of contigs because overlapping low-quality regions are missed.

## Examples

### Assemble a bacterial genome from a single Nanopore FASTQ file
**Args:** `-nanopore reads.fastq.gz -d canu_out -s ecoli_assembly genomeSize=5.0m`
**Explanation:** Canu will auto-detect grid engine (local mode by default), correct reads, trim, and assemble them into contigs, writing output to `canu_out/`.

### Assemble a PacBio bacterial genome with explicit thread and memory limits
**Args:** `-pacbio reads.fq -d canu_out -s bacteria genomeSize=4.5m -t 16 -m 64G`
**Explanation:** Limits the assembly to 16 threads and 64 GB RAM per job, which prevents resource over-scheduling on shared cluster nodes while maintaining high overlap sensitivity.

### Resume a failed Canu assembly at the assemble phase
**Args:** `-nanopore reads.fastq.gz -d canu_out -s resume_assembly genomeSize=5.5m stopAfter=assemble -t 24`
**Explanation:** Skips the correction and trimming phases (already completed in previous runs) by setting `stopAfter=assemble`, saving hours of recomputation while using 24 threads for overlap layout.

### Assemble a diploid genome with haplotig purging enabled
**Args:** `-pacbio reads.fq -d diploid_out -s polygenome genomeSize=120m purgeHaplotigs=1 redMemory=8G oeaMemory=8G -t 32`
**Explanation:** Enables haplotig purging to remove duplicated allele sequences, reducing heterozygosity-driven contig duplication, but increases runtime due to extra alignment steps.

### Assemble a large eukaryotic genome using SLURM grid engine
**Args:** `-nanopore reads.fastq.gz -d canu_out -s eukaryote genomeSize=2.5g useGrid=1 -t 64 -tm 2000`
**Explanation:** Enables SLURM submission via `useGrid=1` and sets a per-node memory limit of 2000 GB, allowing Canu to distribute overlap and consensus jobs across a cluster efficiently.

### Assemble with a custom corrected error rate for pre-polished reads
**Args:** `-nanopore polished_reads.fq -d canu_out -s polished_assembly genomeSize=6.5m corErrorRate=0.001 asmErrorRate=0.01`
**Explanation:** Overrides the default 4.5% assembled error rate with a stricter 1% rate because the reads have already been error-corrected by another tool, preventing haplotype collapse while maintaining overlaps.