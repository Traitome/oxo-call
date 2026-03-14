---
name: flye
category: assembly
description: Long-read de novo assembler for Oxford Nanopore and PacBio sequencing data
tags: [assembly, long-read, nanopore, pacbio, de-novo, metagenome, hifi]
author: oxo-call built-in
source_url: "https://github.com/fenderglass/Flye"
---

## Concepts

- Flye assembles long reads (ONT, PacBio CLR, PacBio HiFi) into complete or near-complete genome assemblies.
- Use --nano-raw for ONT raw reads, --nano-hq for ONT high-quality, --pacbio-raw for PacBio CLR, --pacbio-hifi for HiFi.
- Specify estimated genome size with --genome-size (e.g., 5m for 5 Mb, 3g for 3 Gb) — required unless --meta is used.
- Use -o for output directory; --threads N for parallelism.
- Flye output: assembly.fasta (main assembly), assembly_graph.gfa, assembly_info.txt (contig statistics).
- Use --meta for metagenomic assembly with long reads.
- Use --iterations N (default: 1) for additional polishing rounds; more iterations improve accuracy but take longer.
- Flye incorporates Medaka-like internal polishing — for ONT data, further polishing with Medaka is recommended.

## Pitfalls

- --genome-size is required for isolate assemblies — omitting it causes an error (except with --meta).
- Do not use --nano-raw for R10 or high-accuracy ONT reads — use --nano-hq for better results.
- Flye is memory-intensive for large genomes; human genome assembly may require 40-80 GB RAM.
- The output directory must not already exist (or use --resume to continue).
- For very high coverage (>200x), subsample reads before assembly to speed up and improve quality.
- Flye assembly quality improves with polishing — always run Medaka or Racon after Flye for ONT data.

## Examples

### assemble bacterial genome from Oxford Nanopore reads
**Args:** `--nano-raw reads.fastq.gz --genome-size 5m --out-dir flye_output/ --threads 16`
**Explanation:** --nano-raw for ONT reads; --genome-size 5m (5 Mb); --out-dir output directory

### assemble genome from PacBio HiFi reads
**Args:** `--pacbio-hifi hifi_reads.fastq.gz --genome-size 3g --out-dir hifi_assembly/ --threads 32`
**Explanation:** --pacbio-hifi for CCS/HiFi reads; --genome-size 3g for 3 Gb human-sized genome

### assemble metagenomic community from ONT reads
**Args:** `--meta --nano-raw meta_reads.fastq.gz --out-dir meta_flye/ --threads 32`
**Explanation:** --meta for metagenomic mode; no --genome-size required; handles uneven coverage

### assemble with high-quality ONT reads (R10, Q20+)
**Args:** `--nano-hq hq_reads.fastq.gz --genome-size 4.5m --out-dir hq_assembly/ --threads 16 --iterations 2`
**Explanation:** --nano-hq for high-accuracy ONT reads; --iterations 2 for additional polishing

### resume an interrupted Flye assembly
**Args:** `--nano-raw reads.fastq.gz --genome-size 5m --out-dir flye_output/ --threads 16 --resume`
**Explanation:** --resume continues from the last successfully completed stage
