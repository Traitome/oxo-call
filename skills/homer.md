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
- -size parameter in findMotifsGenome.pl controls the window around peak centers: 50bp for primary TF motifs, 200bp for co-enriched motifs, 1000bp for histone regions.
- -len parameter specifies motif lengths to discover; shorter motifs (8-10bp) are faster and often sufficient; longer motifs (12-20bp) require more time and memory.
- makeUCSCfile creates bedGraph or bigWig files for UCSC Genome Browser visualization.
- getDifferentialPeaksReplicates.pl performs differential peak analysis with replicates using DESeq2.
- analyzeChIP-Seq.pl automates the full ChIP-seq pipeline from tag directory to motif analysis.

## Pitfalls

- CRITICAL: HOMER is a multi-binary suite. Each tool is a separate command: findPeaks, makeTagDirectory, annotatePeaks.pl, findMotifsGenome.pl, mergePeaks, pos2bed.pl, makeUCSCfile, configureHomer.pl. ARGS for each tool start with their own flags — there is NO single 'homer' subcommand pattern.
- makeTagDirectory requires the genome size or a genome identifier; omitting -genome causes incorrect normalization and peak calling.
- Using -style histone without providing a control (-i) greatly inflates broad peak calls — always supply matched input.
- findMotifsGenome.pl requires write access to the output directory; it creates many intermediate files and fails silently on permission errors.
- annotatePeaks.pl outputs a large table to stdout by default — always redirect to a file or use -o.
- mergePeaks -d given must match biological expectations: default 100 bp is suitable for TF peaks but too narrow for histone domains.
- HOMER genome packages must be pre-installed (perl configureHomer.pl -install hg38); specifying an uninstalled genome silently uses wrong sizes.
- Peak files must have unique IDs in the first column; duplicate IDs cause crashes or incorrect results; use renamePeaks.pl if needed.
- Peak files saved from Excel on Mac must be in "Text (Windows)" format; use checkPeakFile.pl to verify format.
- Long motif lengths (>12bp) with large -size values can cause excessive memory usage and long runtimes.
- findMotifsGenome.pl -mask is recommended to avoid repeat regions confounding motif discovery.
- batchParallel.pl is useful for automating multiple samples but can mask QC issues; review results carefully.

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

### create UCSC Genome Browser visualization track
**Args:** `makeUCSCfile chipseq_tags/ -o chipseq_track.bedGraph -fsize 1e20`
**Explanation:** creates bedGraph for UCSC visualization; -fsize 1e20 disables fragment size adjustment for sharp peaks

### find motifs with custom background regions
**Args:** `findMotifsGenome.pl peaks.txt hg38 motif_output/ -size 200 -bg background_peaks.txt -mask -p 8`
**Explanation:** -bg uses custom background peaks instead of random genomic regions; useful when comparing specific peak subsets

### run differential peak analysis with replicates
**Args:** `getDifferentialPeaksReplicates.pl -t treatment_tag1/ treatment_tag2/ -c control_tag1/ control_tag2/ -genome hg38 -o diff_peaks.txt`
**Explanation:** compares treatment vs control with biological replicates using DESeq2; requires at least 2 replicates per condition

### batch process multiple ChIP-seq samples
**Args:** `batchParallel.pl makeTagDirectory tags -genome hg38 -checkGC -f sample1.bam sample2.bam sample3.bam`
**Explanation:** batchParallel.pl runs makeTagDirectory in parallel on multiple BAM files; outputs to sample-specific directories

### find motifs of different lengths
**Args:** `findMotifsGenome.pl peaks.txt hg38 motif_output/ -size 100 -len 8,10,12 -S 50 -mask -p 8`
**Explanation:** -len 8,10,12 discovers motifs of 8, 10, and 12 bp; -S 50 finds top 50 motifs of each length

### get differential peaks between two conditions
**Args:** `getDifferentialPeaks peaks1.txt peaks2.txt input1_tag/ input2_tag/ -genome hg38 > diff_peaks.txt`
**Explanation:** identifies peaks enriched in one condition vs another using tag density comparison; requires input controls for both

### analyze RNA-seq with HOMER
**Args:** `analyzeRNA.pl rna_tags/ mm10 -strand both -count exons -o rna_counts.txt`
**Explanation:** quantifies gene expression from RNA-seq tags; -strand both for unstranded libraries; -count exons for exon-level quantification
