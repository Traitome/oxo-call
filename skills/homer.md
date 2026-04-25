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

- HOMER is a multi-binary suite. Each tool is a separate command: findPeaks, makeTagDirectory, annotatePeaks.pl, findMotifsGenome.pl, mergePeaks, pos2bed.pl, makeUCSCfile, configureHomer.pl. ARGS for each tool start with their own flags — there is NO single 'homer' subcommand pattern.
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
**Explanation:** makeTagDirectory command; chipseq_tags/ output directory; sample.bam input BAM; -genome hg38 sets chromosome sizes; -checkGC reports GC content for bias assessment

### call narrow transcription factor peaks with findPeaks
**Args:** `findPeaks chipseq_tags/ -style factor -i input_tags/ -o peaks.txt`
**Explanation:** findPeaks command; chipseq_tags/ tag directory input; -style factor for sharp TF peaks; -i input_tags/ control directory; -o peaks.txt output file

### call broad histone modification peaks with findPeaks
**Args:** `findPeaks chipseq_tags/ -style histone -i input_tags/ -o broad_peaks.txt`
**Explanation:** findPeaks command; chipseq_tags/ tag directory input; -style histone for broad chromatin domains; -i input_tags/ control directory; -o broad_peaks.txt output file

### annotate peaks with annotatePeaks.pl using hg38 RefSeq annotation
**Args:** `annotatePeaks.pl peaks.txt hg38 -gtf genes.gtf > annotated_peaks.txt`
**Explanation:** annotatePeaks.pl command; peaks.txt input peaks; hg38 genome; -gtf genes.gtf custom annotation; > annotated_peaks.txt output file; maps peaks to promoter/exon/intron/intergenic

### run de novo and known motif analysis with findMotifsGenome.pl
**Args:** `findMotifsGenome.pl peaks.txt hg38 motif_output/ -size 200 -mask -p 8`
**Explanation:** findMotifsGenome.pl command; peaks.txt input peaks; hg38 genome; motif_output/ output directory; -size 200 bp window; -mask soft-masks repeats; -p 8 threads

### merge peak files from two replicates with mergePeaks
**Args:** `mergePeaks rep1_peaks.txt rep2_peaks.txt -d 100 -prefix merged_peaks -venn venn.txt`
**Explanation:** mergePeaks command; rep1_peaks.txt rep2_peaks.txt input peaks; -d 100 merges within 100 bp; -prefix merged_peaks output naming; -venn venn.txt overlap statistics

### convert HOMER peak file to BED with pos2bed.pl
**Args:** `pos2bed.pl peaks.txt > peaks.bed`
**Explanation:** pos2bed.pl command; peaks.txt input peaks; > peaks.bed output BED file; converts 1-based HOMER to 0-based BED

### create UCSC Genome Browser visualization track
**Args:** `makeUCSCfile chipseq_tags/ -o chipseq_track.bedGraph -fsize 1e20`
**Explanation:** makeUCSCfile command; chipseq_tags/ tag directory; -o chipseq_track.bedGraph output bedGraph; -fsize 1e20 disables fragment size adjustment

### find motifs with custom background regions
**Args:** `findMotifsGenome.pl peaks.txt hg38 motif_output/ -size 200 -bg background_peaks.txt -mask -p 8`
**Explanation:** findMotifsGenome.pl command; peaks.txt input; hg38 genome; motif_output/ output; -size 200 bp window; -bg background_peaks.txt custom background; -mask soft-masks repeats; -p 8 threads

### run differential peak analysis with replicates
**Args:** `getDifferentialPeaksReplicates.pl -t treatment_tag1/ treatment_tag2/ -c control_tag1/ control_tag2/ -genome hg38 -o diff_peaks.txt`
**Explanation:** getDifferentialPeaksReplicates.pl command; -t treatment_tag1/ treatment_tag2/ treatment tag directories; -c control_tag1/ control_tag2/ control directories; -genome hg38; -o diff_peaks.txt output; uses DESeq2

### batch process multiple ChIP-seq samples
**Args:** `batchParallel.pl makeTagDirectory tags -genome hg38 -checkGC -f sample1.bam sample2.bam sample3.bam`
**Explanation:** batchParallel.pl command; makeTagDirectory tool name; tags output prefix; -genome hg38; -checkGC GC bias; -f sample1.bam sample2.bam sample3.bam input BAMs

### find motifs of different lengths
**Args:** `findMotifsGenome.pl peaks.txt hg38 motif_output/ -size 100 -len 8,10,12 -S 50 -mask -p 8`
**Explanation:** findMotifsGenome.pl command; peaks.txt input; hg38 genome; motif_output/ output; -size 100 bp window; -len 8,10,12 motif lengths; -S 50 top 50 per length; -mask repeats; -p 8 threads

### get differential peaks between two conditions
**Args:** `getDifferentialPeaks peaks1.txt peaks2.txt input1_tag/ input2_tag/ -genome hg38 > diff_peaks.txt`
**Explanation:** getDifferentialPeaks command; peaks1.txt peaks2.txt peak files; input1_tag/ input2_tag/ control directories; -genome hg38; > diff_peaks.txt output

### analyze RNA-seq with HOMER
**Args:** `analyzeRNA.pl rna_tags/ mm10 -strand both -count exons -o rna_counts.txt`
**Explanation:** analyzeRNA.pl command; rna_tags/ tag directory; mm10 genome; -strand both unstranded; -count exons exon-level; -o rna_counts.txt output
