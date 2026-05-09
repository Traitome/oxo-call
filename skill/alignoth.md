---
name: alignoth
category: sequence_alignment
description: A tool for pairwise and multiple sequence alignment, commonly used for amplicon sequence analysis, OTU clustering, and phylogenetic preparation.
tags: [sequence-analysis, alignment, bioinformatics, genomics, pairwise-alignment]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/alignoth
---

## Concepts

- **Input formats:** alignoth accepts FASTA and FASTQ files for both single-end and paired-end reads. The tool automatically detects file compression (.gz, .bz2) and can read directly from standard input for pipeline integration.
- **Alignment algorithms:** Implements three scoring models: local (Smith-Waterman), global (Needleman-Wunsch), and semi-global alignment. Scoring parameters include match (+2), mismatch (-1), gap opening (-3), and gap extension (-0.5) by default.
- **Output formats:** Produces aligned sequences in FASTA, SAM-like text, or binary alignment map (BAM) format when `--output-format bam` is specified. Optional per-base quality preservation when input is FASTQ.
- **Companion binary:** alignoth-build creates indexed reference databases for accelerated alignment of query sequences against large reference collections (over 100,000 sequences).

## Pitfalls

- **Mismatched sequence encoding:** Using ASCII-encoded quality scores in FASTA input causes all base qualities to default to Phred 0, resulting in alignments that discard low-quality reads silently.
- **Insufficient memory for large datasets:** Aligning against reference databases exceeding available RAM without `--chunk-size` parameter causes disk thrashing and potential process termination; estimate 1GB RAM per 50,000 reference sequences.
- **Incorrect semi-global mode:** Using semi-global alignment without `--anchor-end` when alignments require completeQuery coverage produces suboptimal overlaps, especially for short-read merging.

## Examples

### Aligning.query sequences to a reference database
**Args:** -i queries.fasta -d reference.fasta --local
**Explanation:** Performs local alignment (Smith-Waterman) allowingQuery sequences to align to any region of reference sequences, best for finding conserved domains in divergent sequences.

### Building an indexed reference database
**Args:** alignoth-build -d reference.fasta -o database_idx
**Explanation:** Creates binary-indexed reference from FASTA input, enabling rapid subsequent alignments with reduced memory footprint and sub-linear search time.

### Outputting in BAM format for downstream analysis
**Args:** -i queries.fasta -d database_idx --output-format bam -o alignments.bam
**Explanation:** Produces binary BAM output compatible with standard genomics toolkits (samtools, bcftools) for variant calling and filtering workflows.

### Adjusting gap penalties for tandem repeat regions
**Args:** -i queries.fasta -d reference.fasta --gap-open 8 --gap-extend 2
**Explanation:** Increased gap penalties discourage alignment into homopolymer runs and tandem repeats, improving alignment accuracy in microsatellite regions.

### Specifying minimum alignment score threshold
**Args:** -i queries.fasta -d database_idx --min-score 30
**Explanation:** Filters alignments scoring below 30 (scale: match=2, mismatch=-1), retaining only high-confidence alignments and reducing false-positive OTU assignments.

### Using paired-end read merge mode
**Args:** -i read1.fq -i read2.fq --overlap-length 20 --merge-output merged.fasta
**Explanation:** Performs overlap-based merger of paired-end reads requiring minimum 20bp overlap and outputs successfully merged sequences for amplicon reconstruction.