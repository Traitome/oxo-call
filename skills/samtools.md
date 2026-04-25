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
- samtools mpileup generates pileup format for variant calling (use bcftools mpileup for VCF/BCF output).
- samtools consensus produces consensus sequences from alignments in FASTA/FASTQ/PILEUP format.
- samtools collate groups alignments by name without full sorting; faster than sort -n for some workflows.
- samtools coverage computes alignment depth and percent coverage per chromosome or region.
- samtools quickcheck verifies if SAM/BAM/CRAM files appear intact without full validation.

## Pitfalls

- samtools ARGS must start with a subcommand (view, sort, index, flagstat, fastq, fasta, markdup, merge, depth, stats, fixmate, mpileup, faidx, fqidx, dict, calmd, collate, reheader, addreplacerg, ampliconclip, cat, consensus, split, import, reference, reset, bedcov, coverage, idxstats, phase, ampliconstats, checksum, flags, head, tview, depad, samples) — never with flags like -b, -@, -o. The subcommand ALWAYS comes first.
- Without -o, samtools writes to stdout — pipe carefully or always use -o output.bam.
- CRAM output (-C flag in view, or -O cram in sort) requires --reference; omitting it causes an error.
- samtools index on an unsorted BAM will appear to succeed but region queries will give wrong results.
- samtools view without -b or -O bam outputs SAM text, not BAM — the file will be much larger.
- samtools sort -n sorts by read name (needed before fixmate/markdup); the default is coordinate sort.
- Piping samtools sort to samtools index does not work — sort must complete and write a file first.
- markdup requires fixmate -m to have been run first; running markdup directly on coordinate-sorted BAM without fixmate will not correctly detect duplicates.
- samtools mpileup no longer outputs VCF/BCF; use bcftools mpileup for variant calling instead.
- samtools consensus requires an indexed BAM for region-specific consensus with -r option.
- The -@ threads option affects decompression threads, not all operations; some commands like index are single-threaded.

## Examples

### sort a BAM file by genomic coordinates
**Args:** `sort -o sorted.bam input.bam`
**Explanation:** samtools sort subcommand; -o sorted.bam output BAM; input.bam input file; coordinate sort is the default

### create an index for a sorted BAM file
**Args:** `index sorted.bam`
**Explanation:** samtools index subcommand; sorted.bam input BAM; creates sorted.bam.bai index file; must be run on a coordinate-sorted BAM

### filter to keep only properly paired primary alignments
**Args:** `view -b -f 2 -F 256 -F 2048 -o proper_paired.bam input.bam`
**Explanation:** samtools view subcommand; -b outputs BAM format; -f 2 keeps properly paired; -F 256 removes secondary; -F 2048 removes supplementary; -o proper_paired.bam output BAM; input.bam input file

### get alignment statistics (mapped, unmapped, duplicates)
**Args:** `flagstat input.bam`
**Explanation:** samtools flagstat subcommand; input.bam input BAM; outputs counts for each alignment category to stdout; redirect with > stats.txt to save

### convert BAM to FASTQ for paired-end reads
**Args:** `fastq -1 R1.fastq.gz -2 R2.fastq.gz -0 /dev/null -s /dev/null -n input.bam`
**Explanation:** samtools fastq subcommand; -1 R1.fastq.gz read 1 output; -2 R2.fastq.gz read 2 output; -0 /dev/null unpaired output; -s /dev/null supplementary output; -n preserves original read names; input.bam input BAM

### extract reads mapping to chromosome 1 between 100000 and 200000
**Args:** `view -b -o region.bam input.bam chr1:100000-200000`
**Explanation:** samtools view subcommand; -b outputs BAM; -o region.bam output BAM; input.bam input file; chr1:100000-200000 region specification; region queries require sorted + indexed BAM

### mark PCR duplicates
**Args:** `markdup -f stats.txt input_namesorted.bam output_markdup.bam`
**Explanation:** samtools markdup subcommand; -f stats.txt duplicate stats output; input_namesorted.bam name-sorted input BAM; output_markdup.bam output BAM; input must be name-sorted then fixmate'd

### merge multiple BAM files into one
**Args:** `merge -f merged.bam sample1.bam sample2.bam sample3.bam`
**Explanation:** samtools merge subcommand; -f overwrites output if exists; merged.bam output BAM; sample1.bam sample2.bam sample3.bam input BAMs; all inputs should be sorted

### compute per-base depth of coverage
**Args:** `depth -a -o coverage.txt input.bam`
**Explanation:** samtools depth subcommand; -a includes positions with zero coverage; -o coverage.txt output file; input.bam input BAM

### view the BAM header
**Args:** `view -H input.bam`
**Explanation:** samtools view subcommand; -H output header only; input.bam input BAM; outputs only the header lines (starting with @) to stdout

### sort BAM by read name for fixmate preprocessing
**Args:** `sort -n -o namesorted.bam input.bam`
**Explanation:** samtools sort subcommand; -n sorts by read name; -o namesorted.bam output BAM; input.bam input file; required before fixmate and markdup

### add mate information required for duplicate marking
**Args:** `fixmate -m namesorted.bam fixmate.bam`
**Explanation:** samtools fixmate subcommand; -m adds mate score tags needed by markdup; namesorted.bam name-sorted input BAM; fixmate.bam output BAM; input must be name-sorted; output is still name-sorted

### convert BAM to CRAM with reference for smaller storage
**Args:** `view -C --reference reference.fa -o output.cram input.bam`
**Explanation:** samtools view subcommand; -C outputs CRAM format; --reference reference.fa required for CRAM; -o output.cram output CRAM; input.bam input BAM; much smaller than BAM for WGS data

### calculate insert size and coverage statistics
**Args:** `stats input.bam > stats.txt`
**Explanation:** samtools stats subcommand; input.bam input BAM; > stats.txt output statistics file; outputs comprehensive statistics including insert size distribution, coverage, and error rates

### sort BAM using coordinate sort with temporary directory
**Args:** `sort -m 2G -T /tmp/sort_tmp -o sorted.bam input.bam`
**Explanation:** samtools sort subcommand; -m 2G limits per-thread memory; -T /tmp/sort_tmp temporary directory; -o sorted.bam output BAM; input.bam input file; avoids filling default tmpdir

### generate pileup for variant calling
**Args:** `mpileup -f reference.fa -o output.pileup input.bam`
**Explanation:** samtools mpileup subcommand; -f reference.fa reference FASTA; -o output.pileup pileup output file; input.bam input BAM; outputs pileup format for downstream analysis; use bcftools mpileup for direct VCF output

### generate consensus sequence from alignments
**Args:** `consensus -f FASTA -o consensus.fa input.bam`
**Explanation:** samtools consensus subcommand; -f FASTA output format; -o consensus.fa output FASTA; input.bam input BAM; produces consensus sequence from aligned reads; useful for viral genomes or amplicon sequencing

### collate alignments by name without full sorting
**Args:** `collate -o collated.bam input.bam`
**Explanation:** samtools collate subcommand; -o collated.bam output BAM; input.bam input BAM; groups reads by name faster than sort -n; useful for workflows needing paired reads together without strict name ordering

### compute coverage statistics per chromosome
**Args:** `coverage -o coverage.txt input.bam`
**Explanation:** samtools coverage subcommand; -o coverage.txt output file; input.bam input BAM; outputs depth and percent coverage per chromosome; useful for assessing sequencing completeness across the genome

### quickly check if BAM file is intact
**Args:** `quickcheck input.bam`
**Explanation:** samtools quickcheck subcommand; input.bam input BAM; fast integrity check without full validation; exits with non-zero status if file appears corrupted; useful for pipeline validation steps

### extract FASTA from BAM
**Args:** `fasta -o output.fa input.bam`
**Explanation:** samtools fasta subcommand; -o output.fa output FASTA; input.bam input BAM; converts BAM to FASTA format; useful for extracting sequences from aligned reads

### calculate depth per BED region
**Args:** `bedcov regions.bed input.bam > coverage.bed`
**Explanation:** samtools bedcov subcommand; regions.bed BED regions input; input.bam input BAM; > coverage.bed output BED with coverage; computes read depth for each region in BED file; outputs BED with additional column for total base count
