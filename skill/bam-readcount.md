---
name: bam-readcount
category: variant-analysis
description: Extracts per-base readcount metrics from BAM files at specified genomic positions, reporting nucleotide frequencies, mapping qualities, and coverage statistics for each base.
tags:
  - bam
  - coverage
  - variant-analysis
  - vcf
  - readcount
  - genomics
author: AI-generated
source_url: https://github.com/genome/bam-readcount
---

## Concepts

- **Position-based per-base metrics**: bam-readcount generates statistics for each queried genomic position, including coverage depth, count of A/C/G/T bases observed, mean mapping quality, mean base quality, and number of insertions/deletions. Each position in the output corresponds to a row with columns separated by colons.

- **Input requires both a sorted BAM and a genomic position list**: You must provide a coordinate-sorted BAM file (with accompanying index) and either a text file listing chromosome:position coordinates (one per line) or a BED file containing genomic intervals. The tool does not accept a VCF file directly; convert VCF variants to position format first.

- **Quality filtering controls which reads are included**: The `-q` flag sets a minimum mapping quality threshold (Phred scale), and `-m` sets a minimum base quality threshold. Reads below these thresholds are excluded from the count, which is critical for avoiding false positives in low-complexity or repetitive regions.

- **Output format is line-oriented with colon-delimited fields**: Each output line follows the pattern: `chr:pos:ref:depth:base counts:qualities`. The base counts section lists `A:# C:# G:# T:#` with the number of reads supporting each nucleotide. This format is designed for easy parsing by downstream scripts.

- **Companion tool for large-scale batch queries**: For querying thousands of positions efficiently, use `bam-readcount-build` to create an index file first, then pass it to `bam-readcount` with the `-f` flag. This approach significantly reduces runtime when processing multiple genomic regions.

## Pitfalls

- **Mismatched reference genome between BAM and query positions**: If the BAM file was aligned to GRCh37 but your positions are from GRCh38 (or vice versa), the output will show near-zero coverage or completely incorrect base counts. Always verify the reference assembly before running the tool.

- **Forgetting to sort and index the BAM file**: bam-readcount requires a coordinate-sorted and indexed BAM (`.bai` file must exist). Running on an unsorted BAM produces silently incorrect results or crashes. Always run `samtools sort` and `samtools index` before analysis.

- **Not excluding duplicates leads to inflated coverage**: Duplicate reads (PCR artifacts) are counted by default. In low-coverage experiments this may cause false variant calls. Use the `-d` flag to exclude reads marked with the duplicate flag, or preprocess your BAM with `samtools rmdup`.

- **Ignoring overlapping read pairs in paired-end data**: When a read pair overlaps the query position from both ends, bam-readcount counts each base observation separately, effectively double-counting that position. This inflates coverage and can mask real low-frequency variants. Be aware of this bias in amplicon-based or low-fragment-size sequencing.

- **Using a position list with wrong chromosome naming convention**: Some pipelines use `chr1` while others use `1`. A mismatch causes the tool to report zero coverage for every position. Check that chromosome names in your input file exactly match those in the BAM header.

## Examples

### Query a single genomic position from a BAM file
**Args:** `-f reference.fa -l positions.txt sample.bam`
**Explanation:** This runs bam-readcount on all positions listed in `positions.txt` using the provided BAM file, reporting per-base metrics for each location.

### Filter reads by minimum mapping quality of 30
**Args:** `-f reference.fa -q 30 -l positions.txt sample.bam`
**Explanation:** Setting `-q 30` excludes reads with mapping quality below Phred 30, reducing false positives from ambiguously aligned reads in repetitive regions.

### Use a BED file to query all positions within genomic regions
**Args:** `-f reference.fa -l regions.bed -b sample.bam`
**Explanation:** The `-b` flag indicates the input is a BED file rather than a simple position list, allowing efficient querying of entire genomic intervals instead of single-base positions.

### Build an index for fast batch querying of many positions
**Args:** `build -f reference.fa -l positions.txt sample.bam`
**Explanation:** Running `bam-readcount-build` (the companion binary) creates an index file, which can then speed up subsequent queries when the same position set is analyzed across multiple BAM files.

### Exclude duplicate reads from the analysis
**Args:** `-f reference.fa -d -l positions.txt sample.bam`
**Explanation:** The `-d` flag instructs bam-readcount to skip reads marked as duplicates, preventing PCR amplification artifacts from skewing base frequency calculations.

### Specify minimum base quality threshold to avoid sequencing errors
**Args:** `-f reference.fa -m 20 -l positions.txt sample.bam`
**Explanation:** Setting `-m 20` requires that individual base calls have Phred quality of at least 20 to be counted, filtering out likely sequencing errors that could mimic low-frequency variants.

### Query a list of variant positions from a VCF file
**Args:** `-f reference.fa -l