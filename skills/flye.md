---
name: flye
category: assembly
description: Long-read de novo assembler for Oxford Nanopore and PacBio sequencing data
tags: [assembly, long-read, nanopore, pacbio, de-novo, metagenome, hifi, repeat-graph]
author: oxo-call built-in
source_url: "https://github.com/mikolmogorov/Flye"
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
- --asm-coverage N uses only Nx longest reads for initial disjointig assembly; reduces memory for large genomes (typically 40x sufficient).
- --scaffold enables graph-based scaffolding to improve contiguity (disabled by default since v2.9).
- --keep-haplotypes preserves heterozygous variants instead of collapsing; --no-alt-contigs excludes alternative haplotype contigs.
- --read-error adjusts parameters for specific error rates (e.g., 0.03 for 3%); useful for R9 Guppy5+ with --nano-hq.
- --min-overlap sets minimum read overlap; auto-selected by default.
- --resume-from and --stop-after enable fine-grained pipeline control for debugging or partial runs.
- --deterministic runs disjointig assembly single-threaded for reproducible results.

## Pitfalls

- CRITICAL: Flye has NO subcommands. ARGS starts directly with flags (e.g., --nano-raw, --pacbio-hifi, --genome-size, --out-dir). Do NOT put a subcommand like 'assemble' or 'run' before flags.
- --genome-size is required for isolate assemblies — omitting it causes an error (except with --meta).
- Do not use --nano-raw for R10 or high-accuracy ONT reads — use --nano-hq for better results.
- Flye is memory-intensive for large genomes; human genome assembly may require 40-80 GB RAM.
- The output directory must not already exist (or use --resume to continue).
- For very high coverage (>200x), subsample reads before assembly to speed up and improve quality.
- Flye assembly quality improves with polishing — always run Medaka or Racon after Flye for ONT data.
- --asm-coverage requires both --genome-size and read type specification; only affects initial disjointig, all reads used later.
- --scaffold is disabled by default since v2.9; explicitly enable if scaffolding needed.
- --read-error takes fraction (0.03 = 3%), not percentage; use for R9 Guppy5+ with --nano-hq.
- --keep-haplotypes increases assembly size; combine with --no-alt-contigs to simplify output.
- Mixing different read types in one run is not supported; run separately and merge assemblies if needed.

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

### assemble large genome with reduced memory using coverage subset
**Args:** `--pacbio-hifi hifi_reads.fastq.gz --genome-size 3g --out-dir human_assembly/ --threads 32 --asm-coverage 40`
**Explanation:** --asm-coverage 40 uses only 40x longest reads for initial disjointig; reduces memory from 80GB to ~40GB for human genome

### assemble with scaffolding enabled
**Args:** `--nano-hq hq_reads.fastq.gz --genome-size 4.5m --out-dir scaffolded_assembly/ --threads 16 --scaffold --iterations 2`
**Explanation:** --scaffold enables graph-based scaffolding; improves contiguity by linking contigs using repeat graph information

### preserve haplotypes for diploid genome assembly
**Args:** `--pacbio-hifi hifi_reads.fastq.gz --genome-size 600m --out-dir diploid_assembly/ --threads 32 --keep-haplotypes --iterations 2`
**Explanation:** --keep-haplotypes preserves heterozygous variants; outputs both haplotypes instead of collapsed mosaic

### assemble with custom error rate for Guppy5+ R9 data
**Args:** `--nano-hq guppy5_reads.fastq.gz --genome-size 5m --out-dir flye_output/ --threads 16 --read-error 0.05`
**Explanation:** --read-error 0.05 sets 5% error rate; recommended for R9 Guppy5+ data with --nano-hq (slightly higher error than R10)

### run only specific pipeline stages
**Args:** `--nano-raw reads.fastq.gz --genome-size 5m --out-dir flye_output/ --threads 16 --stop-after contigger`
**Explanation:** --stop-after contigger stops after contig generation; useful for debugging or when polishing will be done separately

### polish existing assembly with Flye polisher
**Args:** `--nano-raw reads.fastq.gz --polish-target draft_assembly.fasta --out-dir polished_output/ --threads 16`
**Explanation:** --polish-target runs Flye polisher standalone on existing assembly; useful for re-polishing with different read sets
