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
- Install the CLI locally from the workspace root: `cargo install --path .`

Maintainer-only license tooling lives in the workspace too:

- Generate a signing keypair: `cargo run --bin license-issuer -- generate-keypair`
- Issue a license file: `cargo run --bin license-issuer -- issue --org "Example Org" --type academic --output license.oxo.json`

## High-level architecture

This repository is a Rust workspace with two crates:

- The root crate is the end-user CLI, `oxo-call`.
- `crates/license-issuer` is a maintainer-only offline signing tool for generating `license.oxo.json` files.

Within the main CLI crate, the execution flow matters more than the file list:

- `src/main.rs` is the command dispatcher and enforces the license gate before almost every command. Only the `license` subcommands are exempt once execution reaches `run()`. `--help` and `--version` are handled earlier by Clap.
- `src/cli.rs` defines the user-facing command tree: `run`, `dry-run`, `index`, `docs`, `config`, `history`, `skill`, and `license`.
- `src/runner.rs` is the core orchestration path for `run` and `dry-run`: fetch documentation, load any matching skill, ask the LLM for arguments, optionally execute the tool, then append command history for real runs.
- `src/docs.rs` is the documentation resolver. It combines cached docs, live help output from the installed tool, optional local doc directories from config, and optional remote fetches.
- `src/index.rs` manages the persistent documentation index and cached combined docs used for repeated tool lookups.
- `src/skill.rs` implements the skill system. Built-in skills are compiled from `skills/*.toml`, while user and community skill files override built-ins at runtime.
- `src/llm.rs` builds the prompt and parses the model response back into argv-style arguments.
- `src/config.rs` and `src/history.rs` use platform-specific directories from `directories::ProjectDirs` for config, cached docs, skills, and JSONL history.
- `src/license.rs` performs offline Ed25519 license verification for the runtime CLI.

The maintainer crate is tightly coupled to runtime licensing:

- `crates/license-issuer/src/main.rs` defines the signing CLI and mirrors the runtime license payload schema from `src/license.rs`.

## Key conventions

- License enforcement is a core product behavior, not an optional wrapper. If you change command flow in `src/main.rs`, preserve the rule that core commands require a valid license while `license`, `--help`, and `--version` still work without one.

- Keep `src/license.rs` and `crates/license-issuer/src/main.rs` in sync. The issuer signs the same payload shape that the runtime verifies, so schema or field changes usually require coordinated edits in both places.

- The main command-generation path is docs-first, not prompt-only. `Runner::prepare()` fetches tool documentation before loading a skill and calling the LLM. When changing generation behavior, preserve that ordering unless you intentionally want to weaken grounding.

- Skills are part of the prompt contract. Built-in skills are embedded with `include_str!`, and runtime precedence is `user > community > built-in`. Reuse `SkillManager` instead of adding ad hoc skill loading.

- The LLM response format is intentionally strict. `src/llm.rs` expects lines starting with `ARGS:` and `EXPLANATION:` and retries when the format is invalid. If you change prompting or parsing, keep that contract aligned with `Runner`.

- Reuse the existing validation and sanitization in `src/docs.rs` when touching tool names, file-backed docs, or remote documentation URLs. The current code explicitly rejects traversal-like tool names and non-HTTP(S) remote URLs.

- Integration tests in `tests/cli_tests.rs` execute the compiled binary with `Command`, not mocked internals. They inject the fixture license through `OXO_CALL_LICENSE` and exercise real commands like `index`, `docs`, `skill`, and `license`.

- The built-in skill files in `skills/*.toml` are not just reference data; they materially shape the LLM prompt via concepts, pitfalls, and worked examples. If you add or edit supported tools, update both the TOML skill file and the built-in registry in `src/skill.rs`.
