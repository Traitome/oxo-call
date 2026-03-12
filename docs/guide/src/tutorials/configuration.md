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
