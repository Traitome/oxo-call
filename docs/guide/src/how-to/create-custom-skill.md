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

## Skill File Format

A skill is a **Markdown file** (`.md`) with a YAML front-matter block followed by Markdown sections:

```markdown
---
name: mytool
category: alignment     # domain category
description: One-line summary of what this tool does
tags: [bam, ngs]        # searchable tags
author: your-name       # optional
source_url: https://...  # link to tool docs (optional)
---

## Concepts

- Concept 1: fundamental knowledge about this tool
- Concept 2: key behavior to understand
- Concept 3: important flag semantics

## Pitfalls

- Common mistake 1 — and how to avoid it
- Common mistake 2 — and how to avoid it
- Common mistake 3 — and how to avoid it

## Examples

### a plain English description of a task
**Args:** `--flag1 value1 --flag2 value2 input output`
**Explanation:** why these flags were chosen

### another task description
**Args:** `--other-flags input`
**Explanation:** explanation of the flags
```

**Minimum requirements for a valid skill:**
- At least 3 concepts
- At least 3 pitfalls
- At least 5 examples

> **Format rules for examples:**
> - The task goes in a level-3 heading (`### task description`)
> - Args are on the next line as `**Args:** \`command flags here\`` (backtick-wrapped)
> - Explanation is on the following line as `**Explanation:** text`
> - `args` should contain only the arguments, **not** the tool name itself



## Step 1: Generate a Template

```bash
oxo-call skill create mytool -o ~/.config/oxo-call/skills/mytool.md
```

This creates a Markdown template with placeholder content at the user skills directory.

---

## Step 2: Edit the Template

Open the file and fill in real content. Here is a complete example for `kallisto`:

```markdown
---
name: kallisto
category: rna-seq
description: Near-optimal RNA-seq quantification by pseudoalignment
tags: [rna-seq, quantification, pseudoalignment, transcript]
author: oxo-call built-in
source_url: https://pachterlab.github.io/kallisto/
---

## Concepts

- kallisto uses pseudoalignment — it does not produce a BAM file; output is abundance.tsv, abundance.h5, and run_info.json
- Build a transcriptome index with 'kallisto index', NOT a genome index — use cDNA FASTA, not genomic FASTA
- The -b flag sets bootstrap samples for uncertainty estimation; 0 = no bootstraps; 100 is typical for differential expression
- Use --single for single-end reads with --fragment-length and --sd required
- Output directory must be specified with -o; kallisto will not create nested directories

## Pitfalls

- Using a genome FASTA instead of a transcriptome FASTA for the index — use Ensembl cDNA or GENCODE transcripts FASTA
- Forgetting -b for bootstraps when using downstream tools like sleuth that require uncertainty estimates
- Using --single without --fragment-length and --sd — both are required for single-end quantification
- Not specifying -o — output goes to current directory and may overwrite previous runs
- Using the wrong strand flag — check library preparation: --fr-stranded or --rf-stranded for stranded libraries

## Examples

### build a transcriptome index from human cDNA FASTA
**Args:** `index -i hg38_transcriptome.idx gencode.v44.pc_transcripts.fa`
**Explanation:** -i specifies the output index filename; input is the transcriptome FASTA (not genome)

### quantify paired-end RNA-seq reads with 100 bootstraps
**Args:** `quant -i hg38_transcriptome.idx -o quant/sample1 -b 100 sample1_R1.fq.gz sample1_R2.fq.gz`
**Explanation:** -i is the index; -o sets output directory; -b 100 generates 100 bootstraps for sleuth

### quantify single-end reads with 150bp mean fragment length
**Args:** `quant -i hg38_transcriptome.idx -o quant/sample1 --single -l 150 -s 20 sample1.fq.gz`
**Explanation:** --single enables single-end mode; -l is mean fragment length; -s is standard deviation

### quantify stranded paired-end reads (forward-reverse orientation)
**Args:** `quant -i hg38_transcriptome.idx -o quant/sample1 --fr-stranded -b 50 R1.fq.gz R2.fq.gz`
**Explanation:** --fr-stranded for dUTP-based stranded library; check your protocol documentation

### inspect the transcriptome index
**Args:** `inspect hg38_transcriptome.idx`
**Explanation:** prints index statistics: number of k-mers, transcripts, and k-mer length
```

---

## Step 3: Verify the Skill Loads

```bash
oxo-call skill show kallisto
```

If you see the skill content, it is working. If not, check:
- File is at `~/.config/oxo-call/skills/kallisto.md`
- Filename matches the binary name exactly
- Front-matter block starts with `---` and ends with `---`
- Each example has a `### Task` heading, then `**Args:**` and `**Explanation:**` lines

---

## Step 4: Test the Skill

Compare dry-run output with and without the skill:

```bash
# With skill (user skill takes priority)
oxo-call dry-run kallisto "quantify paired-end reads R1.fq R2.fq against human transcriptome"

# Temporarily disable by moving the skill file
mv ~/.config/oxo-call/skills/kallisto.md /tmp/
oxo-call dry-run kallisto "quantify paired-end reads R1.fq R2.fq against human transcriptome"
mv /tmp/kallisto.md ~/.config/oxo-call/skills/kallisto.md
```

Compare the outputs — the skill-augmented version should include the correct index path pattern and bootstrap flag.

## Debugging Skills

### See what the LLM receives

Use the `--verbose` flag to see the full prompt sent to the LLM, including your skill content:

```bash
oxo-call dry-run --verbose kallisto \
  "quantify paired-end reads R1.fq R2.fq against human transcriptome"
```

With `--verbose`, the output includes:
- The system prompt rules
- Injected skill concepts, pitfalls, and examples
- The tool documentation sent to the LLM
- The user task description
- The raw LLM response

This helps you verify that your skill content is being injected correctly and identify whether issues are in the skill, the documentation, or the LLM's interpretation.

### Common debugging steps

1. **Skill not loading?** Check with `oxo-call skill show <tool>`. If it returns nothing, verify:
   - Filename matches the tool binary name exactly (case-sensitive)
   - File is in `~/.config/oxo-call/skills/` (user) or `~/.local/share/oxo-call/skills/` (community)
   - File starts with `---` (YAML front-matter delimiter)
   - Each `### Example` heading is followed by `**Args:**` and `**Explanation:**` lines

2. **Skill loaded but LLM ignores it?** Compare dry-run output with and without the skill. If the LLM ignores your examples, try:
   - Making concepts more specific and actionable
   - Adding more examples that directly match common tasks
   - Ensuring pitfalls describe concrete failure modes

3. **Validation warnings?** Skills must meet minimum depth requirements:
   - At least 5 examples
   - At least 3 concepts
   - At least 3 pitfalls
   
   Skills below these thresholds will produce a validation warning.

### Testing skills with oxo-bench

For systematic skill evaluation, the `oxo-bench` benchmarking crate can test skills programmatically:

```bash
# Run benchmark for a specific tool
cargo run -p oxo-bench -- evaluate --tool kallisto

# Export results for analysis
cargo run -p oxo-bench -- export-csv --output results/
```

See the [oxo-bench crate](https://github.com/Traitome/oxo-call/tree/main/crates/oxo-bench) for full usage details.

---

## Writing Good Skills

### Concepts: fundamental knowledge

Concepts should be facts that the LLM needs to understand to use the tool correctly, but that might not be obvious from `--help` output alone.

```markdown
# ✓ GOOD: explains non-obvious behavior
- BAM files MUST be coordinate-sorted before indexing with samtools index

# ✗ BAD: restates what --help already says
- Use -o to specify the output file
```

### Pitfalls: common mistakes

Pitfalls should be mistakes that users (or LLMs) commonly make, with the consequence explained.

```markdown
# ✓ GOOD: specific mistake + consequence
- Using --gtf with a genome FASTA (not transcriptome) for kallisto index — will produce incorrect k-mer counts

# ✗ BAD: vague
- Be careful with input files
```

### Examples: task → args mappings

Examples should cover the most common real-world tasks in your domain. The args are the actual command-line flags (without the tool name prefix).

```markdown
# ✓ GOOD: complete, realistic task
### align paired-end reads to hg38, output sorted BAM, 8 threads
**Args:** `mem -t 8 hg38.fa R1.fq.gz R2.fq.gz | samtools sort -o aligned.bam`
**Explanation:** pipes to samtools sort to avoid intermediate SAM file

# ✗ BAD: too generic
### align reads
**Args:** `mem ref.fa reads.fq`
**Explanation:** basic alignment
```

---

## Sharing Skills with Your Team

You can install a skill from a URL:

```bash
# Install from a URL (shared within your organization)
oxo-call skill install kallisto --url https://your-org.example.com/skills/kallisto.md

# Others on your team can do the same
oxo-call skill install kallisto --url https://your-org.example.com/skills/kallisto.md
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

