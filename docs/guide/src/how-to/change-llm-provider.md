# How-to: Switch LLM Provider

This guide shows you how to change the LLM backend that oxo-call uses for command generation. All providers use the same prompt and produce compatible output.

---

## Supported Providers

| Provider | Models | Requires token | Best for |
|----------|--------|----------------|----------|
| `github-copilot` | auto-selected | GitHub PAT with Copilot | GitHub users, no separate account |
| `openai` | gpt-4.1 (1M), gpt-4o, gpt-4o-mini, ... | OpenAI API key | Best accuracy, production use |
| `anthropic` | claude-sonnet-4-6 (1M), claude-3-5-sonnet, ... | Anthropic API key | Alternative frontier model |
| `deepseek` | deepseek-v3, deepseek-r1 (128K) | DeepSeek API key | Cost-effective, strong reasoning |
| `minimax` | minimax-m2.7 (1M) | MiniMax API key | Large context, Chinese-optimized |
| `ollama` | llama3.2, mistral, ... | None (local) | Air-gapped, private data, free |
| `openai` (custom base) | moonshot-v1-*, kimi-* | Moonshot API key | Kimi / Moonshot AI |
| `openai` (custom base) | glm-4, glm-5.1, ... | ZhipuAI API key | GLM / ZhipuAI |

---

## GitHub Copilot (Default)

GitHub Copilot is the default provider. The recommended way to authenticate is using the interactive OAuth login:

```bash
# Interactive OAuth login (recommended)
oxo-call config login
```

This will:
1. Open a browser window for GitHub authentication
2. Complete OAuth device flow automatically
3. Store the token securely in your config
4. Prompts you to choose a model (default: `gpt-5-mini`, lightweight free tier ⭐)
5. Saves all GA models to `llm.models` for quick switching

### Supported GitHub Copilot Models

The following GA models are available as of the latest GitHub Copilot docs.
Use `oxo-call config model use <id>` to switch between them at any time.

| Model ID | Display Name | Provider |
|----------|-------------|----------|
| `gpt-5-mini` ⭐ | GPT-5 Mini | OpenAI |
| `gpt-4.1` | GPT-4.1 | OpenAI |
| `gpt-5.2` | GPT-5.2 | OpenAI |
| `gpt-5.2-codex` | GPT-5.2-Codex | OpenAI |
| `gpt-5.3-codex` | GPT-5.3-Codex | OpenAI |
| `gpt-5.4` | GPT-5.4 | OpenAI |
| `gpt-5.4-mini` | GPT-5.4 Mini | OpenAI |
| `claude-haiku-4.5` | Claude Haiku 4.5 | Anthropic |
| `claude-sonnet-4` | Claude Sonnet 4 | Anthropic |
| `claude-sonnet-4.5` | Claude Sonnet 4.5 | Anthropic |
| `claude-sonnet-4.6` | Claude Sonnet 4.6 | Anthropic |
| `claude-opus-4.5` | Claude Opus 4.5 | Anthropic |
| `claude-opus-4.6` | Claude Opus 4.6 | Anthropic |
| `gemini-2.5-pro` | Gemini 2.5 Pro | Google |

⭐ = available on all Copilot plans including free tier.
For the full authoritative list, see the [GitHub Copilot supported models](https://docs.github.com/en/copilot/reference/ai-models/supported-models) documentation.

### Manual Token Setup

Alternatively, you can set a GitHub token manually:

```bash
# Set via config
oxo-call config set llm.provider github-copilot
oxo-call config set llm.api_token ghu_xxxxxxxxxxxxxxxxxxxx

# Verify
oxo-call config verify
```

**Important**: For GitHub Copilot, you must use a GitHub App token (starts with `ghu_`), not a Personal Access Token (starts with `ghp_`). The `oxo-call config login` command handles this automatically.

### Environment Variables (Not Recommended for Copilot)

For other providers, environment variables work well, but for GitHub Copilot, environment variables like `GITHUB_TOKEN` often contain Personal Access Tokens that don't work with Copilot's token exchange endpoint. Therefore, GitHub Copilot **ignores** these environment variables and only uses the stored config token from `oxo-call config login`:

```bash
# These are IGNORED for github-copilot provider:
# export GITHUB_TOKEN=ghp_xxxx  # Won't work
# export OXO_CALL_LLM_API_TOKEN=ghp_xxxx  # Won't work

# Instead, use:
oxo-call config login
```

### Get a Token Manually

If you need to obtain a token manually:

1. Use `oxo-call config login` (recommended)
2. Or use the GitHub CLI: `gh auth token` (returns a `ghu_` token if you have Copilot access)

---

## OpenAI

```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token sk-xxxxxxxxxxxxxxxxxxxxxxxx

# Optional: specify a model
oxo-call config set llm.model gpt-4.1       # 1M context (default, April 2025)
oxo-call config set llm.model gpt-4o-mini   # faster, cheaper
oxo-call config set llm.model gpt-4o        # 128K context

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
oxo-call config set llm.model gpt-4.1
```

---

## Anthropic

```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token sk-ant-xxxxxxxxxxxxxxxxxxxxxxxx

# Optional: specify a model
oxo-call config set llm.model claude-sonnet-4-6-20250514   # 1M context (default)
oxo-call config set llm.model claude-3-5-sonnet-20241022   # 200K context

# Verify
oxo-call config verify
```

Environment variable fallback:

```bash
export ANTHROPIC_API_KEY=sk-ant-xxxx
```

---

## DeepSeek

[DeepSeek](https://platform.deepseek.com/) provides cost-effective models with strong reasoning capabilities. DeepSeek V3 and R1 support 128K context.

```bash
oxo-call config set llm.provider deepseek
oxo-call config set llm.api_token sk-xxxxxxxxxxxxxxxxxxxxxxxx

# Optional: specify a model
oxo-call config set llm.model deepseek-chat      # General purpose (default)
oxo-call config set llm.model deepseek-reasoner  # Enhanced reasoning

# Verify
oxo-call config verify
```

| Model | Context | Best for |
|-------|---------|----------|
| `deepseek-chat` | 128K | General purpose, cost-effective |
| `deepseek-reasoner` | 128K | Complex reasoning tasks |

---

## MiniMax

[MiniMax](https://www.minimaxi.com/) provides models optimized for Chinese language with large context windows (up to 1M tokens).

```bash
oxo-call config set llm.provider minimax
oxo-call config set llm.api_token xxxxxxxxxxxxxxxxxxxxxxxx

# Optional: specify a model
oxo-call config set llm.model MiniMax-Text-01   # General purpose (default)
oxo-call config set llm.model abab6.5s-chat     # Fast variant

# Verify
oxo-call config verify
```

| Model | Context | Best for |
|-------|---------|----------|
| `MiniMax-Text-01` | 1M | General purpose, large context |
| `abab6.5s-chat` | 245K | Fast, cost-effective |

---

## Kimi / Moonshot AI

[Moonshot AI](https://platform.moonshot.cn/) provides the Kimi model family via an OpenAI-compatible API. Kimi models feature large context windows (up to 256K with K2.5) and strong multilingual capabilities.

```bash
# Option 1: Use dedicated provider (recommended)
oxo-call config set llm.provider moonshot
oxo-call config set llm.api_token sk-xxxxxxxxxxxxxxxxxxxxxxxx

# Option 2: Use openai provider with custom base
oxo-call config set llm.provider openai
oxo-call config set llm.api_base https://api.moonshot.cn/v1
oxo-call config set llm.api_token sk-xxxxxxxxxxxxxxxxxxxxxxxx

# Choose a context-window variant
oxo-call config set llm.model moonshot-v1-8k     # 8K context (faster)
oxo-call config set llm.model moonshot-v1-32k    # 32K context
oxo-call config set llm.model moonshot-v1-128k   # 128K context (recommended)
oxo-call config set llm.model kimi-k2.5          # 256K context (latest)

# Verify
oxo-call config verify
```

| Model | Context | Best for |
|-------|---------|----------|
| `moonshot-v1-8k` | 8K | Fast, simple tasks |
| `moonshot-v1-32k` | 32K | Most tasks |
| `moonshot-v1-128k` | 128K | Long documentation, complex tasks |
| `kimi-k2.5` | 256K | Latest, largest context |

---

## GLM / ZhipuAI

[ZhipuAI](https://open.bigmodel.cn/) provides the GLM series via an OpenAI-compatible API. GLM-5 supports up to 200K context, and GLM-5.1 supports 202K context.

```bash
# Option 1: Use dedicated provider (recommended)
oxo-call config set llm.provider zhipu
oxo-call config set llm.api_token xxxxxxxxxxxxxxxxxxxxxxxx

# Option 2: Use openai provider with custom base
oxo-call config set llm.provider openai
oxo-call config set llm.api_base https://open.bigmodel.cn/api/paas/v4
oxo-call config set llm.api_token xxxxxxxxxxxxxxxxxxxxxxxx

# Choose a model
oxo-call config set llm.model glm-4           # Standard, 128K context
oxo-call config set llm.model glm-4-flash     # Faster/cheaper variant
oxo-call config set llm.model glm-4-long      # 1M token context (long documents)
oxo-call config set llm.model glm-5           # 200K context
oxo-call config set llm.model glm-5.1         # 202K context

# Verify
oxo-call config verify
```

| Model | Context | Best for |
|-------|---------|----------|
| `glm-4` | 128K | General use |
| `glm-4-flash` | 128K | Cost-sensitive workflows |
| `glm-4-long` | 1M | Very long documentation or multi-tool sessions |
| `glm-5` | 200K | Latest generation |
| `glm-5.1` | 202K | Autonomous execution, large context |

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
oxo-call config set llm.api_base http://my-ollama-server:11434/v1
```

> **Note**: If you were previously using OpenAI or another provider, the old API
> token is safely ignored when using Ollama. oxo-call will remind you of this
> when switching. To remove it: `oxo-call config set llm.api_token ""`

### Recommended models for bioinformatics

| Model | Size | Tier | Notes |
|-------|------|------|-------|
| `qwen2.5-coder:0.5b` | 0.5B | Compact | Fastest, good for simple tasks (83–100% accuracy) |
| `deepseek-coder:1.3b` | 1.3B | Compact | Good balance of speed and accuracy |
| `llama3.2` | 3B | Compact | Best accuracy among ≤3B models (100%) |
| `starcoder2:3b` | 3B | Compact | Good for code tasks (91%) |
| `llama3.1:8b` | 8B | Medium | Better accuracy for complex tasks |
| `mistral` | 7B | Medium | Good instruction following |
| `codellama:13b` | 13B | Full | Best for complex technical commands |
| `qwen2.5-coder:7b` | 7B | Medium | Excellent for bioinformatics commands |

Models ≤3B automatically use the **Compact** prompt tier (few-shot examples,
minimal context). Models 4–7B use the **Medium** tier. Models ≥8B use the
**Full** tier. You can override this with `llm.prompt_tier`.

### Small model configuration

For models ≤3B (e.g., `qwen2.5-coder:0.5b`, `llama3.2`), the Compact tier
is automatically selected. If you want to force a specific tier:

```bash
# Force Compact tier (recommended for ≤3B models)
oxo-call config set llm.prompt_tier compact

# Force Medium tier (for 4–7B models with limited context)
oxo-call config set llm.prompt_tier medium

# Auto-detect based on model size and context window (default)
oxo-call config set llm.prompt_tier auto
```

Or per-invocation:

```bash
OXO_CALL_LLM_PROMPT_TIER=compact oxo-call dry-run samtools "sort bam"
```

---

## Verify Your Configuration

After setting up any provider:

```bash
oxo-call config verify
```

This tests connectivity and returns the model being used:

```
✓ LLM provider: openai
✓ Model: gpt-4.1
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
export OXO_CALL_LLM_API_BASE=http://shared-ollama-server:11434/v1
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
