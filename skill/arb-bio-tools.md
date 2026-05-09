---
name: ARB Bioinformatics Suite
category: Bioinformatics / Sequence Alignment & Phylogenetics
description: ARB (ARBitrator) is a graphically-oriented software package for storing, retrieving, filtering, aligning, and analyzing rRNA and other nucleic acid sequence data. This skill covers the core command-line utilities including arb-merker (alignment editor), arb-bio-tools (sequence merge/filter), arb_db_compile (ADB database compiler), arb-load-db (sequence importer), arb-load-tax (taxonomy importer), arb_conf_check (configuration validator), and arb_ntree (neighbor-joining tree builder). ARB databases store aligned sequences in a custom ADB format with per-position information (base, structure annotation, consensus flag, compatibility mask).
tags:
  - rRNA
  - alignment
  - phylogenetic
  - sequence-database
  - arb
  - adb
  - ssu-rrna
  - lsu-rrna
author: AI-generated
source_url: https://github.com/ARB-FAQ/arb-bio-tools
---

## Concepts

- **ADB database format**: ARB stores aligned sequences in a custom `.adb` database. Each database includes a master pointer table, per-sequence entries (name, accession, aligned sequence, taxonomic path), per-position columns (base, secondary-structure annotation, consensus-call flag, compatibility mask), and a PT Server (propagation tracking server) for synchronization. Never edit `.adb` files manually; always use the provided utilities (`arb_db_compile`, `arb-load-db`).
- **Sequence alignment via arb-merker**: The `arb-merker` tool performs semi-automated insertion of new sequences into an existing structural alignment. It uses a hidden Markov model (HCM) built from the reference database's consensus secondary structure to guide insertions and deletions. The tool requires a structurally aligned reference database, not a de novo alignment, and outputs a temporary `.aln` (aligned FASTA) file that can be imported into the ARB database.
- **arb-bio-tools operations**: The merge tool (`arb-bio-tools`) supports three operations controlled by `-m insert` (add new sequences), `-m replace` (overwrite existing sequences with the same key), and `-m del` (remove entries). Input is read from a SAM/BAM file via `-i`, and the target ARB database is specified with `-d`. Sequences are matched to database entries using a configured key field (e.g., accession number or sequence MD5) unless `--trust-seeds` is used for alignment-based seeding.
- **arb_ntree distance methods and outgroup selection**: `arb_ntree` constructs neighbor-joining trees from aligned sequence data using evolutionary distance models (Kimura 2-parameter, Jukes-Cantor, or F84). The `-t` flag selects the outgroup taxon(s) by name or wildcard pattern, and `-s` specifies the minimum similarity filter to exclude highly dissimilar sequences from tree computation. Outgroups must exist in the alignment or the tree will be silently rooted incorrectly.
- **Taxonomy importer**: `arb-load-tax` accepts tab-delimited or Newick-format taxonomy files. Each sequence entry in the ARB database is matched to a taxonomy node via the configured key field, and a new taxonomy tree is built or merged with the existing one. Conflicts (unresolved nodes) are logged to stderr unless `--force` is specified, and conflicting merges will overwrite existing taxonomy assignments silently.

## Pitfalls

- **Using unaligned sequences with arb-merker**: Passing raw (unaligned) sequences to `arb-merker` without the `--ref-structure` flag causes the HCM to use a generic insertion penalty matrix, producing poor alignments with excessive indels. The output will still merge into the database but with corrupted positional data, corrupting the per-position consensus and compatibility mask for all affected columns.
- **Trusting arb-bio-tools merge without validation**: Merging with `--trust-seeds` bypasses the normal MD5/key verification step and uses alignment覆盖率 as the sole matching criterion. If the reference database contains similar sequences (e.g., within-species variants), `arb-bio-tools` may silently merge into the wrong entry, destroying the original sequence. Always run `arb_conf_check` on the merged database before production use.
- **Forgetting the PT Server state after edits**: After any edit via `arb-merker` or `arb-bio-tools`, the PT Server (propagation tracking) marks modified sequences as dirty. If the database is used in the GUI without running `arb_db_compile --refresh-pt` first, the GUI will show stale positional data and may crash when accessing modified columns. Always run the PT refresh step after batch merges.
- **Outgroup taxon mismatch with arb_ntree**: Specifying an outgroup with `-t "E.coli"` when the actual database entry is labeled `E._coli` (with underscore encoding) produces no match. `arb_ntree` silently uses the first sequence in the alignment as the outgroup instead, inverting the tree root and invalidating all downstream phylogenetic analysis. Print the taxonomy table with `--list-tax` before running tree building.
- **SAM/BAM sorting incompatibility**: `arb-bio-tools merge` requires queryname-sorted SAM/BAM input. Reading a coordinatesorted BAM file produces incorrect pairing flags and can cause the merge tool to skip entries entirely. Always sort SAM/BAM files with `samtools sort -n` before passing them to `arb-bio-tools`.

## Examples

### Create an ARB database from a FASTA alignment and reference taxonomy

**Args:** `arb_db_compile --input-align example_16s_aligned.fasta --output-db example.adb --taxonomy-ref taxonomy.tsv --format auto --log compile.log`
**Explanation:** The `arb_db_compile` tool reads the aligned FASTA, builds the per-position columns using the `--format auto` detection, and imports taxonomy paths from the tab-delimited reference, creating a production-ready `.adb` file with a single log of all imported entries.

### Semi-automatically align new 16S sequences using the structural HCM

**Args:** `arb-merker --input-unaligned new_sequences.fasta --ref-db reference.adb --ref-structure rRNA_HCM.mat --output-temp aligned_output.fasta --log merker.log`
**Explanation:** `arb-merker` uses the structural HCM matrix to insert the new sequences into the reference alignment, preserving secondary-structure helices, and outputs aligned sequences to a temporary file for subsequent import without modifying the reference database.

### Merge SAM alignments into an ARB database, adding new sequences

**Args:** `arb-bio-tools merge --input alignments.sam --db target.adb --output target_merged.adb --operation insert --log merge.log`
**Explanation:** `arb-bio-tools` reads the queryname-sorted SAM file, matches reads to database entries by MD5 key, inserts unmatched reads as new entries, and writes the updated database to a new file while logging all merge decisions.

### Build a neighbor-joining tree from an aligned database with Kimura 2-parameter distance

**Args:** `arb_ntree --db aligned.adb --outgroup "Methanobrevibacter" --distance kimura2 --min-similarity 0.75 --output-tree tree.nwk --log tree.log`
**Explanation:** `arb_ntree` computes pairwise Kimura 2-parameter distances from the aligned database, filters sequences below 75% similarity, roots the tree with the named outgroup, and exports the tree in Newick format for use in phylogenetic viewers.

### Import sequences into an existing ARB database from FASTA with taxonomy assignment

**Args:** `arb-load-db --input new_seqs.fasta --db existing.adb --key-field accession --taxonomy-uri taxonomy.new.tsv --mode merge --log load.log`
**Explanation:** `arb-load-db` reads new sequences from FASTA, matches entries to the database using the accession field, merges them with existing entries on match or inserts them as new entries, and assigns taxonomy paths from the provided taxonomy file.

### Validate a configuration before database operations

**Args:** `arb_conf_check --db candidate.adb --check-integrity --check-pt-server --output-report validation_report.txt`
**Explanation:** `arb_conf_check` scans the database for structural inconsistencies (broken pointers, corrupted columns), verifies the PT Server state, and writes a detailed report so that database corruption is caught before any merge or alignment operation.

### Import taxonomy tree from a Newick file into an existing database

**Args:** `arb-load-tax --input taxonomy.nwk --db existing.adb --key-field accession --mode merge --force --log taxonomy_load.log`
**Explanation:** `arb-load-tax` reads the Newick taxonomy tree, matches each node to database entries by accession, merges the tree with the existing taxonomy, and overwrites conflicting assignments with the `--force` flag, logging all operations.