---
name: check-skill
category: utility
description: Check and update an oxo-call skill.md file by inspecting local tool installation, fetching help documentation, web-searching for comprehensive references, and merging all findings into an accurate, reliable skill file.
tags: [skill, documentation, linting, updating, bioinformatics, quality-control]
author: oxo-call
source_url: "https://github.com/Traitome/oxo-call"
---

## Concepts

- A skill.md file is oxo-call's domain-expert knowledge injection unit: YAML front-matter (name, category, description, tags, author, source_url) + Markdown body (## Concepts, ## Pitfalls, ## Examples). Its accuracy directly determines command-generation quality.
- Tool command names may differ from skill file names or documentation conventions — e.g., `STAR` (binary) vs `star` (skill name), `MultiQC` (docs) vs `multiqc` (command), `R` (binary) vs `r` (skill name). Case-insensitive lookup is essential.
- Three independent documentation sources must be cross-referenced: (1) local `--help` / `-h` / bare invocation output, (2) the skill's `source_url` and linked documentation, (3) internet search results. No single source is sufficient.
- The final skill must reliably cover ≥99% of real-world usage. This means: every common subcommand, every frequently-used flag, every critical ordering/dependency constraint, and at least 5 worked examples spanning basic to advanced usage.
- Conda/pixi environments may install tools under different names than expected. Always verify with `which`, `command -v`, or environment-specific listing commands.

## Pitfalls

- Never assume a tool is missing just because the lowercase skill name is not in PATH. Try case variants (e.g., `STAR`, `Star`, `star`), known aliases, and environment-specific names before declaring a tool unavailable.
- Never blindly trust a single documentation source. Official docs may be outdated, `--help` may omit advanced options, and web search results may describe a different version. Cross-reference at least two sources.
- Do not remove existing correct content from a skill.md unless you have strong evidence it is wrong. Prefer adding missing information and correcting inaccuracies.
- Do not fabricate flags, subcommands, or examples that you cannot verify from at least one concrete source (help output, official docs, or a reliable web reference).
- When updating examples, preserve the exact format: `### description of what to do` → `**Args:** \`command args\`` → `**Explanation:** why these flags were chosen`. The `**Args:**` value must start with the subcommand (or first positional) — never with a flag.
- Some tools (like STAR) use long options as their primary operation selector rather than traditional subcommands. The skill's Pitfalls section must call this out explicitly with a CRITICAL note.
- Avoid duplicating information across Concepts, Pitfalls, and Examples. Each section has a distinct role: Concepts = mental model, Pitfalls = what goes wrong, Examples = concrete commands.
- **Args/Explanation mismatch** is a common error: when reviewing examples, verify that every option in Args is explicitly explained in Explanation, and that Explanation does not describe options absent from Args. This ensures completeness (不多不少) — no missing explanations, no extra explanations.

## Examples

### check and update the admixture skill file
**Args:** `/data/home/wsx/Projects/oxo/oxo-call/skills/admixture.md`
**Explanation:** Reads the skill file, checks if admixture is installed (pixi global list, conda list, which), fetches --help output, web-searches the source_url https://dalexander.github.io/admixture/, cross-references all sources, and updates the skill with any missing flags, concepts, pitfalls, or examples.

### check and update the star skill file with case-insensitive binary lookup
**Args:** `/data/home/wsx/Projects/oxo/oxo-call/skills/star.md`
**Explanation:** The skill name is "star" but the binary is "STAR". Uses case-insensitive search (which STAR, command -v STAR) and pixi global list to confirm installation, then fetches STAR --help output and cross-references with the STARmanual.pdf source_url.

### check a skill for a tool installed via pixi global
**Args:** `/data/home/wsx/Projects/oxo/oxo-call/skills/fastp.md`
**Explanation:** Runs `pixi global list 2>&1 | grep -E "^[├│]"` to find bioconda tools, cross-references with the skill name, then fetches help output from the installed binary.

### check a skill where the command name differs from documentation
**Args:** `/data/home/wsx/Projects/oxo/oxo-call/skills/multiqc.md`
**Explanation:** Documentation writes "MultiQC" but the command is "multiqc". Uses the lowercase command to fetch --help, while the skill name and description may reference the project name "MultiQC".

### check a skill for a tool not installed locally
**Args:** `/data/home/wsx/Projects/oxo/oxo-call/skills/cellranger.md`
**Explanation:** Tool not in PATH, conda, or pixi. Skips local help fetching, relies entirely on source_url web-fetching and internet search to verify and update the skill. Notes in the output that the tool was not locally available for verification.

---

# Workflow

When given a skill.md file path, execute the following steps in order:

## Step 1: Parse the skill file

Read the file and extract:
- Front-matter fields: `name`, `category`, `description`, `tags`, `source_url`
- Body sections: `## Concepts`, `## Pitfalls`, `## Examples`
- Count existing concepts, pitfalls, and examples

## Step 2: Check local installation

Determine if the tool is available on the local system. Try in order:

### 2a: Direct PATH lookup (case-insensitive)
```bash
# Try the skill name and common case variants
tool_name="<name from front-matter>"
command -v "$tool_name" || command -v "$(echo $tool_name | tr '[:lower:]' '[:upper:]')" || command -v "$(echo $tool_name | tr '[:upper:]' '[:lower:]')"
```

Known case-mismatch pairs to check explicitly:
| Skill name | Binary name |
|---|---|
| `star` | `STAR` |
| `r` | `R` |
| `R` | `R` |
| `multiqc` | `multiqc` (docs say MultiQC but command is lowercase) |
| `bwa-mem2` | `bwa-mem2` (binary name matches) |
| `hap_py` | `hap.py` |

### 2b: pixi global list
```bash
pixi global list 2>&1 | grep -E "^[├│]"
```
Look for the tool name (case-insensitive) in the output. Some tools appear under their bioconda package name which may differ from the command name.

### 2c: conda environments
```bash
conda list 2>/dev/null | grep -i "$tool_name"
# Also check named environments
conda env list 2>/dev/null | while read env _; do conda list -n "$env" 2>/dev/null | grep -i "$tool_name"; done
```

Record whether the tool was found and via which method. If not found, note this — you will rely solely on web sources.

## Step 3: Fetch local help documentation

If the tool is installed, retrieve its help text using the same strategy as oxo-call's `DocsFetcher::fetch_help()`:

1. `<tool> --help` (primary)
2. `<tool> -h` (fallback)
3. `<tool> help` (subcommand style, e.g., `git help`, `conda help`)
4. `<tool>` with no arguments (many bioinformatics tools print usage when invoked bare)
5. `man <tool>` (for tools with man pages)

For each command, capture both stdout and stderr (many tools write usage to stderr). Concatenate and deduplicate. Truncate extremely long output (>5000 lines) but preserve the beginning (general usage) and the section listing all flags/options.

### 3b: Enumerate and fetch all subcommand help

Many bioinformatics tools have a rich subcommand hierarchy. The top-level `--help` only lists subcommand names without their individual flags. To capture complete documentation, you must recursively traverse all subcommands:

**Strategy:**

1. Parse the top-level help output to identify all subcommand names. Common patterns:
   - Lines like `  view       View BAM/SAM/CRAM files` (samtools, bcftools, bedtools)
   - Lines like `  index           Index a reference genome` (bwa, minimap2)
   - A "Commands:" or "Subcommands:" section header followed by a list
   - A "usage: tool <command> [options]" pattern in the header

2. For each discovered subcommand, fetch its help:
   - `<tool> <subcommand> --help`
   - `<tool> <subcommand> -h`
   - `<tool> <subcommand>` (bare invocation, some tools print usage when called without arguments)

3. For tools with nested subcommands (e.g., `conda env list`, `git remote add`, `docker image build`), repeat step 1–2 for each level:
   - First level: `conda --help` → discovers `env`, `create`, `install`, `run`, etc.
   - Second level: `conda env --help` → discovers `list`, `create`, `remove`, `export`, etc.
   - Third level (if applicable): `conda env create --help` → flags for that specific sub-subcommand

4. **Known multi-level subcommand tools** (check these explicitly):
   | Tool | Subcommand depth | Example |
   |------|-----------------|---------|
   | `samtools` | 1 | `samtools view --help` |
   | `bcftools` | 1 | `bcftools view --help` |
   | `bedtools` | 1 | `bedtools intersect --help` |
   | `bwa` | 1 | `bwa mem --help` |
   | `conda` | 2+ | `conda env create --help` |
   | `docker` | 2+ | `docker image build --help` |
   | `git` | 2+ | `git remote add --help` |
   | `gatk` | 2+ | `gatk HaplotypeCaller --help` |
   | `snakemake` | 1 | `snakemake run --help` |
   | `nextflow` | 1+ | `nextflow run --help` |

5. For each subcommand's help output, extract:
   - All flags and their descriptions (especially flags not shown at the top level)
   - Positional argument requirements
   - Input/output format expectations
   - Any CRITICAL ordering or dependency constraints

6. Compile a subcommand inventory table:
   ```
   | Subcommand | Help fetched? | Key flags discovered |
   |------------|--------------|---------------------|
   | view       | YES          | -b -h -H -F -f -q  |
   | sort       | YES          | -@ -m -o -n -T     |
   | ...        | ...          | ...                 |
   ```

This exhaustive traversal ensures no subcommand or flag is missed, which is critical for tools like samtools (30+ subcommands), bcftools (20+ subcommands), or GATK (50+ sub-tools).

Also fetch version information:
- `<tool> --version`
- `<tool> -V`
- Version strings embedded in the help output header

## Step 4: Fetch remote documentation

### 4a: Fetch the source_url
If `source_url` is present in the front-matter, fetch it using web_fetch. Extract:
- All command-line flags and options
- Subcommands and their descriptions
- Usage patterns and examples
- Version-specific notes
- Known limitations

### 4b: Internet search for comprehensive documentation
Use web_search with queries like:
- `"<tool_name> command line options flags"`
- `"<tool_name> bioinformatics usage guide"`
- `"<tool_name> manual documentation"`
- `"<tool_name> common errors pitfalls"`

For each search result that looks authoritative (official docs, GitHub README, bioconda page, biostars answers), fetch and extract relevant information.

### 4c: Fetch related documentation
If the source_url points to a general page, look for linked pages:
- `/docs/`, `/documentation/`, `/manual/`, `/usage/` sub-pages
- GitHub `README.md`, `docs/` directory
- Bioconda package page for version-specific usage notes

## Step 5: Cross-reference and identify gaps

Compare the three sources (local help, source_url docs, web search) against the existing skill.md content:

### Front-matter checks:
- Is `name` correct and matches the skill file stem?
- Is `category` appropriate? Standard categories: alignment, qc, variant-calling, expression, assembly, annotation, population-genomics, epigenomics, metagenomics, genome-editing, formatting, programming, hpc, containerization, utility
- Is `description` accurate and concise (one line)?
- Are `tags` comprehensive? Should include the tool name, file formats, and domain keywords
- Is `source_url` pointing to the best available documentation?

### Concepts checks (minimum 5 recommended):
- Are the key mental-model concepts present?
- Is the input/output format clearly stated?
- Are threading/parallelism options documented?
- Are common workflow dependencies noted (e.g., "must run index before align")?

### Pitfalls checks (minimum 3 recommended):
- Is there a CRITICAL note about argument ordering (subcommand-first vs flags-first)?
- Are common error conditions documented?
- Are version-specific gotchas noted?
- Are common misconceptions addressed?

### Examples checks (minimum 5 recommended):
- Do examples cover the most common use cases?
- Does at least one example show basic usage?
- Does at least one example show advanced usage (threads, output format, compression)?
- Are paired-end / multi-file input patterns shown where applicable?
- Does each example follow the exact format: `### description` → `**Args:** \`...\`` → `**Explanation:** ...`?
- Are the Args values realistic and directly usable (real file extensions, realistic paths)?
- **CRITICAL: Args/Explanation consistency** — Every command option/flag in Args MUST be explicitly explained in Explanation, and Explanation MUST NOT describe options that are not present in Args (不多不少). For example, if Args contains `-a file.bed -b ref.fa`, Explanation must mention both `-a` and `-b`. If Args contains `--cv=10`, Explanation must mention `--cv`. Conversely, if Explanation mentions `--threads` but Args does not contain any threading flag, this is an error.

## Step 6: Update the skill file

Apply all identified improvements to the skill.md file:

1. **Preserve correct existing content** — only modify what is wrong or incomplete
2. **Add missing concepts** — append to the existing Concepts section
3. **Add missing pitfalls** — append to the existing Pitfalls section
4. **Add missing examples** — append to the existing Examples section
5. **Fix inaccuracies** — correct any factually wrong statements based on documentation evidence
6. **Update front-matter** — if category, description, tags, or source_url need improvement

When writing Examples, follow these rules strictly:
- The `**Args:**` value must be the exact command arguments (no tool name prefix, no shell prompt)
- For tools with subcommands (samtools, bwa, bcftools): Args starts with the subcommand
- For tools with long-option operations (STAR): Args starts with `--runMode`
- For tools with no subcommands (multiqc, fastp): Args starts with flags or input paths
- The `**Explanation:**` must explain WHY each flag was chosen, not just WHAT it does

## Step 7: Output a summary

After updating the file, print a structured summary:

```
## Skill Check Summary: <tool_name>

### Installation
- Found via: [pixi global / conda / PATH / NOT FOUND]
- Binary name: <actual_binary_name>
- Version: <detected_version_or_unknown>

### Documentation Sources Used
- Local --help: [YES/NO] (<line_count> lines)
- source_url fetch: [YES/NO] (<url>)
- Web search: [YES/NO] (<number_of_sources>)

### Changes Applied
- Front-matter: [NO CHANGES / list of fields updated]
- Concepts: [NO CHANGES / +N added / M corrected]
- Pitfalls: [NO CHANGES / +N added / M corrected]
- Examples: [NO CHANGES / +N added / M corrected]

### Coverage Assessment
- Subcommands documented: X/Y (list any missing)
- Common flags documented: X/Y (list any missing)
- Examples: N (minimum 5 recommended)
- Estimated real-world coverage: ~N%
```
