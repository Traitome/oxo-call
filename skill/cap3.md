---
name: cap3
category: sequence-assembly
description: A DNA sequence assembly and contig building tool that performs pairwise sequence alignment, overlap detection, consensus generation, and quality score calculation for DNA sequence fragments.
tags: dna-assembly, sequence-assembly, bioinformatics, contig-building, overlap-layout-consensus
author: AI-generated
source_url: http://seq.cs.iastate.edu/cap3.html
---

## Concepts

- **Input Format**: cap3 reads FASTA-format DNA sequence files, where each entry represents an individual read or sequence fragment to be assembled into contigs. The tool processes forward and reverse complemented sequences bidirectionally by default.

- **Assembly Algorithm**: The tool uses an overlap-layout-consensus (OLC) approach, computing pairwise alignments between all input sequences to detect overlaps with a user-specified minimum overlap length and minimum identity percentage. Sequences with significant overlaps are merged into contigs.

- **Output Files**: cap3 generates multiple output files including: a `*.contigs` file containing assembled consensus sequences, a `*.ace` file for assembly visualization, and log files with quality scores and detailed overlap information for each contig.

- **Key Parameters**: Critical parameters include `-p` (minimum overlap identity percentage, default 40%), `-o` (minimum overlap length, default 20 bp), and `-s` (maximum gap size within overlaps). These control stringency and affect assembly sensitivity.

## Pitfalls

- **Setting overlap identity too low** (e.g., `-p 30` or below) can cause unrelated sequences to be incorrectly merged, resulting in chimeric contigs with incorrect consensus sequences that propagate errors through downstream analysis.

- **Ignoring quality scores in input data** leads to poor assembly when low-quality or truncated sequences are included; this causes misalignments at noisy regions and produces unreliable consensus sequences with inflated error rates.

- **Processing empty or single-sequence files** produces no meaningful output; cap3 requires multiple sequences to find overlaps and build contigs, so running it on inadequate input wastes computational resources without generating assemblies.

- **Using inconsistent sequence naming** across input files makes it difficult to track which original sequences contributed to each contig in the output; this complicates downstream annotation and verification steps.

## Examples

### Assemble raw read sequences from a sequencing project

**Args