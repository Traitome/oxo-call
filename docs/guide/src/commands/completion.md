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

The `completion` command generates shell completion scripts that enable tab-completion for all oxo-call commands, subcommands, options, and arguments. This command works **without a valid license file** — you can set up completions before acquiring a license.

## Installation

### Bash

```bash
# Option 1: User-level (recommended)
mkdir -p ~/.local/share/bash-completion/completions
oxo-call completion bash > ~/.local/share/bash-completion/completions/oxo-call

# Option 2: System-wide
oxo-call completion bash | sudo tee /etc/bash_completion.d/oxo-call > /dev/null

# Reload in the current shell
source ~/.local/share/bash-completion/completions/oxo-call
```

### Zsh

```bash
# Step 1: create the fpath directory
mkdir -p ~/.zfunc
oxo-call completion zsh > ~/.zfunc/_oxo-call

# Step 2: add to ~/.zshrc (only if not already present)
grep -q 'fpath=(~/.zfunc' ~/.zshrc || echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
grep -q 'autoload -Uz compinit' ~/.zshrc || echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# Step 3: reload
exec zsh
```

> **Tip:** If you use Oh My Zsh or Prezto, drop the file into the custom
> completions directory they manage instead (usually
> `~/.oh-my-zsh/completions/`).

### Fish

```bash
oxo-call completion fish > ~/.config/fish/completions/oxo-call.fish
# Completions are loaded automatically on the next fish session
```

### PowerShell

```powershell
# Append to your PowerShell profile (runs on every session start)
oxo-call completion powershell >> $PROFILE
# Reload
. $PROFILE
```

### Elvish

```bash
oxo-call completion elvish > ~/.config/elvish/lib/oxo-call.elv
# Import in your rc.elv if not loaded automatically
echo 'use oxo-call' >> ~/.config/elvish/rc.elv
```

## Verification

After installing, open a new shell (or reload the current one) and type:

```bash
oxo-call <TAB>
```

You should see a list of all subcommands. For a deeper check:

```bash
oxo-call job <TAB>        # shows job subcommands
oxo-call run --<TAB>      # shows run flags
oxo-call completion <TAB> # shows shell types
```

## What Gets Completed

- **All commands and subcommands** — `run`, `dry-run`, `docs`, `config`, `job`, `skill`, `workflow`, `server`, `history`, `completion`, etc.
- **Command aliases** — `r` for `run`, `d` for `dry-run`, `j` for `job`, `wf` for `workflow`, `srv` for `server`, etc.
- **Job subcommands** — `add`, `list`, `show`, `run`, `edit`, `rename`, `status`, `history`, `schedule`, `generate`, `import`
- **All flags and options** — `--ask`, `--model`, `--json`, `--no-cache`, `--verify`, `--optimize-task`, `--server`, `--dry-run`, `--tag`, `--builtin`, etc.
- **Shell type values** for the `completion` command itself (`bash`, `zsh`, `fish`, `powershell`, `elvish`)

## Keeping Completions Up to Date

When you upgrade oxo-call, regenerate the completion script to pick up new
commands and flags:

```bash
# Bash
oxo-call completion bash > ~/.local/share/bash-completion/completions/oxo-call

# Zsh
oxo-call completion zsh > ~/.zfunc/_oxo-call
```
