---
name: cargo
category: package-management
description: Rust package manager and build tool; compiles, tests, and publishes Rust crates; manages dependencies in Cargo.toml
tags: [rust, cargo, crate, build, package, rustup, toml, dependency, workspace, features, cross-compilation, registry, crates.io, vendor, publish]
author: oxo-call built-in
source_url: "https://doc.rust-lang.org/cargo/"
---

## Concepts
- The Rust toolchain is installed and managed by `rustup`; default installation is in `~/.rustup/` (toolchains) and `~/.cargo/` (binaries, registry, sources).
- `~/.cargo/bin/` is where `cargo`, `rustc`, and installed binaries live; added to PATH by `source ~/.cargo/env` (or `~/.profile` after rustup install).
- The cargo home directory defaults to `~/.cargo/`; override with `CARGO_HOME` environment variable.
- Package registry cache (crates.io index and downloaded crates) lives in `~/.cargo/registry/` — can grow large; safe to delete to free disk.
- Git dependencies are cached in `~/.cargo/git/`.
- `Cargo.toml` at the project root declares dependencies, features, edition, and metadata; `Cargo.lock` pins exact resolved versions.
- Build artefacts are written to `target/` by default; override with `--target-dir` or `CARGO_TARGET_DIR` env var.
- Cargo workspaces (`[workspace]` in root `Cargo.toml`) allow multiple crates to share a single `target/` directory and `Cargo.lock`.
- `cargo build --release` enables optimisations (`opt-level=3`); default debug builds have assertions enabled and no optimisation.
- Cross-compilation requires installing the target with `rustup target add <triple>` and specifying `--target <triple>` on build.
- `rustup show` prints the active toolchain, installed targets, and the location of the rust sysroot.
- Feature flags allow conditional compilation; `--features` enables specific features, `--all-features` enables all, `--no-default-features` disables defaults.
- Cargo profiles (dev, release, test, bench) control compiler settings; custom profiles can be defined in Cargo.toml.
- `cargo vendor` downloads all dependencies to a local `vendor/` directory for offline builds and reproducible builds in restricted environments.

## Pitfalls
- `cargo clean` deletes the entire `target/` directory — all build artefacts; rebuilding from scratch can take minutes for large projects.
- `cargo install` builds and installs a binary to `~/.cargo/bin/`; it does NOT update existing installs — run `cargo install --force <crate>` to upgrade.
- Adding many dependencies without features filtering bloats binary size and compile times; use `default-features = false` and enable only needed features.
- The `Cargo.lock` file should be committed for binaries (reproducible builds) but often excluded from libraries (flexible consumer deps).
- `cargo test` runs tests in parallel by default; tests sharing global state (files, ports) must use `--test-threads 1` or a mutex.
- `RUSTFLAGS=-C target-cpu=native cargo build --release` produces CPU-specific binaries that will not run on older CPUs.
- On HPC systems, `cargo build` may fail if the network is restricted; vendor dependencies first with `cargo vendor` or configure a private registry mirror.
- `cargo update` bumps dependencies to the latest compatible SemVer versions within constraints in `Cargo.toml`; verify with `cargo test` after updating.
- `--locked`, `--offline`, and `--frozen` have distinct meanings: `--locked` fails if Cargo.lock needs changes, `--offline` prevents network access, `--frozen` combines both.
- `cargo publish` uploads to crates.io permanently; versions cannot be deleted, only yanked (hidden from new users but still available to existing users).
- `cargo fix` can automatically apply lint suggestions but may break code; always review changes and run tests after applying fixes.

## Examples

### build a project in debug mode
**Args:** `build`
**Explanation:** build subcommand; compiles the crate and dependencies; output goes to target/debug/; fast incremental builds

### build an optimised release binary
**Args:** `build --release`
**Explanation:** build subcommand; --release enables full optimisations; output in target/release/; use for production and benchmarks

### run the project binary directly
**Args:** `run`
**Explanation:** run subcommand; compiles (if needed) and runs the default binary; equivalent to cargo build && ./target/debug/<name>

### run all tests
**Args:** `test`
**Explanation:** test subcommand; compiles and runs all unit, integration, and doc tests; parallel execution by default

### run tests with a specific filter
**Args:** `test -- --test-threads 1 my_test_name`
**Explanation:** test subcommand; -- passes args to the test binary; --test-threads 1 forces serial execution; my_test_name filters by test name substring

### add a dependency to Cargo.toml
**Args:** `add serde --features derive`
**Explanation:** add subcommand; serde dependency name; --features derive enables the 'derive' feature; inserts the latest compatible serde version into Cargo.toml; no manual TOML editing needed

### install a binary crate
**Args:** `install ripgrep`
**Explanation:** install subcommand; ripgrep crate name; builds and installs the rg binary to ~/.cargo/bin/; the bin directory must be in PATH

### show the dependency tree
**Args:** `tree`
**Explanation:** tree subcommand; prints the full crate dependency graph including transitive deps; useful for auditing and de-duplicating

### check code for errors without producing a binary
**Args:** `check`
**Explanation:** check subcommand; much faster than build; only type-checks and borrow-checks; ideal for tight edit-check loops

### run clippy lints on the project
**Args:** `clippy -- -D warnings`
**Explanation:** clippy subcommand; -- passes args to clippy; -D warnings treats all lint warnings as errors; use in CI to enforce lint cleanliness

### format source code
**Args:** `fmt`
**Explanation:** fmt subcommand; formats all .rs files in the workspace using rustfmt rules defined in rustfmt.toml or Cargo.toml

### show cargo and rustc version information
**Args:** `--version`
**Explanation:** --version flag; prints the cargo version; pair with `rustc --version` to capture full toolchain version for reproducibility logs

### clean build artefacts
**Args:** `clean`
**Explanation:** clean subcommand; removes the target/ directory; use before a full rebuild or to free disk space; project must be recompiled from scratch afterward

### list installed binary crates
**Args:** `install --list`
**Explanation:** install subcommand; --list flag; shows all crates installed to ~/.cargo/bin/ with their version and source; useful for auditing global Rust tools

### vendor dependencies for offline builds
**Args:** `vendor`
**Explanation:** vendor subcommand; downloads all dependencies to a local vendor/ directory; use on HPC or air-gapped systems where network access is restricted

### build with specific features enabled
**Args:** `build --features "serde derive" --release`
**Explanation:** build subcommand; --features "serde derive" enables specified features; --release enables optimisations; multiple features can be space or comma separated; use --all-features to enable everything

### run tests without running them (compile only)
**Args:** `test --no-run`
**Explanation:** test subcommand; --no-run compiles tests but does not execute them; useful for CI to verify tests compile without the time cost of running them

### update dependencies to latest compatible versions
**Args:** `update`
**Explanation:** update subcommand; updates Cargo.lock to use the latest SemVer-compatible versions of all dependencies; run cargo test afterward to verify compatibility

### show package information from registry
**Args:** `info serde`
**Explanation:** info subcommand; serde crate name; displays metadata about a crate from the registry including version, dependencies, and features; useful for exploring crates before adding them

### create a new library crate
**Args:** `new --lib my_library`
**Explanation:** new subcommand; --lib flag; my_library crate name; creates a new library crate (instead of binary) with src/lib.rs as the entry point; use for reusable code packages

### package for publishing (dry run)
**Args:** `publish --dry-run`
**Explanation:** publish subcommand; --dry-run performs all verification steps without uploading; use this to catch packaging errors before the real publish

### build specific package in workspace
**Args:** `build -p package_name --release`
**Explanation:** build subcommand; -p package_name builds only specified package; --release enables optimisations; useful for large workspaces where you only need to compile one crate

### check code with all features enabled
**Args:** `check --all-features`
**Explanation:** check subcommand; --all-features type-checks the code with all feature flags enabled; catches compilation errors that might only appear with certain feature combinations
