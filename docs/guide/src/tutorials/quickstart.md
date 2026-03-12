# Quick Start

This guide walks you through your first oxo-call session in under 5 minutes.

## Step 1: Install oxo-call

```bash
cargo install oxo-call
```

See the [Installation guide](./installation.md) for alternative methods.

## Step 2: Obtain a License

A signed license file is required for core commands (free for academic use).

```bash
# Apply for a free academic license by emailing license@traitome.com
# See the License Setup guide for details

# Place your license.oxo.json at the default path:
# Linux:   ~/.config/oxo-call/license.oxo.json
# macOS:   ~/Library/Application Support/io.traitome.oxo-call/license.oxo.json

# Or point to it at runtime:
export OXO_CALL_LICENSE=/path/to/license.oxo.json
```

## Step 3: Configure Your LLM

### GitHub Copilot (Default)
```bash
oxo-call config set llm.api_token <your-github-token>
```

### OpenAI
```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token <your-openai-key>
```

### Anthropic
```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token <your-anthropic-key>
```

### Ollama (Local, No Token Needed)
```bash
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2
```

Verify your configuration:
```bash
oxo-call config verify
```

## Step 4: Run Your First Command

### Preview a command (dry-run)
```bash
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
# → samtools sort -o sorted.bam input.bam
```

### Execute a command
```bash
oxo-call run samtools "index sorted.bam"
# → samtools index sorted.bam
```

### Ask for confirmation before executing
```bash
oxo-call run --ask bcftools "call variants from my.bam against ref.fa"
```

## Step 5: Explore More Features

### Check available skills
```bash
oxo-call skill list
```

### View cached documentation
```bash
oxo-call docs list
oxo-call docs show samtools
```

### Review command history
```bash
oxo-call history list
```

## What's Next?

- Learn about [Configuration](./configuration.md) options
- Explore the [Command Reference](../commands/run.md)
- Understand the [Skill System](../reference/skill-system.md)
- Try the [Workflow Engine](../commands/workflow.md) for pipeline automation
