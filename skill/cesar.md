---
name: cesar
category: comparative-genomics
description: Comparative gene annotation tool that transfers gene structures from a reference species to a target genome using pairwise alignments. Predicts orthologous transcripts and handles multi-exon gene structures with splicing-aware inference.
tags:
  - gene-annotation
  - comparative-genomics
  - orthology-prediction
  - transcript-prediction
author: AI-Generated
source_url: https://github.com/dieTHO/CESAR
---

## Concepts

- **Reference Annotation Requirement**: CESAR requires a high-quality reference annotation file in GTF format containing transcript, exon, and CDS features. The tool uses these reference gene structures as templates to find orthologous loci in the target genome—without a valid reference GTF, no predictions can be generated.

- **Alignment Input Format**: CESAR accepts either PSL (UCSC), exonerate, or protein alignment formats. Pairwise alignments must map reference transcripts to the target genome with correct genomic coordinates. The alignment quality directly determines prediction accuracy—poor or fragmented alignments produce truncated or missed gene models.

- **Splicing-Aware Gene Prediction**: The tool models intron-exon boundaries by detecting canonical splice sites (GT-AG, GC-AG, AT-AC) and exon junctions within alignments. It penalizes frameshifts, in-frame stop codons, and non-canonical splicing within coding sequences, producing a scored gene prediction with a confidence metric.

- **Output GTF Structure**: Predicted genes are written to a GTF file where the gene_id field groups transcripts and the transcript_id uniquely identifies each prediction. CDS features are included only when protein-coding potential is detected; non-coding transcripts omit CDS lines. Each prediction carries a confidence score used for downstream filtering.

## Pitfalls

- **Reference Annotation Contains Non-Canonical Genes**: Including pseudogenes, non-coding RNAs, or fragmented transcripts in the reference GTF causes CESAR to generate spurious predictions in the target species. The consequence is a bloated GTF with hundreds of false-positive gene models that contaminate downstream analyses.

- **Misaligned Reference and Target Assemblies**: Using alignments from an incorrect genome assembly version produces genomic coordinate mismatches, causing CESAR to place predictions in wrong loci or fail entirely with coordinate errors. Always verify that reference genome and alignment coordinates use the same assembly.

- **Low-Identity or Short Alignments**: Alignments below ~50% identity or shorter than 50% of the reference transcript length trigger heuristics that produce incomplete or truncated gene models. The consequence is missing exons or fusion of adjacent genes, reducing annotation completeness.

- **Missing Splice Site Signals in Alignments**: Exonerate alignments that omit intronic regions (only providing exonic chunks) can cause CESAR to miss internal splice sites and incorrectly join exons, producing genes with frameshifted CDS or incorrect intron structures.

## Examples

### Predict genes for a single reference-target pair
**Args:** reference.gtf target_genome.fa --alignments exonerate_output.txt output_predictions.gtf
**Explanation:** This basic usage predicts gene structures by mapping the reference annotation through exonerate alignments onto the target genome, writing all predictions to the output GTF.

### Filter predictions by confidence score
**Args:** predictions.gtf --min-score 0.8 --score-file scores.txt high_confidence.gtf
**Explanation:** Filtering removes low-confidence predictions below 0.8, retaining only robust gene calls for downstream applications like orthology analysis or functional annotation.

### Use PSL alignments from BLAT as input
**Args:** reference.gtf target_genome.fa --alignments blat_output.psl --format psl output.gtf
**Explanation:** Specifying the PSL format allows BLAT-aligned protein or transcript queries to be used directly, enabling compatibility with standard alignment workflows.

### Process multiple target genomes in batch
**Args:** reference.gtf targets_list.txt --alignments-dir alignments/ --output-dir predictions/
**Explanation:** Providing a list file with multiple target genome paths processes them sequentially, creating a separate GTF output for each target species in the predictions directory.

### Generate a report of annotation completeness
**Args:** predictions.gtf --reference reference.gtf --completeness-report stats.txt
**Explanation:** Comparing predicted genes against the reference annotation produces statistics on exon recovery, CDS completeness, and fraction of reference genes successfully transferred, useful for evaluating annotation quality.