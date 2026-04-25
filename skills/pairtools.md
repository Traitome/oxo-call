---
name: pairtools
category: epigenomics
description: Processing and analysis of Hi-C and similar chromosome conformation capture paired-end data
tags: [hi-c, chromatin-conformation, 3d-genome, pairs, contact-matrix, epigenomics]
author: oxo-call built-in
source_url: "https://pairtools.readthedocs.io/"
---

## Concepts
- pairtools is a workflow for Hi-C data processing: parsing alignments → sorting → deduplication → stats.
- pairtools parse: extracts chromatin contacts from bwa-mem aligned BAM; outputs .pairs format.
- pairtools sort: sorts .pairs file by chromosome for downstream analysis.
- pairtools dedup: removes PCR duplicates from sorted .pairs file.
- pairtools select: filters pairs by type (e.g., keep only trans pairs, or pairs with MAPQ>30).
- Use cooler to bin pairs into contact matrices (.cool format) for visualization with Higlass/cooler.
- distiller-nf is a Nextflow pipeline that wraps pairtools for end-to-end Hi-C processing.
- Hi-C pipeline: bwa mem → pairtools parse → pairtools sort → pairtools dedup → cooler cload.
- pairtools parse2: handles complex walks and multi-way ligation events in advanced Hi-C protocols.
- pairtools phase: assigns pairs to parental haplotypes for diploid genome analysis.
- pairtools restrict: assigns restriction fragments to pairs for restriction enzyme analysis.
- pairtools flip: flips pairs to ensure upper-triangular matrix format (chrom1 < chrom2 or pos1 < pos2).
- pairtools stats: generates comprehensive statistics on pair types, distances, and quality metrics.

## Pitfalls
- pairtools parse requires BWA-MEM aligned SAM/BAM — with -p flag for Hi-C specific parsing.
- Pairs files must be sorted before deduplication — run pairtools sort first.
- Without --output-stats in pairtools dedup, no statistics are generated for QC.
- pairtools generates .pairs.gz (bgzipped) files — use pairtools merge for combining multiple files.
- Hi-C alignment requires specific BWA flags: bwa mem -SP5M for Hi-C ligations.
- pairtools parse2 is for complex protocols (e.g., multi-contact); use parse for standard Hi-C.
- pairtools phase requires VCF with phased variants; unphased VCFs will not work.
- pairtools restrict needs restriction enzyme recognition sites in BED format.
- pairtools flip is required before cooler cload for proper matrix orientation.
- pairtools stats output is essential for QC; always generate for new datasets.

## Examples

### parse Hi-C BWA alignments to pairs format
**Args:** `parse --min-mapq 30 --walks-policy mask --max-inter-align-gap 30 -N sample --chroms-path chromsizes.txt sorted.bam > sample.pairs.gz`
**Explanation:** pairtools parse subcommand; --min-mapq 30 quality filter; --walks-policy mask multi-mapper handling; --max-inter-align-gap 30; -N sample name; --chroms-path chromsizes.txt chromosome sizes; sorted.bam input BAM; > sample.pairs.gz output

### sort pairs file for deduplication
**Args:** `sort sample.pairs.gz --nproc 16 --tmpdir /tmp/ > sample_sorted.pairs.gz`
**Explanation:** pairtools sort subcommand; sample.pairs.gz input pairs; --nproc 16 parallel sorting; --tmpdir /tmp/ temporary files; > sample_sorted.pairs.gz output

### deduplicate sorted pairs file
**Args:** `dedup --nproc 16 --output-stats dedup_stats.txt sample_sorted.pairs.gz > sample_dedup.pairs.gz`
**Explanation:** pairtools dedup subcommand; --nproc 16 threads; --output-stats dedup_stats.txt statistics; sample_sorted.pairs.gz input; > sample_dedup.pairs.gz output

### bin pairs into contact matrix using cooler
**Args:** `cload pairs --chrom1 2 --pos1 3 --chrom2 4 --pos2 5 chromsizes.txt:5000 sample_dedup.pairs.gz sample_5kb.cool`
**Explanation:** cooler cload pairs subcommand; --chrom1 2 --pos1 3 --chrom2 4 --pos2 5 column indices; chromsizes.txt:5000 chromosome sizes and 5000bp bin size; sample_dedup.pairs.gz input; sample_5kb.cool output

### flip pairs to upper-triangular format
**Args:** `flip sample.pairs.gz > sample_flipped.pairs.gz`
**Explanation:** pairtools flip subcommand; sample.pairs.gz input pairs; > sample_flipped.pairs.gz output; ensures upper-triangular format

### generate comprehensive pair statistics
**Args:** `stats sample_dedup.pairs.gz -o pair_stats.txt`
**Explanation:** pairtools stats subcommand; sample_dedup.pairs.gz input pairs; -o pair_stats.txt output statistics file

### phase pairs for diploid genome analysis
**Args:** `phase --vcf phased_variants.vcf.gz sample.pairs.gz > sample_phased.pairs.gz`
**Explanation:** pairtools phase subcommand; --vcf phased_variants.vcf.gz phased VCF; sample.pairs.gz input pairs; > sample_phased.pairs.gz output phased pairs

### assign restriction fragments to pairs
**Args:** `restrict --frags restriction_sites.bed sample.pairs.gz > sample_restricted.pairs.gz`
**Explanation:** pairtools restrict subcommand; --frags restriction_sites.bed restriction fragments BED; sample.pairs.gz input; > sample_restricted.pairs.gz output

### select high-quality cis pairs only
**Args:** `select '(pair_type == "UU") and (chrom1 == chrom2) and (mapq1 >= 30) and (mapq2 >= 30)' sample.pairs.gz > cis_hq.pairs.gz`
**Explanation:** pairtools select subcommand; '(pair_type == "UU") and (chrom1 == chrom2) and (mapq1 >= 30) and (mapq2 >= 30)' filter condition; sample.pairs.gz input; > cis_hq.pairs.gz output

### merge multiple pairs files
**Args:** `merge sample1.pairs.gz sample2.pairs.gz sample3.pairs.gz > combined.pairs.gz`
**Explanation:** pairtools merge subcommand; sample1.pairs.gz sample2.pairs.gz sample3.pairs.gz input pairs files; > combined.pairs.gz output merged pairs

### sample random subset of pairs
**Args:** `sample --number 1000000 sample.pairs.gz > subset.pairs.gz`
**Explanation:** pairtools sample subcommand; --number 1000000 random subset size; sample.pairs.gz input pairs; > subset.pairs.gz output
