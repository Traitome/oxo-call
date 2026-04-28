# Unified CLI Architecture for oxo-call

## Overview

This document describes the unified architecture for achieving high accuracy in CLI command generation across thousands of bioinformatics tools.

## Target Metrics

- **doc mode accuracy**: ≥ 0.80
- **skill mode accuracy**: ≥ 0.95
- **Model scale**: 3B+ models should achieve stable, reliable results

## CLI Pattern Taxonomy

Analysis of 6000+ bioconda CLI tools reveals 5 distinct patterns:

### Pattern A: Subcommand-based
- **Tools**: samtools, bcftools, gatk, git
- **Structure**: `tool subcommand -flags positional_args`
- **Critical Rule**: Subcommand MUST be first argument
- **Example**: `samtools sort -o sorted.bam input.bam`

### Pattern B: Direct flags
- **Tools**: fastp, minimap2, seqkit, seqtk
- **Structure**: `tool -flags positional_args`
- **Critical Rule**: NO subcommand, args start directly with flags
- **Example**: `fastp -i R1.fq -I R2.fq -o out1.fq -O out2.fq`

### Pattern C: Index+Action
- **Tools**: bwa, bowtie2, hisat2
- **Structure**: 
  - Index: `tool-index reference.fa`
  - Action: `tool mem -t N reference.fa reads.fq`
- **Critical Rule**: Requires index building step before alignment

### Pattern D: Long-option only
- **Tools**: STAR, featureCounts
- **Structure**: `tool --option=value --option2=value2 positional_args`
- **Critical Rule**: Uses `--option=value` format

### Pattern E: Quantification workflow
- **Tools**: salmon, kallisto
- **Structure**: 
  - Index: `tool index -i index_name reference.fa`
  - Quant: `tool quant -i index_name -o output reads.fq`

## Architecture Components

### 1. Pattern Detection Layer

Located in `src/doc_summarizer.rs` and `src/skill.rs`

**Key Functions:**
- `detect_cli_pattern(docs, tool)` - Detects pattern from documentation
- `detect_skill_pattern(first_args, tool_name)` - Detects pattern from skill examples

**Output:**
```text
=== CLI PATTERN: SUBCOMMAND REQUIRED ===
samtools REQUIRES a subcommand as FIRST argument.
Correct: 'samtools sort -flags args'
WRONG: 'samtools -flags args' (missing subcommand)
```

### 2. Structured Summary Layer

Located in `src/doc_summarizer.rs`

**Function:** `build_structured_summary(docs, tool)`

**Components:**
- Usage line extraction with positional argument highlighting
- Valid flags whitelist (use ONLY these)
- Subcommand list (if applicable)
- Pattern hint at the beginning

### 3. Skill Enhancement Layer

Located in `src/skill.rs`

**Function:** `render_section(max_examples, task)`

**Enhancements:**
- CLI pattern hint from first example
- Compact format for small models (5 concepts, 3 pitfalls, 3 examples)
- Emphatic section headers: KEY CONCEPTS, CRITICAL WARNINGS
- Task-relevant example selection via keyword + flag matching

### 4. Prompt Construction Layer

Located in `src/runner/core.rs`

**Components:**
- Structured header prepended to documentation
- Enhanced analysis constraints (valid flags, mutually exclusive groups)
- Pattern-specific guidance

## Prompt Format for Small Models

### Doc Mode (3B models)

```text
=== CLI PATTERN: SUBCOMMAND REQUIRED ===
samtools REQUIRES a subcommand as FIRST argument.

=== COMMAND STRUCTURE (CRITICAL) ===
  Usage: samtools sort -o output.bam input.bam

=== VALID FLAGS for samtools ===
Use ONLY flags from this list. Do NOT invent flags.
  -@, -b, -c, -f, -F, -h, -H, -l, -m, -n, -o, -O, -r, -t, -T, -u

=== SUBCOMMANDS (FIRST ARG if needed) ===
Available: view, sort, index, flagstat, merge, markdup, mpileup, ...
If task matches a subcommand, it MUST be the FIRST argument.
```

### Skill Mode (3B models)

```text
=== CLI PATTERN: SUBCOMMAND REQUIRED ===
samtools REQUIRES a subcommand as FIRST argument.
Subcommand 'sort' detected from examples.

=== KEY CONCEPTS ===
1. BAM files MUST be sorted BEFORE indexing.
2. Use -@ N for threading, -o FILE for output.
...

=== CRITICAL WARNINGS (AVOID THESE) ===
⚠ samtools ARGS MUST start with a subcommand — never with flags.
⚠ Without -o, samtools writes to stdout — pipe carefully.
...

=== WORKED EXAMPLES (FOLLOW THIS FORMAT) ===
Example 1:
  Task:  sort a BAM file by genomic coordinates
  ARGS:  sort -o sorted.bam input.bam
```

## Implementation Status

### Completed Improvements

1. **CLI Pattern Detection** - Added in doc_summarizer.rs and skill.rs
2. **Improved Section Headers** - KEY CONCEPTS, CRITICAL WARNINGS, WORKED EXAMPLES
3. **Positional Argument Highlighting** - Replace tool name with [TOOL] placeholder
4. **Pattern-specific Hints** - Correct/WRONG format examples

### Pending Improvements

1. Benchmark evaluation to measure accuracy
2. Error pattern analysis
3. Skill file optimization for specific tool categories
4. Auto-skill generation from bioconda metadata

## Benchmark Testing

Run benchmark:
```bash
oxo-call-test/bench/run_parallel_bench.sh \
  -s doc,skill \
  -m 'llama3.2:3b,qwen2.5-coder:7b'
```

## References

- Benchmark data: `oxo-call/docs/bench/reference_commands.csv`
- Usage descriptions: `oxo-call/docs/bench/usage_descriptions.csv`
- Bioconda tools: `oxo-call-extends/data/bioconda_cli_tools.txt` (6000+ tools)
- Bioconda metadata: `oxo-call-extends/data/bioconda_tools_metadata.jsonl`