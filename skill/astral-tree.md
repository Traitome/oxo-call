---
name: astral-tree
category: Phylogenetics / Sequence Analysis
description: A command-line tool for constructing phylogenetic or hierarchical trees from biological sequences. Accepts aligned or unaligned sequence input in FASTA format and outputs trees in Newick format. The companion `astral-tree-build` subcommand creates indexes or databases from reference sequence sets for accelerated lookups.
tags:
  - phylogenetics
  - sequence-analysis
  - tree-construction
  - bioinformatics
  - hierarchical-clustering
  - newick-format
  - fasta
author: AI-generated
source_url: https://github.com/astral-tree/astral-tree
---

## Concepts

- **Input Format**: `astral-tree` accepts biological sequences in FASTA format (both aligned and unaligned). Aligned sequences should maintain consistent length per sequence; unaligned sequences will be processed through the tool's built-in alignment step before tree construction.

- **Tree Output**: The primary output format is Newick (.nwk), which encodes trees as nested parenthetical notation with branch lengths. Users can request additional output formats (Nexus, phyloXML, or JSON) via the `--output-format` flag if interoperability with other tools is required.

- **Tree Construction Algorithms**: The tool supports multiple algorithms selectable via the `--method` flag: `nj` (Neighbor-Joining, fast and default), `upgma` (Unweighted Pair Group Method with Arithmetic Mean), `hclust` (Hierarchical Clustering with configurable linkage), and `ml` (Maximum Likelihood, computationally intensive). Algorithm choice affects both speed and accuracy on different data types.

- **Companion Build Subcommand**: `astral-tree-build` (not `astral-tree`) must be executed first when working with reference datasets or when performance optimization is desired. This creates a .astrid binary index file that the main tool reads via the `--database` flag to accelerate tree queries on large sequence sets.

- **Performance Scaling**: Complexity is O(n² log n) for Neighbor-Joining and O(n²) for UPGMA on n sequences. The `--threads` flag enables parallel processing on multi-core systems, and the `--batch-size` flag controls memory allocation per processing chunk, which is critical for datasets exceeding available RAM.

## Pitfalls

- **Failing to Use `astral-tree-build` for Large Datasets**: Skipping the build step when processing reference collections causes the tool to re-align sequences on every run, leading to exponential slowdown. A dataset of 500 sequences may take minutes instead of seconds after building an index.

- **Inconsistent Sequence Lengths in Aligned Input**: Providing incorrectly aligned FASTA files with gaps at different positions across sequences causes the pairwise distance calculation to fail or produce meaningless trees. The `--validate` flag catches this but is not enabled by default.

- **Ignoring the `--method` Flag on Divergent Sequences**: Using the default Neighbor-Joining method on highly divergent or biased sequence collections (e.g., PCR amplicons with primer bias) often produces unreliable topologies. The Maximum Likelihood method handles such cases better but requires significantly more compute time.

- **Confusing Output Format Extensions**: Specifying `--output-format nexus` but naming the output file with a `.nwk` extension causes confusion downstream when downstream tools reject the file. Always match the file extension to the declared format or use the `--auto-extend` flag to let the tool manage extensions.

- **Omitting Branch Length Units**: The Newick output does not include units by default. Downstream tools that parse branch lengths may assume substitution rate units or time units depending on context. Use `--length-unit subst` or `--length-unit time` to explicitly annotate the output.

## Examples

### Build a reference index from a FASTA file
**Args:** `build --input reference_seqs.fasta --output reference.astrid --method nj`
**Explanation:** The build subcommand creates a binary index file for rapid access by the main tool, using Neighbor-Joining for initial distance calculations.

### Construct a phylogenetic tree from aligned sequences
**Args:** `--input aligned_sequences.fasta --output tree.nwk --method nj --validate`
**Explanation:** Reads pre-aligned FASTA input and constructs a Neighbor-Joining tree with validation enabled to detect alignment inconsistencies.

### Generate a tree using UPGMA with JSON output
**Args:** `--input sequences.fasta --output tree.json --method upgma --output-format json --length-unit time`
**Explanation:** Uses UPGMA clustering and outputs in JSON format with explicit time units for branch lengths, suitable for downstream visualization tools.

### Process sequences with a pre-built database index
**Args:** `--input query_seqs.fasta --database reference.astrid --output query_tree.nwk --threads 8`
**Explanation:** Queries against the previously built index for accelerated processing, utilizing 8 parallel threads for improved performance on large datasets.

### Create a tree with Maximum Likelihood and custom substitution model
**Args:** `--input genes.fasta --output ml_tree.nwk --method ml --model LG --length-unit subst`
**Explanation:** Runs computationally intensive Maximum Likelihood estimation with the LG substitution matrix to produce statistically robust trees on diverse gene families.