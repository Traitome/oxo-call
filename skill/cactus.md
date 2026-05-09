---
name: cactus
category: multiple-sequence-alignment
description: A progressive multiple genome alignment tool that aligns multiple genomes against a reference genome using the HAL (Hierarchical Alignment) format. Cactus builds a guided alignment tree and progressively adds genomes from closest to most distant relatives.
tags: genomics, multiple-alignment, hal, comparative-genomics, whole-genome-alignment
author: AI-generated
source_url: https://github.com/ComparativeGenomicsToolKit/cactus
---

## Concepts

- **HAL Input/Output Model**: Cactus operates on HAL (Hierarchical Alignment) format files, which store multiple genome alignments as anchor blocks linked to a reference genome. Input genomes must be indexed using `cactus-preprocess` or converted to HAL using `hal2hal` before alignment.
- **Reference Genome Requirement**: The first genome in the input list or explicitly specified via `--referenceGenome` serves as the alignment anchor. All other genomes are aligned relative to this reference, making the choice of reference critical for alignment quality.
- **Progressive Alignment Tree**: Cactus builds a guided alignment tree based on phylogenetic relationships. Genomes closer to the reference are aligned first, then their alignments are used as anchors for more distant genomes, reducing computational cost and improving accuracy.
- **Job Parallelization**: Large alignments benefit from `--batchMode` and specifying `--jobTree` with a parallel job tree implementation (like `local` or `SGE`) to distribute computational work across multiple cores or compute nodes.

## Pitfalls

- **Mismatched Genome Names**: If genome names in the input HAL file do not exactly match the names specified in the command, Cactus will silently fail to find the genomes, producing empty or partial alignments.
- **Insufficient Memory for Large Alignments**: Aligning dozens of mammalian genomes can require hundreds of gigabytes of RAM. Running without `--maxMemory` specification may cause out-of-memory failures on constrained systems.
- **Reference Selection in Low-Quality Regions**: Using a highly fragmented or repeat-rich genome as the reference can propagate errors throughout the alignment. Select the most high-quality, well-assembled genome as the reference.
- **Incompatible HAL Toolkits**: Cactus outputs HAL format files that require the HAL tools (halLiftover, halStats) for extraction. If downstream tools only accept BED, MAF, or FASTA, additional conversion steps are necessary.

## Examples

### Align two genomes using a default reference
**Args:** --referenceGenome=hg38 --outHDF5=alignment.hal genome1.fa genome2.fa
**Explanation:** Aligns genome2.fa against hg38 as the reference, outputting a HAL file containing both genomes and their alignment.

### Specify an explicit FASTA reference genome
**Args:** --referenceGenome=chr22 --outHDF5=chr22_alignment.hal ref.fa query1.fa query2.fa
**Explanation:** Uses the genome in ref.fa explicitly as the reference, ensuring all alignment anchors derive from that sequence.

### Use a preexisting chain file for guided alignment
**Args:** --chainAlignments=existing.chain --outHDF5=guided.hal ref.fa newGenomes.fa
**Explanation:** Incorporates preexisting alignment information from a UCSC chain file to guide cactus to higher-confidence alignment regions, reducing runtime.

### Run with multiple cores for faster processing
**Args:** --jobTree=local --batchMode --outHDF5=parallel_alignment.hal ref.fa genome1.fa genome2.fa genome3.fa
**Explanation:** Enables parallel execution across local cores using batch mode, significantly accelerating alignment of multiple genomes.

### Extract alignment statistics after completion
**Args:** --referenceGenome=hg38 alignment.hal
**Explanation:** Running cactus with only the input HAL and reference genome flag displays alignment statistics including coverage, identity, and coordinate mappings without performing a new alignment.

### Limit memory usage to avoid system crashes
**Args:** --maxMemory=64G --jobTree=local --outHDF5=memory_limited.hal ref.fa query.fa
**Explanation:** Restricts cactus to using at most 64GB of RAM, which is necessary when running on shared compute nodes with memory limits.

### Enable debug logging for troubleshooting
**Args:** --logLevel=DEBUG --outHDF5=debug_output.hal ref.fa query.fa
**Explanation:** Enables verbose debug logging to diagnose alignment failures, missing genomes, or unexpected behavior during runtime.