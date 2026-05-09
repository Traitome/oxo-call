---
name: cladebreaker
category: Phylogenetics and Recombination Detection
description: CladeBreaker is a tool for breaking recombinant clades in phylogenetic trees by identifying and splitting sequences that contain recombination signals. It takes as input a multiple sequence alignment and an optional phylogenetic tree, detects recombination breakpoints using statistical methods, and outputs partitioned sequence sets or modified alignments suitable for downstream phylogenetic analysis without recombination artifacts.
tags:
  - recombination-detection
  - phylogenetics
  - sequence-analysis
  - sequence-partitioning
  - rdp-suite
  - viral-phylogeny
  - hcv-phylogeny
  - sequence-alignment
  - breakpoint-analysis
  - clade-resolution
  - recombinant-sequence
  - phylogeny-reconstruction
  - sequence-filtering
author: AI-generated
source_url: https://web.archive.org/web/2024/cladebreaker
---

## Concepts

- **Input requires both an alignment and a tree**: CladeBreaker expects a multiple sequence alignment (FASTA, NEXUS, or Clustal format) paired with a phylogenetic tree (NEWICK format). Without both, recombination breakpoint detection produces no meaningful output. The alignment drives breakpoint identification while the tree provides the phylogenetic context for clade structure.
- **Output produces partitioned sequence sets**: After detecting recombination breakpoints, CladeBreaker writes one or more sequence subsets as separate alignment files, along with an optional breakpoint report listing genomic coordinates of each split. Downstream tools (e.g., RAxML, IQ-TREE, FastTree) can then independently reconstruct recombination-free trees per partition.
- **Threshold parameters control breakpoint sensitivity**: The `--min-confidence` (or `-c`) flag sets the statistical confidence cutoff (0.0–1.0) below which detected breakpoints are discarded. Lower values increase recall but introduce false positives; higher values improve precision but may miss genuine recombination events. Typical infectious disease datasets use values between 0.6 and 0.9.
- **Sequence overlap is handled through masking**: When a sequence spans multiple recombination-derived clades, CladeBreaker either masks the recombinant region in both subsets (using `N` or `-` characters) or assigns the sequence to the majority clade. This prevents double-counting but requires verification that masked positions are not biologically informative.
- **Clade-breaking affects downstream tree topology**: Removing or reassigning recombinant sequences changes clade membership, which alters bootstrap support values and may change the inferred evolutionary tree. Always compare bootstrap values between the original and partitioned trees to assess whether recombination was a dominant topological influence.

## Pitfalls

- **Omitting the phylogenetic tree input causes runtime failure**: CladeBreaker relies on the tree to establish baseline clade relationships. Providing only an alignment without the corresponding NEWICK tree causes the tool to abort with an error or fall back to a distance-based method, producing unreliable breakpoints. Always supply the tree with `--tree-file` or `-t`.
- **Running with default confidence thresholds on noisy data produces fragmented results**: High-throughput amplicon data (e.g., viral NGS reads) often contain sequencing errors that CladeBreaker can misidentify as recombination breakpoints. Default confidence thresholds may be too permissive, fragmenting a single true clade into dozens of micro-clades. Reduce `--min-confidence` incrementally and visually inspect breakpoints before accepting output.
- **Forgetting that outgroup sequences are treated as recombinant by default**: If your alignment contains an outgroup sequence used for rooting, CladeBreaker may flag it as a recombinant because it is phylogenetically distant from the ingroup. Removing outgroup sequences before running CladeBreaker and re-rooting the tree afterward is the correct workflow.
- **Assuming partitioned sequences are recombination-free after a single run is incorrect**: Some recombination events span multiple breakpoints distributed across a genome (e.g., HCV, HIV-1). One pass of CladeBreaker resolves only the primary breakpoint. For segmented or multi-partite genomes, iterative runs using the partitioned output from each previous run are necessary to fully resolve recombinant histories.
- **Output format mismatches downstream tool expectations**: CladeBreaker can export partitioned alignments in several formats, but not all downstream phylogenetic tools accept all formats. RAxML, for example, requires PHYLIP format for large alignments; specifying FASTA output may silently fail downstream. Always verify that the output format matches your phylogenetic reconstruction tool's input requirements.

## Examples

### Detect recombination breakpoints in a viral HCV alignment with default settings

**Args:** `-a input_HCV_aligned.fasta -t input_HCV_tree.newick --output-dir hc_v_partitioned`
**Explanation:** Providing both the alignment and the tree ensures CladeBreaker has all required inputs to detect recombinant clades and assign breakpoints using default confidence thresholds, writing output files to the specified directory.

### Run breakpoint detection with a high confidence threshold to reduce false positives

**Args:** `-a hbv_alignment.fasta -t hbv_tree.newick -c 0.85 --output-dir hbv_strict --report-breakpoints breakpoints.csv`
**Explanation:** Setting `--min-confidence` to 0.85 discards weak breakpoints that are likely sequencing artifacts or phylogenetic noise, producing a cleaner breakpoint report for review.

### Output partitioned alignments in PHYLIP format for RAxML compatibility

**Args:** `-a dengue_ref.fasta -t dengue_ref.newick -c 0.70 --output-format phylip --output-dir dengue_partitions`
**Explanation:** Specifying PHYLIP as the output format ensures the partitioned alignment files are directly usable as input to RAxML without manual format conversion.

### Iteratively resolve multi-breakpoint recombination in an HIV-1 alignment across three passes

**Args:** `-a hiv1_run1.fasta -t hiv1_run1.newick -c 0.75 --output-dir hiv1_pass2`
**Explanation:** Running CladeBreaker a second time on the first-pass partitioned alignment detects secondary breakpoints that the first pass did not resolve, which is typical for HIV-1 recombinant genomes with complex recombination histories.

### Mask recombinant regions rather than discarding sequences to preserve alignment length

**Args:** `-a zika_alignment.fasta -t zika_tree.newick -c 0.80 --mask-mode yes --output-dir zika_masked`
**Explanation:** Enabling `--mask-mode` replaces recombinant nucleotide positions with `N` characters instead of removing the sequence entirely, preserving alignment length and enabling consistent site-wise phylogenetic comparisons across the dataset.