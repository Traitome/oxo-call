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
- modkit adjust-mods updates modification tags in BAM files with new probability thresholds.
- modkit validate checks the integrity of modification tags in a BAM file.
- modkit repair fixes missing or malformed MM/ML tags in BAM files.
- modkit sample-probs outputs the distribution of modification probabilities to help choose thresholds.

## Pitfalls

- Input BAM must be sorted and indexed; modkit pileup fails with unsorted input.
- Without --ref, modkit pileup cannot collapse CpG pairs on both strands; always provide the reference FASTA for CpG analysis.
- Not specifying --mod-code when the BAM has multiple modification types (e.g., 5mC and 5hmC) reports all types together; filter with --mod-code h or m.
- Very low coverage positions produce unreliable methylation fractions; filter output by the coverage column (column 10 >= 5) before analysis.
- modkit pileup on whole genomes requires substantial RAM; use --region to limit to specific chromosomes for testing.
- The --combine-strands option is only valid for CpG context with a reference; using it on non-CpG motifs produces incorrect results.
- --filter-threshold default is inferred from data but may not be optimal; use sample-probs to assess distribution.
- modkit repair should be used cautiously as it modifies BAM tags in place; always backup original BAM files.

## Examples

### generate a bedMethyl pileup of 5mC methylation from a BAM file
**Args:** `pileup --ref reference.fasta --mod-code m --cpg input.bam output.bedmethyl --threads 16`
**Explanation:** modkit pileup subcommand; --ref reference.fasta reference FASTA; --mod-code m 5-methylcytosine; --cpg restricts to CpG dinucleotides; input.bam modification BAM; output.bedmethyl output file; --threads 16 threads

### generate bedMethyl and combine CpG sites on both strands
**Args:** `pileup --ref reference.fasta --cpg --combine-strands -t 16 input.bam output_combined.bedmethyl`
**Explanation:** modkit pileup subcommand; --ref reference.fasta reference FASTA; --cpg CpG mode; --combine-strands merges strand counts; -t 16 threads; input.bam modification BAM; output_combined.bedmethyl output file

### extract per-read modification data to TSV
**Args:** `extract --ref reference.fasta --mod-code m input.bam per_read_mods.tsv --threads 16`
**Explanation:** modkit extract subcommand; --ref reference.fasta reference FASTA; --mod-code m 5-methylcytosine; input.bam modification BAM; per_read_mods.tsv output TSV; --threads 16 threads

### get a summary of modification calls in a BAM
**Args:** `summary input.bam --threads 8`
**Explanation:** modkit summary subcommand; input.bam modification BAM; --threads 8 threads; reports counts and fractions per modification type

### generate a BED file of all CpG positions in a reference for use as motif targets
**Args:** `motif-bed reference.fasta CG 0 > cpg_positions.bed`
**Explanation:** modkit motif-bed subcommand; reference.fasta input FASTA; CG motif; 0 offset to modified base; > cpg_positions.bed output BED

### pileup restricted to a specific genomic region
**Args:** `pileup --ref reference.fasta --region chr1:1-10000000 --mod-code m input.bam region_output.bedmethyl --threads 8`
**Explanation:** modkit pileup subcommand; --ref reference.fasta reference FASTA; --region chr1:1-10000000 chromosomal interval; --mod-code m 5-methylcytosine; input.bam modification BAM; region_output.bedmethyl output; --threads 8 threads

### sample modification probabilities to assess threshold distribution
**Args:** `sample-probs --mod-code m input.bam --threads 8`
**Explanation:** modkit sample-probs subcommand; --mod-code m 5-methylcytosine; input.bam modification BAM; --threads 8 threads; outputs probability distribution

### adjust modification thresholds in BAM file
**Args:** `adjust-mods --filter-threshold 0.8 input.bam output.bam --threads 8`
**Explanation:** modkit adjust-mods subcommand; --filter-threshold 0.8 probability cutoff; input.bam modification BAM; output.bam output BAM; --threads 8 threads

### validate modification tags in BAM file
**Args:** `validate input.bam --threads 8`
**Explanation:** modkit validate subcommand; input.bam modification BAM; --threads 8 threads; checks integrity of MM/ML tags

### repair malformed modification tags
**Args:** `repair input.bam output.bam --threads 8`
**Explanation:** modkit repair subcommand; input.bam modification BAM; output.bam output BAM; --threads 8 threads; fixes missing or malformed MM/ML tags

### pileup with custom filter threshold
**Args:** `pileup --ref reference.fasta --filter-threshold 0.7 --mod-code m input.bam output.bedmethyl --threads 16`
**Explanation:** modkit pileup subcommand; --ref reference.fasta reference FASTA; --filter-threshold 0.7 probability cutoff; --mod-code m 5-methylcytosine; input.bam modification BAM; output.bedmethyl output; --threads 16 threads

### extract modifications for specific motif
**Args:** `extract --ref reference.fasta --motif CG 0 --mod-code m input.bam cpg_mods.tsv --threads 16`
**Explanation:** modkit extract subcommand; --ref reference.fasta reference FASTA; --motif CG 0 CpG sites with offset; --mod-code m 5-methylcytosine; input.bam modification BAM; cpg_mods.tsv output TSV; --threads 16 threads
