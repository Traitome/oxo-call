# Copilot Instructions for `oxo-call`

## Build, test, lint

```bash
cargo build          # build
cargo test           # run all tests (unit + integration)
cargo fmt -- --check # check formatting
cargo clippy -- -D warnings  # lint (zero warnings allowed)
cargo audit          # security audit
```

Integration tests live in `tests/cli_tests.rs` and execute the compiled binary. They inject a fixture license via `OXO_CALL_LICENSE`.

## Key source files

| File | Purpose |
|------|---------|
| `src/cli.rs` | Clap command definitions |
| `src/main.rs` | Command dispatcher + license gate |
| `src/runner.rs` | Core orchestration: docs → skill → LLM → execute → (verify) |
| `src/llm.rs` | LLM client: command generation, task optimization, result verification |
| `src/engine.rs` | DAG workflow engine for `.oxo.toml` files |
| `src/docs.rs` | Documentation resolver + caching |
| `src/skill.rs` | Built-in and user skill loading |
| `src/history.rs` | JSONL command history with provenance |
| `src/license.rs` | Ed25519 offline license verification |

## Critical conventions

**License gate** — All commands except `license`, `--help`, and `--version` require a valid license. Never bypass the gate in `src/main.rs`.

**Docs-first grounding** — `Runner::prepare()` fetches tool docs *before* loading a skill or calling the LLM. Preserve this order.

**LLM response format** — `src/llm.rs` expects `ARGS:` and `EXPLANATION:` lines and retries on invalid format. The three LLM roles (command generation, `--optimize-task`, `--verify`) each use a dedicated system prompt.

**Skill precedence** — `user > community > built-in`. Reuse `SkillManager`; do not add ad-hoc skill loading.

**Keep issuer in sync** — `crates/license-issuer/src/main.rs` signs the same payload that `src/license.rs` verifies. Schema changes require edits in both.

**Integration tests** — Use real binary + fixture license, not mocked internals.

## Adding / editing things

**New flag on `run` or `dry-run`:**
1. `src/cli.rs` — add the field to `Commands::Run` / `Commands::DryRun`
2. `src/main.rs` — destructure and pass to `Runner`
3. `src/runner.rs` — add field + builder method, wire into `run()` / `dry_run()`
4. `tests/cli_tests.rs` — add a parse test and/or help-text test
5. `docs/guide/src/commands/run.md` or `dry-run.md` — update options table and examples

**New command:**
1–4 same as above, plus `docs/guide/src/SUMMARY.md` if it's a top-level command.

**New built-in skill:**
1. `skills/<tool>.md` — YAML front-matter (`name`, `category`, `description`, `tags`, `author`, `source_url`) + `## Concepts` + `## Pitfalls` + `## Examples` sections (≥3 concepts, ≥3 pitfalls, ≥5 examples). Each example: `### task` → `**Args:** \`flags\`` → `**Explanation:** text`
2. `src/skill.rs` — add to `BUILTIN_SKILLS` array with `include_str!`

**New workflow template:**
1. `workflows/native/<name>.oxo.toml`
2. Optionally `workflows/snakemake/<name>.smk` and `workflows/nextflow/<name>.nf`
3. `src/workflow.rs` — add to `BUILTIN_TEMPLATES`
4. `tests/cli_tests.rs` — add parse + expand test

## Documentation

mdBook source lives under `docs/guide/src/`.

```bash
cd docs/guide && mdbook build   # build
cd docs/guide && mdbook serve   # live preview
```

When CLI behavior changes, update `docs/guide/src/commands/<command>.md`. When architecture changes, update `docs/guide/src/reference/*.md`.
