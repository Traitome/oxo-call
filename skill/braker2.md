---
name: braker2
category: gene_prediction
description: A bioinformatics pipeline for automated gene prediction using AUGUSTUS, trained on RNA-Seq and/or protein homology evidence. BRAKER2 integrates GeneMark-ET and ProtHint to generate accurate ab initio gene predictions without manual training.
tags:
  - gene_prediction
  - augutus
  - rna-seq
  - genome_annotation
  - comparative_genomics
author: AI-generated
source_url: https://github.com/GenePrediction/Braker
---

## Concepts

- BRAKER2 requires a reference genome in FASTA format and evidence in the form of aligned RNA-Seq reads (BAM format) or aligned protein sequences (from ProtHint). The tool uses this evidence to train AUGUSTUS and generate gene predictions.
- The core data model relies on "hints" generated from RNA-Seq alignments (exon boundaries, introns, start/stop codons) or protein alignments (protein coding regions, microsynteny). These hints guide AUGUSTUS training and prediction accuracy.
- Output formats include GTF (gene transfer format) and GFF3, which are compatible with most downstream genome annotation pipelines. The predictions include CDS, exons, start_codon, stop_codon, and transcript attributes.
- BRAKER2 automatically selects an appropriate species parameter set if not specified. For novel genomes, using an evolutionarily close species (e.g., human for mammals, zebrafish for fish) improves initial training.
- The pipeline orchestrates three main tools: GeneMark-ET (for initial gene model training from RNA-Seq), ProtHint (for protein evidence hint generation), and AUGUSTUS (for final prediction). Understanding this workflow helps in troubleshooting failed runs.

## Pitfalls

- Running BRAKER2 without sufficient RNA-Seq coverage or protein homology data often results in fragmented or missing gene predictions. For bacterial genomes, at least 10x coverage is recommended; for eukaryotic genomes, deeply sequenced transcriptome data is critical for full-length gene models.
- Specifying the wrong or missing AUGUSTUS species parameter leads to poor initialization of gene models. Always verify that the closest known species is available in AUGUSTUS (check `augustus --species=help` for the full list) or allow BRAKER2 to auto-select.
- Memory exhaustion occurs when processing large genomes (>500 Mb) with default settings. The GeneMark-ET step is particularly memory-intensive. Use the `--cores` and `--memory` flags to allocate adequate resources or increase the `--geneMarkGtf` buffer size.
- Incompatible BAM files (e.g., unsorted, wrong reference names, or non-coordinate-sorted) cause the pipeline to fail silently or produce empty predictions. Ensure RNA-Seq alignments are coordinate-sorted and indexed with matching header names.
- Overwriting previous results without backing up causes data loss. BRAKER2 does not automatically backup existing output files in the working directory; use `--skip-optin` and manual directory management to preserve intermediate files.

## Examples

### Gene prediction using RNA-Seq alignments only
**Args:** `--genome=genome.fasta --bam=rnaseq_aligned.bam --species=human --gff3_out --outdir=braker_rnaseq`
**Explanation:** This runs BRAKER2 with RNA-Seq data as the sole evidence source, training AUGUSTUS on expressed regions and producing gene calls in GFF3 format.

### Gene prediction using protein homology only
**Args:** `--genome=genome.fasta --prot_seq=proteins.fasta --prot_align_hint --species=zebrafish --gff3_out --outdir=braker_protein`
**Explanation:** This uses protein sequence alignments as hints instead of RNA-Seq, suitable when no transcriptome data is available but related proteins exist.

### Combined RNA-Seq and protein evidence for higher accuracy
**Args:** `--genome=genome.fasta --bam=rnaseq_aligned.bam --prot_seq=proteins.fasta --species=mouse --gff3_out --outdir=braker_combined`
**Explanation:** Combining both evidence types typically yields more complete and accurate gene models by leveraging complementary information from transcription and conservation.

### Specify an alternative AUGUSTUS species model
**Args:** `--genome=genome.fasta --bam=rnaseq_aligned.bam --species=fruitfly --gff3_out --outdir=braker_fly`
**Explanation:** For non-model organisms, selecting the closest evolutionary species (e.g., fruitfly for insects) provides better initial parameters than the auto-selected default.

### Customize output format as GTF instead of GFF3
**Args:** `--genome=genome.fasta --bam=rnaseq_aligned.bam --species=human --gtf_out --outdir=braker_gtf`
**Explanation:** Using `--gtf_out` produces GTF output instead of GFF3, which is required for compatibility with tools like Cufflinks or certain RNA-Seq quantification pipelines.

### Run with multiple RNA-Seq BAM files
**Args:** --genome=genome.fasta --bam=sample1.bam,sample2.bam,sample3.bam --species=human --gff3_out --outdir=braker_multi
**Explanation:** Providing a comma-separated list of multiple BAM files increases evidence coverage, improving prediction completeness across different tissues or conditions.

### Enable AUGUSTUS prediction with introns from RNA-Seq
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --et_rnaseq=on --species=human --gff3_out --outdir=braker_introns
**Explanation:** The `--et_rnaseq=on` flag tells GeneMark-ET to use RNA-Seq intron hints directly for training, often yielding better splice site accuracy.

### Use GeneMark-ET+ with soft-specific masks
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --softmasking --species=human --gff3_out --outdir=braker_softmask
**Explanation:** Enabling `--softmasking` treats repetitive regions as lowercase in the genome, which AUGUSTUS uses as a negative prediction signal, reducing false positive gene calls in repeats.