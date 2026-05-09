---
name: bleach
category: Bisulfite Sequencing Alignment
description: A bisulfite sequencing aligner for DNA methylation analysis that performs ungapped alignment of reads to a reference genome, handling cytosine-to-thymine conversion in both reads and reference. Supports whole-genome bisulfite sequencing (WGBS) and reduced representation bisulfite sequencing (RRBS) workflows.
tags:
  - methylation
  - bisulfite
  - WGBS
  - RRBS
  - DNA-methylation
  - epigenetic
  - alignment
  - NGS
author: AI-generated
source_url: https://code.google.com/archive/p/bleach/
---

## Concepts

- **Bisulfite conversion model**: Bleach exploits the chemical conversion of unmethylated cytosine residues to uracil (read as thymine after PCR) during bisulfite treatment. During alignment, both the read sequence and the reference genome are converted to the bisulfite space, enabling alignment to detect C→T mismatches as potential methylation marks.
- **Ungapped alignment with M-bias handling**: Bleach performs only ungapped alignments, which means it cannot span insertions or deletions introduced by bisulfite conversion or sequencing errors. Users must consider trimming low-quality bases and adapter sequences upfront, especially at read ends where M-bias (positional bias in methylation estimates) is most severe.
- **Three-letter genome indexing**: The companion `bleach-build` tool constructs a three-letter reference genome index (A/G/T replacing C residues) rather than the standard four-letter alphabet, allowing simultaneous alignment to the converted forward and reverse reference strands.
- **Output formats**: Bleach outputs alignments in a custom SAM-like format with added methylation-specific tags: `XM` (mismatch pattern), `XR` (read conversion state), and `XG` (genome conversion state). Downstream tools like Bismark or MOABS parse these tags to extract methylation calls at single-base resolution.
- **Strand-specific reporting**: Each bisulfite-converted read can align to up to four reference strands (original forward, original reverse, converted forward, converted reverse). Bleach reports the single best alignment per read, and the strand origin is encoded in the alignment flags and conversion tags.

## Pitfalls

- **Forgetting to build a bleach index**: Running `bleach` with a standard genome FASTA or Bowtie/BWA index will fail silently or produce incorrect alignments. Users must always run `bleach-build` first to generate the three-letter index, which is specific to bleach and incompatible with other aligners.
- **Mismatching genome version between index and input reads**: If the reference genome version used for `bleach-build` does not match the genome to which reads were aligned (e.g., hg19 vs hg38), all methylation calls will be mispositioned. Always verify the genome build identifier in sample sheet metadata before indexing.
- **Neglecting quality trimming causing M-bias artifacts**: Untrimmed low-quality bases at read ends introduce systematic overestimation of methylation levels in terminal positions. This biases differential methylation analysis, especially at promoter and CpG island boundaries where methylation is biologically informative.
- **Running in single-threaded mode on large WGBS datasets**: By default, bleach uses a single thread, making whole-genome alignment of billions of reads computationally prohibitive. Failing to specify `-t` for thread count dramatically increases runtime, making parallel execution infeasible.
- **Confusing `bleach-build` output directory with reference FASTA path**: Users sometimes pass the `bleach-build` output directory path to `bleach` instead of the original FASTA file path, causing alignment failure. The alignment step requires the original genome FASTA and the corresponding index directory simultaneously.

## Examples

### Build a bisulfite genome index from a reference FASTA
**Args:** `ecoli_ref.fa`
**Explanation:** This command runs `bleach-build` to construct the three-letter bisulfite-converted index from the input FASTA file, which is a prerequisite for all subsequent alignment operations using `bleach`.

### Align WGBS reads to a bisulfite-converted genome using 8 threads
**Args:** `reads_1.fq reads_2.fq -d ecoli_ref -t 8 -f fastq-id`
**Explanation:** Specifying `-t 8` enables multi-threaded alignment for a paired-end whole-genome bisulfite sequencing dataset, significantly reducing runtime on multi-core systems, while `-f fastq-id` sets the read naming scheme for the output SAM.

### Align single-end RRBS reads with quality-score encoding set to Sanger
**Args:** `rrbs_sample.fq -d hg38_ref -t 4 -q sanger`
**Explanation:** Reduced representation bisulfite sequencing reads are typically shorter and more prone to adapter contamination, so setting `-q sanger` ensures correct base quality interpretation for accurate methylation calling downstream.

### Align paired-end reads suppressing duplicate alignments
**Args:** `wgbs_pe_1.fq wgbs_pe_2.fq -d hg19_ref -t 12 -D`
**Explanation:** The `-D` flag suppresses reporting of duplicate alignments that arise from PCR amplification bias in bisulfite sequencing, which would otherwise inflate apparent read coverage and skew methylation beta values.

### Align single-end reads using bowtie1-style quality scoring (Phred+64)
**Args:** `sample.fq -d reference -q illumina1.3q`
**Explanation:** Historical RRBS libraries sequenced on older Illumina platforms used Phred+64 ASCII quality encoding, so specifying `-q illumina1.3q` ensures bleach interprets base quality scores correctly and applies appropriate mismatch thresholds during alignment.