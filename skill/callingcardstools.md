---
name: callingcardstools
category: Genomics / Transposon Insertion Mapping
description: A molecular toolkit for processing CallingCards data, which maps transposon insertion sites in eukaryotic genomes using a yeast-enhanced reporter system. Provides QC, alignment, deconvolution, and count aggregation workflows for paired-end sequencing data.
tags:
  - callingcards
  - transposon
  - insertion-mapping
  - paired-end
  - barcoded-PCR
  - enhancer-trap
  - sleeping-beauty
  - wiggle
  - bowtie
  - genomics
author: AI-Generated
source_url: https://github.com/rpolicastro/callingcardstools
---

## Concepts

- **Paired-end barcoded reads**: CallingCards data consists of paired-end reads where Read 1 contains a randomer barcode used to collapse Polymerase Chain Reaction (PCR) duplicates, and Read 2 spans the genomic insertion junction. The pipeline deconvolves barcodes before alignment to avoid false insertion calls caused by PCR duplication or barcode collisions.
- **Reference index dependency**: Before alignment, a reference genome must be indexed using `callingcardstools-build`, which concatenates a transposable element (TE) sequence to the genome assembly and records chromosome lengths. Both the TE and genomic segments are used during alignment to identify true genomic insertion sites versus TE-internal reads.
- **Deconvolution and ambiguity resolution**: During the `counts` subcommand, reads sharing the same genomic position are grouped and the randomer barcodes are evaluated to resolve which reads originate from independent biological molecules versus PCR duplicates. Reads with the same barcode and position are collapsed into a single count, while barcode collisions (same position, different barcode) are reported with adjusted confidence scores.
- **Output formats and wiggle tracks**: The pipeline produces BED-format files for insertion sites, as well as wiggle/wigVar format for coverage tracks. The wiggle output represents insertion density across the genome and can be loaded directly into genome browsers like UCSC or IGV for visualization.
- **Quality control (QC) checks**: The `qc` subcommand evaluates sequencing run quality by measuring barcode quality scores, estimating library complexity, and flagging samples with excessive PCR bias or low diversity, which are common failure modes in CallingCards experiments.

## Pitfalls

- **Skipping reference index build with a custom TE sequence**: If the TE sequence used in the experiment does not match the one encoded during `callingcardstools-build`, all reads containing the TE junction will fail to align, producing zero output. Always confirm the exact TE construct (e.g., Sleeping Beauty SA, SB10x) used in the experimental library before building the index.
- **Using mismatched Read 1 and Read 2 input files**: CallingCards pipelines are direction-aware; Read 1 carries the randomer barcode and Read 2 carries the genomic sequence. Swapping input files during `callingcardstools-align` causes every read to fail barcode extraction and alignment, resulting in an empty output directory with no error message.
- **Ignoring barcode collision reports**: When two different biological molecules share the same genomic insertion position, the deconvolution step records these as ambiguous hits. Treating all hits as unique leads to overcounting insertion sites, especially in saturated mutagenesis libraries where hundreds of independent insertions can land within a single base pair.
- **Applying a standard genome annotation without excluding the TE**: If the reference genome already contains residual TE sequences (e.g., endogenous retroviruses in mouse) and these are not masked during `callingcardstools-build`, spurious insertion calls will map to endogenous TE loci instead of experimental insertion sites, confounding downstream enhancer or knockout analysis.
- **Specifying the wrong sequencing library type**: CallingCards supports both single-insert and double-insert library designs. Running `callingcardstools counts` with the wrong library type flag causes incorrect barcode-to-sample demultiplexing and throws off the total count normalization, making cross-sample comparisons unreliable.

## Examples

### Build a reference index with the Sleeping Beauty TE concatenated to the mouse genome
**Args:** `callingcardstools-build --genome mm10 --te SE --out mm10_SB/`
**Explanation:** This creates an indexed reference directory containing the mm10 assembly with the Sleeping Beauty (SB) transposable element sequence appended, which is required for all subsequent alignment steps to correctly identify genomic versus TE-spanning reads.

### Run quality control checks on a paired-end sequencing run
**Args:** `callingcardstools qc --fq1 R1.fastq.gz --fq2 R2.fastq.gz --out qc_report/`
**Explanation:** This evaluates barcode quality score distributions, estimates library complexity, and flags potential PCR bias or low-diversity samples before committing to the full alignment pipeline.

### Align deconvolved reads to the reference genome
**Args:** `callingcardstools align --ref mm10_SB --sample sample1 --out alignments/`
**Explanation:** After barcode deconvolution, Read 2 sequences are aligned to the genome-TE reference, and insertions are called by identifying reads that span the genomic junction, producing a sorted BAM file for downstream analysis.

### Aggregate and deduplicate insertion counts with barcode resolution
**Args:** `callingcardstools counts --bam alignments/sample1.sort.bam --ref mm10_SB --out counts/sample1_counts.tsv`
**Explanation:** This collapses PCR duplicates using the deconvolved barcodes, resolves barcode collisions at the same genomic position, and outputs a tab-separated file with chromosome, position, strand, and collapsed count columns.

### Export insertion sites as a wiggle track for genome browser visualization
**Args:** `callingcardstools wiggle --counts counts/sample1_counts.tsv --genome mm10 --out sample1_wig/`
**Explanation:** This converts the collapsed insertion count table into wiggle/wigVar format, scaling insertion density per base pair for direct upload to UCSC or IGV genome browsers.

### Export insertion sites as a BED file for intersect analysis with genomic annotations
**Args:** `callingcardstools export --counts counts/sample1_counts.tsv --format bed --out sample1_insertions.bed`
**Explanation:** This converts insertion count data into standard BED format, preserving position, strand, and score columns, enabling genomic interval operations such as overlap with genes, enhancers, or regulatory elements using tools like bedtools or BEDOPs.

### Perform deconvolution on barcoded reads before alignment
**Args:** `callingcardstools deconvolve --fq1 R1.fastq.gz --fq2 R2.fastq.gz --out deconvolved/`
**Explanation:** This extracts the randomer barcode from Read 1, attaches it as a SAM tag to the aligned Read 2, and groups reads by barcode-sample combination, preventing false insertion calls caused by PCR duplication before the alignment step.

### Process multiple samples in batch mode with a sample manifest
**Args:** `callingcardstools batch --manifest samples.tsv --ref mm10_SB --out batch_output/`
**Explanation:** This automates QC, alignment, and count aggregation across all samples listed in the manifest file, producing a standardized directory structure and consolidated count matrix for cross-sample differential insertion analysis.