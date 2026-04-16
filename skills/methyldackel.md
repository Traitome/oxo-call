---
name: methyldackel
category: epigenomics
description: Methylation extractor for bisulfite-sequencing data from sorted BAM files
tags: [methylation, bisulfite, wgbs, rrbs, cpg, dna-methylation, epigenomics]
author: oxo-call built-in
source_url: "https://github.com/dpryan79/MethylDackel"
---

## Concepts
- MethylDackel extracts CpG, CHG, and CHH methylation from bisulfite-aligned BAM files.
- Input: reference FASTA + sorted indexed BAM from Bismark or BSMAP.
- Default output: bedGraph files with methylation for each cytosine context.
- Use 'MethylDackel extract' for standard methylation extraction; outputs <prefix>_CpG.bedGraph.
- Use 'MethylDackel mbias' to detect read-position biases (M-bias) before extraction.
- Use --CHG and --CHH flags to also extract CHG and CHH methylation (non-CpG).
- Use --ignore to specify bases to ignore at read ends: --ignore 5 5 ignores first and last 5 bases.
- MethylDackel is faster than bismark_methylation_extractor and handles both strand protocols.
- --mergeContext merges per-cytosine metrics into per-CpG/CHG metrics; useful for strand-specific analysis.
- --fraction outputs fractional methylation only (0-1); --counts outputs raw base counts.
- --minOppositeDepth and --maxVariantFrac exclude potential SNPs from methylation calls.
- --cytosine_report generates Bismark-compatible per-base exhaustive reports.
- --methylKit outputs methylKit-compatible format for downstream R analysis.

## Pitfalls
- Input BAM must be coordinate-sorted and indexed — run samtools sort and samtools index first.
- Run MethylDackel mbias first to identify end biases — then use --ignore to trim biased positions.
- The reference FASTA must be the same genome used for bisulfite alignment.
- Without --CHG and --CHH, only CpG methylation is extracted — specify for non-CpG contexts.
- MethylDackel bedGraph output is 0-based — check when comparing with other tools.
- For RRBS data, use --rrbs flag to handle MspI site artifacts.
- --mergeContext requires both cytosines in CpG to have sufficient coverage; sites with one low-coverage cytosine are excluded.
- --minOppositeDepth requires sufficient coverage on opposite strand; low coverage samples may lose many sites.
- --maxVariantFrac 0.0 (default) excludes no sites; increase to 0.1-0.2 to filter SNPs.
- --ignoreFlags 0xF00 (default) filters secondary, QC-failed, duplicate, and supplemental alignments.

## Examples

### extract CpG methylation from bisulfite-aligned BAM
**Args:** `extract reference.fa sorted_bisulfite.bam -o sample_methylation`
**Explanation:** extract subcommand; outputs sample_methylation_CpG.bedGraph; -o sets output prefix

### extract all cytosine contexts (CpG, CHG, CHH)
**Args:** `extract --CHG --CHH reference.fa sorted_bisulfite.bam -o sample_all_contexts`
**Explanation:** --CHG --CHH enables CHG and CHH context extraction in addition to CpG

### detect M-bias before extraction
**Args:** `mbias reference.fa sorted_bisulfite.bam sample_mbias`
**Explanation:** mbias generates M-bias plots; use to determine --ignore parameters for biased read positions

### extract methylation ignoring biased read ends
**Args:** `extract --ignore 5 5 reference.fa sorted_bisulfite.bam -o trimmed_methylation`
**Explanation:** --ignore 5 5 ignores first and last 5 bases of each read to remove end-of-read bias

### merge per-cytosine metrics into per-CpG metrics
**Args:** `extract --mergeContext reference.fa sorted_bisulfite.bam -o merged_cpg`
**Explanation:** --mergeContext combines strand metrics; outputs per-CpG instead of per-cytosine

### extract fractional methylation only
**Args:** `extract --fraction reference.fa sorted_bisulfite.bam -o fractional`
**Explanation:** --fraction outputs only methylation fraction (0-1); smaller files for visualization

### exclude SNPs from methylation calls
**Args:** `extract --minOppositeDepth 5 --maxVariantFrac 0.1 reference.fa sorted_bisulfite.bam -o snp_filtered`
**Explanation:** --minOppositeDepth 5 requires 5x coverage on opposite strand; --maxVariantFrac 0.1 excludes sites with >10% variants

### generate Bismark-compatible cytosine report
**Args:** `extract --cytosine_report reference.fa sorted_bisulfite.bam -o cytosine_report`
**Explanation:** --cytosine_report produces Bismark-compatible output; for compatibility with Bismark pipelines

### output methylKit format for R analysis
**Args:** `extract --methylKit reference.fa sorted_bisulfite.bam -o methylkit`
**Explanation:** --methylKit outputs methylKit-compatible format; for downstream analysis in R

### extract with all contexts and filtering
**Args:** `extract --CHG --CHH --mergeContext --minOppositeDepth 3 --maxVariantFrac 0.05 reference.fa sorted.bam -o comprehensive`
**Explanation:** comprehensive extraction with all contexts, merged CpGs, and SNP filtering
