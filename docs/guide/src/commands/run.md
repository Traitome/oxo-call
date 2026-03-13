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
| `-v`, `--verbose` | Show docs source, skill info, and LLM details (global) |
| `--license <PATH>` | Path to license file (global option) |

## Description

The `run` command is the primary way to use oxo-call. It:

1. Fetches the tool's documentation (from cache or `--help` output)
2. *(with `--optimize-task`)* Expands and clarifies your task description with the LLM
3. Loads any matching skill (built-in, community, or user-defined)
4. Sends the grounded prompt to the configured LLM
5. Parses the response to extract command arguments
6. Executes the tool with the generated arguments
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

When `--json` is used, the output is a JSON object:

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
