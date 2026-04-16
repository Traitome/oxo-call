# LLM Integration

![LLM Prompt Architecture and Grounding Strategy](../images/llm-prompt.svg)

## Overview

oxo-call supports four LLM providers for command generation:

| Provider | Default Model | Token Required |
|----------|--------------|----------------|
| GitHub Copilot | gpt-5-mini (⭐ free tier) | Yes (GitHub App token via `oxo-call config login`) |
| OpenAI | gpt-4o | Yes |
| Anthropic | claude-3-5-sonnet-20241022 | Yes |
| Ollama | llama3.2 | No (local) |

## LLM Roles

oxo-call uses the LLM in up to three distinct roles per invocation:

| Role | Trigger | System Prompt |
|------|---------|---------------|
| **Command generation** (always) | Every `run` / `dry-run` | Expert bioinformatics command generator |
| **Task optimization** (`--optimize-task`) | Pre-generation step | Expand and clarify the user's task description |
| **Result verification** (`--verify`) | Post-execution step | Expert bioinformatics QC analyst |

Each role uses a separate system prompt so the LLM behaves appropriately for the job.

## Command Generation Prompt

### System Prompt

The command generation system prompt uses 10 concise rules that are optimised for
reliability across all model sizes. Three variants exist (see [Adaptive Prompt
Compression](#adaptive-prompt-compression) below); this section describes the **Full**
variant used for large models.

**Format Rule**

1. Respond with EXACTLY two labeled lines: `ARGS:` and `EXPLANATION:`. No other text.

**Invocation Rules (2–5)**

2. NEVER start ARGS with the tool name (auto-prepended by system).
3. First token = subcommand (sort, view, mem, index, etc), NEVER a flag.
4. Companion binaries (e.g. `bowtie2-build`) or scripts (e.g. `bbduk.sh`) go as first token when skill docs say so.
5. Multi-step: join with `&&`. Tool name auto-prepended ONLY to first segment — later commands MUST include their full binary name.

**Accuracy Rules (6–7)**

6. Use ONLY flags from docs or skill examples — never invent flags.
7. Include every file/path from the task. Prefer skill example flags. Include threads (`-@`/`-t`/`--threads`) and output (`-o`) when applicable.

**Convention Rules (8–10)**

8. Default conventions: paired-end, coordinate-sorted BAM, hg38, gzipped FASTQ, Phred+33.
9. Match format flags to actual types (BAM/SAM/CRAM, gzipped/plain, paired/single, FASTA/FASTQ).
10. If no arguments needed: `ARGS: (none)`.

### Response Format

The LLM must respond with exactly two labeled lines:

```
ARGS: <generated arguments>
EXPLANATION: <brief explanation of why these arguments were chosen>
```

If the response doesn't match this format, oxo-call retries the request.

### Raw Prompt Example

Below is an example of the complete user prompt sent to the LLM for a `samtools sort` task using the **Full** tier. The system prompt (shown above) is sent separately; the user prompt focuses on context and task.

```
# Tool: `samtools`

## Expert Knowledge (from skill)

### Key Concepts
- BAM files MUST be coordinate-sorted before indexing with samtools index
- Use -@ to set additional threads for parallel processing
- samtools view -F 0x904 filters out unmapped, secondary, and supplementary reads

### Common Pitfalls
- Forgetting to index after sorting — samtools index requires a coordinate-sorted BAM
- Using -q without -b — quality filtering without BAM output produces SAM to stdout
- Not specifying -o — output goes to stdout by default, which can corrupt terminal

### Worked Examples
Task: sort a BAM file by coordinate
Args: sort -o sorted.bam input.bam
Explanation: coordinate sort is the default; -o specifies output file

Task: index a sorted BAM file
Args: index sorted.bam
Explanation: creates .bai index required for random access

## Tool Documentation
<captured --help output and cached documentation>

## Task
sort input.bam by coordinate and output to sorted.bam

## Output
ARGS: <subcommand then flags, NO tool name>
EXPLANATION: <brief>
```

For the **Compact** tier (used with ≤3B models), the prompt uses a few-shot format:

```
Tool: samtools

Task: Sort a BAM file by coordinate

---FEW-SHOT---

ARGS: sort -@ 4 -o sorted.bam input.bam
EXPLANATION: Sort BAM by coordinate with 4 threads.

---FEW-SHOT---

Tool: samtools
Task: sort bam by coordinate

```

Use `--verbose` mode to see the actual prompt for any command:

```bash
oxo-call dry-run --verbose samtools "sort input.bam by coordinate"
```

## Task Optimization (`--optimize-task`)

When `--optimize-task` is set, an extra LLM call is made **before** command generation. The LLM is asked to rewrite the user's task into a precise bioinformatics instruction:

- Expands ambiguous terms into specific operations (e.g., "sort bam" → "sort BAM file input.bam by genomic coordinate and write to sorted.bam")
- Infers bioinformatics defaults (paired-end reads, hg38, 8 threads, gzipped output, Phred+33 encoding)
- Specifies output file names when omitted (derived from input names)
- Preserves all file names, paths, and sample identifiers from the original task
- Responds in the same language as the original task

The optimized task is shown to the user when it differs from the original and replaces the original in the command generation prompt.

## Result Verification (`--verify`)

When `--verify` is set on `run` or `workflow run`, an extra LLM call is made **after** execution. The LLM acts as a bioinformatics QC analyst and analyses:

- The exit code (with awareness that some tools use non-zero for warnings, exit 137 = OOM, exit 139 = segfault)
- Error signals in stderr (ERROR, FATAL, Exception, Traceback, Segmentation fault, OOM, Permission denied, etc.)
- Declared output files — their existence and sizes (zero-byte = suspicious)
- Tool-specific patterns (e.g., samtools truncated-BAM warnings, STAR alignment rate, GATK exceptions, BWA reference errors)
- Distinguishes fatal failures from harmless noise (progress bars, INFO/NOTE messages, version banners)

The structured response includes:

- `STATUS: success | warning | failure`
- `SUMMARY:` a one-sentence verdict in the same language as the task
- `ISSUES:` a list of detected problems (empty when clean)
- `SUGGESTIONS:` actionable fixes

Verification is advisory — it never changes the process exit code. In JSON mode (`--json`), a `verification` block is appended to the output.

## Provider Configuration

See the [Configuration tutorial](../tutorials/configuration.md) for setup instructions.

## Grounding Strategy

oxo-call uses a "docs-first" grounding strategy:

1. Tool documentation is fetched and included in the prompt
2. If a skill exists, expert knowledge is injected
3. The combined context prevents the LLM from hallucinating flags

This approach is critical for accuracy, especially with:

- Complex tools with hundreds of options
- Tools with version-specific flag differences
- Smaller or weaker LLM models

## Adaptive Prompt Compression

When `llm.context_window` is configured (or auto-detected from the model
name), oxo-call automatically compresses prompts to fit the model's context
budget. Three tiers are used, each with a **purpose-built system prompt** and
**user prompt strategy**:

| Tier | Context Window | System Prompt | User Prompt Strategy | Target Models |
|------|---------------|---------------|---------------------|---------------|
| **Full** | ≥ 16k or unknown | 10 rules (~450 tokens) | Skill → Docs → Task → concise Output | 7B+, ≥16K context |
| **Medium** | 4k – 16k | Medium-specific (~120 tokens) | Skill(5 examples) → truncated Docs → Task → Output | 4–7B, 4K–16K context |
| **Compact** | ≤ 4k | Concrete example + 3 rules (~80 tokens) | Few-shot(2 examples or fallback) → optional Docs → Task | ≤3B, any context |

### Tier Design Philosophy

**Full** — For models that can effectively use all available context. The system
prompt contains 10 comprehensive rules; the user prompt injects full skill
knowledge and complete documentation before the task.

**Medium** — For mid-range models with limited but usable context. Uses a
dedicated, shorter system prompt. Documentation is truncated to fit the
remaining budget after skill examples (up to 5, task-relevant selection) are
included. Docs are placed **after** skill but **before** task, so the model
focuses on expert knowledge first.

**Compact** — For small models (≤3B) that suffer from **context overflow**.
Key design decisions:

1. **Few-shot > instructions**: Small models imitate better than they follow
   rules. The `---FEW-SHOT---` markers create user/assistant/user turns that
   demonstrate the exact output format.
2. **Concrete examples > abstract placeholders**: The system prompt uses
   `ARGS: sort -@ 4 -o out.bam in.bam` instead of `ARGS: <subcommand then
   flags>`, because some models (e.g., starcoder2) would literally output the
   placeholder text.
3. **No format template in the final user message**: Including `Output:\nARGS:
   sort...` causes some models to output empty — they interpret the template
   as the answer already being provided.
4. **Fallback generic example**: When no skill is loaded, a `samtools sort`
   example is injected so the model always sees the correct output format.
5. **Selective documentation injection**: When no skill examples are available,
   a heavily truncated doc section is injected as the only grounding source.

### Auto-Detection

The context window is inferred from common model name patterns:

| Model Name Pattern | Detected Context | Tier |
|-------------------|-----------------|------|
| `qwen2.5-coder:0.5b`, `phi-3:3b` | 2,048 | Compact |
| `llama3:8b`, `deepseek-coder-v2:16b` | 8,192 | Medium |
| `qwen2.5:72b`, `llama3:70b` | 32,768 | Full |
| `gpt-4o`, `gpt-5-mini` | 128,000 | Full |
| `claude-3-5-sonnet` | 200,000 | Full |

### Manual Configuration

Override auto-detection via `config.toml` or environment variables:

```toml
[llm]
context_window = 4096   # force Medium tier
prompt_tier = "compact"  # force Compact tier regardless of context_window
```

Or per-invocation:

```bash
# Force a specific tier
oxo-call config set llm.prompt_tier compact    # ≤3B models
oxo-call config set llm.prompt_tier auto       # auto-detect (default)

# Override context window
oxo-call config set llm.context_window 2048    # force Compact

# Per-invocation via environment
OXO_CALL_LLM_PROMPT_TIER=compact oxo-call dry-run samtools "sort bam"
```

### Design Rationale

Mini models (≤ 3B parameters) suffer from **context overflow** — when the
prompt exceeds their effective context, the output quality degrades sharply
(empty output, format violations, hallucinated flags).  The Compact tier
addresses this by:

1. **Reducing system prompt** from ~1,600 characters (16 rules) to ~200
   characters (concrete example + 3 rules)
2. **Using few-shot instead of format instructions** — small models imitate
   better than they follow abstract rules
3. **Limiting to 2 most-relevant examples** as few-shot assistant messages
4. **Omitting raw documentation** unless no skill examples are available
5. **Avoiding format templates in the user message** that confuse small models

### Small Model Performance

After the 3-tier prompt system redesign, small model accuracy improved
dramatically:

| Model | Parameters | Before | After |
|-------|-----------|--------|-------|
| qwen2.5-coder | 0.5B | ~0% | 83–100% |
| deepseek-coder | 1.3B | ~20% | 75–100% |
| llama3.2 | 3B | ~0% | 100% |
| starcoder2 | 3B | ~0% | 91% |
| ministral | 3B | ~0% | 100% |

"Before" = original Full-tier prompt on all models. "After" = automatic tier
selection with the redesigned prompt system.
