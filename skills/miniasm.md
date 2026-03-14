---
name: miniasm
category: assembly
description: Ultrafast de novo long-read assembler using overlap-layout-consensus approach without error correction
tags: [assembly, long-read, nanopore, pacbio, de-novo, draft-assembly, fast]
author: oxo-call built-in
source_url: "https://github.com/lh3/miniasm"
---

## Concepts

- Miniasm is an ultrafast long-read assembler (OLC approach); it skips error correction so output has high error rate (~5-10%).
- Miniasm requires pre-computed overlaps from minimap2: minimap2 -x ava-ont reads.fq reads.fq > overlaps.paf
- Two-step pipeline: (1) minimap2 for all-vs-all overlaps; (2) miniasm for assembly.
- Output is in GFA format; convert to FASTA: awk '/^S/ {print ">"$2; print $3}' assembly.gfa > assembly.fasta
- Miniasm assembly MUST be polished with Racon or Medaka before use — raw assembly is too noisy.
- Miniasm is much faster than Flye/Hifiasm but produces lower quality initial assemblies.
- Use -f parameter to filter reads shorter than a threshold; -I for minimum overlap score.

## Pitfalls

- Miniasm output is NOT ready to use without polishing — always run Racon + Medaka afterward.
- Miniasm GFA output requires conversion to FASTA for downstream tools.
- Without minimap2 overlaps first, miniasm cannot run — the overlap step is mandatory.
- Miniasm is best for quick draft assemblies or when speed is more important than accuracy.
- For high-quality assemblies, use Flye or Hifiasm instead of miniasm.

## Examples

### compute all-vs-all overlaps for ONT reads with minimap2
**Args:** `-x ava-ont -t 16 reads.fastq.gz reads.fastq.gz | gzip > overlaps.paf.gz`
**Explanation:** -x ava-ont preset for ONT all-vs-all overlap; -t 16 threads; gzip output overlaps

### assemble ONT reads from precomputed overlaps
**Args:** `-f reads.fastq.gz overlaps.paf.gz > assembly.gfa`
**Explanation:** -f reads FASTQ; overlaps PAF (can be gzipped); outputs GFA assembly

### convert miniasm GFA output to FASTA
**Args:** `/^S/ {print ">"$2"\n"$3}`
**Explanation:** awk command: awk '/^S/ {print ">"$2"\n"$3}' assembly.gfa > assembly.fasta
