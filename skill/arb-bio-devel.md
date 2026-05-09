---
name: arb-bio-devel
category: sequence-analysis
description: A graphical nucleotide sequence alignment editor and analysis tool from the ARB (ARBitrary) suite. Used for interactive editing, visualization, and validation of DNA/RNA multiple sequence alignments with support for phylogenetic tree construction and sequence database management.
tags: [alignment, sequence-editing, phylogenetics, arb-suite, bioinformatics]
author: AI-generated
source_url: https://arb.online/
---

## Concepts

- ARB uses a hierarchical database structure where sequences are organized in a tree format, with species linked to aligned sequence data and secondary structure information that persists across editing sessions.
- The tool reads common alignment formats including FASTA, PHYLIP, NEXUS, CLUSTAL, and Stockholm, and exports edited alignments in multiple formats suitable for downstream phylogenetic analysis.
- ARB employs a real-time consistency-checking system that validates base-pairing in RNA alignments against predefined covariance models, making it particularly valuable for ribosomal and catalytic RNA analysis.

## Pitfalls

- Opening very large alignment databases (>10,000 sequences) without pre-filtering can cause significant memory consumption and UI lag, as the entire database is loaded into RAM at startup.
- Automatic alignment algorithms integrated within ARB may introduce errors that are difficult to spot visually, especially in variable regions with frequent indels; manual curation is strongly recommended for publication-quality alignments.
- Saving an alignment overwrites the original database file without creating an automatic backup, risking permanent data loss if the edit was incorrect or unintended.

## Examples

### Open an alignment database and view sequences visually
**Args:** /path/to/alignment.adb
**Explanation:** This opens the binary ARB database format directly, allowing full interactive editing and tree navigation within the graphical interface.

### Import a FASTA multiple sequence alignment into ARB format
**Args:** --import=FASTA --in=sequences.fasta --out=arb_database.adb
**Explanation:** Converts a standard FASTA alignment file into the native ARB binary database format for use with the full editor suite.

### Export an edited alignment to PHYLIP format for phylogenetics software
**Args:** --export=PHYLIP --in=arb_database.adb --out=edited.phy
**Explanation:** Writes the currently loaded alignment back to PHYLIP format, which is compatible with most phylogenetic reconstruction programs like RAxML or PAUP.

### Run a quick consistency check on RNA secondary structure
**Args:** --check-rna-structure --db=srrnadb.adb
**Explanation:** Validates base-pairing consistency against covariance models in the loaded RNA alignment database, reporting positions with suspicious substitution patterns.

### Generate a neighbor-joining tree from the current alignment
**Args:** --nj-tree --distance=jukes-cantor --out=tree.nh
**Explanation:** Computes a neighbor-joining phylogenetic tree using the Jukes-Cantor nucleotide substitution model and outputs the result in New Hampshire format.

### Filter sequences by species and save subset
**Args:** --filter=species:Enterococcus --in=full_db.adb --out=subset.adb
**Explanation:** Extracts all sequences belonging to the specified genus or species name and saves them to a new, smaller database file for focused analysis.