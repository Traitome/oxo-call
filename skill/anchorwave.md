---
name: anchorwave
category: genomic-alignment
description: Whole-genome alignment tool that uses anchor points to guide alignment and detect large-scale structural variations between genomes with high accuracy in divergent and repetitive regions.
tags: [whole-genome-alignment, structural-variants, comparative-genomics, anchor-points, sequence-alignment]
author: AI-generated
source_url: https://github.com/baoxingzhou/anchorwave
---

## Concepts

- AnchorWave discovers short exact match anchors (typically 12-19bp) between two sequences and extends alignments outward from these anchors, enabling accurate detection of insertions, deletions, rearrangements, and duplications even in highly divergent or repeat-rich genomic regions where traditional aligners fail.
- The alignment output is SAM format for read alignments, and FASTA format for synthesized reference sequences; input genomes accept FASTA files for both reference and query sequences, with optional annotation files (GFF3) for coding sequence (CDS) regions to bias alignment toward coding regions.
- genoAlign performs pairwise whole-genome alignment between a reference and a query sequence, while aliVariants calls variants based on alignment results, and refMultiAli aligns multiple query sequences to a common reference; the tool is designed for cross-species alignments where rearrangements and structural variations are expected.
- AnchorWave can align Oxford Nanopore Technology (ONT) or PacBio HiFi reads directly to a reference genome using the same anchor-based approach, making it suitable for haplotyping and detecting phased variants across large structural events.
- The tool requires sufficient computing resources for large genomes (recommended 8+ CPU cores and 32GB+ RAM for mammalian genomes) due to the dynamic programming steps involved in anchor extension and alignment scoring.

## Pitfalls

- Specifying an excessively short or long anchor length (-m flag) severely impacts alignment quality: too short anchors create false anchors leading to misalignments, while too long anchors may fail to find sufficient anchors in divergent regions, resulting in fragmented or missing alignments.
- Omitting the CDS annotation file (-as flag) when aligning eukaryotic genomes causes alignments to ignore gene structure, potentially aligning coding sequences incorrectly across intron boundaries and producing biologically meaningless alignments for gene-level analyses.
- Using default scoring parameters for highly divergent species (default match=2, mismatch=-1) without adjusting the -s and -z flags leads to inflated alignments in low-complexity regions and under-detection of actual structural variations; species with >30% divergence require tuned scoring matrices.
- Attempting to align multiple chromosomes in parallel without specifying proper chromosome ranges causes memory exhaustion and race conditions; always process large genomes chromosome-by-chromosome using the -c flag with appropriate ranges.
- Confusing input order of reference (-r) and query (-q) sequences produces inverted alignments where query coordinates are reported relative to the wrong strand, yielding variants on the opposite chromosome orientation and invalidating downstream analyses.

## Examples

### Whole-genome alignment of a query genome to a reference
**Args:** genoAlign -i query_genome.fa -r reference_genome.fa -as reference_CDS.gff3 -o alignment.sam -m 19
**Explanation:** Aligns the entire query genome to the reference using 19bp anchors, with alignment biased toward CDS regions specified in the annotation file, outputting a SAM file of coordinate-mapped alignments.

### Detecting structural variants between two assemblies
**Args:** aliVariants -r ref.fa -q query.fa -as ref_CDS.gff3 -d alignment.sam -o variants.vcf -m 19
**Explanation:** Identifies and outputs all structural variants (insertions, deletions, inversions, translocations) between the reference and query assemblies into VCF format based on the genoAlign alignment results.

### Cross-species alignment with tuned scoring parameters
**Args:** genoAlign -i mouse_genome.fa -r rat_genome.fa -as rat_CDS.gff3 -o cross_species.sam -m 16 -s 2 -z -3
**Explanation:** Aligns mouse to rat genome using shorter 16bp anchors and adjusted mismatch penalty (-3) to accommodate ~30 million years of evolutionary divergence, producing biologically meaningful whole-genome alignments.

### Aligning long-read sequences to a reference genome
**Args:** genoAlign -i reads.fastq -r reference.fa -as reference_CDS.gff3 -o reads_aligned.sam -m 12 -t 16
**Explanation:** Aligns Oxford Nanopore or PacBio reads to the reference using smaller 12bp anchors suitable for error-prone long reads, utilizing 16 threads for faster processing of whole-dataset alignment.

### Multi-sample alignment to a common reference
**Args:** refMultiAli -r reference.fa -as reference_CDS.gff3 -q sample1.fa sample2.fa sample3.fa -o multi_alignment.fa -m 19
**Explanation:** Aligns multiple query genomes simultaneously to the same reference, synthesizing aligned sequences in FASTA format for downstream comparative genomics across populations or species.