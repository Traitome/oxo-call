---
name: cargo
category: package-management
description: Rust package manager and build tool; compiles, tests, and publishes Rust crates; manages dependencies in Cargo.toml
tags: [rust, cargo, crate, build, package, rustup, toml, dependency]
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

## Pitfalls
- DANGER: `cargo clean` deletes the entire `target/` directory — all build artefacts; rebuilding from scratch can take minutes for large projects.
- `cargo install` builds and installs a binary to `~/.cargo/bin/`; it does NOT update existing installs — run `cargo install --force <crate>` to upgrade.
- Adding many dependencies without features filtering bloats binary size and compile times; use `default-features = false` and enable only needed features.
- The `Cargo.lock` file should be committed for binaries (reproducible builds) but often excluded from libraries (flexible consumer deps).
- `cargo test` runs tests in parallel by default; tests sharing global state (files, ports) must use `--test-threads 1` or a mutex.
- `RUSTFLAGS=-C target-cpu=native cargo build --release` produces CPU-specific binaries that will not run on older CPUs.
- On HPC systems, `cargo build` may fail if the network is restricted; vendor dependencies first with `cargo vendor` or configure a private registry mirror.
- `cargo update` bumps dependencies to the latest compatible SemVer versions within constraints in `Cargo.toml`; verify with `cargo test` after updating.

## Examples

### build a project in debug mode
**Args:** `build`
**Explanation:** compiles the crate and dependencies; output goes to target/debug/; fast incremental builds

### build an optimised release binary
**Args:** `build --release`
**Explanation:** enables full optimisations; output in target/release/; use for production and benchmarks

### run the project binary directly
**Args:** `run`
**Explanation:** compiles (if needed) and runs the default binary; equivalent to cargo build && ./target/debug/<name>

### run all tests
**Args:** `test`
**Explanation:** compiles and runs all unit, integration, and doc tests; parallel execution by default

### run tests with a specific filter
**Args:** `test -- --test-threads 1 my_test_name`
**Explanation:** -- passes args to the test binary; --test-threads 1 forces serial execution; filters by test name substring

### add a dependency to Cargo.toml
**Args:** `add serde --features derive`
**Explanation:** inserts the latest compatible serde version into Cargo.toml with the 'derive' feature enabled; no manual TOML editing needed

### install a binary crate
**Args:** `install ripgrep`
**Explanation:** builds and installs the rg binary to ~/.cargo/bin/; the bin directory must be in PATH

### show the dependency tree
**Args:** `tree`
**Explanation:** prints the full crate dependency graph including transitive deps; useful for auditing and de-duplicating

### check code for errors without producing a binary
**Args:** `check`
**Explanation:** much faster than build; only type-checks and borrow-checks; ideal for tight edit-check loops

### run clippy lints on the project
**Args:** `clippy -- -D warnings`
**Explanation:** -D warnings treats all lint warnings as errors; use in CI to enforce lint cleanliness

### format source code
**Args:** `fmt`
**Explanation:** formats all .rs files in the workspace using rustfmt rules defined in rustfmt.toml or Cargo.toml

### show cargo and rustc version information
**Args:** `--version`
**Explanation:** prints the cargo version; pair with `rustc --version` to capture full toolchain version for reproducibility logs

### clean build artefacts
**Args:** `clean`
**Explanation:** removes the target/ directory; use before a full rebuild or to free disk space; project must be recompiled from scratch afterward

### list installed binary crates
**Args:** `install --list`
**Explanation:** shows all crates installed to ~/.cargo/bin/ with their version and source; useful for auditing global Rust tools
