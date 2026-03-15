# Your First Bioinformatics Command

This tutorial walks you through using oxo-call for the very first time, from a blank terminal to a successfully executed bioinformatics command. No prior experience with the tool is required — just a working installation and a license file.

**Time to complete:** 10–15 minutes
**Prerequisites:** oxo-call installed, license configured, at least one LLM provider set up
**You will learn:** how to preview commands, execute them, and review your history

---

## What We Will Do

We will use `samtools` — one of the most common tools in bioinformatics — to:

1. Preview a sort command without running it (dry-run)
2. Run the command for real
3. Review what was executed in history

If you do not have `samtools` installed, you can still follow steps 1 and 3 using dry-run and the `docs add` command.

---

## Step 1: Verify your setup

Before running anything, confirm oxo-call is ready:

```bash
oxo-call --version
oxo-call license verify
oxo-call config verify
```

Expected output from `config verify`:

```
✓ LLM provider: github-copilot
✓ API token: configured
✓ Connection: OK
```

If `config verify` fails, go back to the [Configuration guide](./configuration.md) and [License Setup](./license.md).

---

## Step 2: Preview your first command (dry-run)

The `dry-run` command asks the LLM to generate the right flags **but does not execute anything**. This is the safest way to start.

```bash
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
```

What happens behind the scenes:

1. oxo-call runs `samtools --help` and caches the output (first time only)
2. The built-in `samtools` skill injects expert knowledge into the prompt
3. The LLM receives: task + docs + skill, and returns the correct flags
4. The command is printed for you to inspect — nothing is executed

Expected output:

```
Command: samtools sort -o sorted.bam input.bam
Explanation: Uses -o to specify the output file; coordinate sort is the default behavior.
```

> **Tip:** The `-o` flag is easy to forget with samtools. The skill knows this pitfall and guides the LLM to always include it.

---

## Step 3: Run the command for real

Once you are happy with the dry-run output, use `run` to execute it. Replace `input.bam` with a real BAM file on your system:

```bash
oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"
```

oxo-call will generate the same command as dry-run, execute it, and print the result:

```
→ samtools sort -o sorted.bam input.bam
[M::bam_sort_core] merging from 0 files and 4 in-memory blocks...
Exit code: 0
```

### Want to confirm before running?

Use `--ask` to get a confirmation prompt before execution:

```bash
oxo-call run --ask samtools "sort input.bam by coordinate and output to sorted.bam"
```

```
Command: samtools sort -o sorted.bam input.bam
Explanation: ...
Execute? [y/N] y
```

This is especially useful for destructive or long-running commands.

---

## Step 4: Review your history

Every command run by oxo-call is logged automatically:

```bash
oxo-call history list
```

Output:

```
ok    samtools   0   2025-06-01 12:00:00   samtools sort -o sorted.bam input.bam
      [ver=samtools 1.21, model=gpt-4o-mini, skill=samtools, docs=a1b2c3d4]
```

Each history entry includes:

- **Status**: `ok` (exit 0) or `err` (non-zero exit)
- **Tool**: the CLI tool that ran
- **Exit code**: process exit code
- **Timestamp**: when it executed
- **Command**: the full generated command
- **Provenance**: tool version, model, skill used, docs hash

To filter by tool:

```bash
oxo-call history list --tool samtools
```

---

## Step 5: Try more commands

Now that you know the basic pattern, try a few more:

```bash
# Index a sorted BAM file
oxo-call dry-run samtools "create an index for sorted.bam"
# → samtools index sorted.bam

# View only mapped reads
oxo-call dry-run samtools "extract only mapped reads from aligned.bam into mapped.bam"
# → samtools view -F 4 -b -o mapped.bam aligned.bam

# Check alignment statistics
oxo-call dry-run samtools "show alignment statistics for sorted.bam"
# → samtools flagstat sorted.bam

# Count reads in a region
oxo-call dry-run samtools "count reads mapping to chromosome 1 between 1000000 and 2000000"
# → samtools view -c sorted.bam chr1:1000000-2000000
```

Each of these uses the same pattern: describe what you want in plain English.

---

## What You Learned

- `oxo-call dry-run <tool> "<task>"` — preview any command safely
- `oxo-call run <tool> "<task>"` — execute the command immediately
- `oxo-call run --ask <tool> "<task>"` — confirm before executing
- `oxo-call history list` — review all past commands with provenance

**Next:** try the [SAM/BAM processing tutorial](./bam-workflow.md) for a complete multi-step workflow, or jump to [RNA-seq walkthrough](./rnaseq-walkthrough.md) for a full analysis pipeline.
