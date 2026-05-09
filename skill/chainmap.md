---
name: chainmap
category: Genome Alignment / Assembly
description: Generates UCSC chain format files from pairwise alignments between two genomic sequences. Used to create custom liftOver chain files for converting coordinates between different genome assemblies when standard chains are unavailable.
tags:
  - chain-format
  - liftOver
  - genome-assembly
  - coordinate-conversion
  - alignment
  - ucsc
  - bioinformatics
author: AI-generated
source_url: https://hgdownload.soe.ucsc.edu/admin/exe/
---

## Concepts

- **Chain format structure**: Chain files represent alignments as a series of aligned blocks separated by gaps. Each chain begins with a header line containing score, source sequence ID, size, and target sequence ID, followed by aligned blocks defined by match lengths and gap sizes (deletions in the query or insertions in the target).

- **Input requirements**: The tool requires two genomic sequences in FASTA or 2bit format—one serving as the reference (target assembly) and one as the query (source assembly)—and aligns them to produce a coordinate mapping.

- **Output for liftover**: Generated chain files can be used with UCSC's liftOver utility to convert BED, GTF, or other coordinate-based files from one genome assembly to another, enabling cross-assembly analysis when pre-built chains from UCSC do not exist for your specific species or strain.

- **Alignment algorithm**: ChainMap employs a local alignment strategy that identifies collinear syntenic blocks between the two sequences, prioritizing larger alignments over fragmented matches to produce coherent coordinate mappings.

## Pitfalls

- **Incorrect sequence orientation**: Failing to specify the correct strand orientation (forward vs reverse) results in chain files that produce inverted coordinates, causing all converted positions to appear on the wrong strand and leading to downstream annotation errors.

- **Version mismatches between assemblies**: Using sequences from different and incompatible assembly versions without accounting for structural variations (e.g., different chromosome naming schemes or centromere insertions) produces incorrect alignments that cannot be used for accurate coordinate conversion.

- **Overly divergent sequences**: Attempting to align sequences with extremely low identity (>30% divergence) generates fragmented chains with many small blocks, which liftOver typically rejects or converts with low recovery rates, wasting computational resources and producing unreliable results.

- **Memory constraints with large genomes**: Aligning whole chromosome-scale sequences without sufficient memory allocation causes the tool to fail or terminate prematurely, leaving incomplete chain files that cannot be validated for coordinate conversion.

## Examples

### Generate a chain file from two genomic sequences aligned via global alignment

**Args:** -t=Target.2bit -q=Query.2bit -o=align.chain

**Explanation:** This creates a chain file by aligning the query sequence to the target 2bit file, outputting the alignment result for downstream liftOver operations.

### Specify minimum alignment score threshold

**Args:** -t=ref.fa -q=query.fa -o=output.chain -minScore=1000

**Explanation:** Setting a minimum score filters out lower-quality alignments, ensuring only statistically significant matches are included in the generated chain file.

### Use a specific alignment algorithm

**Args:** -t=hg38.2bit -q=panTro6.2bit -o=human_chimp.chain -algorithm=LA

**Explanation:** The -algorithm=LA flag instructs chainmap to use a specific alignment algorithm (in this case, a local alignment strategy) that may better capture evolutionary relationships between primate genomes.

### Generate chain with verbose logging for debugging

**Args:** -t=target.2bit -q=query.2bit -o=debug.chain -verbose

**Explanation:** The verbose flag outputs detailed alignment statistics and intermediate processing steps, useful for diagnosing why a chain file has unexpected gaps or orientation issues.

### Limit the maximum gap size in alignments

**Args:** -t=mm10.2bit -q=rn7.2bit -o=mouse_rat.chain -maxGap=10000

**Explanation:** Restricting the maximum gap size prevents the alignment algorithm from creating overly large jump regions, producing more granular but potentially more accurate coordinate mappings for closely related rodent genomes.

### Convert a BED file using the generated chain

**Args:** input.bed output.bed -chain=align.chain -bedPlus=5

**Explanation:** This example demonstrates the downstream usage—taking the chain file produced by chainmap and applying it to convert a BED format file with 5+ columns from one assembly to another.

### Specify an alternative output format for compatibility

**Args:** -t=target.fa -q=query.fa -o=custom.chain -format=chain

**Explanation:** Explicitly specifying the output format ensures the generated chain file conforms to the standard UCSC chain specification, guaranteeing compatibility with liftOver and other UCSC utility tools.

### Override default alignment scoring parameters

**Args:** -t=target.2bit -q=query.2bit -o=scored.chain -match=2 -mismatch=-3

**Explanation:** Custom scoring parameters adjust the alignment penalties for matches and mismatches, allowing fine-tuning of alignment sensitivity based on the expected divergence between the two genomic sequences.