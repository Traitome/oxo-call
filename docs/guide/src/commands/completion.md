# completion

Generate shell completion scripts for oxo-call.

## Synopsis

```
oxo-call completion <SHELL>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<SHELL>` | Shell to generate completions for: `bash`, `zsh`, `fish`, `powershell`, `elvish` |

## Description

The `completion` command generates shell completion scripts that enable tab-completion for all oxo-call commands, subcommands, options, and arguments. This command works without a valid license file.

## Installation

### Bash

```bash
# Option 1: User-level (recommended)
mkdir -p ~/.local/share/bash-completion/completions
oxo-call completion bash > ~/.local/share/bash-completion/completions/oxo-call

# Option 2: System-wide
oxo-call completion bash | sudo tee /etc/bash_completion.d/oxo-call > /dev/null
```

### Zsh

```bash
# Add to your fpath (e.g. in ~/.zshrc: fpath=(~/.zfunc $fpath))
mkdir -p ~/.zfunc
oxo-call completion zsh > ~/.zfunc/_oxo-call

# Then reload completions
autoload -Uz compinit && compinit
```

### Fish

```bash
oxo-call completion fish > ~/.config/fish/completions/oxo-call.fish
```

### PowerShell

```powershell
# Add to your PowerShell profile
oxo-call completion powershell >> $PROFILE
```

### Elvish

```bash
oxo-call completion elvish > ~/.config/elvish/lib/oxo-call.elv
```

## What Gets Completed

- All commands and subcommands (`run`, `dry-run`, `docs`, `config`, `job`, etc.)
- Command aliases (`r` for `run`, `d` for `dry-run`, `j` for `job`, `wf` for `workflow`, etc.)
- All flags and options (`--ask`, `--model`, `--json`, `--no-cache`, etc.)
- Shell type values for the `completion` command itself
