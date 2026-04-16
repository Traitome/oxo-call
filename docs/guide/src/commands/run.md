# run

Generate parameters with LLM and execute the tool.

## Synopsis

```
oxo-call run [OPTIONS] <TOOL> <TASK>
oxo-call r   [OPTIONS] <TOOL> <TASK>
```

## Options

| Option | Description |
|--------|-------------|
| `-a`, `--ask` | Prompt for confirmation before executing |
| `-m`, `--model <MODEL>` | Override the LLM model for this invocation |
| `--no-cache` | Skip cached documentation and fetch fresh `--help` output |
| `--json` | Output result as JSON (useful for scripting and CI) |
| `--verify` | After execution, use LLM to validate results (output files, stderr, exit code) |
| `--optimize-task` | Before generating the command, use LLM to expand and refine the task description |
| `-V`, `--var KEY=VALUE` | Substitute `{KEY}` in the task description before the LLM call (repeatable) |
| `-i`, `--input-list <FILE>` | Read input items from a file; runs the generated command for each item |
| `--input-items <ITEMS>` | Comma-separated input items; runs the generated command for each item |
| `-j`, `--jobs <N>` | Maximum parallel jobs when using `--input-list` / `--input-items` (default: 1) |
| `-x`, `--stop-on-error` | Abort remaining items after the first failure |
| `-v`, `--verbose` | Show docs source, skill info, and LLM details (global) |
| `--license <PATH>` | Path to license file (global option) |

### Prompt Tier

The prompt compression tier is auto-detected from the model size and context
window. You can override it with the `llm.prompt_tier` config key or the
`OXO_CALL_LLM_PROMPT_TIER` environment variable:

```bash
# Force Compact tier for a small model
OXO_CALL_LLM_PROMPT_TIER=compact oxo-call run samtools "sort bam by coordinate"

# Force Medium tier
OXO_CALL_LLM_PROMPT_TIER=medium oxo-call run samtools "sort bam by coordinate"
```

Use `--verbose` to see which tier was selected for a given invocation.

## Description

The `run` command is the primary way to use oxo-call. It:

1. Fetches the tool's documentation (from cache or `--help` output)
2. *(with `--optimize-task`)* Expands and clarifies your task description with the LLM
3. Loads any matching skill (built-in, community, or user-defined)
4. Sends the grounded prompt to the configured LLM
5. Parses the response to extract command arguments
6. *(with `--input-list` / `--input-items`)* Executes the command template for each item
7. Records the execution in command history
8. *(with `--verify`)* Asks the LLM to review the outputs and report issues

## Examples

```bash
# Basic usage
oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"

# With confirmation prompt
oxo-call run --ask bcftools "call variants from aligned.bam using ref.fa"

# Use LLM to verify outputs after execution
oxo-call run --verify samtools "sort input.bam by coordinate, output sorted.bam"

# Expand a vague task before generating the command
oxo-call run --optimize-task bwa "align reads to ref"

# Combine both for maximum accuracy and feedback
oxo-call run --optimize-task --verify samtools "sort bam"

# Override LLM model for a single invocation
oxo-call run --model gpt-4 samtools "index sorted.bam"

# Force fresh documentation (skip cache)
oxo-call run --no-cache samtools "sort by name"

# Get JSON output for scripting
oxo-call run --json samtools "flagstat input.bam"

# Variable substitution: replace {SAMPLE} in the task before the LLM call
oxo-call run --var SAMPLE=NA12878 samtools \
    "sort {SAMPLE}.bam by coordinate and output to {SAMPLE}.sorted.bam"

# Batch mode: generate the command template once, run for each BAM in a list
oxo-call run samtools "sort {item} by coordinate, output {item}.sorted.bam" \
    --input-list bam_files.txt --jobs 4

# Same with inline items
oxo-call run samtools "index {item}" --input-items s1.bam,s2.bam,s3.bam --jobs 2

# Preview all batch commands without executing (dry-run)
oxo-call dry-run samtools "flagstat {item}" --input-items s1.bam,s2.bam

# Combine vars and batch input
oxo-call run bwa "align {item} to {REF} with {THREADS} threads" \
    --var REF=hg38.fa --var THREADS=8 \
    --input-list samples.txt --jobs 4
```

## Variable substitution (`--var`)

Use `--var KEY=VALUE` to inject values into the task description before
the LLM receives it. Multiple `--var` flags are allowed:

```bash
oxo-call run --var TOOL=samtools --var INPUT=sample.bam \
    samtools "sort {INPUT} by coordinate"
```

This substitutes `{INPUT}` → `sample.bam` in the task string before the LLM
call, so the LLM generates a concrete command rather than a template.

## Batch / parallel mode (`--input-list` / `--input-items` / `--jobs`)

When you provide a list of input items, the LLM is called **once** to generate
a command template (which may contain `{item}`). The template is then executed
for each item in the list.

| Placeholder | Expands to |
|-------------|-----------|
| `{item}` / `{line}` / `{}` | The current input item (`{}` is the rush-compatible form) |
| `{nr}` | 1-based item number |
| `{basename}` | Filename without directory |
| `{dir}` | Directory portion of the item path (or `.`) |
| `{stem}` | Filename without last extension |
| `{ext}` | File extension without dot |

**Input list file format**: one item per line; blank lines and lines starting
with `#` are ignored. IO errors during reading are propagated immediately
(no silent truncation).

**Parallelism**: set `-j N` (or `--jobs N`) to run up to N items concurrently.
The default is 1 (sequential). Exit codes are collected after all items finish;
any failure causes the overall command to exit non-zero.

**Stop-on-error** (`-x` / `--stop-on-error`): abort after the first item
failure — useful in pipelines where continuing on error would produce incorrect
downstream results.

**JSON output** (`--json`) in batch mode returns an array of per-item results:

```json
{
  "tool": "samtools",
  "task_template": "flagstat {item}",
  "command_template": "samtools flagstat {item}",
  "total": 3,
  "failed": 0,
  "success": true,
  "results": [
    { "item": "s1.bam", "command": "samtools flagstat s1.bam", "exit_code": 0, "success": true },
    { "item": "s2.bam", "command": "samtools flagstat s2.bam", "exit_code": 0, "success": true },
    { "item": "s3.bam", "command": "samtools flagstat s3.bam", "exit_code": 0, "success": true }
  ]
}
```

## LLM Result Verification (`--verify`)

When `--verify` is set, oxo-call captures the tool's stderr and probes the declared output files. It then asks the LLM to analyse the run and returns a structured verdict:

```
────────────────────────────────────────────────────────────
  LLM Verification: Issues detected
  Alignment completed but output BAM is suspiciously small.

  Issues:
    • sorted.bam — 0 bytes (likely empty output)
    • Stderr contains 'truncated file'

  Suggestions:
    → Check that input.bam is not corrupted
    → Re-run with --no-cache to refresh the tool documentation
────────────────────────────────────────────────────────────
```

Verification is advisory — it never changes the process exit code. Use `--json` to get the verification block in machine-readable form.

## Task Optimization (`--optimize-task`)

When `--optimize-task` is set, an extra LLM call refines the user's task description before command generation. For example:

- Input: `"sort bam"`
- Optimized: `"sort BAM file input.bam by coordinate using samtools sort with 8 threads, output to sorted.bam"`

The optimized task is shown when it differs from the original and is used for the command generation prompt. This is particularly useful for short or ambiguous task descriptions.

## JSON Output

When `--json` is used for a single-item run, the output is a JSON object:

```json
{
  "tool": "samtools",
  "task": "flagstat input.bam",
  "effective_task": "flagstat input.bam",
  "command": "samtools flagstat input.bam",
  "args": ["flagstat", "input.bam"],
  "explanation": "Generates alignment statistics for the BAM file",
  "dry_run": false,
  "exit_code": 0,
  "success": true,
  "skill": "samtools",
  "model": "gpt-4o"
}
```

When `--verify` is also used, an additional `verification` block is appended.

## Behavior

- Documentation is fetched automatically on first use and cached
- If a matching skill exists, expert knowledge is injected into the prompt
- The LLM response must contain `ARGS:` and `EXPLANATION:` lines
- On execution failure, the exit code is recorded in history
- Use `dry-run` to preview commands without executing
- Use `--no-cache` to force a fresh `--help` fetch when docs may be stale
- Use `--model` to quickly test different models without changing config
- With `--input-list` / `--input-items`, the LLM is called once; each item
  execution uses `sh -c` with `{item}` (and other placeholders) substituted
