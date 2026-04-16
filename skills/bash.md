---
name: bash
category: programming
description: Bash shell interpreter and scripting language; pipeline automation, job control, file management, and text processing
tags: [bash, shell, scripting, pipeline, automation, cron, configuration, bashrc, profile]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/bash/manual/"
---

## Concepts
- Bash configuration files (sourced in order): `~/.bash_profile` (login shells), `~/.bashrc` (interactive non-login shells), `~/.profile` (POSIX fallback).
- `/etc/environment`, `/etc/profile`, and `/etc/profile.d/*.sh` set system-wide environment variables.
- `~/.bash_history` stores command history; controlled by `HISTSIZE` (in-memory entries) and `HISTFILESIZE` (file entries).
- `~/.bash_aliases` is often sourced in `~/.bashrc` for alias definitions; cleaner than putting aliases directly in `.bashrc`.
- Shebang line `#!/usr/bin/env bash` is more portable than `#!/bin/bash` as it respects the user's PATH.
- `set -euo pipefail` is the safest script preamble: `-e` exits on error, `-u` errors on unset variables, `-o pipefail` catches pipe failures.
- Process substitution `<(cmd)` creates a virtual file from a command's stdout; useful with tools that require file arguments.
- `$()` is the modern form of command substitution (preferred over backticks).
- Here-documents (`<<EOF`) pass multi-line strings as stdin to commands without creating temporary files.
- Arrays: `arr=("a" "b" "c")`; iterate with `for elem in "${arr[@]}"`; associative arrays (bash 4+): `declare -A map; map[key]=value`.
- `source file` (or `. file`) runs a script in the current shell; changes to environment variables persist.
- `export VAR=value` makes a variable available to child processes.
- Bash on macOS is version 3.2 (very old); install modern bash 5+ via `brew install bash` or use zsh.
- Parameter expansion: `${var:-default}` uses default if var unset; `${var%.ext}` strips suffix; `${var#*/}` strips prefix; `${#var}` is string length.
- Redirections: `> file` stdout, `2> file` stderr, `&> file` both, `2>&1` redirect stderr to stdout, `>> file` append.
- `xargs -I{} cmd {}` converts stdin lines into command arguments; `parallel` (GNU parallel) is more powerful for parallel execution.

## Pitfalls
- CRITICAL: When using bash in oxo-call, ARGS is the argument string passed to bash. Use `-c 'command'` for inline commands, or just the script path for script execution.
- omitting quotes around variables (`rm $file` vs `rm "$file"`) causes word splitting and glob expansion on filenames with spaces.
- `set -e` alone does not catch failures in conditionals or pipelines; always pair with `-o pipefail`.
- `/bin/sh` on Ubuntu/Debian is dash, not bash; scripts using bash-specific syntax (`[[ ]]`, arrays) will fail with `#!/bin/sh`.
- Variables are global by default; use `local varname` inside functions to avoid leaking into the parent scope.
- `~` inside double quotes is NOT expanded to the home directory; use `$HOME` instead.
- Modifying `~/.bash_profile` vs `~/.bashrc`: login shell vs interactive shell distinction bites many users; on macOS Terminal, every shell is a login shell.
- `export PATH="$PATH:/new/dir"` must be in `~/.bash_profile` (or `~/.profile`) to persist across login sessions, not just in `~/.bashrc`.
- `[[ ]]` is bash-specific; `[ ]` is POSIX. Use `[[ ]]` for pattern matching (`[[ $var == pattern* ]]`) and `&&`/`||` inside tests.
- `echo` has inconsistent behavior across systems (especially with `-e`, `-n`); use `printf` for portability.

## Examples

### run a bash script
**Args:** `script.sh arg1 arg2`
**Explanation:** executes script.sh with arguments; script must be executable (chmod +x) or invoked directly via bash

### run a script with strict error handling
**Args:** `-euo pipefail -c 'command1 | command2'`
**Explanation:** -e exits on any error; -u treats unset variables as errors; pipefail catches pipeline failures; -c runs the inline command

### source a configuration file into the current shell
**Args:** `-c 'source ~/.bashrc && printenv'`
**Explanation:** sources .bashrc to load aliases and functions, then prints the environment; useful for debugging PATH/environment issues

### check bash version
**Args:** `--version`
**Explanation:** prints the bash version; check whether bash 4+ features (associative arrays, etc.) are available on the system

### run a script and print each command as it executes (debugging)
**Args:** `-x script.sh`
**Explanation:** -x enables xtrace mode; prints each command with expanded values before executing; invaluable for debugging pipeline scripts

### list all loaded functions and aliases
**Args:** `-c 'declare -f; alias'`
**Explanation:** declare -f prints all shell functions; alias prints all aliases; useful for auditing what a sourced config file adds

### execute a command in a subshell without affecting the current environment
**Args:** `-c 'export MY_VAR=test && echo $MY_VAR'`
**Explanation:** -c runs the string in a new bash subshell; changes to variables do not affect the calling shell

### write a multi-command pipeline using process substitution
**Args:** `-c 'diff <(sort file1.txt) <(sort file2.txt)'`
**Explanation:** process substitution <(cmd) creates a virtual file from command output; diff compares sorted versions of two files without creating temp files

### run a background pipeline job and capture its PID
**Args:** `-c 'long_running_command &; PID=$!; wait $PID; echo "exit: $?"'`
**Explanation:** & backgrounds the command; $! captures its PID; wait blocks until it finishes; $? captures the exit code

### loop over a list of files and process each
**Args:** `-c 'for f in *.bam; do samtools flagstat "$f" > "${f%.bam}.stats"; done'`
**Explanation:** glob expands *.bam; ${f%.bam} strips the suffix for naming output files; quoting "$f" handles spaces in filenames

### redirect both stdout and stderr to a log file
**Args:** `-c 'command &> output.log'`
**Explanation:** &> redirects both stdout and stderr to output.log; use >> for append mode; equivalent to > file 2>&1

### use parameter expansion with defaults
**Args:** `-c 'output_dir="${1:-results}"; mkdir -p "$output_dir"'`
**Explanation:** ${1:-results} uses the first argument if provided, otherwise defaults to "results"; robust for scripts with optional arguments

### pipe find results into xargs for batch processing
**Args:** `-c 'find . -name "*.fastq.gz" -print0 | xargs -0 -I{} fastqc {}'`
**Explanation:** find -print0 and xargs -0 handle filenames with spaces safely; -I{} substitutes each filename into the command

### iterate over lines in a file
**Args:** `-c 'while IFS= read -r line; do echo "Processing: $line"; done < input.txt'`
**Explanation:** IFS= prevents trimming whitespace; -r prevents backslash interpretation; < redirects file as stdin to the while loop
