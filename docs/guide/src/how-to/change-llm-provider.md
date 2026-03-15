# How-to: Switch LLM Provider

This guide shows you how to change the LLM backend that oxo-call uses for command generation. All providers use the same prompt and produce compatible output.

---

## Supported Providers

| Provider | Models | Requires token | Best for |
|----------|--------|----------------|----------|
| `github-copilot` | auto-selected | GitHub PAT with Copilot | GitHub users, no separate account |
| `openai` | gpt-4o, gpt-4o-mini, ... | OpenAI API key | Best accuracy, production use |
| `anthropic` | claude-3-5-sonnet, ... | Anthropic API key | Alternative frontier model |
| `ollama` | llama3.2, mistral, ... | None (local) | Air-gapped, private data, free |

---

## GitHub Copilot (Default)

GitHub Copilot is the default provider. You need a GitHub personal access token (PAT) with the `copilot` scope, or use a token from `gh auth token`.

```bash
# Set via config
oxo-call config set llm.provider github-copilot
oxo-call config set llm.api_token ghp_xxxxxxxxxxxxxxxxxxxx

# Or use environment variables
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
# or
export GH_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
```

Get a token:

1. Go to [github.com/settings/tokens](https://github.com/settings/tokens)
2. Create a new token with `read:user` and GitHub Copilot access
3. Or use the GitHub CLI: `gh auth token`

---

## OpenAI

```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token sk-xxxxxxxxxxxxxxxxxxxxxxxx

# Optional: specify a model
oxo-call config set llm.model gpt-4o-mini   # faster, cheaper
oxo-call config set llm.model gpt-4o        # higher accuracy

# Verify
oxo-call config verify
```

Environment variable fallbacks:

```bash
export OXO_CALL_LLM_PROVIDER=openai
export OXO_CALL_LLM_API_TOKEN=sk-xxxx
# or the standard OpenAI variable:
export OPENAI_API_KEY=sk-xxxx
```

### Azure OpenAI

```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_base https://your-resource.openai.azure.com/openai/deployments/your-deployment
oxo-call config set llm.api_token your-azure-key
oxo-call config set llm.model gpt-4o
```

---

## Anthropic

```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token sk-ant-xxxxxxxxxxxxxxxxxxxxxxxx

# Optional: specify a model
oxo-call config set llm.model claude-3-5-sonnet-20241022

# Verify
oxo-call config verify
```

Environment variable fallback:

```bash
export ANTHROPIC_API_KEY=sk-ant-xxxx
```

---

## Ollama (Local, No Token)

Ollama runs models locally — no API key or internet required. Ideal for sensitive data or air-gapped environments.

### Install and start Ollama

```bash
# Install Ollama (Linux/macOS)
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama3.2

# Start the server (usually auto-started)
ollama serve
```

### Configure oxo-call

```bash
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2

# Custom Ollama server URL (if not localhost)
oxo-call config set llm.api_base http://my-ollama-server:11434
```

### Recommended models for bioinformatics

| Model | Size | Notes |
|-------|------|-------|
| `llama3.2` | 3B | Fast, good for simple tasks |
| `llama3.1:8b` | 8B | Better accuracy, still fast |
| `mistral` | 7B | Good instruction following |
| `codellama:13b` | 13B | Best for technical commands |

---

## Verify Your Configuration

After setting up any provider:

```bash
oxo-call config verify
```

This tests connectivity and returns the model being used:

```
✓ LLM provider: openai
✓ Model: gpt-4o-mini
✓ Connection: OK
```

---

## Compare Providers Side-by-Side

Run the same dry-run with different providers to compare output:

```bash
# Test with current provider
oxo-call dry-run samtools "sort input.bam by coordinate using 4 threads"

# Switch temporarily via environment variable
OXO_CALL_LLM_PROVIDER=ollama OXO_CALL_LLM_MODEL=llama3.2 \
  oxo-call dry-run samtools "sort input.bam by coordinate using 4 threads"
```

---

## Air-Gapped / Offline Mode

oxo-call can run completely offline with no external network calls. This requires:

1. **Ollama for local LLM inference** — no API key or internet needed
2. **Pre-cached documentation** — tool `--help` output is cached after first use
3. **Offline license verification** — Ed25519 verification is entirely local

### Complete offline setup

```bash
# 1. Install Ollama and pull a model (requires internet once)
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llama3.2

# 2. Configure oxo-call for offline use
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2

# 3. Pre-cache documentation for your tools (requires internet once)
oxo-call docs add samtools
oxo-call docs add bcftools
oxo-call docs add fastp
# ... add all tools you plan to use

# 4. Verify offline readiness
oxo-call config verify      # Should show Ollama connection OK
oxo-call docs list           # Should show cached tools
oxo-call license verify      # Should pass without network
```

After this setup, disconnect from the network. All subsequent `oxo-call run` and `dry-run` commands will work offline using cached documentation and local Ollama inference.

### What requires network access

| Feature | Requires Network | Offline Alternative |
|---------|:---:|---|
| LLM inference (GitHub/OpenAI/Anthropic) | ✅ | Use Ollama |
| LLM inference (Ollama) | ❌ | Already local |
| `docs add --url` (remote fetch) | ✅ | Use `docs add --file` or `docs add --dir` |
| `docs add` (first-time `--help` capture) | ❌ | Tool must be installed locally |
| License verification | ❌ | Already offline |
| `skill install --url` | ✅ | Copy skill files manually |

---

## Team Setup / Organizational Deployment

### Sharing configuration across a team

You can standardize oxo-call settings across your team using environment variables or shared configuration:

```bash
# Option 1: Shared environment variables (recommended for clusters)
# Add to your team's shared .bashrc or module file:
export OXO_CALL_LLM_PROVIDER=ollama
export OXO_CALL_LLM_API_BASE=http://shared-ollama-server:11434
export OXO_CALL_LICENSE=/shared/licenses/license.oxo.json

# Option 2: Shared skill directory
# Place team-specific skills in a shared location and symlink:
ln -s /shared/oxo-call/skills/ ~/.config/oxo-call/skills
```

### Sharing custom skills

Distribute team skills via a shared directory or Git repository:

```bash
# Team lead creates skills
oxo-call skill create internal-tool -o /shared/oxo-call/skills/internal-tool.md
# Edit the skill file with team-specific conventions

# Team members install
cp /shared/oxo-call/skills/*.md ~/.config/oxo-call/skills/
# Or symlink the entire directory
```

### Multi-user license

A single commercial license covers all employees and contractors within the organization. Distribute the `license.oxo.json` file to team members via:

- Shared filesystem path (`export OXO_CALL_LICENSE=/shared/license.oxo.json`)
- Configuration management (Ansible, Puppet, etc.)
- Container image with pre-installed license

---

## Troubleshooting

**"Connection refused" for Ollama**

Make sure the Ollama server is running: `ollama serve`

**"Invalid API key" errors**

Check that the token is set correctly:

```bash
oxo-call config get llm.api_token
oxo-call config show
```

**LLM output is incorrect or hallucinated**

Try a more capable model, or enrich the documentation:

```bash
oxo-call docs update <tool>
oxo-call docs add <tool> --url <docs-url>
```

**Rate limiting**

For high-volume use, consider:

- Using a local Ollama model (no rate limits)
- Increasing retry settings
- Caching `--help` output to reduce API calls (done automatically after first run)
