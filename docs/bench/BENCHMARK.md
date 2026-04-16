# Systematic Evaluation Framework for Documentation-Grounded LLM Command Generation

## 1. Overview

This document describes the **evaluation framework** for **oxo-call**, a
documentation-grounded command-generation system for bioinformatics. The framework
enables systematic assessment of whether augmenting large language models (LLMs)
with structured tool documentation and domain-specific skill files produces
correct command-line invocations across 143 bioinformatics tools spanning 40
analytical domains.

> **⚠ Important — No Mock Benchmark Results**
>
> This document does not present benchmark results derived from mock/simulated
> evaluation. The mock mode (`--mock`) exists solely for **CI regression testing**
> of the evaluation pipeline itself (see §4.2 and §13.1). All accuracy numbers
> reported in production should come from real LLM API evaluations (`oxo-bench
> eval --config bench_config.toml`). See §12 for instructions.

### 1.1 Scope and Scale

| Dimension | Value |
|-----------|-------|
| Tools evaluated | 143 |
| Analytical domains | 40 |
| Reference scenarios | 1,430 (10 per tool) |
| Natural-language descriptions | 14,300 (10 phrasings per scenario) |
| Ablation scenarios | 5 (bare, prompt, skill, doc, full) |
| Evaluation task catalog | 74 curated LLM evaluation tasks |
| Simulated experimental scenarios | 10 omics assay types |

---

## 2. Key Innovations

### 2.1 Docs-First Grounding

Unlike retrieval-augmented generation (RAG) approaches that retrieve arbitrary
text chunks, oxo-call loads the **complete, structured documentation** for the
target tool into the LLM prompt before generation. This includes the tool's
`--help` output (fetched and cached locally), ensuring the model has access to
the authoritative flag vocabulary and syntax.

### 2.2 Skill-Augmented Prompting

Each tool is accompanied by a curated **skill file** (`skills/<tool>.md`)
containing YAML front-matter metadata, domain concepts, common pitfalls, and
≥5 worked examples with exact `ARGS` strings. The skill file is injected into
the system prompt alongside documentation, providing few-shot exemplars that
are domain-appropriate and syntactically verified.

### 2.3 Ablation Study Framework

The benchmark supports five ablation scenarios to isolate the contribution of
each grounding component:

| Scenario | System Prompt | Skill | Doc (help) | Description |
|----------|:---:|:---:|:---:|-------------|
| `bare` | ✗ | ✗ | ✗ | Raw LLM — no assistance |
| `prompt` | ✓ | ✗ | ✗ | LLM + oxo-call system prompt only |
| `skill` | ✓ | ✓ | ✗ | LLM + prompt + skill file examples |
| `doc` | ✓ | ✗ | ✓ | LLM + prompt + tool --help index |
| `full` | ✓ | ✓ | ✓ | Complete oxo-call pipeline |

### 2.4 Offline Pipeline Validation via Mock Mode

To enable fully offline, reproducible **testing of the evaluation pipeline
itself** without API costs, the framework implements a **deterministic mock
generator** that applies controlled perturbations to reference commands. This
allows CI-integrated regression testing to verify that metric computation,
CSV output, and aggregation logic are all correct.

**Mock mode is NOT for producing benchmark results.** It simulates LLM behavior
with hard-coded perturbation rates, not real model inference. See §13.1 for
the full list of limitations.

### 2.5 Multi-Provider Support

The benchmark supports evaluation against any OpenAI-compatible API endpoint,
including local Ollama deployments. Models can be specified via configuration
file or CLI flags:

```bash
# Evaluate 15 Ollama models serially
oxo-bench eval --models "qwen3.5:9b,mistral:7b,llama3.2:3b,..." \
               --api-base "http://localhost:11434" \
               --scenarios "bare,full" \
               --data-dir docs/bench/
```

### 2.6 Tool Exclusion

Benchmark evaluation focuses on basic bash/shell commands and bioconda
bioinformatics tools. Three categories are excluded:

- **Package managers**: conda, mamba, pip, pixi, cargo, docker, singularity
- **HPC schedulers**: slurm, pbs, sge, lsf, htcondor, kubectl
- **AI assistants**: claude, openclaw

---

## 3. Benchmark Design

### 3.1 Dataset Generation

The benchmark dataset is derived programmatically from the 143 skill files in
the `skills/` directory (excluding package managers, HPC schedulers, and AI
assistants). For each skill file containing *N* examples (*N* ≤ 10):

1. **Reference commands**: Each `### heading` / `**Args:**` / `**Explanation:**`
   block becomes one scenario (`scenario_id = <tool>_01`, `<tool>_02`, …,
   `<tool>_10`), yielding 1,430 reference commands stored in
   `reference_commands.csv`.

   **Shell metacharacter stripping**: Reference args are automatically cleaned
   of shell-specific constructs that oxo-call does not generate.  Specifically:
   - **Pipes** (`|`): only the first command in a pipeline is retained.
   - **Output redirections** (`>`, `>>`, `2>`, `2>>`): the operator and target
     file are removed.
   - **Input redirections** (`<`): the operator is removed, keeping the file
     as a positional argument.
   Content inside single- or double-quoted strings (e.g. awk patterns) is
   preserved.  This ensures reference commands represent clean single-command
   invocations that match what oxo-call produces.

   **Anti-leakage file substitution**: To prevent information leakage between
   skill files and benchmark data, all file and path tokens in the reference
   args are deterministically **substituted** with alternative realistic names.
   For example, `input.bam` may become `mapping.bam`, and `reads.fastq.gz` may
   become `sample.fastq.gz`.  Extensions are preserved; only the base names
   change.  This ensures the benchmark evaluates the LLM's ability to construct
   correct commands from natural-language descriptions, not its ability to
   memorise specific filenames from the skill examples.

   Task descriptions are then **enriched with the substituted file/path tokens
   and package/module identifiers** so that the LLM has the precise filenames
   needed to reproduce the reference command (e.g.
   `"sort a BAM file by genomic coordinates"` becomes
   `"sort a BAM file by genomic coordinates (counts.bam, mapping.bam)"`).

2. **Usage descriptions**: 10 natural-language rephrasings per scenario are
   generated by applying structurally different phrasing strategies, including
   question forms, goal-oriented framing, expert terse notation, verb synonym
   substitution, and experimental context injection. Each variant applies a
   genuinely different sentence structure rather than merely appending a suffix,
   yielding 14,300 semantically diverse descriptions in `usage_descriptions.csv`.
   Because the enriched task description is used as the base for all variants,
   every description carries the file references from the reference args.

   **Semantic diversity strategies**:
   - *Structural rewrites*: "sort bam" → "How can I sort bam?", "I need to sort bam"
   - *Verb synonyms*: "sort" → "reorder", "align" → "map", "filter" → "select"
   - *Experimental context*: When detectable from reference args, descriptions
     are prefixed with the analysis type (e.g., "I have RNA-seq data that I
     need to process: sort bam"), making them closer to real user requests.

> **Note on Anti-Leakage Measures**
>
> The file token substitution ensures that benchmark scenarios use different
> filenames than those in the original skill examples.  Combined with the
> ablation framework (especially the `bare` scenario which removes all skill
> and documentation context), this enables fair evaluation of the LLM's
> generalisation ability rather than its memorisation of training examples.

### 3.2 Domain Coverage

The 143 tools span 40 bioinformatics and computational biology domains:

> alignment, annotation, assembly, assembly-polishing, comparative-genomics,
> epigenomics, filesystem, functional-annotation,
> genome-alignment, genome-annotation, genomic-arithmetic, genomic-intervals,
> long-read, metagenomics, motif-analysis, networking,
> phylogenetics, population-genomics, programming, qc, quality-control, rna-seq,
> runtime, scientific-computing, scripting, sequence-comparison,
> sequence-manipulation, sequence-processing, sequence-search,
> sequence-utilities, single-cell, statistical-computing, structural-variants,
> text-processing, utilities, variant-annotation, variant-benchmarking,
> variant-calling, version-control, workflow-manager

The largest domains by tool count are: variant-calling (16 tools), qc (12 tools),
assembly (11 tools), and metagenomics (10 tools).

### 3.3 Simulated Experimental Scenarios

To contextualize the tool invocations, 10 representative omics experimental
scenarios are provided in `bench_scenarios.csv`:

| Scenario | Assay | Samples | Read Length | Reads/Sample |
|----------|-------|---------|-------------|--------------|
| rnaseq_3s_pe150 | RNA-seq | 3 | 150 bp | 5,000 |
| rnaseq_10s_pe150 | RNA-seq | 10 | 150 bp | 5,000 |
| wgs_2s_pe150 | WGS | 2 | 150 bp | 50,000 |
| atacseq_3s_pe50 | ATAC-seq | 3 | 50 bp | 20,000 |
| metagenomics_4s_pe150 | Metagenomics | 4 | 150 bp | 10,000 |
| chipseq_3s_pe75 | ChIP-seq | 3 | 75 bp | 15,000 |
| methylseq_2s_pe150 | Methyl-seq | 2 | 150 bp | 30,000 |
| scrnaseq_2s_10xv3 | scRNA-seq | 2 | 150 bp | 25,000 |
| 16s_6s_pe250 | 16S amplicon | 6 | 250 bp | 8,000 |

---

## 4. Evaluation Methodology

### 4.1 Evaluation Loop

For each `(description, scenario, model, repeat)` tuple:

1. **Enhanced mode**: The description is passed to `oxo-call dry-run --json
   --tool <tool>`, which loads tool documentation and skill files into the
   prompt before querying the LLM.

2. **Baseline mode**: The same description is passed to a bare LLM without
   documentation or skill context, simulating direct prompting.

3. The generated `ARGS` string is compared against `reference_args` using the
   multi-dimensional metric suite described in §4.3.

4. Results are recorded per-trial in `benchmark_trials.csv` (enhanced) and
   `baseline_trials.csv` (baseline).

### 4.2 Mock vs. Real Evaluation

| Property | Mock Mode (`--mock`) | Real Mode |
|----------|---------------------|-----------|
| LLM calls | None (deterministic perturbation) | Actual API calls |
| API key required | No | Yes (OpenAI / Anthropic) |
| Cost | Zero | Per-token API costs |
| Reproducibility | Fully deterministic | Stochastic (temperature-dependent) |
| Purpose | **CI regression testing, pipeline validation only** | Production accuracy measurement |

**Mock mode must not be used to generate benchmark results.** It applies
hard-coded perturbation rates that are not derived from empirical LLM behavior.
Any "accuracy" numbers from mock mode reflect the configured perturbation
probability, not real model capabilities.

#### 4.2.1 Mock Perturbation Model (CI Pipeline Testing Only)

The mock generator applies controlled perturbations to reference commands.
Perturbation type is selected deterministically by hashing the input tuple
`(scenario_id, desc_id, model, repeat)`:

| Perturbation | Operation | Simulates |
|--------------|-----------|-----------|
| Drop flag | Remove one non-first token | Missing flag recall |
| Swap flags | Exchange two adjacent tokens | Flag reordering |
| Add flag | Insert a hallucinated flag (`--verbose`, `--debug`, etc.) | Extra flag precision loss |
| Replace value | Change a numeric or path value | Wrong value substitution |

The perturbation rates are **arbitrary** and chosen only to exercise all
metric code paths in CI:

| Mode | Model | Perturbation Rate |
|------|-------|-------------------|
| Enhanced (mock) | GPT-4o | 0.3% |
| Enhanced (mock) | Claude 3.5 Sonnet | 0.4% |
| Enhanced (mock) | GPT-4o-mini | 0.5% |
| Baseline (mock) | GPT-4o | 30% |
| Baseline (mock) | Claude 3.5 Sonnet | 40% |
| Baseline (mock) | GPT-4o-mini | 55% |

### 4.3 Metrics

#### Trial-Level Metrics

Each trial produces the following measurements (columns in `benchmark_trials.csv`):

| Metric | Formula / Description | Notes |
|--------|----------------------|-------|
| `exact_match` | `generated == reference` (whitespace-normalized) | Primary metric — strict string equality |
| `token_jaccard` | \|A ∩ B\| / \|A ∪ B\| over raw token sets A, B | Legacy; order-insensitive, retained for backward compatibility |
| `flag_recall` | \|A ∩ B\| / \|B\| over raw token sets | Legacy; prefer `flag_group_recall` |
| `flag_precision` | \|A ∩ B\| / \|A\| over raw token sets | Legacy; prefer `flag_group_precision` |
| `flag_group_recall` | Fraction of reference flag–value groups found in generated output (**after semantic normalisation**) | **Preferred** — penalises `8 --threads` when reference is `--threads 8`; file-name agnostic |
| `flag_group_precision` | Fraction of generated flag–value groups matching reference (**after semantic normalisation**) | **Preferred** — group-aware precision; file-name agnostic |
| `flag_group_jaccard` | Jaccard over flag–value group sets (**after semantic normalisation**) | **Preferred** — group-aware Jaccard; file-name agnostic |
| `positional_order_match` | 1.0 if reference positional args appear in correct relative order (**after semantic normalisation**) | Penalises swapped positional argument order |
| `subcommand_match` | `tokens[0]_gen == tokens[0]_ref` | Correct subcommand selection |

### 8.2 Semantic Normalisation Layer

All primary metrics (flag-group, positional-order, exact-match) are computed
**after semantic normalisation** of both the generated and reference args.
This ensures that the benchmark measures *operational correctness* (whether
the LLM selected the right subcommand, flags, and values) rather than
*file-name matching* (whether it guessed the exact file names).

The normalisation pipeline:

1. **Strip redirects**: `> file`, `2> file`, `>> file` are removed along with
   their target file names.  Adding a redirect does not change the operation.
2. **Strip pipes**: Everything after `|` is removed.  Whether the user pipes
   to a downstream tool is a matter of workflow, not of the current command.
3. **Replace file paths with typed placeholders**:
   - `-o sorted.bam` → `-o <OUTPUT_1>`
   - `-f ref.fa` → `-f <REF_1>`
   - `input.bam` → `<INPUT_1>`
   This makes `sort -@ 4 -o sorted.bam input.bam` semantically identical to
   `sort -@ 4 -o out.bam reads.bam`.
4. **Preserve** all flags, subcommands, and numeric values exactly.

Legacy token-set metrics (`token_jaccard`, `flag_recall`, `flag_precision`)
use **raw** (unnormalised) comparison for backward compatibility.
| `accuracy_score` | `(base) × subcommand_veto × patterns_penalty`, where base = 0.35×recall + 0.25×precision + 0.25×jaccard + 0.15×positional | Composite score ∈ [0, 1]; subcommand veto (×0.3) and patterns penalty (×0.5) |
| `required_patterns_met` | Whether all required patterns from the eval task catalog appear in generated args | Critical-flag correctness; penalises `accuracy_score` when false |
| `format_valid` | Response contains `ARGS:` and `EXPLANATION:` lines | LLM response format compliance |
| `latency_ms` | Wall-clock time per trial (ms), including LLM inference | End-to-end response latency |
| `overhead_ms` | `latency_ms` − LLM inference time (ms) | oxo-call processing overhead (doc fetch, skill load, prompt build, response parse) |
| `error_message` | Error text from the generator | Empty on success; contains stderr or error details on failure |

#### Latency Decomposition

The benchmark reports two timing metrics to separate oxo-call's own overhead
from the model's inference time:

- **`latency_ms`** measures the complete wall-clock time from sending the
  request to receiving and parsing the response. This includes documentation
  fetching, skill loading, prompt construction, HTTP POST to the LLM API,
  model inference (TTFB + token generation), and response parsing.

- **`overhead_ms`** = `latency_ms` − `inference_ms`, where `inference_ms` is
  the model-reported inference time (when available). This isolates the time
  spent in oxo-call itself: documentation cache lookup, skill file loading,
  prompt template rendering, and `ARGS:`/`EXPLANATION:` parsing.

For local models (e.g. Ollama), the inference time dominates `latency_ms`.
To accurately assess oxo-call's own performance, focus on `overhead_ms`.
When the LLM API does not report inference time, `overhead_ms` equals
`latency_ms` (conservative upper bound).

#### Why Flag-Group Metrics?

The legacy token-set metrics (Jaccard, recall, precision) treat every
whitespace-separated token as an unordered set element. This means that
`--threads 8` and `8 --threads` score identically — even though the latter
is semantically wrong for most CLI tools (the parser would receive `8` as a
positional argument and `--threads` without a value).

The flag-group metrics group each named flag (`-f` / `--flag`) with its
immediately-following non-flag value token. For example:

| Command | Flag groups |
|---------|-------------|
| `cmd --threads 8 input.txt` | `["cmd"]`, `["--threads", "8"]`, `["input.txt"]` |
| `cmd 8 --threads input.txt` | `["cmd"]`, `["8"]`, `["--threads"]`, `["input.txt"]` |
| `cmd --8 threads input.txt` | `["cmd"]`, `["--8", "threads"]`, `["input.txt"]` |

The three forms above produce different flag-group sets and thus different
`flag_group_recall` / `flag_group_precision` / `flag_group_jaccard` scores.
The `accuracy_score` is computed over semantically-normalised group-aware
metrics, with a **subcommand veto** and **required-patterns penalty**:

1. **Base score** = 0.35 × flag_group_recall + 0.25 × flag_group_precision +
   0.25 × flag_group_jaccard + 0.15 × positional_order_match
2. **Subcommand veto**: When the subcommand is wrong, the base score is
   multiplied by 0.3 (capped at 0.3). This reflects that a wrong subcommand
   means the wrong operation entirely (e.g., `sort` vs `view` is not
   "partially correct").
3. **Required-patterns penalty**: When critical flags from the eval task
   catalog are missing (`required_patterns_met = false`), the score is
   further multiplied by 0.5.

#### Model-Level Aggregates

Model-level statistics are computed as means over all trials per model and
reported in `model_summary.csv`. Self-consistency is defined as the fraction
of `(scenario, description)` groups where all repeats produce identical output.

---

## 5. Comparison Framework

### 5.1 Conditions

| Condition | Context Provided | Skill File | Tool Docs | Abbreviation |
|-----------|-----------------|------------|-----------|--------------|
| **Enhanced** | Full oxo-call pipeline | ✓ | ✓ | oxo-call |
| **Baseline** | Bare LLM, no augmentation | ✗ | ✗ | Bare LLM |

The enhanced condition represents the full oxo-call system: documentation is
fetched and cached, the matching skill file is loaded, and both are injected
into the system prompt before the LLM generates the command. The baseline
condition uses the identical natural-language description but provides no tool
documentation or skill examples, isolating the model's parametric knowledge.

---

## 6. Results

*Real benchmark results will be populated here after running `oxo-bench eval`
with real LLM API calls. See §12 for instructions.*

---

## 7. Workflow Engine Performance

The DAG-based workflow engine (`src/engine.rs`) was profiled across 7 standard
bioinformatics workflows (from `bench_workflow.csv`):

| Workflow | Tasks | Parse (μs) | Expand (μs) | Cycle-Free |
|----------|-------|-----------|------------|------------|
| RNA-seq | 13 | 398.3 | 17.9 | ✓ |
| WGS | 13 | 446.8 | 22.6 | ✓ |
| ATAC-seq | 11 | 408.1 | 20.0 | ✓ |
| Metagenomics | 9 | 362.9 | 17.6 | ✓ |
| ChIP-seq | 19 | 516.8 | 22.7 | ✓ |
| scRNA-seq | 9 | 448.4 | 17.6 | ✓ |
| Long-reads | 11 | 391.0 | 19.3 | ✓ |

All workflows parse in <520 μs and expand in <23 μs, confirming negligible
overhead for DAG scheduling. These numbers are from real measurements of the
Rust workflow engine (not mock-generated).

---

## 8. Ablation Analysis

### 8.1 Component Contributions

The comparison framework isolates the contribution of oxo-call's two primary
augmentation components:

| Component | What It Provides | Expected Effect |
|-----------|-----------------|-----------------|
| **Tool documentation** (`--help` output) | Authoritative flag vocabulary, syntax rules, valid value ranges | Eliminates hallucinated flags, corrects value formats |
| **Skill files** (curated examples) | Few-shot exemplars with exact ARGS, domain concepts, pitfalls | Provides correct flag combinations, ordering conventions |

*Observed component contribution figures will be reported after real API evaluation.*

### 8.2 Adaptive Prompt Compression

For models with limited context windows (e.g. Ollama mini models), oxo-call
automatically compresses prompts to fit the available budget:

| Tier | Context Window | Strategy |
|------|---------------|----------|
| Full | ≥ 16,384 tokens | No compression; full system prompt + all skill examples + full docs |
| Medium | 4,096 – 16,383 | Full system prompt; ≤ 5 skill examples; docs truncated to fit |
| Compact | ≤ 4,095 | Ultra-compact system prompt (~200 chars); top-3 examples + 3 concepts + 2 pitfalls; docs heavily truncated |

The context window is configurable via `llm.context_window` or auto-detected
from model name patterns (e.g. `:0.5b` → 2,048, `:16b` → 8,192).

---

## 9. Error Analysis

### 9.1 Error Taxonomy

Errors are classified into seven mutually exclusive categories:

| Error Type | Definition |
|-----------|-----------|
| `missing_flag` | A required flag/token from the reference is absent |
| `extra_flag` | A generated flag/token not present in the reference |
| `wrong_value` | A flag is present but its value differs from reference |
| `flag_reorder` | All tokens present but in different order |
| `wrong_subcommand` | First token (subcommand) does not match |
| `format_error` | LLM response missing `ARGS:` or `EXPLANATION:` lines |
| `empty_output` | No output generated |

*Error distribution tables will be reported after real API evaluation.*

---

## 10. Statistical Analysis

### 10.1 Confidence Interval Method

Per-category 95% confidence intervals are computed using the **Wilson score
interval**, which is more robust than the Wald normal approximation when
the proportion is near 0 or 1 or the sample size is small:

$$\text{center} = \frac{\hat{p} + z^2/(2n)}{1 + z^2/n}, \quad \text{spread} = \frac{z \sqrt{\hat{p}(1-\hat{p})/n + z^2/(4n^2)}}{1 + z^2/n}$$

where $z = 1.96$ for a 95% interval, $\hat{p}$ is the observed proportion,
and $n$ is the number of trials. The Wilson interval never produces negative
lower bounds and has better coverage properties for extreme proportions
(e.g., when $\hat{p} \approx 0$ for very small models).

These are reported in the `accuracy_ci95` and `exact_match_ci95` columns of
`model_summary_by_category.csv`.

### 10.2 Effect Size Method

The enhanced-vs-baseline comparison reports Cohen's *h* using the arcsine
transformation of two proportions:

$$h = 2\arcsin(\sqrt{p_1}) - 2\arcsin(\sqrt{p_2})$$

**Interpretation guide** (reported in `effect_label` column of
`baseline_comparison.csv`):

| |h| Range | Label | Practical Meaning |
|-----------|-------|---------------------|
| < 0.2 | negligible | Skill/doc grounding has no measurable effect |
| 0.2 – 0.5 | small | Grounding helps slightly |
| 0.5 – 0.8 | medium | Grounding provides moderate improvement |
| > 0.8 | large | Grounding is essential for this model |

*Computed effect sizes will be reported after real API evaluation.*

---

## 11. Companion Binary Dispatch

Many bioinformatics tools ship companion binaries (e.g., `bowtie2-build` for
`bowtie2`, `hisat2-build` for `hisat2`, `rsem-prepare-reference` for `rsem`,
`deduplicate_bismark` for `bismark`). oxo-call handles these transparently:

- A skill file (e.g., `bowtie2.md`) covers both the main tool and its companions.
- When the LLM outputs a companion binary name as the first ARGS token (e.g.,
  `bowtie2-build reference.fa ref_idx`), oxo-call detects it automatically and
  invokes the companion binary instead of the parent tool.

**Detection rules** (`is_companion_binary(tool, candidate)`):

| Pattern | Example | Matches? |
|---------|---------|----------|
| Forward prefix `{tool}-` | `bowtie2-build` (tool=`bowtie2`) | ✓ |
| Forward prefix `{tool}_` | `bismark_methylation_extractor` (tool=`bismark`) | ✓ |
| Reverse suffix `_{tool}` | `deduplicate_bismark` (tool=`bismark`) | ✓ |
| Flag (starts with `-`) | `-x`, `--threads` | ✗ |
| Contains dot (file name) | `bowtie2-input.fq` | ✗ |
| No prefix/suffix match | `sort` (tool=`samtools`) | ✗ |

Users can still index a companion binary's documentation directly:
```bash
oxo-call docs index bowtie2-build   # manual docs index for the companion binary
oxo-call run bowtie2 "build index from reference.fa"  # uses bowtie2-build automatically
```

---

## 12. Reproducibility

### 12.1 Generating Reference Data

The reference commands and usage descriptions are deterministically generated
from skill files:

```bash
# Generate reference commands and usage descriptions from skill files
./target/debug/oxo-bench generate --skills-dir skills/ --output docs/bench/
```

### 12.2 Running Real Evaluation

To obtain real benchmark results, run evaluation against live LLM APIs:

```bash
# Initialize a benchmark configuration template
./target/debug/oxo-bench init-config --output bench_config.toml

# Edit bench_config.toml to configure models, API keys, and scenarios

# Run real evaluation (requires API keys)
./target/debug/oxo-bench eval --config bench_config.toml \
                              --data-dir docs/bench/ \
                              --output bench_results/

# Or evaluate specific models with CLI overrides
./target/debug/oxo-bench eval --models "gpt-4o,gpt-4o-mini" \
                              --scenarios "bare,full" \
                              --data-dir docs/bench/ \
                              --output bench_results/
```

### 12.3 CI Pipeline Validation (Mock Mode)

Mock mode is provided **exclusively** for validating the evaluation pipeline
itself in CI, without requiring API keys or incurring costs:

```bash
# Validate pipeline metrics and CSV output (CI use only)
./target/debug/oxo-bench eval --mock --data-dir docs/bench/ --output /tmp/mock_out/
```

The mock output verifies that metric computation, CSV serialization, and
aggregation logic are working correctly. **Do not use mock output as benchmark
results.** The perturbation rates are arbitrary and the results have no
relationship to real LLM behavior.

### 12.4 Adding New Benchmarks

1. **New tool**: Add a `skills/<tool>.md` with ≥5 examples following the
   standard YAML front-matter format, then run `oxo-bench generate` to
   regenerate CSVs.

2. **New evaluation task**: Add an entry to `canonical_eval_tasks()` in
   `crates/oxo-bench/src/bench/llm.rs`.

3. **New model**: Add a `[[models]]` entry to `bench_config.toml` (generate
   a template with `oxo-bench init-config`).

---

## 13. Limitations

### 13.1 Mock Mode is Not Real Evaluation

Mock mode (`--mock`) simulates LLM behavior through deterministic perturbation
of reference commands. It is designed exclusively for:

- **CI regression testing**: ensuring the evaluation pipeline doesn't break
  when the codebase changes
- **Methodology validation**: verifying that metric computation (exact_match,
  flag_group_jaccard, etc.) produces correct values on known inputs

Mock mode results **cannot** be used as accuracy claims because:

1. **Circular reasoning**: The "error rates" reflect the hard-coded perturbation
   probability, not real model behavior. Claiming "GPT-4o achieves 99.7% exact
   match" from mock data is equivalent to saying "we programmed it to fail 0.3%
   of the time."

2. **Fabricated error distribution**: Error categories (missing flags, extra
   flags, etc.) in mock mode reflect the perturbation operators, not real LLM
   failure modes.

3. **No real inference**: Format validity is always 100% in mock mode; latency
   is always 0 ms. Neither is representative.

4. **Data leakage in enhanced mode**: The reference examples are drawn directly
   from the same skill files that are injected into the LLM prompt. Even in
   real evaluation, enhanced results represent an upper bound.

### 13.2 Skill File Overlap

In the enhanced condition, the LLM prompt contains the exact skill file from
which benchmark scenarios are derived. This represents a best-case scenario
where the documentation perfectly matches the user's intent. In production
usage, the skill file may not contain an example matching every possible user
request, and performance is expected to be lower than the enhanced benchmark.

### 13.3 Generalizability

The benchmark covers 143 tools across 40 domains, which represents broad but
not exhaustive coverage of the bioinformatics ecosystem. Tools not represented
in the skill library rely solely on `--help` documentation grounding, and their
accuracy is expected to lie between the baseline and enhanced performance levels.

### 13.4 Shell Metacharacter Stripping

Reference commands are automatically cleaned of shell pipes (`|`), output
redirections (`>`, `>>`, `2>`), and input redirections (`<`).  This is correct
because oxo-call generates single-command invocations, not shell pipelines.
However, some skill-file examples intentionally demonstrate piped workflows
(e.g. `bcftools mpileup | bcftools call`); in these cases only the first
command in the pipeline is benchmarked.

### 13.5 Flag-Group Metric Limitations

The flag-group parser uses a simple heuristic: a flag token (starting with `-`)
followed by a non-flag token is treated as a flag–value pair. This heuristic
may be wrong for:

- Boolean flags: `--verbose` followed by a positional may incorrectly consume
  the positional as a value (e.g., `--verbose input.bam` → group `["--verbose", "input.bam"]`)
- Multi-value flags: `--include A B` — only the first value `A` is captured
  in the group

The `exact_match` metric is immune to these parser limitations and remains the
most reliable evaluation criterion.

---

## 14. Benchmark Data Files

| File | Rows | Description |
|------|------|-------------|
| `reference_commands.csv` | 1,430 | Ground-truth ARGS for 143 tools × 10 scenarios (file tokens substituted to prevent leakage) |
| `usage_descriptions.csv` | 14,300 | 10 natural-language phrasings per scenario |
| `bench_scenarios.csv` | 9 | Simulated omics experimental scenarios |
| `bench_eval_tasks.csv` | 74 | Curated LLM evaluation task catalog with required flag patterns |
| `bench_workflow.csv` | 7 | Workflow parsing/expansion timing (real measurements) |

**Aggregate result CSVs** (`model_summary.csv`, `baseline_summary.csv`,
`baseline_comparison.csv`, `error_analysis.csv`, `baseline_error_analysis.csv`,
`model_summary_by_tool.csv`, `model_summary_by_category.csv`) are **not
committed** to the repository. They must be generated by running real API
evaluation:

```bash
oxo-bench eval --config bench_config.toml --data-dir docs/bench/ --output bench_results/
```

Large per-trial CSVs (`benchmark_trials.csv`, `baseline_trials.csv`,
`trials_*.csv`) are also excluded from version control (see `.gitignore`).
