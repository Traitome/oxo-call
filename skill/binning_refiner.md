---
name: binning_refiner
category: Metagenomics / Genome Binning
description: Refines draft genome bins from metagenomic assemblies by integrating read coverage, k-mer signatures, and taxonomic consistency scores to improve completion and reduce contamination.
tags: [metagenomics, binning, refinement, coverage, assembly, MAGs]
author: AI-generated
source_url: https://github.com/symenstcal/binning_refiner
---

## Concepts

- binning_refiner takes as input a set of draft genome bins (`.fa`/`.fna` files) produced by upstream binners such as MetaBAT, MaxBin2, or CONCOCT, along with their co-assembly BAM files and an assembly FASTA. It re-scores each contig-to-bin assignment using coverage depth profiles and k-mer composition, then outputs a refined set of bins in which mis-binned contigs are reassigned or removed.
- Output bins are written to a single directory as individual FASTA files (one per refined bin), alongside a `bin_stats.tsv` summary table that reports per-bin estimated completeness, contamination, and N50-like metrics derived from the contigs within each bin.
- The tool operates in two phases: (1) a coverage-based coherence pass that flags contigs whose read depth deviates sharply from the bin median, and (2) a k-mer signature pass that compares each flagged contig's tetranucleotide frequencies against the recipient bin's centroid, applying a similarity threshold below which reassignment is blocked.
- binning_refiner does NOT perform de novo binning; it requires an existing bin set and a co-assembly (either per-sample or pooled). It is agnostic to the binner used to generate the input bins, provided they are supplied as individual FASTA files.

## Pitfalls

- Supplying BAM files aligned to individual sample assemblies rather than the co-assembly used for binning causes coverage values to be miscomputed, leading to spurious contig reassignments or excessive contamination scores in the output.
- Omitting the `-c` / `--checkm` flag when checkm2 results are available results in lower-quality refinement because the tool falls back to an internal, less accurate completeness estimator, producing bins that appear cleaner than they actually are.
- Setting the `--depth-threshold` to an extreme value (e.g., 0.01) disables the coverage pass entirely, causing the tool to rely solely on k-mer signatures which are less discriminative for contigs shorter than 1 kbp, increasing the risk of fragmented or merged bins.
- Mixing bins generated from different assembly versions or different co-assemblies in the same input directory causes alignment ambiguities, since contig identifiers will not match the BAM reference, resulting in a crash or all-zero coverage values.
- Using a single-sample BAM when the original binning was performed on a pooled co-assembly leads to incomplete coverage profiles for low-abundance species, causing their contigs to be erroneously expelled from their bins.

## Examples

### Refine a set of bins using a pooled co-assembly BAM and default thresholds
**Args:** `-b bins/ -a assembly.fasta -m aligned.bam -o refined_output/`
**Explanation:** This provides the tool with all required inputs: the directory of draft FASTA bins, the co-assembly, its BAM alignment, and the output directory, triggering both coverage and k-mer refinement passes with default settings.

### Refine bins and reuse pre-computed checkm2 completeness scores
**Args:** `-b bins/ -a assembly.fasta -m aligned.bam -o refined_output/ -c checkm2_out.tsv`
**Explanation:** Passing a pre-existing checkm2 report via the `-c` flag allows binning_refiner to bypass its internal completeness estimator and use the more accurate scores, improving reassignment decisions near quality thresholds.

### Perform a quick coverage-only pass without k-mer refinement to save compute time
**Args:** `-b bins/ -a assembly.fasta -m aligned.bam -o refined_output/ --depth-threshold 0.05 --skip-kmers`
**Explanation:** Setting a moderate depth threshold and skipping the k-mer pass runs only the coverage coherence check, useful for iterating quickly on a large bin set during preliminary analysis.

### Increase stringency to reduce contamination at the cost of bin fragmentation
**Args:** `-b bins/ -a assembly.fasta -m aligned.bam -o refined_output/ --depth-threshold 0.02 --kmers-cutoff 0.85 --contamination-max 5`
**Explanation:** Tightening both the depth threshold and the k-mer similarity cutoff alongside a maximum allowable contamination of 5% causes more contigs to be removed, yielding cleaner but potentially more fragmented bins with lower completion.

### Export refined bins alongside per-bin N50 and contig count statistics
**Args:** `-b bins/ -a assembly.fasta -m aligned.bam -o refined_output/ --stats-file refined_stats.tsv --stats-detail`
**Explanation:** The `--stats-file` flag writes a tab-separated table of per-bin metrics, and adding `--stats-detail` includes contig counts and N50 values, facilitating downstream quality filtering or comparison with the original bin set.

### Refine bins using only the k-mer pass (no BAM) for assembled contigs with insufficient read coverage
**Args:** `-b bins/ -a assembly.fasta -o refined_output/ --no-coverage --kmers-cutoff 0.75`
**Explanation:** When BAM coverage data is unreliable (e.g., for low-depth samples), disabling the coverage pass and relying on k-mer signatures with a lowered cutoff still produces refinement, though with reduced sensitivity for short contigs.

### Parallelize refinement across 16 threads to speed up large datasets
**Args:** `-b bins/ -a assembly.fasta -m aligned.bam -o refined_output/ -t 16 --stats-file stats.tsv`
**Explanation:** The `-t 16` flag distributes k-mer computation and scoring across 16 threads, while the stats file is still written, enabling faster processing of hundreds of bins without altering the quality of the output.