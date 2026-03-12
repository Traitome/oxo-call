# dry-run

Generate parameters and print the command without executing.

## Synopsis

```
oxo-call dry-run <TOOL> <TASK>
oxo-call d       <TOOL> <TASK>
```

## Description

`dry-run` follows the same pipeline as `run` (documentation fetch → skill loading → LLM generation) but prints the resulting command instead of executing it. Use this to:

- Preview commands before running them
- Verify oxo-call understands your intent
- Generate commands to copy into scripts
- Test with tools that aren't installed locally

## Examples

```bash
# Preview a samtools command
oxo-call dry-run samtools "view only primary alignments from file.bam"
# → samtools view -F 0x904 file.bam

# Preview a complex alignment
oxo-call d bwa "align paired reads R1.fq R2.fq to hg38.fa using 16 threads"
# → bwa mem -t 16 hg38.fa R1.fq R2.fq
```
