---
name: featurecounts
category: rna-seq
description: Efficient and accurate read summarization for RNA-seq and ChIP-seq aligned reads against genomic features
tags: [rna-seq, quantification, counts, gene-expression, gtf, subread, chip-seq, bam, junction]
author: oxo-call built-in
source_url: "https://subread.sourceforge.net/featureCounts.html"
---

## Concepts

- featureCounts is part of the Subread package; it counts reads mapping to genomic features (genes, exons) from BAM files.
- Use -a to specify the GTF/GFF annotation file; -o for the output count file; input BAM files are positional arguments.
- Use -T N for multi-threading; -p for paired-end counting (count read pairs, not individual reads).
- Strandedness: -s 0 (unstranded), -s 1 (forward-stranded), -s 2 (reverse-stranded/dUTP) — must match library prep.
- Default feature type is 'exon' (-t exon) aggregated to 'gene_id' (-g gene_id) — matches standard RNA-seq workflows.
- Use -O to allow multi-mapping reads to be counted (each feature they overlap); default requires unique assignment.
- --primary counts only primary alignments; --ignoreDup ignores marked duplicates.
- The output file has 6 annotation columns followed by one count column per BAM file — easy to parse for DESeq2.
- -J counts exon-exon junctions; output saved to <output>.jcounts; use -G with reference FASTA for improved accuracy.
- --fraction assigns fractional counts with -M/-O; 1/x for multi-mapping, 1/y for overlapping features.
- --splitOnly counts only split alignments (splice junction reads); --nonSplitOnly counts only non-split reads.
- --countReadPairs counts fragments instead of reads for paired-end data (used with -p).
- -L enables long read counting (Nanopore/PacBio); runs single-threaded, no CIGAR 'M' limit.
- -Q sets minimum mapping quality; -B requires both ends aligned; -C excludes chimeric pairs.

## Pitfalls

- featureCounts has NO subcommands. ARGS starts directly with flags (e.g., -T, -a, -o, -p, -s). Do NOT put a subcommand like 'count' or 'summarize' before flags.
- Wrong strandedness (-s 0/1/2) is the most common error — check your library prep protocol or use Salmon/RSeQC to determine.
- For paired-end data, use -p flag — without it, each read is counted individually instead of each read pair.
- The GTF/GFF feature_type (-t) and attribute (-g) must match what is in your annotation file.
- featureCounts requires coordinate-sorted indexed BAM files — unsorted input produces incorrect counts.
- Reads counted in multiple features (multi-mapping) are discarded by default; add -O for permissive counting.
- The output file header begins with '# Program:featureCounts...' — strip this when importing into R/Python.
- -J (junction counting) requires reference FASTA (-G) for accurate results; otherwise may miss junctions.
- --fraction must be used with -M or -O; alone it has no effect.
- -L (long reads) runs single-threaded regardless of -T setting; use for Nanopore/PacBio only.
- --splitOnly and --nonSplitOnly are mutually exclusive; cannot use both together.
- -s can take comma-separated values for per-file strandedness when counting multiple BAMs with different protocols.

## Examples

### count reads per gene for paired-end RNA-seq with reverse-strand library
**Args:** `-T 8 -a genes.gtf -o counts.txt -p -s 2 sample1.bam sample2.bam sample3.bam`
**Explanation:** -p paired-end; -s 2 reverse-strand (dUTP); -T 8 threads; multiple BAMs counted in one run

### count reads per gene for unstranded single-end RNA-seq
**Args:** `-T 8 -a genes.gtf -o counts.txt -s 0 sample.bam`
**Explanation:** -s 0 unstranded; outputs count matrix with 1 count column per BAM file

### count reads allowing multi-mapping reads to be counted
**Args:** `-T 8 -a genes.gtf -o counts.txt -p -s 2 --primary -M -O sample.bam`
**Explanation:** --primary counts only primary alignments; -M allows multi-mappers; -O allows reads to count to multiple features

### count ChIP-seq reads per peak region using BED file
**Args:** `-T 4 -a peaks.saf -F SAF -o chip_counts.txt sample_sorted.bam`
**Explanation:** -F SAF for simple annotation format (BED-like); -a peaks.saf; used for ChIP/ATAC peak quantification

### count exon-level reads for exon usage analysis
**Args:** `-T 8 -f -a genes.gtf -o exon_counts.txt -p -s 2 sample.bam`
**Explanation:** -f counts at feature level (exon) instead of meta-feature (gene); used for exon usage/splicing analysis

### count exon-exon junctions for splicing analysis
**Args:** `-T 8 -a genes.gtf -o counts.txt -J -G reference.fa -p -s 2 sample.bam`
**Explanation:** -J counts junctions; -G provides reference FASTA for accurate junction detection; .jcounts file created

### count with fractional assignment for multi-mapping reads
**Args:** `-T 8 -a genes.gtf -o counts.txt -p -s 2 -M -O --fraction sample.bam`
**Explanation:** --fraction assigns 1/x count to each multi-mapping alignment; more accurate than integer counting

### count only split alignments (splice junction reads)
**Args:** `-T 8 -a genes.gtf -o counts.txt -p -s 2 --splitOnly sample.bam`
**Explanation:** --splitOnly counts only reads spanning splice junctions; useful for splicing-focused analysis

### count long reads from Nanopore/PacBio
**Args:** `-a genes.gtf -o counts.txt -L -s 0 long_reads.bam`
**Explanation:** -L enables long read mode; runs single-threaded; no CIGAR 'M' operation limit

### count with fragment-level quantification
**Args:** `-T 8 -a genes.gtf -o counts.txt -p --countReadPairs -s 2 sample.bam`
**Explanation:** --countReadPairs counts fragments instead of reads; proper paired-end quantification

### count with minimum mapping quality filter
**Args:** `-T 8 -a genes.gtf -o counts.txt -p -s 2 -Q 20 --primary sample.bam`
**Explanation:** -Q 20 requires MAPQ >= 20; --primary counts only primary alignments; higher confidence counts

### count with chimeric read exclusion
**Args:** `-T 8 -a genes.gtf -o counts.txt -p -s 2 -B -C sample.bam`
**Explanation:** -B requires both ends aligned; -C excludes chimeric pairs (different chromosomes/strands); cleaner counts
