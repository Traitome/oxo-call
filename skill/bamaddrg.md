---
name: bamaddrg
category: readsmapping
description: Adds read group tags to BAM/SAM alignment records, enabling proper sample identification and duplicate marking in downstream pipelines
tags: [bam, sam, read-group, rg, alignment, duplicates, gatk]
author: AI-generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- **Read group tags**: The tool injects RG (read group) metadata into BAM/SAM alignment records. Each read group must have a unique ID, and optional fields include SM (sample name), LB (library), PL (platform), and PU (platform unit). These tags are essential for GATK workflows and duplicate marking tools like Picard MarkDuplicates.
- **Multiple read group support**: Using multiple `-R` flags allows adding different read groups to different sets of reads, or assigning reads to specific groups based on read name patterns. The reads are matched to read groups by their @RG ID in the read name or by explicit assignment.
- **I/O flexibility**: Input is read from stdin when no input file is specified, and output is written to stdout. This enables piping in BEDTools/SAMtools pipelines. The tool accepts both BAM and SAM formats, auto-detected by file extension.
- **Assignment methods**: Reads can be assigned to read groups either by using the `-r` option to specify a single RG ID for all reads, or by including RG IDs in read names (e.g., `@RG_ID:readname`), or through pattern matching with the `-p` option.

## Pitfalls

- **Duplicate read group IDs**: Using the same ID string for multiple `-R` flags causes the tool to overwrite rather than add, leading to lost read groups. Each read group must have a unique ID string. Consequence: downstream tools cannot distinguish samples, resulting in merged genotypes and incorrect variant calls.
- **Missing required -R flag**: Running bamaddrg without specifying at least one read group with `-R` results in no modifications to the file. Consequence: downstream duplicate marking fails because RG tags are required by MarkDuplicates to properly track duplicate sources per sample.
- **Mismatched read name format**: If expecting automatic assignment via read names, the read names must contain an @RG ID field in their header. Consequence: reads remain untagged, and GATK tools will error with "RG" missing warnings.
- **Output format mismatch**: Outputting to a BAM file without the `-b` flag when working with piped input can cause format errors. Consequence: SAM output to a .bam file corrupts or is unreadable by downstream tools.

## Examples

### Add a single read group tag to all reads in a BAM file
**Args:** `-R "ID:lane1" -R "SM:sample1" -b input.bam`
**Explanation:** The `-R` flags specify the read group ID and sample name, and `-b` outputs BAM format to preserve the alignment binary encoding.

### Tag reads using a specific read group ID for all alignments
**Args:** `-r "rg1" -R "ID:rg1" -R "SM:NA12878" -b input.bam`
**Explanation:** The `-r` flag assigns all reads to read group "rg1", while the `-R` defines what "rg1" contains. This is required when reads lack embedded RG IDs.

### Add read groups via stdin from a piped BAM stream
**Args:** `-R "ID:flowcell1" -R "SM:experimentA" -R "LB:lib1"`
**Explanation:** Reads from stdin, allowing integration in pipelines like `samtools view -b in.bam | bamaddrg -R "ID:flowcell1" -R "SM:experimentA" -R "LB:lib1" | samtools sort -o sorted.bam -`.

### Add multiple distinct read groups for different read ID patterns
**Args:** `-p ".*_L1_" -R "ID:lane1" -R "SM:sample1" -p ".*_L2_" -R "ID:lane2" -R "SM:sample1" -b input.bam`
**Explanation:** The `-p` option uses regex patterns to match read names and apply the following read groups to matching reads, enabling lane-based separation.

### Specify platform and platform unit metadata for GATK compliance
**Args:** `-R "ID:lane1" -R "SM:tumor" -R "LB:libraryA" -R "PL:ILLUMINA" -R "PU:flowcellA.lane1" -b tumor.bam`
**Explanation:** Adding PL (platform) and PU (platform unit) fields satisfies GATK requirements for proper read批 quality evaluation and duplicate tracking per flowcell.