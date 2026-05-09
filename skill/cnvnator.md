---
name: cnvnator
category: Structural Variant Detection
description: Read-depth-based tool for discovering copy number variants (CNVs) from BAM/CRAM alignment files using mean-shift statistics and GC-corrected regional depth analysis.
tags: [cnv, copy-number-variation, read-depth, structural-variants, sam-bam]
author: AI-generated
source_url: https://github.com/abyzovlab/CNVnator
---

## Concepts

- **Root binary format**: CNVnator reads must be indexed into a `.root` file via `cnvnator2root` (companion binary), which stores GC-corrected read counts per genomic bin. This index is mandatory for all subsequent analysis steps and cannot be reused across different bin sizes.

- **Binning strategy**: CNV calling operates on fixed-size genomic windows (e.g., 100bp, 500bp). Smaller bins increase resolution but reduce statistical power; bin size must be chosen at index creation and remains fixed throughout the analysis pipeline.

- **Multi-stage workflow**: CNVnator runs as a staged pipeline where each `cnvnator` invocation takes a single action flag (`-genotype`, `-statistics`, `-call`, `-eval`): results are stored in the Root file, and later stages consume prior-stage outputs to compute adjusted read depths and call CNVs.

- **RD signal normalization**: CNVnator applies GC-content bias correction internally and reports normalized read depth ratios relative to the whole genome median. The mean-shift algorithm iteratively converges these ratios toward discrete copy number states (deletion, normal, duplication).

## Pitfalls

- **Mismatched bin size**: Creating the Root index with one bin size (e.g., 100) then running genotyping with another (e.g., 500) causes silent failures or empty output because the read count histogram stored in the file has a different granularity than expected by the calling engine.

- **Insufficient sequencing depth**: CNVnator requires ≥15–20x whole-genome coverage for reliable CNV detection. Below this threshold, the mean-shift algorithm may fail to converge or produce spurious heterozygous deletions due to low-count bins in high-GC regions.

- **Missing or corrupted BAM index**: The `.bai` file must exist alongside the input BAM; without it, `cnvnator2root` aborts without a clear error message. If the BAM is sorted by query name instead of coordinate, read counting will be incorrect for repetitive regions.

- **Forgetting `-unique` for PCR-free data**: Using deduplicated BAMs without the `-unique` flag during read extraction causes inflated read counts for low-complexity regions, leading to false-positive duplications because duplicate reads are counted as independent coverage signals.

## Examples

### Generate Root index from BAM aligned to GRCh37 reference
**Args:** `-root reference.root -bam sample.bam -fasta GRCh37.fa -gctype hg19 -chrom 1-22,X,Y`
**Explanation:** Extracts GC-corrected read counts into a Root file using hg19 GC content annotations, restricted to canonical chromosomes to avoid mitochondrial and random contigs.

### Extract and count reads for CNV analysis
**Args:** `unique -root sample.root -bam sample.dam -gctype hg19`
**Explanation:** Counts uniquely mapping reads (MAPQ ≥1) into the pre-built Root histogram, applying hg19 GC correction to mitigate reference bias in high-GC regions.

### Perform region-based genotyping at 100bp resolution
**Args:** `-root sample.root -window 100 -type target`
**Explanation:** Assigns reads to 100bp genomic bins and computes statistical significance per bin against the background distribution, enabling visualization of regional depth alterations.

### Call copy number variants with a p-value threshold
**Args:** `-call 100 -pvalue 0.01`
**Explanation:** Invokes the mean-shift CNV calling algorithm using 100bp bins and a 1% significance cutoff to distinguish true copy number changes from stochastic noise in read depth.

### Export CNV calls to VCF format
**Args:** `cnvnator2bed -root sample.root -organism human -bed CallCNVs.bed`
**Explanation:** Converts called CNV regions into a BED file with standardized genomic coordinates, suitable for intersection with functional annotation databases or import into downstream tools.

### Evaluate CNV detection performance against a truth set
**Args:** `-eval 100 -callfile Calls.bed -baseline Truth.bed`
**Explanation:** Computes precision, recall, and F1 score by comparing the detected CNV BED file against an orthogonal benchmark set at 100bp resolution, producing a contingency table summary.

### Detect events in a specific genomic interval
**Args:** `-region 1:1000000-5000000 -window 100`
**Explanation:** Restricts mean-shift analysis to chromosome 1 positions 1–5 Mb, reducing compute time for targeted validation studies while maintaining full statistical rigor within the focal region.

### Batch process multiple BAMs with a shell loop
**Args:** `for bam in lane1.bam lane2.bam; do cnvnator2root -root "${bam%.bam}.root" -bam "$bam" -fasta ref.fa; done`
**Explanation:** Iterates over multiple BAM files to create separate Root indices, enabling parallel CNV calling when samples are sequenced across multiple flow cell lanes.