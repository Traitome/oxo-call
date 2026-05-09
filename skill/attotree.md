---
name: attotree
category: Phylogenetics
description: A tool for compressing, indexing, and querying large phylogenetic trees using reference-based compression algorithms. Attotree reduces storage requirements for tree collections while enabling fast tree traversal and subtree extraction operations.
tags: [phylogenetics, tree-compression, newick, index, compression, bioinformatics]
author: AI-generated
source_url: https://github.com/attroptools/attotree
---

## Concepts

- **Input Formats**: Attotree accepts Newick (.nw, .nwk) and Nexus (.nex) formatted phylogenetic trees. Multiple trees can be batch-processed from a directory or a single file containing concatenated trees.
- **Compression Model**: The tool uses reference-based compression where common subtrees (shared clades) are stored once and referenced multiple times, dramatically reducing file size for collections of similar trees such as bootstrap replicates or Bayesian posterior samples.
- **Index Structure**: Creates a binary index (.ati) alongside the compressed tree (.atc) that enables fast operations including subtree extraction, most recent common ancestor (MRCA) queries, and branch length lookups without decompressing the entire tree.
- **Output Formats**: Compressed trees can be exported back to Newick, Nexus, or a compact JSON representation. The index supports programmatic access through the attotree API library.

## Pitfalls

- **Missing Index File**: Attempting to query a compressed tree without the corresponding .ati index file will fail. Always preserve both .atc and .ati files together, or rebuild the index using attotree-index if either file is lost.
- **Non-ultrametric Trees**: The MRCA query functions assume timed (ultrametric) trees with valid branch lengths. Running MRCA queries on unscaled trees produces undefined results or crashes.
- **Memory Limits with Large Inputs**: Trees with more than 10 million taxa may exceed default memory allocation. Use the --max-memory flag to specify larger limits, otherwise the process will be terminated by the system.
- **Incompatible Newick Syntax**: Attotree requires strict Newick syntax—missing semicolons, unquoted names with special characters, or malformed parentheses will cause parsing failures with unclear error messages.

## Examples

### Compress a single Newick tree file
**Args:** -i tree.nw -o tree.atc
**Explanation:** Reads a Newick tree and creates a compressed archive, reducing storage size by storing repeated subtree patterns only once.

### Compress multiple trees from a directory
**Args:** -i input_trees/ -o trees.atc --batch
**Indexes:** Recursively processes all .nw and .nwk files in the input directory and creates a single compressed archive containing all trees with a unified index.

### Extract a subtree by taxon list
**Args:** tree.atc --extract-taxon "Homo_sapiens,Pan_troglodytes" -o human_chimp.nw
**Explanation:** Uses the compressed index to quickly locate and extract the clade containing only the specified taxa without decompressing the entire tree.

### Query the most recent common ancestor
**Args:** tree.atc --mrcas "TaxonA,TaxonB,TaxonC" --output-json
**Explanation:** Returns the node representing the MRCA of the specified taxa in JSON format, leveraging the pre-built index for fast lookups.

### Rebuild the index for an existing compressed tree
**Args:** tree.atc --rebuild-index -o tree_new.ati
**Explanation:** Regenerates the binary index file if it became corrupted or was lost, scanning the compressed tree to rebuild lookup structures.

### Export compressed tree to Newick format
**Args:** tree.atc --export newick_output.nw
**Explanation:** Decompresses the archive and writes a standard Newick file, useful for converting between formats or converting legacy tree collections.

### Set maximum memory allocation for large trees
**Args:** -i huge_tree.nw -o huge.atc --max-memory 16G
**Explanation:** Allocates 16 gigabytes of RAM for processing, preventing out-of-memory errors when compressing trees with millions of taxa.

### List all taxon names in a compressed tree
**Args:** tree.atc --list-taxa --output-file taxa.txt
**Explanation:** Extracts and writes all unique taxon names from the compressed tree to a text file, using the index for efficient traversal without full decompression.

### Compress with custom branch length precision
**Args:** -i tree.nw -o tree.atc --precision 4
**Explanation:** Quantizes branch lengths to 4 decimal places during compression, trading slight numerical precision for improved compression ratios.

### Verify integrity of compressed tree file
**Args:** tree.atc --verify
**Explanation:** Performs a checksum validation of the compressed archive and index, reporting any corruption or mismatches between files.