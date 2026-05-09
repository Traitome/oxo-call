---
name: bleties
category: sequence_alignment
description: A fast nucleotide/protein alignment tool for short reads and sequences. Performs local alignment searches using indexed databases for rapid mapping of query sequences against reference sequences.
tags: [alignment, sequence-mapping, fastq, fasta, genomics, short-reads, indexing]
author: AI-generated
source_url: https://github.com/bleties/bleties
---

## Concepts

- **Input Formats**: Accepts FASTA, FASTQ, and plain text query sequences. Reference databases can be built from FASTA files using the companion `bleties-build` tool for accelerated lookups.
- **Alignment Modes**: Supports local alignment (primary mode), global alignment, and gap-free exact matching. Outputs alignments in PSL ( BLAST-like format), SAM, or custom tab-delimited formats.
- **Index-Based Acceleration**: Uses disk-backed index files (`.bit` extension) created by `bleties-build`, enabling queries against multi-gigabyte reference databases without loading them entirely into memory.
- **Output Stream**: Writes to standard output by default; use shell redirection (`>`) to capture results. Can emit multiple alignment hits per query when configured.
- **Sensitivity vs Speed Tradeoff**: Adjustable via tile size and step size parameters — smaller tiles increase sensitivity for divergent sequences at the cost of runtime; larger tiles improve speed but may miss weak matches.

## Pitfalls

- **Reference Index Not Built**: Running `bleties` without a pre-built index using `bleties-build` will fail or produce extremely slow output, as the tool searches the raw FASTA file sequentially without acceleration.
- **Incompatible File Encodings**: Query files containing non-standard characters (binary data, control characters, or Windows-style line endings without proper preprocessing) cause parsing errors and empty outputs.
- **Memory Limits on Large Queries**: Submittingextremely large query files (millions of sequences) without increasing the memory allocation via the `--maxmem` flag results in out-of-memory errors or premature termination.
- **Output Format Mismatch**: Using an output format flag that the installed version does not support silently defaults to PSL format, potentially breaking downstream pipelines expecting SAM/BAM.

## Examples

### Build an index from a reference FASTA file
**Args:** -build reference.fasta reference_index
**Explanation:** Creates a binary index named `reference_index.bit` from the input FASTA for accelerated subsequent alignment searches.

### Align a single query sequence against an indexed database
**Args:** query.fasta -db reference_index -outfmt psl
**Explanation:** Searches the query sequence against the pre-built index and outputs results in PSL format.

### Output alignments in SAM format
**Args:** queries.fastq -db reference_index -outfmt sam -o alignments.sam
**Explanation:** Writes alignments in Sequence Alignment/Map (SAM) format, suitable for visualization in genome browsers and downstream variant calling tools.

### Restrict to top-scoring alignment only
**Args:** reads.fq -db hg38_index -maxhits 1 -outfmt psl
**Explanation:** Returns only the single best-scoring alignment per query sequence, reducing output volume for strictly primary mappings.

### Adjust tile size for higher sensitivity
**Args:** divergent_queries.fasta -db ref_index -tilesize 8 -step 1 -outfmt tabular
**Explanation:** Uses smaller tile size (8 bp) and step (1 bp) to detect more-divergent alignments, at the cost of slower runtime.

### Filter alignments by minimum alignment score
**Args:** input.fq -db index -minscore 30 -outfmt psl
**Explanation:** Excludes alignments scoring below 30, effectively filtering out low-confidence or spurious matches from the result set.