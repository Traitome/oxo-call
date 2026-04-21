# chat

Interactive chat with AI about bioinformatics tools and general topics.

## Synopsis

```
oxo-call chat [OPTIONS] [QUESTION]
oxo-call chat [OPTIONS] <TOOL> <QUESTION>
oxo-call chat -i [OPTIONS]
oxo-call c   [OPTIONS] [QUESTION]
```

## Options

| Option | Description |
|--------|-------------|
| `-i`, `--interactive` | Start interactive multi-turn chat session |
| `-m`, `--model <MODEL>` | Override the LLM model for this invocation |
| `--no-cache` | Skip cached documentation and fetch fresh `--help` output |
| `--scenario <SCENARIO>` | Context injection mode: `bare`, `prompt`, `skill`, `doc`, `full` (default: `full`) |
| `--json` | Output result as JSON (non-interactive mode only) |
| `--no-stream` | Disable streaming (SSE) output from the LLM |
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

The `chat` command provides three modes for interacting with AI:

### General Q&A (Non-interactive, no tool required)

Ask any question — about shell commands, file operations, concepts, research, or bioinformatics — without specifying a tool:

```bash
oxo-call chat "How do I create temp files starting with result in the current directory?"
oxo-call chat "What is the difference between paired-end and single-end sequencing?"
```

In general mode a broad system prompt is used that covers shell scripting, OS resources, bioinformatics, programming, and general research.

### Tool-specific Q&A (Non-interactive)

Ask a single question about a specific tool and get an immediate response:

```bash
oxo-call chat samtools "How do I sort a BAM file?"
```

In single-shot mode:

- A **spinner** is displayed while fetching context and waiting for the LLM response
- The response is **rendered as formatted Markdown** in the terminal (headings, bold, code blocks, lists, etc.)
- Use `--json` for machine-readable output

### Interactive Multi-turn Chat

Start an interactive session for extended conversations:

```bash
oxo-call chat -i
```

In interactive mode:

- Questions are answered in **general mode** by default (no tool context required)
- Use `/tool <name>` to switch to tool-specific mode with full documentation context
- Change scenarios with `/scenario <mode>`
- Clear conversation history with `/clear`
- View conversation message count with `/history`
- Display or change the LLM model with `/model [name]`
- Exit with `/quit` or `Ctrl+D`

**Progress indicators**: A spinner is shown while the LLM is generating a response, so you know the system is working. Errors are caught gracefully — if a request fails, an error message is shown and the conversation continues.

**Markdown rendering**: LLM responses are rendered with terminal-friendly formatting (headings, bold, italic, code blocks, lists).

## Interactive Commands

When in interactive mode (`-i`), the following commands are available:

| Command | Description |
|---------|-------------|
| `/tool <name>` | Set tool context for subsequent questions |
| `/clear` | Clear conversation history |
| `/history` | Show conversation message count |
| `/model [name]` | Display or change the current LLM model |
| `/scenario <s>` | Change scenario mode (bare/prompt/skill/doc/full) |
| `/help` | Show available commands |
| `/quit` or `Ctrl+D` | Exit the chat |

## Examples

### General Conversation (No Tool Required)

```bash
# Ask about shell commands
oxo-call chat "How do I create temp files starting with result in the current directory?"

# Ask a general bioinformatics question
oxo-call chat "What is the difference between paired-end and single-end sequencing?"

# Ask about programming and scripting
oxo-call chat "How do I write a bash loop to process all FASTQ files in a directory?"

# Get JSON output for scripting
oxo-call chat --json "Explain the FASTQ format"
```

### Tool-specific Q&A

```bash
# Ask about a specific tool
oxo-call chat samtools "Explain the difference between SAM and BAM formats"

# Ask about common pitfalls (skill mode)
oxo-call chat --scenario skill bwa "What are common pitfalls when using BWA?"

# Ask with documentation context only
oxo-call chat --scenario doc gatk "How do I call variants with HaplotypeCaller?"

# Use a different LLM model
oxo-call chat --model gpt-4 samtools "How to extract unmapped reads from a BAM file?"

# Get JSON output for scripting
oxo-call chat --json samtools "What does the -F flag do?"
```

### Interactive Mode

```bash
# Start interactive chat (general mode by default)
oxo-call chat -i

# Start with a pre-set tool context
oxo-call chat -i --tool samtools

# Start with a specific scenario
oxo-call chat -i --scenario skill
```

### Interactive Session Example

```
$ oxo-call chat -i

  ╔══════════════════════════════════════════════════════════╗
  ║ 🧬 oxo-call Interactive Chat                            ║
  ╚══════════════════════════════════════════════════════════╝

  Commands:
    /tool <name>   Set tool context for subsequent questions
    /clear         Clear conversation history
    /history       Show conversation message count
    /model [name]  Display or change the current LLM model
    /scenario <s>  Change scenario (bare|prompt|skill|doc|full)
    /help          Show this help message
    /quit, Ctrl+D  Exit the chat

  Usage:
    <question>       Ask any question (general mode, no tool context required)
    /tool <name>     Set a tool context, then ask tool-specific questions

oxo▶ How do I create some temporary files starting with "result" in the current directory?
⠋ Thinking...

──────────────────────────────────────────────────────────────
You can use a shell loop or brace expansion:

    # Brace expansion (bash/zsh)
    touch result_{1..5}.tmp

    # Or with a loop
    for i in $(seq 1 5); do touch "result_${i}.tmp"; done

──────────────────────────────────────────────────────────────

oxo▶ /tool samtools
  ✔ Tool context set to: samtools

▶ samtools How do I sort a BAM file by coordinate?
⠋ Thinking...

──────────────────────────────────────────────────────────────
To sort a BAM file by coordinate using samtools:

    samtools sort -o sorted.bam input.bam

For multi-threading:

    samtools sort -@ 8 -o sorted.bam input.bam
──────────────────────────────────────────────────────────────

▶ samtools /quit
👋 Goodbye!
```

## Use Cases

### General Research and Learning

Use `chat` without a tool name to ask any question directly in the terminal:

```bash
oxo-call chat "How do I use awk to extract columns from a TSV file?"
oxo-call chat "Explain the concept of read depth in sequencing"
oxo-call chat "What is the recommended way to handle paired-end FASTQ files in bash?"
```

### Learning a Specific Tool

Use `--scenario skill` to focus on expert knowledge and common pitfalls:

```bash
oxo-call chat --scenario skill minimap2 "What are the key concepts I should understand?"
```

### Quick Reference

Use `--scenario doc` to get answers based on the tool's official documentation:

```bash
oxo-call chat --scenario doc bcftools "What filtering options are available?"
```

### Debugging and Troubleshooting

Interactive mode is ideal for iterative debugging:

```bash
oxo-call chat -i --tool samtools
▶ samtools Why am I getting empty output from my view command?
▶ samtools How can I check if my BAM file is corrupted?
▶ samtools What's the difference between -f and -F flags?
```

## Comparison with Other Commands

| Command | Purpose |
|---------|---------|
| `run` | Generate and execute commands |
| `dry-run` | Preview generated commands without execution |
| `chat` | Ask questions and learn about tools or any topic |

Use `chat` when you want to:

- Ask general questions about shell, scripting, or research without leaving the CLI
- Learn about a tool's concepts and options
- Understand best practices and pitfalls
- Get explanations rather than commands
- Have an interactive conversation

Use `run` or `dry-run` when you want to:

- Generate actual command arguments
- Execute a specific task
- Build automation scripts
