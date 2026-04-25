---
name: racon
category: assembly
description: Ultrafast consensus module for raw de novo DNA assembly of long-uncorrected reads
tags: [assembly, polishing, consensus, long-read, nanopore, pacbio, correction]
author: oxo-call built-in
source_url: "https://github.com/lbcb-sci/racon"
---

## Concepts
- Racon polishes long-read assemblies by generating consensus sequences from overlapping reads.
- Requires: raw reads (FASTQ), mapping to assembly (PAF/SAM), and draft assembly (FASTA).
- Generate mapping with minimap2: minimap2 -x map-ont draft.fasta reads.fastq > mapping.paf
- Run multiple rounds of Racon polishing (2-4 rounds) before Medaka for ONT data.
- Use -t N for multi-threading; output is consensus FASTA to stdout.
- For ONT data after Racon, always run Medaka for additional polishing with neural network models.
- For PacBio CLR, 1-2 rounds of Racon followed by Arrow/GCPP is the standard workflow.
- -u (--include-unpolished) outputs unpolished target sequences; useful for retaining all contigs.
- -f (--fragment-correction) performs fragment correction instead of contig polishing.
- -w (--window-length) sets POA window size (default 500 bp); adjust for different read types.
- -q (--quality-threshold) sets minimum average base quality for windows (default 10.0).
- -e (--error-threshold) sets maximum allowed error rate for filtering overlaps (default 0.3).

## Pitfalls
- racon has NO subcommands. ARGS starts directly with positional arguments (reads, overlaps, target_sequences) or flags (e.g., -t, -u, -w). Do NOT put a subcommand like 'polish' or 'correct' before arguments.
- Racon requires fresh mapping to the CURRENT draft for each polishing round — remap before each Racon run.
- Racon output is to stdout — redirect to a file for the polished assembly.
- Too many Racon rounds (>4) may degrade assembly quality — use Medaka after 2-3 Racon rounds for ONT.
- The PAF alignment file must be computed with minimap2 against the DRAFT assembly, not the original reference.
- Racon requires all three inputs: reads, mapping PAF/SAM, and draft assembly FASTA.
- -u (--include-unpolished) is needed when some contigs have no read coverage; otherwise they are dropped.
- -w (--window-length) 500 (default) works for most data; increase for highly accurate reads.
- -e (--error-threshold) 0.3 (default) allows 30% error; decrease for more stringent overlap filtering.
- -q (--quality-threshold) 10.0 filters low-quality windows; increase for higher confidence consensus.
- Fragment correction (-f) requires dual/self overlaps; different from standard contig polishing workflow.

## Examples

### run one round of Racon polishing on an ONT assembly
**Args:** `-t 16 reads.fastq.gz mapping.paf draft_assembly.fasta > polished_round1.fasta`
**Explanation:** racon command; -t 16 threads; reads.fastq.gz input reads; mapping.paf alignment file from minimap2; draft_assembly.fasta input draft; > polished_round1.fasta output polished assembly; run racon after minimap2 mapping

### run second round of Racon polishing
**Args:** `-t 16 reads.fastq.gz round2_mapping.paf polished_round1.fasta > polished_round2.fasta`
**Explanation:** racon command; -t 16 threads; reads.fastq.gz input reads; round2_mapping.paf alignment file; polished_round1.fasta input from previous round; > polished_round2.fasta output; re-map to round1 output first

### run Racon polishing using SAM alignment instead of PAF
**Args:** `-t 16 reads.fastq.gz alignment.sam draft_assembly.fasta > polished_assembly.fasta`
**Explanation:** racon command; -t 16 threads; reads.fastq.gz input reads; alignment.sam SAM alignment input; draft_assembly.fasta input draft; > polished_assembly.fasta output; SAM format also accepted; use minimap2 -a flag for SAM

### include unpolished sequences in output
**Args:** `-t 16 -u reads.fastq.gz mapping.paf draft_assembly.fasta > polished_with_unpolished.fasta`
**Explanation:** racon command; -t 16 threads; -u includes unpolished target sequences; reads.fastq.gz input reads; mapping.paf alignment file; draft_assembly.fasta input draft; > polished_with_unpolished.fasta output; retains contigs with no read coverage

### fragment correction mode
**Args:** `-t 16 -f reads.fastq.gz self_overlaps.paf fragments.fasta > corrected_fragments.fasta`
**Explanation:** racon command; -t 16 threads; -f fragment correction mode; reads.fastq.gz input reads; self_overlaps.paf overlaps file with dual/self overlaps; fragments.fasta input fragments; > corrected_fragments.fasta output

### increase window length for accurate reads
**Args:** `-t 16 -w 1000 reads.fastq.gz mapping.paf draft_assembly.fasta > polished.fasta`
**Explanation:** racon command; -t 16 threads; -w 1000 increases POA window size; reads.fastq.gz input reads; mapping.paf alignment file; draft_assembly.fasta input draft; > polished.fasta output; may improve consensus for highly accurate reads

### stricter error threshold for filtering
**Args:** `-t 16 -e 0.2 reads.fastq.gz mapping.paf draft_assembly.fasta > polished_strict.fasta`
**Explanation:** racon command; -t 16 threads; -e 0.2 reduces maximum allowed error rate from 30% to 20%; reads.fastq.gz input reads; mapping.paf alignment file; draft_assembly.fasta input draft; > polished_strict.fasta output; more stringent filtering

### higher quality threshold for consensus
**Args:** `-t 16 -q 15 reads.fastq.gz mapping.paf draft_assembly.fasta > polished_highq.fasta`
**Explanation:** racon command; -t 16 threads; -q 15 increases minimum average base quality; reads.fastq.gz input reads; mapping.paf alignment file; draft_assembly.fasta input draft; > polished_highq.fasta output; higher confidence consensus windows

### disable consensus trimming at window ends
**Args:** `-t 16 --no-trimming reads.fastq.gz mapping.paf draft_assembly.fasta > polished_notrim.fasta`
**Explanation:** racon command; -t 16 threads; --no-trimming disables trimming at window ends; reads.fastq.gz input reads; mapping.paf alignment file; draft_assembly.fasta input draft; > polished_notrim.fasta output; may preserve more sequence

### PacBio CLR polishing workflow
**Args:** `-t 16 reads.fastq.gz mapping.paf draft_assembly.fasta > racon_polished.fasta`
**Explanation:** racon command; -t 16 threads; reads.fastq.gz input PacBio CLR reads; mapping.paf alignment file; draft_assembly.fasta input draft; > racon_polished.fasta output; for PacBio CLR: run 1-2 rounds of racon, then polish with Arrow/GCPP
