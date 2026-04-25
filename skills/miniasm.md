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
- Use -f parameter to provide read sequences; -m for minimum match length (default 100).
- -s sets minimum span (default 2000); -c sets minimum coverage (default 3); -i sets minimum identity (default 0.05).
- -o sets minimum overlap (defaults to -s value); -h sets max overhang length (default 0).
- -I sets minimum end-to-end match ratio (default 0.8); -g sets max gap for trans-reduction (default 1000).
- -p sets output format: ug (unitig, default), sg (string graph), bed, or paf.

## Pitfalls
- Miniasm output is NOT ready to use without polishing — always run Racon + Medaka afterward.
- Miniasm GFA output requires conversion to FASTA for downstream tools.
- Without minimap2 overlaps first, miniasm cannot run — the overlap step is mandatory.
- Miniasm is best for quick draft assemblies or when speed is more important than accuracy.
- For high-quality assemblies, use Flye or Hifiasm instead of miniasm.
- -R flag pre-filters contained reads but requires 2-pass; increases runtime but may improve assembly.
- -m 100 (default) may be too low for noisy data; increase to 200-500 for better specificity.
- -i 0.05 (5% identity) is very permissive; increase to 0.1-0.2 for cleaner assemblies.
- -s 2000 (default) filters short overlaps; reduce for small genomes or increase for large/complex ones.
- -c 3 (default) requires 3 overlapping reads; increase for repetitive genomes to reduce misassemblies.

## Examples

### compute all-vs-all overlaps for ONT reads with minimap2
**Args:** `-x ava-ont -t 16 reads.fastq.gz reads.fastq.gz | gzip > overlaps.paf.gz`
**Explanation:** minimap2 command; -x ava-ont preset for ONT all-vs-all overlap; -t 16 threads; reads.fastq.gz input reads twice; output piped to gzip; > overlaps.paf.gz output

### assemble ONT reads from precomputed overlaps
**Args:** `-f reads.fastq.gz overlaps.paf.gz > assembly.gfa`
**Explanation:** miniasm command; -f reads.fastq.gz read FASTQ; overlaps.paf.gz input overlaps; > assembly.gfa output GFA

### convert miniasm GFA output to FASTA
**Args:** `/^S/ {print ">"$2"\n"$3}`
**Explanation:** awk pattern; awk '/^S/ {print ">"$2"\n"$3}' assembly.gfa > assembly.fasta; converts GFA to FASTA

### assemble PacBio reads with stricter parameters
**Args:** `-f reads.fq.gz -m 200 -i 0.1 -s 3000 -c 5 overlaps.paf.gz > assembly.gfa`
**Explanation:** miniasm command; -f reads.fq.gz read FASTQ; -m 200 min match; -i 0.1 min identity 10%; -s 3000 min span; -c 5 min coverage; overlaps.paf.gz input overlaps; > assembly.gfa output

### assemble with pre-filtering of contained reads
**Args:** `-R -f reads.fq.gz overlaps.paf.gz > assembly.gfa`
**Explanation:** miniasm command; -R pre-filters contained reads (2-pass); -f reads.fq.gz read FASTQ; overlaps.paf.gz input overlaps; > assembly.gfa output

### output string graph instead of unitigs
**Args:** `-p sg -f reads.fq.gz overlaps.paf.gz > string_graph.gfa`
**Explanation:** miniasm command; -p sg outputs string graph format; -f reads.fq.gz read FASTQ; overlaps.paf.gz input overlaps; > string_graph.gfa output

### assemble with custom overlap drop ratios
**Args:** `-f reads.fq.gz -r 0.8,0.6 -F 0.9 overlaps.paf.gz > assembly.gfa`
**Explanation:** miniasm command; -f reads.fq.gz read FASTQ; -r 0.8,0.6 max/min overlap drop ratio; -F 0.9 aggressive drop ratio; overlaps.paf.gz input overlaps; > assembly.gfa output

### quick assembly for small genomes with relaxed parameters
**Args:** `-f reads.fq.gz -m 50 -s 500 -c 2 overlaps.paf.gz > assembly.gfa`
**Explanation:** miniasm command; -f reads.fq.gz read FASTQ; -m 50 -s 500 -c 2 relaxed parameters; overlaps.paf.gz input overlaps; > assembly.gfa output

### complete miniasm pipeline with polishing (Racon + Medaka)
**Args:** `-f reads.fq.gz overlaps.paf.gz > assembly.gfa && awk '/^S/ {print ">"$2"\n"$3}' assembly.gfa > draft.fasta && racon -t 8 reads.fq.gz overlaps.paf.gz draft.fasta > polished.fasta && medaka_consensus -i reads.fq.gz -d polished.fasta -o medaka_output -t 8`
**Explanation:** miniasm command; -f reads.fq.gz input; overlaps.paf.gz input; > assembly.gfa output; awk converts GFA to FASTA; racon -t 8 polishing; medaka_consensus -i -d -o -t parameters

### compute overlaps for HiFi PacBio reads with higher accuracy
**Args:** `-x ava-pb -t 16 reads.fastq.gz reads.fastq.gz | gzip > overlaps_hifi.paf.gz`
**Explanation:** minimap2 command; -x ava-pb preset for PacBio HiFi; -t 16 threads; reads.fastq.gz input reads twice; output piped to gzip; > overlaps_hifi.paf.gz output

### assemble with BED output for visualization
**Args:** `-p bed -f reads.fq.gz overlaps.paf.gz > assembly.bed`
**Explanation:** miniasm command; -p bed outputs BED format; -f reads.fq.gz read FASTQ; overlaps.paf.gz input overlaps; > assembly.bed output

### assemble with PAF output for downstream analysis
**Args:** `-p paf -f reads.fq.gz overlaps.paf.gz > assembly_overlaps.paf`
**Explanation:** miniasm command; -p paf outputs PAF format; -f reads.fq.gz read FASTQ; overlaps.paf.gz input overlaps; > assembly_overlaps.paf output

### check assembly statistics from GFA output
**Args:** `grep "^S" assembly.gfa | awk '{len+=$3} END {print "Total length:", len, "Contigs:", NR}'`
**Explanation:** grep "^S" assembly.gfa extracts S lines; awk computes total length and contig count; quick quality assessment

### assemble with high coverage threshold for repetitive genomes
**Args:** `-f reads.fq.gz -c 10 -s 5000 overlaps.paf.gz > assembly.gfa`
**Explanation:** miniasm command; -f reads.fq.gz read FASTQ; -c 10 requires 10 overlapping reads; -s 5000 min span; overlaps.paf.gz input overlaps; > assembly.gfa output
