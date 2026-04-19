# config

Read and write LLM and behavior settings.

## Synopsis

```
oxo-call config login
oxo-call config set    <KEY> <VALUE>
oxo-call config get    <KEY>
oxo-call config show
oxo-call config verify
oxo-call config path
oxo-call config model  <SUBCOMMAND>
```

## Subcommands

### `config login`

Interactive OAuth login for GitHub Copilot (recommended for GitHub Copilot users):

```bash
oxo-call config login
```

This command:
1. Initiates OAuth device flow with GitHub
2. Opens a browser window for authentication
3. Waits for you to authorize the application
4. Stores the GitHub App token (`ghu_`) securely
5. Prompts you to select a GitHub Copilot model (default: `gpt-5-mini`, lightweight free tier ⭐)
6. Saves all available models to `llm.models` for quick switching

**Important**: For GitHub Copilot, you must use a GitHub App token (`ghu_`), not a Personal Access Token (`ghp_`). The `config login` command handles this automatically.

### `config set`

Persist a configuration value:

```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token sk-...
oxo-call config set llm.temperature 0.2
```

**Switching providers**: When you switch to a different provider (e.g., from
OpenAI to Ollama), oxo-call gives a context-aware hint:

- Switching **to Ollama**: reminds you that no token is needed and offers to clear the leftover token
- Switching **to OpenAI/Anthropic**: reminds you to set an API token if none is configured

```bash
# Switch to Ollama (no token needed)
oxo-call config set llm.provider ollama
# hint: Switched to Ollama (local inference, no API token needed).
#       Your existing API token is still stored but will be ignored.

# Switch to OpenAI (token required)
oxo-call config set llm.provider openai
# hint: Provider 'openai' requires an API token.
#       Set one with: oxo-call config set llm.api_token <your-token>
```

### `config get`

Show the effective value (with environment variable overrides applied):

```bash
oxo-call config get llm.provider
```

### `config show`

Display all stored and effective values side-by-side:

```bash
oxo-call config show
```

### `config verify`

Make a real API call to confirm the LLM configuration works:

```bash
oxo-call config verify
```

### `config path`

Print the path to `config.toml`:

```bash
oxo-call config path
```

### `config model`

Manage the configured model list and switch the active model without re-running login.
The active model is displayed with a ★ marker.

```bash
# List configured models
oxo-call config model list

# Add a model to your list
oxo-call config model add gpt-5.4

# Switch the active model
oxo-call config model use gpt-4.1        # alias: switch
oxo-call config model switch gemini-2.5-pro

# Remove a model from the list
oxo-call config model remove claude-sonnet-4
```

After `config login`, the full list of supported GitHub Copilot models is automatically populated.
You can use `config model add <id>` to add models not included in the login selection (e.g., preview models).

### Streaming Configuration

Enable or disable SSE streaming for LLM responses:

```bash
# Disable streaming globally (useful for CI/batch scripts and benchmarks)
oxo-call config set llm.stream false

# Re-enable streaming (default)
oxo-call config set llm.stream true

# Check the current setting
oxo-call config get llm.stream
```

You can also disable streaming per-invocation with `--no-stream` on any LLM-backed command.

## Configuration Reference

See the [Configuration tutorial](../tutorials/configuration.md) for complete details on all keys, defaults, and environment variables.

## Security

The config file (`config.toml`) may contain sensitive data such as API tokens.
On Unix systems, oxo-call automatically sets file permissions to `0600` (owner
read/write only) when saving the configuration. This prevents other users on the
same system from reading your API token.

You can also use the `OXO_CALL_LLM_API_TOKEN` environment variable to provide
the API token without storing it in the config file at all — this is recommended
for shared environments and CI/CD pipelines:

```bash
export OXO_CALL_LLM_API_TOKEN="sk-your-token"
oxo-call run samtools "sort input.bam"
```
