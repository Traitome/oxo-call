# Configuration

oxo-call uses a layered configuration system with sensible defaults, file-based overrides, and environment variable support.

## Configuration File

Settings are stored in a TOML file at the platform-specific configuration directory:

| Platform | Path |
|----------|------|
| Linux | `~/.config/oxo-call/config.toml` |
| macOS | `~/Library/Application Support/io.traitome.oxo-call/config.toml` |
| Windows | `%APPDATA%\traitome\oxo-call\config.toml` |

Find your config path:

```bash
oxo-call config path
```

## Configuration Keys

| Key | Default | Environment Variable | Description |
|-----|---------|---------------------|-------------|
| `llm.provider` | `github-copilot` | `OXO_CALL_LLM_PROVIDER` | LLM provider |
| `llm.api_token` | *(unset)* | `OXO_CALL_LLM_API_TOKEN` | API token |
| `llm.api_base` | *(auto)* | `OXO_CALL_LLM_API_BASE` | Override API base URL |
| `llm.model` | *(auto)* | `OXO_CALL_LLM_MODEL` | Model name |
| `llm.max_tokens` | `2048` | `OXO_CALL_LLM_MAX_TOKENS` | Maximum tokens |
| `llm.temperature` | `0.0` | `OXO_CALL_LLM_TEMPERATURE` | Temperature (0.0 = deterministic) |
| `docs.auto_update` | `true` | `OXO_CALL_DOCS_AUTO_UPDATE` | Auto-refresh docs on first use |

## Setting Values

```bash
# Set a value
oxo-call config set llm.provider openai
oxo-call config set llm.api_token sk-...

# Get the effective value (includes env overrides)
oxo-call config get llm.provider

# Show all configuration
oxo-call config show

# Verify LLM connectivity
oxo-call config verify
```

## Environment Variables

Environment variables override `config.toml` values. Provider-specific token variables are also supported as fallbacks:

- **GitHub**: `GITHUB_TOKEN`, `GH_TOKEN`
- **OpenAI**: `OPENAI_API_KEY`
- **Anthropic**: `ANTHROPIC_API_KEY`

## LLM Provider Details

### GitHub Copilot (Default)
- Default model: auto-selected
- API base: `https://api.githubcopilot.com`
- Token: GitHub personal access token with Copilot access

### OpenAI
- Default model: `gpt-4o`
- API base: `https://api.openai.com/v1`
- Compatible with Azure OpenAI via `llm.api_base` override

### Anthropic
- Default model: `claude-3-5-sonnet-20241022`
- API base: `https://api.anthropic.com`

### Ollama
- Default model: `llama3.2`
- API base: `http://localhost:11434`
- No API token required (local inference)

---

## Troubleshooting

### Wrong or Missing License

If your license file is missing, expired, or invalid, you will see an error like:

```
Error: License verification failed — no valid license found.
Checked locations (in order):
  1. --license CLI flag
  2. OXO_CALL_LICENSE environment variable
  3. ~/.config/oxo-call/license.oxo.json

Run `oxo-call license verify` for details.
```

**Fix:** Ensure your `license.oxo.json` is at one of the checked paths. See [License Setup](./license.md).

### Failed LLM Connection

If `oxo-call config verify` fails with a connection error:

```
✗ LLM provider: openai
✗ Connection: Failed — could not reach https://api.openai.com/v1
```

**Common causes:**
- **No API token**: Run `oxo-call config get llm.api_token` to check
- **Wrong provider**: Verify with `oxo-call config get llm.provider`
- **Network issue**: Check internet connectivity or proxy settings
- **Ollama not running**: Start with `ollama serve`

### Config File Not Found

```
Config file not found at ~/.config/oxo-call/config.toml
Using default values.
```

This is normal on first use. Set your first value to create the file:

```bash
oxo-call config set llm.provider openai
```

### CI / HPC Cluster Considerations

When running oxo-call in non-interactive environments (CI pipelines, SLURM job scripts, HPC clusters):

1. **License**: Set `OXO_CALL_LICENSE` to the path of your license file in your job script or CI environment
2. **API tokens**: Use environment variables instead of config files:
   ```bash
   export OXO_CALL_LLM_PROVIDER=openai
   export OXO_CALL_LLM_API_TOKEN=$OPENAI_API_KEY
   ```
3. **No `GITHUB_TOKEN`**: If your CI environment does not set `GITHUB_TOKEN`, switch to OpenAI, Anthropic, or Ollama
4. **Ollama on clusters**: Run Ollama as a service on a shared node, then set `llm.api_base` to point to it:
   ```bash
   export OXO_CALL_LLM_PROVIDER=ollama
   export OXO_CALL_LLM_API_BASE=http://ollama-node:11434
   ```
5. **SLURM example**:
   ```bash
   #!/bin/bash
   #SBATCH --job-name=oxo-call-pipeline
   #SBATCH --cpus-per-task=8
   
   export OXO_CALL_LICENSE=/shared/licenses/license.oxo.json
   export OXO_CALL_LLM_PROVIDER=ollama
   export OXO_CALL_LLM_API_BASE=http://ollama-node:11434
   
   oxo-call run samtools "sort input.bam by coordinate using 8 threads"
   ```
