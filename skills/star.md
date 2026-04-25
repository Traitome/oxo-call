---
name: star
category: alignment
description: Spliced Transcripts Alignment to a Reference — ultrafast RNA-seq aligner
tags: [rna-seq, alignment, splicing, transcriptome, junction, star, ngs]
author: oxo-call built-in
source_url: "https://github.com/alexdobin/STAR/blob/master/doc/STARmanual.pdf"
---

## Concepts
- STAR requires a genome index directory built with 'STAR --runMode genomeGenerate'; this step uses ~30 GB RAM for human.
- STAR uses long options as its primary operations. The first token in ARGS should be --runMode (with value genomeGenerate or alignReads) for the two main operations.
- The genome index is built from the reference FASTA and a GTF annotation file; always use the matching GTF for your genome build.
- STAR outputs Aligned.sortedByCoord.out.bam (sorted BAM), Log.final.out (mapping stats), SJ.out.tab (splice junctions) by default.
- Use --runThreadN to specify threads; --outSAMtype BAM SortedByCoordinate for sorted BAM output.
- For paired-end, provide two FASTQ files separated by a space after --readFilesIn.
- For gzipped FASTQ input, add --readFilesCommand zcat (Linux) or --readFilesCommand 'gzip -dc'.
- --outSAMattributes NH HI AS NM for standard attributes needed by downstream tools like featureCounts.
- --quantMode GeneCounts outputs gene-level counts (ReadsPerGene.out.tab) for differential expression.
- --twopassMode Basic improves splice junction detection by using 1st pass junctions in 2nd pass alignment.
- STARsolo (--soloType CB_UMI_Simple) analyzes single-cell RNA-seq data (10x Genomics) with barcode/UMI processing.
- --soloCBwhitelist specifies cell barcode whitelist; --soloUMIlen sets UMI length (10 for V2, 12 for V3).
- --chimOutType WithinBAM outputs chimeric alignments (fusion genes) in the main BAM file.
- --outFilterMultimapNmax controls maximum multi-mapping loci; default 10, reduce to 1 for unique-only.
- --genomeLoad LoadAndKeep loads genome into shared memory for multiple sequential jobs.

## Pitfalls
- STAR requires the genome index to be pre-built; the index directory (--genomeDir) must exist.
- STAR ARGS must start with --runMode (genomeGenerate or alignReads) — never with short flags like -d or -t. STAR is not like samtools/bwa; it uses long options as its operation selector.
- For gzipped input, you MUST add --readFilesCommand zcat — otherwise STAR reads the binary gzip data.
- STAR uses 30+ GB RAM for human genome alignment — ensure enough memory is available.
- The default output prefix is the current directory; always set --outFileNamePrefix to a unique prefix.
- --outSAMtype BAM Unsorted is faster but downstream tools usually need sorted BAM.
- Two-pass mode (--twopassMode Basic) improves splice junction detection for novel junctions — use for discovery projects.
- STARsolo requires cDNA read first, barcode read second in --readFilesIn order (opposite of typical FASTQ naming).
- --quantMode GeneCounts counts reads per gene; stranded libraries need correct --quantModeTranscriptomeSAM settings.
- Shared memory (--genomeLoad) requires cleanup with --genomeLoad Remove to free RAM after jobs complete.
- --outFilterMismatchNoverLmax default 0.3 may be too permissive; tighten for higher specificity.
- Index must be built with same STAR version used for alignment; version mismatch causes errors.

## Examples

### build genome index for STAR alignment
**Args:** `--runMode genomeGenerate --genomeDir /path/to/star_index --genomeFastaFiles genome.fa --sjdbGTFfile genes.gtf --runThreadN 8`
**Explanation:** --runMode genomeGenerate builds the index; --genomeDir output directory; --genomeFastaFiles reference FASTA; --sjdbGTFfile adds splice junctions; --runThreadN 8 threads; index is reusable

### align paired-end RNA-seq gzipped FASTQ files to the genome
**Args:** `--runMode alignReads --genomeDir /path/to/star_index --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample1/ --outSAMattributes NH HI AS NM`
**Explanation:** --runMode alignReads for alignment; --genomeDir index directory; --readFilesIn R1 and R2; --readFilesCommand zcat handles gzipped input; --runThreadN 8 threads; --outSAMtype outputs sorted BAM; --outFileNamePrefix output prefix; --outSAMattributes standard SAM attributes

### align single-end RNA-seq reads with two-pass mode for better junction detection
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn reads.fastq.gz --readFilesCommand zcat --twopassMode Basic --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn input file; --readFilesCommand zcat for gzip; --twopassMode Basic improves junction detection; --runThreadN threads; --outSAMtype sorted BAM; --outFileNamePrefix output prefix; recommended for RNA-seq discovery projects

### align reads and output unmapped reads to a FASTQ file
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn R1.fq.gz R2.fq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --outReadsUnmapped Fastx`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn inputs; --readFilesCommand zcat; --runThreadN threads; --outSAMtype sorted BAM; --outFileNamePrefix prefix; --outReadsUnmapped Fastx writes unmapped reads to Unmapped.out.mate1/mate2 files

### align with gene count quantification for differential expression
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --quantMode GeneCounts`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn inputs; --readFilesCommand zcat; --runThreadN threads; --outSAMtype sorted BAM; --outFileNamePrefix prefix; --quantMode GeneCounts outputs ReadsPerGene.out.tab for DESeq2/edgeR

### align 10x Genomics single-cell data with STARsolo
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn cDNA.fastq.gz barcode.fastq.gz --readFilesCommand zcat --runThreadN 16 --outSAMtype BAM Unsorted --outFileNamePrefix sample/ --soloType CB_UMI_Simple --soloCBwhitelist 737K-august-2016.txt --soloUMIlen 10 --soloFeatures Gene`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn cDNA then barcode; --readFilesCommand zcat; --runThreadN threads; --outSAMtype unsorted BAM; --outFileNamePrefix prefix; --soloType CB_UMI_Simple enables STARsolo; --soloCBwhitelist barcode whitelist; --soloUMIlen 10 for V2; --soloFeatures Gene; outputs Gene/filtered matrix like CellRanger

### detect fusion genes with chimeric alignment output
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --chimOutType WithinBAM --chimSegmentMin 20`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn inputs; --readFilesCommand zcat; --runThreadN threads; --outSAMtype sorted BAM; --outFileNamePrefix prefix; --chimOutType WithinBAM outputs chimeric/fusion reads in main BAM; --chimSegmentMin 20 filters short segments

### load genome into shared memory for multiple jobs
**Args:** `--runMode alignReads --genomeDir /star_index --genomeLoad LoadAndKeep --runThreadN 1`
**Explanation:** --runMode alignReads; --genomeDir index; --genomeLoad LoadAndKeep loads genome into RAM once; --runThreadN 1 thread for loading; subsequent jobs use same memory; remember to --genomeLoad Remove when done

### align with unique-only mapping (no multi-mappers)
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --outFilterMultimapNmax 1 --outSAMattributes NH HI AS NM`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn inputs; --readFilesCommand zcat; --runThreadN threads; --outSAMtype sorted BAM; --outFileNamePrefix prefix; --outFilterMultimapNmax 1 keeps only unique alignments; --outSAMattributes standard attributes; useful for ChIP-seq

### align with strict mismatch filtering for high specificity
**Args:** `--runMode alignReads --genomeDir /star_index --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --outFilterMismatchNoverLmax 0.05 --outFilterScoreMinOverLread 0.9`
**Explanation:** --runMode alignReads; --genomeDir index; --readFilesIn inputs; --readFilesCommand zcat; --runThreadN threads; --outSAMtype sorted BAM; --outFileNamePrefix prefix; --outFilterMismatchNoverLmax 0.05 mismatch ratio; --outFilterScoreMinOverLread 0.9 score threshold; tighter filtering for high-confidence alignments

### remove genome from shared memory after jobs complete
**Args:** `--runMode alignReads --genomeDir /star_index --genomeLoad Remove`
**Explanation:** --runMode alignReads; --genomeDir index; --genomeLoad Remove unloads shared genome; essential cleanup after using LoadAndKeep to free system RAM
