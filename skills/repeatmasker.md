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

## Pitfalls

- RepeatMasker is slow for large genomes (plant/mammal) — use -pa for parallelization.
- -xsmall creates softmasked output (lowercase repeats); default creates hard-masked (N) output.
- The species parameter must match RepBase/Dfam species naming — check available species.
- Without -dir, output files go to the same directory as input — use -dir for clean organization.
- Gene prediction tools like AUGUSTUS need softmasked, not hard-masked input.
- RepeatMasker may need updated databases (Dfam/RepBase) — check database version before running.

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
