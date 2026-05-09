---
name: carna
category: genome-assembly
description: A genome assembly tool that constructs consensus sequences from overlapping NGS reads using the overlap-layout-consensus algorithm. It builds contiguous sequences (contigs) from short read inputs by detecting read overlaps, laying out reads into contigs, and generating consensus sequences.
tags: genome-assembly, NGS, genomics, contigs, overlap-layout-consensus, sequencing, read-assembly
author: AI-generated
source_url: https://github.com/marbl/cabog
---

## Concepts

- **Input Format**: Carna accepts short read sequences in FASTA or FASTQ format, where each read contains a sequence identifier, bases (A, T, G, C, N), and optional quality scores. Reads need not be pre-sorted but should be from a single library or combined libraries with similar error profiles.

- **Overlap Detection**: The tool performs all-versus-all read alignment to identify overlapping read pairs. Overlaps must meet a minimum overlap length threshold (controlled by the `-ol` parameter) and minimum identity (controlled by `-oi`). Longer overlaps increase specificity but reduce coverage; shorter overlaps increase false positives.

- **Output Data Model**: Carna outputs assembled contigs in FASTA format. Each contig is a consensus sequence derived from the overlap graph. The output also includes an optional assembly graph file (AFG format) showing the layout of reads within contigs for downstream scaffolding.

- **Key Parameters**: Essential parameters include the minimum overlap length (`-ol`), minimum overlap identity (`-oi`), read type specification (`-rt`), and maximum gap size (`-mx`). These directly affect assembly contiguity and accuracy.

## Pitfalls

- **Setting minimum overlap too short**: Using overly permissive overlap settings (e.g., `-ol 10 -oi 80` on 100bp reads) creates spurious overlaps between non-homologous reads, leading to chimeric contigs and fragmented assemblies. The resulting contigs may stitch together distant genomic regions incorrectly.

- **Ignoring read quality**: Feeding low-quality or adaptor-contaminated reads without quality trimming causes the assembler to incorporate errors into the consensus sequence. This results in base-call errors in assembled contigs that propagate through downstream analysis.

- **Insufficient memory allocation**: Carna builds an overlap graph in memory; datasets with hundreds of millions of reads may exceed default allocations, causing crashes or premature termination. Always estimate memory needs based on read count and allocate accordingly.

- **Mismatched read type parameters**: Specifying the wrong read type (`-rt`) relative to the input data (e.g., using Illumina vs. Sanger read specifications) causes incorrect error modeling. This degrades overlap detection accuracy because the assembler applies incorrect quality score thresholds.

- **Combining incompatible libraries**: Attempting to assemble mixed read sets from different organisms, highly divergent coverage depths, or different sequencing technologies without separate assembly can produce chimeric contigs or failed assemblies.

## Examples

### Assemble reads from a single library
**Args:** `-l reads.fasta -o assembly_output -ol 30 -oi 95 -rt illumina`
**Explanation:** This runs a basic assembly on an input FASTA file using a 30bp minimum overlap and 95% identity threshold appropriate for high-quality Illumina reads.

### Assemble FASTQ reads with quality scores
**Args:** `-l reads.fastq -o assembly_output -ol 35 -oi 97 -rt illumina -q`
**Explanation:** When input reads include quality scores (FASTQ format), the `-q` flag enables quality-aware overlap detection for improved consensus accuracy.

### Control memory usage for large datasets
**Args:** `-l large_reads.fasta -o assembly_output -ol 40 -oi 95 -mem 32G`
**Explanation:** The `-mem` flag allocates 32GB of memory for the overlap graph, preventing out-of-memory errors on large datasets with high read counts.

### Produce assembly graph file
**Args:** `-l reads.fasta -o assembly_output -ol 32 -oi 94 -rt illumina -afg graph.afg`
**Explanation:** The `-afg` flag outputs the assembly graph in AFG format, which can be used for visualization or scaffolding with other tools.

### Handle variable read lengths in input
**Args:** `-l mixed_reads.fasta -o assembly_output -ol 25 -oi 90 -rt generic -vl`
**Explanation:** The `-vl` flag enables variable-length read handling, allowing mixed read length inputs while adjusting overlap detection accordingly.