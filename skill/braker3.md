---
name: braker3
category: gene_prediction
description: BRAKER3 is an unsupervised RNA-Seq-based gene prediction pipeline that uses aligned RNA-Seq data to train ab initio gene finders (AUGUSTUS and GeneMark-EP/ET) and predict protein-coding genes in eukaryotic genomes. It produces predictions in GFF3 format.
tags: [gene_prediction, RNA-Seq, ab_initio, eukaryotic, AUGUSTUS, GeneMark, genome_annotation]
author: AI-generated
source_url: https://github.com/GenePrediction/Braker
---

## Concepts

- **Input Requirements**: BRAKER3 requires a genome assembly in FASTA format and aligned RNA-Seq reads in BAM/SAM format. The RNA-Seq alignments must be sorted by genomic coordinate and indexed. Multiple RNA-Seq datasets (different tissues/conditions) can be combined to improve gene predictions.
- **Training and Prediction Model**: BRAKER3 uses RNA-Seq evidence to train two independent ab initio gene finders—AUGUSTUS and GeneMark-EP/ET. It extracts intron boundaries from RNA-Seq alignments to create training sets, then runs each gene finder independently to produce complementary predictions that are finally merged.
- **Output Format**: Predictions are written to a GFF3 file containing protein-coding gene models with exons, CDS, start/stop codons, and transcript support evidence. BRAKER3 also outputs protein FASTA files derived from the predicted CDS and optionally produces training evidence files for downstream use.
- **Integration with Protein Evidence**: When protein alignments (from ProtHint or other sources) are provided via the `--prg=protHint` option, BRAKER3 combines RNA-Seq and protein evidence for improved accuracy, particularly in detecting genes with poor RNA-Seq coverage.
- **Species Configuration**: BRAKER3 supports various eukaryotic species through species-specific parameters. The `--species` flag must match an available AUGUSTUS species (e.g., human, mouse, arabidopsis) or be configured for new species using training data.

## Pitfalls

- **Insufficient RNA-Seq Coverage**: Using RNA-Seq data with low coverage or poor quality alignments produces incomplete or fragmented gene models. Genes expressed only in missing tissues or at low levels will be missed entirely. Consequences include under-prediction of genes and incorrect exon boundaries.
- **Genome/Alignment Mismatch**: Providing a genome assembly that doesn't match the RNA-Seq read origin (different strain, version, or species) produces nonsensical predictions. Alignments may map incorrectly or not at all. This leads to chimeric gene predictions and wasted computation.
- **Missing Species Parameters**: Running without specifying a valid `--species` parameter or using an incompatible species config causes AUGUSTUS training to fail or produce poor predictions. Consequences include runtime errors or low-accuracy gene models that don't reflect the true gene structure.
- **Insufficient Memory for GeneMark**: GeneMark-EP/ET requires substantial RAM (often 20-40+ GB for larger genomes). Insufficient memory causes the pipeline to crash during the training phase. Always estimate requirements based on genome size before running.
- **Incorrect BAM Sorting**: Providing unsorted or incorrectly sorted BAM files (must be coordinate-sorted with valid BAI index) causes BRAKER3 to fail during intron extraction. The tool cannot proceed without proper alignment indexing.

## Examples

### Run basic gene prediction with RNA-Seq alignments only
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --species=human
**Explanation:** This runs BRAKER3 with RNA-Seq evidence to train AUGUSTUS and GeneMark, producing gene predictions for the human genome using the human species parameters.

### Run with protein evidence using ProtHint
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --prg=protHint --prot=protein_alignments.gff --species=arabidopsis
**Explanation:** Combines RNA-Seq and protein alignment evidence through ProtHint to improve gene prediction accuracy in Arabidopsis, leveraging both transcript and protein evidence.

### Use multiple RNA-Seq datasets
**Args:** --genome=genome.fasta --bam=rnaseq_tissue1.bam,rnaseq_tissue2.bam,rnaseq_tissue3.bam --species=mouse
**Explanation:** Provides three different RNA-Seq datasets (e.g., different tissues) to improve training completeness, capturing genes expressed across multiple conditions.

### Limit BRAKER3 to Augustus only
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --species=fly --usingAug=only
**Explanation:** Runs only the AUGUSTUS portion of the pipeline forDrosophila, useful when GeneMark licensing is unavailable or for faster execution with reduced memory.

### Specify output directory
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --species=zebrafish --outdir=/project/annotations/braker_output
**Explanation:** Specifies a custom output directory for the GFF3 results and auxiliary files instead of writing to the current working directory.

### Enable threading for parallel execution
**Args:** --genome=genome.fasta --bam=rnaseq_aligned.bam --species=human --threads=16
**Explanation:** Allocates 16 threads to parallelize BRAKER3 components, significantly reducing runtime on multi-core systems for large genomes.