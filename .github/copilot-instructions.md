# Copilot Instructions for `oxo-call`

## ⚠️ Mandatory pre-commit CI gate

**Before every call to `report_progress`, ALL of the following checks MUST pass locally with zero errors.**
Pushing code that fails any of these checks will break the CI "Test" job and is not acceptable.

```bash
# Option A – run everything in one command (preferred):
make ci

# Option B – run each step individually:
cargo fmt -- --check          # formatting (MUST pass – most commonly forgotten)
cargo clippy -- -D warnings   # zero lint warnings allowed
cargo build                   # must compile
cargo test                    # all unit + integration tests must pass
```

If `cargo fmt -- --check` reports diff output, fix it first with `cargo fmt` and re-run the check.
**Never call `report_progress` until `make ci` (or all four individual commands) exits with code 0.**

## Build, test, lint (individual command reference)

The `make ci` target above runs the four mandatory checks. The full reference, including the additional security audit step used in CI, is:

```bash
cargo build          # build
cargo test           # run all tests (unit + integration)
cargo fmt -- --check # check formatting
cargo clippy -- -D warnings  # lint (zero warnings allowed)
cargo audit          # security audit (run when changing dependencies)
```

Integration tests live in `tests/cli_tests.rs` and execute the compiled binary. They inject a fixture license via `OXO_CALL_LICENSE`.

## Key source files

| File | Purpose |
|------|---------|
| `src/cli.rs` | Clap command definitions |
| `src/main.rs` | Command dispatcher + license gate |
| `src/runner/` | Command generation: docs → skill → LLM → execute → verify |
| `src/llm/` | LLM client: provider abstraction, prompt templates, streaming |
| `src/command_pipeline.rs` | Single/multi-stage command generation pipeline |
| `src/docs.rs` | Documentation resolver + caching |
| `src/skill.rs` | Built-in and user skill loading |
| `src/flag_extractor.rs` | Pattern-based flag extraction from --help |
| `src/tool_resolver.rs` | Dynamic tool resolution with alias mapping |
| `src/history.rs` | JSONL command history with provenance |
| `src/license.rs` | Ed25519 offline license verification |

## Critical conventions

**License gate** — All commands except `license`, `--help`, and `--version` require a valid license. Never bypass the gate in `src/main.rs`.

**Docs-first grounding** — `Runner::prepare()` fetches tool docs *before* loading a skill or calling the LLM. Preserve this order.

**LLM response format** — `src/llm/` expects `ARGS:` and `EXPLANATION:` lines and retries on invalid format. The LLM roles (command generation, task optimization, result verification) each use a dedicated system prompt.

**Skill precedence** — `user > community > mcp > built-in`. Reuse `SkillManager`; do not add ad-hoc skill loading. Use `load_async()` in async contexts (runner, CLI commands) and `load()` only where sync is unavoidable.

**MCP skill provider** — `src/mcp.rs` implements the minimal MCP JSON-RPC client (HTTP POST transport, no SSE). `McpServerConfig` is defined in `config.rs`. MCP servers are registered via `skill mcp add <url>` and stored under `[[mcp.servers]]` in `config.toml`.

**Keep issuer in sync** — `crates/license-issuer/src/main.rs` signs the same payload that `src/license.rs` verifies. Schema changes require edits in both.

**Integration tests** — Use real binary + fixture license, not mocked internals.

## Adding / editing things

**New flag on `run` or `dry-run`:**
1. `src/cli.rs` — add the field to `Commands::Run` / `Commands::DryRun`
2. `src/main.rs` — destructure and pass to `Runner`
3. `src/runner/core.rs` — add field + builder method, wire into `run()` / `dry_run()`
4. `tests/cli_tests.rs` — add a parse test and/or help-text test
5. `docs/guide/src/commands/run.md` or `dry-run.md` — update options table and examples

**New command:**
1–4 same as above, plus `docs/guide/src/SUMMARY.md` if it's a top-level command.

**New built-in skill:**
1. `skills/<tool>.md` — YAML front-matter (`name`, `category`, `description`, `tags`, `author`, `source_url`) + `## Concepts` + `## Pitfalls` + `## Examples` sections (≥3 concepts, ≥3 pitfalls, ≥5 examples). Each example: `### task` → `**Args:** \`flags\`` → `**Explanation:** text`
2. `src/skill.rs` — add to `BUILTIN_SKILLS` array with `include_str!`

**Scope boundary:**
- Do not add a native workflow format, workflow templates, a DAG engine, or
  scheduler behavior to oxo-call.
- Keep `run` and `dry-run` focused on one reviewable tool invocation. The
  command-generation pipeline may use several LLM calls internally (e.g., task
  normalization + generation), but must not become multi-step execution.
- Send dependency-aware workflows, environments, resume behavior, and resource
  scheduling to [oxo-flow](https://github.com/Traitome/oxo-flow).

## Documentation

MkDocs source lives under `docs/guide/src/`.

```bash
cd docs/guide && mkdocs build   # build
cd docs/guide && mkdocs serve   # live preview
```

When CLI behavior changes, update `docs/guide/src/commands/<command>.md`. When architecture changes, update `docs/guide/src/reference/*.md`.
