---
name: break-point-inspector
category: Structural Variant Analysis
description: A tool for inspecting, validating, and reporting on genomic breakpoints from structural variant calls. Supports VCF, BED, and BAM/CRAM inputs to provide detailed breakpoint annotations, adjacency validation, and microhomology analysis.
tags:
  - structural-variants
  - breakpoints
  - vcf
  - sv-analysis
  - genomics
author: AI-generated
source_Url: https://github.com/bioinformatics-tools/break-point-inspector
---

## Concepts

- **VCF/BCF Input Format**: break-point-inspector reads structural variant calls from VCF version 4.2+ files, expecting INFO fields such as END, SVLEN, SVTYPE, CIPOS, CIEND, and HOMLEN. Untagged or malformed VCF records will produce incomplete or silent failure in breakpoint extraction.
- **BAM/CRAM Validation**: The tool can cross-validate reported breakpoints against aligned reads by scanning split-read patterns and discordant read pairs. The --alignment/-a flag requires a coordinate-sorted and indexed BAM/CRAM file; unindexed files cause the validation step to abort with an indexing error.
- **Breakpoint Annotation Output**: Three output formats are supported: JSON for programmatic pipelines (--format json), TSV for manual review (--format tsv), and VCF for downstream tools (--format vcf-out). Each format captures different resolution levels of breakpoint evidence including exact coordinates, assembly sequences, and homology features.
- **Chromosomal Coordinate Handling**: Coordinates are interpreted as 1-based inclusive by default for VCF compatibility. The --zero-based flag switches interpretation for BED-style coordinate systems; mixing coordinate conventions in multi-file workflows produces systematic 1-base offset errors in all reported positions.

## Pitfalls

- **Mismatched Genome Assemblies**: Using a BAM/CRAM file aligned to GRCh38 with a reference genome file for GRCh37 produces coordinate mismatches that silently offset all breakpoint validations by approximately 1000 base pairs on acrocentric chromosomes. Always verify assembly concordance with --genome-version before running validation.
- **Unfiltered Multi-Sample VCF**: Passing a joint-called VCF with many low-confidence variants causes the inspector to report excessive candidate breakpoints, inflating runtime and producing noisy output. Using --min-svlen 50 combined with --min-qual 20 reduces input variants to biologically meaningful structural alterations.
- **Missing Mate Information in BAM**: When validating breakpoints using split-read evidence, single-end reads without proper mate information (NH:i:1 without a valid MRNM field) are silently discarded, causing artificially low support values. Verify BAM proper-pair flags with samtools flagstat before inspection.
- **Memory Allocation for Large VCFs**: Processing VCF files with more than 50,000 structural variants without specifying --chunk-size causes excessive memory consumption. The default chunking is optimized for call sets of 10,000 variants; use --chunk-size 5000 for whole-genome analyses to maintain predictable memory usage.

## Examples

### Inspect breakpoints from a single-sample VCF file
**Args:** input.vcf.gz --output breakpoint_report.tsv --format tsv
**Explanation:** This reads all structural variants from the gzipped VCF, extracts breakpoint coordinates and types, and writes a tab-delimited report suitable for manual review or spreadsheet analysis.

### Cross-validate breakpoints against aligned reads
**Args:** input.vcf.gz -a alignment.bam --output validated_breakpoints.json --format json
**Explanation:** The tool scans the BAM file for split-read and discordant pair evidence at each reported breakpoint, embedding validation metrics in the JSON output for downstream filtering.

### Filter to large deletions only with quality thresholds
**Args:** input.vcf.gz --output large_dels.tsv --format tsv --sv-type DEL --min-svlen 1000 --min-qual 30
**Explanation:** Specifying SV type and size filters reduces the output to deletions longer than 1kb with GATK or DeepVariant quality scores above 30, focusing analysis on high-confidence structural events.

### Export breakpoints in VCF format with annotations
**Args:** input.vcf.gz --output annotated.sv.vcf --format vcf-out --add-info HOMLEN,CIPOS,SU
**Explanation:** This generates a new VCF containing the original variants enriched with additional INFO field annotations specified by the user, enabling seamless integration into standard variant calling workflows.

### Analyze breakpoints using zero-based BED coordinates
**Args:** input.bed -g hg38.fa --output bed_report.tsv --format tsv --zero-based
**Explanation:** When the input uses zero-based half-open intervals common in BED files, this flag ensures correct coordinate translation to the reporting format without introducing off-by-one errors at every breakpoint.