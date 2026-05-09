---
name: connor
category: circRNA Detection / RNA-Seq Analysis
description: A bioinformatics tool for detecting and quantifying circular RNAs (circRNAs) from RNA-seq data by identifying back-spliced junction reads. Uses chimeric alignment analysis to discover circRNAs in eukaryotic transcriptomes.
tags:
  - circRNA
  - RNA-seq
  - back-spliced junction
  - chimeric reads
  - non-coding RNA
  - transcriptomics
author: AI-generated
source_url: https://github.com/BioinformaticsDatabases/connor
---

## Concepts

- **Back-spliced Junction Detection**: Connor identifies circRNAs by detecting reads that map to back-spliced junctions, where the 3' end of an exon connects to the 5' end of an upstream exon in the reverse orientation. These appear as chimeric alignments in standard RNA-seq aligners.

- **Input Format Requirements**: The tool requires aligned BAM files (not raw FASTQ) as input, with specific requirements for alignment indices. The alignments must retain chimeric/split reads to enable circRNA detection. Both unspliced and spliced alignment modes are supported.

- **Minimum Read Support Threshold**: Connor applies a minimum number of supporting reads (default: 2) to report a circRNA. This filter reduces false positives from alignment artifacts. The `-minReads` parameter controls this threshold and directly impacts sensitivity versus specificity.

- **Annotation-Driven Discovery**: The tool can leverage existing gene annotations to prioritize circRNAs that overlap with known exon boundaries. It reports both annotated and novel circRNA candidates with associated genomic coordinates and strand information.

## Pitfalls

- **Using Unaligned FASTQ Files**: Attempting to run connor on unaligned FASTQ files will fail because the tool explicitly requires pre-aligned BAM files. Users must first perform RNA-seq alignment using tools like STAR, TopHat, or hisat2, ensuring chimeric alignment output is preserved.

- **Low MinReads Threshold Producing False Positives**: Setting `-minReads` to 1 dramatically increases sensitivity but also captures alignment artifacts and random chimeric reads that do not represent true circRNAs. This is especially problematic in datasets with low sequencing depth or high background noise.

- **Ignoring Strand Information**: Failing to specify correct strandness in the input BAM files leads to misattribution of circRNA origin, particularly for genes on opposite strands. This causes false positive calls in antisense gene regions.

- **Incompatible Alignment Parameters**: Using alignment configs that discard split/chimeric reads (e.g., `--no-chimeric` in STAR) removes precisely the reads needed for circRNA detection. The alignment step must retain these reads.

## Examples

### Detect circRNAs from a pre-aligned BAM file
**Args:** sample1.bam --outDir results/sample1/
**Explanation:** Runs circRNA detection on an aligned BAM file, outputting results to a specified directory. The BAM must contain chimeric/split reads from the original alignment.

### Adjust minimum read support threshold for high-depth data
**Args:** sample2.bam --outDir results/sample2/ --minReads 3
**Explanation:** Increases the minimum supporting read count to 3, useful when dealing with deeply sequenced samples where lower thresholds would produce excessive false positives.

### Run with gene annotation to prioritize known circRNAs
**Args:** sample3.bam --outDir results/sample3/ -g annotation.gtf
**Explanation:** Uses an existing gene annotation file to guide circRNA detection, prioritizing candidates that match annotated exon boundaries and reporting novel discoveries separately.

### Process multiple samples with batch mode
**Args:** --batch file_list.txt --outDir batch_results/
**Explanation:** Processes multiple BAM files listed in a text file in one run, generating comparative output across all samples. Useful for cohort studies.

### Set minimum junction coverage filter
**Args:** sample4.bam --outDir results/sample4/ --minJunctionCoverage 5
**Explanation:** Filters circRNA predictions based on minimum junction coverage depth, requiring at least 5 reads spanning the back-spliced junction.