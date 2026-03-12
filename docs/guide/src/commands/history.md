# history

Browse past command runs with exit codes and timestamps.

## Synopsis

```
oxo-call history list  [-n <N>] [--tool <TOOL>]
oxo-call history clear [-y]
```

## Subcommands

### `history list`

Show recent command history:

```bash
# Last 20 commands (default)
oxo-call history list

# Last 50 commands
oxo-call history list -n 50

# Filter by tool name
oxo-call history list --tool samtools
```

### `history clear`

Clear all command history:

```bash
oxo-call history clear
oxo-call history clear -y  # Skip confirmation
```

## History Entry Format

Each history entry includes:
- **ID**: Unique UUID
- **Tool**: The CLI tool that was executed
- **Task**: The natural language task description
- **Command**: The full generated command
- **Exit code**: The process exit code
- **Timestamp**: When the command was executed
- **Dry-run flag**: Whether it was a dry-run or real execution

## Storage

History is stored as JSONL (JSON Lines) at:
- Linux: `~/.local/share/oxo-call/history.jsonl`
- macOS: `~/Library/Application Support/io.traitome.oxo-call/history.jsonl`
