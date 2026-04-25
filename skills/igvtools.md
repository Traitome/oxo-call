---
name: igvtools
category: utilities
description: IGV tools for preprocessing genomic data files for visualization in the Integrative Genomics Viewer
tags: [visualization, igv, bam, vcf, bigwig, tdf, genome-browser, coverage]
author: oxo-call built-in
source_url: "https://software.broadinstitute.org/software/igv/igvtools"
---

## Concepts

- IGVtools preprocesses BAM, VCF, BED, and other files for fast loading in IGV browser.
- Main commands: toTDF (generate TDF coverage), index (create .idx index), count (coverage density), sort, formatexp.
- Use 'igvtools toTDF' to convert wig/cn/igv/gct to IGV's .tdf format for fast zoom-level loading.
- Use 'igvtools index' to create .idx index for tab-indexed formats (not BAM — use samtools for that).
- Use 'igvtools count' to generate coverage tracks from BAM/SAM/BED/PSL files; outputs TDF or WIG.
- Use 'igvtools sort' to sort files by chromosome and position before indexing or toTDF.
- Use 'igvtools formatexp' to center, scale, and log2 normalize expression files.
- IGVtools is bundled with IGV desktop application; also available as standalone command line tool.
- TDF files are IGV-specific binary format for fast zoom-level access; WIG is ASCII format.
- Genome argument can be an ID (hg38, mm10), a .chrom.sizes file, or an IGV .genome file.

## Pitfalls

- igvtools index is for IGV-specific formats (VCF, BED, GFF) — NOT for BAM (use samtools index).
- TDF files are IGV-specific — not compatible with other genome browsers (use bigWig for universal coverage).
- igvtools sort creates a sorted version — input must be sorted for indexed access.
- The genome parameter specifies the genome build: hg38, mm10, etc.
- Input files for toTDF (wig, cn, igv, gct) must be sorted by start position; use igvtools sort first.
- -z zoom level (default 7) controls precomputed resolution; lower values reduce file size but decrease performance.
- -w window size (default 25bp) for count command; smaller windows give higher resolution but larger files.
- -e extend reads option is useful for ChIP-seq/RNA-seq; set to average fragment length.
- -f functions (min,max,mean) for count/toTDF; default is mean only.
- Memory settings may need adjustment with -Xmx parameter when running via Java directly.

## Examples

### create coverage TDF track from BAM file
**Args:** `count -z 5 -w 25 sorted.bam coverage.tdf hg38`
**Explanation:** count subcommand; -z 5 max zoom level; -w 25 window size; sorted.bam input BAM; coverage.tdf output TDF; hg38 genome

### index a VCF file for IGV
**Args:** `index variants.vcf`
**Explanation:** index subcommand; variants.vcf input VCF; creates .idx for fast random access in IGV

### sort a BED file for IGV indexing
**Args:** `sort input.bed sorted.bed`
**Explanation:** sort subcommand; input.bed input BED; sorted.bed output sorted BED; required before igvtools index

### convert wig file to TDF format
**Args:** `toTDF -z 5 -f mean input.wig output.tdf hg38`
**Explanation:** toTDF subcommand; -z 5 max zoom level; -f mean function; input.wig input wiggle file; output.tdf output TDF; hg38 genome

### generate coverage with extended reads for ChIP-seq
**Args:** `count -z 5 -w 50 -e 200 -f mean input.bam coverage.tdf hg38`
**Explanation:** count subcommand; -z 5 max zoom level; -w 50 window size; -e 200 extends reads by 200bp (fragment length); -f mean function; input.bam input BAM; coverage.tdf output TDF; hg38 genome

### format expression file with log2 normalization
**Args:** `formatexp -c input.gct output.gct`
**Explanation:** formatexp subcommand; -c centers data; input.gct input expression file; output.gct output file; applies log2 transformation

### output coverage as WIG instead of TDF
**Args:** `count -z 5 -w 25 input.bam coverage.wig hg38`
**Explanation:** count subcommand; -z 5 max zoom level; -w 25 window size; input.bam input BAM; coverage.wig output WIG; hg38 genome; outputs ASCII WIG format

### generate coverage with multiple functions
**Args:** `count -z 5 -w 25 -f mean,min,max input.bam coverage.tdf hg38`
**Explanation:** count subcommand; -z 5 max zoom level; -w 25 window size; -f mean,min,max computes three statistics; input.bam input BAM; coverage.tdf output TDF; hg38 genome

### convert copy number file to TDF
**Args:** `toTDF -z 5 copynumber.cn copynumber.tdf hg38`
**Explanation:** toTDF subcommand; -z 5 max zoom level; copynumber.cn input copy number file; copynumber.tdf output TDF; hg38 genome
