# config

Read and write LLM and behavior settings.

## Synopsis

```
oxo-call config set    <KEY> <VALUE>
oxo-call config get    <KEY>
oxo-call config show
oxo-call config verify
oxo-call config path
```

## Subcommands

### `config set`

Persist a configuration value:

```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token sk-...
oxo-call config set llm.temperature 0.2
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

## Configuration Reference

See the [Configuration tutorial](../tutorials/configuration.md) for complete details on all keys, defaults, and environment variables.
