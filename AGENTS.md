# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Test, Lint Commands

```bash
# Full CI quality gate (run before committing)
make ci

# Individual commands:
cargo build                    # build
cargo test                      # all tests (unit + integration)
cargo test --test cli_tests     # integration tests only
cargo test --test cli_tests test_help_output -- --exact  # single test
cargo fmt -- --check            # check formatting
cargo fmt                       # apply formatting
cargo clippy -- -D warnings     # lint (zero warnings allowed)
cargo audit                     # security audit
```

## Architecture Overview

oxo-call is an AI-powered CLI assistant for bioinformatics that translates natural-language tasks into grounded CLI commands.

**Core flow:**
```
User task → Docs fetch → Skill load → LLM prompt → Command generation → Execution → Verification
```

**Key modules:**
- `src/main.rs` — Entry point; license gate + command dispatch
- `src/cli.rs` — Clap command tree (run, dry-run, docs, config, skill, workflow, etc.)
- `src/runner/core.rs` — Core orchestration: docs → skill → LLM → execute
- `src/llm/provider.rs` — LLM client (GitHub Copilot, OpenAI, Anthropic, Ollama)
- `src/llm/prompt.rs` — Prompt builders; expects `ARGS:` and `EXPLANATION:` lines
- `src/docs.rs` — Documentation resolver (cache, `--help`, remote URLs)
- `src/skill.rs` — Skill system: built-in (158 tools), community, user skills
- `src/workflow.rs` — DAG workflow engine with Snakemake/Nextflow export
- `src/config.rs` — Platform-aware TOML config
- `src/license.rs` — Offline Ed25519 license verification
- `src/history.rs` — JSONL command history with provenance

**Workspace crates:**
- `crates/license-issuer/` — License signing tool (maintainer-only)
- `crates/oxo-bench/` — Benchmarking/evaluation framework

## Critical Conventions

**License gate** — All commands except `license`, `--help`, `--version` require a valid license. Never bypass the gate in `src/main.rs`.

**Docs-first grounding** — `Runner::prepare()` fetches tool docs *before* loading a skill or calling the LLM. Preserve this order.

**LLM response format** — `src/llm/provider.rs` expects `ARGS:` and `EXPLANATION:` lines and retries on invalid format.

**Skill precedence** — `user > community > mcp > built-in`. Use `SkillManager::load_async()` in async contexts; `load()` only where sync is unavoidable.

**Keep issuer in sync** — `crates/license-issuer/src/main.rs` signs the same payload that `src/license.rs` verifies. Schema changes require edits in both.

**Integration tests** — Use real binary + fixture license (`OXO_CALL_LICENSE` env var), not mocked internals.

## Adding New Features

### New CLI flag
1. Add field to `Commands::Run` / `Commands::DryRun` in `src/cli.rs`
2. Destructure and pass to `Runner` in `src/main.rs`
3. Add builder method in `src/runner/core.rs`, wire into `run()` / `dry_run()`
4. Add parse test in `tests/cli_tests.rs`
5. Update `docs/guide/src/commands/*.md`

### New built-in skill
1. Create `skills/<tool>.md` with YAML front-matter (`name`, `category`, `description`, `tags`, `author`, `source_url`)
2. Add sections: `## Concepts` (≥3), `## Pitfalls` (≥3), `## Examples` (≥5)
3. Each example: `### task` → `**Args:** \`flags\`` → `**Explanation:** text`
4. Register in `src/skill.rs` `BUILTIN_SKILLS` array: `builtin!("toolname")`

### New workflow template
1. Create `workflows/native/<name>.oxo.toml`
2. Optionally add `workflows/snakemake/<name>.smk` and `workflows/nextflow/<name>.nf`
3. Register in `src/workflow.rs` `BUILTIN_TEMPLATES` constant
4. Add test in `tests/cli_tests.rs`

## Skill Format

```markdown
---
name: toolname
category: alignment
description: One-line summary
tags: [bam, alignment]
author: oxo-call built-in
source_url: https://docs.example.com
---

## Concepts
- Key concept 1
- Key concept 2

## Pitfalls
- Common mistake 1
- Common mistake 2

## Examples

### Task description
**Args:** `sort -@ 4 -o output.bam input.bam`
**Explanation:** explanation text
```

## Testing

Integration tests in `tests/cli_tests.rs` execute the compiled binary with fixture license via `OXO_CALL_LICENSE` env var.

Unit tests are `#[cfg(test)]` modules inside source files.

## Documentation

MkDocs source in `docs/guide/src/`. Build with:
```bash
cd docs/guide && mkdocs serve  # live preview
cd docs/guide && mkdocs build  # static output
```