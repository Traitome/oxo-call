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
