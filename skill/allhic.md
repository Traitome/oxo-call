---
name: allhic
category: Genome Assembly / Hi-C Analysis
description: A tool for Hi-C based genome scaffolding that filters and leverages chromatin interaction data to order and orient contigs into chromosome-scale scaffolds. Supports allele-specific Hi-C analysis for improved assembly of diploid genomes.
tags:
  - Hi-C
  - genome assembly
  - chromatin interaction
  - scaffolding
  - 3D genomics
  - diploid assembly
author: AI-generated
source_url: https://github.com/xchangecnuc/allhic
---

## Concepts

- **Input Data Model**: AllHiC processes Hi-C interaction data from `.pairs` format files (containing read pairs with chromosome positions) and contact matrices. The tool requires a reference genome (FASTA) and a set of contigs to be scaffolded, along with valid Hi-C read pairs mapped to both sequences.

- **Filtering Pipeline**: Before scaffolding, Hi-C read pairs must be filtered for invalid interactions (e.g., self-ligations, random breaks, PCR duplicates). The `allhic-filter` subcommand removes low-quality pairs based on distance, mapping quality, and sequence similarity thresholds to reduce noise in the contact matrix.

- **Allele-Specific Assembly**: AllHiC can separate maternal and paternal haplotypes in diploid genomes by grouping contigs based on allele-specific Hi-C signals. This produces separate scaffolds for each homologous chromosome, improving assembly completeness for heterozygous organisms.

- **Scaffolding Algorithm**: The tool uses a maximum weighted matching algorithm to order and orient contigs. It builds a directed graph where edges represent Hi-C interaction weights between contig ends, then finds the optimal path that maximizes cumulative interaction signals while respecting orientation constraints.

- **Output Formats**: Results are exported as FASTA files (scaffolded genome sequences), AGP files (describing scaffold architecture), and text files containing interaction statistics. The AGP format is compatible with standard genome browsers and downstream assembly tools.

---

## Pitfalls

- **Skipping Read Pair Filtering**: Running scaffolding on unfiltered Hi-C data includes invalid pairs (self-ligations, duplicates, short-range contacts), which introduces false positive interactions and produces misoriented or incorrectly ordered contigs. The resulting scaffolds will have fragmented or shuffled assembly.

- **Using Low-Resolution Contact Matrices**: Applying AllHiC to sparse or low-depth Hi-C datasets (fewer than 100 million valid read pairs) yields insufficient interaction coverage, causing ambiguous ordering of distant contigs and chromosome-scale scaffolds to remain fragmented.

- **Mismatched Reference Genomes**: Providing a reference genome that is not syntenic to the input contigs (e.g., using a related species instead of the target genome) results in misaligned Hi-C signals, leading to incorrect scaffolding orders that do not reflect the true chromosome structure.

- **Ignoring Minimum Interaction Thresholds**: Setting `--minInteract` too low allows random noise to drive scaffolding decisions, while setting it too high excludes legitimate weak interactions, leaving many contigs unscaffolded. The optimal threshold typically requires empirical testing on the specific dataset.

- **Inconsistent Read Processing**: Using different mapping parameters (e.g., read length, mismatch tolerance) between Hi-C read mapping and AllHiC filtering creates inconsistencies that reduce the signal-to-noise ratio, degrading the accuracy of the final scaffolding.

---

## Examples

### Filter Hi-C read pairs for scaffolding
**Args:** `allhic-filter --minPos 500 --maxPos 20000000 --minMapQ 30 input.pairs output.pairs`
**Explanation:** This filters read pairs to retain only valid intra-chromosomal interactions with a genomic distance between 500 bp and 20 Mb and a minimum mapping quality of 30, removing noise before scaffolding.

### Build scaffolds using a contact matrix
**Args:** `allhic-build -t 32 -m 10000000 genome.fa contigs.fa matrix.txt scaffolds_out.txt`
**Explanation:** This runs the scaffolding algorithm using 32 threads, requiring at least 10 million valid Hi-C interactions to consider a contig pair for joining, producing ordered scaffolds for the genome.

### Scaffold with allele separation for diploid assembly
**Args:** `allhic-build --heterozygote -o diploid_scaffolds genome.fa contigs.fa matrix.txt`
**Explanation:** This enables allele-specific scaffolding mode to separately scaffold maternal and paternal chromosomes based on heterozygous Hi-C signals, producing distinct homologous scaffolds.

### Evaluate scaffolding quality using Hi-C map
**Args:** `allhic-eval scaffolds.fa genome.fa hic_obs.matrix eval_results.txt`
**Explanation:** This compares the scaffolded assembly against observed Hi-C interaction maps to generate statistics on scaffolding accuracy, including coverage and orientation correctness scores.

### Extract high-confidence interaction pairs for specific chromosomes
**Args:** `allhic-filter -c chr1,chr2,chr3 --minInterChrom 5 -o chr1to3.pairs all.pairs`
**Explanation:** This extracts inter-chromosomal Hi-C read pairs involving chromosomes 1-3 that have at least 5 supporting read pairs per interaction, useful for targeted chromosome-scale scaffolding.

### Run iterative scaffolding with multiple rounds
**Args:** `allhic-build -i 3 --minInteract 5000000 -o round3_scaffolds genome.fa round2.fa matrix.txt`
**Explanation:** This performs 3 iterative rounds of scaffolding, progressively increasing stringency, with each round using the previous scaffolded assembly as input for the next iteration.

### Generate AGP file for assembly submission
**Args:** `allhic-build -a agp_output.agp genome.fa contigs.fa matrix.txt output.fa`
**Explanation:** This produces an AGP format file describing how contigs are ordered, oriented, and joined into scaffolds, required for GenBank submission and genome database integration.