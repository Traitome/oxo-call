---
name: canu
category: assembly
description: High-quality de novo assembler for long reads (ONT and PacBio) with built-in error correction
tags: [assembly, long-read, nanopore, pacbio, de-novo, hifi, error-correction, triocanu]
author: oxo-call built-in
source_url: "https://canu.readthedocs.io/"
---

## Concepts

- Canu performs error correction, trimming, and assembly for long reads; handles PacBio CLR, ONT, and HiFi.
- Key parameters: -p (output prefix), -d (output directory), genomeSize (estimated genome size with g/m/k suffix).
- Use -pacbio-raw for PacBio CLR reads; -nanopore-raw for ONT raw reads; -pacbio-hifi for CCS/HiFi reads.
- Canu runs three stages by default: correction, trimming, assembly — use -correct, -trim, or -assemble to run individual stages.
- For HiFi reads, Canu skips error correction (already accurate enough): canu -pacbio-hifi reads.fastq.gz.
- Output: <prefix>.contigs.fasta, <prefix>.unassembled.fasta, <prefix>.report, and correction/trimming/assembly subdirectories.
- Canu uses SLURM/PBS (useGrid=true) or runs locally (useGrid=false); for local use, specify maxMemory and maxThreads.
- rawErrorRate sets overlap tolerance for raw reads (default: 0.300 PacBio, 0.500 ONT); correctedErrorRate for corrected reads (default: 0.045 PacBio, 0.144 ONT).
- minReadLength filters short reads (default 1000); minOverlapLength filters short overlaps (default 500).
- TrioCanu mode (-haplotype{NAME}) separates parental haplotypes using Illumina short reads for diploid assembly.
- Canu is slower than Flye but may produce more contiguous assemblies for some datasets.

## Pitfalls

- genomeSize is required — use k, m, g suffixes (e.g., 5m for 5 Mb, 3g for 3 Gb). Fractional values allowed: 4.7m.
- Canu requires significant RAM for large genomes — human genome needs ~256 GB RAM.
- Without maxMemory and maxThreads, Canu may try to use all available resources on shared systems.
- For HiFi reads, use -pacbio-hifi not -pacbio-raw — correction stage is unnecessary for HiFi data and wastes time.
- Canu can be slow for large datasets — Flye is generally faster with comparable quality.
- The output directory (-d) should not already exist with incomplete Canu runs — use a new directory.
- -pacbio is legacy; prefer -pacbio-raw for CLR or -pacbio-hifi for CCS data.
- For low-coverage data, increase correctedErrorRate (e.g., correctedErrorRate=0.15) to find more overlaps.
- stopOnLowCoverage can halt assembly if coverage drops below threshold — check coverage before assembly.

## Examples

### assemble bacterial genome from ONT reads
**Args:** `-p ecoli_assembly -d canu_ecoli/ genomeSize=5m -nanopore-raw reads.fastq.gz maxMemory=16g maxThreads=8`
**Explanation:** -p prefix; -d output dir; genomeSize required; -nanopore-raw for raw ONT; maxMemory/maxThreads for local run

### assemble genome from PacBio HiFi reads
**Args:** `-p hifi_assembly -d canu_hifi/ genomeSize=3g -pacbio-hifi hifi_reads.fastq.gz maxMemory=64g maxThreads=32`
**Explanation:** -pacbio-hifi skips error correction; genomeSize=3g for human-sized genome

### assemble metagenome from ONT reads
**Args:** `-p metagenome -d canu_meta/ genomeSize=100m -nanopore-raw meta_reads.fastq.gz maxMemory=128g maxThreads=32 useGrid=false`
**Explanation:** useGrid=false for local run; genomeSize estimate for metagenome; may need higher memory

### run only the assembly stage (skip correction and trimming)
**Args:** `-p assembly_only -d canu_assembly_only/ -assemble genomeSize=5m -nanopore-corrected corrected_reads.fasta maxMemory=16g maxThreads=8`
**Explanation:** -assemble runs only assembly stage; -nanopore-corrected for pre-corrected reads

### run TrioCanu for diploid assembly with parental short reads
**Args:** `-p trio_assembly -d canu_trio/ genomeSize=3g -haplotypeMAT dad/*.fastq.gz -haplotypePAT mom/*.fastq.gz -nanopore-raw offspring.fastq.gz maxMemory=256g maxThreads=64`
**Explanation:** -haplotypeMAT/PAT separates parental haplotypes using Illumina reads; produces haplotype-specific assemblies

### assemble PacBio CLR reads with adjusted error rates
**Args:** `-p clr_assembly -d canu_clr/ genomeSize=500m -pacbio-raw reads.fastq.gz rawErrorRate=0.350 correctedErrorRate=0.05 maxMemory=64g maxThreads=32`
**Explanation:** rawErrorRate=0.350 for lower-quality CLR data; correctedErrorRate=0.05 for stringent corrected overlaps

### run only correction stage for later use with different assembler
**Args:** `-p corrected_only -d canu_correct/ -correct genomeSize=5m -nanopore-raw reads.fastq.gz maxMemory=16g maxThreads=8`
**Explanation:** -correct runs only correction stage; output corrected reads can be used with Flye or other assemblers

### assemble with coverage limiting for high-depth data
**Args:** `-p assembly -d canu_cov/ genomeSize=5m -nanopore-raw reads.fastq.gz maxInputCoverage=100 corOutCoverage=100 maxMemory=16g maxThreads=8`
**Explanation:** maxInputCoverage limits coverage used; corOutCoverage limits corrected read output; saves time/memory for high-coverage data
