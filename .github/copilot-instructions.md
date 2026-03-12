# Copilot Instructions for `oxo-call`

## Build, test, and lint

Run all commands from the repository root.

- Build the workspace: `cargo build --verbose`
- Build a release binary: `cargo build --release`
- Run the full test suite: `cargo test --verbose`
- Run one integration test file: `cargo test --test cli_tests`
- Run one integration test by name: `cargo test --test cli_tests test_help_output -- --exact`
- Run one unit test by name: `cargo test test_valid_academic_license_passes -- --exact`
- Check formatting: `cargo fmt -- --check`
- Run clippy with warnings denied: `cargo clippy -- -D warnings`
- Security audit: `cargo audit` (install with `cargo install cargo-audit`)
- Install the CLI locally from the workspace root: `cargo install --path .`

Maintainer-only license tooling lives in the workspace too:

- Generate a signing keypair: `cargo run -p license-issuer --bin license-issuer -- generate-keypair`
- Issue a license file: `cargo run -p license-issuer --bin license-issuer -- issue --org "Example Org" --type academic --output license.oxo.json`

## Documentation

The project uses [mdBook](https://rust-lang.github.io/mdBook/) for its documentation website. Source files live under `docs/guide/src/`.

- Build the documentation: `cd docs/guide && mdbook build`
- Serve locally with live reload: `cd docs/guide && mdbook serve`
- The built site appears at `docs/guide/book/` and is deployed to GitHub Pages under `/documentation/`
- The landing page at `docs/index.html` links to the documentation site

### Documentation structure

```
docs/guide/src/
├── SUMMARY.md                         # Table of contents (mdBook navigation)
├── introduction.md                    # Project overview
├── tutorials/                         # Getting started guides
│   ├── installation.md                # Install from crates.io / releases / git clone
│   ├── quickstart.md                  # First-use walkthrough
│   ├── configuration.md               # LLM provider setup, config keys
│   └── license.md                     # License types and setup
├── commands/                          # Per-command reference
│   ├── run.md, dry-run.md, docs.md, config.md,
│   │   history.md, skill.md, workflow.md, license.md
├── reference/                         # Architecture & design
│   ├── architecture.md                # Module graph, execution flow, design principles
│   ├── documentation-system.md        # Docs resolver, caching, validation
│   ├── skill-system.md                # Skill format, loading, coverage table
│   ├── llm-integration.md             # Providers, prompt rules, response format
│   ├── workflow-engine.md             # DAG engine, wildcard expansion, export
│   └── license-system.md              # Ed25519 verification, payload schema
└── development/                       # Contributor resources
    ├── contributing.md                # Dev setup, PR guidelines, adding skills
    └── evaluation-reports.md          # 12-role expert evaluation + action items
```

When you change CLI behavior, update the corresponding `docs/guide/src/commands/*.md` file. When you change architecture, update `docs/guide/src/reference/*.md`.

## High-level architecture

This repository is a Rust workspace with three crates:

| Crate | Purpose | Published |
|-------|---------|-----------|
| `oxo-call` (root) | End-user CLI | Yes (crates.io) |
| `crates/license-issuer` | Maintainer-only offline signing tool | No |
| `crates/oxo-bench` | Benchmarking and evaluation suite | No |

Within the main CLI crate, the execution flow matters more than the file list:

- `src/main.rs` is the command dispatcher and enforces the license gate before almost every command. Only the `license` subcommands are exempt once execution reaches `run()`. `--help` and `--version` are handled earlier by Clap.
- `src/cli.rs` defines the user-facing command tree: `run`, `dry-run`, `index`, `docs`, `config`, `history`, `skill`, `workflow`, and `license`.
- `src/runner.rs` is the core orchestration path for `run` and `dry-run`: fetch documentation, load any matching skill, ask the LLM for arguments, optionally execute the tool, then append command history for real runs.
- `src/docs.rs` is the documentation resolver. It combines cached docs, live help output from the installed tool, optional local doc directories from config, and optional remote fetches.
- `src/index.rs` manages the persistent documentation index and cached combined docs used for repeated tool lookups.
- `src/skill.rs` implements the skill system. Built-in skills are compiled from `skills/*.toml` (150+ bioinformatics tools), while user and community skill files override built-ins at runtime.
- `src/llm.rs` builds the prompt and parses the model response back into argv-style arguments. Supports GitHub Copilot, OpenAI, Anthropic, and Ollama providers.
- `src/config.rs` and `src/history.rs` use platform-specific directories from `directories::ProjectDirs` for config, cached docs, skills, and JSONL history.
- `src/license.rs` performs offline Ed25519 license verification for the runtime CLI.
- `src/workflow.rs` and `src/engine.rs` implement the native DAG-based workflow engine with Snakemake/Nextflow export. Built-in templates live in `workflows/`.

The maintainer crate is tightly coupled to runtime licensing:

- `crates/license-issuer/src/main.rs` defines the signing CLI and mirrors the runtime license payload schema from `src/license.rs`.

## Module dependencies

```
main.rs (command dispatcher + license gate)
  ├─→ cli.rs (Clap command definitions)
  ├─→ license.rs (Ed25519 verification)
  ├─→ runner.rs (core orchestration)
  │     ├─→ docs.rs (documentation resolver)
  │     ├─→ skill.rs (skill loading)
  │     ├─→ llm.rs (LLM client + prompt builder)
  │     └─→ history.rs (JSONL history)
  ├─→ workflow.rs (templates + registry)
  │     └─→ engine.rs (DAG execution)
  ├─→ config.rs (configuration management)
  ├─→ index.rs (documentation index)
  └─→ error.rs (error types)
```

## Key conventions

- License enforcement is a core product behavior, not an optional wrapper. If you change command flow in `src/main.rs`, preserve the rule that core commands require a valid license while `license`, `--help`, and `--version` still work without one.

- Keep `src/license.rs` and `crates/license-issuer/src/main.rs` in sync. The issuer signs the same payload shape that the runtime verifies, so schema or field changes usually require coordinated edits in both places.

- The main command-generation path is docs-first, not prompt-only. `Runner::prepare()` fetches tool documentation before loading a skill and calling the LLM. When changing generation behavior, preserve that ordering unless you intentionally want to weaken grounding.

- Skills are part of the prompt contract. Built-in skills are embedded with `include_str!`, and runtime precedence is `user > community > built-in`. Reuse `SkillManager` instead of adding ad hoc skill loading.

- The LLM response format is intentionally strict. `src/llm.rs` expects lines starting with `ARGS:` and `EXPLANATION:` and retries when the format is invalid. If you change prompting or parsing, keep that contract aligned with `Runner`.

- Reuse the existing validation and sanitization in `src/docs.rs` when touching tool names, file-backed docs, or remote documentation URLs. The current code explicitly rejects traversal-like tool names and non-HTTP(S) remote URLs.

- Integration tests in `tests/cli_tests.rs` execute the compiled binary with `Command`, not mocked internals. They inject the fixture license through `OXO_CALL_LICENSE` and exercise real commands like `index`, `docs`, `skill`, and `license`.

- The built-in skill files in `skills/*.toml` are not just reference data; they materially shape the LLM prompt via concepts, pitfalls, and worked examples. If you add or edit supported tools, update both the TOML skill file and the built-in registry in `src/skill.rs`.

- Skill TOML files require `= "..."` assignment syntax for all `explanation`, `task`, and `args` fields in `[[examples]]` blocks. Missing `=` causes silent TOML parse failure.

## CI/CD pipeline

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs:

1. **Quality gate**: `cargo build` → `cargo test` → `cargo fmt --check` → `cargo clippy -D warnings` → `cargo audit`
2. **Cross-platform builds** (on version tags only): Linux (x86_64/aarch64, glibc/musl), macOS (Intel/Apple Silicon), Windows (x86_64/aarch64), WebAssembly (wasm32-wasip1)
3. **GitHub Release**: Attach all binaries with SHA256 checksums (`SHA256SUMS.txt`)
4. **crates.io publish**: Automated version verification and `cargo publish`
5. **GitHub Pages**: Build mdBook documentation and deploy landing page + docs site

## Adding or editing commands

1. Define the command in `src/cli.rs` (Clap derive)
2. Add the handler in `src/main.rs` (match arm in `run()`)
3. Add integration tests in `tests/cli_tests.rs`
4. Document the command in `docs/guide/src/commands/<command>.md`
5. Update `docs/guide/src/SUMMARY.md` if adding a new command

## Adding a new built-in skill

1. Create the TOML file at `skills/<tool>.toml` following the `[meta]` + `[context]` + `[[examples]]` structure
2. Register it in `src/skill.rs` in the `BUILTIN_SKILLS` array with `("<tool>", include_str!("../skills/<tool>.toml"))`
3. Use the exact binary name (not aliases) as the tool name
4. Include at minimum: 3 concepts, 3 pitfalls, 5 worked examples
5. Run `cargo test` to verify the skill loads correctly

## Adding a workflow template

1. Create the native template at `workflows/native/<template>.oxo.toml`
2. Optionally create export targets at `workflows/snakemake/<template>.smk` and `workflows/nextflow/<template>.nf`
3. Register the template in `src/workflow.rs` in the `BUILTIN_TEMPLATES` array
4. Add integration tests in `tests/cli_tests.rs`

## Project files

| File | Purpose |
|------|---------|
| `CITATION.cff` | Academic citation metadata (CFF 1.2.0 format) |
| `CONTRIBUTING.md` | Contributor guide with development setup and PR guidelines |
| `LICENSE` | Primary license file |
| `LICENSE-ACADEMIC` | Academic (free) license terms |
| `LICENSE-COMMERCIAL` | Commercial license terms |
| `README.md` | Project overview with architecture diagram and command reference |
| `docs/index.html` | Landing page deployed to GitHub Pages |
| `docs/guide/` | mdBook documentation source and configuration |
