---
name: catch
category: sequence-analysis
description: A bioinformatics tool for retrieving sequences from multi-FASTA databases using tag-based selection criteria. CATCH enables fast extraction of subsequences based on identifiers, taxonomic annotations, or sequence length filters.
tags: [sequence-retrieval, multi-fasta, sequence-filtering, genomics, bioinformatics, sequence-extraction]
author: AI-generated
source_url: https://github.com/harrysuzhou/CATCH
---

## Concepts

- **Multi-FASTA Input Handling**: CATCH operates on indexed multi-FASTA files, allowing rapid random access to individual sequence records without parsing the entire file sequentially.
- **Tag-Based Filtering**: Sequences are selected using identifier tags, annotation patterns, or metadata fields embedded in sequence headers, enabling precise subset extraction for downstream analysis.
- **Output Formats**: Extracted sequences can be written to single or multiple output FASTA files, preserving quality scores and header annotations from the source database.
- **Indexing Requirement**: Before retrieval, the multi-FASTA file must be indexed using the companion `catch-index` or `catch-build` utility to create lookup structures for efficient access.

## Pitfalls

- **Unindexed FASTA Files**: Running `catch` on a FASTA file that has not been previously indexed will produce errors or incomplete results, as the tool relies on pre-built lookup indices.
- **Ambiguous Tag Matches**: Using overly generic or partial tags may result in unintended sequence matches, leading to incorrect or bloated output datasets.
- **Memory Constraints with Large Databases**: When filtering very large multi-FASTA files with broad criteria, memory usage can spike significantly, causing performance degradation or crashes on resource-limited systems.
- **Mismatched Header Formats**: If sequence headers do not follow the expected tag format, tag-based filtering will silently fail to match any sequences, producing empty output files without warning.

## Examples

### Extract sequences by exact identifier match
**Args:** `input.db.fasta --tag ENST00000263255 --out extracted.fa`
**Explanation:** Retrieves a single sequence record from the indexed database whose header contains the exact identifier "ENST00000263255".

### Filter sequences by taxonomic annotation pattern
**Args:** `refseq_proteins.fasta --tag-path /Homo_sapiens/ --out human_proteins.fa`
**Explanation:** Extracts all protein sequences with taxonomic path annotations containing "Homo_sapiens" in their header metadata fields.

### Select sequences by length range
**Args:** `transcriptome.fasta --min-length 500 --max-length 3000 --out medium_transcripts.fa`
**Explanation:** Filters the input database to include only sequences with lengths between 500 and 3000 nucleotides or amino acids.

### Exclude specific identifiers from output
**Args:** `sequences.fasta --exclude-tag mitochondrial --out nuclear_only.fa`
**Explanation:** Removes any sequences whose headers contain the tag "mitochondrial", outputting all remaining sequences from the database.

### Batch extraction using tag list file
**Args:** `genome.fasta --tag-list target_genes.txt --out batch_results.fa`
**Explanation:** Reads multiple target tags from the specified file and extracts all matching sequences in a single operation, preserving separate headers in output.