---
name: circrna_finder
category: RNA-Seq Analysis / Circular RNA Detection
description: A computational pipeline that identifies circular RNAs (circRNAs) from RNA-seq data by detecting backsplice junctions where sequencing reads span the junction between the 3' and 5' backsplice sites of circularized transcripts.
tags:
  - circRNA
  - noncoding RNA
  - backsplice junction
  - RNA-seq
  - splicing
  - alternative circularization
author: AI-generated
source_url: https://circrna-finder.readthedocs.io
---

## Concepts

- **Backsplice Junction Detection**: circrna_finder identifies circular RNAs by detecting reads that map across backsplice junctions, meaning the read spans from the downstream exon into the upstream exon, which only occurs when RNA is circularized. This requires that reads be aligned to a genome index using BWA or STAR with specific junction-spanning flags.

- **Two-Stage Mapping Strategy**: The pipeline first aligns all reads to the reference genome, then realigns unmapped reads with a special protocol that attempts to map them as if the genome were circularized at each detected splice junction, allowing discovery of previously unknown backsplice junctions.

- **Minimum Junction Read Threshold**: The tool reports circRNAs only when at least two junction-spanning reads support the backsplice event, which is controlled by the `-o min_score` parameter. Increasing this threshold reduces false positives but may miss low-abundance circRNAs that are biologically relevant.

- **Strand-Specificity Flags**: For stranded library preparations (dUTP or Illumina TruSeq), the `-ss` flag must be used to correctly orient reads for junctions on the antisense strand. Using stranded libraries without this flag results in missing all circRNAs transcribed from the negative strand.

- **Output Format**: The tool produces a tab-delimited file listing circRNA candidates with chromosome, start position, end position, strand, number of supporting reads, and the genomic sequence at the junction, allowing downstream validation and annotation.

## Pitfalls

- **Insufficient Sequencing Depth**: RNA-seq datasets with fewer than 20 million read pairs will have poor sensitivity for detecting circRNAs, especially tissue-specific or lowly expressed circular transcripts. Running circrna_finder on shallow sequencing data yields an incomplete circRNA catalog with many false negatives.

- **Mismatched Library Kit Configuration**: Using `-ss` when the library was actually unstranded causes the tool to filter out all circRNAs on the positive strand as being artifactual, resulting in a systematically biased circRNA list missing half of the true circular transcriptome.

- **Ungapped Alignment Without Junction Detection**: When reads are aligned with a short seed length (e.g., BWA with `-k 0`) or with an aligner not configured to detect splice junctions, no backsplice events can be identified because all junction-spanning reads are discarded as multimapping, producing an empty output file.

- **Incompatible Genome Build**: Using a genome annotation file from a different species or an older genome build causes the tool to report junction coordinates that do not match the reference genome, making downstream validation impossible and producing nonsensical fusion-like results.

- **Ignoring the Minimum Score Threshold**: Setting `-o min_score` to 1 produces an excessive number of false positive circRNAs that are supported by single reads from alignment artifacts, while setting it too high (e.g., above 10) filters out genuine low-abundance circRNAs that are valid circular transcripts.

## Examples

### Basic circRNA detection from single-end RNA-seq data

**Args:** `-i sample_RNAseq.fastq -o output_dir -g hg38`
**Explanation:** Runs the complete circrna_finder pipeline on single-end RNA-seq data, aligning reads to the hg38 genome build and saving all output files to the specified directory.

### Detecting circRNAs from paired-end stranded RNA-seq

**Args:** `-i PE_data_R1.fastq PE_data_R2.fastq -ss -o output_dir -g hg38`
**Explanation:** Runs circrna_finder on paired-end stranded RNA-seq data where the library kit preserves strand information, enabling detection of circRNAs transcribed from both genomic strands.

### Raising the minimum support threshold to reduce false positives

**Args:** `-i sample.fastq -o output_dir -g hg38 -o min_score:5`
**Explanation:** Runs the pipeline with a minimum of 5 junction-spanning reads required per circRNA, reducing the output to high-confidence candidates at the cost of missing some valid but lowly expressed circular RNAs.

### Adjusting mismatch tolerance for highly polymorphic samples

**Args:** `-i tumor_sample.fastq -o output_dir -g hg38 -mm 3`
**Explanation:** Allows up to 3 mismatches per read during alignment, which is necessary for tumor or exotic samples with high genetic variation but increases the risk of alignment artifacts and false positive circRNAs.

### Specifying a custom splice junction annotation file

**Args:** `-i sample.fastq -o output_dir -g hg38 -j known_junctions.bed`
**Explanation:** Uses a custom BED file of known splice junctions to prioritize detection of backsplice events involving those junctions, improving sensitivity for annotated circular RNAs and reducing computational time.

### Running detection with verbose logging for debugging

**Args:** `-i sample.fastq -o output_dir -g hg38 -v`
**Explanation:** Enables verbose logging that prints detailed progress messages at each pipeline stage, which is useful for troubleshooting unexpected empty output files or alignment failures.