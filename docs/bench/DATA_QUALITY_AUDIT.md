# Reference Data Quality Audit

> **Date:** 2026-08-06
> **Scope:** 133 tools, 1,330 reference scenarios, 13,300 usage descriptions
> **Conclusion:** Reference dataset requires systematic rebuild grounded in real tool documentation and executable verification.

## 1. Test Infrastructure

### 1.1 Local Tool Environment

All benchmarked tools are installed via [pixi](https://pixi.sh/) and accessible through system PATH.
A verification script (`check_tools.sh`) confirms availability:

- **Total tools configured:** 123
- **Installed & callable:** 118
- **Missing (5):** deepvariant, homer, humann3, rseqc, strelka2
- **Skills in repository:** 158 (includes non-tool skills: bash, awk, conda, etc.)

Tool-to-binary mapping is maintained in `check_tools.sh` — some tools use non-obvious
binary names (e.g., `gatk` → `gatk4`, `featurecounts` → `featureCounts`,
`snpeff` → `snpEff`).

### 1.2 Local LLM Models

15 Ollama models available for multi-model comparison testing:

| Model | Size | Category |
|-------|------|----------|
| deepseek-coder:1.3b | 776 MB | tiny code |
| qwen3.5:0.8b | 1.0 GB | tiny general |
| starcoder2:3b | 1.7 GB | small code |
| llama3.2:3b | 2.0 GB | small general |
| qwen3.5:2b | 2.7 GB | small general |
| ministral-3:3b | 3.0 GB | small general |
| mistral:7b | 4.4 GB | medium general |
| qwen2.5-coder:7b | 4.7 GB | medium code |
| ministral-3:8b | 6.0 GB | medium general |
| qwen3.5:9b | 6.6 GB | medium general |
| gemma4:e2b | 7.2 GB | medium general |
| deepseek-coder-v2:16b | 8.9 GB | large code |
| gemma4:e4b | 9.6 GB | large general |

Endpoint: `http://localhost:11434` (Ollama Docker container).
Connection stability: 100% success rate across all models.

## 2. Reference Data Quality Assessment

### 2.1 Data Origin

The reference dataset was **AI-generated in bulk** across 133 tools.
Each tool has 10 scenarios, and each scenario has 10 description phrasings
(original, beginner, intermediate, expert, verbose, terse, chinese, question,
command-like, vague), yielding 13,300 total entries.

### 2.2 Student Review Summary

A student review process was conducted (documented in the CSV at
`oxo-call-test/oxo-call reference_commands与skill检查-sheet1.csv`):

| Metric | Count |
|--------|-------|
| Total scenarios | 1,330 |
| Reviewed (核对人1 filled) | 892 (67%) |
| Issues flagged (核对人1问题 non-empty) | 491 (37% of total) |
| Unreviewed | 438 (33%) |

### 2.3 Systematic Issues Identified

Analysis of the student feedback and direct inspection reveals several
categories of systematic errors:

#### Category A: Placeholder/Invented Filenames

AI-generated filenames follow a repetitive pattern (`data.bed`, `results.gff3`,
`counts.fa`, `analysis.txt`) that does not reflect real bioinformatics workflows.
Real file names carry semantic meaning (e.g., `NA12878_final.bam`, `GRCh38.fa`).

**Impact:** The LLM sees meaningless filenames and cannot learn the association
between file extensions/content types and appropriate tool flags.

#### Category B: Incorrect Flag Semantics

Several reference commands use flags that don't exist or with incorrect syntax:

- `admixture data.bed K --cv=10` — `K` is not a positional argument; it should be an integer
- `agat config --expose` — subcommand may not exist in current AGAT version
- Some tools use `--flag value` where `--flag=value` is required

**Impact:** Benchmark measures accuracy against wrong answers; a correct LLM output
would be scored as incorrect.

#### Category C: Skill/Reference Mismatch

Some skill files recommend different flag patterns than the reference commands
for the same tool. Since the skill is injected into the LLM prompt, this creates
a conflict: the LLM receives conflicting guidance about "correct" usage.

**Impact:** Unfairly penalizes LLMs that follow skill guidance over reference.

#### Category D: Unrealistic Task Descriptions

Many descriptions are generic ("sort input.bam by coordinate") rather than
reflecting actual analysis tasks ("sort HG002 aligned reads by coordinate
for duplicate marking"). Generic descriptions don't test whether the LLM
understands biological context.

#### Category E: Bash/Scripting Syntax Errors

Several `bash` and scripting tool scenarios contain syntactically invalid
commands (e.g., `$;` in variable expansion).

#### Category F: Tool Subcommand Confusion

Some scenarios confuse a tool's subcommands (e.g., STAR alignment flags used
as Arriba flags, since Arriba wraps STAR).

### 2.4 Severity Distribution (Estimated)

| Severity | Description | ~% of issues |
|----------|-------------|-------------|
| Critical | Command would fail to execute or produce wrong results | ~30% |
| Major | Command works but description/flags misleading | ~35% |
| Minor | Filename conventions, style issues | ~25% |
| Cosmetic | Wording improvements only | ~10% |

## 3. Rebuild Strategy

### 3.1 Principles

1. **Documentation-first.** Every flag must be verified against the tool's `--help` output.
2. **Real scenarios.** Task descriptions must reflect actual bioinformatics analysis tasks.
3. **Executable verification.** Commands must be syntactically valid (flags exist, types correct).
4. **Semantic consistency.** Skill files, reference commands, and descriptions must agree.
5. **Meaningful filenames.** File names should carry biological semantics (sample IDs, reference builds).

### 3.2 Process Per Tool

For each of the 133 tools:

1. Run `tool --help` (or equivalent) locally to extract real flags and subcommands
2. Research the tool's primary use cases via official documentation
3. Review any student feedback from the CSV
4. Design 5-10 scenarios that cover the tool's most common real-world uses
5. Write correct reference commands verified against `--help`
6. Write clear, specific task descriptions
7. Cross-check with the existing skill file for consistency

### 3.3 Outputs

- **Updated `reference_commands.csv`** — verified commands with real flags
- **Updated `usage_descriptions.csv`** — corresponding descriptions
- **Updated skill files** — where reference/skill mismatches are found
- **`TOOL_VERIFICATION_LOG.md`** — per-tool verification records

## 4. References

- Wilkinson et al., *FAIR Guiding Principles* (2016)
- ROADMAP.md §M2 — "Scientific correctness and reproducibility"
- ROADMAP.md §5.4 — "Build functional evaluation from real tool documentation"
