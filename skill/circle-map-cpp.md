---
name: circle-map-cpp
category: bioinformatics/circular-rna-mapping
description: A circular DNA/RNA sequencing data analysis tool that identifies and quantifies circular RNA junctions from split-read alignments. Circle-Map reconstructs backspliced junctions by extending split alignments at each genomic break point, enabling discovery and annotation of circular transcripts from standard RNA-seq pipelines.
tags: [circRNA, circular-RNA, backsplicing, RNA-seq, junction-detection, split-reads, splicing]
author: AI-generated
source_url: https://github.com/creative-diversity/circle-map
---

## Concepts

- Circle-Map processes BAM/SAM alignment files from standard RNA-seq aligners (STAR,HISAT2) to identify reads spanning circular RNA junctions. The algorithm examines split alignments with genomic break points, extends soft-clipped sequences at each break, and realigns reads to detect back-spliced connections that linear reference genomes cannot capture.
- Input files require SAM format with header (@SQ reference sequence dictionary lines) and CIGAR strings containing N operators (skip/large-deletion) to indicate splice junctions. Circle-Map-Build must generate a genomic index using companion binary circle-map-cpp-build before running the mapper.
- Output produces a BED file of predicted circular junctions with columns: chromosome, start, end, junction_id, split_read_count, strand. A junctions.fa file provides fasta sequences for junction regions (±50bp flanking) enabling downstream validation or PCR primer design.
- The tool operates on single-end or paired-end data, accepting read names as input for tracking read pairs. Thread count (-t/--threads) scales linearly for large datasets; memory usage scales with genome size and read length distribution.

## Pitfalls

- Using alignments without N operators in CIGAR strings causes Circle-Map to report zero junctions. TopHat produces many N:CIGAR entries and is recommended over BWA-mem or STAR's default settings for circRNA detection.
- Mismatched library protocols (stranded vs unstranded) produce incorrect strand annotations in output. Always specify the --strandness option matching your RNA-seq kit; TruSeq Stranded kits require 'forward' while dUTP-based methods require 'reverse'.
- Setting --min-split-count below 2 discards genuine low-abundance circRNAs but also increases false positives from misalignment artifacts. Use min-split-count >= 2 for discovery and >= 5 for confident annotation.
- Attempting to run circle-map-cpp without a pre-built index generates a segmentation fault. The companion binary circle-map-cpp-build must execute successfully with genome fasta before any mapping operation.
- Specifying an empty or corrupted junctions file as input when using annotate mode causes runtime errors. Always validate junction BED files with column count = 6 before annotation runs.

## Examples

### Build genomic index for circular mapping
**Args:** circle-map-cpp-build --genome genome.fa --output-index human_index
**Explanation:** Generates binary index files required for split-read extension at genomic break points. Execute once per genome version; index reuse across samples is valid.

### Detect circular junctions from RNA-seq alignments
**Args:** circle-map-cpp read-alignment --bam rnaseq_aligned.bam --genome human_index --output junctions_output --min-split-count 2
**Explanation:** Reconstructs backspliced junctions by examining split alignments with N:CIGAR operations, extending soft-clipped bases, and identifying circular connections.

### Annotate junctions using a reference GTF
**Args:** circle-map-cpp annotate --junctions junctions.bed --annotation annotations.gtf --output annotated_results --min-overlap 10
**Explanation:** Overlaps predicted circular junctions with reference gene annotations to identify known circRNA genes and novel transcript origins within gene bodies.

### Process paired-end sequencing data
**Args:** circle-map-cpp read-alignment --bam paired_end.bam --genome human_index --output paired_junctions --read-names names.txt --min-split-count 3
**Explanation:** Tracks read pairs using names file to reduce false positives from concordance artifacts; --read-names maps split reads to their mated pairs for validation.

### Quantify junctions with stranded library protocol
**Args:** circle-map-cpp read-alignment --bam stranded_rnaseq.bam --genome human_index --output stranded_output --min-split-count 2 --strandness forward
**Explanation:** Specifies library strandedness to correctly orient junction strands; TruSeq stranded kit libraries use forward specification for accurate anti-sense circRNA detection.

### Run with parallel processing for large datasets
**Args:** circle-map-cpp read-alignment --bam large_dataset.bam --genome human_index --output large_output --min-split-count 2 --threads 16
**Explanation:** Distributes split-read processing across 16 CPU cores for faster throughput; memory usage increases linearly with thread count.

### Filter junctions by minimum supporting reads
**Args:** circle-map-cpp read-alignment --bam rnaseq_aligned.bam --genome human_index --output filtered_output --min-split-count 5
**Explanation:** Applies stringent threshold to reduce false-positive junctions from sequencing errors; recommended for validation studies requiring high-confidence circRNA lists.