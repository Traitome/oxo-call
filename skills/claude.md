---
name: claude
category: ai-assistant
description: Anthropic Claude AI assistant CLI (Claude Code) — interactive REPL, one-shot queries, and agentic coding tasks from the terminal
tags: [ai, assistant, anthropic, llm, coding, chat, repl, claude-code]
author: oxo-call built-in
source_url: "https://docs.anthropic.com/en/docs/claude-code/overview"
---

## Concepts

- Claude Code is Anthropic's official CLI for interacting with Claude models from the terminal; install with `npm install -g @anthropic-ai/claude-code` and authenticate once with `claude login`.
- Two main modes: **interactive REPL** (`claude` with no args) for multi-turn conversations, and **print mode** (`claude -p "prompt"`) for non-interactive single-turn output suitable for scripting and pipelines.
- The **`--model` flag** selects the Claude model (e.g., `claude-opus-4-5`, `claude-sonnet-4-5`, `claude-haiku-4-5`); omit to use the account default.
- **Conversation continuity**: `-c` resumes the most recent conversation; `-r <session-id>` resumes a specific past session; use `claude sessions list` to see available sessions.
- **Output formats**: `--output-format text` (default), `--output-format json` (structured output with metadata), and `--output-format stream-json` (newline-delimited JSON for streaming).
- **System prompt override**: `--system-prompt "instructions"` replaces the built-in system prompt; useful for constraining Claude to a specific role or persona in scripts.
- **File and stdin input**: pipe content directly with `cat file | claude -p "summarise this"` or use `--input-file path` to read from a file; Claude reads the piped content as context.
- **Agentic tools**: in interactive mode Claude can read/write files, run shell commands, and browse the web; use `/tools` to list enabled tools and `/settings` to enable or disable them.
- **Context window management**: use `/clear` to reset the conversation context; use `/compact` to summarise and compress long conversations while retaining key information.
- **`ANTHROPIC_API_KEY`** environment variable can be set as an alternative to `claude login`; useful in CI/CD pipelines and server environments.

## Pitfalls

- running Claude in agentic mode with write/execute tools enabled allows it to modify files and run commands; always review the proposed plan before confirming destructive or irreversible actions.
- `claude -p` print mode is non-interactive and exits after one response; do NOT use it for multi-turn tasks — use the interactive REPL (`claude`) instead.
- Long conversations accumulate tokens and may hit context limits; use `/compact` or start a fresh session with `/clear` to stay within the model's context window.
- The `--output-format json` flag changes the exit code behaviour: the process always exits 0 even on model errors; check the `error` field in the JSON response when scripting.
- Piping large files directly into the context may exhaust the context window; prefer summarising or chunking large inputs before sending them to Claude.
- Authentication tokens set via `ANTHROPIC_API_KEY` take precedence over the `claude login` session; mismatched keys can cause unexpected billing or rate-limit errors.
- On headless servers the interactive REPL requires a proper TTY; use `claude -p "..."` (print mode) or pipe input via stdin for non-interactive server usage.

## Examples

### start an interactive chat session with no tool access
**Args:** `--no-tools`
**Explanation:** launches the interactive REPL with all tools disabled; safe for pure conversational use without file or shell access

### ask a one-shot question and print the answer
**Args:** `-p "What is the difference between RNA-seq and scRNA-seq?"`
**Explanation:** -p (print mode) sends a single prompt and prints the response to stdout then exits; suitable for scripting and chaining with other tools

### summarise a file using stdin
**Args:** `-p "Summarise this file in three bullet points"`
**Explanation:** pipe a file into claude with cat and use -p to get a summary; the file content is passed as context along with the prompt

### ask Claude to explain a bash script
**Args:** `-p "Explain what this script does" --input-file analyse_samples.sh`
**Explanation:** --input-file reads the script and provides it as context; useful for code review, documentation, or learning unfamiliar scripts

### use a specific model (Opus) for a complex task
**Args:** `-p "Design a nextflow pipeline for bulk RNA-seq analysis" --model claude-opus-4-5`
**Explanation:** --model selects the Claude variant; claude-opus-4-5 is the most capable model, suitable for complex reasoning and design tasks

### output the response as JSON for scripting
**Args:** `-p "List five popular bioinformatics tools in JSON array format" --output-format json`
**Explanation:** --output-format json returns structured output including the model response and metadata; parse with jq for downstream processing

### continue the most recent conversation
**Args:** `-c`
**Explanation:** -c resumes the last session in interactive mode; useful for picking up a previous coding or analysis task without re-explaining context

### list past sessions and resume a specific one
**Args:** `sessions list`
**Explanation:** prints all saved sessions with their IDs and timestamps; follow with 'claude -r <session-id>' to resume a specific session

### set model via environment and run a quick query
**Args:** `-p "Generate a Python one-liner to count lines in a FASTQ file"`
**Explanation:** run after exporting ANTHROPIC_MODEL=claude-haiku-4-5 to pick a faster/cheaper model; -p streams the answer to stdout for immediate use

### check the installed version
**Args:** `--version`
**Explanation:** prints the installed Claude Code CLI version; useful for debugging and ensuring the latest release is active
