---
name: homer
category: epigenomics
description: Suite for ChIP-seq and ATAC-seq peak calling, motif analysis, and annotation
tags: [chipseq, atacseq, peaks, motif, annotation, epigenomics, histone]
author: oxo-call built-in
source_url: "http://homer.ucsd.edu/homer/ngs/index.html"
---

## Concepts

- HOMER workflows begin with makeTagDirectory, which converts aligned BAM/SAM reads into an internal tag format used by all downstream commands.
- findPeaks operates in two main modes: -style factor for transcription factors (sharp peaks) and -style histone for broad histone marks.
- annotatePeaks.pl maps peaks to genomic features (promoter, exon, intron, intergenic) using a GTF/GFF annotation; it also computes motif enrichment at peaks.
- findMotifsGenome.pl performs de novo and known motif enrichment analysis; it requires a genome FASTA or a HOMER genome package (e.g., hg38).
- mergePeaks combines peak files from multiple experiments, reporting union or intersection sets with -d to set the merging distance.
- HOMER uses a tag density-based normalization; always provide an input/control tag directory with -i for ChIP experiments to reduce false positives.

## Pitfalls

- makeTagDirectory requires the genome size or a genome identifier; omitting -genome causes incorrect normalization and peak calling.
- Using -style histone without providing a control (-i) greatly inflates broad peak calls — always supply matched input.
- findMotifsGenome.pl requires write access to the output directory; it creates many intermediate files and fails silently on permission errors.
- annotatePeaks.pl outputs a large table to stdout by default — always redirect to a file or use -o.
- mergePeaks -d given must match biological expectations: default 100 bp is suitable for TF peaks but too narrow for histone domains.
- HOMER genome packages must be pre-installed (perl configureHomer.pl -install hg38); specifying an uninstalled genome silently uses wrong sizes.

## Examples

### create a HOMER tag directory with makeTagDirectory
**Args:** `makeTagDirectory chipseq_tags/ sample.bam -genome hg38 -checkGC`
**Explanation:** -genome hg38 sets chromosome sizes; -checkGC reports GC content for bias assessment

### call narrow transcription factor peaks with findPeaks
**Args:** `findPeaks chipseq_tags/ -style factor -i input_tags/ -o peaks.txt`
**Explanation:** -style factor is for sharp TF peaks; -i supplies the matched input control directory

### call broad histone modification peaks with findPeaks
**Args:** `findPeaks chipseq_tags/ -style histone -i input_tags/ -o broad_peaks.txt`
**Explanation:** -style histone uses a sliding window approach suited for broad chromatin domains

### annotate peaks with annotatePeaks.pl using hg38 RefSeq annotation
**Args:** `annotatePeaks.pl peaks.txt hg38 -gtf genes.gtf > annotated_peaks.txt`
**Explanation:** maps each peak to promoter/exon/intron/intergenic regions; -gtf overrides default HOMER annotation

### run de novo and known motif analysis with findMotifsGenome.pl
**Args:** `findMotifsGenome.pl peaks.txt hg38 motif_output/ -size 200 -mask -p 8`
**Explanation:** -size 200 uses 200 bp window centered on peak summit; -mask soft-masks repeats; -p 8 uses 8 threads

### merge peak files from two replicates with mergePeaks
**Args:** `mergePeaks rep1_peaks.txt rep2_peaks.txt -d 100 -prefix merged_peaks -venn venn.txt`
**Explanation:** -d 100 merges peaks within 100 bp; -prefix names output files; -venn reports overlap statistics

### convert HOMER peak file to BED with pos2bed.pl
**Args:** `pos2bed.pl peaks.txt > peaks.bed`
**Explanation:** converts HOMER peak coordinates (1-based) to standard BED format (0-based) for downstream tools
