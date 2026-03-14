---
name: modkit
category: epigenomics
description: Processing Oxford Nanopore base modification (methylation) BAM files for CpG and other modified bases
tags: [methylation, ont, nanopore, cpg, 5mc, 6ma, base-modification, epigenomics]
author: oxo-call built-in
source_url: "https://github.com/nanoporetech/modkit"
---

## Concepts

- modkit operates on MM/ML tag BAM files produced by Dorado or Guppy basecallers with modification calling enabled.
- modkit pileup aggregates per-read modification probabilities at each reference position, outputting a bedMethyl file.
- bedMethyl output has columns: chrom, start, end, mod_code, score, strand, coverage, fraction_modified — compatible with methylation analysis tools.
- modkit extract writes per-read modification records to a TSV for single-read analysis or machine learning input.
- modkit summary provides QC statistics on modification calls per read and per modification type.
- Threshold filtering (--filter-threshold) controls the confidence cutoff for counting a base as modified or canonical; default is inferred from data.

## Pitfalls

- Input BAM must be sorted and indexed; modkit pileup fails with unsorted input.
- Without --ref, modkit pileup cannot collapse CpG pairs on both strands; always provide the reference FASTA for CpG analysis.
- Not specifying --mod-code when the BAM has multiple modification types (e.g., 5mC and 5hmC) reports all types together; filter with --mod-code h or m.
- Very low coverage positions produce unreliable methylation fractions; filter output by the coverage column (column 10 >= 5) before analysis.
- modkit pileup on whole genomes requires substantial RAM; use --region to limit to specific chromosomes for testing.
- The --combine-strands option is only valid for CpG context with a reference; using it on non-CpG motifs produces incorrect results.

## Examples

### generate a bedMethyl pileup of 5mC methylation from a BAM file
**Args:** `pileup --ref reference.fasta --mod-code m --cpg input.bam output.bedmethyl --threads 16`
**Explanation:** --mod-code m selects 5-methylcytosine; --cpg restricts to CpG dinucleotides; --ref required for strand collapsing

### generate bedMethyl and combine CpG sites on both strands
**Args:** `pileup --ref reference.fasta --cpg --combine-strands -t 16 input.bam output_combined.bedmethyl`
**Explanation:** --combine-strands merges the + and - strand CpG counts into a single record per CpG site

### extract per-read modification data to TSV
**Args:** `extract --ref reference.fasta --mod-code m input.bam per_read_mods.tsv --threads 16`
**Explanation:** outputs one row per modified or canonical base call per read; useful for read-level methylation analysis

### get a summary of modification calls in a BAM
**Args:** `summary input.bam --threads 8`
**Explanation:** reports counts and fractions of each modification type per read; useful for QC before pileup

### generate a BED file of all CpG positions in a reference for use as motif targets
**Args:** `motif-bed reference.fasta CG 0 > cpg_positions.bed`
**Explanation:** CG is the motif; 0 is the offset to the modified base (C); output BED used with --include-bed in pileup

### pileup restricted to a specific genomic region
**Args:** `pileup --ref reference.fasta --region chr1:1-10000000 --mod-code m input.bam region_output.bedmethyl --threads 8`
**Explanation:** --region limits output to a chromosomal interval; useful for targeted analysis or testing

### sample modification probabilities to assess threshold distribution
**Args:** `sample-probs --mod-code m input.bam --threads 8`
**Explanation:** outputs the distribution of modification probabilities to help choose an appropriate --filter-threshold value
