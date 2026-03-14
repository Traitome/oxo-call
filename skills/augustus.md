---
name: augustus
category: annotation
description: Gene prediction for eukaryotic genomes using Hidden Markov Models with training support
tags: [annotation, gene-prediction, eukaryote, hmm, genome, gff]
author: oxo-call built-in
source_url: "https://bioinf.uni-greifswald.de/augustus/"
---

## Concepts

- AUGUSTUS predicts eukaryotic genes using species-specific trained HMM parameters.
- Use --species to specify a pre-trained species model (e.g., human, arabidopsis, zebrafish, fly).
- List available species with 'augustus --species=help'.
- Use --gff3=on for GFF3 output (default is AUGUSTUS-specific format); --outfile for output file.
- Evidence integration: use --hintsfile with extrinsic hints (RNA-seq, protein) for better accuracy.
- For training on a new species, use the AUGUSTUS training pipeline (autoAug.pl or BRAKER).
- AUGUSTUS handles multi-gene prediction across entire chromosomes/scaffolds.
- BRAKER (BRAKER1/2/3) automates AUGUSTUS training with RNA-seq or protein evidence.

## Pitfalls

- Using the wrong species model severely reduces accuracy — choose the closest related organism.
- AUGUSTUS without RNA-seq hints may miss many genes in organisms with complex gene structures.
- The GFF3 output requires --gff3=on — default AUGUSTUS format is not standard GFF3.
- For large genomes, split into chromosomes/scaffolds and run in parallel.
- AUGUSTUS is sensitive to repeat-masked input — run RepeatMasker before AUGUSTUS for best results.
- Training AUGUSTUS from scratch requires ~1000 manually curated gene structures.

## Examples

### predict genes in a eukaryotic genome using human parameters
**Args:** `--species=human genome.fasta --gff3=on > augustus_predictions.gff3`
**Explanation:** --species=human trained model; --gff3=on standard output format; output redirected to GFF3 file

### predict genes with RNA-seq hints for improved accuracy
**Args:** `--species=arabidopsis --hintsfile=rnaseq_hints.gff --extrinsicCfgFile=extrinsic.cfg genome.fasta --gff3=on > improved_predictions.gff3`
**Explanation:** --hintsfile provides RNA-seq intron/exon hints; --extrinsicCfgFile sets weights for hint integration

### predict genes and output protein sequences
**Args:** `--species=fly --gff3=on --protein=on --codingseq=on genome.fasta > fly_predictions.gff3`
**Explanation:** --protein=on outputs protein sequences in GFF3; --codingseq=on outputs CDS sequences

### run Augustus on a repeat-masked genome
**Args:** `--species=zebrafish zebrafish_masked.fasta --gff3=on --softmasking=1 > zebrafish_genes.gff3`
**Explanation:** --softmasking=1 uses softmasked repeat regions (lowercase) to avoid predicting genes in repeats
