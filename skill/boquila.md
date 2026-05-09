---
name: boquila
category: Sequence Alignment / Read Mapping
description: A fast DNA sequence read mapper that aligns short sequencing reads to a reference genome using indexed lookups. Supports single-end and paired-end reads, various output formats, and alignment scoring parameters.
tags: [read-mapping, sequence-alignment, NGS, bioinformatics, genomics, FASTQ, SAM]
author: AI-generated
source_url: https://github.com/ncbi/boquila
---

## Concepts

- **Data Model**: Boquila takes FASTQ (or FASTQ.gz compressed) read files as input and aligns them to an indexed reference genome (built with boquila-build). Output is in SAM/BAM format by default.
- **Index Structure**: The reference genome must be pre-indexed using the companion binary `boquila-build` before alignment. The index consists of multiple files with `.bt2` extensions.
- **Paired-End Alignment**: When processing paired-end reads, use `-1` and `-2` flags to specify the two read files in the correct order. Boquila automatically calculates proper pairs and sets the paired flag in SAM output.
- **Scoring Parameters**: Alignment scoring uses match (+1), mismatch (-1), and gap penalties that can be customized via command-line options to balance sensitivity and speed.

## Pitfalls

- **Using unindexed reference**: Running boquila without first building an index with boquila-build will cause the tool to fail with a file-not-found error. Always run `boquila-build reference.fasta index_name` before alignment.
- **Mismatched read file formats**: Supplying files in the wrong order for paired-end reads (e.g., specifying the reverse read file as `-1` instead of `-2`) produces invalid pairing information in the SAM output, corrupting downstream analysis.
- **Memory exhaustion with large genomes**: Specifying insufficient memory allocation (`--max-bgz` default may be too low for mammalian-sized genomes) causes indexing or alignment to terminate abnormally.
- **Overwriting index files**: Re-running boquila-build with the same output index name overwrites existing index files without prompting, potentially breaking workflows that depend on the previous index.

## Examples

### Align single-end reads to an indexed reference
**Args:** -x index_name -U reads.fq.gz -S alignment.sam
**Explanation:** Maps single-end reads from a gzipped FASTQ file to the reference using the pre-built index and outputs results in SAM format.

### Align paired-end reads with standard settings
**Args:** -x index_name -1 read1.fq -2 read2.fq -S paired_output.sam
**Explanation:** Aligns both ends of paired-end reads, marking proper pairs in the SAM output and using default scoring.

### Output in BAM format with sorted coordinates
**Args:** -x index_name -U reads.fq --output-bam -o sorted_output.bam
**Explanation:** Produces binary BAM output instead of text SAM, automatically sorting by genomic coordinates.

### Adjust mismatch penalty for stricter alignment
**Args:** -x index_name -U reads.fq -S output.sam --mp 3
**Explanation:** Increases mismatch penalty to 3, making the aligner prefer gaps over mismatches for higher specificity.

### Use 8 threads for parallel alignment
**Args:** -x index_name -U reads.fq -S output.sam -p 8
**Explanation:** Enables multi-threaded processing with 8 threads, significantly speeding up alignment on multi-core systems.

### Set minimum anchor length for spliced alignment
**Args:** -x index_name -U reads.fq -S output.sam --min-anchor 8
**Explanation:** Requires at least 8 bases on each side of a splice junction, reducing false positive spliced alignments.

### Limit maximum number of alignments reported per read
**Args:** -x index_name -U reads.fq -S output.sam -k 3
**Explanation:** Reports only the top 3 alignments for multi-mapping reads, keeping output file size manageable.

### Build index with smaller bucket size for faster lookup
**Args:** index_name reference.fasta --offrate 2
**Explanation:** Creates index with reduced offset rate (faster but larger index file) when building with boquila-build.