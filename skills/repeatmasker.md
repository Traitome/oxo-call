---
name: repeatmasker
category: annotation
description: Screens DNA sequences for interspersed repeats, low-complexity regions, and transposable elements
tags: [repeat, transposon, annotation, masking, genome, te, line, sine]
author: oxo-call built-in
source_url: "https://www.repeatmasker.org/"
---

## Concepts
- RepeatMasker identifies and masks transposable elements, repetitive DNA, and low-complexity regions.
- Use -species to specify the organism (e.g., human, mouse, arabidopsis) for the repeat database.
- Output: <input>.masked (hard-masked N), <input>.softmasked or use -xsmall for softmasked (lowercase).
- RepeatMasker uses RepBase and/or Dfam databases; Dfam is freely available.
- Use -pa N for parallel processing (N processes); -dir for output directory.
- Softmasking (lowercase) is preferred for gene prediction tools like AUGUSTUS, MAKER.
- RepeatMasker output .out file contains coordinates, family, class, and score for each repeat.
- bedtools intersect with the .out file can mask custom repeat annotations.
- -s slow search is 0-5% more sensitive but 2-3x slower; -q quick search is faster but less sensitive.
- -qq rush job is fastest but ~10% less sensitive; suitable for most routine work.
- -div masks only repeats less than N% diverged from consensus; for younger repeats only.
- -gff creates GFF format output; useful for genome browsers and downstream analysis.
- -uncurated uses both curated and uncurated families from Dfam.

## Pitfalls
- RepeatMasker is slow for large genomes (plant/mammal) — use -pa for parallelization.
- -xsmall creates softmasked output (lowercase repeats); default creates hard-masked (N) output.
- The species parameter must match RepBase/Dfam species naming — check available species.
- Without -dir, output files go to the same directory as input — use -dir for clean organization.
- Gene prediction tools like AUGUSTUS need softmasked, not hard-masked input.
- RepeatMasker may need updated databases (Dfam/RepBase) — check database version before running.
- -s (slow) is rarely needed; -q (quick) or -qq (rush) are usually sufficient.
- -div filters by divergence; lower values mask only younger, less diverged repeats.
- -uncurated increases sensitivity but may include more false positives.
- -cutoff 225 (default) is for custom libraries; adjust based on library quality.
- -nolow skips simple repeats; use only when specifically avoiding low-complexity masking.

## Examples

### softmask repeats in a mammalian genome assembly
**Args:** `-species human -xsmall -pa 16 -dir repeatmasker_output/ genome.fasta`
**Explanation:** -species human; -xsmall softmasking (lowercase repeats); -pa 16 parallel; -dir output directory

### hard-mask repeats in a plant genome
**Args:** `-species arabidopsis -pa 8 -dir masked_output/ genome.fasta`
**Explanation:** default hard-masking with N; -species arabidopsis; -pa 8 for parallel processing

### mask repeats using a custom library
**Args:** `-lib custom_repeats.fasta -xsmall -pa 8 -dir custom_masked/ genome.fasta`
**Explanation:** -lib custom repeat library; useful when species-specific repeats are known

### mask only simple repeats and low-complexity regions
**Args:** `-noint -xsmall -pa 4 -dir simple_masked/ genome.fasta`
**Explanation:** -noint skips interspersed repeats (TEs); only masks simple repeats and low-complexity regions

### quick search for faster processing
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -q genome.fasta`
**Explanation:** -q quick search; 5-10% less sensitive but 2-5x faster; suitable for most work

### rush job for fastest processing
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -qq genome.fasta`
**Explanation:** -qq rush job; ~10% less sensitive but 4-10x faster; for preliminary analysis

### slow search for maximum sensitivity
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -s genome.fasta`
**Explanation:** -s slow search; 0-5% more sensitive but 2-3x slower; for final annotation

### mask only young repeats (low divergence)
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -div 20 genome.fasta`
**Explanation:** -div 20 masks only repeats <20% diverged from consensus; younger repeats only

### include uncurated Dfam families
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -uncurated genome.fasta`
**Explanation:** -uncurated uses both curated and uncurated families; increases sensitivity

### generate GFF output
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -gff genome.fasta`
**Explanation:** -gff creates GFF format output; useful for genome browsers and analysis

### mask with custom cutoff score
**Args:** `-lib custom_repeats.fasta -xsmall -pa 8 -dir custom_masked/ -cutoff 250 genome.fasta`
**Explanation:** -cutoff 250 sets higher threshold for custom library masking; more stringent

### exclude simple repeats from masking
**Args:** `-species human -xsmall -pa 16 -dir rm_output/ -nolow genome.fasta`
**Explanation:** -nolow skips low-complexity and simple repeats; only masks interspersed repeats
