# dry-run

Generate parameters and print the command without executing.

## Synopsis

```
oxo-call dry-run [OPTIONS] <TOOL> <TASK>
oxo-call d       [OPTIONS] <TOOL> <TASK>
```

## Options

| Option | Description |
|--------|-------------|
| `-m`, `--model <MODEL>` | Override the LLM model for this invocation |
| `--no-cache` | Skip cached documentation and fetch fresh `--help` output |
| `--json` | Output result as JSON (useful for scripting and CI) |
| `-V`, `--var KEY=VALUE` | Substitute `{KEY}` in the task description before the LLM call (repeatable) |
| `-i`, `--input-list <FILE>` | Read input items from a file; shows the command for each item |
| `--input-items <ITEMS>` | Comma-separated input items; shows the command for each item |
| `--scenario <SCENARIO>` | Force a workflow scenario: `basic`, `prompt`, `doc`, `skill`, or `full` (auto-detected by default) |
| `-v`, `--verbose` | Show docs source, skill info, and LLM details (global) |
| `--license <PATH>` | Path to license file (global option) |

### Prompt Tier

The prompt compression tier is auto-detected from the model size and context
window. You can override it with the `llm.prompt_tier` config key or the
`OXO_CALL_LLM_PROMPT_TIER` environment variable:

```bash
# Force Compact tier for a small model
OXO_CALL_LLM_PROMPT_TIER=compact oxo-call dry-run samtools "sort bam"

# Force Medium tier
OXO_CALL_LLM_PROMPT_TIER=medium oxo-call dry-run samtools "sort bam"
```

Use `--verbose` to see which tier was selected for a given invocation.

## Description

`dry-run` follows the same pipeline as `run` (documentation fetch → skill loading → LLM generation) but prints the resulting command instead of executing it. Use this to:

- Preview commands before running them
- Verify oxo-call understands your intent
- Generate commands to copy into scripts
- Test with tools that aren't installed locally
- Produce JSON output for pipeline integration
- Preview batch command expansions before a real run

## Examples

```bash
# Preview a samtools command
oxo-call dry-run samtools "view only primary alignments from file.bam"
# → samtools view -F 0x904 file.bam

# Preview a complex alignment
oxo-call d bwa "align paired reads R1.fq R2.fq to hg38.fa using 16 threads"
# → bwa mem -t 16 hg38.fa R1.fq R2.fq

# Use a specific model
oxo-call dry-run --model gpt-4 gatk "call variants on sample.bam"

# Get JSON output for scripting
oxo-call dry-run --json samtools "flagstat input.bam"

# Force fresh documentation
oxo-call dry-run --no-cache samtools "sort input.bam"

# Variable substitution — preview with {SAMPLE} replaced
oxo-call dry-run --var SAMPLE=NA12878 samtools \
    "sort {SAMPLE}.bam by coordinate"

# Preview all batch commands that would run
oxo-call dry-run samtools "flagstat {item}" \
    --input-items s1.bam,s2.bam,s3.bam

# Preview from a file list
oxo-call dry-run samtools "sort {item} by coordinate, output {stem}.sorted.bam" \
    --input-list bam_files.txt
```

## Automatic Task Normalization

oxo-call automatically detects vague, short, or non-English task descriptions and normalizes them via an extra LLM call before command generation. The normalized task is shown when it differs from the original, and is used as the actual prompt for the LLM.

## Batch Preview (`--input-list` / `--input-items`)

When input items are provided, `dry-run` calls the LLM once to generate a
command template, then prints the interpolated command for **each item** without
executing any of them. This lets you verify the expansion before committing to a
real batch run.

Supported placeholders in the task / generated command:

| Placeholder | Expands to |
|-------------|-----------|
| `{item}` / `{line}` | The current input item |
| `{nr}` | 1-based item number |
| `{basename}` | Filename without directory |
| `{dir}` | Directory portion (or `.`) |
| `{stem}` | Filename without last extension |
| `{ext}` | File extension without dot |

## JSON Output

When `--json` is used for a single-item dry-run, the output is a JSON object:

```json
{
  "tool": "samtools",
  "task": "flagstat input.bam",
  "effective_task": "flagstat input.bam",
  "command": "samtools flagstat input.bam",
  "args": ["flagstat", "input.bam"],
  "explanation": "Generates alignment statistics for the BAM file",
  "inference_ms": 342.5,
  "dry_run": true,
  "skill": "samtools",
  "model": "gpt-4o"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `tool` | string | Tool name |
| `task` | string | Original task description |
| `effective_task` | string | Task after automatic normalization (same as `task` if unchanged) |
| `command` | string | Full shell command (tool + args) |
| `args` | array | Argument tokens (without tool name prefix) |
| `explanation` | string | LLM-generated explanation |
| `inference_ms` | number | Time (ms) spent in LLM API inference (sum of all retries) |
| `dry_run` | boolean | Always `true` |
| `skill` | string/null | Matched skill file name |
| `model` | string | LLM model used |

For batch dry-run (`--input-list` / `--input-items`) with `--json`:

```json
{
  "tool": "samtools",
  "task_template": "flagstat {item}",
  "command_template": "samtools flagstat {item}",
  "commands": [
    "samtools flagstat s1.bam",
    "samtools flagstat s2.bam"
  ],
  "dry_run": true,
  "skill": "samtools",
  "model": "gpt-4o"
}
```


