---
name: cap-mirseq
category: Bioinformatics - miRNA Analysis
description: A command-line toolkit for microRNA (miRNA) sequencing analysis. Processes raw small RNA-seq data to detect, quantify, and profile miRNA expression from FASTQ inputs. Includes companion tools for building custom miRNA reference databases.
tags:
  - miRNA
  - small RNA-seq
  - microRNA
  - expression quantification
  - alignment
  - NGS
  - bioinformatics
author: AI-generated
source_url: https://github.com/cap-mirseq/cap-mirseq
---

## Concepts

- **Input/Output Data Model**: The tool operates on FASTQ files containing small RNA sequencing reads (typically 18-30 nucleotides). Output includes miRNA expression tables (RAW or normalized counts), alignment summaries in SAM/BAM format, and optional variant call format (VCF) files for miRNA editing detection.

- **Reference Database Building**: The companion binary `cap-mirseq-build` constructs indexed reference databases from miRBase GFF3 annotations and mature miRNA sequences. The index supports rapid read alignment and enables detection of both canonical miRNAs and isomiRs.

- **Normalization Strategies**: Expression quantification supports multiple normalization methods including TPM (Transcripts Per Million), RPM (Reads Per Million), and exact count tables. The tool applies length normalization appropriate for miRNA length variation.

- **Adapter and Quality Trimming**: The pipeline includes built-in trimming for common small RNA-seq adapters (Illumina, NEBNext) and quality filtering using a sliding window approach with configurable minimum quality thresholds.

- **IsomiR Detection**: The tool reports miRNA sequence variants (isomiRs) including 3' end additions, 5' end trimming, and internal nucleotide substitutions, distinguishing them from canonical miRNA sequences.

## Pitfalls

- **Failing to Specify Read Length Range**: Processing all read lengths without filtering to the typical miRNA size range (18-30 nt) will inflate false positive detections from fragmented mRNA or rRNA fragments, reducing detection specificity and increasing noise in expression estimates.

- **Using Outdated Reference Databases**: Building indices with older miRBase releases missing recently discovered miRNAs will cause those miRNAs to be unalignable, resulting in missing detections for novel miRNAs present in newer organism-specific databases.

- **Mismatching strandedness options**: Specifying the wrong strandedness orientation (--fr vs --rf) will cause reads to align to the antisense strand, producing completely inverted expression profiles with most counts mapping to the wrong strand.

- **Insufficient Memory for Large Indexes**: Building databases without adequate RAM (typically 8GB+ for vertebrate miRBase) causes index construction to fail or produce corrupt indices leading to alignment crashes during runtime.

- **Ignoring Modifications in Variant Calling**: Not enabling the --detect-edits or --allow-modifications flags will cause the tool to discard reads with A-to-I editing or 3' nucleotide additions that are biologically relevant post-transcriptional modifications.

## Examples

### Build a miRNA reference database from miRBase files

**Args:** --build -g annotations.gff3 -f mature_seqs.fa -o miRBase_v22_index

**Explanation:** Constructs an indexed reference database from miRBase GFF3 annotations and FASTA sequences for use in subsequent alignment steps.

### Align raw small RNA-seq FASTQ files to a reference

**Args:** -i sample1.fastq.gz -r miRBase_v22_index -o aligned_output.sam --threads 8

**Explanation:** Aligns a compressed FASTQ file against the pre-built miRNA index using 8 CPU threads for parallel processing.

### Quantify miRNA expression with TPM normalization

**Args:** -a aligned_output.sam --normalize TPM -e expression_counts.txt

**Explanation:** Converts alignment hits to expression values using TPM normalization, producing a count table suitable for downstream differential expression analysis.

### Filter reads by quality and length before alignment

**Args:** -i raw_reads.fastq.gz --min-quality 20 --length-min 18 --length-max 30 -o filtered_reads.fq

**Explanation:** Applies quality filtering (minimum Phred score 20) and size selection (18-30 nt) to remove low-quality and off-target reads prior to alignment.

### Detect isomiRs and report sequence variants

**Args:** -a aligned_output.sam --detect-isomirs --output-variants variant_calls.txt

**Explanation:** Analyzes aligned reads to identify isomiR variants including 5' and 3' end modifications, exporting detailed variant information to a text file.

### Run complete pipeline with default parameters

**Args:** -i sample.fastq.gz --build-ref miRBase_index --output-dir results/ --summary stats.json

**Explanation:** Executes the full analysis pipeline (trimming, alignment, quantification) in one command, writing results to the specified output directory with a JSON summary.