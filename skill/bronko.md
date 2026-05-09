---
name: bronko
category: sequence_analysis/alignment
description: A short-read alignment tool for mapping sequencing reads to reference genomes with support for paired-end data and variant calling workflows.
tags: [alignment, sequencing, genomics, variant-calling,short-reads]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bronko
---

## Concepts

- **Input Formats**: bronko accepts FASTQ (single-end or paired-end) and can use a reference genome in FASTA format. The tool performs gapped alignment with support for mismatches, insertions, and deletions.
- **Output Formats**: Generates SAM (Sequence Alignment/Map) format by default, with optional BAM output when paired with samtools for sorting and indexing. Output includes mapping quality scores, CIGAR strings, and optional per-base quality values.
- **Indexing**: Requires a pre-built index of the reference genome created using the companion `bronko-build` binary. Index files store compressed reference sequences and lookup tables for fast alignment.
- **Alignment Modes**: Supports local mode (Smith-Waterman) for soft-clipping and end-to-end mode for full-length alignment. Paired-end reads are processed with configurable insert size constraints and mate pairing validation.
- **paired-end Processing**: When processing paired-end data, bronko uses both read orientation (FR/ RF) and fragment length distributions to improve mapping accuracy and detect anomalous pairs.

## Pitfalls

- **Missing Index**: Running bronko without first generating an index using `bronko-build` causes immediate failure with a "reference not indexed" error, wasting computational time and requiring a restart.
- **Mismatched Read Types**: Providing single-end FASTQ files while specifying paired-end flags (e.g., `-1` and `-2` for read files) results in silent alignment failures or all reads mapping as unmapped, producing incorrect results.
- **Fragment Length Misspecification**: Setting incorrect minimum or maximum insert sizes (`-X`/`-I` flags) for paired-end libraries can cause valid alignments to be rejected, reducing mapping rates and creating biased variant calls.
- **Insufficient Memory for Large Genomes**: Aligning to large reference genomes (e.g., human) without adequate RAM causes excessive disk swapping and extreme slowdowns; ensure available memory exceeds index size by a minimum of 2x.
- **Output File Overwrites**: Specifying an existing SAM/BAM file as output without confirming overwrite permissions causes bronko to fail with a file access error, requiring manual deletion or alternative output path selection.

## Examples

### Build an index for a bacterial reference genome
**Args:** -p ecoli_ref bronko-build -c 2
**Explanation:** Creates a 2-threaded index named "ecoli_ref" from the default input FASTA for fast alignment queries.

### Align single-end reads to an indexed reference
**Args:** -1 reads.fq -S output.sam -t 4
**Explanation:** Aligns single-end FASTQ using 4 threads, writes result to SAM file in the default end-to-end mode.

### Perform local alignment with reporting of all valid alignments
**Args:** -1 reads.fq -a -k 2 -S output.sam
**Explanation:** Uses local alignment mode to find up to 2 equally best alignments per read for downstream variant analysis.

### Align paired-end reads with strict insert size bounds
**Args:** -1 read1.fq -2 read2.fq -X 500 -I 100 -S output.sam -t 8
**Explanation:** Constrains fragment size between 100-500bp and aligns using 8 threads for higher throughput.

### Convert output to sorted BAM for variant calling
**Args:** -1 reads.fq -S output.sam && samtools sort -o output.bam output.sam
**Explanation:** Produces SAM then sorts to BAM for compatibility with GATK or other variant callers requiring indexed binary input.

### Align with custom seed length for speed/accuracy tradeoff
**Args:** -1 reads.fq -S output.sam -l 20 -t 4
**Explanation:** Uses 20bp seed length with 4 threads, faster but potentially less accurate for reads with many polymorphisms.

### Suppress unaligned reads from output to reduce file size
**Args:** -1 reads.fq -S output.sam --no-unaligned
**Explanation:** Includes only mapped reads in output SAM, useful when all records are expected to align or downstream tools require only aligned sequences.
---