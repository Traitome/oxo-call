---
name: chromosomer
category: Genome Assembly / Scaffolding Tool
description: A reference-guided genome scaffolding tool that orders and orients draft contigs using whole-genome alignments to improve genome assemblies.
tags: [scaffolding, genome-assembly, reference-guided, synteny, contigs]
author: AI-generated
source_url: https://github.com/mk僵蚕/chromosomer
---

## Concepts

- Chromosomer reconstructs scaffolds from a draft assembly by aligning contigs against a reference genome and selecting only those alignment blocks that pass user-defined quality thresholds (minimum alignment length and identity).
- The tool generates AGP (American Golden Path) files describing how contigs are ordered and oriented within each scaffold, enabling downstream processing by genome browsers and annotation pipelines.
- Input sequences must be in FASTA format; the reference genome should be a closely related species to ensure sufficient alignment coverage for accurate scaffolding.
- Chromosomer supports iterative scaffolding, allowing users to adjust thresholds and re-run the process to progressively improve assembly quality.
- Output consists of improved scaffold sequences (FASTA) paired with corresponding AGP files that record the exact coordinate mapping between the draft and scaffolded assembly.

## Pitfalls

- Using a reference genome that is too diverged results in poor alignment coverage, causing many contigs to remain unscaffolded or placed incorrectly, degrading overall assembly quality.
- Setting the minimum alignment length threshold too high filters out legitimate short but accurate alignment blocks, fragmenting the scaffold output unnecessarily.
- Specifying a minimum identity threshold that is too low permits misaligned contig placements, which can introduce structural errors into the improved assembly.
- Providing a draft assembly with ambiguous bases (N characters) scattered throughout may cause alignment failures or incorrect orientation assignments.
- Failing to validate the output AGP file after scaffolding produces assemblies where contig orientation may not match strand expectations in downstream tools.

## Examples

### Scaffold a fragmented draft assembly using a reference genome with default thresholds

**Args:** `scaffold --draft draft_contigs.fasta --reference ref_genome.fasta --output scaffolds/`
**Explanation:** This runs chromosomer with default alignment filtering thresholds (typically 500 bp minimum length and 90% identity) to order and orient contigs based on their alignment to the reference genome.

### Scaffold with custom minimum alignment length threshold of 1000 bp

**Args:** `scaffold --draft draft_contigs.fasta --reference ref_genome.fasta --min-align-len 1000 --output scaffolds_custom/`
**Explanation:** Increasing the minimum alignment length to 1000 bp excludes shorter alignment blocks that may represent repetitive or low-confidence regions, producing more conservative scaffold connections.

### Scaffold with reduced minimum identity threshold of 80% to retain divergent alignments

**Args:** `scaffold --draft draft_contigs.fasta --reference ref_genome.fasta --min-identity 0.80 --output scaffolds_divergent/`
**Explanation:** Lowering the identity threshold to 80% allows chromosomer to retain alignment blocks from more diverged regions, useful when the draft and reference have accumulated sufficient species-specific mutations.

### Export scaffold sequences from the generated AGP file

**Args:** `scaffold --draft draft_contigs.fasta --reference ref_genome.fasta --output scaffolds/ --export`
**Explanation:** The export flag instructs chromosomer to not only generate the AGP file but also produce the actual scaffold sequences by concatenating the ordered and oriented contigs into chromosome-level sequences.

### Perform iterative scaffolding with progressively relaxed thresholds

**Args:** `scaffold --draft scaffolds_iter1.fasta --reference ref_genome.fasta --min-align-len 500 --min-identity 0.85 --output scaffolds_iter2/`
**Explanation:** Running a second scaffolding iteration on output from the first round with relaxed thresholds (500 bp, 85% identity) allows additional contig placements that were filtered during the initial round.