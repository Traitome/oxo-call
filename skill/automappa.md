---
name: automappa
category: genomics/read_mapping
description: A tool for automated read-to-reference mapping with built-in quality filtering and sam/bam output
tags: [mapping, alignment, genomics, reads, sam, bam, automated]
author: AI-generated
source_url: https://github.com/automappa/automappa
---

## Concepts

- Automappa accepts FASTQ or FASTA input files and aligns reads to a reference genome index built with automappa-build, producing SAM or BAM output depending on the --outfmt flag.
- The tool automatically detects read type (single-end or paired-end) based on input file naming conventions: use _1.fastq and _2.fastq for paired-end reads.
- Quality filtering thresholds control which aligned reads are retained in the output; use --min-qual to set the minimum mapping quality (MAPQ) score.
- Output is written to stdout by default, but redirect to a file using standard shell redirection (>) to preserve the output for downstream tools like samtools.
- The internal indexing system uses a suffix array or FM-index internally, meaning alignment speed scales sub-linearly with reference size.

## Pitfalls

- Using the wrong reference index (e.g., built from a different species) produces meaningless alignments even if the tool completes without error; always verify the index matches your input reads.
- Forgetting to specify --outfmt when you need BAM output results in SAM format, which is larger and slower for large datasets; BAM is recommended for production pipelines.
- Setting --min-qual too high (e.g., above 60) can discard valid alignments on some datasets, especially with shorter reads or higher error rates; adjust based on read length.
- Running on insufficient memory causes the process to be killed by the OS; ensure available RAM exceeds the reference genome size plus the read input size.

## Examples

### Map single-end reads to a reference index

**Args:** -i ref_index.fmi -q reads.fq --outfmt sam

**Explanation:** This aligns single-end reads from reads.fq to the prebuilt index ref_index.fmi and outputs alignments in SAM format.

### Map paired-end reads and save to a file

**Args:** -i ref_index.fmi -1 read_1.fq -2 read_2.fq --outfmt bam > output.bam

**Explanation:** This aligns paired-end reads using both _1 and _2 files, outputs BAM format, and redirects to output.bam for downstream processing.

### Apply quality filtering to keep high-quality alignments

**Args:** -i ref_index.fmi -q reads.fq --outfmt bam --min-qual 20

**Explanation:** This aligns reads and filters out alignments with mapping quality below 20, keeping higher-confidence mappings in the output.

### Reduce alignment runtime on large datasets using threads

**Args:** -i ref_index.fmi -q reads.fq --outfmt sam -t 8

**Explanation:** This runs the alignment with 8 threads to speed up processing on multi-core systems, which is beneficial for whole-genome datasets.

### Output alignments sorted by coordinate

**Args:** -i ref_index.fmi -q reads.fq --outfmt bam --sort-coord

**Explanation:** This produces BAM output sorted by genomic coordinate rather than by input order, suitable for direct use with variant callers like freebayes.