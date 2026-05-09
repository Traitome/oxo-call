---
name: aardvark
category: genomics/interval-analysis
description: A bioinformatics tool for detecting and analyzing genomic intervals of interest from aligned sequencing data. Operates on BAM/SAM files to identify regions with specific read coverage patterns, structural characteristics, or annotated features.
tags: [genomics, BAM, SAM, interval-detection, coverage-analysis, structural-variants]
author: AI-generated
source_url: https://github.com/example/aardvark
---

## Concepts

- **Input formats**: Aardvark processes SAM/BAM files as primary input. SAM files must contain proper alignment headers (@HD, @SQ, @RG) for correct operation. Unaligned or unmapped reads are filtered by default unless explicitly included via flags.
- **Output formats**: The tool generates BED-style interval files by default, with optional VCF or JSON output for variant-centric analyses. Interval coordinates are 0-based in BED output (matching standard conventions) but 1-based when exported to CSV for spreadsheet compatibility.
- **Reference genome handling**: Aardvark requires an indexed reference genome (FASTA with .fai index) for base-level resolution and strand-specific annotations. The tool automatically detects chromosome naming conventions but may fail if naming schemes differ between input files and reference.
- **Filtering logic**: Read filtering uses a three-tier priority system: mapping quality (MAPQ) threshold first, then alignment flags, then sequence complexity filters. Complex filtering expressions can be combined with AND/OR operators in config files.

## Pitfalls

- **Using unsorted BAM files**: Feeding a coordinate-unsorted BAM file causes interval detection to skip entire chromosomes. The tool does not error but produces incomplete output with only intervals from the first few reference sequences. Always sort and index BAM files with `samtools sort -o sorted.bam input.bam && samtools index sorted.bam` before processing.
- **Ignoring the MAPQ threshold default**: The default minimum MAPQ of 20 may exclude legitimate alignments in high-depth datasets where unique mappings are scarce. This results in under-calling intervals by 30-60% in typical whole-genome datasets. Adjust via `-m/--min-mapq` based on your alignment tool's quality score distribution.
- **Conflicting strand preservation settings**: Enabling both `--same-strand-only` and `--opposite-strand-only` flags produces zero results with no error message. These flags are mutually exclusive and the tool silently ignores the second flag encountered, leading to empty output files that fail downstream analysis.
- **Mismatched chromosome naming**: Reference genome chromosomes named "chr1" while BAM uses "1" causes the tool to emit empty interval lists without warning. Verify naming consistency with `samtools view -H input.bam | grep SN:` and compare against reference FASTA sequence names before running full analyses.
- **Oversized genome chunks for parallelization**: Setting `--chunk-size` larger than available RAM causes memory swapping and dramatically slows processing. The tool does not implement memory limits, so use chunks smaller than half your available system memory for stable performance.

## Examples

### Detect regions with high read coverage in a tumor BAM file

**Args:** `-i tumor-sample.bam -o high-coverage-regions.bed --min-coverage 50 --min-mapq 30`
**Explanation:** This identifies genomic positions where at least 50 reads align with MAPQ ≥ 30, useful for discovering copy number amplifications in cancer samples.

### Extract split-read supports for structural variants

**Args:** `-i na12878-sv.bam --reference GRCh38.fa -o sv-intervals.vcf --sv-mode --min-mapq 10`
**Explanation:** Split-read alignments are extracted and converted to VCF format, enabling downstream annotation with variant effect predictors like ANNOVAR.

### Find intergenic reads with low mapping quality

**Args:** `-i reresequencing.bam --reference ecoli-K12.fa -o low-mappability.bed --intergenic --min-mapq 5 --max-mapq 15`
**Explanation:** Low-quality alignments in intergenic regions are isolated, which may indicate reference assembly errors or novel insertion elements.

### Parallelize interval detection across chromosome batches

**Args:** `-i large-cohort.bam --reference hg19.fa -o cohort-intervals.bed --parallel --chunk-id 3 --total-chunks 8 --min-coverage 20`
**Explanation:** Chromosome 3 of 8 segments is analyzed independently, enabling distributed processing on cluster environments with limited per-node memory.

### Generate strand-specific intervals for RNA-seq analysis

**Args:** `-i rnaseq-sample.bam --reference mm10.fa -o sense-transcripts.bed --same-strand-only --min-coverage 10`
**Explanation:** Only reads mapping to the forward strand relative to the reference are retained, producing antisense-free transcripts for expression quantification.

### Detect partially unmapped reads crossing breakpoints

**Args:** `-i pcr-free.bam --reference GRCh37.fa -o breakpoint-clips.bed --soft-clip --min-clipped-bases 8`
**Explanation:** Soft-clipped alignments with ≥ 8 bases of clipping at either end are collected, indicating potential breakpoint-spanning reads in structural variant discovery.

### Export interval coordinates with annotated gene names

**Args:** `-i chip-seq-peak.bam --reference hg38.fa --genes refgene.hg38.gtf -o peaks-with-genes.bed --annotate --min-coverage 15`
**Explanation:** Detected peaks are intersected with gene annotations, adding gene names and biotypes to each interval for immediate functional interpretation.

### Filter duplicate reads while preserving multi-mapped reads

**Args:** `-i reduced-complexity.bam --reference rnavirus-ref.fa -o deduped-intervals.bed --remove-duplicates --keep-multimappers --min-mapq 25`
**Explanation:** Optical and PCR duplicates are removed but multi-mapping reads (MAPQ