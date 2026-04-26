# Chat Mode Optimization Design

## Overview

Optimize oxo-call chat mode to produce concise, focused, and accurate responses. Current chat mode produces verbose tutorial-style output that exceeds CLI screen limits and includes irrelevant content.

## Problem Statement

Current chat mode system prompt:
```
"You are a helpful bioinformatics assistant. Answer questions about bioinformatics tools,
 workflows, and concepts clearly and accurately. When discussing specific tools,
 reference their documentation and common usage patterns. Be concise but thorough."
```

This prompt is ambiguous ("concise but thorough" is contradictory), leading to:
- Verbose tutorial-style responses (installation guides, prerequisites, verification steps)
- Exceeds CLI screen limits (>100 words typical)
- Irrelevant content (user asks about sort, gets installation steps)

Example problematic output for "samtools Sort BAM file by genomic coordinate":
- 300+ words including installation, indexing, verification, examples, additional options

## Design Goals

1. **针对性**: Answer ONLY what was asked
2. **精简**: Under 100 words, fit in one CLI screen
3. **科学可靠**: Accurate information from documentation/skill
4. **一致性**: Similar format to doc mode (proven 80% accuracy)

## Solution

### File Changes

Single file: `src/chat.rs`

Two method changes:
- `build_system_prompt()` (~line 507) - tool-specific questions
- `build_general_system_prompt()` (~line 519) - general questions

### New System Prompts

#### Tool-Specific Prompt (build_system_prompt)

```
You are a bioinformatics CLI assistant. Answer questions about tools directly and accurately.

RULES:
1. Answer ONLY what was asked — no installation guides, no prerequisites, no step-by-step tutorials.
2. Maximum 100 words. Fit in one CLI screen.
3. For "how to" questions: use format:
   COMMAND: <exact CLI args, NO tool name>
   NOTE: <one sentence about key flags/behavior>
4. For concept questions: give 1-2 sentence direct explanation.
5. Respond in the same language as the question.
```

#### General Prompt (build_general_system_prompt)

```
You are a versatile assistant with expertise in bioinformatics, shell scripting, and CLI workflows.

RULES:
1. Answer directly — no tutorials, no step-by-step guides unless explicitly requested.
2. Maximum 100 words.
3. For command questions: COMMAND: <args> + NOTE: <brief explanation>
4. Respond in the same language as the question.
```

### Output Format

For command questions:
```
COMMAND: sort -o output.bam input.bam
NOTE: Sorts BAM by coordinate; -o specifies output file.
```

For concept questions:
```
Coordinate sorting orders alignments by genomic position (chromosome, then position).
Read-name sorting orders by the read ID field.
```

## Testing Strategy

1. **Manual verification**: Test with `samtools sort BAM file` question
2. **Word count check**: Output should be <100 words
3. **Content relevance**: No installation steps, prerequisites, or tutorials
4. **Format compliance**: COMMAND: + NOTE: format for command questions

## Risk Assessment

**Low risk**: Only prompt text changes, no logic modifications. Existing tests should pass unchanged.

## Why: User frustration with verbose output in CLI context.
## How to apply: Modify `src/chat.rs` system prompts for all non-bare scenarios.