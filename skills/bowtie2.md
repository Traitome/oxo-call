---
name: bowtie2
category: alignment
description: Fast and sensitive short read aligner for gapped alignment to reference genomes
tags: [alignment, mapping, short-read, ngs, illumina, bowtie2]
author: oxo-call built-in
source_url: "https://bowtie-bio.sourceforge.net/bowtie2/manual.shtml"
---

## Concepts

- bowtie2 index building requires the companion binary 'bowtie2-build ref.fa index_prefix'. When asked to build an index, output ARGS starting with 'bowtie2-build' — the system will use it as the actual executable automatically.
- Use -x index_prefix (without the .bt2 extension), -U for single-end or -1/-2 for paired-end reads.
- bowtie2 outputs SAM to stdout by default — pipe to samtools or use -S output.sam.
- The --very-sensitive preset improves sensitivity at the cost of speed; --fast and --very-fast are faster but less sensitive.
- Use -p N for multi-threading; --no-unal suppresses unmapped reads in output.
- Local alignment mode (--local) allows soft-clipping of the ends of reads; global mode (default, --end-to-end) requires full read alignment.

## Pitfalls

- bowtie2 outputs SAM to stdout — always pipe to 'samtools view -b -o output.bam' or use -S for SAM output.
- Index building uses the companion binary 'bowtie2-build', not 'bowtie2 -build'. Always start the ARGS with 'bowtie2-build' for index tasks; the system detects it and invokes it as the executable.
- The -x argument takes the index prefix (from 'bowtie2-build'), not the .fa file or any .bt2 file.
- For paired-end reads, -1 and -2 must be used (not -U); using -U with paired files treats them as single-end.
- The --very-sensitive-local mode allows soft-clipping which changes the CIGAR string — verify downstream tools support this.
- bowtie2 alignment rate in the log is to stderr; always check it after alignment.

## Examples

### build a bowtie2 index from a reference FASTA file
**Args:** `bowtie2-build reference.fa reference_index`
**Explanation:** bowtie2-build is the companion binary; creates reference_index.*.bt2 files used by bowtie2 -x

### build a bowtie2 index using multiple threads for a large genome
**Args:** `bowtie2-build --threads 8 reference.fa reference_index`
**Explanation:** bowtie2-build companion binary; --threads 8 parallelizes index construction for large references

### align paired-end reads to a reference genome using 8 threads
**Args:** `-x reference_index -1 R1.fastq.gz -2 R2.fastq.gz -p 8 | samtools view -b -o aligned.bam`
**Explanation:** -x is the index prefix (built by bowtie2-build); output is SAM piped to samtools view -b for BAM output

### align single-end reads with sensitive settings
**Args:** `-x reference_index -U reads.fastq.gz --very-sensitive -p 8 | samtools sort -o sorted.bam`
**Explanation:** --very-sensitive increases accuracy; output SAM piped directly to samtools sort

### align paired-end reads and save the alignment statistics
**Args:** `-x reference_index -1 R1.fq.gz -2 R2.fq.gz -p 8 --no-unal -S aligned.sam 2> align_stats.txt`
**Explanation:** --no-unal suppresses unmapped reads; 2> redirects alignment stats to a file

### align paired-end reads with read group tags for GATK downstream analysis
**Args:** `-x reference_index -1 R1.fastq.gz -2 R2.fastq.gz -p 8 --rg-id sample1 --rg SM:sample1 --rg LB:lib1 --rg PL:ILLUMINA | samtools view -b -o sample1.bam`
**Explanation:** --rg-id sets the read group ID; --rg adds RG tags required by GATK; output is BAM

### align in local mode to allow soft-clipping of read ends
**Args:** `-x reference_index -1 R1.fastq.gz -2 R2.fastq.gz --local --very-sensitive-local -p 8 | samtools view -b -o local_aligned.bam`
**Explanation:** --local enables local alignment with soft-clipping; --very-sensitive-local is the most accurate preset for local mode

### align paired-end RNA-seq reads discarding unaligned reads
**Args:** `-x reference_index -1 R1.fastq.gz -2 R2.fastq.gz -p 16 --no-unal | samtools sort -@ 4 -o sorted.bam`
**Explanation:** --no-unal saves disk space by not writing unmapped reads; 16 threads for speed; output sorted directly

### align single-end reads in fast mode for a quick quality check
**Args:** `-x reference_index -U reads.fastq.gz --fast -p 4 -S quick_check.sam`
**Explanation:** --fast trades sensitivity for speed; -S writes SAM file; useful for initial quality assessment

### align paired-end reads writing unmapped reads to separate files
**Args:** `-x reference_index -1 R1.fastq.gz -2 R2.fastq.gz -p 8 --un-conc unmapped_%.fq | samtools view -b -o aligned.bam`
**Explanation:** --un-conc writes unmapped read pairs to unmapped_1.fq and unmapped_2.fq for downstream analysis
