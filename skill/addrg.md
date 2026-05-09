---
name: addrg
category: BAM/SAM Manipulation
description: Adds read group information to unmapped BAM files and optionally generates synthetic barcoded reads. Essential for preparing Bionano data for downstream genomics workflows.
tags: [bam, read-group, bionano, sbotools, genomics, sam-manipulation]
author: AI-generated
source_ul: https://docs.sevenbridges.com/reference/addrg
---

## Concepts

- Read groups are stored in the @RG SAM header tag and include identifiers such as ID, SM (sample name), LB (library), PL (platform), and PU (flowcell/lane). These fields enable downstream tools to trace reads to their source and batch.
- `addrg` processes unmapped BAM files and injects or replaces read group metadata at the header level, ensuring compatibility with GATK and other variant calling pipelines that require read group annotations.
- The tool can generate synthetic reads that embed barcode labels, supporting Bionano-specific workflows where additional labeled reads are needed for hybrid assembly or validation.
- Output is written as an unmapped BAM file with the augmented @RG header, preserving all original alignment records while adding the requested group information.

## Pitfalls

- Reusing a read group ID (RGID) that already exists in the input file produces duplicate @RG header entries, which causes GATK to error with "Lexicographically smaller" duringCombineGVCFs and requires manual cleanup.
- Omitting the platform (PL) field results in missing metadata, making multi-platform cohort analyses incomplete and preventing proper platform-stratified QC reporting.
- Providing mismatched sample names (SM) between the command-line argument and the actual specimen label embedded in the reads can silently pass validation but corrupt downstream sample identity in joint variant calling, leading to incorrect genotype calls.
- Running `addrg` on an already-mapped BAM file without unmapping records first may corrupt alignment coordinates; the tool is designed for unmapped input only.
- Forgetting to increment the RGID for each lane when processing multiplexed data results in a single read group spanning multiple flow cells, which hampers lane-level quality control.

## Examples

### Add a read group to an unmapped BAM with all required fields
**Args:** `-i input.bam -o output.bam --rg-id RGLANE001 --rg-sm SAMPLE_001 --rg-lb LIB001 --rg-pl Sequel --rg-pu FLOWCELL001`
**Explanation:** This command annotates the BAM with a complete read group specification, enabling compatibility with GATK HaplotypeCaller for variant discovery.

### Add a read group with a custom sequencing center field
**Args:** `-i input.bam -o output.bam --rg-id RGLANE002 --rg-sm SAMPLE_002 --rg-cn BroadInstitute --rg-pl HiSeq --rg-pu LANE2`
**Explanation:** The `--rg-cn` field documents the sequencing center, which is useful for audit trails and reproducibility documentation in collaborative projects.

### Generate synthetic barcoded reads alongside read group injection
**Args:** `-i input.bam -o output.bam --rg-id RGBCODE --rg-sm SAMPLE_BC --generate-reads --rg-pl Sequel`
**Explanation:** The `--generate-reads` flag produces additional synthetic labeled reads that embed barcode information, supporting Bionano hybrid assembly validation workflows.

### Add read group to multiple lane BAMs for later merging
**Args:** `-i lane1.bam -o lane1_rg.bam --rg-id RGLANE1 --rg-sm COHORT_A --rg-pl Sequel --rg-pu FLOWCELL_A --rg-lb LIB_A`
**Explanation:** Assigning lane-specific RGID values while sharing the sample name allows `samtools merge` to later combine BAMs correctly with preserved read group metadata.

### Add minimal read group information for quick downstream compatibility
**Args:** `-i input.bam -o output.bam --rg-id QUICKRG --rg-sm QUICKSM`
**Explanation:** Providing only the mandatory ID and SM fields is useful in exploratory pipelines where full metadata will be added later in a validation step.