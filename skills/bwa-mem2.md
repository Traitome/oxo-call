---
name: bwa-mem2
category: alignment
description: Faster version of BWA-MEM with 2-3x speedup for short read alignment using SIMD acceleration
tags: [alignment, mapping, short-read, ngs, illumina, bwa, reference, simd, avx512]
author: oxo-call built-in
source_url: "https://github.com/bwa-mem2/bwa-mem2"
---

## Concepts

- BWA-MEM2 is a drop-in replacement for BWA-MEM with 2-3x faster alignment using SIMD instructions (AVX512/AVX2/SSE4.1).
- All BWA-MEM flags and parameters are compatible with BWA-MEM2 — use the same arguments (-t, -R, -x, -M, -Y, -B, -O, -E, etc.).
- Index build: bwa-mem2 index ref.fa; creates .bwt.2bit.64, .ann, .amb, .pac, .0123 files.
- Use -t N for threads; -R for read group (required by GATK); outputs SAM to stdout.
- BWA-MEM2 requires more disk space for the index than BWA (about 6x genome size).
- BWA-MEM2 automatically selects the best SIMD instruction set at runtime; prints which mode is active (e.g., "Executing in AVX512 mode").
- For GATK best practices, add read group: -R '@RG\tID:id\tSM:sample\tLB:lib\tPL:ILLUMINA'.
- -x presets supported: pacbio, ont2d, intractg — same as BWA-MEM.
- -M marks shorter split hits as secondary (Picard compatibility); -Y soft-clips supplementary alignments (SV calling).

## Pitfalls

- bwa-mem2 ARGS must start with 'mem' or 'index' — never with flags like -t or -R. The subcommand ALWAYS comes first.
- BWA-MEM2 index is NOT interchangeable with BWA index — must re-index with bwa-mem2 index.
- On older CPUs without AVX2/SSE4.1 support, BWA-MEM2 may not run or may fall back to generic mode.
- BWA-MEM2 uses more RAM than BWA during alignment due to pre-loaded index (~65 GB for human genome with AVX512).
- Output is SAM to stdout — always pipe to samtools view -b or redirect to a file.
- For paired-end data with GATK downstream, always include read groups with -R flag.
- The -R read group string must preserve all field values exactly as specified — do not simplify or shorten sample names, IDs, or library tags.

## Examples

### build BWA-MEM2 index from reference genome
**Args:** `index reference.fa`
**Explanation:** index subcommand; reference.fa input FASTA; creates reference.fa.* index files; may take 30-60 min and ~60 GB RAM for human genome

### align paired-end reads to reference using 16 threads
**Args:** `mem -t 16 reference.fa R1.fastq.gz R2.fastq.gz | samtools sort -@ 4 -o sorted.bam`
**Explanation:** mem subcommand; -t 16 threads; reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; BWA-MEM2 mem has same flags as BWA-MEM; pipe to samtools sort for sorted BAM

### align paired-end reads with GATK read group
**Args:** `mem -t 16 -R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA' reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o aligned.bam`
**Explanation:** mem subcommand; -t 16 threads; -R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA' adds read group required for GATK downstream; reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; pipe to samtools view for BAM output; same syntax as BWA

### align long reads with preset
**Args:** `mem -t 8 -x ont2d reference.fa reads.fastq | samtools view -b -o aligned.bam`
**Explanation:** mem subcommand; -t 8 threads; -x ont2d preset for Oxford Nanopore; reference.fa indexed reference; reads.fastq input; pipe to samtools view for BAM output; -x pacbio for PacBio; -x intractg for intra-species contigs

### align with soft-clipping for structural variant calling
**Args:** `mem -t 16 -Y -M reference.fa R1.fastq.gz R2.fastq.gz | samtools sort -o sorted.bam`
**Explanation:** mem subcommand; -t 16 threads; -Y soft-clips supplementary alignments; -M marks shorter split hits as secondary (Picard compatible); reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; pipe to samtools sort; recommended for SV callers

### full pipeline with read group, sort, and index
**Args:** `mem -t 16 -R '@RG\tID:s1\tSM:s1\tLB:lib1\tPL:ILLUMINA' reference.fa R1.fq.gz R2.fq.gz | samtools sort -@ 4 -o s1.bam && samtools index s1.bam`
**Explanation:** mem subcommand; -t 16 threads; -R '@RG\tID:s1\tSM:s1\tLB:lib1\tPL:ILLUMINA' read group; reference.fa indexed reference; R1.fq.gz R2.fq.gz paired-end input; pipe to samtools sort → samtools index; complete pipeline: align with RG → sort → index; preserving exact identifiers

### align with split alignment output for chimeric reads
**Args:** `mem -t 16 -M -a reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o split_align.bam`
**Explanation:** mem subcommand; -t 16 threads; -M marks secondary alignments; -a outputs all alignments for split reads; reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; pipe to samtools view; output split_align.bam; useful for fusion gene detection and chimeric read analysis

### align with custom gap open and extension penalties
**Args:** `mem -t 16 -O 6 -E 1 reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o aligned.bam`
**Explanation:** mem subcommand; -t 16 threads; -O 6 gap open penalty; -E 1 gap extension penalty; reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; pipe to samtools view; higher gap penalties reduce false alignments in repetitive regions

### align with clipping penalty adjustment for soft-clipped reads
**Args:** `mem -t 16 -L 5 reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o aligned.bam`
**Explanation:** mem subcommand; -t 16 threads; -L 5 clipping penalty; reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; pipe to samtools view; lower values allow more soft-clipping; useful for reads with adapter contamination or partial alignments

### align multiple samples in batch with consistent read groups
**Args:** `mem -t 16 -R '@RG\tID:${sample}\tSM:${sample}\tLB:lib1\tPL:ILLUMINA' reference.fa ${sample}_R1.fq.gz ${sample}_R2.fq.gz | samtools sort -@ 4 -o ${sample}.bam`
**Explanation:** mem subcommand; -t 16 threads; -R '@RG\tID:${sample}\tSM:${sample}\tLB:lib1\tPL:ILLUMINA' read group with shell variable expansion; reference.fa indexed reference; ${sample}_R1.fq.gz ${sample}_R2.fq.gz paired-end input; pipe to samtools sort; batch processing; essential for multi-sample GATK pipelines

### align with minimum seed length adjustment for short reads
**Args:** `mem -t 16 -k 19 reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o aligned.bam`
**Explanation:** mem subcommand; -t 16 threads; -k 19 minimum seed length; reference.fa indexed reference; R1.fastq.gz R2.fastq.gz paired-end input; pipe to samtools view; default k=19; increase for longer reads to reduce false seeds, decrease for very short reads (<50bp)

### check SIMD mode and CPU compatibility before alignment
**Args:** `mem -t 1 reference.fa test_R1.fq.gz test_R2.fq.gz 2>&1 | head -5`
**Explanation:** mem subcommand; -t 1 thread; reference.fa indexed reference; test_R1.fq.gz test_R2.fq.gz paired-end input; 2>&1 captures stderr; head -5 shows first lines; BWA-MEM2 prints SIMD mode (AVX512/AVX2/SSE4.1) at startup; verify CPU compatibility before large-scale alignment jobs
