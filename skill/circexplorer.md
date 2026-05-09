---
name: circexplorer
category: Circular RNA Detection and Analysis
description: A toolkit for detecting, annotating, and quantifying circular RNAs from RNA-seq data by identifying back-spliced junction (BSJ) reads and integrating with genomic annotations.
tags:
  - circular RNA
  - non-coding RNA
  - alternative splicing
  - RNA-seq
  - back-spliced junction
  - transcriptome assembly
  - BSJ
  - splicing
author: AI-generated
source_url: https://github.com/Yanglab/CircExplorer
---

## Concepts

- **Back-spliced junction (BSJ) reads are the molecular signature of circular RNAs.** During circular RNA formation, a downstream 5' splice donor joins to an upstream 3' splice acceptor in reverse orientation, creating a junction absent from the linear reference. CircExplorer detects these BSJ reads by scanning alignment anchors where one read end aligns to one exon and the other end aligns to a distant downstream exon, effectively reversing the canonical splicing direction.

- **CircExplorer integrates with STAR-aligned BAM files and UCSC gene annotations.** The recommended workflow uses `STAR` to map RNA-seq reads, then passes sorted BAM files to CircExplorer along with gene annotation files in GenePred format (`.refGene` or `.genePred` extension). This integration allows automatic classification of circular RNAs as exonic, intronic, or intergenic based on the genomic positions of the BSJ anchor points.

- **Paired-end reads provide stronger circular RNA evidence than single-end reads.** CircExplorer's detection algorithm is significantly more accurate when both read pairs span the BSJ site, as the concordant alignment evidence rules out alignment artifacts or linear transcripts with unusual splicing patterns. Single-end data requires more stringent filtering parameters and yields higher false-positive rates.

- **Quantification of circular RNAs involves normalizing BSJ read counts to back-spliced junction points per million mapped reads (BSJPM).** Unlike gene expression quantification, circular RNA abundance is measured at the junction level rather than the transcript level. CircExplorer can export expression tables that include both raw counts and normalized values, enabling comparative analysis across samples while accounting for sequencing depth differences.

## Pitfalls

- **Using an outdated or mismatched gene annotation file causes misclassification of circular RNA types.** If the annotation file does not match the genome version used for STAR alignment (e.g., mixing hg19 annotations with hg38-aligned reads), many BSJ sites will be called as intergenic when they actually reside in annotated exons. This leads to inflated intergenic circular RNA calls and loss of biologically meaningful exonic circular RNAs.

- **Failing to filter ribosomal RNA (rRNA) reads before running CircExplorer inflates false-positive circular RNA calls.** rRNA sequences are highly repetitive and can generate alignment patterns that mimic BSJ events when multiple copies align to different genomic locations. Preprocessing with tools like `rDNA` masks or dedicated rRNA removal pipelines significantly reduces noise and improves detection specificity.

- **Accepting CircExplorer's default detection thresholds without adjusting for sequencing depth produces inconsistent results across samples.** Low-coverage datasets benefit from slightly relaxed anchor length thresholds, while deeply sequenced samples may require stricter cutoffs to avoid an overwhelming number of junction candidates that are subsequently difficult to validate experimentally.

- **Attempting to quantify individual circular RNA isoforms without strand-specific library information leads to ambiguous expression estimates.** Non-directional (unstranded) libraries cannot distinguish whether the BSJ read originated from a genuine circular RNA or from a linear precursor transcript. Applying CircExplorer to such data without accounting for library type may misrepresent the true circular RNA expression landscape.

- **Ignoring the linear fallback problem causes overestimation of circular RNA abundance.** Some putative BSJ reads may actually derive from linear transcripts with unusual splicing patterns that loop back to earlier genomic positions. CircExplorer's optional linear filtering step compares BSJ read counts against local linear read coverage; bypassing this step can yield circular RNAs that are technically detectable as linear species.

## Examples

### Detect circular RNAs from STAR-aligned BAM files using default parameters
**Args:** `circExplorer parse --lowercase -t 8 -o results.txt accepted_hits.bam`
**Explanation:** This command instructs CircExplorer to parse the STAR-aligned BAM file `accepted_hits.bam`, enabling lowercase sequence output for flanking region extraction and allocating 8 threads for parallel processing, with results written to `results.txt`.

### Detect circular RNAs with a custom gene annotation file in GenePred format
**Args:** `circExplorer parse --gene-annot custom_annotation.genePred -o custom_results.txt accepted_hits.bam`
**Explanation:** By specifying a custom GenePred annotation file with `--gene-annot`, CircExplorer will use the provided annotation rather than the built-in database to classify detected BSJ sites, enabling analysis with species or transcriptomes not included in the default installation.

### Assemble novel circular RNAs and extend flanking regions
**Args:** `circExplorer assemble -o assembled_candidates.txt accepted_hits.bam`
**Explanation:** The `assemble` subcommand reconstructs the full circular RNA sequence by extending the flanking exonic regions from the BAM file, producing candidate sequences that can be used for primer design in validation experiments like divergent PCR.

### Filter detected circular RNAs to retain only those with high-confidence junction spanning reads
**Args:** `circExplorer filter --site reads >= 2 --min-length 100 -o filtered.txt detected.txt`
**Explanation:** Applying post-detection filtering with `--site reads >= 2` requires at least 2 independent BSJ-spanning reads per junction and `--min-length 100` enforces a minimum of 100 nucleotides total flanking sequence, stringently removing low-coverage artifacts and singleton alignment noise.

### Generate a summary report of circular RNA categories by genomic region type
**Args:** `circExplorer summary --input detected.txt --output summary_report.txt`
**Explanation:** The `summary` subcommand aggregates detected circular RNAs by classification (exonic, intronic, intergenic) and generates descriptive statistics, providing a quick overview of the circular RNA landscape without detailed per-junction inspection.