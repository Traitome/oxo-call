---
name: abismal
category: aligner
description: A fast and memory-efficient short-read aligner based on the Burrows-Wheeler Transform. Maps Illumina short reads to a reference genome with high accuracy and speed.
tags: [alignment, short-read, BWT, FM-index, SAM, FASTQ]
author: AI-generated
source_url: https://github.com/bioinform/abismal
---

## Concepts

- **Input formats**: abismal accepts FASTQ files (single or paired-end) as input and requires a reference genome indexed with the companion `abismal-index` tool. The reference must be provided in FASTA format.
- **Output formats**: By default, abismal produces alignments in SAM (Sequence Alignment/Map) format. Use the `-b` flag to output BAM format directly, which saves disk space but requires more CPU.
- **Indexing**: The companion tool `abismal-index` builds the FM-index from a FASTA reference. This index is loaded into memory at runtime and determines both speed and memory usage—the larger the index, the more memory consumed.
- **Paired-end alignment**: When provided with two FASTQ files (via `-1` and `-2`), abismal automatically performs paired-end alignment and sets the `T` and `2` flags in the SAM output to indicate proper pairs.

## Pitfalls

- **Mismatched read groups**: If the `@RG` read group header in the FASTQ does not match what was specified at alignment time, downstream tools like GATK will fail to recognize samples, causing analysis pipelines to break silently.
- **Forgetting to index the reference**: Running abismal without first building an index with `abismal-index` produces a cryptic error about "missing index file" and wastes computation time.
- **Insufficient seed length**: Using `-k` too small (e.g., 10) dramatically increases runtime and false positives because the algorithm searches for more potential alignment sites; too large (e.g., 50) may miss valid alignments in repetitive regions.
- **Mixed quality score encodings**: Providing FASTQ files with Phred+64 encoding (old Illumina quality) while the tool expects Phred+33 produces incorrect alignment quality scores and may cause downstream variant calling to miss true variants.

## Examples

### Map single-end reads to an indexed reference
**Args:** -t 8 ref.fa reads.fq -o alignments.sam
**Explanation:** This runs abismal with 8 threads on a pre-indexed reference, writing output to SAM format.

### Map paired-end reads with automatic pair detection
**Args:** -t 16 -1 reads1.fq -2 reads2.fq ref.fa -o paired_alignments.sam
**Explanation:** This alignment automatically identifies proper read pairs and marks them with the SAM flag 0x1 (pair) and 0x2 (properly paired).

### Use a custom seed length for repetitive genomes
**Args:** -k 30 -t 4 ref.fa reads.fq -o alignment.sam
**Explanation:** Using a longer seed (30bp) reduces runtime in highly repetitive genomes but may miss alignments in segmental duplications.

### Output directly in BAM format
**Args:** -b -t 8 ref.fa reads.fq -o alignments.bam
**Explanation:** Writing BAM directly compresses output and avoids a separate samtools conversion step, saving storage and time.

### Apply read group information for GATK compatibility
**Args:** -r "@RG\tID:sample1\tSM:sample1\tPL:ILLUMINA" -t 8 ref.fa reads.fq -o sample1.sam
**Explanation:** Adding the read group ensures GATK and other tools correctly associate reads with the sample and platform.