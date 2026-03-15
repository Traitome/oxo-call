# RNA-seq Analysis Walkthrough

This tutorial walks through a complete bulk RNA-seq analysis pipeline — from raw FASTQ reads to a count matrix — using oxo-call to generate every command. This mirrors a real-world analysis workflow.

**Time to complete:** 30–45 minutes (reading) + compute time
**Prerequisites:** oxo-call configured, tools installed: `fastp`, `STAR`, `featureCounts` (or `salmon`)
**You will learn:** end-to-end pipeline construction, workflow integration, skill-driven accuracy

---

## RNA-seq Pipeline Overview

```
Raw FASTQ reads
       │
  ▼ Quality Control (fastp)
Trimmed FASTQ + QC report
       │
  ▼ Alignment (STAR)
Aligned BAM (coordinate-sorted)
       │
  ▼ QC Aggregation (MultiQC)
Combined QC report
       │
  ▼ Quantification (featureCounts)
Count matrix (gene × sample)
```

We will use oxo-call at each step. The commands shown use example filenames — adapt them to your data.

---

## Setup

### Sample data assumptions

This tutorial assumes:

- Paired-end RNA-seq data: `sample1_R1.fastq.gz`, `sample1_R2.fastq.gz`
- STAR genome index at: `/data/star_hg38/`
- GTF annotation at: `/data/gencode.v44.gtf`

### Verify tools and skills

```bash
# Check tools are installed
fastp --version
STAR --version
featureCounts -v

# Check built-in skills
oxo-call skill list | grep -E "fastp|star|featurecounts"
# fastp        ✓ built-in
# star         ✓ built-in
# featurecounts ✓ built-in
```

### Add remote documentation (optional but recommended)

```bash
oxo-call docs add fastp --url https://github.com/OpenGene/fastp#usage
oxo-call docs add star --url https://physiology.med.cornell.edu/faculty/skrabanek/lab/angsd/lecture_notes/STARmanual.pdf
```

---

## Step 1: Quality Control with fastp

fastp trims adapters, removes low-quality reads, and generates a QC report in one step.

### Dry-run first

```bash
oxo-call dry-run fastp \
  "trim adapters from paired-end reads sample1_R1.fastq.gz and sample1_R2.fastq.gz, \
   output trimmed reads to trimmed/sample1_R1.fastq.gz and trimmed/sample1_R2.fastq.gz, \
   use 4 threads, generate HTML report at qc/sample1.html and JSON at qc/sample1.json"
```

Expected:

```
Command: fastp \
  --in1 sample1_R1.fastq.gz --in2 sample1_R2.fastq.gz \
  --out1 trimmed/sample1_R1.fastq.gz --out2 trimmed/sample1_R2.fastq.gz \
  --thread 4 \
  --html qc/sample1.html --json qc/sample1.json
Explanation: fastp auto-detects adapters; --thread 4 for parallelism; HTML/JSON for MultiQC compatibility.
```

### Execute

```bash
mkdir -p trimmed qc
oxo-call run fastp \
  "trim adapters from paired-end reads sample1_R1.fastq.gz and sample1_R2.fastq.gz, \
   output trimmed reads to trimmed/sample1_R1.fastq.gz and trimmed/sample1_R2.fastq.gz, \
   use 4 threads, generate HTML report at qc/sample1.html and JSON at qc/sample1.json"
```

> **Why no `--adapter_sequence`?**  
> The fastp skill includes the concept: *"fastp auto-detects adapters — do not hardcode adapter sequences unless they are being missed."* This is a common mistake when migrating from trimmomatic.

---

## Step 2: Alignment with STAR

STAR is the gold-standard splice-aware aligner for RNA-seq. It requires a pre-built genome index.

### Build the index (if you have not already)

```bash
oxo-call dry-run STAR \
  "build a genome index for hg38, genome fasta at /data/hg38.fa, \
   GTF at /data/gencode.v44.gtf, output to /data/star_hg38, use 8 threads, \
   sjdbOverhang 100 (read length minus 1)"
```

Expected:

```
Command: STAR \
  --runMode genomeGenerate \
  --genomeDir /data/star_hg38 \
  --genomeFastaFiles /data/hg38.fa \
  --sjdbGTFfile /data/gencode.v44.gtf \
  --sjdbOverhang 100 \
  --runThreadN 8
```

### Align trimmed reads

```bash
oxo-call dry-run STAR \
  "align trimmed paired-end reads trimmed/sample1_R1.fastq.gz and trimmed/sample1_R2.fastq.gz \
   to genome index at /data/star_hg38, output to aligned/sample1/, \
   sort BAM by coordinate, use 8 threads, generate a BAM file"
```

Expected:

```
Command: STAR \
  --genomeDir /data/star_hg38 \
  --readFilesIn trimmed/sample1_R1.fastq.gz trimmed/sample1_R2.fastq.gz \
  --readFilesCommand zcat \
  --outSAMtype BAM SortedByCoordinate \
  --outFileNamePrefix aligned/sample1/ \
  --runThreadN 8
Explanation: --readFilesCommand zcat handles .gz files; SortedByCoordinate produces a ready-to-use BAM.
```

> **The `--readFilesCommand zcat` pitfall:**  
> Forgetting this flag when using `.gz` input is one of the most common STAR mistakes. The STAR skill includes it as a pitfall: *"Always add `--readFilesCommand zcat` for compressed FASTQ."*

Execute:

```bash
mkdir -p aligned/sample1
oxo-call run STAR \
  "align trimmed paired-end reads trimmed/sample1_R1.fastq.gz and trimmed/sample1_R2.fastq.gz \
   to genome index at /data/star_hg38, output to aligned/sample1/, \
   sort BAM by coordinate, use 8 threads, generate a BAM file"
```

> **STAR Two-Pass Mode:**  
> For novel splice junction discovery (e.g., tumor RNA-seq, rare transcripts), consider using STAR's two-pass mode. In the first pass, STAR discovers splice junctions; in the second pass, it re-maps reads using the discovered junctions for improved sensitivity. Add `--twopassMode Basic` to your alignment task description. Note that two-pass mode increases runtime and memory usage. For standard differential expression analyses with well-annotated genomes (e.g., human, mouse), one-pass mode with a comprehensive GTF annotation is usually sufficient.

### Index the BAM

```bash
oxo-call run samtools "index aligned/sample1/Aligned.sortedByCoord.out.bam"
```

---

## Step 3: Aggregate QC with MultiQC

MultiQC reads all QC outputs and creates a single interactive report. It should be run after all samples are processed.

```bash
oxo-call dry-run multiqc \
  "aggregate QC reports from qc/ and STAR log files from aligned/ into a single report at results/multiqc/"
```

Expected:

```
Command: multiqc qc/ aligned/ -o results/multiqc/
Explanation: multiqc automatically discovers fastp JSON reports and STAR Log.final.out files in the specified directories.
```

```bash
mkdir -p results/multiqc
oxo-call run multiqc \
  "aggregate QC reports from qc/ and STAR log files from aligned/ into a single report at results/multiqc/"
```

Open `results/multiqc/multiqc_report.html` in a browser to review:

- Per-sample read quality before/after trimming
- Alignment rates
- Duplication rates
- Any samples that need attention

---

## Step 4: Quantification with featureCounts

featureCounts counts reads per gene. It is part of the `subread` package.

```bash
oxo-call dry-run featureCounts \
  "count reads per gene in aligned/sample1/Aligned.sortedByCoord.out.bam \
   using GTF at /data/gencode.v44.gtf, for paired-end data, \
   use 4 threads, output to results/counts.txt"
```

Expected:

```
Command: featureCounts \
  -a /data/gencode.v44.gtf \
  -o results/counts.txt \
  -p \
  -T 4 \
  aligned/sample1/Aligned.sortedByCoord.out.bam
Explanation: -p specifies paired-end mode; -a is the GTF annotation; -T sets threads.
```

> **Using Salmon for quantification instead?**  
> featureCounts works on aligned BAMs (alignment-based). Salmon uses pseudo-alignment directly on FASTQ files (alignment-free) and is faster for many use cases. Try:  
> `oxo-call dry-run salmon "quantify sample1_R1.fastq.gz and sample1_R2.fastq.gz against index at /data/salmon_hg38_index, output to quant/sample1/"`

Execute:

```bash
mkdir -p results
oxo-call run featureCounts \
  "count reads per gene in aligned/sample1/Aligned.sortedByCoord.out.bam \
   using GTF at /data/gencode.v44.gtf, for paired-end data, \
   use 4 threads, output to results/counts.txt"
```

---

## Step 5: Running Multiple Samples

For a real experiment you will have many samples. Instead of repeating commands manually, use the workflow engine:

```bash
# View the built-in RNA-seq template
oxo-call workflow show rnaseq

# Dry-run the complete pipeline
oxo-call workflow dry-run rnaseq
```

To customize for your samples and paths, see the [Workflow Builder tutorial](./workflow-builder.md).

---

## Checking the Full Run History

```bash
oxo-call history list | head -20
```

This shows every command in the order it was executed, with exit codes, timestamps, and provenance metadata.

---

## What You Learned

- How to run a complete RNA-seq pipeline step-by-step with oxo-call
- How built-in skills prevent common mistakes (`--readFilesCommand zcat`, `-p` for paired-end)
- How to add remote documentation for richer LLM context
- How MultiQC integrates naturally into the oxo-call workflow
- Where to go next: the workflow engine for multi-sample automation

**Next steps:**
- [Workflow Builder tutorial](./workflow-builder.md) — automate this for multiple samples
- [featureCounts command reference](../commands/run.md) — full options
- [Skill System reference](../reference/skill-system.md) — how skills improve accuracy
