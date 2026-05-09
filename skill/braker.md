---
name: braker
category: gene_prediction
description: BRAKER is a tool for automated eukaryotic gene prediction that integrates RNA-Seq and/or protein evidence with ab initio gene predictors (GeneMark-EP/ET and AUGUSTUS). It produces accurate gene annotations in GFF3 format without requiring manual training.
tags: [gene-prediction, annotation, eukaryotes, rna-seq, protein-evidence, augustus, genemark, genomics]
author: AI-generated
source_url: https://github.com/GenePrediction/EXON
---

## Concepts

- **Evidence-based prediction**: BRAKER requires either RNA-Seq alignments (BAM format) or protein homology evidence (FASTA) to guide gene prediction. Without evidence,BRAKER cannot operate—the tool uses these alignments to infer exon/intron boundaries and coding sequences.
- **Two-step pipeline**: First, GeneMark-EP/ET uses evidence to generate initial gene models; second, AUGUSTUS refines these predictions using the same evidence, producing cleaner annotations with accurate start/stop codons and splice sites.
- **Output format**: BRAKER produces GFF3 files containing predicted mRNAs, exons, CDS features, and coding regions. The output can be directly imported into genome browsers or used as input for downstream functional annotation tools.
- **Species configuration**: BRAKER supports two modes—with RNA-Seq evidence (requires Genome-wide RNA-Seq alignment in BAM format) or with protein evidence (FASTA files from related species). Each mode requires specifying a species parameter for AUGUSTUS.

## Pitfalls

- **Missing evidence data**: Running BRAKER without either `--bam` or `--prot` will fail. The tool relies entirely on extrinsic evidence to train the gene finders; using an empty or missing BAM file produces nonsensical predictions with incorrect gene structures.
- **Incorrect species parameter**: Using a non-existent or misspelled species name (e.g., `--species human` instead of `--species human`) causes BRAKER to fail during AUGUSTUS initialization, as the species-specific training files cannot be located.
- **Softmasked genome not specified**: If your genome FASTA contains softmasked repeats (lowercase letters), you must use `--softmasked`. Without this flag, BRAKER treats lowercase bases as N characters, leading to fragmented or incomplete gene predictions in repeat-rich regions.
- **Memory exhaustion with large genomes**: BRAKER stores large alignment databases in memory. For genomes >500 Mb, allocate at least 8 GB RAM or use `--cores` to distribute computation, otherwise the process will be killed by the system.

## Examples

### Predict genes using RNA-Seq alignments
**Args:** `--genome genome.fasta --bam alignments.bam --species arabidopsis --workingdir braker_rna`
**Explanation:** This runs BRAKER with RNA-Seq evidence, using the arabidopsis species parameter for AUGUSTUS training and writing all outputs to the specified working directory.

### Predict genes using protein homology evidence
**Args:** `--genome genome.fasta --prot proteins.fasta --species fly --workingdir braker_prot`
**Explanation:** This runs BRAKER with protein sequences from related species as evidence, useful when high-quality RNA-Seq data is unavailable.

### Combined RNA-Seq and protein evidence
**Args:** `--genome genome.fasta --bam alignments.bam --prot proteins.fasta --species maize --workingdir braker_hybrid`
**Explanation:** Using both RNA-Seq and protein evidence improves prediction accuracy, especially for genes with poor RNA-Seq coverage or for conserved genes across species.

### Using softmasked genome with repeats
**Args:** --genome softmasked_genome.fasta --bam alignments.bam --species human --softmasked --workingdir braker_masked
**Explanation:** The `--softmasked` flag tells BRAKER to interpret lowercase bases as repeat sequences and avoid predicting genes in these regions.

### Specify custom AUGUSTUS species name
**Args:** --genome genome.fasta --bam alignments.bam --species my_species --augustus_species new_species --workingdir braker_custom
**Explanation:** This allows using an existing AUGUSTUS species parameter or creating a custom species, useful when working with non-standard organisms or reusing trained species models.

### Adjust computational resources
**Args:** --genome genome.fasta --bam alignments.bam --species mouse --cores 4 --workingdir braker_parallel
**Explanation:** Using multiple cores speeds up BRAKER execution on multi-core systems, particularly beneficial for large eukaryotic genomes.

### Generate output in GFF3 format explicitly
**Args:** --genome genome.fasta --bam alignments.bam --species zebrafish --gff3 --workingdir braker_gff3
**Explanation:** The `--gff3` flag ensures output is written in GFF3 format, which is compatible with most genome browsers and annotation databases.