# chat

Interactive chat with AI about bioinformatics tools and concepts.

## Synopsis

```
oxo-call chat [OPTIONS] <TOOL> <QUESTION>
oxo-call chat -i [OPTIONS]
oxo-call c   [OPTIONS] <TOOL> <QUESTION>
```

## Options

| Option | Description |
|--------|-------------|
| `-i`, `--interactive` | Start interactive multi-turn chat session |
| `-m`, `--model <MODEL>` | Override the LLM model for this invocation |
| `--no-cache` | Skip cached documentation and fetch fresh `--help` output |
| `--scenario <SCENARIO>` | Context injection mode: `bare`, `prompt`, `skill`, `doc`, `full` (default: `full`) |
| `--json` | Output result as JSON (non-interactive mode only) |
| `-v`, `--verbose` | Show docs source, skill info, and LLM details (global) |
| `--license <PATH>` | Path to license file (global option) |

### Scenarios

The `--scenario` flag controls what context is injected into the conversation:

| Scenario | Description |
|----------|-------------|
| `bare` | Plain chat with no system prompt, documentation, or skill |
| `prompt` | Use oxo-call system prompt only |
| `skill` | Load the tool's skill file only |
| `doc` | Load the tool's documentation only |
| `full` | Load everything: system prompt + skill + documentation (default) |

## Description

The `chat` command provides two modes for interacting with AI about bioinformatics tools:

### Single-shot Q&A (Non-interactive)

Ask a single question about a specific tool and get an immediate response:

```bash
oxo-call chat samtools "How do I sort a BAM file?"
```

### Interactive Multi-turn Chat

Start an interactive session for extended conversations:

```bash
oxo-call chat -i
```

In interactive mode, you can:

- Ask multiple questions in sequence
- Switch between tools with `/tool <name>`
- Change scenarios with `/scenario <mode>`
- Clear conversation history with `/clear`
- Exit with `/quit` or `Ctrl+D`

## Interactive Commands

When in interactive mode (`-i`), the following commands are available:

| Command | Description |
|---------|-------------|
| `/tool <name>` | Set tool context for subsequent questions |
| `/clear` | Clear conversation history |
| `/scenario <s>` | Change scenario mode (bare/prompt/skill/doc/full) |
| `/help` | Show available commands |
| `/quit` or `Ctrl+D` | Exit the chat |

## Examples

### Single-shot Q&A

```bash
# Ask about a specific tool
oxo-call chat samtools "Explain the difference between SAM and BAM formats"

# Ask about common pitfalls (skill mode)
oxo-call chat --scenario skill bwa "What are common pitfalls when using BWA?"

# Ask with documentation context only
oxo-call chat --scenario doc gatk "How do I call variants with HaplotypeCaller?"

# Plain chat without any context
oxo-call chat --scenario bare "What is the difference between paired-end and single-end sequencing?"

# Use a different LLM model
oxo-call chat --model gpt-4 samtools "How to extract unmapped reads from a BAM file?"

# Get JSON output for scripting
oxo-call chat --json samtools "What does the -F flag do?"
```

### Interactive Mode

```bash
# Start interactive chat
oxo-call chat -i

# Start with a pre-set tool context
oxo-call chat -i --tool samtools

# Start with a specific scenario
oxo-call chat -i --scenario skill
```

### Interactive Session Example

```
$ oxo-call chat -i

╔════════════════════════════════════════════════════════════╗
║ oxo-call Interactive Chat                                  ║
╚════════════════════════════════════════════════════════════╝

Commands:
  /tool <name>    Set tool context for subsequent questions
  /clear          Clear conversation history
  /scenario <s>   Change scenario (bare|prompt|skill|doc|full)
  /help           Show this help message
  /quit, Ctrl+D   Exit the chat

Usage:
  <tool> <question>    Ask about a specific tool
  <question>           Ask about the current tool (if set)

oxo> /tool samtools
Tool context set to: samtools

samtools> How do I sort a BAM file by coordinate?

To sort a BAM file by coordinate using samtools, you can use the `sort` subcommand:

```bash
samtools sort -o sorted.bam input.bam
```

For multi-threading:
```bash
samtools sort -@ 8 -o sorted.bam input.bam
```

samtools> /scenario skill
Scenario changed to: skill

samtools> What are common pitfalls?
...

samtools> /quit
Goodbye!
```

## Use Cases

### Learning a New Tool

Use `--scenario skill` to focus on expert knowledge and common pitfalls:

```bash
oxo-call chat --scenario skill minimap2 "What are the key concepts I should understand?"
```

### Quick Reference

Use `--scenario doc` to get answers based on the tool's official documentation:

```bash
oxo-call chat --scenario doc bcftools "What filtering options are available?"
```

### General Bioinformatics Questions

Use `--scenario bare` or `--scenario prompt` for general questions not tied to a specific tool:

```bash
oxo-call chat --scenario prompt "Explain the concept of mapping quality"
```

### Debugging and Troubleshooting

Interactive mode is ideal for iterative debugging:

```bash
oxo-call chat -i --tool samtools
samtools> Why am I getting empty output from my view command?
samtools> How can I check if my BAM file is corrupted?
samtools> What's the difference between -f and -F flags?
```

## Comparison with Other Commands

| Command | Purpose |
|---------|---------|
| `run` | Generate and execute commands |
| `dry-run` | Preview generated commands without execution |
| `chat` | Ask questions and learn about tools |

Use `chat` when you want to:

- Learn about a tool's concepts and options
- Understand best practices and pitfalls
- Get explanations rather than commands
- Have an interactive conversation

Use `run` or `dry-run` when you want to:

- Generate actual command arguments
- Execute a specific task
- Build automation scripts
