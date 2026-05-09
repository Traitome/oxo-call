---
name: bellmans-gapc
category: Grammar Compiler / Sequence Analysis
description: A compiler that translates GAP grammar specifications into optimized C++ parsers and aligners using dynamic programming. The tool generates CYK (Cocke-Younger-Kasami) style parsers from formal grammars defined in the GAP language, enabling high-performance sequence analysis, RNA secondary structure prediction, and SCFG-based alignment.
tags:
  - grammar-based-dp
  - parser-generation
  - scfg
  - rna-structure
  - sequence-alignment
  - gap-language
  - cyk-algorithm
author: AI-generated
source_url: https://www.bioinf.uni-freiburg.de/Software/BellmansGapC/
---

## Concepts

- The GAP language lets you declare **terminals**, **non-terminals**, and **production rules** with associated scores (or probabilities for SCFGs). Bellmans- gapc translates these into a CYK dynamic programming engine: the generated C++ code contains $O(n^3 \\vert G \\vert)$ time complexity by default, where $n$ is the input length and $\\vert G \\vert$ is grammar size.
- Two-step build model: `bellmans-gapc` compiles a `.gap` grammar file into intermediate C++ (typically `foo CYK.cpp` and `foo-grammar.cpp`). Then `bellmans-gapc-build` links these against the Bellman runtime library to produce a final executable (`foo` or `foo.exe`). The `-o` flag for `bellmans-gapc` sets the output prefix; `bellmans-gapc-build` takes the matching base name.
- Input sequences for the generated executable must be in **FASTA or plain text** format (controlled by `--input-format` at generate time). For RNA covariance models, include **profile alignments** as `.stm` Stockholm format using the `-t stockholm` option.
- The `--algebra` flag selects the scoring semiring: default is Viterbi-style max-scoring, but `--algebra tropical` selects the standard log-sum-exp tropical semiring, and `--algebra probability` enables stochastic parsing where rule probabilities must sum to 1.0 per left-hand side non-terminal.
- The generated binary outputs the **inside score** (log-likelihood or Viterbi score) per sequence by default. Use `--analysis viterbi` to emit the single best parse tree (best derivation), or `--analysis inside` for full partition function scores across all parses.

## Pitfalls

- Forgetting to run `bellmans-gapc-build` after recompiling the grammar — the `.gap` source alone is not executable. Without the second build step, you will get a segfault or "undefined symbol" error when you try to run a raw `.gap` file.
- Defining a grammar where non-terminal probabilities do **not sum to 1.0** per left-hand side (required for `--algebra probability`). Bellmans-gapc will silently compile but the generated stochastic parser will produce unnormalized probability distributions, making downstream comparative analysis unreliable.
- Using `--memory cubic` on very long sequences (>5 kb) without the `--pruning` option causes the CYK chart to exhaust available RAM. The peak memory grows as $O(n^3)$ — a 10 kb sequence can require >8 GB RAM. Always use `--memory cubic --pruning` for large inputs.
- Omitting the `--output-file` flag on the generated executable causes it to write results only to stdout, which may truncate or corrupt large outputs (e.g., thousands of parse trees). Always redirect with `--output-file results.txt` for batch processing.
- Writing ambiguous grammars (multiple derivations with equal scores) without the `--disambiguation best` flag means the parser will silently return the first-found parse, not the globally best one under the selected algebra.

## Examples

### Compile a basic GAP grammar and build the executable
**Args:** `-o myscorer src/smallest-gap.gap`
**Explanation:** The `-o` flag sets the output prefix to `myscorer`; bellmans-gapc emits `myscorer-CYK.cpp` and `myscorer-grammar.cpp`, which are then linked by `bellmans-gapc-build myscorer`.

### Generate a parser with probability semiring for SCFG-based analysis
**Args:** `-o scorer --algebra probability -t plain src/covariance-gap.gap`
**Explanation:** `--algebra probability` generates a stochastic context-free grammar parser that emits log-probability scores; `-t plain` sets the input format to raw FASTA, suitable for covariance model sequence scoring.

### Build the executable with pruning enabled for large sequences
**Args:** `-o aligner --memory cubic --pruning src/rna-align-gap.gap`
**Explanation:** `--memory cubic` selects the full CYK chart, and `--pruning` activates Bellman-Ford-style shortest-path pruning in the generated binary to reduce peak RAM usage for RNA alignment grammars.

### Run the compiled executable on a FASTA input and save structured output
**Args:** `myscorer -o results.txt --analysis viterbi input.fa`
**Explanation:** After `bellmans-gapc` and `bellmans-gapc-build` produce the `myscorer` binary, this invocation runs Viterbi analysis on `input.fa` and writes per-sequence best-parse scores to `results.txt`.

### Compile a grammar targeting Stockholm format for RNA structural alignment
**Args:** `-o cmodel -t stockholm -a tropical src/cov-stoch-gap.gap`
**Explanation:** `-t stockholm` configures the generated parser to read Stockholm-format alignments as input, while `-a tropical` selects max-scoring (log-sum-exp) semiring for consensus structural analysis.