---
name: ciri-full
category: RNA-seq Analysis / Circular RNA Detection
description: A bioinformatics pipeline for detecting circular RNAs (circRNAs) from RNA-seq data by identifying back-spliced junction (BSJ) reads. The tool scans aligned BAM files or FASTQ inputs to find split-read alignments indicating circular RNA junctions, then filters and annotates candidates using genome annotations.
tags:
  - circular RNA
  - RNA-seq
  - back-spliced junction
  - BSJ
  - non-coding RNA
  - alternative splicing
  - spliceome
author: AI-generated
source_url: https://github.com/bioinfo-biolab/CIRI
---

## Concepts

- **Back-spliced Junction (BSJ) Detection**: ciri-full identifies circular RNAs by detecting split-read alignments where a read spans a junction formed by the 3' splice site of a downstream exon connecting to the 5' splice site of an upstream exon — the hallmark of circular RNA backsplicing.
- **Input Format Requirements**: The tool accepts aligned BAM files (preferred) or FASTQ files. When using BAM input, reads must be aligned to the reference genome with standard spliced aligners (e.g., BWA, STAR) and must retain alignment tags indicating split/supplementary alignments.
- **Annotation-Dependent Filtering**: ciri-full uses genome annotation files (GTF/GFF) to distinguish genuine circular RNAs from linear transcript artifacts by checking whether BSJ junctions overlap with annotated splice sites or fall within known genes.
- **Output Formats**: Results are typically provided as a list of circular RNA candidates in a tab-delimited text file containing coordinates, read counts, associated gene names, and confidence scores; some versions also export GTF files for downstream visualization.
- **Paired-End and Strand Information**: When input data includes strandedness information (e.g., from dUTP-based library prep) or proper mate pairing, ciri-full uses this to disambiguate circular RNAs from linear transcripts and to determine the strand orientation of the circular RNA.

## Pitfalls

- **Using Unspliced Alignments**: Providing BAM files from aligners with splice-aware alignment disabled (or with very small maximum intron size) will eliminate most true BSJ reads because circular RNAs involve non-canonical splicing that aligners may not detect as valid splice events.
- **Ignoring Library Strandness**: For stranded RNA-seq libraries, failing to specify the correct strandness convention (e.g., `--fr` or `--rf` for reverse-forward or reverse-forward orientation) can cause ciri-full to call the wrong strand for circular RNAs or discard valid candidates as artifacts.
- **Threshold Misconfiguration**: Setting the minimum read count (e.g., `-min_read_count`) too high will cause the tool to miss low-abundance circular RNAs, especially in samples where circRNAs are expressed at low levels; setting it too low increases false positives from misaligned reads.
- **Reference Genome Mismatch**: Using a reference genome or annotation file that does not match the exact genome build used for alignment (e.g., GRCh38 vs. GRCh37) will produce coordinate mismatches between the BAM file and the annotation, leading to incorrect filtering and gene association.
- **Insufficient Filtering of Linear Overlaps**: Not enabling filters that exclude circular RNAs where the same genomic region also produces linear readthrough transcripts can result in false positives — circular RNAs are harder to validate when the host gene is heavily expressed.

## Examples

### Detect circular RNAs from an aligned BAM file
**Args:** `-input data/aligned.bam -REF genome.fa -GTF annotation.gtf -output results/circrnas.txt`
**Explanation:** This runs ciri-full on a pre-aligned BAM file with the reference genome and gene annotation, outputting a list of detected circular RNAs with their genomic coordinates and associated gene annotations.

### Adjust the minimum junction read count threshold
**Args:** `-input sample.bam -REF genome.fa -GTF genes.gtf -min_read_count 2 -output circ/candidates.txt`
**Explanation:** Lowering the minimum read count to 2 allows detection of circular RNAs supported by only two spanning reads, which is useful for low-expression samples but may increase false positive rates.

### Run with FASTQ input instead of BAM
**Args:** `-input sample1.fastq sample2.fastq -REF genome.fa -GTF annotation.gtf -outdir fastq_results/`
**Explanation:** When starting from raw FASTQ files, ciri-full performs internal alignment or uses the provided files directly to detect BSJ split reads, useful for pipelines without a separate alignment step.

### Enable stranded library mode for directional RNA-seq data
**Args:** `-input sample.bam -REF genome.fa -GTF genes.gtf -strandness fr -output circ/stranded_circ.txt`
**Explanation:** Specifying strandness as `fr` (forward-reverse) tells ciri-full that the library was prepared with a stranded protocol, improving strand assignment accuracy for circular RNAs.

### Specify a custom minimum junction overlap length
**Args:** `-input sample.bam -REF genome.fa -GTF annotation.gtf -min_overlap 20 -output filtered_circ.txt`
**Explanation:** Requiring a minimum 20-base overlap across the BSJ junction ensures that spans are long enough to be confidently called as genuine backbone reads rather than short spurious alignments.

### Filter out circular RNAs overlapping highly expressed linear isoforms
**Args:** `-input sample.bam -REF genome.fa -GTF genes.gtf -filter_linear -output clean_circRNAs.txt`
**Explanation:** Enabling the `-filter_linear` flag discards circular RNA candidates where overlapping linear transcripts are highly expressed, reducing false positives from readthrough or artifact alignments.