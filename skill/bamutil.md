---
name: bamutil
category: BAM/SAM Manipulation
description: bamutil is a suite of C++ programs for BAM/SAM file manipulation, including read trimming, duplicate removal, MD-tag calculation, region extraction, and statistics reporting. Originally developed at the University of Michigan.
tags: [bam, sam, alignment, quality-control, sequencing, variant-calling]
author: AI-generated
source_url: https://genome.sph.umich.edu/wiki/BamUtil
---

## Concepts

- **Core Utilities**: The bamutil suite consists of several independent programs: `trimBam` for trimming genomic regions from reads, `calMD` for recomputing MD tags using a reference genome, `bam` (with subcommands) for statistics and filtering, and `write2csrg` for combining multiple SAM files into a single merged file.

- **Input/Output Formats**: Programs accept SAM (text) and BAM (binary) inputs auto-detected by file extension (.sam vs .bam), and output to stdout or specified file paths. The `bam` utility requires explicit subcommands (`stat`, `writeRegion`, `filterReads`, `rmDup`, `splitBam`) passed as the first positional argument.

- **Reference Genome Requirements**: `calMD` requires a reference FASTA file for recalculating MD and NM tags. `trimBam` uses genomic coordinates (1-based, inclusive) to specify which regions to trim from read sequences, storing results in the XR tag.

- **Duplicate Removal Modes**: `bam rmDup` offers two approaches: `rmDup` mode physically removes duplicate read pairs from output, while `rmDupByReadGroup` mode marks duplicates in the XT tag without removing them, preserving all reads for downstream analysis.

## Pitfalls

- **Confusing Program vs Subcommand Structure**: Users often attempt `bamutil stat` expecting a single command, but `stat` is a subcommand passed as `bam stat`. Running `bam` without a subcommand produces an error listing valid subcommands—read this output to identify available operations.

- **Reference Genome Incompatibilities**: When using `calMD`, the reference genome must exactly match the alignment reference; using a different reference build causes incorrect MD/NM tag calculation, leading to silent errors in variant calling downstream. Always verify reference consistency.

- **Trimming Produces Unmapped Reads**: After `trimBam` removes genomic regions, some reads may become entirely unmapped (CIGAR becomes `*`). These reads are retained in the output and will cause issues in tools expecting mapped reads unless filtered separately.

- **Memory Usage on Large BAM Files**: Programs operate on BAM files sequentially in streaming mode, but certain operations may hold reads in memory temporarily; insufficient RAM can cause crashes on deeply sequenced datasets. Consider sorting and indexing (`samtools index`) before processing large files.

- **Missing SAM Header Causes Errors**: Many bamutil programs require a valid SAM header (`@HD`, `@SQ`, `@RG` lines) in the input file. BAM files without proper headers or with corrupted headers will cause unexpected crashes or empty outputs. Validate headers with `samtools view -H` before processing.

## Examples

### Calculate alignment statistics for a BAM file

**Args:** `stat --in merged.bam`
**Explanation:** The `stat` subcommand computes coverage depth, alignment rates, insert size distributions, and mapping quality histograms across the entire BAM file, outputting a comprehensive statistics report to stdout.

### Extract reads from a specific genomic region

**Args:** `writeRegion --in sample.bam --region chr1:1000000-2000000 --out chr1_1-2mb.bam`
**Explanation:** The `writeRegion` subcommand extracts all reads overlapping the specified genomic interval, writing them to a new BAM file subset containing only alignments within chr1:1,000,000-2,000,000 for focused downstream analysis.

### Trim adapter/contaminant sequences from reads using genomic coordinates

**Args:** `trimBam in.bam out.bam --region chr1:0-100 --region chrX:1000-2000`
**Explanation:** `trimBam` removes the specified genomic coordinate ranges from read sequences (stored in the XR tag) and updates CIGAR strings accordingly, outputting a trimmed BAM file where reads outside specified regions remain unchanged.

### Remove duplicate reads from paired-end alignment

**Args:** `rmDup --in paired.bam --out deduped.bam`
**Explanation:** The `rmDup` subcommand identifies read pairs mapping to the same genomic positions and removes duplicates, keeping only the read with the best mapping quality, which is essential for accurate variant allele frequency estimation.

### Recalculate MD and NM tags using a reference genome

**Args:** `calMD --in aln.bam --ref reference.fa --out recalc.bam`
**Explanation:** `calMD` recomputes the MD (mismatching positions) and NM (edit distance) tags by comparing alignments against the provided reference FASTA, correcting any incorrectly calculated tags from the aligner and enabling accurate variant calling with GATK or similar tools.