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
