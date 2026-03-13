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
