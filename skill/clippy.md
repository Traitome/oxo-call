---
name: clippy
category: Bioinformatics / CLIP-seq Analysis
description: A command-line tool for processing CLIP-seq (Cross-Linking Immunoprecipitation sequencing) data to identify RNA-protein interaction sites and call peaks from immunoprecipitated reads.
tags:
  - clip-seq
  - rna-protein-interactions
  - peak-calling
  - genomics
  - rna-binding-protein
  - ber
author: AI-generated
source_url: https://github.com/ohlerlab/clippy
---

## Concepts

- **Data Model**: clippy operates on aligned sequencing reads (BAM/CRAM format) from CLIP-seq experiments. It models crosslink events as pseudo-reads at positions where reverse transcriptase terminates due to the crosslinked protein-RNA adduct. The tool distinguishes between true crosslink sites and background noise using statistical modeling.
- **Input Formats**: Primary inputs include aligned IP (immunoprecipitation) reads in BAM format, matched input/control reads, and a reference genome in FASTA format. Optional inputs are biological replicate BAM files and quality score files.
- **Output Formats**: clippy produces peak calls in BED format (6+ columns), normalized coverage tracks in bigWig format for visualization, and summary statistics in JSON/tabular format. Peak files include genomic coordinates, crosslink scores, and statistical significance values.
- **Peak Calling Pipeline**: The tool performs read extension to fragment length, crosslink site counting at each genomic position, background estimation from input controls, statistical testing using negative binomial or Poisson models, and false discovery rate (FDR) adjustment.
- **Strand Specificity**: CLIP-seq data is inherently strand-specific; clippy preserves strand information and reports peaks on the relevant strand. Crosslink events on the positive strand indicate protein binding to positive-strand RNAs (e.g., mRNAs, lncRNAs).

## Pitfalls

- **Omitting Input Control**: Running clippy without an input control sample (IP without antibody) leads to high false positive rates, as the tool cannot distinguish specific protein-RNA interactions from non-specific background binding. Always include matched input or IgG control data.
- **Incorrect Read Length Setting**: Mis-specifying the fragment or read length causes improper read extension, resulting in shifted peak coordinates and incorrect crosslink site assignment. Verify fragment sizes via independent methods (e.g., bioanalyzer) before analysis.
- **Ignoring Replicate Consistency**: Analyzing single replicates without biological replicates masks technical variability and can inflate significance calls. Biological replicates should be processed separately and merged using clippy's built-in replication handling.
- **Overly Liberal Thresholds**: Setting low significance thresholds (e.g., FDR > 0.05 or p-value > 0.01) generates thousands of spurious peaks that do not represent true binding sites. Use FDR ≤ 0.01 and log2 fold change ≥ 2 as starting parameters.
- **Mixed Genome Alignments**: Analyzing reads aligned to a concatenated genome (with multiple genomes in one index) produces peaks at repetitive or homology regions. Always use a species-specific genome index for peak calling.

## Examples

### Call peaks from a single CLIP-seq experiment against an input control
**Args:** --bam_ip reads.bam --bam_input input.bam --genome hg38.fa --out peaks.bed
**Explanation:** This compares IP reads against input control to identify specific protein-RNA interaction sites, producing genomic coordinates where binding is enriched above background.

### Call peaks from CLIP-seq data with biological replicates
**Args:** --bam_ip rep1.bam rep2.bam --bam_input input_rep1.bam input_rep2.bam --genome hg38.fa --out peaks_replicates.bed --replicates
**Explanation:** The `--replicates` flag enables replication-aware peak calling, combining statistical evidence across replicates to increase specificity and reduce false positives.

### Generate normalized coverage tracks for visualization
**Args:** --bam_ip reads.bam --bam_input input.bam --genome hg38.fa --out tracks --bigwig --normalize
**Explanation:** Produces genome browser tracks with input-normalized coverage, enabling direct visualization of binding sites in IGV or UCSC Genome Browser.

### Adjust peak calling stringency with custom FDR threshold
**Args:** --bam_ip reads.bam --bam_input input.bam --genome hg38.fa --out peaks_stringent.bed --fdr 0.001 --min_height 10
**Explanation:** Setting FDR to 0.001 and minimum height to 10 produces fewer but more confident peaks, suitable for high-confidence binding site lists.

### Process CLIP-seq data with strand-specific peak reporting
**Args:** --bam_ip reads.bam --bam_input input.bam --genome hg38.fa --out peaks_stranded.bed --strand_specific
**Explanation:** The `--strand_specific` flag ensures peaks are reported on the correct strand, maintaining strand information critical for interpreting binding to antisense RNAs or specific transcripts.

### Limit peak calling to specific genomic regions using a BED file
**Args:** --bam_ip reads.bam --bam_input input.bam --genome hg38.fa --out peaks_targeted.bed --region_bed targets.bed
**Explanation:** Restricting analysis to target regions in targets.bed reduces computational time and focuses analysis on genes or loci of interest, such as known binding targets.