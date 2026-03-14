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

## Pitfalls

- Input BAM must be coordinate-sorted and indexed — run samtools sort and samtools index first.
- Run MethylDackel mbias first to identify end biases — then use --ignore to trim biased positions.
- The reference FASTA must be the same genome used for bisulfite alignment.
- Without --CHG and --CHH, only CpG methylation is extracted — specify for non-CpG contexts.
- MethylDackel bedGraph output is 0-based — check when comparing with other tools.
- For RRBS data, use --rrbs flag to handle MspI site artifacts.

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
