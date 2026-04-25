---
name: check-skill
category: utilities
description: Guidelines for reviewing and validating skill files in oxo-call/skills/ directory
tags: [skill, validation, review, format, quality-control]
author: oxo-call built-in
source_url: "https://github.com/oxo-call/oxo-call"
---

## Concepts

- Skill files follow a consistent YAML frontmatter + Markdown format for documenting CLI tools.
- Each skill file includes: frontmatter (name, category, description, tags, author, source_url), ## Concepts section, ## Pitfalls section, and ## Examples section.
- Examples must follow the format: `### description`, `**Args:** `command_args``, `**Explanation:** explanation_text`.
- **Args/Explanation Consistency Rule**: Every component in Args MUST be explicitly explained in Explanation ("不多不少" - not too many, not too few). Explanation must not describe options absent from Args.

## Pitfalls

- **Missing Args explanations**: Common missing items include subcommand names, input/output files (-i/-o/-O/--input/--output), threads (-p/-t/-j/--threads), reference files (-r/-f/--ref), positional arguments, and all flags.
- **Extra Explanation content**: Explanation must only describe items present in Args; explaining options not shown in Args violates the "不多不少" principle.
- **Inconsistent naming**: Use full tool name + subcommand format (e.g., "samtools sort subcommand", "bcftools view subcommand") rather than just "subcommand".
- **For tools without subcommands**: Use "tool command" format (e.g., "spades command", "prokka command", "quast command") when Args starts directly with flags or positional arguments.
- **For multi-binary suites**: Identify each companion binary explicitly (e.g., "fasterq-dump companion binary", "prefetch companion binary" for sra-tools).

## Args/Explanation Consistency Checklist

When reviewing each example, verify:

1. **Subcommand/command name**: Is the tool name and subcommand (if applicable) explicitly mentioned?
2. **Input files**: Are all input files mentioned with their full flag and value?
3. **Output files**: Are all output files mentioned with their full flag and value?
4. **Threads**: Is the thread count flag explicitly explained?
5. **Reference files**: Is the reference genome/file explicitly explained?
6. **All flags**: Does every flag in Args appear in Explanation?
7. **No extras**: Does Explanation avoid describing options not present in Args?

## Example Review Pattern

For each example, the Explanation should follow this pattern:

```
tool [subcommand]; [flag] [value] description; [input_file] input; [output_file] output; [threads] threads; [other_flags] description
```

Example of correct format:
```
**Args:** `sort -o sorted.bam input.bam`
**Explanation:** samtools sort subcommand; -o sorted.bam output BAM; input.bam input file; coordinate sort is the default
```

Example of incorrect format (missing components):
```
**Args:** `sort -o sorted.bam input.bam`
**Explanation:** -o writes the sorted BAM file; coordinate sort is the default
```
(Missing: "samtools sort subcommand", "input.bam input file")

## Files to Check

Review all files in `oxo-call/skills/*.md` alphabetically. For each file:
1. Read the ## Pitfalls section to understand subcommand/command structure
2. Check each example for Args/Explanation consistency
3. Apply fixes for any mismatches found