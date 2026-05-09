---
name: cameo
category: motif_analysis
description: Bioinformatics tool for comparative motif enrichment analysis in genomic regions, typically used with ChIP-seq peaks to identify overrepresented DNA motifs against background sequences.
tags: [motif, chip-seq, enrichment, genomics, dna-sequences]
author: AI-generated
source_url: https://academic.oup.com/nar/article/38/suppl_2/W441/2881511
---

## Concepts

- **Input Format**: cameo accepts BED files containing genomic regions (e.g., ChIP-seq peaks) and FASTA files with background sequences for statistical comparison. The tool extracts sequences from the provided genomic regions using a specified genome assembly.
- **Statistical Enrichment**: The tool calculates motif enrichment by comparing observed motif frequencies in target regions versus background sequences using Fisher's exact test or binomial test, reporting p-values and enrichment fold changes.
- **Output Reports**: Cameo produces tabular outputs listing enriched motifs with their statistical significance (-log10 p-value), enrichment fold, and genomic location annotations. Results include chromosomal distribution of peaks overlapping each motif.
- **Genome Annotation Dependency**: Requires genome annotation files (e.g., in BED format with gene definitions) to determine whether enriched motifs fall within promoters, gene bodies, or intergenic regions.
- **Companion Binary**: The companion binary `cameo-build` constructs the background sequence dataset from genomic regions specified by the user for unbiased enrichment analysis.

## Pitfalls

- **Mismatch between BED and genome version**: Using a BED file aligned to hg19 while specifying an hg38 genome build for extraction will cause sequence retrieval failures or silently produce incorrect sequences, leading to meaningless enrichment results.
- **Insufficient background sequence count**: Providing too few background sequences (fewer than the number of target regions) reduces statistical power and produces unreliable p-values, often missing true positive motifs.
- **Overlapping peak regions**: Passing overlapping ChIP-seq peaks without merging them causes double-counting of sequence stretches, artificially inflating enrichment scores for certain motifs that overlap at peak boundaries.
- **Missing gene annotation file**: Running cameo without specifying a genome annotation BED file eliminates the chromosomal distribution output, making it impossible to determine whether motifs reside in promoters or other genomic features.
- **Improper motif database format**: Supplying motif files in the wrong format (e.g., plain text instead of MEME format) causes the tool to skip all motifs silently, resulting in an empty enrichment report with no error message.

## Examples

### Identify enriched motifs in ChIP-seq peaks from human tissue
**Args:** `-b peaks.bed --genome hg19 --fasta fa --motif-database jaspar2018.meme -o results/`
**Explanation:** Analyzes motif enrichment in the provided peak BED file using hg19 genome sequences, comparing against motifs in the JASPAR database and writing results to the specified output directory.

### Use a custom background sequence file for motif comparison
**Args:** `-b target.bed --background background_sequences.fa --motif-database cure_motifs.meme -o custom_results/`
**Explanation:** Compares motif frequencies in target regions against a user-provided FASTA file of background sequences rather than generating random genomic sequences.

### Build background from promoter regions only
**Args:** `--genome mm10 --region-type promoters --output promoter_bg.fa --gene-annotation refseq_genes.bed`
**Explanation:** Uses the cameo-build companion binary to extract promoter sequences from the mm10 genome annotation, creating a focused background set for enrichment analysis.

### Run cameo with binomial test for smaller datasets
**Args:** `-b small_peaks.bed --genome dm3 --fasta fa --method binomial --motif-database flybase.meme -o fly_results/`
**Explanation:** Uses the binomial test method which is more appropriate for smaller datasets where Fisher's exact test may be underpowered, with Drosophila melanogaster genome.

### Analyze motif enrichment and export summary statistics only
**Args:** `-b chip_peaks.bed --genome hg38 --fasta fa --motif-database homer.motif --summary-only -o summary_out/`
**Explanation:** Generates only the summary statistics table without detailed per-motif reports, useful for quick exploratory analysis of large peak sets.

### Filter results by minimum enrichment fold
**Args:** `-b enhancers.bed --genome mm9 --fasta fa --motif-database selex.motif --min-fold 2.0 -o filtered_results/`
**Explanation:** Filters the output to include only motifs with at least 2.0-fold enrichment, reducing the report size and highlighting the most strongly overrepresented motifs.

### Specify custom sequence extraction window size around peak centers
**Args:** `-b narrow_peaks.bed --genome hg19 --fasta fa --window 200 --motif-database corscan.meme -o window_results/`
**Explanation:** Extracts 200bp sequences centered on each peak rather than using the full peak extent, which is useful when peaks have variable widths.