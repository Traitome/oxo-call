# How-to: Create a Custom Skill

This guide shows you how to write a custom skill file that teaches oxo-call expert knowledge about a specific tool. Skills dramatically improve LLM accuracy by injecting curated concepts, pitfalls, and worked examples into the prompt.

---

## When Should You Create a Custom Skill?

Create a skill when:
- You are using a tool not in the built-in library (check with `oxo-call skill list`)
- The built-in skill for a tool is missing important domain-specific knowledge
- You have institutional conventions or preferred parameter sets you want enforced
- You want to share best-practice guidance with your team

---

## Skill File Structure

A skill file is a TOML file with three sections:

```toml
[meta]
name     = "mytool"        # must match the binary name exactly
category = "alignment"     # domain category
tags     = ["bam", "ngs"]  # searchable tags

[context]
concepts = [
    "Concept 1: fundamental knowledge about this tool",
    "Concept 2: key behavior to understand",
    "Concept 3: important flag semantics",
]

pitfalls = [
    "Common mistake 1 — and how to avoid it",
    "Common mistake 2 — and how to avoid it",
    "Common mistake 3 — and how to avoid it",
]

[[examples]]
task        = "a plain English description of a task"
args        = "mytool --flag1 value1 --flag2 value2 input output"
explanation = "why these flags were chosen"

[[examples]]
task        = "another task description"
args        = "mytool --other-flags input"
explanation = "explanation of the flags"
```

**Minimum requirements for a valid skill:**
- At least 3 concepts
- At least 3 pitfalls
- At least 5 examples

---

## Step 1: Generate a Template

```bash
oxo-call skill create mytool -o ~/.config/oxo-call/skills/mytool.toml
```

This creates a TOML template with placeholder content at the user skills directory.

---

## Step 2: Edit the Template

Open the file and fill in real content. Here is a complete example for `kallisto`:

```toml
[meta]
name     = "kallisto"
category = "rna-seq"
tags     = ["rna-seq", "quantification", "pseudoalignment", "transcript"]

[context]
concepts = [
    "kallisto uses pseudoalignment — it does not produce a BAM file; output is abundance.tsv, abundance.h5, and run_info.json",
    "Build a transcriptome index with 'kallisto index', NOT a genome index — use cDNA FASTA, not genomic FASTA",
    "The -b flag sets bootstrap samples for uncertainty estimation; 0 = no bootstraps; 100 is typical for differential expression",
    "Use --single for single-end reads with --fragment-length and --sd required",
    "Output directory must be specified with -o; kallisto will not create nested directories",
]

pitfalls = [
    "Using a genome FASTA instead of a transcriptome FASTA for the index — use Ensembl cDNA or GENCODE transcripts FASTA",
    "Forgetting -b for bootstraps when using downstream tools like sleuth that require uncertainty estimates",
    "Using --single without --fragment-length and --sd — both are required for single-end quantification",
    "Not specifying -o — output goes to current directory and may overwrite previous runs",
    "Using the wrong strand flag — check library preparation: --fr-stranded or --rf-stranded for stranded libraries",
]

[[examples]]
task        = "build a transcriptome index from human cDNA FASTA"
args        = "index -i hg38_transcriptome.idx gencode.v44.pc_transcripts.fa"
explanation = "-i specifies the output index filename; input is the transcriptome FASTA (not genome)"

[[examples]]
task        = "quantify paired-end RNA-seq reads with 100 bootstraps"
args        = "quant -i hg38_transcriptome.idx -o quant/sample1 -b 100 sample1_R1.fq.gz sample1_R2.fq.gz"
explanation = "-i is the index; -o sets output directory; -b 100 generates 100 bootstraps for sleuth"

[[examples]]
task        = "quantify single-end reads with 150bp mean fragment length"
args        = "quant -i hg38_transcriptome.idx -o quant/sample1 --single -l 150 -s 20 sample1.fq.gz"
explanation = "--single enables single-end mode; -l is mean fragment length; -s is standard deviation"

[[examples]]
task        = "quantify stranded paired-end reads (forward-reverse orientation)"
args        = "quant -i hg38_transcriptome.idx -o quant/sample1 --fr-stranded -b 50 R1.fq.gz R2.fq.gz"
explanation = "--fr-stranded for dUTP-based stranded library; check your protocol documentation"

[[examples]]
task        = "inspect the transcriptome index"
args        = "inspect hg38_transcriptome.idx"
explanation = "prints index statistics: number of k-mers, transcripts, and k-mer length"
```

---

## Step 3: Verify the Skill Loads

```bash
oxo-call skill show kallisto
```

If you see the skill content, it is working. If not, check:
- File is at `~/.config/oxo-call/skills/kallisto.toml`
- Filename matches the binary name exactly
- TOML syntax is valid (use `toml-validator` if needed)

---

## Step 4: Test the Skill

Compare dry-run output with and without the skill:

```bash
# With skill (user skill takes priority)
oxo-call dry-run kallisto "quantify paired-end reads R1.fq R2.fq against human transcriptome"

# Temporarily disable by moving the skill file
mv ~/.config/oxo-call/skills/kallisto.toml /tmp/
oxo-call dry-run kallisto "quantify paired-end reads R1.fq R2.fq against human transcriptome"
mv /tmp/kallisto.toml ~/.config/oxo-call/skills/kallisto.toml
```

Compare the outputs — the skill-augmented version should include the correct index path pattern and bootstrap flag.

---

## Writing Good Skills

### Concepts: fundamental knowledge

Concepts should be facts that the LLM needs to understand to use the tool correctly, but that might not be obvious from `--help` output alone.

```toml
# ✓ GOOD: explains non-obvious behavior
"BAM files MUST be coordinate-sorted before indexing with samtools index"

# ✗ BAD: restates what --help already says
"Use -o to specify the output file"
```

### Pitfalls: common mistakes

Pitfalls should be mistakes that users (or LLMs) commonly make, with the consequence explained.

```toml
# ✓ GOOD: specific mistake + consequence
"Using --gtf with a genome FASTA (not transcriptome) for kallisto index — will produce incorrect k-mer counts"

# ✗ BAD: vague
"Be careful with input files"
```

### Examples: task → args mappings

Examples should cover the most common real-world tasks in your domain. The `args` field is the actual command-line flags (without the tool name prefix).

```toml
# ✓ GOOD: complete, realistic task
task        = "align paired-end reads to hg38, output sorted BAM, 8 threads"
args        = "mem -t 8 hg38.fa R1.fq.gz R2.fq.gz | samtools sort -o aligned.bam"
explanation = "pipes to samtools sort to avoid intermediate SAM file"

# ✗ BAD: too generic
task = "align reads"
args = "mem ref.fa reads.fq"
```

---

## Sharing Skills with Your Team

You can install a skill from a URL:

```bash
# Install from a URL (shared within your organization)
oxo-call skill install kallisto --url https://your-org.example.com/skills/kallisto.toml

# Others on your team can do the same
oxo-call skill install kallisto --url https://your-org.example.com/skills/kallisto.toml
```

---

## Skill Precedence

When oxo-call finds multiple skill sources:

1. **User-defined** (`~/.config/oxo-call/skills/`) — highest priority
2. **Community-installed** (`~/.local/share/oxo-call/skills/`)
3. **Built-in** (compiled into the binary)

Your custom skill always wins over the built-in.

---

## Related

- [Skill System reference](../reference/skill-system.md) — full format specification
- [skill command reference](../commands/skill.md) — all subcommands
- [Contributing built-in skills](../development/contributing.md) — contribute to the project
