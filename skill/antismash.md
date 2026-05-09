---
name: antismash
category: Secondary Metabolite Gene Cluster Detection
description: Predicts biosynthetic gene clusters (BGCs) for secondary metabolites in bacterial, fungal, and plant genomes. Detects NRPS, PKS, RiPP, terpene, ladderane, and other specialized metabolite clusters using HMM profiles, rule-based reasoning, and homology to known clusters in MIBiG.
tags: [bioinformatics, genomics, natural-products, biosynthetic-gene-clusters, antiSMASH, secondary-metabolites, NRPS, PKS, RiPP]
author: AI-generated
source_url: https://antismash.secondarymetabolites.org
---

## Concepts

- **Input formats**: antismash accepts nucleotide sequences in FASTA format (genomes, contigs, or scaffolds). Raw unannotated sequences require internal annotation via Prodigal, while pre-annotated GenBank files from tools like Prokka can be directly input for faster analysis.
- **Output artifacts**: Generates browsable HTML reports, structured GenBank files recording cluster boundaries and annotations, machine-readable JSON containing detailed cluster features, and text summaries. GenBank outputs interface directly with MIBiG for cluster comparison.
- **Detection modules**: Operates modular detection for diverse BGC types—NRPS/PKS analysis, RiPP recognition (via RiPPfinder), terpene cyclase identification, siderophore detection, and more. Each module runs independently and produces type-specific feature annotations.
- **Database homology**: Performs HMMer searches against established BGC profiles from the MIBiG database, enabling functional prediction of cluster products. This returns known compound families and confidence scores for potential outputs.

## Pitfalls

- **Unannotated input sequences**: Inputting raw FASTA genome sequences without prior gene calling extends runtime substantially because antismash invokes Prodigal internally. For repeated analyses on the same genome, pre-annotating with Prokka and using the resulting GenBank file is far more efficient.
- **Ignoring genus-specific databases**: Running default detection on fungal genomes misses specialized clusters because fungal-specific modules (e.g., fuzzy ORF detection, FungalClust) require explicit configuration. Results will miss relevant BGCs if genus-appropriate profiles are not enabled.
- **Insufficient computational resources**: Large bacterial genomes or multi-chromosome analyses with comprehensive detection (full mode versus minimal) demand significant RAM and CPU cycles. Underprovisioned systems cause timeouts or incomplete output generation.
- **Output directory conflicts**: Re-running antismash without clearing previous output directories triggers errors or silent overwrites. Always specify a fresh output directory or explicitly manage existing files to prevent data loss.

## Examples

### Run full secondary metabolite detection on an annotated genome

**Args:** --output-dir results_full --genbank --html --json --full --cpus 8 sample.gbk

**Explanation:** Uses full detection mode to exhaustively identify all cluster types, outputting structured GenBank, HTML visualization, and JSON for downstream analysis, with parallel processing across 8 cores.

### Minimal analysis for quick cluster overview

**Args:** --output-dir quick_results --minimal --genbank genome_convs.fasta

**Explanation:** Runs minimal detection to rapidly identify major cluster types without extensive homology searches, suitable for initial screening before pursuing detailed analysis.

### Detect only NRPS and PKS clusters in a bacterial genome

**Args:** --output-dir nrps_pks --genbank --skip-clusters --enable-cluster-nrps --enable-cluster-pks input.fasta

**Explanation:** Restricts detection to nonribosomal peptide synthetase and polyketide synthase clusters, reducing runtime while focusing analysis on these medically relevant cluster families.

### Run with Pfam domain analysis for functional annotation

**Args:** --output-dir pfam_results --genbank --html --pfam2go --cpus 4 annotated.gbk

**Explanation:** Includes Pfam domain identification with Gene Ontology mapping to annotate cluster features with protein family functional predictions, outputting both HTML visualization and structured annotations.

### Generate results compatible with MIBiG cluster comparison

**Args:** --output-dir mibig_comparison --genbank --asf --cassis --smcog-trees input.gbk

**Explanation:** Enables antisense feature detection, CASSiS for precursor prediction, and SMCoG phylogenetic trees to generate GenBank outputs directly comparable to MIBiG database entries for known cluster homology.

### Analyze multiple genomes in batch from an input directory

**Args:** --output-dir batch_output --genbank --html --json --cpus 16 input_dir/

**Explanation:** Processes the entire input directory of genomes in parallel (16 threads), producing structured outputs for each genome for high-throughput cluster discovery across multiple strains or isolates.