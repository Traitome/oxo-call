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
- OPTICAL_DUPLICATE_PIXEL_DISTANCE sets the max distance for optical duplicates (default 100 for patterned flowcells).
- TAGGING_POLICY controls DT tag: All (default), OpticalOnly, or DontTag.
- REMOVE_DUPLICATES=true removes duplicates instead of just marking them.
- REMOVE_SEQUENCING_DUPLICATES=true removes only optical/sequencing duplicates.
- ASSUME_SORT_ORDER skips sorting validation when input is already sorted.
- DUPLICATE_SCORING_STRATEGY chooses which read to keep: SUM_OF_BASE_QUALITIES (default) or TOTAL_MAPPED_REFERENCE_LENGTH.

## Pitfalls
- Picard ARGS must start with a tool name subcommand (MarkDuplicates, AddOrReplaceReadGroups, SortSam, ValidateSamFile, CollectAlignmentSummaryMetrics, CollectInsertSizeMetrics, CreateSequenceDictionary, CollectGcBiasMetrics, CollectInsertSizeMetrics, CollectQualityYieldMetrics, EstimateLibraryComplexity, FastqToSam, SamToFastq, MergeSamFiles) — never with flags like -I, -O, -M. The tool name ALWAYS comes first.
- MarkDuplicates on an unsorted BAM will fail — always sort first with samtools sort or Picard SortSam.
- MarkDuplicates marks but does NOT remove duplicates by default; add REMOVE_DUPLICATES=true only if required.
- The METRICS_FILE (-M) argument is mandatory for MarkDuplicates — omitting it causes an error.
- Picard tools use TMP_DIR for large temporary files — set TMP_DIR to a directory with sufficient space.
- Java heap size must be set with -Xmx for large files: java -Xmx8g -jar picard.jar; or use JAVA_OPTS env var.
- CREATE_INDEX=true requires SORT_ORDER=coordinate — it only works on coordinate-sorted BAM files.
- VALIDATION_STRINGENCY=LENIENT silences non-critical warnings; VALIDATION_STRINGENCY=SILENT suppresses all validation.
- OPTICAL_DUPLICATE_PIXEL_DISTANCE default 100 is for patterned flowcells; use 2500 for non-patterned.
- TAGGING_POLICY=All adds DT tag to all duplicates; increases file size.
- REMOVE_DUPLICATES=true discards reads permanently; consider marking first for QC.
- ASSUME_SORT_ORDER without verification may cause errors if input is not actually sorted.

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

### mark and remove duplicates
**Args:** `MarkDuplicates -I sorted.bam -O dedup.bam -M metrics.txt --REMOVE_DUPLICATES true --CREATE_INDEX true`
**Explanation:** REMOVE_DUPLICATES=true permanently removes duplicates instead of just marking

### mark duplicates with optical duplicate detection for non-patterned flowcells
**Args:** `MarkDuplicates -I sorted.bam -O marked.bam -M metrics.txt --OPTICAL_DUPLICATE_PIXEL_DISTANCE 2500 --CREATE_INDEX true`
**Explanation:** OPTICAL_DUPLICATE_PIXEL_DISTANCE 2500 for non-patterned flowcells (e.g., HiSeq 2000)

### mark duplicates with DT tag for all duplicates
**Args:** `MarkDuplicates -I sorted.bam -O marked.bam -M metrics.txt --TAGGING_POLICY All --CREATE_INDEX true`
**Explanation:** TAGGING_POLICY All adds DT tag to all duplicate reads; OpticalOnly for optical only

### remove only sequencing (optical) duplicates
**Args:** `MarkDuplicates -I sorted.bam -O no_optical.bam -M metrics.txt --REMOVE_SEQUENCING_DUPLICATES true --CREATE_INDEX true`
**Explanation:** REMOVE_SEQUENCING_DUPLICATES removes only optical/sequencing duplicates, keeps PCR duplicates

### mark duplicates assuming sorted input
**Args:** `MarkDuplicates -I sorted.bam -O marked.bam -M metrics.txt --ASSUME_SORT_ORDER coordinate --CREATE_INDEX true`
**Explanation:** ASSUME_SORT_ORDER skips sorting validation; use when certain input is sorted

### merge multiple BAM files
**Args:** `MergeSamFiles -I input1.bam -I input2.bam -I input3.bam -O merged.bam --CREATE_INDEX true`
**Explanation:** MergeSamFiles combines multiple BAM files; use multiple -I for each input

### convert BAM to FASTQ
**Args:** `SamToFastq -I input.bam -FASTQ output_R1.fastq -SECOND_END_FASTQ output_R2.fastq`
**Explanation:** SamToFastq extracts paired FASTQ from BAM; use for re-alignment workflows
