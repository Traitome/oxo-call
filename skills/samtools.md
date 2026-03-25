---
name: samtools
category: alignment
description: Suite of programs for interacting with high-throughput sequencing data in SAM/BAM/CRAM format
tags: [bam, sam, cram, alignment, ngs, sequencing, indexing, sorting]
author: oxo-call built-in
source_url: "http://www.htslib.org/doc/samtools.html"
---

## Concepts

- SAM is plain text, BAM is binary (smaller/faster), CRAM is reference-compressed (smallest). Use BAM for most workflows.
- BAM files MUST be coordinate-sorted (samtools sort) BEFORE indexing (samtools index). Random-access region queries require both steps.
- Use -@ N to enable N extra threads; -o FILE to write to a file instead of stdout; use -b flag to output BAM.
- samtools view filters reads: -F N excludes reads with flag N set; -f N keeps only reads with flag N set. Common flags: 4=unmapped, 256=secondary, 2048=supplementary.
- CRAM output requires --reference /path/to/ref.fa because it stores differences from the reference.
- Many subcommands (view, sort, flagstat) accept a region like chr1:1000-2000 to limit output.
- Complete PCR duplicate marking workflow: (1) sort by name with 'sort -n', (2) fixmate with '-m', (3) sort by coordinate, (4) markdup.

## Pitfalls

- Without -o, samtools writes to stdout — pipe carefully or always use -o output.bam.
- CRAM output (-C flag in view, or -O cram in sort) requires --reference; omitting it causes an error.
- samtools index on an unsorted BAM will appear to succeed but region queries will give wrong results.
- samtools view without -b or -O bam outputs SAM text, not BAM — the file will be much larger.
- samtools sort -n sorts by read name (needed before fixmate/markdup); the default is coordinate sort.
- Piping samtools sort to samtools index does not work — sort must complete and write a file first.
- markdup requires fixmate -m to have been run first; running markdup directly on coordinate-sorted BAM without fixmate will not correctly detect duplicates.

## Examples

### sort a BAM file by genomic coordinates
**Args:** `sort -@ 4 -o sorted.bam input.bam`
**Explanation:** -@ 4 uses 4 threads; -o writes BAM file; coordinate sort is the default

### create an index for a sorted BAM file
**Args:** `index sorted.bam`
**Explanation:** creates sorted.bam.bai; must be run on a coordinate-sorted BAM

### filter to keep only properly paired primary alignments
**Args:** `view -b -f 2 -F 256 -F 2048 -o proper_paired.bam input.bam`
**Explanation:** -f 2 keeps properly paired; -F 256 removes secondary; -F 2048 removes supplementary; -b outputs BAM

### get alignment statistics (mapped, unmapped, duplicates)
**Args:** `flagstat input.bam`
**Explanation:** outputs counts for each alignment category to stdout; redirect with > stats.txt to save

### convert BAM to FASTQ for paired-end reads
**Args:** `fastq -@ 4 -1 R1.fastq.gz -2 R2.fastq.gz -0 /dev/null -s /dev/null -n input.bam`
**Explanation:** -1/-2 for read 1/2; -0 for unpaired; -s for supplementary; -n preserves original read names

### extract reads mapping to chromosome 1 between 100000 and 200000
**Args:** `view -b -o region.bam input.bam chr1:100000-200000`
**Explanation:** region queries require a sorted + indexed BAM; -b outputs BAM

### mark PCR duplicates
**Args:** `markdup -@ 4 -f stats.txt input_namesorted.bam output_markdup.bam`
**Explanation:** input must be name-sorted (samtools sort -n), then fixmate'd; -f writes duplicate marking statistics

### merge multiple BAM files into one
**Args:** `merge -@ 4 -f merged.bam sample1.bam sample2.bam sample3.bam`
**Explanation:** -f overwrites output if it exists; all inputs should be sorted

### compute per-base depth of coverage
**Args:** `depth -a -o coverage.txt input.bam`
**Explanation:** -a includes positions with zero coverage; -o writes to file

### view the BAM header
**Args:** `view -H input.bam`
**Explanation:** outputs only the header lines (starting with @) to stdout

### sort BAM by read name for fixmate preprocessing
**Args:** `sort -n -@ 4 -o namesorted.bam input.bam`
**Explanation:** -n sorts by read name (required before fixmate and markdup); -@ 4 uses 4 threads

### add mate information required for duplicate marking
**Args:** `fixmate -m -@ 4 namesorted.bam fixmate.bam`
**Explanation:** -m adds mate score tags needed by markdup; input must be name-sorted; output is still name-sorted

### convert BAM to CRAM with reference for smaller storage
**Args:** `view -C --reference reference.fa -o output.cram input.bam`
**Explanation:** -C outputs CRAM format; --reference is required for CRAM; much smaller than BAM for WGS data

### calculate insert size and coverage statistics
**Args:** `stats -@ 4 input.bam > stats.txt`
**Explanation:** outputs comprehensive statistics including insert size distribution, coverage, and error rates

### sort BAM using coordinate sort with temporary directory
**Args:** `sort -@ 8 -m 2G -T /tmp/sort_tmp -o sorted.bam input.bam`
**Explanation:** -m limits per-thread memory; -T sets temporary directory to avoid filling default tmpdir
