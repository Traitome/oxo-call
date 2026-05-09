---
name: acms
category: sequence_alignment
description: A read alignment tool for mapping sequencing reads to reference genomes with specific support for amplified consensus sequences and multi-region targeting. Handles SAM/BAM input and output with flexible filtering options.
tags: [alignment, sequencing, genomics,reads, mapping]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/acms
---

## Concepts

- **Input formats**: acms accepts FASTQ, FASTA, and SAM/BAM formats for query sequences. References must be in FASTA format. Multi-region targets can be specified in a BED file.
- **Output formats**: Primary output is SAM format by default; use `--bam` to output BAM. Alignment summaries are written to stderr; use `-o` to write alignments to a specific file.
- **Scoring model**: Uses a modified Needleman-Wunsch algorithm with match reward (+2), mismatch penalty (-3), and gap opening/extension penalties (-5/-2). These can be adjusted via `--match`, `--mismatch`, `--gap-open`, and `--gap-extend`.
- **Multi-region targeting**: When a BED file is provided via `--regions`, acms restricts alignment to those genomic intervals and reports whether reads map to expected target regions.
- **Companion binary**: Use `acms-build` (not acms) to index reference genomes first. The index must be built before alignment: `acms-build reference.fa reference_index`.

## Pitfalls

- **Forgetting to build an index**: Running acms without a pre-built index causes the tool to fail with an unclear error. Always run `acms-build` first on the reference genome.
- **Mismatched read types**: Supplying single-end reads when expecting paired-end data (or vice versa) results in only half the expected alignments. Use `--paired` or `--single` explicitly to avoid this.
- **Incorrect encoding**: Using Phred+64 quality scores (common in older Illumina data) without specifying `--quality64` causes alignment failures or silent quality misreading. Verify your quality score encoding before running.
- **Ignoring strand specificity**: By default, acms aligns to both strands. For stranded RNA-seq or CRISPR amplicon data, use `--fr` or `--rf` to specify expected orientation.
- **Output file overwrite**: Specifying an output file that already exists does not prompt for confirmation—it silently overwrites. Use `--append` to append to existing files.

## Examples

### Align single-end reads to a reference genome
**Args:** `--single -x reference_index reads.fq -o alignments.sam`
**Explanation:** Maps single-end reads from reads.fq to the pre-built reference_index and writes SAM output to alignments.sam.

### Align paired-end reads with specific fragment size
**Args:** `--paired -1 read1.fq -2 read2.fq -x reference_index --min-isize 100 --max-isize 500 -o paired_aligments.sam`
**Explanation:** Aligns paired-end reads with an expected inner fragment size between 100 and 500 bp, filtering out improperly paired alignments.

### Output BAM format directly
**Args:** `--bam -x reference_index reads.fq -o output.bam`
**Explanation:** Writes output in BAM (binary) format instead of the default SAM text format, reducing file size and enabling efficient downstream processing.

### Restrict alignment to specific genomic regions
**Args:** `--regions targets.bed -x reference_index reads.fq -o targeted_aligments.sam`
**Explanation:** Limits alignment search to genomic intervals defined in targets.bed, useful for amplicon sequencing or panel enrichment analysis.

### Adjust scoring parameters for higher stringency
**Args:** `--match 3 --mismatch -5 --gap-open -8 --gap-extend -4 -x reference_index reads.fq -o strict_aligments.sam`
**Explanation:** Increases alignment stringency by raising match reward and gap penalties, reducing false-positive alignments at the cost of sensitivity.

### Specify PHRED+64 quality encoding
**Args:** `--quality64 -x reference_index illumina_reads.fq -o output.sam`
**Explanation:** Tells acms to interpret quality scores as PHRED+64 encoding (standard for Illumina 1.7 and earlier), preventing quality score misreading.

### Align to reverse strand only for stranded library preparation
**Args:** `--rf -x reference_index reads.fq -o rf_aligments.sam`
**Explanation:** Aligns reads assuming they originated from the reverse strand (typical for dUTP-based stranded RNA-seq), filtering out antisense alignments.

### Append alignments to an existing SAM file
**Args:** `--append -x reference_index new_reads.fq -o combined_alignments.sam`
**Explanation:** Adds new alignments to the end of an existing SAM file rather than overwriting, useful for pooling multiple samples.

### Build an index for the reference genome
**Args:** reference.fasta acms_index`
**Explanation:** Creates index files from reference.fasta (the first argument without a flag) to the prefix acms_index, required before running alignment commands.