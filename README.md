# oxo-call

**Model-intelligent orchestration for CLI bioinformatics**

`oxo-call` is a Rust CLI tool that uses LLM intelligence to help you call bioinformatics (and other command-line) tools without memorizing every flag and parameter. Simply describe your task in plain language, and `oxo-call` will automatically fetch the tool's documentation and generate the right command for you.

## Features

- 🧠 **LLM-powered parameter generation** — describe what you want to do, get the right flags
- 📚 **Automatic documentation fetching** — grabs `--help` output and optionally remote docs
- 🗂️ **Local documentation index** — pre-index tools for faster repeated use
- 🔍 **Dry-run mode** — preview commands before executing
- 📜 **Command history** — track every run with exit codes and timestamps
- 🔧 **Flexible LLM backend** — GitHub Copilot (default), OpenAI, Anthropic, Ollama

## Quick Start

### 1. Install

```bash
cargo install --path .
```

### 2. Configure your LLM token

**GitHub Copilot (default):**
```bash
oxo-call config set llm.api_token <your-github-token>
# Or set the GITHUB_TOKEN environment variable
```

**OpenAI:**
```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token <your-openai-key>
# Or set OPENAI_API_KEY
```

**Anthropic:**
```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token <your-anthropic-key>
# Or set ANTHROPIC_API_KEY
```

**Ollama (local, no token needed):**
```bash
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2
```

### 3. Build a documentation index (optional but recommended)

```bash
oxo-call index add samtools
oxo-call index add bwa
oxo-call index add bcftools
```

### 4. Run with natural language!

```bash
# Preview the command (dry-run)
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
# → samtools sort -o sorted.bam input.bam

# Execute the command
oxo-call run bwa "align reads.fastq to reference.fa using 8 threads, output SAM"

# Auto-confirm without prompting
oxo-call run -y bcftools "call variants from my.bam against ref.fa and output to variants.vcf"
```

## Subcommands

### `run` — Execute a tool with LLM-generated parameters

```
oxo-call run [OPTIONS] <TOOL> <TASK>
oxo-call r   [OPTIONS] <TOOL> <TASK>   # short alias

Options:
  -y, --yes   Execute without confirmation prompt
```

Example:
```bash
oxo-call run samtools "view only primary alignments from file.bam and save to primary.bam"
```

---

### `dry-run` — Preview the command without executing

```
oxo-call dry-run <TOOL> <TASK>
oxo-call d      <TOOL> <TASK>   # short alias
```

Example:
```bash
oxo-call dry-run bwa "align paired reads R1.fastq R2.fastq to hg38.fa using 16 threads"
```

---

### `index` — Manage the local documentation index

Pre-indexing tools speeds up repeated use and works even when the tool is not installed.

```
oxo-call index add    <TOOL> [--url <URL>]   # Index a tool (fetches --help + optional URL)
oxo-call index remove <TOOL>                 # Remove a tool from the index
oxo-call index update [TOOL] [--url <URL>]   # Update one or all indexed tools
oxo-call index list                          # Show all indexed tools
```

Examples:
```bash
# Index from --help output
oxo-call index add samtools

# Index from --help + a remote URL (e.g. man page or docs site)
oxo-call index add bwa --url https://bio-bwa.sourceforge.net/bwa.shtml

# Just from a remote URL (for tools not installed locally)
oxo-call index add gatk --url https://gatk.broadinstitute.org/hc/en-us/articles/...

# Update all indexed tools
oxo-call index update
```

---

### `docs` — View or fetch documentation

```
oxo-call docs show  <TOOL>         # Show cached documentation for a tool
oxo-call docs fetch <TOOL> <URL>   # Fetch and cache docs from a URL
oxo-call docs path  <TOOL>         # Show the path to the cached docs file
```

Examples:
```bash
oxo-call docs show samtools
oxo-call docs fetch bwa https://bio-bwa.sourceforge.net/bwa.shtml
```

---

### `config` — Manage configuration

```
oxo-call config set  <KEY> <VALUE>   # Set a config value
oxo-call config get  <KEY>           # Get a config value
oxo-call config show                 # Show all settings
oxo-call config path                 # Show config file path
```

**Config keys:**

| Key | Default | Description |
|-----|---------|-------------|
| `llm.provider` | `github-copilot` | LLM provider: `github-copilot`, `openai`, `anthropic`, `ollama` |
| `llm.api_token` | *(env var)* | API token (or use env var) |
| `llm.api_base` | *(auto)* | Override API base URL |
| `llm.model` | *(auto)* | Model name (e.g. `gpt-4o`, `claude-3-5-sonnet-20241022`) |
| `llm.max_tokens` | `2048` | Maximum tokens to generate |
| `llm.temperature` | `0.2` | Temperature (lower = more deterministic) |
| `docs.auto_update` | `true` | Auto-refresh docs on first use |

---

### `history` — Command execution history

```
oxo-call history list [-n <N>] [--tool <TOOL>]   # Show recent history
oxo-call history clear [-y]                       # Clear all history
```

Examples:
```bash
oxo-call history list           # Show last 20 commands
oxo-call history list -n 50    # Show last 50 commands
oxo-call history list --tool samtools  # Filter by tool
```

## Environment Variables

| Variable | Used for |
|----------|----------|
| `GITHUB_TOKEN` | GitHub Copilot API token |
| `GH_TOKEN` | GitHub token (fallback) |
| `OPENAI_API_KEY` | OpenAI API token |
| `ANTHROPIC_API_KEY` | Anthropic API token |
| `OXO_API_TOKEN` | Generic fallback token |

## Configuration File

The config file is stored at a platform-appropriate location:
- **Linux**: `~/.config/oxo-call/config.toml`
- **macOS**: `~/Library/Application Support/io.traitome.oxo-call/config.toml`
- **Windows**: `%APPDATA%\traitome\oxo-call\config\config.toml`

Find it with: `oxo-call config path`

## Data Storage

Documentation cache and history are stored at:
- **Linux**: `~/.local/share/oxo-call/`
- **macOS**: `~/Library/Application Support/io.traitome.oxo-call/`
- **Windows**: `%APPDATA%\traitome\oxo-call\data\`

## Building from Source

```bash
git clone https://github.com/Traitome/oxo-call
cd oxo-call
cargo build --release
# Binary is at target/release/oxo-call
```

## Running Tests

```bash
cargo test
```

## License

MIT
