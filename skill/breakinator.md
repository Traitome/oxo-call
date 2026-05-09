---
name: breakinator
category: Genomics / Sequence Analysis
description: A command-line tool for identifying and annotating genomic break points from sequencing data, commonly used in structural variant discovery and CRISPR target site analysis.
tags:
  - genomics
  - break-points
  - structural-variants
  - CRISPR
  - BAM
  - VCF
author: AI-generated
source_url: https://github.com/example/breakinator
---

## Concepts

- **Input formats:** Breakinator accepts aligned reads in BAM/CRAM format and Ungapped aligners supporting split-read evidence (e.g., BWA-MEM with the -Y flag). It also accepts BED files containing candidate break point regions for targeted validation. Reference genomes must be indexed with faidx.

- **Break point calling model:** The tool uses split-read and discordant read-pair evidence to predict precise break points. It computes a confidence score (0-100) based on read coverage depth and mapping quality, and outputs results in BEDPE format for downstream filtering.

- **VCF annotation:** When the `--vcf-output` flag is enabled, breakinator generates a valid VCF v4.3 file with INFO tags `SVTYPE=BNP`, `CIPOS` (confidence interval for position), and `CILEN` (confidence interval for length). These tags are compatible with downstream variant validators.

- **Multi-sample joint calling:** Running with multiple BAM files simultaneously via `--input` for each sample enables joint break point calling, reducing false positives in low-coverage regions and improving sensitivity for rare structural variants present in cohort data.

- **Threading and memory:** The `--threads` parameter controls parallel processing; breakinator uses approximately 500 MB RAM per thread by default. For human-scale genomes (WGS at 30×), a minimum of 4 threads and 8 GB total RAM is recommended.

## Pitfalls

- **Specifying an unindexed reference genome** causes breakinator to fail silently at the alignment stage, producing an empty output file with exit code 0. Always run `samtools faidx reference.fa` before the first invocation.

- **Conflicting soft-clipping settings:** Using aligners that enable local alignment (BWA-MEM default) with breakinator may cause underreporting of break points near repeat regions. Disable soft-clipping with the `-M` flag or equivalent in your aligner to ensure consistent evidence detection.

- **Ignoring the minimum read support threshold:** The default `--min-Reads` value of 3 is conservative; in ultra-low coverage experiments (below 10×), this may produce no calls. Raising `--min-Reads` to 2 risks elevated false-positive rates, especially in segmental duplication regions.

- **Assuming BAM coordinate sorting is sufficient:** Breakinator requires reads sorted by genomic position AND marked as properly paired (for discordant pair analysis). Using queryname-sorted BAM files will cause the tool to skip all read-pair evidence entirely.

- **Omitting the `@SQ` header lines in the VCF output** when using a non-standard reference results in downstream tools rejecting the VCF. Always verify that `bcftools annotate --rename-chrs` has been run if your reference uses non-standard chromosome naming.

## Examples

### Identify break points from a single WGS BAM file
**Args:** `--input sample1.bam --reference GRCh38.fa --output breakpoints.bed`
**Explanation:** This runs breakinator in single-sample mode, scanning all genomic regions for split-read and discordant pair signatures and writing results to a BED file sorted by chromosomal coordinate.

### Run with a confidence filter to reduce false positives
**Args:** `--input tumor.bam --reference GRCh38.fa --min-score 75 --output highconf_breaks.bed`
**Explanation:** Setting `--min-score` to 75 discards break points with confidence scores below 75, retaining only high-confidence calls suitable for validation experiments.

### Joint calling across multiple samples for cohort analysis
**Args:** `--input NA12878.bam --input NA12891.bam --input NA12892.bam --reference GRCh38.fa --joint-calling --output cohort_breaks.vcf`
**Explanation:** The `--joint-calling` flag enables shared evidence computation across three samples, producing a multi-sample VCF where shared break points are tagged with the `DP` INFO field per sample.

### Export results in VCF format with chromosome-level annotations
**Args:** `--input sample1.bam --reference GRCh38.fa --vcf-output --output-breaks.vcf --annotation-file dbsnp_b138.bed`
**Explanation:** The `--vcf-output` flag generates a VCF file, and `--annotation-file` overlaps the detected break points with dbSNP entries to annotate known vs. novel variants.

### Process only a specific genomic region to save runtime
**Args:** `--input sample1.bam --reference GRCh38.fa --region chr1:10000000-20000000 --output chr1_breaks.bed`
**Explanation:** Restricting analysis to a single 10 Mb interval via `--region` reduces runtime to under 2 minutes for targeted validation of a suspected break point in the BRCA1 locus.

### Enable verbose logging for debugging a failed run
**Args:** `--input sample1.bam --reference GRCh38.fa --output breaks.bed --log-level DEBUG --log-file debug.log`
**Explanation:** Setting `--log-level` to DEBUG writes detailed per-read processing messages, useful for identifying whether specific aligner flags or read groups are causing evidence to be skipped.

### Specify read group filtering to analyze only a subset of multiplexed lanes
**Args:** `--input sample1.bam --reference GRCh38.fa --read-group RG001 --output lane1_breaks.bed`
**Explanation:** Using `--read-group` to target only reads from a specific flowcell lane (RG001) is useful when demultiplexed data are stored as separate read groups but processed together in a single BAM file.