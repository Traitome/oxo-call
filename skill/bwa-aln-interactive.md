---
name: bwa-aln-interactive
category: sequence-alignment
description: Aligns short sequencing reads (≤200 bp) to a reference genome using an indexed Burrows-Wheeler transform and generates suffix array coordinates for subsequent SAM conversion. Works as part of a two-step pipeline with bwa-samse or bwa-sampe.
tags:
- alignment
- short-reads
- bwt
- sequencing
- genomics
author: AI-Generated
source_url: http://bio-bwa.sourceforge.net/
---

## Concepts

- The reference genome must be indexed with `bwa index` before `bwa aln` can function. The index files (`.amb`, `.ann`, `.bwt`, `.pac`, `.sa`) are required input. Running `bwa aln` without a valid index produces an immediate error.
- `bwa aln` produces an SA (suffix array) coordinates file, NOT a SAM file. This intermediate binary file must be passed to `bwa samse` (single-end) or `bwa sampe` (paired-end) to generate the final aligned SAM output viewable in IGV or other alignment browsers.
- The `-n` option sets the maximum number of differences (edit distance) per read, controlling sensitivity. Lower values (e.g., `-n 0.01`) yield strict alignments only; higher values (e.g., `-n 4`) allow more mismatches and increase sensitivity at the cost of specificity. A value of `0.04` (4%) is a common default for typical Illumina data.
- The alignment process uses a seed-and-extend strategy with configurable seed length (`-l`, default 32 for reads >28bp) and maximum seed mismatches (`-k`, default 2). For very short reads, automatic seed length reduction occurs to maintain alignment quality.
- Output to stdout allows piping directly to `bwa samse` or `bwa sampe`, enabling one-command workflows without intermediate disk files.

## Pitfalls

- Forgetting to run `bwa samse` or `bwa sampe` after `bwa aln` leaves you with an unusable SA file. Standard SAM/BAM viewers cannot read the SA coordinates format. Attempting to view an `.sai` file directly in a genome browser produces no data and no intuitive error message.
- Using `bwa samse` when reads are actually paired-end produces invalid SAM output with incorrect pairing flags and zero quality scores in the BAM file, corrupting downstream variant calling and fragment size estimation.
- Specifying `-n` with a value that exceeds reasonable edit distance thresholds (e.g., `-n 10`) causes excessive runtime and generates spurious alignments to paralogous or repetitive regions, degrading variant calling accuracy downstream.
- Running `bwa aln` on long reads (>200bp) with default parameters produces poor alignment quality because the seed length is too short for accurate mapping, leading to many multi-mapping reads and false structural variant calls.
- Omitting `-f` to specify an output filename causes the SA file to write to stdout. When redirecting with `>` in a pipeline with multiple samples, this overwrites rather than creates separate SA files, losing all but the last sample's alignment data.

## Examples

### Align single-end Illumina reads to an indexed reference genome

**Args:** `hg19.fa SRR001468.fastq > aln_out.sai`
**Explanation:** This aligns a single-end FASTQ file to an already-indexed reference, outputting SA coordinates that must be converted using `bwa samse` before obtaining a SAM file.

### Align paired-end reads and save SA coordinates to named files

**Args:** `-f reads1.sai hg19.fa pair1.fastq`
**Explanation:** The `-f` flag writes SA coordinates to a named file instead of stdout, which is essential when processing paired-end data where two separate SA files are required for `bwa sampe`.

### Perform sensitive alignment allowing up to 5% edit distance

**Args:** `hg19.fa sensitive_reads.fastq -n 0.05 > sensitive.sai`
**Explanation:** Setting `-n 0.05` permits alignments with up to 5% sequence differences, increasing sensitivity for low-quality reads or reads from diverged strains while maintaining a reasonable specificity balance.

### Align with a reduced seed length for short 36-bp reads

**Args:** `hg19.fa reads36bp.fastq -l 20 > short_reads.sai`
**Explanation:** The `-l` parameter reduces the seed length from the default 32 to 20, accommodating 36-bp reads that would otherwise have insufficient seed length for reliable alignment.

### Align paired-end read batch with maximum gap opens set to 3

**Args:** `-f read2.sai hg19.fa pair2.fastq -o 3`
**Explanation:** The `-o 3` parameter permits up to 3 gap openings per alignment, improving alignment quality in regions with insertion/deletion polymorphisms without allowing unbounded indel length.

### Align reads while allowing extension-free gaps of maximum length 1

**Args:** `hg19.fa reads.fastq -e 1 > gaps.sai`
**Explanation:** The `-e 1` parameter penalizes gap extensions heavily, preventing long insertions or deletions and favoring split alignments when structural variants are suspected.

### Align with a maximum of 100,000 allowed mapq values for repetitive reads

**Args:** `hg19.fa repetitive.fastq -m 100000 > rep_map.sai`
**Explanation:** The `-m` parameter controls the maximum number of occurrences allowed before a read is marked with zero mapping quality, helping manage multi-mapping in repetitive genomic regions.

### Align reads with a read group ID for downstream GATK compatibility

**Args:** `-f aln.sai hg19.fa reads.fastq -R '@RG\tID:sample1\tSM:sample1'`
**Explanation:** The `-R` parameter embeds read group metadata into the SA file, which propagates through `bwa samse` into the final SAM file for compatibility with GATK and other variant calling tools.

### Align reads with fastq filename containing N characters masked by default

**Args:** `hg19.fa ambiguous_reads.fastq -N > masked.sai`
**Explanation:** The `-N` flag forces alignment with `N` as a regular match character rather than masking it, which is sometimes required when working with modified base calling or haploid assembly inputs.