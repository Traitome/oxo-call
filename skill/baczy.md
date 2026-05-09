---
name: baczy
category: Sequence Alignment
description: A fast short read aligner that maps sequencing reads to a reference genome using FM-index based algorithms. Supports single-end and paired-end reads with configurable mismatch and gap penalties.
tags:
  - alignment
  - fm-index
  - genomics
  - short-reads
  - ngs
author: AI-generated
source_url: https://github.com/baczy/baczy
---

## Concepts

- **FM-index based alignment**: baczy uses a full-text minute index (FM-index) for memory-efficient and fast read mapping, similar to BWA and Bowtie2. The index must be built using the companion `baczy-build` tool before alignment.
- **Input formats**: Accepts FASTQ (.fq/.fastq) and FASTA (.fa/.fasta) files for reads, and FASTA for the reference genome. Supports gzip-compressed inputs (.fq.gz).
- **Output formats**: Produces SAM (Sequence Alignment/Map) format by default, with optional BAM output when piped through `samtools view`. Includes mapping quality (MAPQ) scores and CIGAR strings.
- **Paired-end support**: Uses `-1` and `-2` flags for mate pairs in FR (forward-reverse) orientation. Automatically infers library type when not specified.
- **Seed-based algorithm**: Aligns reads using a seed-and-extend heuristic, with configurable seed length (`-l`) and maximum seed extensions (`-e`).

## Pitfalls

- **Forgetting to build the index**: Running `baczy` without first creating an index with `baczy-build` will fail with an "index not found" error. Always run `baczy-build reference.fa index` before alignment.
- **Mismatching read files**: Specifying the wrong order of mate files in `-1` and `-2` produces incorrect pair information and artificially elevates the duplicate mapping rate.
- **Excessive mismatch tolerance**: Setting `-v` (max mismatches) too high (e.g., 10) drastically increases runtime and produces spurious alignments, especially in low-complexity regions.
- **Incompatible paired-end library orientation**: Using default FR orientation for RF-stranded libraries produces inverted mate pairs in output, corrupting downstream variant calling.
- **Uncompressed large outputs**: Writing SAM to disk for large datasets fills storage quickly; always pipe output through `samtools view -bS` to produce BAM.

## Examples

### Build an FM-index from a reference genome
**Args:** reference.fa myindex
**Explanation:** Creates the FM-index files required for alignment. Run once per reference; index files reuse across multiple samples.

### Map single-end reads to a reference
**Args:** -f reads.fq -i myindex -o output.sam
**Explanation:** Aligns single-end reads in FASTQ format to the indexed reference and outputs alignments in SAM format.

### Map paired-end reads with mate files
**Args:** -1 left.fq -2 right.fq -i myindex -o paired.sam
**Explanation:** Aligns paired-end reads in proper mate orientation, computing proper pairs based on insert size distribution.

### Set maximum allowed mismatches to 3
**Args:** -f reads.fq -i myindex -o output.sam -v 3
**Explanation:** Filters alignments allowing up to 3 mismatches in the seed region, balancing sensitivity and specificity.

### Align with a seed length of 20 bases
**Args:** -f reads.fq -i myindex -o output.sam -l 20
**Explanation:** Uses a 20-base seed for initial matching before extension, improving speed at the cost of sensitivity in highly divergent reads.

### Specify FR library orientation for paired-end data
**Args:** -1 left.fq -2 right.fq -i myindex -o output.sam --fr
**Explanation:** Tells baczy the library is forward-reverse oriented, enabling correct mate pair validation and orientation flags.

### Increase maximum gap extensions to 10
**Args:** -f reads.fq -i myindex -o output.sam -e 10
**Explanation:** Allows up to 10 base extensions during gap realignment, improving alignment near indels but increasing runtime.

### Output to BAM format via pipe
**Args:** -f reads.fq -i myindex -o - | samtools view -bS - > output.bam
**Explanation:** Streams SAM to standard output and converts to BAM in a single pipeline, reducing disk I/O for large datasets.

### Map reads with maximum fragment size of 500bp
**Args:** -1 left.fq -2 right.fq -i myindex -o output.sam -X 500
**Explanation:** Sets the maximum allowed distance between properly paired mates to 500bp, filtering misassemblies in small-fragment libraries.

### Use 8 threads for parallel alignment
**Args:** -f reads.fq -i myindex -o output.sam -t 8
**Explanation:** Parallelizes alignment across 8 threads, significantly reducing runtime on multi-core systems.

### Skip unaligned reads in output
**Args:** -f reads.fq -i myindex -o output.sam --no-unmapped
**Explanation:** Suppresses lines for reads with no valid alignment, reducing output file size for downstream processing.