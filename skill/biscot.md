---
name: biscot
category: bisulfite_seq
description: A tool for read-level analysis of bisulfite sequencing data to assess C-to-T conversion rates and methylation patterns in aligned reads.
tags: [bisulfite, methylation, bs-seq, conversion, cytosine, read-level]
author: AI-generated
source_url: https://github.com/biscot-team/biscot
---

## Concepts

- **I/O Model:** biscot takes sorted BAM/SAM alignment files as input, where reads must be aligned to a reference genome that has been pre-indexed. It outputs conversion statistics per read and aggregate metrics to stdout or files.
- **Cytosine Conversion Tracking:** The tool identifies individual cytosine positions in reads and determines whether they appear as cytosine (unconverted) or thymine (converted) in the alignment, enabling read-level methylation calling.
- **Strand-Specific Analysis:** biscot distinguishes between canonical Watson (CT) reads and complementary Crick (GA) reads that arise from bisulfite conversion, reporting conversion rates separately for each strand.
- **Filtering Criteria:** Reads can be filtered based on minimum mapping quality (MAPQ), minimum number of CpG sites covered, and maximum mismatches to remove low-quality alignments from analysis.

## Pitfalls

- **Using unindexed references:** Running biscot without first creating a reference index using the companion `biscot-build` command will cause the tool to fail at runtime with an obscure error.
- **Mixing incompatible read types:** Including bisulfite-converted libraries that differ in酶mentation protocols ( directional vs non-directional) in the same analysis will produce misleading conversion rates.
- **Neglecting read orientation:** Treating all reads as having the same strand orientation results in incorrect identification of converted cytosines because the conversion pattern differs between CT and GA strands.
- **Ignoring PCR duplicates:** Failing to mark or filter duplicate reads before analysis inflates coverage and creates bias toward PCR-amplified alleles, skewing conversion rate calculations.

## Examples

### Calculate conversion rate for a single BAM file
**Args:** -i sample.bam -o conversion_report.txt
**Explanation:** This runs read-level conversion analysis on an aligned bisulfite-seq BAM file and saves per-read conversion metrics to the specified output file for downstream interpretation.

### Analyze with high-quality read filter
**Args:** -i sample.bam --min-mapq 30 --min-cpg 3
**Explanation:** Requiring a minimum mapping quality of 30 and at least 3 CpG sites per read removes low-confidence alignments that could introduce noise into methylation estimates.

### Output verbose per-cycle statistics
**Args:** -i sample.bam -v --per-cycle-stats cycle_metrics.txt
**Explanation:** Enabling verbose output with per-cycle breakdown reveals conversion patterns at each position in reads, helpful for identifying systematic biases.

### Specify context-specific analysis for CpG sites only
**Args:** -i sample.bam --context CpG --min-depth 5
**Explanation:** Focusing analysis exclusively on CpG dinucleotides with at least 5x coverage ensures robust methylation estimates while reducing computational overhead.

### Run batch mode on multiple samples
**Args:** -i samples.txt --batch --output-dir ./results/
**Explanation:** Processing multiple BAM files listed in an input file (one per line) in batch mode automates large-scale studies and organizes outputs into a dedicated directory.

### Compare conversion rates between two conditions
**Args:** -i treated.bam -c control.bam --compare -o comparison.txt
**Explanation:** Computing conversion rate differences between a treated sample and control enables quantitative assessment of treatment effects on methylation patterns.