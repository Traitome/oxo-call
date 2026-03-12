# docs

Manage tool documentation — add, remove, update, list, show, or inspect cached documentation.

## Synopsis

```
oxo-call docs add    <TOOL> [--url <URL>] [--file <PATH>] [--dir <DIR>]
oxo-call docs remove <TOOL>
oxo-call docs update [TOOL] [--url <URL>]
oxo-call docs list
oxo-call docs show   <TOOL>
oxo-call docs path   <TOOL>
```

## Subcommands

### `docs add`

Index a tool's documentation. Sources can be combined:

```bash
# From --help output (usually automatic on first run)
oxo-call docs add samtools

# Enrich with a remote documentation URL
oxo-call docs add bwa --url https://bio-bwa.sourceforge.net/bwa.shtml

# From a local file
oxo-call docs add mytool --file /path/to/manual.md

# From a directory of docs
oxo-call docs add mytool --dir /path/to/docs/
```

### `docs remove`

Remove cached documentation for a tool:

```bash
oxo-call docs remove samtools
```

### `docs update`

Refresh documentation:

```bash
# Update a specific tool
oxo-call docs update samtools

# Update all indexed tools
oxo-call docs update

# Update with a new remote URL
oxo-call docs update bwa --url https://new-docs.example.com
```

### `docs list`

List all indexed tools with metadata:

```bash
oxo-call docs list
```

### `docs show`

Display the cached documentation for a tool:

```bash
oxo-call docs show samtools
```

### `docs path`

Show the filesystem path to a tool's cached documentation:

```bash
oxo-call docs path samtools
```

## How Documentation Works

1. On first use of a tool, oxo-call automatically runs `<tool> --help` and caches the output
2. Additional documentation sources (URLs, files, directories) are merged with the help output
3. Documentation is deduplicated (80% overlap detection) and validated (80–16K characters)
4. The combined documentation is sent to the LLM as grounding context
