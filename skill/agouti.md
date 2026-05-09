---
name: agouti
category: Transcriptomics/RNA-seq Analysis
description: Agouti is a bioinformatics tool for isoform detection and quantification from RNA-seq data. It leverages genome annotation and read alignment information to identify and estimate the abundance of transcript isoforms in a sample.
tags:
  - RNA-seq
  - isoform
  - quantification
  - transcriptomics
  - expression
  - alignment
author: AI-generated
source_url: https://github.com/agouti-tool/agouti
---

## Concepts

- **Input Data Model**: Agouti accepts SAM/BAM alignment files from common RNA-seq aligners (e.g., STAR, TopHat) along with reference genome annotation files in GTF/GFF3 format to define known transcript structures.
- **Isoform Detection Algorithm**: The tool uses a graph-based approach to reconstruct potential transcript isoforms by connecting exonic regions based on splicing patterns observed in the aligned reads.
- **Output Formats**: Agouti outputs quantified isoform expressions in tab-delimited text format, with columns for isoform ID, transcript structure, and estimated abundance (FPKM or TPM values).
- **Quantification Units**: Expression values are normalized using FPKM (Fragments Per Kilobase Million) or TPM (Transcripts Per Million) based on transcript length and sequencing depth.

## Pitfalls

- **Using unfiltered BAM files**: Providing BAM files with multimapping reads that haven't been properly handled can lead to ambiguous isoform assignments and inflated expression estimates.
- **Mismatched genome builds**: Using a GTF annotation from a different genome build than the one used for read alignment will cause incorrect isoform assignments and zero-count artifacts.
- **Specifying incorrect strandness**: Mis-specifying the library strandedness (forward vs reverse) will swap sense and antisense isoform counts, producing inverted expression profiles.
- **Ignoring read length requirements**: Agouti requires minimum read lengths (typically >30bp after adapters) for reliable isoform detection; overly trimmed or short reads increase false positive isoform predictions.

## Examples

### Detect isoforms from a single RNA-seq alignment
**Args:** --annotation annotations.gtf --output isoforms.txt --bam alignment.bam
**Explanation:** Performs isoform detection on a single RNA-seq alignment file using the provided gene annotation to define known transcript structures, outputting results to the specified file.

### Quantify isoforms with TPM normalization
**Args:** --annotation annotations.gtf --bam alignment.bam --normalize TPM --output isoform_expression.tsv
**Explanation:** Runs isoform quantification using TPM normalization instead of the default FPKM, producing expression values that are normalized for both transcript length and sequencing depth.

### Process paired-end RNA-seq data
**Args:** --annotation annotations.gtf --bam paired_end.bam --paired-end --output isoforms_pe.txt
**Explanation:** Processes paired-end RNA-seq alignment data, using the insert size and read pairing information to improve isoform reconstruction accuracy.

### Run on multiple samples in batch
**Args:** --annotation annotations.gtf --batch sample_list.txt --output-dir ./results
**Explanation:** Performs isoform detection on multiple samples listed in a text file (one BAM path per line), saving results for each sample into separate files in the output directory.

### Set minimum read support threshold
**Args:** --annotation annotations.gtf --bam alignment.bam --min-reads 5 --output isoforms_filtered.txt
**Explanation:** Filters out isoforms with fewer than 5 supporting reads, reducing false positive predictions from low-coverage or spurious alignment artifacts.

### Specify strand-specific library protocol
**Args:** --annotation annotations.gtf --bam alignment.bam --stranded reverse --output stranded_isoforms.txt
**Explanation:** Processes the alignment assuming a reverse-stranded library preparation (most common), ensuring correct assignment of reads to sense strands of isoforms.