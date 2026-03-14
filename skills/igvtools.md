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
- Main commands: toTDF (generate TDF coverage), index (create .idx index), count (coverage density).
- Use 'igvtools toTDF' to convert BAM/wig to IGV's .tdf format for fast zoom-level loading.
- Use 'igvtools index' to create .idx index for tab-indexed formats (not BAM — use samtools for that).
- Use 'igvtools count' to generate coverage tracks from BAM files.
- IGVtools is bundled with IGV desktop application; also available as standalone command line tool.
- For modern workflows, deeptools bamCoverage is preferred over igvtools for coverage generation.

## Pitfalls

- igvtools index is for IGV-specific formats (VCF, BED, GFF) — NOT for BAM (use samtools index).
- TDF files are IGV-specific — not compatible with other genome browsers (use bigWig for universal coverage).
- igvtools sort creates a sorted version — input must be sorted for indexed access.
- The genome parameter specifies the genome build: hg38, mm10, etc.

## Examples

### create coverage TDF track from BAM file
**Args:** `count -z 5 -w 25 sorted.bam coverage.tdf hg38`
**Explanation:** -z 5 max zoom level; -w 25 window size; hg38 genome; output .tdf for IGV visualization

### index a VCF file for IGV
**Args:** `index variants.vcf`
**Explanation:** creates variants.vcf.idx for fast random access in IGV; not needed for bgzipped+tabix VCF

### sort a BED file for IGV indexing
**Args:** `sort input.bed sorted.bed`
**Explanation:** sorts BED file by chromosome and position; required before igvtools index
