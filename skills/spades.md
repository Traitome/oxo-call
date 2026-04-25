---
name: spades
category: assembly
description: De novo genome assembly tool for small genomes, single-cell data, metagenomes, and plasmids
tags: [assembly, de-novo, genome, metagenome, plasmid, single-cell, ngs, illumina]
author: oxo-call built-in
source_url: "https://github.com/ablab/spades"
---

## Concepts
- SPAdes assembles genomes from short reads using de Bruijn graphs; key modes: genomic, metagenomic (--meta), plasmid (--plasmid).
- Use -1 and -2 for paired-end reads; -s for single-end reads; --pe-12 for interleaved paired-end.
- SPAdes automatically selects k-mer values; use -k to specify custom k-mers (e.g., -k 21,33,55,77).
- Use -o for output directory; --threads N for parallelism; --memory N (GB) to limit RAM usage.
- Output files: scaffolds.fasta (final assembly), contigs.fasta, assembly_graph.fastg.
- For metagenomes, use --meta flag; for single-cell (MDA), use --sc flag.
- Use --careful mode to reduce mismatches in final assemblies (slower, recommended for small genomes).
- SPAdes also has rnaSPAdes for RNA assembly and hybridSPAdes for hybrid assembly.
- --isolate flag is recommended for high-coverage bacterial/viral isolates; improves quality and speed.
- --rnaviral mode assembles viral RNA genomes; --corona mode is specialized for coronaviruses using HMMs.
- --bio (biosyntheticSPAdes) assembles non-ribosomal and polyketide gene clusters.
- --metaviral detects viruses in metagenomic data; --metaplasmid detects plasmids in metagenomes.
- --sewage mode deconvolves SARS-CoV-2 strains from wastewater samples.
- --only-error-correction runs BayesHammer/IonHammer only; --only-assembler skips error correction.
- --continue resumes from last checkpoint; --restart-from allows restarting from specific stage.

## Pitfalls
- SPAdes has NO subcommands. ARGS starts directly with flags (e.g., -1, -2, -o, --meta, --careful). Do NOT put a subcommand like 'assemble' or 'run' before flags.
- SPAdes requires significant RAM — human genome assembly needs ~250 GB RAM; for bacteria use 16-32 GB.
- For metagenomes, --meta flag is required — running without it gives poor metagenome assemblies.
- The --careful option is NOT compatible with --meta mode.
- SPAdes creates a large output directory with intermediate files — ensure sufficient disk space.
- Input reads should be trimmed before SPAdes for better assembly quality.
- Do NOT mix different insert size libraries without specifying separate library groups (--pe1-1, --pe2-1, etc.).
- --isolate is incompatible with --careful and --only-error-correction; choose one approach.
- --rnaviral, --rna, and --corona modes are incompatible with --careful and --only-error-correction.
- Default memory limit is 250GB; explicitly set -m for smaller systems to avoid termination.
- --continue requires exact same -o directory; cannot change parameters when continuing.
- k-mer sizes must be odd and < 128; even numbers or values ≥128 cause errors.
- Hybrid assembly (--pacbio, --nanopore) with metagenomes is experimental; results may vary.

## Examples

### assemble a bacterial genome from paired-end reads
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o spades_output/ --threads 16 --memory 32 --careful`
**Explanation:** spades command; -1 R1.fastq.gz paired-end R1 input; -2 R2.fastq.gz paired-end R2 input; -o spades_output/ output directory; --threads 16 cores; --memory 32 limits RAM to 32 GB; --careful reduces mismatches

### assemble a metagenome from paired-end reads
**Args:** `--meta -1 R1.fastq.gz -2 R2.fastq.gz -o metaspades_output/ --threads 32 --memory 128`
**Explanation:** spades command; --meta enables metaSPAdes mode for metagenomic assembly; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o metaspades_output/ output directory; --threads 32 cores; --memory 128 GB RAM; higher memory for diverse communities

### assemble plasmids from paired-end reads
**Args:** `--plasmid -1 R1.fastq.gz -2 R2.fastq.gz -o plasmidspades_output/ --threads 8 --memory 16`
**Explanation:** spades command; --plasmid mode for plasmid assembly and recovery; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o plasmidspades_output/ output directory; --threads 8 cores; --memory 16 GB RAM

### assemble single-cell MDA amplified data
**Args:** `--sc -1 R1.fastq.gz -2 R2.fastq.gz -o sc_spades_output/ --threads 8 --memory 32`
**Explanation:** spades command; --sc single-cell mode handles uneven coverage typical of single-cell MDA amplified data; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o sc_spades_output/ output directory; --threads 8 cores; --memory 32 GB RAM

### resume interrupted SPAdes assembly
**Args:** `-o spades_output/ --continue`
**Explanation:** spades command; -o spades_output/ output directory; --continue resumes from the last successfully completed stage; requires the same output directory

### assemble with both paired-end and long reads (hybrid assembly)
**Args:** `-1 short_R1.fastq.gz -2 short_R2.fastq.gz --nanopore long_reads.fastq.gz -o hybrid_output/ --threads 16 --memory 64`
**Explanation:** spades command; -1 short_R1.fastq.gz -2 short_R2.fastq.gz short paired-end inputs; --nanopore long_reads.fastq.gz ONT long reads for hybrid assembly; -o hybrid_output/ output directory; --threads 16 cores; --memory 64 GB RAM; SPAdes integrates short and long reads

### assemble bacterial isolate with --isolate mode (recommended)
**Args:** `--isolate -1 R1.fastq.gz -2 R2.fastq.gz -o isolate_output/ --threads 16 --memory 32`
**Explanation:** spades command; --isolate optimized for high-coverage isolate data; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o isolate_output/ output directory; --threads 16 cores; --memory 32 GB RAM; faster and better quality than default mode

### assemble viral RNA genome
**Args:** `--rnaviral -1 R1.fastq.gz -2 R2.fastq.gz -o rnaviral_output/ --threads 8 --memory 16`
**Explanation:** spades command; --rnaviral for viral RNA assembly; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o rnaviral_output/ output directory; --threads 8 cores; --memory 16 GB RAM; handles high mutation rates and variable coverage

### assemble coronavirus genome with HMM guidance
**Args:** `--corona -1 R1.fastq.gz -2 R2.fastq.gz -o corona_output/ --threads 8 --memory 16`
**Explanation:** spades command; --corona uses Pfam HMMs for SARS-CoV-2 assembly; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o corona_output/ output directory; --threads 8 cores; --memory 16 GB RAM; more accurate for coronavirus genomes

### assemble biosynthetic gene clusters
**Args:** `--bio -1 R1.fastq.gz -2 R2.fastq.gz -o bio_output/ --threads 16 --memory 64`
**Explanation:** spades command; --bio biosyntheticSPAdes for non-ribosomal and polyketide gene cluster assembly; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o bio_output/ output directory; --threads 16 cores; --memory 64 GB RAM; specialized for secondary metabolites

### detect viruses in metagenomic data
**Args:** `--metaviral -1 R1.fastq.gz -2 R2.fastq.gz -o metaviral_output/ --threads 32 --memory 128`
**Explanation:** spades command; --metaviral for viral discovery in metagenomes; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o metaviral_output/ output directory; --threads 32 cores; --memory 128 GB RAM; outputs linear putative viral contigs

### run only read error correction
**Args:** `--only-error-correction -1 R1.fastq.gz -2 R2.fastq.gz -o ec_output/ --threads 16 --memory 32`
**Explanation:** spades command; --only-error-correction runs BayesHammer only; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -o ec_output/ output directory; --threads 16 cores; --memory 32 GB RAM; useful for correcting reads before using other assemblers

### run only assembly (skip error correction)
**Args:** `--only-assembler -1 corrected_R1.fastq.gz -2 corrected_R2.fastq.gz -o asm_output/ --threads 16 --memory 32`
**Explanation:** spades command; --only-assembler skips error correction; -1 corrected_R1.fastq.gz -2 corrected_R2.fastq.gz corrected paired-end inputs; -o asm_output/ output directory; --threads 16 cores; --memory 32 GB RAM; use when input reads are already corrected

### restart from specific checkpoint with updated options
**Args:** `-o spades_output/ --restart-from k55 --memory 64`
**Explanation:** spades command; -o spades_output/ output directory; --restart-from k55 resumes from k=55 stage; --memory 64 GB RAM; useful for increasing memory or changing parameters mid-run
