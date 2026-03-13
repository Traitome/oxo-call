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
| `-v`, `--verbose` | Show docs source, skill info, and LLM details (global) |
| `--license <PATH>` | Path to license file (global option) |

## Description

The `run` command is the primary way to use oxo-call. It:

1. Fetches the tool's documentation (from cache or `--help` output)
2. Loads any matching skill (built-in, community, or user-defined)
3. Sends the grounded prompt to the configured LLM
4. Parses the response to extract command arguments
5. Executes the tool with the generated arguments
6. Records the execution in command history

## Examples

```bash
# Basic usage
oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"

# With confirmation prompt
oxo-call run --ask bcftools "call variants from aligned.bam using ref.fa"

# Complex multi-step task
oxo-call run gatk "run HaplotypeCaller on sample.bam with reference hg38.fa, output to variants.g.vcf in GVCF mode"

# Using short alias
oxo-call r bwa "align reads.fq to ref.fa with 8 threads"

# Override LLM model for a single invocation
oxo-call run --model gpt-4 samtools "index sorted.bam"

# Force fresh documentation (skip cache)
oxo-call run --no-cache samtools "sort by name"

# Get JSON output for scripting
oxo-call run --json samtools "flagstat input.bam"

# Verbose mode shows what's happening under the hood
oxo-call run --verbose samtools "sort input.bam"
```

## JSON Output

When `--json` is used, the output is a JSON object:

```json
{
  "tool": "samtools",
  "task": "flagstat input.bam",
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

## Behavior

- Documentation is fetched automatically on first use and cached
- If a matching skill exists, expert knowledge is injected into the prompt
- The LLM response must contain `ARGS:` and `EXPLANATION:` lines
- On execution failure, the exit code is recorded in history
- Use `dry-run` to preview commands without executing
- Use `--no-cache` to force a fresh `--help` fetch when docs may be stale
- Use `--model` to quickly test different models without changing config
