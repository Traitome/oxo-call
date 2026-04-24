---
name: skill-generator
category: utility
description: Generate comprehensive skill.md files for bioinformatics tools using a structured 7-step workflow and local tool documentation extraction
tags: [skill, creation, generation, bioinformatics, template, workflow, documentation]
author: oxo-call built-in
source_url: "https://github.com/Traitome/oxo-call/blob/main/check-skill/skill.md"
---

## Concepts

- A skill.md file is oxo-call's domain-expert knowledge injection unit: YAML front-matter (name, category, description, tags, author, source_url) + Markdown body (## Concepts, ## Pitfalls, ## Examples). Minimum requirements: 5 examples, 3 concepts, 3 pitfalls.
- Tool command names may differ from skill file names — case-insensitive lookup is essential. Examples: `STAR` (binary) vs `star` (skill), `R` (binary) vs `r` (skill), `MultiQC` (docs) vs `multiqc` (command).
- Many bioinformatics tools require the subcommand FIRST in ARGS: bwa mem, samtools sort, gatk HaplotypeCaller. This pattern is CRITICAL — Args MUST start with the subcommand, never with flags like -t or -o.
- Some tools have companion binaries or scripts: bowtie2-build, bbduk.sh, samtools-fixmate. When documentation specifies these, use the companion name as the first Args token.
- Documentation hierarchy for skill generation: (1) local --help output (most reliable), (2) source_url/fetched docs, (3) web search (if available). Local --help is essential for accurate flag extraction.
- The ## Examples section format is strict: `### <task description>` → `**Args:** \`<flags without tool name>\`` → `**Explanation:** <why these flags were chosen>`. Args NEVER start with the tool name itself.
- Common workflow dependencies must be documented: index before align (bwa, minimap2), sort before index (samtools), fixmate before markdup. These are key concepts and pitfalls.
- Threading flags (-t, -@, --threads) and output flags (-o, --output, -O) are ubiquitous. Always document these in concepts and show usage in examples.

## Pitfalls

- Starting Args with a flag instead of the subcommand — e.g., `mem -t 8` instead of `-t 8` for bwa mem. CRITICAL: For subcommand tools, Args starts with the subcommand.
- Fabricating flags or examples that are not verified from documentation — only include flags and behaviors you can verify from --help output or official docs.
- Writing vague concepts like "use -o for output" that restate --help — concepts should explain non-obvious behaviors, paradigms, and workflow dependencies.
- Writing pitfalls without consequences — every pitfall must explain what goes wrong: "Running index before sort will fail because..."
- Incorrect example format — missing `**Args:**` backticks, or including the tool name in Args. The format must be exactly: `### task` → `**Args:** \`...\`` → `**Explanation:** ...`
- Case mismatch between skill name and binary — skill name is lowercase (star) but binary may be uppercase (STAR). Use case-insensitive lookup.
- Missing minimum depth requirements — skills must have ≥5 examples, ≥3 concepts, ≥3 pitfalls. Below this threshold, validation will warn.
- Removing existing correct content during regeneration — preserve verified information; only add missing or fix incorrect content.

## Examples

### generate a skill for samtools (tool installed locally)
**Args:** `samtools`
**Explanation:** Parse skill-generator workflow, check PATH for samtools, fetch `samtools --help` to discover all subcommands, extract key flags (-@, -o, -b), write concepts about BAM/SAM/CRAM and coordinate-sort requirements, write pitfalls about subcommand-first and index-before-region-query, write 10+ examples covering sort, view, index, flagstat, fastq.

### generate a skill for bwa with local help extraction
**Args:** `bwa`
**Explanation:** Run `bwa --help` to discover subcommands (index, mem, aln, samse, sampe), run `bwa mem --help` for mem-specific flags, write concept about index requirement, write pitfalls about mem output going to stdout (must pipe), write examples showing index, mem paired-end, mem single-end, mem with read group.

### generate a skill for a tool not installed locally
**Args:** `cellranger`
**Explanation:** Tool not in PATH, pixi, or conda — skip local help fetching, rely on source_url if provided or generate template with placeholder concepts/pitfalls noting tool was not locally verified, still meet minimum requirements with generic bioinformatics guidance.

### generate a skill for a multi-subcommand tool (gatk)
**Args:** `gatk`
**Explanation:** GATK has 50+ sub-tools — enumerate from `gatk --help`, focus on most common (HaplotypeCaller, Mutect2, BaseRecalibrator), write concept about the sub-tool pattern, write pitfalls about MarkDuplicates requirement before BQSR, write examples for each major sub-tool.

### generate a skill for a single-command tool (fastp)
**Args:** `fastp`
**Explanation:** fastp has no subcommands — ARGS start with flags directly, fetch `fastp --help` to extract QC/trimming flags, write concepts about all-in-one QC+trimming, write pitfalls about adapter detection auto-mode, write examples showing basic QC, trimming with qualified-quality-phred, adapter trimming.

### generate a skill with threading and output pattern documentation
**Args:** `minimap2`
**Explanation:** Extract `-t N` for threads and `-o FILE` for output from help, write concept about threading support, write examples showing `-t 8` and `-o aligned.sam` usage, explain in Explanation that minimap2 writes to stdout by default so -o is needed for direct file output.

### generate a skill preserving workflow dependencies
**Args:** `samtools`
**Explanation:** Document in Concepts: "BAM files MUST be coordinate-sorted BEFORE indexing", write Pitfall: "samtools index on an unsorted BAM will appear to succeed but region queries will give wrong results", write Examples showing the complete workflow: sort → index.

### generate a skill for a tool with case mismatch
**Args:** `star`
**Explanation:** Skill name is `star` (lowercase) but binary is `STAR` (uppercase) — note this in Pitfalls, try case-insensitive lookup (command -v STAR, command -v star), fetch `STAR --help` for documentation.

### generate a skill documenting output format choices
**Args:** `samtools view`
**Explanation:** Write Concept: "samtools view without -b or -O bam outputs SAM text, not BAM — the file will be much larger", write Example showing `-b` flag for BAM output, write Pitfall about forgetting -b causing large SAM files.

### generate a skill with read group pattern for GATK compatibility
**Args:** `bwa mem`
**Explanation:** Write Concept about -R read group requirement for GATK, write Example: `-R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA'`, write Pitfall: "For GATK downstream analysis, always add a read group — GATK requires RG information."

---

## Workflow

When generating a skill.md file for a bioinformatics tool, execute the following steps in order:

### Step 1: Identify tool name and check local installation

Determine if the tool is available on the local system. Try in order:

**1a: Direct PATH lookup (case-insensitive)**
```bash
# Try the tool name and common case variants
tool_name="<name from request>"
command -v "$tool_name" || command -v "$(echo $tool_name | tr '[:lower:]' '[:upper:]')" || command -v "$(echo $tool_name | tr '[:upper:]' '[:lower:]')"
```

Known case-mismatch pairs to check explicitly:
| Skill name | Binary name |
|---|---|
| `star` | `STAR` |
| `r` | `R` |
| `multiqc` | `multiqc` (docs say MultiQC but command is lowercase) |
| `hap_py` | `hap.py` |

**1b: pixi global list**
```bash
pixi global list 2>&1 | grep -E "^[├│]"
```

**1c: conda environments**
```bash
conda list 2>/dev/null | grep -i "$tool_name"
```

Record whether the tool was found and via which method. If not found, note this — you will generate a template-only skill.

### Step 2: Fetch local help documentation

If the tool is installed, retrieve its help text:

**2a: Top-level help**
```bash
<tool> --help      # primary
<tool> -h          # fallback
<tool> help        # subcommand style (git, conda)
<tool>             # bare invocation (many bio tools print usage)
man <tool>         # if man page exists
```

Capture both stdout and stderr (many tools write usage to stderr). Concatenate and deduplicate.

**2b: Subcommand enumeration**

For tools with subcommands, parse top-level help to identify all subcommand names, then fetch each subcommand's help:

```bash
<tool> <subcommand> --help
<tool> <subcommand> -h
<tool> <subcommand>   # bare invocation
```

Known multi-subcommand tools:
| Tool | Subcommand depth | Example |
|------|-----------------|---------|
| `samtools` | 1 | `samtools view --help` |
| `bcftools` | 1 | `bcftools view --help` |
| `bwa` | 1 | `bwa mem --help` |
| `gatk` | 2+ | `gatk HaplotypeCaller --help` |
| `conda` | 2+ | `conda env create --help` |

For each subcommand, extract: flags, descriptions, positional arguments, I/O format expectations.

**2c: Version information**
```bash
<tool> --version
<tool> -V
```

### Step 3: Analyze tool structure

From the help output, determine:

1. **Subcommand pattern**: Does the tool require a subcommand first? (bwa, samtools, gatk) Or can flags come first? (fastp, multiqc)
2. **Most common operations**: What are the top 5-10 most frequently used commands?
3. **Key flags to document**: threading (-t, -@, --threads), output (-o, --output), format (-b, -O, -C), filtering (-f, -F, -q)
4. **Workflow dependencies**: Does this tool require another tool to run first? (index before align, sort before index)

### Step 4: Write Concepts section

Write 5-10 concepts covering:

- **Data model**: What file formats? What's the input/output paradigm?
- **Key behaviors**: Non-obvious behaviors not clear from --help alone
- **Workflow dependencies**: "MUST run X before Y"
- **Threading/parallelism**: How to use multiple cores
- **Output defaults**: Where does output go? (stdout vs file)
- **Version-specific notes**: If version matters

**Concept quality rules:**
- ✓ GOOD: "BAM files MUST be coordinate-sorted BEFORE indexing — region queries require sorted+indexed BAM"
- ✗ BAD: "Use -o to specify output file" (restates --help)

### Step 5: Write Pitfalls section

Write 5-10 pitfalls covering:

- **Argument ordering**: Subcommand-first vs flags-first mistakes
- **Missing dependencies**: Running a tool without prerequisite steps
- **Output surprises**: Output going to stdout when user expects a file
- **Format mismatches**: Wrong file format assumptions
- **Case sensitivity**: Binary name vs expected name
- **Version differences**: Breaking changes between versions

**Pitfall quality rules:**
- ✓ GOOD: "samtools index on an unsorted BAM will appear to succeed but region queries will give wrong results"
- ✗ BAD: "Be careful with input files" (vague)

### Step 6: Write Examples section

Write 5-15 examples covering:

- **Basic usage**: The simplest common operation
- **Intermediate usage**: With threading, output flags
- **Advanced usage**: Complex workflows, filtering, specialized flags
- **Pipeline patterns**: Piping to other tools

**Example format (strict):**
```markdown
### <task description in plain English>
**Args:** `<flags and arguments WITHOUT tool name>`
**Explanation:** <one sentence explaining why these flags were chosen>
```

**Args rules:**
- NEVER start with the tool name
- For subcommand tools: start with subcommand (e.g., `sort -@ 4 -o sorted.bam`)
- For single-command tools: start with flags (e.g., `--thread 4 --output result.txt`)
- For companion binaries: start with companion name (e.g., `bowtie2-build ref.fa index`)
- Include realistic file extensions (.bam, .fastq.gz, .fa)
- Show threading and output flags in at least one example

### Step 7: Validate and output

Before outputting the skill, verify:

1. **Minimum requirements met**: ≥5 examples, ≥3 concepts, ≥3 pitfalls
2. **Format correctness**: Each example has `### task`, `**Args:** \`...\``, `**Explanation:**`
3. **Args format**: No tool name at start of Args
4. **Front-matter complete**: name, category, description, tags, author, source_url

Output the complete skill.md file starting with `---` (YAML front-matter delimiter).

---

## Output Format

When generating a skill, output ONLY the complete skill.md file content:

```markdown
---
name: <tool>
category: <domain>
description: <one-line description>
tags: [<relevant tags>]
author: AI-generated
source_url: <docs URL if known, or empty>
---

## Concepts

- <concept 1>
- <concept 2>
...

## Pitfalls

- <pitfall 1 — with consequence>
- <pitfall 2 — with consequence>
...

## Examples

### <task description>
**Args:** `<flags without tool name>`
**Explanation:** <why these flags>

...
```

Do NOT add any preamble, explanation, or code fences around the output. The output must be a valid skill.md file that oxo-call can parse directly.