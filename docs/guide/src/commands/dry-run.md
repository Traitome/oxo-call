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
| `-v`, `--verbose` | Show docs source, skill info, and LLM details (global) |
| `--license <PATH>` | Path to license file (global option) |

## Description

`dry-run` follows the same pipeline as `run` (documentation fetch → skill loading → LLM generation) but prints the resulting command instead of executing it. Use this to:

- Preview commands before running them
- Verify oxo-call understands your intent
- Generate commands to copy into scripts
- Test with tools that aren't installed locally
- Produce JSON output for pipeline integration

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
  "dry_run": true,
  "skill": "samtools",
  "model": "gpt-4o"
}
```
