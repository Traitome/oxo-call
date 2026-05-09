---
name: cassiopeia
category: Phylogenetics / Venom Peptide Analysis
description: Cassiopeia is a maximum-parsimony phylogenetics toolkit for analyzing hybrid venom gland peptide sequences. It reconstructs phylogenetic trees from character-state matrices derived from venom toxin sequences, with support for preprocessing, parsimony scoring, ancestral state reconstruction, and statistical reporting. Companion scripts handle sequence grouping, character processing, and workflow automation.
tags:
  - phylogenetics
  - maximum-parsimony
  - venom-peptides
  - character-states
  - newick
  - hybrid-venoms
author: AI-generated
source_url: https://github.com/cassiopeia-project/cassiopeia
---

## Concepts

- **Character-State Matrix Format**: Cassiopeia operates on a character matrix where each character represents a residue position and states are assigned from aligned peptide sequences (e.g., "A", "C", "G" for alanine, cysteine, glycine). The matrix is typically tab-delimited or CSV with sequence names as rows and character positions as columns. Understanding that missing or ambiguous characters (e.g., "?", "-") are treated as polymorphic and can affect parsimony scoring is critical for accurate tree reconstruction.

- **Maximum Parsimony Scoring**: Cassiopeia computes tree scores as the minimum number of state changes required across all characters on the tree. Lower scores indicate more parsimonious trees. When multiple equally parsimonious trees exist, Cassiopeia can report them all or collapse near-optimal branches. The `--score` subcommand evaluates a fixed topology, while `cassiopeia-build` searches the tree space to find parsimonious topologies.

- **Newick Tree Output**: All reconstructed trees are output in standard Newick format, with optional support for extended Newick annotations (branch lengths, support values). The output can be piped directly to visualization tools such as FigTree, DendroPy, or `ete3 view`. Newick tip labels must exactly match sequence names in the character matrix for downstream analyses to succeed.

- **Workflow Orchestration**: The `cassiopeia-complete` command orchestrates a full analysis pipeline: sequence grouping → character assignment → parsimony search → scoring → report generation. This is the recommended entry point for users who want a single command to go from raw FASTA sequences to a finished report. Individual steps can be overridden with dedicated companion scripts when fine-grained control is needed.

- **Companion Binaries**: Cassiopeia ships with several standalone binaries: `cassiopeia-group-seqs` (clusters input sequences into groups), `cassiopeia-process-characters` (converts aligned sequences to character matrices), `cassiopeia-build` (parsimony tree search), `cassiopeia-regress` (statistical regression on character data), and `cassiopeia-preprocess` (prepares raw FASTA for downstream steps). These binaries accept independent arguments and are invoked directly from the shell.

## Pitfalls

- **Tip Label Mismatch Between Matrix and Newick**: If sequence names in the character matrix do not exactly match tip labels in the Newick tree output, downstream operations (e.g., scoring, ancestral reconstruction) will silently skip those taxa or raise cryptic index errors. Always verify label一致性 before running multi-step pipelines.

- **Missing Characters Treated as Polymorphic by Default**: Characters marked with "?" or "-" are treated as wildcard states (matching any state) during parsimony optimization. This is mathematically correct but can artificially reduce tree scores and produce spuriously short trees when the data genuinely has gaps or missing regions. Use the `--no-missing-wildcard` flag with `cassiopeia-process-characters` to instead treat these as excluded characters.

- **Overwriting Output Files Without Confirmation**: Many Cassiopeia subcommands write output files with fixed names (e.g., `character_matrix.csv`, `tree.newick`) without a `--force` flag prompt. If you re-run a pipeline in the same directory, previous output files are silently overwritten, destroying intermediate results. Always use dedicated output subdirectories for each run.

- **Insufficient Iterations for Complex Character Matrices**: When the character matrix contains many polymorphic sites or large numbers of taxa, the parsimony search may terminate prematurely if the iteration limit (`--max-iterations`) is too low, producing non-parsimonious or partial trees. Increase `--max-iterations` to at least 5000 for matrices with more than 100 taxa or 500+ characters.

- **Inconsistent Grouping Before Tree Building**: Running `cassiopeia-group-seqs` with lenient similarity thresholds can merge genuinely distinct sequences into the same group, forcing them to share ancestral states and distorting the final tree topology. Always inspect group assignments visually or with `cassiopeia-report` before proceeding to the build step.

## Examples

### Generate a character-state matrix from aligned FASTA sequences
**Args:** `process-characters --input aligned_peptides.fasta --output character_matrix.csv --separator comma`
**Explanation:** The `process-characters` script converts aligned FASTA sequences into a tab/comma-delimited character matrix where each column corresponds to a residue position and cell values are single-character states, which is the required input format for all Cassiopeia parsimony routines.

### Perform a parsimony tree search on a character matrix
**Args:** `build --character-matrix character_matrix.csv --output tree.newick --algorithm ratchet --iterations 5000 --random-seed 42`
**Explanation:** The `build` command runs a maximum-parsimony heuristic search using the Ratchet algorithm with 5000 iterations and a fixed random seed for reproducibility, outputting a Newick tree file representing the most parsimonious topology.

### Score an existing phylogenetic tree against a character matrix
**Args:** `score --tree tree.newick --character-matrix character_matrix.csv --report scores.txt --include-branch-lengths`
**Explanation:** The `score` subcommand evaluates a provided Newick tree by computing the minimum number of state changes per character, writing a detailed score report and optionally annotating branch lengths onto the tree.

### Group venom peptide sequences into clusters before phylogenetic analysis
**Args:** `group-seqs --input raw_venom_sequences.fasta --output groups.csv --method cluster --threshold 0.85 --linkage complete`
**Explanation:** The `group-seqs` companion script clusters input sequences using hierarchical clustering with a defined similarity threshold, producing a group assignment file that can be used to constrain or weight the parsimony search.

### Run a complete analysis pipeline from raw FASTA to final report
**Args:** `complete --input raw_venom_sequences.fasta --reference reference_alignment.fasta --output-dir results/ --workflow hybrid_venom --report-format html`
**Explanation:** The `complete` command orchestrates the full pipeline end-to-end: it groups sequences, assigns characters, builds a parsimony tree, scores it, and generates an HTML report in the specified output directory, making it ideal for single-command analyses.

### Perform ancestral state reconstruction on a scored tree
**Args:** `reconstruct --tree tree.newick --character-matrix character_matrix.csv --output ancestral_states.csv --method sankoff`
**Explanation:** The `reconstruct` subcommand performs ancestral state reconstruction using the Sankoff algorithm to infer the most likely character state at each internal node, outputting a CSV file with ancestral state probabilities for downstream evolutionary interpretation.

### Preprocess raw FASTA sequences for downstream character processing
**Args:** `preprocess --input raw_peptides.fasta --output preprocessed.fasta --trim-gaps --min-length 10 --remove-duplicates`
**Explanation:** The `preprocess` companion script cleans raw input by trimming terminal gaps, discarding sequences shorter than 10 residues, and removing exact duplicates, ensuring a clean and consistent input for `process-characters` and downstream steps.