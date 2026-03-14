# skill

List, show, or manage expert knowledge profiles for bioinformatics tools.

## Synopsis

```
oxo-call skill list
oxo-call skill show     <TOOL>
oxo-call skill install  <TOOL> [--url <URL>]
oxo-call skill remove   <TOOL>
oxo-call skill create   <TOOL> [-o <FILE>]
oxo-call skill path
```

## Description

Skills are Markdown files with YAML front-matter that inject **domain-expert knowledge** into the LLM prompt for a specific tool. They contain key concepts, common pitfalls, and worked command examples. When oxo-call finds a matching skill, it includes this knowledge in the prompt, dramatically improving accuracy.

## Subcommands

### `skill list`

List all available skills (built-in, community, user-defined):

```bash
oxo-call skill list
```

### `skill show`

Display the full skill content for a tool:

```bash
oxo-call skill show samtools
```

### `skill install`

Install a community skill or from a custom URL:

```bash
oxo-call skill install bismark
oxo-call skill install mytool --url https://example.com/mytool.md
```

Both `.md` (YAML front-matter + Markdown, preferred) and legacy `.toml` formats are supported.

### `skill remove`

Remove a community or user-installed skill:

```bash
oxo-call skill remove mytool
```

### `skill create`

Generate a skill Markdown template:

```bash
oxo-call skill create mytool
oxo-call skill create mytool -o ~/.config/oxo-call/skills/mytool.md
```

### `skill path`

Show the user skills directory path:

```bash
oxo-call skill path
```

## Skill Load Priority

1. **User-defined**: `~/.config/oxo-call/skills/<tool>.md` (`.toml` also accepted)
2. **Community-installed**: `~/.local/share/oxo-call/skills/<tool>.md` (`.toml` also accepted)
3. **Built-in**: Compiled into the binary

## Skill File Format

Skills use a Markdown file with YAML front-matter:

```markdown
---
name: mytool
category: alignment
description: One-line description of the tool
tags: [bam, sam, ngs]
author: your-name   # optional
source_url: https://tool-docs.example.com   # optional
---

## Concepts

- Key concept 1 about the tool
- Key concept 2 about the tool

## Pitfalls

- Common mistake 1 and how to avoid it
- Common mistake 2 and how to avoid it

## Examples

### description of what to do
**Args:** `--flag1 --flag2 input output`
**Explanation:** why these flags were chosen
```

## Built-in Skill Coverage

oxo-call ships with 150+ built-in skills covering all major omics domains. See [Skill System Reference](../reference/skill-system.md) for the full list.

