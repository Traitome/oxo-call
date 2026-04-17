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
| `-V`, `--var KEY=VALUE` | Substitute `{KEY}` in the task description before the LLM call (repeatable) |
| `-i`, `--input-list <FILE>` | Read input items from a file; runs the generated command for each item |
| `--input-items <ITEMS>` | Comma-separated input items; runs the generated command for each item |
| `-j`, `--jobs <N>` | Maximum parallel jobs when using `--input-list` / `--input-items` (default: 1) |
| `-x`, `--stop-on-error` | Abort remaining items after the first failure |
| `--auto-retry` | On failure, ask the LLM to analyze stderr and re-run with a corrected command (up to 2 retries) |
| `--scenario <SCENARIO>` | Force a workflow scenario: `basic`, `prompt`, `doc`, `skill`, or `full` (auto-detected by default) |
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
2. **Extracts structured knowledge** — flag catalog and command examples from the help text
3. Loads any matching skill (built-in, community, or user-defined)
4. Sends the doc-enriched prompt to the configured LLM (single call by default)
5. Parses the response to extract command arguments
6. *(with `--input-list` / `--input-items`)* Executes the command template for each item
7. Records the execution in command history
8. *(with `--verify`)* Asks the LLM to review the outputs and report issues

Step 2 is the key innovation: the flag catalog prevents hallucinated flags and
doc-extracted examples serve as few-shot demonstrations — enabling reliable
command generation even with small models (≤3B) and no skill files.

## Examples

```bash
# Basic usage
oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"

# With confirmation prompt
oxo-call run --ask bcftools "call variants from aligned.bam using ref.fa"

# Use LLM to verify outputs after execution
oxo-call run --verify samtools "sort input.bam by coordinate, output sorted.bam"

# Auto-retry on failure (LLM analyzes stderr and corrects the command)
oxo-call run --auto-retry samtools "sort input.bam with 8 threads"

# Maximum reliability: auto-retry and verify
oxo-call run --auto-retry --verify samtools "sort and index"

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

## Automatic Task Normalization

oxo-call uses a two-step process for task normalization:

1. **Quality mode selection**: When there is no static skill file for the tool and
   documentation is available, oxo-call selects Quality mode (multi-stage pipeline).
   If `--scenario` is set, the scenario's default mode takes precedence.

2. **Optional normalization within Quality mode**: Within the Quality pipeline, an extra
   LLM call normalizes the task **only if** it is considered vague or ambiguous:

- Input: `"sort bam"`
- Normalized: `"sort BAM file input.bam by coordinate using samtools sort with 8 threads, output to sorted.bam"`

The normalized task is shown when it differs from the original and is used for the command
generation prompt. This secondary normalization triggers when:

- The task is shorter than 10 characters
- The task contains vague keywords (e.g., "just", "simply", "basically")
- The task contains non-ASCII characters (e.g., Chinese, Japanese)

## Risk Assessment

oxo-call automatically assesses the risk level of generated commands before execution:

| Risk Level | Trigger | Behavior |
|------------|---------|----------|
| **Safe** | Normal bioinformatics commands | Proceeds normally |
| **Warning** | Force flags (`-f`, `--force`), output redirection (`>`), same input/output file | Shows warning, proceeds with normal `--ask` behavior |
| **Dangerous** | `rm`, `sudo`, `dd`, `mkfs`, `chmod`, `chown` commands | **Forces confirmation prompt** regardless of `--ask` |

Example output for a dangerous command:

```
────────────────────────────────────────────────────────────
  ⚠️  RISK: Dangerous command detected
  The generated command contains 'rm' which can delete files.
  Proceed with caution.
────────────────────────────────────────────────────────────

? Execute this command? [y/N]
```

This safety feature ensures that potentially destructive operations always require explicit user confirmation.

## Input File Validation

Before execution, oxo-call validates that input files exist on disk. Files following these flags are checked:

- `-i`, `--input`, `-I`, `--in`
- `-1`, `-2`, `--in1`, `--in2` (paired-end inputs)
- `-x`, `-U` (reference/index inputs)
- `--ref`, `--reference`, `--genome`, `--genome-dir`, `--genomeDir`
- `--sjdbGTFfile`, `--gtf`, `--bed`

If a specified input file doesn't exist, oxo-call fails early with a clear error message, preventing confusing downstream failures.

## Auto-Retry (`--auto-retry`)

When `--auto-retry` is enabled and the generated command fails, oxo-call automatically:

1. Captures the stderr and exit code
2. Sends the failure context to the LLM
3. Generates a corrected command
4. Executes the corrected command

Up to 2 retry attempts are made.  This is especially useful for complex tools where
the first LLM attempt may miss a required flag or get a parameter format wrong.

```
────────────────────────────────────────────────────────────
  ⟳ Analyzing failure and generating corrected command...
────────────────────────────────────────────────────────────
  Auto-retry: (attempt 1/2)
  Corrected command: samtools sort -@ 8 -o sorted.bam input.bam
  Fix: Added missing -o flag for output file
────────────────────────────────────────────────────────────

────────────────────────────────────────────────────────────
  ✓ Auto-retry succeeded on attempt 1
────────────────────────────────────────────────────────────
```

All retry attempts are recorded in the command history with an `[auto-retry #N]` prefix on the task description.

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
