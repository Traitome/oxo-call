---
name: bindash
category: genomics/comparison/sketching
description: A tool for fast genome sketch comparison using MinHash-based algorithms to compute k-mer sketches and distances between genomes or metagenomes.
tags: [minhash, sketch, genome-comparison, distance, k-mer, mash, bioinformatics]
author: AI-generated
source_url: https://github.com/bindash/bindash
---

## Concepts

- Bindash operates in two primary modes: `sketch` generates MinHash sketches from FASTA/FASTQ input files using configurable k-mer sizes and sketch parameters, while `dist` computes pairwise distances between precomputed sketch files. Sketches are stored as compact binary representations that enable rapid similarity queries across large genome collections.

- K-mer size (typically 21–51) and sketch size (number of hashes, commonly 1000–10000) directly control the sensitivity-speed tradeoff: larger k-mers capture more specific features but may miss conserved regions, while larger sketch sizes increase accuracy but require more memory and computation time.

- Bindash supports streaming input and can process multiple genomes in a single run, generating a distance matrix as output. The tool accepts standard bioinformatics formats (FASTA, FASTQ, compressed with gzip/bgzf) and outputs Newick-formatted trees or distance tables depending on the chosen mode.

- The underlying algorithm uses a modified MinHash approach with 64-bit hash values, ensuring collision-resistant representation of k-mer sets. When comparing sketches, Bindash reports the estimated Jaccard similarity, which correlates with ANI (Average Nucleotide Identity) for closely related genomes.

- Reference-free operation allows Bindash to compare genomes without requiring alignments or external databases, making it suitable for draft genomes, metagenomes, and rapid screening of large datasets.

## Pitfalls

- Specifying an incorrectly sized k-mer (too small, e.g., 17, may cause excessive false matches; too large, e.g., 63, may result in zero k-mers for short sequences) leads to inaccurate or uninformative distances. Always verify that the k-mer size is smaller than the shortest sequence in your input.

- Attempting to compute distances between sketches generated with mismatched parameters (different k-mer sizes or sketch sizes) will fail or produce meaningless results. All sketches in a comparison must share identical k and sketch-size parameters.

- Insufficient sketch sizes (e.g., 100) may mask true evolutionary relationships, producing distance estimates with high variance. For publication-quality analyses, use sketch sizes of at least 1000, adjusting based on genome size and desired confidence intervals.

- Forgetting that bindash operates on raw sequences rather than aligned references can cause confusion when interpreting results for highly divergent genomes. Distances reflect k-mer sharing, not nucleotide-level alignment.

- Large-scale comparisons may exhaust memory when sketch files are loaded simultaneously. Processing in batches or using the streaming `dist` mode with pre-built indexes mitigates this issue for datasets exceeding available RAM.

## Examples

### Generate a MinHash sketch from a single bacterial genome FASTA file
**Args:** `sketch --threads 4 -k 31 -s 2000 input_genome.fna.gz -o genome.sketch`
**Explanation:** Creates a compact sketch file from a compressed FASTA file using 4 threads, k-mer size 31, and 2000 hashes, enabling rapid comparisons with other sketched genomes.

### Compute pairwise distances between three bacterial genome sketches
**Args:** `dist -k 31 genome1.sketch genome2.sketch genome3.sketch -o distances.tsv`
**Explanation:** Calculates pairwise Jaccard similarity distances between pre-computed sketches and outputs a tab-delimited distance matrix for downstream phylogenetic or clustering analysis.

### Generate sketches from all FASTA files in a directory for batch comparison
**Args:** `sketch -k 45 -s 5000 --threads 8 genomes/*.fna -o all_sketches.db`
**Explanation:** Processes multiple genome files in parallel to create a unified sketch database, using larger k-mers (45) and sketch sizes (5000) suitable for highly complex or eukaryotic genomes.

### Build a neighbor-joining tree from computed distances
**Args:** `dist -k 31 -t newick reference_sketches/ -o tree.nwk`
**Explanation:** Computes distances between all pairs in a directory of reference sketches and outputs a Newick-formatted tree directly, bypassing intermediate matrix files for streamlined phylogenetics.

### Compare a metagenome assembly against a database of reference genomes
**Args:** `dist -k 21 -s 10000 metagenome.sketch ref_sketches/*.sketch -o metagenome_distances.txt`
**Explanation:** Estimates taxonomic composition by comparing a metagenome sketch against a collection of reference genome sketches using smaller k-mers optimized for short-read-derived assemblies.