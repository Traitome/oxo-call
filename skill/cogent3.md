---
name: cogent3
category: Bioinformatics / Sequence Analysis
description: A Python library and CLI toolkit for comparative genomics, evolutionary analysis, and sequence manipulation. Supports alignment, phylogenetics, tree visualization, and I/O for common biological formats including FASTA, Newick, Nexus, Phylip, and Stockholm.
tags:
  - sequence-analysis
  - phylogenetics
  - alignment
  - genomics
  - Python
  - CLI
  - I/O
author: AI-generated
source_url: https://cogent3.org
---

## Concepts

- **Aligned or unaligned sequence collections**: Cogent3 loads sequences into `SeqView` and `Aligned` objects. When the `--aligned` flag is provided, sequences are forced into a fixed-width alignment matrix where gaps are represented with `-` characters; without it, sequences are treated as plain collections. This distinction affects downstream operations like evolutionary model fitting and distance calculations.
- **Multiple I/O format support**: Cogent3 auto-detects format from file extension or accepts explicit `--format` specifiers (`fasta`, `newick`, `nexus`, `phylip`, `stockholm`). For Newick trees, loading with `load` produces a `PhyloNode` tree object; for alignments, the output is an `Aligned` object. Using the wrong `--format` value silently corrupts data or raises a confusing `ValueError`.
- **Tree construction and node annotation**: After loading a Newick tree with `load`, nodes are accessible as `PhyloNode` objects with attributes `name`, `length`, and `children`. Adding named features to nodes requires passing `--with-features` at load time or calling `annotate()` on the tree object; features are then available as a dictionary via the `.info` attribute on each node.
- **Visualization pipeline**: The `plot` subcommand accepts `--output-format` (`png`, `pdf`, `svg`) and `--width` / `--height` for canvas sizing. When plotting trees, tip labels and branch lengths can be customized via `--tip-label` and `--branch-lengths` flags; omitting these renders a minimal unannotated tree that may be unreadable in publications.
- **Align subcommand and gap handling**: The `align` subcommand supports multiple algorithms controlled by `--method` (`clustal`, `muscle`, `fast_protein`). When aligning, `--gap-open` and `--gap-extend` penalty values are passed directly to the underlying aligner; mismatched penalty units cause alignment artifacts such as over-gapped or under-gapped regions.

## Pitfalls

- **Loading a Newick file without specifying `--format newick`**: Cogent3 does not reliably auto-detect Newick format from `.tree` or `.nw` extensions. If `--format` is omitted, the file is loaded as a plain sequence collection, producing a `SeqCollection` object instead of a `PhyloNode` tree. Downstream tree-specific methods like `ladderize()` or `get_tips()` then fail with `AttributeError: 'SeqCollection' object has no attribute 'ladderize'`.
- **Applying evolutionary model fitting to unaligned sequences**: Model fitting commands (e.g., `cogent3 fit`) assume a fixed-width `Aligned` object with equal-length sequences. Running it on an unaligned `SeqCollection` silently produces nonsense distance values or crashes with a shape-mismatch error. Always verify alignment status with `cogent3 info` before fitting models.
- **Using `--gap-open` or `--gap-extend` with integer values on `--method muscle`**: The Muscle aligner interprets gap penalties in its own internal units, which differ from the standard Cogent3 convention. Supplying integer penalties designed for `clustal` causes Muscle to produce either excessively gapped or insufficiently gapped alignments. Use float values (e.g., `2.9` instead of `3`) when specifying penalties for Muscle.
- **Plotting large trees without `--max-width`**: Trees with more than ~200 tips render overlapping labels when `--width` is not set. The default canvas size clips tips beyond the viewport boundary, producing a publication figure that silently omits taxa. Always set an appropriate `--width` value for trees with many terminals.
- **Overwriting input files when using `convert` without `--output`**: The `convert` subcommand writes back to the input file path when `--output` is not provided, destroying the original alignment or tree data. Cogent3 does not create automatic backups. The original file is permanently replaced with the reformatted data.

## Examples

### Load a Newick tree from a `.tree` file
**Args:** `load --format newick --input /path/to/tree.newick --info`
**Explanation:** The `--format newick` flag is required because Cogent3 does not auto-detect Newick from non-standard extensions, and `--info` confirms the tree topology before any downstream analysis.

### Load a FASTA alignment with alignment enforcement
**Args:** `load --aligned --input /path/to/sequences.fasta --format fasta`
**Explanation:** The `--aligned` flag forces Cogent3 to interpret the loaded sequences as fixed-width rows in an alignment matrix, inserting gap characters where sequences differ in length.

### Align two unaligned FASTA files using Muscle with custom gap penalties
**Args:** `align --method muscle --input /path/to/unaligned.fasta --gap-open 2.9 --gap-extend 0.5 --output /path/to/aligned.fasta`
**Explanation:** Specifying `--gap-open 2.9` as a float value ensures correct unit handling for Muscle, and `--output` prevents accidental in-place overwrite of the input file.

### Plot a phylogenetic tree as a PDF with tip labels and branch lengths
**Args:** `plot --input /path/to/tree.newick --format newick --output tree_plot.pdf --tip-label name --branch-lengths length --width 800`
**Explanation:** Setting `--tip-label name` displays species or taxa labels at the tips, `--branch-lengths length` annotates each branch with its evolutionary distance, and `--width 800` provides sufficient canvas space to prevent label overlap.

### Get information summary about an alignment
**Args:** `info --input /path/to/alignment.fasta`
**Explanation:** The `info` subcommand reports sequence count, alignment length, gaps per column, and motif frequencies without modifying the input, making it safe for exploratory analysis.

### Convert a Nexus alignment to FASTA format
**Args:** `convert --input /path/to/alignment.nex --format nexus --output /path/to/alignment.fasta`
**Explanation:** The `--format nexus` flag tells Cogent3 the input format, and specifying `--output` with a `.fasta` extension ensures the output format is inferred correctly, avoiding the default in-place overwrite behavior.

### Fit an evolutionary model to an alignment for distance estimation
**Args:** `fit --input /path/to/alignment.fasta --model LG --test --output /path/to/model_summary.txt`
**Explanation:** The `--model LG` flag selects the LG (Le-Gascuel) amino acid substitution matrix, and `--test` performs a bootstrap approximation to validate model fit before committing to a full analysis.

### Load a Stockholm-formatted alignment with feature annotations
**Args:** `load --format stockholm --input /path/to/align.sto --with-features --info`
**Explanation:** Loading Stockholm files with `--format stockholm` is required because Cogent3 cannot auto-detect this format, and `--with-features` preserves sequence-level annotations (such as consensus columns and structure annotations) in the resulting `Aligned` object.