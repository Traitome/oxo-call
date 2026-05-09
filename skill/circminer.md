---
name: circminer
category: Bioinformatics/RNA-seq/Circular-RNA-Detection
description: Detect circular RNAs (circRNAs) from RNA-seq alignment data by identifying back-splice junction events. Supports BAM/SAM input, reference genomes, and outputs cirRNA predictions in BED or tabular format.
tags: [circRNA, back-splice junction, RNA-seq, alternative splicing, non-coding RNA, splice junction]
author: AI-generated
source_url: https://github.com/circminer/circminer
---

## Concepts

- **Back-splice junction detection**: circminer identifies circRNAs by detecting "back-splice junctions" where a downstream 5' splice site connects to an upstream 3' splice site, creating a circular junction not present in the linear genome. Reads spanning these junctions are called "junction reads" and are the primary evidence for circRNA candidates.
- **Input format flexibility**: The tool accepts aligned reads in BAM or SAM format, or directly processes FASTQ files if a reference genome is provided. BAM inputs should be coordinate-sorted and indexed for efficient junction spanning read extraction.
- **Output formats**: Results are typically reported in BED format (for genome browsers) or as a tab-separated file with columns for chromosome, start, end, circRNA ID, read count, and splice motif. The BED format uses 0-based start and 1-based end coordinates.
- **Read support thresholds**: A minimum number of unique junction-spanning reads (default 2) is required to call a circRNA. Additional filters on read continuity, mapping quality, and edit distance reduce false positives from alignment artifacts.

## Pitfalls

- **Using too-low read support threshold**: Setting `--minReads` to 1 will report many false positive circRNAs caused by alignment errors or sequencing artifacts. A threshold of at least 2 independent junction reads is recommended for reliable detection.
- **Ignoring strand information**: Failing to specify `--strand` (plus, minus, or reverse) correctly will misassign circRNA read orientation, especially for stranded RNA-seq libraries. This leads to circRNAs called on the wrong strand.
- **Inputting unsorted or wrong-format BAM files**: circminer requires coordinate-sorted BAM files; feeding unsorted or name-sorted BAMs will cause junction detection to fail or produce empty results. Always index BAMs with `samtools index`.
- **Not filtering multimodal reads**: Reads that map to multiple genomic locations (multi-mapping) can artificially inflate junction read counts. Use the `--maxMultiHits` parameter to filter these reads, otherwise you'll get many false positive circRNAs.

## Examples

### Detect circRNAs from an aligned BAM file
**Args:** `--input alignment.bam --genome hg38 --output circRNAs.bed`
**Explanation:** This runs circminer on a pre-aligned BAM file using the hg38 reference, writing detected circRNAs to a BED file for downstream analysis or visualization.

### Set a higher minimum read support threshold
**Args:** `--input alignment.bam --genome hg38 --minReads 5 --output high_confidence_circRNAs.bed`
**Explanation:** Requiring at least 5 junction-spanning reads reduces false positives by filtering out low-support artifacts, yielding only high-confidence circRNA predictions.

### Specify stranded RNA-seq library orientation
**Args:** `--input alignment.bam --genome hg38 --strand reverse --output circRNAs_stranded.bed`
**Explanation:** For Illumina stranded libraries using the dUTP method, the reverse strand contains the original transcript orientation; specifying this ensures correct circRNA strand assignment.

### Limit multi-mapping reads to reduce false positives
**Args:** `--input alignment.bam --genome hg38 --maxMultiHits 1 --output filtered_circRNAs.bed`
**Explanation:** Discarding reads that map to more than one genomic location prevents alignment ambiguity from creating spurious junction evidence, improving circRNA call specificity.

### Output tabular format with read counts
**Args:** `--input alignment.bam --genome hg38 --outputFormat tsv --output circRNA_counts.tsv`
**Explanation:** Generating a tab-separated file instead of BED includes numerical read counts per junction and metadata, useful for differential expression analysis between samples.