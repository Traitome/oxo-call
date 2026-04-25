---
name: metabat2
category: metagenomics
description: Metagenome binning tool that clusters contigs into draft genomes using tetranucleotide frequency and coverage
tags: [metagenomics, binning, mag, assembly, contigs, clustering, genome]
author: oxo-call built-in
source_url: "https://bitbucket.org/berkeleylab/metabat"
---

## Concepts
- MetaBAT2 bins assembled contigs into metagenome-assembled genomes (MAGs) using tetranucleotide frequency + coverage.
- Coverage information is generated from BAM files using jgi_summarize_bam_contig_depths (bundled with MetaBAT2).
- Two-step workflow: (1) compute contig depths from BAMs; (2) run metabat2 with contigs + depth file.
- Use -i for input assembly FASTA; -a for coverage depth file; -o for output bin prefix.
- MetaBAT2 outputs bins as FASTA files named <prefix>.N.fa where N is the bin number.
- Use -m for minimum contig length (default 2500bp); shorter contigs are excluded from binning.
- After binning, assess bin quality with CheckM2 or BUSCO.
- --maxP controls percentage of 'good' contigs for binning (default 95); higher values increase sensitivity.
- --minS sets minimum edge score for binning (default 60); higher values (e.g., 80) increase specificity.
- --maxEdges limits edges per node (default 200); lower values reduce runtime but may decrease sensitivity.
- --unbinned outputs unbinned contigs to a separate file for downstream analysis.
- --seed enables reproducible binning results across runs; important for pipeline consistency.

## Pitfalls
- Coverage depth file must be generated from BAMs aligned to the SAME assembly used for binning.
- jgi_summarize_bam_contig_depths is part of MetaBAT2 — use this specific script, not other coverage tools.
- Without coverage information, MetaBAT2 bins only on tetranucleotide frequency (less accurate).
- Minimum contig length (-m 2500) is appropriate for most assemblies — reducing it adds noise.
- MetaBAT2 does not evaluate bin quality — always run CheckM2 or BUSCO after binning.
- Multiple BAMs (different samples/conditions) provide better coverage variation for binning accuracy.
- --minS values must be between 1-99; values outside this range cause errors.
- --maxP 100 includes all contigs; may reduce bin purity but increase completeness.
- Binning results are stochastic without --seed; use --seed for reproducibility.
- --cvExt is required when using coverage files from third-party tools (not jgi_summarize).

## Examples

### compute contig depths from BAM files for MetaBAT2
**Args:** `jgi_summarize_bam_contig_depths --outputDepth contig_depths.txt sample1.bam sample2.bam sample3.bam`
**Explanation:** jgi_summarize_bam_contig_depths command; --outputDepth contig_depths.txt output depth table; sample1.bam sample2.bam sample3.bam input BAM files

### bin metagenomic assembly contigs into MAGs
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix for bin FASTA files; -m 2500 min contig length; -t 8 threads

### run MetaBAT2 binning without coverage information (tetranucleotide only)
**Args:** `-i assembly.fasta -o bins/bin -m 1500 -t 8`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -o bins/bin output prefix; -m 1500 min contig length; -t 8 threads; without -a coverage file, MetaBAT2 uses only tetranucleotide frequency

### bin with custom sensitivity settings
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin --sensitive -m 2000 -t 8`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; --sensitive mode for more permissive binning; -m 2000 min contig length; -t 8 threads

### bin with high specificity for pure genomes
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8 --minS 80 --maxEdges 100`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --minS 80 increases specificity; --maxEdges 100 reduces edges for stringent binning

### output unbinned contigs for downstream analysis
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8 --unbinned`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --unbinned generates bin.unbinned.fa

### use seed for reproducible binning results
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8 --seed 42`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --seed 42 ensures identical results across runs

### bin with third-party coverage file
**Args:** `-i assembly.fasta -a custom_depths.txt -o bins/bin -m 2500 -t 8 --cvExt`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a custom_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --cvExt indicates coverage file lacks variance column

### set minimum bin size to filter small bins
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8 --minClsSize 500000`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --minClsSize 500000 excludes bins smaller than 500kb

### bin with multiple samples for improved differential coverage
**Args:** `jgi_summarize_bam_contig_depths --outputDepth multi_depth.txt sample1.bam sample2.bam sample3.bam sample4.bam sample5.bam && metabat2 -i assembly.fasta -a multi_depth.txt -o bins/bin -m 2500 -t 8`
**Explanation:** jgi_summarize_bam_contig_depths command; --outputDepth multi_depth.txt output; sample BAMs input; metabat2 command; -i assembly.fasta input; -a multi_depth.txt depth file; -o bins/bin output prefix; -m 2500 min contig; -t 8 threads

### run MetaBAT2 with verbose output for debugging
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8 --verbose`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --verbose prints detailed binning statistics

### bin with specific maxP and minS combination for balanced results
**Args:** `-i assembly.fasta -a contig_depths.txt -o bins/bin -m 2500 -t 8 --maxP 90 --minS 70`
**Explanation:** metabat2 command; -i assembly.fasta input assembly; -a contig_depths.txt depth file; -o bins/bin output prefix; -m 2500 min contig length; -t 8 threads; --maxP 90 includes 90% of contigs; --minS 70 moderate specificity

### assess bin quality with CheckM2 after binning
**Args:** `checkm2 predict -i bins/ -o checkm2_output -x fa -t 8`
**Explanation:** checkm2 predict subcommand; -i bins/ input directory; -o checkm2_output output directory; -x fa file extension; -t 8 threads

### combine MetaBAT2 with other binners using DASTool
**Args:** `DASTool -i metabat2_bins.tsv,maxbin2_bins.tsv -l metabat2,maxbin2 -c contig_depths.txt -t 8 -o das_tool_output`
**Explanation:** DASTool command; -i metabat2_bins.tsv,maxbin2_bins.tsv input bin files; -l metabat2,maxbin2 labels; -c contig_depths.txt depth file; -t 8 threads; -o das_tool_output output prefix

### extract bin statistics from MetaBAT2 output
**Args:** `ls bins/*.fa | while read f; do echo -n "$f: "; grep -c "^>" $f; done`
**Explanation:** count contigs per bin; quick assessment of bin size distribution; identify potential contamination (too many contigs)
