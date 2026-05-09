---
name: abacas
category: Genome assembly / Contig ordering
description: |
  ABACAS (Approximate BAsed Coverage for AssemblyS) orders and orients fragmented assembly contigs against a reference genome to produce a pseudo-contiguous sequence. It uses NUCmer (MUMmer) for mapping contigs to the reference, then reconstructs an ordered assembly based on reference positions.
tags:
  - assembly
  - scaffolding
  - contig ordering
  - genome assembly
  - reference-guided assembly
  - MUMmer
  - nucmer
author: AI-generated
source_url: https://sourceforge.net/projects/abacas/
---

## Concepts

- ABACAS takes two FASTA inputs: a query assembly (contigs) and a reference genome. It uses the reference coordinates to order all query contigs, producing a pseudo-contig that concatenates ordered contigs with N-padded gaps where reference segments lack query coverage.
- The tool internally runs NUCmer (part of MUMmer) to map contigs to the reference. Contigs must have sufficient alignment coverage (default ≥60%) and minimum length (default ≥100bp) to be included in the final ordering; others are excluded from the pseudo-contig.
- ABACAS produces multiple outputs: an ordered FASTA file (`-ordered.fasta`), a runlog (`-runlog.txt`) with statistics, and optionally a visual block representation (`-block.txt`). The pseudo-contig includes lowercase 'n' characters for gaps between ordered contigs.
- The ordering is deterministic when contigs align uniquely, but ambiguous placements (multiple equal-scoring positions) cause warning messages in the runlog; manual inspection may be required for complex rearrangements.
- ABACAS can be run in "order-only" mode by skipping pseudo-contig generation, useful when only the reordered FASTA is needed for downstream tools like Mauve.

## Pitfalls

- **Specifying the wrong reference direction**: ABACAS assumes the reference is in the correct orientation. If the reference FASTA contains reverse-complemented sequences, the ordering will be wrong, producing inverted blocks that don't match the expected genome structure.
- **Using unmapped contigs without checking**: If most contigs fail to map (due to high divergence or low sequence quality), ABACAS still produces output but with many gaps; ignoring the runlog means missing critical warnings about low coverage.
- **Inconsistent sequence naming**: If the query FASTA uses non-unique or spaces-containing headers, downstream tools parsing ABACAS output may fail; headers should be simple (e.g., >contig_001) without special characters.
- **Assuming contiguity without gaps**: Users expecting a fully closed sequence may misinterpret the N-padded pseudo-contig as a finished assembly; ABACAS explicitly generates gaps, and the true coverage fraction is reported in the runlog.
- **Mismatched NUCmer parameters**: ABACAS calls NUCmer with default settings; very short contigs or highly diverged sequences may need explicit `-c` (minimum coverage) or `-l` (minimum alignment length) flags passed through ABACAS to NUCmer, otherwise many valid alignments are missed.

## Examples

### Basic contig ordering against a reference genome

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix`

**Explanation:** The core ABACAS invocation maps all query contigs to the reference via NUCmer, orders them by reference coordinate, and outputs an ordered FASTA file with gap-padded pseudo-contig.

### Using a custom minimum alignment coverage threshold

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix -c 80`

**Explanation:** Increases the minimum alignment coverage required for a contig to be included from default 60% to 80%, filtering out more fragmented or ambiguous alignments.

### Skipping pseudo-contig generation (order-only mode)

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix -o`

**Explanation:** Orders contigs but stops before generating the concatenated pseudo-contig, outputting only the ordered FASTA file; useful when downstream tools expect individual ordered sequences.

### Using a custom minimum contig length filter

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix -l 200`

**Explanation:** Sets the minimum contig length to 200bp, excluding shorter fragments from both mapping and the final ordered output to reduce noise in the assembly.

### Generating a visual block representation file

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix -b`

**Explanation:** Produces an additional block-level text file showing the correspondence between reference regions and ordered contigs, useful for inspecting rearrangement patterns.

### Running ABACAS without creating gaps in the pseudo-contig

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix -ng`

**Explanation:** Outputs a pseudo-contig without N-gap padding between ordered contigs, concatenating them directly; note that this loses positional information about unrepresented regions.

### Using explicit minimum exact match length for NUCmer

**Args:** `-r reference.fasta -q query_contigs.fasta -p output_prefix -n 25`

**Explanation:** Passes the `-l 25` flag to NUCmer, setting minimum exact match length to 25bp; lower values increase sensitivity but may cause spurious alignments in repetitive regions.

---