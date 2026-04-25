---
name: wtdbg2
category: assembly
description: Ultrafast and memory-efficient long-read de novo assembler using fuzzy Bruijn graphs
tags: [assembly, long-read, nanopore, pacbio, de-novo, fast, draft-assembly]
author: oxo-call built-in
source_url: "https://github.com/ruanjue/wtdbg2"
---

## Concepts
- WTDBG2 (Redbean) assembles long reads using fuzzy Bruijn graphs; it's very fast but produces draft assemblies.
- Two-step process: (1) wtdbg2 for overlapping and assembly; (2) wtpoa-cns for consensus polishing.
- Use -x for read type: ont (Oxford Nanopore), rs (PacBio CLR/RSII), sq (PacBio Sequel), ccs (HiFi).
- Use -g for estimated genome size; -i for input reads; -fo for output prefix.
- WTDBG2 outputs a .ctg.lay.gz file for consensus and produces final .ctg.fa after wtpoa-cns.
- For ONT data: wtdbg2 -x ont -g 5m -i reads.fq -fo assembly; then wtpoa-cns for polishing.
- Additional polishing with Medaka (ONT) or PBCCS/Arrow (PacBio) is recommended after WTDBG2.
- -X selects best depth from input reads (default 50x); effective with -g.
- -L filters reads shorter than specified length; -L5000 recommended for PacBio.
- -k sets k-mer fsize (0-23); -p sets k-mer psize (0-23); k+p <= 25.
- -K filters high-frequency kmers (repetitive); default 1000.05.
- -S subsamples kmers (default 4.0 = 1/4); lower values increase memory but improve sensitivity.
- -l sets minimum alignment length (default 2048); -m sets minimum k-mer match length (default 200).
- -e sets minimum read depth for valid edges (default 3); adjust based on coverage.

## Pitfalls
- WTDBG2 skips error correction — raw assembly needs polishing before downstream analysis.
- The genome size estimate (-g) is required for assembly parameter tuning.
- WTDBG2 -x preset must match the read type — wrong preset degrades assembly quality.
- The consensus step (wtpoa-cns) is separate from the overlap step (wtdbg2).
- For complex or highly heterozygous genomes, WTDBG2 may produce fragmented assemblies.
- -S subsampling reduces memory but may miss overlaps; decrease for low coverage data.
- -K filtering removes repetitive kmers; too aggressive filtering may lose real overlaps.
- -L filtering drops short reads; balance between read length and coverage.
- ONT assemblies may be smaller than true genome size.
- Memory usage scales with genome size and coverage; large genomes need substantial RAM.

## Examples

### assemble genome from Oxford Nanopore reads
**Args:** `-x ont -g 5m -i reads.fastq.gz -fo assembly -t 16 && wtpoa-cns -t 16 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x ont ONT preset; -g 5m estimated genome size; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 16 threads; && wtpoa-cns -t 16 -i assembly.ctg.lay.gz -fo assembly.ctg.fa consensus polishing

### assemble genome from PacBio HiFi reads
**Args:** `-x ccs -g 3g -i hifi_reads.fastq.gz -fo hifi_assembly -t 32 && wtpoa-cns -t 32 -i hifi_assembly.ctg.lay.gz -fo hifi_assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x ccs CCS/HiFi preset; -g 3g 3 Gb genome size; -i hifi_reads.fastq.gz input HiFi reads; -fo hifi_assembly output prefix; -t 32 threads; && wtpoa-cns consensus

### assemble bacterial genome from PacBio CLR reads
**Args:** `-x rs -g 4m -i clr_reads.fastq.gz -fo clr_assembly -t 8 && wtpoa-cns -t 8 -i clr_assembly.ctg.lay.gz -fo clr_assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x rs PacBio RSII/CLR preset; -g 4m 4 Mb genome size; -i clr_reads.fastq.gz input CLR reads; -fo clr_assembly output prefix; -t 8 threads; && wtpoa-cns consensus

### assemble with increased k-mer sampling for low coverage
**Args:** `-x rs -g 5m -i reads.fastq.gz -fo assembly -t 16 -S 2 && wtpoa-cns -t 16 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x rs PacBio preset; -g 5m genome size; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 16 threads; -S 2 increases k-mer sampling to 1/2 (from default 1/4); && wtpoa-cns consensus

### assemble with longer minimum read length
**Args:** `-x rs -g 5m -i reads.fastq.gz -fo assembly -t 16 -L 10000 && wtpoa-cns -t 16 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x rs PacBio preset; -g 5m genome size; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 16 threads; -L 10000 filters reads shorter than 10kb; && wtpoa-cns consensus

### assemble with adjusted edge depth for high coverage
**Args:** `-x ont -g 3g -i reads.fastq.gz -fo assembly -t 32 -e 5 && wtpoa-cns -t 32 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x ont ONT preset; -g 3g 3 Gb genome; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 32 threads; -e 5 minimum edge depth; && wtpoa-cns consensus; reduces spurious connections

### assemble with custom k-mer parameters
**Args:** `-x ont -g 5m -i reads.fastq.gz -fo assembly -t 16 -k 15 -p 10 && wtpoa-cns -t 16 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x ont ONT preset; -g 5m genome size; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 16 threads; -k 15 -p 10 k-mer sizes (k+p <= 25); && wtpoa-cns consensus

### assemble with reduced depth selection
**Args:** `-x ont -g 3g -i reads.fastq.gz -fo assembly -t 32 -X 30 && wtpoa-cns -t 32 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x ont ONT preset; -g 3g genome size; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 32 threads; -X 30 selects best 30x depth (from default 50x); && wtpoa-cns consensus; faster for high coverage

### assemble with realignment mode
**Args:** `-x rs -g 5m -i reads.fastq.gz -fo assembly -t 16 -R && wtpoa-cns -t 16 -i assembly.ctg.lay.gz -fo assembly.ctg.fa`
**Explanation:** wtdbg2 command; -x rs PacBio preset; -g 5m genome size; -i reads.fastq.gz input reads; -fo assembly output prefix; -t 16 threads; -R enables realignment mode; && wtpoa-cns consensus; may improve accuracy
