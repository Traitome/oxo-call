---
name: hisat2
category: alignment
description: Graph-based alignment of short reads to a reference genome; successor to TopHat2 for RNA-seq alignment
tags: [alignment, rna-seq, splicing, ngs, illumina, spliced-alignment, transcriptome]
author: oxo-call built-in
source_url: "https://daehwankimlab.github.io/hisat2/"
---

## Concepts

- HISAT2 index building requires the companion binary 'hisat2-build genome.fa index_prefix'. When asked to build a genome index, output ARGS starting with 'hisat2-build' — the system uses it as the actual executable automatically.
- For RNA-seq, use the splice-site and exon-aware index (hisat2-build with --ss and --exon files from the GTF) for better splicing.
- HISAT2 outputs SAM to stdout by default — pipe to 'samtools view -b' or use -S/-o to write to a file.
- Use -p N for multi-threading; -x for index prefix; -1/-2 for paired-end reads; -U for single-end reads.
- The --dta flag (downstream transcriptome assembly) is recommended when using StringTie; --dta-cufflinks for Cufflinks.
- HISAT2 summary statistics are printed to stderr — capture with 2> align_summary.txt.
- Use --no-spliced-alignment for DNA alignment (genomic mode); default allows spliced alignments for RNA.
- --max-intronlen controls maximum intron length (default 500000); increase for organisms with larger introns.
- --known-splicesite-infile provides known splice sites to improve alignment accuracy at annotated junctions.
- --novel-splicesite-outfile reports novel splice sites discovered during alignment for downstream analysis.
- -k controls the maximum number of distinct primary alignments to report per read (default 5 for linear, 10 for graph index).
- --no-softclip disables soft-clipping; useful when exact alignment boundaries are critical.
- --avoid-pseudogene tries to avoid aligning reads to pseudogenes (experimental).

## Pitfalls

- CRITICAL: hisat2 has NO subcommands. ARGS starts directly with flags (e.g., -x, -p, -1, -2, -U). Do NOT put a subcommand like 'align' or 'map' before flags. Index building uses the separate binary 'hisat2-build' (not a subcommand).
- Index building uses companion binary 'hisat2-build', not 'hisat2'. Always start ARGS with 'hisat2-build' for index tasks; the system detects and invokes it as the executable.
- HISAT2 outputs SAM to stdout — always pipe to samtools or redirect to a file to avoid filling your terminal.
- For RNA-seq downstream of StringTie, use --dta flag — without it, StringTie produces suboptimal transcripts.
- The -x index prefix must match the exact prefix used during hisat2-build (not the .fa file name).
- HISAT2 does not work well with very long reads (>300 bp) — use STAR or minimap2 for long reads.
- Paired-end read order matters: -1 must be R1, -2 must be R2; swapping them reduces mapping rate.
- Providing --rna-strandness (RF/FR) for strand-specific libraries improves quantification accuracy.
- Large -k values (>20) can significantly slow down alignment in repetitive genomes; use --max-seeds to control seed extension instead.
- --max-intronlen default (500000) may be insufficient for some organisms (e.g., plants with megabase introns); adjust accordingly.
- --known-splicesite-infile requires a specific format (chr start end strand); GTF must be converted using hisat2_extract_splice_sites.py.
- --novel-splicesite-outfile output format is not the same as --known-splicesite-infile input format; cannot directly feed output back as input.

## Examples

### build a HISAT2 genome index from a reference FASTA
**Args:** `hisat2-build -p 8 genome.fa genome_index`
**Explanation:** hisat2-build is the companion binary; creates genome_index.*.ht2 files used by hisat2 -x

### align paired-end RNA-seq reads to the genome with 8 threads
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --dta -S aligned.sam`
**Explanation:** --dta optimizes for StringTie; -x index prefix; -1/-2 paired-end reads; -S output SAM file

### align paired-end RNA-seq reads and output sorted BAM directly
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --dta | samtools sort -@ 4 -o sorted.bam`
**Explanation:** pipe HISAT2 output to samtools sort to produce a sorted BAM without intermediate SAM file

### align strand-specific paired-end RNA-seq (reverse-strand library)
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --rna-strandness RF --dta -S aligned.sam`
**Explanation:** --rna-strandness RF for dUTP/reverse-strand libraries (most modern RNA-seq); use FR for forward-strand

### align single-end RNA-seq reads
**Args:** `-p 4 -x genome_index -U reads.fastq.gz --dta -S aligned.sam`
**Explanation:** -U for single-end reads; same index and --dta flag apply

### build splice-site aware index using GTF annotation for improved RNA-seq
**Args:** `hisat2-build -p 8 genome.fa genome_spliceaware_index --ss splice_sites.txt --exon exons.txt`
**Explanation:** hisat2-build companion binary; --ss and --exon files extracted from GTF improve spliced alignment

### align paired-end reads with strand information and save alignment statistics
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --rna-strandness RF --dta -S aligned.sam 2> align_summary.txt`
**Explanation:** 2> redirects HISAT2 alignment stats to file; --rna-strandness RF for reverse-strand library

### align single-end reads in genomic (non-spliced) mode for DNA-seq
**Args:** `-p 8 -x genome_index -U reads.fastq.gz --no-spliced-alignment -S aligned.sam`
**Explanation:** --no-spliced-alignment disables splicing; use for ChIP-seq or WGS instead of RNA-seq

### align paired-end reads and discard unmapped reads
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --dta --no-unal -S aligned.sam`
**Explanation:** --no-unal suppresses unmapped reads in output; reduces output file size

### align paired-end reads and output only uniquely mapped reads
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --dta | samtools view -b -q 1 -o unique_aligned.bam`
**Explanation:** samtools view -q 1 filters to reads with mapping quality ≥1, effectively keeping uniquely mapped reads

### align with known splice sites for improved junction accuracy
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --known-splicesite-infile known_splicesites.txt --dta -S aligned.sam`
**Explanation:** --known-splicesite-infile provides annotated splice sites; improves alignment accuracy at known junctions; file format: chr start end strand

### report novel splice sites discovered during alignment
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --novel-splicesite-outfile novel_splicesites.txt --dta -S aligned.sam`
**Explanation:** --novel-splicesite-outfile writes discovered splice sites to file; useful for identifying unannotated isoforms

### increase maximum intron length for organisms with large introns
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --max-intronlen 1000000 --dta -S aligned.sam`
**Explanation:** --max-intronlen 1000000 allows introns up to 1Mb; necessary for some plants and organisms with large introns (default 500kb)

### align with multiple alignments per read for expression quantification
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz -k 10 --dta -S aligned.sam`
**Explanation:** -k 10 reports up to 10 primary alignments per read; useful for multi-mapped read quantification with tools like Salmon/RSEM

### align without soft-clipping for precise boundary detection
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --no-softclip --dta -S aligned.sam`
**Explanation:** --no-softclip disables soft-clipping; ensures exact alignment boundaries; useful for variant calling from RNA-seq

### avoid pseudogene alignments (experimental)
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --avoid-pseudogene --dta -S aligned.sam`
**Explanation:** --avoid-pseudogene attempts to align reads to functional genes instead of pseudogenes; experimental feature

### trim reads before alignment
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --trim5 5 --trim3 10 --dta -S aligned.sam`
**Explanation:** --trim5 5 removes 5bp from 5' end; --trim3 10 removes 10bp from 3' end; useful for removing adapter sequences or low-quality ends
