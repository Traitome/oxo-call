---
name: mummer
category: alignment
description: MUMmer4 whole-genome alignment suite; nucmer, promer, dnadiff, and mummerplot for comparing large genomes at the nucleotide and protein level
tags: [mummer, nucmer, promer, dnadiff, whole-genome-alignment, comparative-genomics, snps, indels]
author: oxo-call built-in
source_url: "https://mummer4.github.io/"
---

## Concepts
- MUMmer4 provides: `nucmer` (nucleotide alignment), `promer` (6-frame translated alignment), `dnadiff` (pairwise genome comparison report), `mummer` (maximal unique matches).
- **nucmer** aligns a query FASTA against a reference FASTA; outputs a `.delta` file encoding all alignments.
- **dnadiff** wraps nucmer and generates a comprehensive comparison report (`.report`, `.snps`, `.1diff`, `.mdiff`, `.rdiff`); ideal for quickly characterising two genomes.
- `.delta` format: a binary-like text format storing all alignment positions; parsed by `show-snps`, `show-coords`, `show-aligns`, `delta-filter`.
- `show-coords` converts a `.delta` file to a human-readable coordinate table; `-r -c -l` flags add reference/query coordinates, coverage, and length.
- `show-snps` extracts SNPs and small indels from a `.delta` file; `-Clr` gives clean, concise output.
- `delta-filter` filters alignments by identity (`-i`), length (`-l`), or keeps only unique alignments (`-q`, `-r`, `-1` for 1-to-1).
- **mummerplot** generates a dot-plot (requires gnuplot); `--png` produces a PNG; `--filter` removes non-unique hits.
- MUMmer4 is a significant rewrite of MUMmer3; the API is compatible but performance and accuracy are improved.
- All MUMmer tools accept multi-FASTA inputs; for many-vs-many, consider NUCmer's `--mum` or use a loop.
- Default minimum cluster length is 65 bp (`nucmer -c 65`); lower for short sequences or highly similar genomes.
- `--mum` finds matches unique in both sequences; `--mumreference` finds matches unique in reference (default); `--maxmatch` finds all matches regardless of uniqueness.
- `show-tiling` constructs a tiling path of query contigs on reference; useful for assembly validation and scaffolding.
- `show-diff` classifies breakpoints and rearrangements from alignments; outputs `.rdiff` and `.qdiff` files.
- `--nosimplify` preserves all alignments including shadowed clusters; essential for self-alignment and repeat detection.

## Pitfalls
- not using `delta-filter -1` before `show-snps` reports duplicated SNPs from repetitive regions; always filter for 1-to-1 alignments before SNP analysis.
- `nucmer` creates a `out.delta` in the CWD by default; use `--prefix` to specify a different output base name and avoid file collisions in multi-genome runs.
- Very large genomes (>500 Mb) are slow with default parameters; increase `-l` (minimum MUM length) and `-c` (minimum cluster length) to speed up.
- Chromosome naming must be unique within each FASTA; duplicate sequence names cause incorrect alignment assignments.
- `promer` uses translated BLAST-like alignment and is slower than `nucmer`; only use it when comparing highly divergent sequences (e.g., bacteria vs distant relatives).
- `mummerplot` requires gnuplot and sometimes postscript rendering; install gnuplot via conda if it is missing.
- MUMmer3 and MUMmer4 produce slightly different delta files; do not mix utilities from different versions.
- `--maxmatch` generates many more alignments than `--mum`; can be very slow for large genomes with repeats.
- Self-alignment requires `--nosimplify` to see all repeats; default `--simplify` removes shadowed alignments.
- `show-tiling` assigns each contig to only one location; repetitive contigs may be misplaced or excluded.
- `dnadiff` overwrites existing output files without warning; use unique prefixes for different comparisons.

## Examples

### align a query genome to a reference genome
**Args:** `nucmer --prefix=myrun reference.fna query.fna`
**Explanation:** aligns query.fna to reference.fna; writes myrun.delta; --prefix avoids overwriting default out.delta

### generate a comprehensive pairwise genome comparison report
**Args:** `dnadiff reference.fna query.fna`
**Explanation:** wraps nucmer + all show-* tools; produces out.report (global stats), out.snps (SNPs/indels), out.rdiff/.qdiff (rearrangements)

### filter alignments to 1-to-1 (unique) and extract SNPs
**Args:** `delta-filter -1 myrun.delta > myrun.filtered.delta && show-snps -Clr myrun.filtered.delta > myrun.snps`
**Explanation:** -1 keeps only one-to-one alignments; -Clr in show-snps gives clean tab-delimited SNP output without context sequences

### show alignment coordinates
**Args:** `show-coords -r -c -l myrun.delta > myrun.coords`
**Explanation:** -r sorts by reference position; -c adds percent identity and coverage columns; -l adds sequence lengths; human-readable alignment summary

### generate a synteny dot-plot image
**Args:** `mummerplot --png --prefix=dotplot myrun.delta`
**Explanation:** creates dotplot.png visualising alignment positions; --filter removes repetitive hits for a cleaner image; requires gnuplot

### compare two genomes with verbose SNP output
**Args:** `nucmer --mum -p compare reference.fa query.fa && show-snps -Clrx compare.delta`
**Explanation:** --mum uses only maximal unique matches (more stringent); -x in show-snps shows flanking sequence context for each SNP

### align with a custom minimum match length
**Args:** `nucmer -c 100 -l 20 --prefix large_genome ref.fa query.fa`
**Explanation:** -c 100 sets minimum cluster length; -l 20 sets minimum MUM length; larger values speed up alignment of large genomes with fewer spurious hits

### find all matches including repeats with maxmatch
**Args:** `nucmer --maxmatch --prefix=all_matches ref.fa query.fa`
**Explanation:** --maxmatch finds all matches regardless of uniqueness; useful for repeat analysis but slower

### align genome to itself for repeat detection
**Args:** `nucmer --maxmatch --nosimplify --prefix=self_align genome.fa genome.fa`
**Explanation:** --nosimplify preserves shadowed alignments; essential for self-alignment to find all repeats

### generate tiling path for assembly validation
**Args:** `show-tiling -i 95 -l 1000 alignment.delta > tiling.txt`
**Explanation:** show-tiling constructs best placement of query contigs on reference; -i 95 -l 1000 filters by identity and length

### identify structural rearrangements with show-diff
**Args:** `show-diff -rH alignment.mdelta > rearrangements.rdiff`
**Explanation:** show-diff classifies breakpoints; -rH outputs reference breakpoints in human-readable format

### filter alignments by minimum identity and length
**Args:** `delta-filter -i 95 -l 10000 alignment.delta > filtered.delta`
**Explanation:** -i 95 requires 95% identity; -l 10000 requires 10kb length; removes spurious short alignments

### extract alignments for specific sequences
**Args:** `show-aligns alignment.delta ref_id query_id`
**Explanation:** show-aligns displays full alignment for specific reference/query ID pair; useful for inspecting individual alignments

### compare divergent genomes with protein-level alignment
**Args:** `promer --prefix=protein_align ref.fa query.fa`
**Explanation:** promer uses 6-frame translation; better for distant species where DNA similarity is low but protein conserved
