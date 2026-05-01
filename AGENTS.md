# oxo-call — Agent Guide

> This file is for AI coding agents. It assumes you know nothing about the project.
> All instructions, comments, and documentation in the codebase are written in **English**.

---

## Project Overview

**oxo-call** is a Rust CLI application that uses LLM intelligence to generate, verify, and execute bioinformatics tool commands from natural-language descriptions.

A user types a task like *"sort input.bam by coordinate and output to sorted.bam"*, and oxo-call:
1. Fetches and caches the tool's `--help` documentation.
2. Loads an expert **skill** file (if available) with tool-specific pitfalls and examples.
3. Sends a structured prompt to an LLM (GitHub Copilot, OpenAI, Anthropic, or local Ollama).
4. Validates the generated command against a parsed CLI schema.
5. Executes the command (or previews it in dry-run mode) and optionally verifies the result.

The project also supports:
- **Workflow engine** — native DAG-based pipelines (`.oxo.toml`) with Snakemake/Nextflow export.
- **Job library** — named command shortcuts with scheduling, history, and LLM generation.
- **Remote execution** — SSH-based execution on workstations and HPC clusters (Slurm, PBS, SGE, LSF).
- **Benchmark suite** — systematic accuracy evaluation across 133+ bioinformatics tools (`oxo-bench`).

Repository: https://github.com/Traitome/oxo-call  
Documentation: https://traitome.github.io/oxo-call/documentation/

---

## Technology Stack

- **Language**: Rust (Edition 2024, requires **Rust 1.85+**).
- **Async runtime**: `tokio` (full features).
- **CLI parsing**: `clap` v4 with derive macros.
- **Serialization**: `serde` + `serde_json` + `toml`.
- **Error handling**: `thiserror` for structured errors, `color-eyre` for enhanced panic/error reporting.
- **HTTP client**: `reqwest` (rustls backend, JSON + streaming).
- **Terminal UX**: `colored`, `indicatif` (spinners), `termimad` (Markdown rendering), `rustyline` (interactive chat).
- **Crypto**: `ed25519-dalek` + `sha2` + `base64` (license signing).
- **Documentation site**: MkDocs Material (Python).

---

## Workspace Structure

This is a Cargo workspace with three members:

```
Cargo.toml          # Workspace root + main package (oxo-call v0.12.1)
crates/
  license-issuer/   # Offline Ed25519 license-signing tool (maintainer-only, publish = false)
  oxo-bench/        # Benchmark/evaluation CLI + library (oxo-bench)
```

The main binary is `oxo-call`. The benchmark binary is `oxo-bench`.

---

## Source Code Organization (`src/`)

Modules are grouped by domain. Most files have extensive doc comments explaining their role.

| Module | Purpose |
|--------|---------|
| `main.rs` | Binary entry point. Parses CLI, validates license, loads config, dispatches to command handlers. |
| `lib.rs` | Library API — re-exports public modules for downstream crates (e.g., `oxo-bench`). |
| `cli.rs` | All clap-derived structs: `Cli`, `Commands`, subcommand enums, and argument definitions. |
| `config.rs` | Configuration loading/saving (`config.toml`), environment variable overrides, config path resolution. |
| `error.rs` | Central `OxoError` enum (`thiserror`) and `Result<T>` alias. |
| `license.rs` | Dual-license verification (Academic/Commercial). Ed25519 signature checking. |
| `llm/` | LLM interaction: prompt building, provider HTTP clients, response parsing, SSE streaming. |
| `runner/` | Execution pipeline: `core` (Runner struct), `batch` (parallel jobs), `retry` (auto-retry on failure). |
| `docs.rs`, `index.rs`, `doc_*.rs` | Documentation fetching, caching, processing, and indexing. |
| `skill.rs` | Skill file loading and management. Built-in skills are embedded; community/user skills live on disk. |
| `workflow.rs`, `workflow_graph.rs`, `pipeline.rs` | Native workflow engine: parse `.oxo.toml`, build DAG, execute, export to Snakemake/Nextflow. |
| `orchestrator/` | Planner → Executor → Validator → Supervisor agent pipeline for advanced generation. |
| `schema/` | CLI schema IR: parse `--help` into structured `CliSchema`, validate commands against it. |
| `confidence/` | Confidence estimation to decide generation strategy (SingleCall / ValidationRetry / ThinkingMode). |
| `knowledge/` | Tool knowledge base, best-practices DB, and error-pattern DB. |
| `execution/` | Result analysis and feedback collection. |
| `history.rs` | Persistent command history with provenance metadata. |
| `job.rs` | Named job library (CRUD + scheduling + execution). |
| `server.rs` | Remote server registry and SSH-based execution. |
| `chat.rs` | Interactive LLM chat mode with Markdown rendering. |
| `copilot_auth.rs` | GitHub OAuth device flow for Copilot authentication. |
| `mcp.rs` | MCP (Model Context Protocol) skill provider client. |
| `handlers.rs` | Utility output helpers used by `main.rs`. |

---

## Build Commands

```bash
# Debug build
cargo build

# Release build (LTO + strip + opt-level 3; configured in Cargo.toml)
cargo build --release

# Build all workspace members
cargo build --workspace

# Install locally from source
cargo install --path .
```

A convenience `Makefile` exists:
```bash
make ci     # Runs: fmt + clippy + build + test
make fmt    # cargo fmt -- --check
make clippy # cargo clippy -- -D warnings
make test   # cargo test -- --test-threads=4
```

---

## Test Commands

```bash
# Run all tests (unit + integration)
cargo test --verbose

# Run a specific integration test file
cargo test --test cli_tests

# Run a single test by exact name
cargo test --test cli_tests test_help_output -- --exact

# Run with limited threads (used by Makefile)
cargo test -- --test-threads=4
```

### Test Structure
- **Unit tests**: Inline `#[cfg(test)] mod tests` inside source files. Very common — there are 70+ such modules.
- **Integration tests**: `tests/cli_tests.rs` — runs the compiled `oxo-call` binary as a subprocess and asserts on stdout/stderr/exit code.
- **Test fixtures**: `tests/fixtures/test_license.oxo.json` — a pre-signed license used by integration tests so core commands can run in CI without a real license.
- **Shared test mutex**: `main.rs` and `lib.rs` both define `ENV_LOCK: std::sync::Mutex<()>` to prevent races when tests modify process-global env vars like `OXO_CALL_DATA_DIR`.

### Dev Dependencies
- `tempfile` — temporary directories in tests.
- `wiremock` — HTTP mocking for LLM provider tests.

---

## Code Style Guidelines

- **Formatting**: Standard `rustfmt`. No custom `rustfmt.toml` or `.rustfmt.toml` — use `cargo fmt`.
- **Linting**: Clippy with `-D warnings` (warnings are errors in CI). Run `cargo clippy -- -D warnings`.
- **Documentation**: Every public module and type should have a `//!` or `///` doc comment.
- **Error handling**: Use `crate::error::{Result, OxoError}`. Prefer `?` propagation. Use `thiserror` derives for new error variants.
- **Async**: `tokio::main` in `main.rs`; most I/O is async. Use `.await?` consistently.
- **Imports**: Group as `std`, then crates, then `crate::` internals.
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/enums, `SCREAMING_SNAKE_CASE` for constants.
- **Comments**: Use `// ── Section ──` dividers in long files (common convention in this codebase).

---

## License System (Critical for Agents)

oxo-call requires a **signed license file** to run core commands.

- **Search order**: `--license <path>` → `OXO_CALL_LICENSE` env var → platform config dir (`directories::ProjectDirs`) → legacy `~/.config/oxo-call/license.oxo.json`.
- **License-exempt commands**: `--help`, `--version`, `license`, and `completion` work without a license.
- **Test fixture**: Integration tests rely on `tests/fixtures/test_license.oxo.json` (set via `OXO_CALL_LICENSE`). If you add new integration tests that call core commands, use the `oxo_call()` helper from `tests/cli_tests.rs` which injects this fixture automatically.
- **Dual license**: Academic (free) and Commercial (paid per organization). Do not remove or bypass license checks.

---

## Configuration & Environment Variables

Config is stored in the platform config directory (e.g., `~/.config/oxo-call/config.toml` on Linux).

Key env vars (all prefixed with `OXO_CALL_`):
- `OXO_CALL_LICENSE` — path to license file.
- `OXO_CALL_LLM_PROVIDER` — override provider (github-copilot, openai, anthropic, ollama).
- `OXO_CALL_LLM_API_TOKEN` — override API token.
- `OXO_CALL_LLM_API_BASE` — override API base URL.
- `OXO_CALL_LLM_MODEL` — override model.
- `OXO_CALL_LLM_MAX_TOKENS`, `OXO_CALL_LLM_TEMPERATURE` — generation params.
- `OXO_CALL_DOCS_AUTO_UPDATE` — boolean, controls auto doc refresh.
- `OXO_CALL_CONFIG_DIR`, `OXO_CALL_DATA_DIR` — used in tests to isolate state.

Legacy provider-specific env vars are also supported for tokens (e.g., `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`) but the `OXO_CALL_`-prefixed versions take precedence.

---

## Skills

Skills are expert knowledge files that improve LLM accuracy. They live in:
- `skills/` (built-in, 160+ files, embedded at compile time).
- User skills directory (`oxo-call skill path` shows the path).
- Community registry (fetchable via `oxo-call skill install <tool>`).
- MCP servers (`oxo-call skill mcp add <url>`).

**Format**: Markdown with YAML front-matter:
```markdown
---
name: samtools
category: alignment
description: ...
tags: [bam, sam, ...]
author: ...
source_url: "..."
---

## Concepts
...

## Pitfalls
...

## Examples
...
```

When adding a new built-in skill, create the `.md` file in `skills/` and register it in `src/skill.rs`.

---

## Workflows

Native workflows use **`.oxo.toml`** format:
```toml
[workflow]
name = "rnaseq"
description = "..."
version = "1.0"

[wildcards]
sample = ["sample1", "sample2"]

[params]
threads = "8"

[[step]]
name = "fastp"
cmd = "fastp --in1 data/{sample}_R1.fastq.gz ..."
inputs  = [...]
outputs = [...]
```

Built-in templates live in `workflows/native/`, with Snakemake and Nextflow equivalents in `workflows/snakemake/` and `workflows/nextflow/`.

---

## CI / CD & Release Process

GitHub Actions (`.github/workflows/ci.yml`):

1. **sync-version** (tag pushes only): bumps `Cargo.toml`, `CITATION.cff`, and `Cargo.lock` to match the git tag.
2. **test**: `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo audit`, and `cargo tarpaulin` (code coverage → Codecov).
3. **build-linux**: Cross-compiles release binaries for `x86_64`, `aarch64`, `armv7` (GNU + MUSL) using `cross`.
4. **build-macos**: Builds for `x86_64-apple-darwin` and `aarch64-apple-darwin`.
5. **build-windows**: Builds for `x86_64`, `aarch64`, `i686` Windows targets.
6. **release** (tags only): Generates changelog with `git-cliff`, creates GitHub Release, attaches tarballs/zip + SHA256SUMS.
7. **publish-crate** (tags only): Publishes `oxo-call` to crates.io.
8. **deploy-pages** (main branch): Builds MkDocs site and deploys to GitHub Pages.

### Release checklist (for maintainers)
- Ensure `CHANGELOG.md` is meaningful; release notes are auto-generated by `git-cliff` from conventional commits.
- Tag format must be `v*`, e.g., `v0.12.1`.

---

## Security Considerations

- **License verification**: Ed25519 signatures prevent tampering with license files. The public key is embedded in `src/license.rs` (`EMBEDDED_PUBLIC_KEY_BASE64`).
- **API tokens**: Stored in `config.toml` in the user's config directory. Never log raw tokens.
- **HTTPS enforcement**: `config verify` warns/fails on insecure `http://` API base URLs for non-local providers.
- **Path traversal**: Tool names are validated before being used as cache filenames (e.g., `../etc/passwd` is rejected).
- **URL schemes**: Only `http://` and `https://` are accepted for remote documentation fetching; `file://` is rejected.
- **Command risk assessment**: The runner assesses risk levels (safe / caution / dangerous) before executing generated commands and warns the user.

---

## Adding New Features — Quick Reference

- **New CLI subcommand/flag**: Add to `src/cli.rs`, then handle in `main.rs`'s `run()` match statement.
- **New config key**: Add to `src/config.rs` (`VALID_CONFIG_KEYS`, structs, and `set`/`get` logic).
- **New skill**: Add Markdown to `skills/`, register in `src/skill.rs`.
- **New workflow template**: Add `.toml` to `workflows/native/`, plus `.smk`/`.nf` if you want pre-built exports.
- **New test**: Prefer inline `#[cfg(test)]` for unit tests; add to `tests/cli_tests.rs` for end-to-end CLI coverage.

---

## Useful Commands for Agents

```bash
# Full local CI gate
make ci

# Run just the binary to see help
 cargo run -- --help

# Run with test license and a simple command
OXO_CALL_LICENSE=tests/fixtures/test_license.oxo.json cargo run -- config show

# Build docs site locally
pip install mkdocs-material
cd docs/guide && mkdocs serve

# Run benchmark suite
cargo run -p oxo-bench -- --help
```
