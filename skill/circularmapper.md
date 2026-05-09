---
name: circularmapper
category: read_mapping
description: Remap reads to circular genomes by accounting for circular wrap-around at sequence ends, part of the BBTools suite. Primarily used for mapping reads to circular references like plasmids, mitochondria, and viral genomes.
tags: [bbtools, read_mapping, circular genomes, remapping, sam, bam]
author: AI-generated
source_url: https://sourceforge.net/projects/bbmap/
---

## Concepts

- **Circular Reference Handling**: circularmapper remaps reads that were originally mapped to linear representations of circular genomes. It accounts for the sequence wrap-around by detecting reads that span the end of the reference and re-mapping them correctly to the beginning (and vice versa).
- **Companion Binary**: The tool uses `circularmapper-build` to generate indices for circular references. The index must be built specifically for circular genomes using the `-circular=true` flag for proper wrap-around handling.
- **Input/Output Formats**: Accepts SAM or BAM format files as input (-in=) and outputs remapped SAM/BAM (-out=). The reference must be provided in FASTA format (-ref=).
- **Index Building**: Uses a k-mer based index (similar to Bowtie2) that stores the circular nature of the reference. The index is stored in a directory alongside the reference file.

## Pitfalls

- **Forgetting to Rebuild Index**: Running circularmapper without first rebuilding the index with `circularmapper-build` causes reads to map incorrectly because the original index doesn't account for circular wrapping.
- **Not Specifying Circular Flag**: Forgetting to use `-circular=true` when building the index with circularmapper-build results in a linear index that doesn't handle wrap-around, causing incorrect mapping at genome ends.
- **Reference Name Mismatch**: If the sequence header in your FASTA file doesn't match the @SQ header in your SAM/BAM file exactly, the tool will fail to find the reference or produce empty output.
- **Memory with Large Genomes**: Building indices for large circular genomes (like bacterial chromosomes) without sufficient memory can cause the process to crash or run extremely slowly.

## Examples

### Remap reads from a circular plasmid reference
**Args:** -in=mapped_plasmid.sam -out=remapped_plasmid.sam -ref=plasmid.fa
**Explanation:** Takes previously mapped SAM reads and remaps them accounting for the circular nature of the plasmid sequence, fixing reads that span the ends.

### Build an index for a circular viral genome
**Args:** -ref=virus.fa -path=virus_index -circular=true
**Explanation:** Creates a k-mer index specifically designed for circular genome wrap-around, enabling proper read remapping for viral sequences.

### Remap BAM input to BAM output
**Args:** -in=mapped.bam -out=remapped.bam -ref=mtDNA.fa -Xmx4g
**Explanation:** Process binary BAM format directly for mitochondrial genomes, with 4GB of memory allocated for larger datasets.

### Rebuild an existing circular index
**Args:** -ref=existing.fa -path=existing_index -rebuild -circular=true
**Explanation:** Regenerates an existing index with circular parameters, useful when updating references or fixing broken indices.

### Process multiple input files in a directory
**Args:** -inDir=./mapped_reads/ -outDir=./remapped_reads/ -ref=genome.fa
**Explanation:** Batch processes all SAM/BAM files in a directory, remapping each to account for circular genome wrap-around.