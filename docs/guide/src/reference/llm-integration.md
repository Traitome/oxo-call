# LLM Integration

## Overview

oxo-call supports four LLM providers for command generation:

| Provider | Default Model | Token Required |
|----------|--------------|----------------|
| GitHub Copilot | auto | Yes (GitHub PAT) |
| OpenAI | gpt-4o | Yes |
| Anthropic | claude-3-5-sonnet-20241022 | Yes |
| Ollama | llama3.2 | No (local) |

## Prompt Architecture

### System Prompt

The system prompt contains 11 rules that constrain the LLM's behavior:

1. Only use flags documented in the provided documentation
2. Never include the tool name in the ARGS output
3. Use realistic filenames from the user's task description
4. Generate production-ready commands
5. Follow bioinformatics conventions
6. Never hallucinate flags or options
7. Support multi-step operations
8. Use threading when available
9. Match the output format to the task
10. Handle strand-specific protocols correctly
11. Output ASCII-only characters

### Response Format

The LLM must respond with exactly two labeled lines:

```
ARGS: <generated arguments>
EXPLANATION: <brief explanation of why these arguments were chosen>
```

If the response doesn't match this format, oxo-call retries the request.

### Raw Prompt Example

Below is an example of the complete user prompt sent to the LLM for a `samtools sort` task. This shows the actual structure including skill injection and format instructions.

```
# Tool: `samtools`

## Expert Knowledge (from skill)

### Key Concepts
- BAM files MUST be coordinate-sorted before indexing with samtools index
- Use -@ to set additional threads for parallel processing
- samtools view -F 0x904 filters out unmapped, secondary, and supplementary reads

### Common Pitfalls
- Forgetting to index after sorting — samtools index requires a coordinate-sorted BAM
- Using -q without -b — quality filtering without BAM output produces SAM to stdout
- Not specifying -o — output goes to stdout by default, which can corrupt terminal

### Worked Examples
Task: sort a BAM file by coordinate
Args: sort -o sorted.bam input.bam
Explanation: coordinate sort is the default; -o specifies output file

Task: index a sorted BAM file
Args: index sorted.bam
Explanation: creates .bai index required for random access

## Tool Documentation
<captured --help output and cached documentation>

## Task
sort input.bam by coordinate and output to sorted.bam

## Output Format (STRICT — do not add any other text)
Respond with EXACTLY two lines:

ARGS: <all command-line arguments, space-separated, WITHOUT the tool name itself>
EXPLANATION: <one concise sentence explaining what the command does>

RULES:
- ARGS must NOT start with the tool name
- ARGS must only contain valid CLI flags and values (ASCII, tool syntax)
- EXPLANATION should be written in the same language as the Task above
- Include every file path mentioned in the task
- Use only flags documented above or shown in the skill examples
- Prefer flags from the skill examples when they match the task
- If no arguments are needed, write: ARGS: (none)
- Do NOT add markdown, code fences, or extra explanation
```

Use `--verbose` mode to see the actual prompt for any command:

```bash
oxo-call dry-run --verbose samtools "sort input.bam by coordinate"
```

## Provider Configuration

See the [Configuration tutorial](../tutorials/configuration.md) for setup instructions.

## Grounding Strategy

oxo-call uses a "docs-first" grounding strategy:

1. Tool documentation is fetched and included in the prompt
2. If a skill exists, expert knowledge is injected
3. The combined context prevents the LLM from hallucinating flags

This approach is critical for accuracy, especially with:
- Complex tools with hundreds of options
- Tools with version-specific flag differences
- Smaller or weaker LLM models
