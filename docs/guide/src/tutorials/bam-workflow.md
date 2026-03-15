# SAM/BAM Processing Tutorial

This tutorial walks through a complete SAM/BAM processing pipeline using `samtools`. By the end, you will have sorted, indexed, filtered, and summarized a BAM file — all with natural-language commands.

**Time to complete:** 20–30 minutes
**Prerequisites:** oxo-call configured, `samtools` installed
**You will learn:** multi-step BAM workflows, dry-run validation, docs enrichment

---

## Overview: The BAM Processing Pipeline

A typical BAM processing workflow looks like this:

```
raw alignment (.bam)
        │
   ▼ sort by coordinate
sorted.bam
        │
   ▼ index
sorted.bam.bai
        │
   ▼ filter (keep primary, mapped reads)
filtered.bam + filtered.bam.bai
        │
   ▼ statistics
flagstat.txt + idxstats.txt
```

We will use oxo-call at every step, demonstrating how natural-language commands map to correct samtools flags.

---

## Setting Up

### Check that samtools is available

```bash
samtools --version
# samtools 1.21
```

### Add rich documentation (optional)

oxo-call fetches `--help` automatically, but you can add the samtools manual for richer context:

```bash
oxo-call docs add samtools --url https://www.htslib.org/doc/samtools.html
```

Verify the documentation is indexed:

```bash
oxo-call docs show samtools | head -30
```

### Check the built-in skill

```bash
oxo-call skill show samtools
```

The skill contains concepts like "BAM MUST be coordinate-sorted BEFORE indexing" — this knowledge guides the LLM to generate the right flags in the right order.

---

## Step 1: Sort by Coordinate

Most downstream tools require a coordinate-sorted BAM. Let us generate the command:

```bash
oxo-call dry-run samtools "sort aligned.bam by coordinate using 4 threads and output to sorted.bam"
```

Expected output:

```
Command: samtools sort -@ 4 -o sorted.bam aligned.bam
Explanation: -@ 4 uses 4 worker threads; -o specifies the output file; coordinate sort is the default.
```

Now execute it:

```bash
oxo-call run samtools "sort aligned.bam by coordinate using 4 threads and output to sorted.bam"
```

> **Why does `-@ 4` appear?**  
> The samtools skill includes the concept: *"Use -@ N for N additional threads."* This guides the LLM to include threading flags whenever you mention thread count.

---

## Step 2: Index the Sorted BAM

Indexing is required for random-access queries and most genome browsers:

```bash
oxo-call dry-run samtools "create a BAI index for sorted.bam"
# → samtools index sorted.bam
```

Execute:

```bash
oxo-call run samtools "create a BAI index for sorted.bam"
```

This creates `sorted.bam.bai`.

> **Note:** The skill reminds the LLM that you **must sort before indexing**. If you try to index an unsorted BAM, samtools will error. Try running `oxo-call dry-run samtools "index unsorted.bam"` — the explanation will mention sorting as a prerequisite.

---

## Step 3: Filter to Primary Mapped Reads

Remove secondary alignments, supplementary alignments, and unmapped reads:

```bash
oxo-call dry-run samtools \
  "extract only primary mapped reads from sorted.bam into filtered.bam, then index filtered.bam"
```

Expected:

```
Command: samtools view -F 0x904 -b -o filtered.bam sorted.bam && samtools index filtered.bam
Explanation: -F 0x904 excludes unmapped (0x4), secondary (0x100), and supplementary (0x800) reads; -b writes BAM format.
```

Run it:

```bash
oxo-call run samtools \
  "extract only primary mapped reads from sorted.bam into filtered.bam, then index filtered.bam"
```

> **Understanding `-F 0x904`:**  
> `0x904 = 0x4 | 0x100 | 0x800` — this bitmask excludes unmapped, secondary, and supplementary alignments. The samtools skill includes this as a common pitfall: *"Use -F 0x904 to keep only primary mapped reads."* Without the skill, the LLM might use a less complete flag set.

---

## Step 4: Check Alignment Statistics

Get a summary of the alignment quality:

```bash
oxo-call dry-run samtools "show alignment statistics for filtered.bam and save to flagstat.txt"
# → samtools flagstat filtered.bam > flagstat.txt
```

```bash
oxo-call run samtools "show alignment statistics for filtered.bam and save to flagstat.txt"
cat flagstat.txt
```

Example output:

```
10234567 + 0 in total (QC-passed reads + QC-failed reads)
0 + 0 secondary
0 + 0 supplementary
0 + 0 duplicates
10234567 + 0 mapped (100.00% : N/A)
10234567 + 0 paired in sequencing
5117284 + 0 read1
5117283 + 0 read2
...
```

Get per-chromosome read counts:

```bash
oxo-call dry-run samtools "show per-chromosome read counts for filtered.bam and save to idxstats.txt"
# → samtools idxstats filtered.bam > idxstats.txt
```

```bash
oxo-call run samtools "show per-chromosome read counts for filtered.bam and save to idxstats.txt"
```

---

## Step 5: Mark Duplicates (Optional)

For most variant-calling pipelines, duplicate reads should be marked:

```bash
oxo-call dry-run picard \
  "mark duplicates in filtered.bam, output to dedup.bam, write metrics to dedup_metrics.txt"
```

Expected:

```
Command: picard MarkDuplicates I=filtered.bam O=dedup.bam M=dedup_metrics.txt
Explanation: I/O are input/output; M writes duplication metrics; duplicates are marked not removed by default.
```

> **Picard vs samtools markdup?**  
> Ask oxo-call to explain: `oxo-call dry-run samtools "mark duplicate reads in filtered.bam"` will generate `samtools markdup`. Both approaches are valid; the choice depends on your pipeline. Try both dry-runs to compare.

---

## Reviewing the Full Pipeline

Look at everything that ran:

```bash
oxo-call history list --tool samtools
```

You will see each command with its exit code and provenance metadata, giving you a complete audit trail of the BAM processing steps.

---

## Putting It Together with a Workflow

All these steps can be automated with the native workflow engine. See the [Workflow Builder tutorial](./workflow-builder.md) to learn how to turn this pipeline into a reproducible `.oxo.toml` file.

---

## What You Learned

- How to run a multi-step BAM processing pipeline with oxo-call
- Why skills matter — `samtools` skill provides `sort-before-index` and bitmask guidance
- How to enrich documentation with remote URLs
- How to review full execution history with provenance
