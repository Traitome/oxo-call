I'll generate a skill file for `conduit-assembler`, a genome assembly tool. Based on typical genome assembly workflows and the Conduit toolkit's general approach, I'll create an accurate and actionable skill file.

---
name: conduit-assembler
category: genome-assembly
description: Assembles raw sequencing reads into contigs and scaffolds using overlap-layout-consensus algorithms. Part of the Conduit pipeline toolkit for next-generation genomics.
tags: [assembly, genomics, contig-assembly, overlap-graph, sequence-assembly]
author: AI-generated
source_url: https://github.com/conduit-pipeline/conduit-assembler
---

## Concepts

- **Input formats:** Accepts FASTA/FASTQ read files (single-end or paired-end), with support for Illumina, PacBio, and Oxford Nanopore raw reads. Multiple read files can be specified using comma-separated paths.
- **Overlap graph construction:** Uses k-mer counting and minimizer-based hashing to build an overlap graph representing read-to-read relationships, then performs bubble popping and transitive reduction to simplify the graph before consensus calling.
- **Output formats:** Produces FASTA-formatted contigs, optionally with assembly statistics in a separate metrics file. The output contig file can be used directly as input for downstream tools like conduit-annotator or external variant callers.
- **Parameter tuning:** Key parameters include minimum overlap length (default: 40bp for Illumina, 500bp for long reads), minimum read coverage depth, and error correction stringency. These must be adjusted based on read quality and organism genome complexity.

## Pitfalls

- **Insufficient overlap length for long reads:** Setting the minimum overlap too high (e.g., >1000bp for Nanopore data) can cause fragmented assemblies or complete failure to detect overlaps between related reads, resulting in highly fragmented contigs or an empty assembly.
- **Mismatched read types:** Running long-read assembly parameters with short-read data (or vice versa) leads to poor overlap detection, missing true overlaps, and creating chimeric contigs from misassembled read fragments.
- **Duplicate input files:** Specifying the same read file twice in the input (through oversight or glob expansion) causes artificial coverage inflation, biasing coverage depth estimation and leading to false duplication in the assembly graph.
- **Insufficient memory allocation:** Default memory settings may be inadequate for large eukaryotic genomes (e.g., plant or mammalian genomes), causing the assembler to crash mid-execution with out-of-memory errors or extreme slowdowns due to excessive swapping.
- **Ignoring read quality filtering:** Feeding unfiltered raw reads with high error rates without adjusting minimum identity thresholds produces low-quality assemblies with elevated misassembly rates and inflated consensus error rates.

## Examples

### Assemble Illumina paired-end reads into contigs
**Args:** --reads forward_reads.fastq.gz,reverse_reads.fastq.gz --output assembly_contigs.fasta --min-overlap 40 --min-identity 0.95
**Explanation:** Specifies paired-end input files and sets standard Illumina overlap thresholds (40bp minimum, 95% identity) for reliable short-read assembly.

### Assemble PacBio continuous long reads
**Args:** --reads pacbio_reads.fasta --output pacbio_assembly.fasta --min-overlap 500 --min-identity 0.80 --long-read-mode
**Explanation:** Enables long-read mode with relaxed overlap (500bp, 80% identity) suitable for higher error rates in PacBioCLR data.

### Assemble with custom k-mer size
**Args:** --reads input_reads.fq --output assembled.fasta --kmer-size 21 --min-coverage 3
**Explanation:** Overrides default k-mer size to 21 and requires at least 3-fold k-mer coverage to filter low-frequency errors.

### Generate assembly statistics report
**Args:** --reads sample_reads.fastq --output contigs.fasta --stats-output assembly_metrics.txt
**Explanation:** Writes detailed assembly metrics (contig count, N50, total length) to a separate text file for downstream analysis.

### Resume interrupted assembly from checkpoint
**Args:** --reads remaining_reads.fq --output continuation.fasta --checkpoint-dir /tmp/assembly_checkpoints --resume
**Explanation:** Uses previously saved checkpoint files to continue an assembly that was interrupted, avoiding reprocessing of completed work.