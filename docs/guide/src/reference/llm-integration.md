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

The command generation system prompt constrains the LLM's behavior with 16 rules organised into four categories:

**Critical Output Rules (1–3)**

1. Respond with EXACTLY two labeled lines: `ARGS:` and `EXPLANATION:`. No other text.
2. ARGS must be valid ASCII CLI flags and values — no markdown, code fences, or backticks.
3. EXPLANATION should be in the same language as the task description.

**Tool Invocation Rules (4–6)**

4. Never start ARGS with the tool name (it is prepended automatically). Companion binary and script executable exceptions allow the first token to be a companion binary (e.g., `bowtie2-build`) or a script (e.g., `bbduk.sh`).
5. For tools with subcommands, put the subcommand as the first ARGS token.
6. For tools using positional arguments before flags, place input files before flags.

**Multi-Step and Pipeline Rules (7–8)**

7. Join multi-step tasks with `&&`; the tool name is auto-prepended only to the first segment.
8. Include piping (`|`) and redirection (`>`) directly in ARGS.

**Accuracy and Bioinformatics Best Practices (9–16)**

9. Only use flags from the provided documentation or skill examples — never hallucinate.
10. Include every file name and path from the task description.
11. Prefer flags from the skill examples when they match the task.
12. Include thread counts, explicit output files, and reference/index files.
13. Use common bioinformatics conventions for ambiguous tasks (paired-end, coordinate-sorted BAM, hg38, gzipped FASTQ, Phred+33).
14. Match file format flags to actual input/output types.
15. Set correct strand-specific flags when library strandedness is mentioned.
16. Write `ARGS: (none)` when no arguments are needed.

### Response Format

The LLM must respond with exactly two labeled lines:

```
ARGS: <generated arguments>
EXPLANATION: <brief explanation of why these arguments were chosen>
```

If the response doesn't match this format, oxo-call retries the request.

### Raw Prompt Example

Below is an example of the complete user prompt sent to the LLM for a `samtools sort` task. This shows the actual structure including skill injection and format instructions.

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

## Output Format (STRICT — do not add any other text)
Respond with EXACTLY two lines:

ARGS: <all command-line arguments, space-separated, WITHOUT the tool name>
EXPLANATION: <one concise sentence — same language as the Task>

RULES:
- ARGS must NOT start with the tool name (it is prepended by the system)
- COMPANION BINARY: if the skill says the task needs a companion binary
  (e.g., 'bowtie2-build'), put it as the FIRST token in ARGS
- SCRIPT EXECUTABLE: if the skill shows a script (e.g., 'bbduk.sh',
  'infer_experiment.py') as the first token, use it directly in ARGS
- Use ONLY flags from the documentation or skill examples above — never invent flags
- Prefer flags from the skill examples when they match the task
- Include every file path mentioned in the task
- ARGS must be valid ASCII CLI flags and values — no markdown, no code fences
- If no arguments are needed, write: ARGS: (none)
- Piping (|) and redirection (>) go directly in ARGS
- Multi-step: join with '&&'; the tool name is auto-prepended ONLY to the
  first segment — each subsequent command MUST include its full binary name
  (e.g., 'sort ... && samtools index ...', NOT 'sort ... && index ...')
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
