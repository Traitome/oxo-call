---
name: picard
category: alignment
description: "Java toolkit for NGS data manipulation: duplicate marking, metrics collection, format conversion, and more"
tags: [bam, sam, duplicates, metrics, illumina, ngs, gatk, markduplicates]
author: oxo-call built-in
source_url: "https://broadinstitute.github.io/picard/"
---

## Concepts

- Picard is invoked as 'picard <ToolName>' (conda) or 'java -jar picard.jar <ToolName>'; most tools accept -I input -O output.
- MarkDuplicates is the most common Picard tool; it marks (not removes by default) PCR/optical duplicates for downstream variant calling.
- MarkDuplicates requires a name-sorted or coordinate-sorted input; outputs a new BAM and a metrics file (-M).
- Read groups are required by many GATK tools — add them with AddOrReplaceReadGroups if missing from the BAM.
- SortSam can sort a BAM coordinate or name order (SORT_ORDER=coordinate or queryname); equivalent to samtools sort.
- ValidateSamFile is essential before GATK — it checks for common SAM/BAM format errors.
- CREATE_INDEX=true automatically creates a .bai index alongside the output BAM (equivalent to running samtools index).
- CollectAlignmentSummaryMetrics and CollectInsertSizeMetrics provide important QC statistics for WGS/WES data.

## Pitfalls

- MarkDuplicates on an unsorted BAM will fail — always sort first with samtools sort or Picard SortSam.
- MarkDuplicates marks but does NOT remove duplicates by default; add REMOVE_DUPLICATES=true only if required.
- The METRICS_FILE (-M) argument is mandatory for MarkDuplicates — omitting it causes an error.
- Picard tools use TMP_DIR for large temporary files — set TMP_DIR to a directory with sufficient space.
- Java heap size must be set with -Xmx for large files: java -Xmx8g -jar picard.jar; or use JAVA_OPTS env var.
- CREATE_INDEX=true requires SORT_ORDER=coordinate — it only works on coordinate-sorted BAM files.
- VALIDATION_STRINGENCY=LENIENT silences non-critical warnings; VALIDATION_STRINGENCY=SILENT suppresses all validation.

## Examples

### mark PCR duplicates in a sorted BAM file
**Args:** `MarkDuplicates -I sorted.bam -O marked_dup.bam -M markdup_metrics.txt --CREATE_INDEX true`
**Explanation:** -I input sorted BAM; -O marked output BAM; -M required metrics file; CREATE_INDEX creates .bai index

### add or replace read groups in a BAM file
**Args:** `AddOrReplaceReadGroups -I input.bam -O rg_added.bam --RGLB lib1 --RGPL ILLUMINA --RGPU unit1 --RGSM sample1 --CREATE_INDEX true`
**Explanation:** RGLB=library, RGPL=platform, RGPU=platform unit, RGSM=sample name — required by GATK tools

### sort a BAM file by coordinate using Picard
**Args:** `SortSam -I input.bam -O sorted.bam --SORT_ORDER coordinate --CREATE_INDEX true`
**Explanation:** SORT_ORDER coordinate; CREATE_INDEX automatically creates .bai file

### collect alignment summary metrics from a BAM file
**Args:** `CollectAlignmentSummaryMetrics -I aligned.bam -O alignment_metrics.txt -R reference.fa`
**Explanation:** outputs per-category alignment statistics including mapping rate, mismatch rate, and paired-end metrics

### collect insert size distribution metrics from paired-end BAM
**Args:** `CollectInsertSizeMetrics -I sorted.bam -O insert_size_metrics.txt -H insert_size_histogram.pdf`
**Explanation:** insert size histogram useful for QC and detecting unexpected library preparation issues

### convert SAM to sorted BAM with index
**Args:** `SortSam -I input.sam -O sorted.bam --SORT_ORDER coordinate --CREATE_INDEX true`
**Explanation:** Picard SortSam handles SAM-to-BAM conversion and sorting in one step

### validate a BAM file for GATK compatibility
**Args:** `ValidateSamFile -I input.bam -O validation_report.txt --MODE SUMMARY`
**Explanation:** MODE SUMMARY lists error types and counts; use MODE VERBOSE for all error locations
