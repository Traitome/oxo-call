# Contributing

## Development Setup

```bash
# Clone the repository
git clone https://github.com/Traitome/oxo-call.git
cd oxo-call

# Build the workspace
cargo build --verbose

# Run tests
cargo test --verbose

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings
```

## Project Structure

```
oxo-call/
├── src/              # Main CLI source code
├── skills/           # 150+ built-in skill TOML files
├── tests/            # Integration tests
├── crates/
│   ├── license-issuer/   # Maintainer license signing tool
│   └── oxo-bench/        # Benchmarking suite
├── workflows/        # Built-in workflow templates
│   ├── native/       # .oxo.toml templates
│   ├── snakemake/    # Exported .smk files
│   └── nextflow/     # Exported .nf files
└── docs/             # Documentation and website
```

## Running Tests

```bash
# Full test suite
cargo test --verbose

# Single integration test file
cargo test --test cli_tests

# Single test by name
cargo test --test cli_tests test_help_output -- --exact

# Single unit test
cargo test test_valid_academic_license_passes -- --exact
```

## Adding a New Skill

1. Create a TOML file in `skills/<tool>.toml`
2. Register it in `src/skill.rs` in the `BUILTIN_SKILLS` array
3. Follow the skill format (see [Skill System](../reference/skill-system.md))
4. Run tests to verify

## Code Style

- Follow Rust idioms and the existing codebase style
- Run `cargo fmt` before committing
- All clippy warnings must be resolved (`cargo clippy -- -D warnings`)
- Use `anyhow` for error handling in application code
- Use `thiserror` for library-style error types

## Documentation

Documentation is built with [mdBook](https://rust-lang.github.io/mdBook/):

```bash
cd docs/guide
mdbook build    # Build the documentation
mdbook serve    # Serve locally at http://localhost:3000
```

## Benchmarking with oxo-bench

The `oxo-bench` crate provides automated evaluation for testing skill quality and LLM accuracy:

```bash
# Run the full benchmark suite (50+ tasks across 15 categories)
cargo run -p oxo-bench -- evaluate

# Benchmark a specific tool
cargo run -p oxo-bench -- evaluate --tool samtools

# Run ablation tests (docs-only vs. docs+skills vs. full pipeline)
cargo run -p oxo-bench -- evaluate --ablation

# Export benchmark data as CSV for analysis
cargo run -p oxo-bench -- export-csv --output docs/
```

Benchmark categories include: alignment, variant-calling, SAM/BAM, quantification, QC, metagenomics, epigenomics, single-cell, assembly, annotation, and more.

When contributing a new built-in skill, run the benchmark for that tool to verify accuracy improvements:

```bash
# Before: evaluate without the skill
cargo run -p oxo-bench -- evaluate --tool mytool

# After: evaluate with the skill added
cargo run -p oxo-bench -- evaluate --tool mytool
```

Results are exported to `docs/bench_eval_tasks.csv`, `docs/bench_scenarios.csv`, and `docs/bench_workflow.csv`.

## Changelog

This project uses [git-cliff](https://git-cliff.org) to generate `CHANGELOG.md` from
[Conventional Commit](https://www.conventionalcommits.org/) messages.

### Commit message format

Use Conventional Commits in your PR titles and squash-merge messages:

```
<type>(<optional scope>): <description>

feat(skill): add kallisto built-in skill with 7 examples
fix(engine): correct MultiQC dependency to fastp-only
docs: add air-gapped mode guide to LLM provider how-to
ci: add git-cliff changelog generation to release workflow
chore: bump clap to 4.5
```

Supported types: `feat`, `fix`, `perf`, `refactor`, `docs`, `style`, `ci`, `chore`, `test`, `build`.

### Preview the changelog locally

```bash
# Install git-cliff
cargo install git-cliff

# Preview unreleased changes since the last tag
git cliff --unreleased

# Regenerate the full CHANGELOG.md
git cliff --output CHANGELOG.md

# Preview what the next version entry will look like (auto-bumps version)
git cliff --unreleased --bump
```

### Release process

The CI pipeline automatically generates the GitHub Release body from git-cliff when a version tag is pushed:

```bash
# 1. Update version in Cargo.toml
# 2. Commit and tag
git tag -a v0.4.0 -m "v0.4.0"
git push origin v0.4.0
# CI generates the release notes automatically from cliff.toml
```

