# Contributing to oxo-call

Thank you for your interest in improving **oxo-call**! This guide covers
everything you need to get started: setting up a development environment,
understanding the codebase, adding skills and workflows, and submitting
high-quality pull requests.

---

## Table of contents

- [Contributing to oxo-call](#contributing-to-oxo-call)
  - [Table of contents](#table-of-contents)
  - [Development setup](#development-setup)
    - [Prerequisites](#prerequisites)
    - [Clone and build](#clone-and-build)
    - [Run the test suite](#run-the-test-suite)
    - [Lint and format](#lint-and-format)
    - [Install locally](#install-locally)
  - [Project structure](#project-structure)
    - [Execution flow](#execution-flow)
  - [Adding a new skill](#adding-a-new-skill)
    - [1. Create the Markdown file](#1-create-the-markdown-file)
    - [2. Register the skill in `src/skill.rs`](#2-register-the-skill-in-srcskillrs)
    - [3. Verify](#3-verify)
  - [Adding workflow templates](#adding-workflow-templates)
  - [Code style](#code-style)
  - [Testing](#testing)
    - [Integration tests](#integration-tests)
    - [Unit tests](#unit-tests)
    - [Running all tests](#running-all-tests)
  - [Documentation](#documentation)
    - [Build docs locally](#build-docs-locally)
    - [When to update docs](#when-to-update-docs)
  - [Pull request guidelines](#pull-request-guidelines)
  - [Issue guidelines](#issue-guidelines)
  - [License](#license)

---

## Development setup

### Prerequisites

| Tool | Minimum version | Purpose |
|------|----------------|---------|
| [Rust](https://rustup.rs/) | 1.85+ (edition 2024) | Build and test |
| Git | 2.x | Version control |
| [MkDocs](https://www.mkdocs.org/) | 1.5+ | Documentation (optional) |
| [mkdocs-material](https://squidfunk.github.io/mkdocs-material/) | 9.0+ | Documentation theme (optional) |

### Clone and build

```bash
git clone https://github.com/Traitome/oxo-call.git
cd oxo-call

# Debug build
cargo build --verbose

# Release build (LTO + strip enabled)
cargo build --release
```

### Run the test suite

```bash
# All tests
cargo test --verbose

# A single integration test file
cargo test --test cli_tests

# A single test by name
cargo test --test cli_tests test_help_output -- --exact
```

### Lint and format

```bash
# Check formatting
cargo fmt -- --check

# Apply formatting
cargo fmt

# Run clippy with warnings denied (CI enforces this)
cargo clippy -- -D warnings
```

### Install locally

```bash
cargo install --path .
```

---

## Project structure

```
oxo-call/
├── Cargo.toml              # Workspace root
├── src/
│   ├── main.rs             # Entry point; license gate + command dispatch
│   ├── cli.rs              # Clap command tree (run, dry-run, docs, config, …)
│   ├── runner.rs            # Core orchestration: docs → skill → LLM → execute
│   ├── docs.rs              # Documentation resolver (cache, help, remote)
│   ├── index.rs             # Persistent documentation index
│   ├── skill.rs             # Skill system: built-in, community, user skills
│   ├── llm.rs               # Prompt building and ARGS:/EXPLANATION: parsing
│   ├── config.rs            # Platform-aware configuration
│   ├── history.rs           # JSONL command history
│   └── license.rs           # Offline Ed25519 license verification
├── skills/                  # 158 built-in skill Markdown files (.md)
├── workflows/
│   ├── native/              # .oxo.toml workflow format
│   ├── snakemake/           # Snakemake (.smk) templates
│   └── nextflow/            # Nextflow (.nf) templates
├── crates/
│   ├── license-issuer/      # Maintainer-only license signing tool
│   └── oxo-bench/           # Benchmarking crate
├── tests/
│   ├── cli_tests.rs         # Integration tests (binary execution)
│   └── fixtures/            # Test license and data
├── docs/
│   └── guide/               # MkDocs documentation source
└── .github/
    └── workflows/ci.yml     # CI: fmt, clippy, test, multi-platform build
```

### Execution flow

1. `main.rs` — Parses CLI args, enforces the license gate (all commands
   except `license`, `--help`, `--version` require a valid license).
2. `runner.rs` — For `run`/`dry-run`: fetches tool documentation **first**,
   loads any matching skill, builds the LLM prompt, optionally executes the
   tool, and records history.
3. `llm.rs` — Sends the prompt and expects a strict response containing
   `ARGS:` and `EXPLANATION:` lines (retries on format errors).

---

## Adding a new skill

Skills ground the LLM in domain-specific knowledge. Each skill is a Markdown
file (`.md`) with YAML front-matter that lives in `skills/`.

### 1. Create the Markdown file

Create `skills/<toolname>.md` following this structure:

```markdown
---
name: toolname
category: alignment          # e.g. alignment, variant-calling, qc, …
description: One-line summary of the tool
tags: [bam, alignment]       # Relevant search keywords
author: oxo-call built-in
source_url: https://tool-docs.example.com
---

## Concepts

- Key concept the LLM should know when generating arguments
- Another concept — be specific and actionable

## Pitfalls

- Common mistake users make with this tool
- Another pitfall — explain what goes wrong and the fix

## Examples

### Sort a BAM file by coordinate
**Args:** `sort -@ 4 -o sorted.bam input.bam`
**Explanation:** -@ 4 uses 4 threads; -o writes output; coordinate sort is the default

### Another representative task
**Args:** `view -b -q 30 input.bam`
**Explanation:** Filters to reads with MAPQ ≥ 30 and outputs BAM
```

**Tips:**

- `args` should contain only the arguments, **not** the tool name itself.
- Include 5+ representative examples covering common use cases.
- Concepts and pitfalls directly shape the LLM prompt — make them concise and
  accurate.
- Each example: `### Task description` → `**Args:** \`flags\`` → `**Explanation:** text`

### 2. Register the skill in `src/skill.rs`

Add an entry to the `BUILTIN_SKILLS` array using the `builtin!` macro:

```rust
const BUILTIN_SKILLS: &[(&str, &str)] = &[
    // … existing entries …
    builtin!("toolname"),       // ← add this line
];
```

Place the entry in the appropriate category section (the array is grouped by
bioinformatics domain).

### 3. Verify

```bash
cargo build --verbose
cargo test --verbose
```

Ensure the new skill appears in `oxo-call skill list` and that
`oxo-call skill show <toolname>` renders correctly.

---

## Adding workflow templates

Workflow templates live under `workflows/` in three formats:

| Directory | Format | Extension |
|-----------|--------|-----------|
| `workflows/native/` | oxo-call native | `.oxo.toml` |
| `workflows/snakemake/` | Snakemake | `.smk` |
| `workflows/nextflow/` | Nextflow | `.nf` |

To add a new template:

1. Create the workflow file in the appropriate subdirectory.
2. Include a header comment describing the analysis, required inputs, and
   expected outputs.
3. Reference only tools that have corresponding skill files in `skills/`.
4. Test locally with `oxo-call workflow show <name>` if applicable.

---

## Code style

- **Formatting**: `cargo fmt` — the CI enforces `cargo fmt -- --check`.
- **Linting**: `cargo clippy -- -D warnings` — all warnings are errors in CI.
- **Comments**: Only add comments where the logic is non-obvious. Avoid
  restating what the code already says.
- **Error handling**: Use `anyhow::Result` for application errors and
  `thiserror` for library-level typed errors.
- **Naming**: Follow standard Rust conventions (`snake_case` for functions
  and variables, `PascalCase` for types).

---

## Testing

### Integration tests

Tests in `tests/cli_tests.rs` execute the compiled binary with
`std::process::Command`. A fixture license is injected via the
`OXO_CALL_LICENSE` environment variable.

```rust
fn oxo_call() -> Command {
    let mut cmd = Command::cargo_bin("oxo-call").unwrap();
    cmd.env("OXO_CALL_LICENSE", test_license_path());
    cmd
}
```

When writing new integration tests:

- Use `oxo_call()` for commands that require a license.
- Use `oxo_call_no_license()` for testing license enforcement.
- Assert on both stdout content and exit codes.
- Run `cargo test --test cli_tests` to validate.

### Unit tests

Add `#[cfg(test)]` modules inside the relevant source file. Keep unit tests
focused on a single function or struct.

### Running all tests

```bash
cargo test --verbose
```

CI runs the full test matrix on every PR across Linux, macOS, and Windows.

---

## Documentation

The user-facing guide is built with [MkDocs](https://www.mkdocs.org/) and the [Material theme](https://squidfunk.github.io/mkdocs-material/)
from source files in `docs/guide/src/`.

### Build docs locally

```bash
# Install dependencies (first time only)
pip install mkdocs-material

# Serve or build
cd docs/guide
mkdocs serve          # Live-reload at http://localhost:8000
mkdocs build          # Static output in docs/guide/site/
```

### When to update docs

- Adding a new subcommand → update the relevant guide page.
- Changing config options → update the configuration reference.
- Adding a skill or workflow → mention it in the tools/skills section.

CI automatically deploys the guide to GitHub Pages on pushes to `main`.

---

## Pull request guidelines

1. **Branch from `main`** — use descriptive branch names like
   `feat/add-blast-skill` or `fix/config-path-windows`.

2. **Keep PRs focused** — one feature or fix per PR. Large PRs are harder to
   review.

3. **Pass CI** — ensure all of the following pass before requesting review:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test --verbose
   ```

4. **Write a clear description** — explain *what* changed and *why*. Link
   related issues with `Closes #123`.

5. **Add tests** — new features need integration tests; bug fixes need
   regression tests.

6. **Update documentation** — if user-facing behavior changes, update the
   relevant documentation pages and/or `--help` text.

7. **License gate awareness** — if you change command flow in `main.rs`,
   preserve the rule that core commands require a valid license while
   `license`, `--help`, and `--version` work without one.

8. **Keep `license.rs` and `license-issuer` in sync** — the issuer signs
   the same payload shape that the runtime verifies; schema changes must be
   coordinated across both crates.

---

## Issue guidelines

When opening an issue:

- **Bug reports** — include the oxo-call version (`oxo-call --version`),
  OS, the exact command you ran, and the full error output.
- **Feature requests** — describe the use case, not just the desired
  solution.
- **Skill requests** — mention the tool name, homepage, and a few example
  tasks you'd like supported.

---

## License

oxo-call uses a **dual-license model**:

| Use case | License | Cost |
|----------|---------|------|
| Academic, educational, personal research | [Academic License](LICENSE-ACADEMIC) | Free |
| Commercial / for-profit | [Commercial License](LICENSE-COMMERCIAL) | Paid |

By contributing to this repository, you agree that your contributions will
be licensed under the same terms. See [LICENSE](LICENSE) for the full
details.

If you have licensing questions, contact **w_shixiang@163.com**.
