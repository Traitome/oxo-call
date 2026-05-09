---
name: circle-map
category: RNA-seq Analysis
description: A bioinformatics tool for identifying, mapping, and quantifying circular RNAs (circRNAs) from RNA-seq data by detecting back-spliced junction (BSJ) reads.
tags:
  - circRNA
  - back-spliced junction
  - RNA-seq
  - non-coding RNA
  - isoform detection
author: AI-generated
source_url: https://github.com/HowToFind/circle-map
---

## Concepts

- **Back-Spliced Junction (BSJ) Detection**: circle-map identifies circRNAs by detecting reads that span the junction where the 3' end of an exon connects to the 5' start of an upstream exon (back-splicing). These BSJ reads are the primary evidence for circular RNA existence.

- **Input Format Requirements**: The tool operates on aligned SAM/BAM files from standard RNA-seq aligners (such as STAR or BWA). It requires that reads be mapped to a reference genome, and expects the SAM format to contain proper CIGAR strings indicating how each read aligns.

- **Output Data Structure**: circle-map produces BED and tabular outputs listing identified circRNAs with genomic coordinates (chromosome, start, end, strand), read counts, and associated gene annotations. The quantification step outputs expression matrices suitable for differential expression analysis.

- **Paired-End and Single-End Support**: The tool supports both paired-end and single-end RNA-seq data. For paired-end data, both reads must span the back-spliced junction for reliable detection, while single-end data requires longer reads that fully cover the junction region.

- **Quantification Modes**: circle-map offers multiple quantification modes including raw read counts, reads per million (RPM) normalization, and junction-spanning read (JSR) counting, allowing flexible expression level comparisons across samples.

## Pitfalls

- **Insufficient Read Length**: If RNA-seq reads are shorter than the circRNA junction span (the distance between the 5' and 3' end of the circle), the tool cannot detect BSJ reads. This leads to false negatives, especially for circRNAs with large loop sizes. Ensure read lengths exceed the expected junction distance.

- **Misaligned or Multimapped Reads**: Reads that map to multiple genomic locations (multimappers) can produce false positive circRNA calls. Using aligners with strict multimapping settings or filtering out multimapped reads before running circle-map improves accuracy.

- **Incorrect Genome Annotation**: Running circle-map without a proper gene annotation file (GTF/GFF) results in circRNAs being reported without gene labels, making biological interpretation difficult. Always provide a comprehensive transcript annotation file for accurate gene assignment.

- **Memory Usage with Large Datasets**: Processing whole-transcriptome RNA-seq BAM files can require significant memory (8+ GB for human data). Running on systems with limited RAM may cause crashes or slow processing. Split large files by chromosome or use downsampling for initial exploratory analysis.

- **Strand-Specificity Confusion**: Failing to specify the correct strand-specific library protocol (forward, reverse, or non-strand-specific) can invert the reported strand orientation of circRNAs. Verify your RNA-seq library type and pass the appropriate `--strand` parameter.

## Examples

### Basic circRNA detection from an aligned BAM file
**Args:** `joinequivalent -i sample1.bam -o circCandidates.bed`
**Explanation:** Joins equivalent back-spliced junction reads from an input BAM file and outputs potential circRNA candidates in BED format for downstream filtering.

### Adding BSJ reads with gene annotation
**Args:** `add -i sample1.bam -o annotatedCirc.bed -g genes.gtf`
**Explanation:** Adds gene annotation information to the detected BSJ reads by cross-referencing genomic coordinates with a provided GTF annotation file.

### Quantifying circRNA expression with RPM normalization
**Args:** `quantify -i circList.bed -b sample1.bam -o expression.tsv -n RPM`
**Explanation:** Quantifies the expression of circRNAs listed in the input BED file using reads per million normalization to account for sequencing depth differences.

### Running full detection pipeline with multiple samples
**Args:** `pipeline -i sample1.bam sample2.bam -o results/ -g genes.gtf -t 8`
**Explanation:** Executes the complete circRNA detection and quantification pipeline across multiple BAM files in parallel using 8 threads, producing organized output directories.

### Filtering circRNAs by minimum read support
**Args:** `filter -i allCirc.bed -o highConfidence.bed -m 2`
**Explanation:** Filters circRNA candidates to retain only those supported by at least 2 distinct back-spliced junction reads, reducing false positive detections.

### Extracting circRNAs from a specific chromosome
**Args:** `filter -i allCirc.bed -o chr22Circ.bed -c chr22`
**Explanation:** Extracts only circRNAs located on chromosome 22 from a genome-wide list, useful for targeted analysis or regional studies.

### Converting output to BEDGraph format
**Args:** `joinequivalent -i sample.bam -o output.bg -f bedgraph`
**Explanation:** Outputs circRNA junction read counts in BEDGraph format for direct visualization in genome browsers like IGV or UCSC.

### Specifying minimum junction overlap length
**Args:** `joinequivalent -i sample.bam -o circ.bed -j 10`
**Explanation:** Requires at least 10 base pairs of overlap on each side of the back-spliced junction, filtering out cases with minimal junction evidence.