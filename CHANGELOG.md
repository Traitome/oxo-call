# Changelog

All notable changes to oxo-call are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

From v0.4.0 onward, entries are generated automatically by
[git-cliff](https://git-cliff.org) during the release process
(`cargo install git-cliff`).

<!-- next-header -->

## [Unreleased]

### Features

- **HPC cluster management skills:** 6 new built-in skills for cluster job schedulers — `slurm`, `pbs` (PBS Pro/Torque), `sge` (Sun Grid Engine), `lsf` (IBM Spectrum LSF), `htcondor`, and `kubectl` (Kubernetes). Each skill includes common submission templates, resource query commands, array job patterns, and bioinformatics-specific examples for quick job script generation without memorizing scheduler syntax.
- **`server` subcommand:** New top-level command for managing SSH-connected remote servers. Register workstations and HPC cluster login nodes, check connectivity, and generate commands for remote execution:
  - `server add` — register a server with SSH credentials, server type (workstation/hpc), and optional scheduler
  - `server remove` / `server list` / `server status` — manage and monitor registered servers
  - `server ssh-config` — discover and import hosts from `~/.ssh/config`
  - `server run` / `server dry-run` — generate LLM-powered commands targeting a remote server
  - **HPC login node safety:** Automatically detects compute-intensive commands and warns against direct execution on login nodes, suggesting scheduler submission (sbatch, qsub, etc.) instead
  - **Scheduler auto-detection:** For HPC servers, attempts to detect the installed scheduler (Slurm, PBS, SGE, LSF, HTCondor) automatically

## [0.3.0] — 2026-03-13

### Features

- **shell completion:** Add `completion` subcommand for bash/zsh/fish/elvish/powershell shell completion scripts ([#24](https://github.com/Traitome/oxo-call/pull/24))
- **`--verbose` flag:** Global verbose flag prints the full LLM prompt, grounding sources, and raw model response for debugging ([#24](https://github.com/Traitome/oxo-call/pull/24))
- **`--model` flag:** Per-invocation model override on `run` and `dry-run` without changing the stored config ([#24](https://github.com/Traitome/oxo-call/pull/24))
- **`--no-cache` flag:** Skip cached documentation and force a fresh `--help` capture for `run` and `dry-run` ([#24](https://github.com/Traitome/oxo-call/pull/24))
- **`--json` flag:** Output the generated command as a JSON object `{"command":…,"explanation":…}` for programmatic use ([#24](https://github.com/Traitome/oxo-call/pull/24))
- **Per-step `env` field in workflows:** Each `[[step]]` in `.oxo.toml` now accepts an `env` string prepended as a shell preamble — enables conda environment activation, virtualenv sourcing, and module system commands per step ([#23](https://github.com/Traitome/oxo-call/pull/23))
- **Workflow DAG phase visualization:** New `workflow vis` / `workflow dag` subcommand renders a text-based phase diagram of the pipeline ([#19](https://github.com/Traitome/oxo-call/pull/19))
- **Workflow progress display:** Execution now shows `[N/M]` step counters and elapsed time for each step ([#19](https://github.com/Traitome/oxo-call/pull/19))
- **`workflow verify` / `workflow fmt` subcommands:** Validate TOML + DAG integrity and auto-format workflow files ([#19](https://github.com/Traitome/oxo-call/pull/19))
- **Command provenance tracking:** Every history entry now records `tool_version`, `docs_hash` (SHA-256 of combined documentation), `skill_name`, and `model` for full reproducibility audits ([#16](https://github.com/Traitome/oxo-call/pull/16))
- **`lib.rs` programmatic API:** All 13 modules re-exported via `lib.rs` for embedding oxo-call as a library ([#16](https://github.com/Traitome/oxo-call/pull/16))
- **`LlmProvider` trait:** Plugin-style trait in `src/llm.rs` allows adding new LLM providers without modifying core logic ([#16](https://github.com/Traitome/oxo-call/pull/16))
- **Data anonymization:** `src/sanitize.rs` with `redact_paths()` and `redact_env_tokens()` for stripping sensitive data before LLM submission ([#16](https://github.com/Traitome/oxo-call/pull/16))
- **`handlers.rs`:** Extracted formatting helpers from `main.rs` to reduce complexity ([#16](https://github.com/Traitome/oxo-call/pull/16))
- **`CODE_OF_CONDUCT.md`:** Contributor Covenant v2.1 adopted ([#22](https://github.com/Traitome/oxo-call/pull/22))

### Bug Fixes

- **MultiQC positioning:** Corrected MultiQC dependency to `["fastp"]` only — it now runs in parallel with alignment (STAR/BWA-MEM2/Bowtie2), not at the end of the pipeline. Fixes incorrect pipeline DAG in all 9 built-in workflow templates ([#19](https://github.com/Traitome/oxo-call/pull/19))
- **WASM build:** Gate workflow execution commands behind `cfg(not(target_arch = "wasm32"))` to fix `wasm32-wasip1` compilation ([#17](https://github.com/Traitome/oxo-call/pull/17))

### Documentation

- **Complete mdBook tutorial rewrite:** Restructured documentation into Getting Started, Tutorials, How-to Guides, Command Reference, and Architecture & Design sections ([#21](https://github.com/Traitome/oxo-call/pull/21))
- **Expert evaluation reports:** 12-role evaluation with resolution status tracking for all action items ([#22](https://github.com/Traitome/oxo-call/pull/22))
- **Container image references in workflow templates:** BioContainers Docker/Singularity URIs added to all Snakemake (`.smk`) and Nextflow (`.nf`) export files

### CI/CD

- **Build all platforms on every push:** Cross-platform CI (Linux x86_64/aarch64, macOS Intel/Apple Silicon, Windows, WASM) now runs on every push, not only on tags ([#20](https://github.com/Traitome/oxo-call/pull/20))

---

## [0.2.0] — 2026-03-12

### Features

- **`workflow` subcommand:** Native Rust DAG-based workflow engine with `.oxo.toml` format, `tokio` parallelism, wildcard expansion, and output-freshness caching ([#11](https://github.com/Traitome/oxo-call/pull/11))
- **Snakemake and Nextflow export:** `workflow export --snakemake` and `workflow export --nextflow` generate HPC-ready pipeline files from `.oxo.toml` ([#11](https://github.com/Traitome/oxo-call/pull/11))
- **9 built-in workflow templates:** RNA-seq, WGS, ATAC-seq, ChIP-seq, metagenomics, methylation-seq, scRNA-seq, amplicon 16S, and long-reads ([#11](https://github.com/Traitome/oxo-call/pull/11))
- **Multilingual task input:** The LLM now accepts task descriptions in any language (English, Chinese, Japanese, Korean, etc.) ([#12](https://github.com/Traitome/oxo-call/pull/12))
- **Unified `docs` command:** `docs add`, `docs remove`, `docs update`, `docs list`, `docs show`, `docs path`, `docs fetch` as the primary documentation management interface; `index` retained for backward compatibility ([#12](https://github.com/Traitome/oxo-call/pull/12))
- **Auto-indexing on first use:** Tool documentation is automatically captured from `--help` on the first invocation with no manual indexing required ([#12](https://github.com/Traitome/oxo-call/pull/12))
- **Multi-format local docs:** `docs add --file` supports `.txt`, `.md`, and `.rst` local documentation files; `docs add --dir` indexes a directory recursively ([#12](https://github.com/Traitome/oxo-call/pull/12))
- **Remote documentation fetch:** `docs add --url` fetches documentation from HTTP/HTTPS URLs ([#12](https://github.com/Traitome/oxo-call/pull/12))
- **mdBook documentation site:** Comprehensive documentation guide published to GitHub Pages via CI ([#14](https://github.com/Traitome/oxo-call/pull/14))
- **Expert evaluation reports:** 12-role evaluation report (Computational Biologist, Biostatistician, Reproducibility Scientist, Clinical Bioinformatician, Journal Reviewer, Funding Agency Evaluator, Scientific Impact Analyst, Domain Expert, Systems Architect, Security Engineer, DevOps/CI Engineer, Open-Source Community Manager) with prioritized action items ([#14](https://github.com/Traitome/oxo-call/pull/14))

### Bug Fixes

- **Atomic write race condition:** Prevent `ENOENT` race on atomic temp-file rename during concurrent writes to shared JSONL history ([#13](https://github.com/Traitome/oxo-call/pull/13))
- **`cargo publish --locked` failure:** Resolve stale `Cargo.lock` preventing crates.io publish ([#10](https://github.com/Traitome/oxo-call/pull/10))

### CI/CD

- **CI hardening:** `cargo audit`, code coverage with `cargo-tarpaulin`, Codecov upload, and SHA256 checksums for all release binaries ([#14](https://github.com/Traitome/oxo-call/pull/14))

---

## [0.1.2] — 2026-03-12

### Other Changes

- Expand built-in skill coverage (increased skill count); patch release bump

---

## [0.1.1] — 2026-03-12

### Features

- **Initial public release** of oxo-call — model-intelligent CLI orchestration for bioinformatics
- **`run` command:** Generate and execute CLI arguments for any tool using LLM intelligence
- **`dry-run` command:** Preview generated command without executing
- **`--ask` flag:** Prompt for confirmation before executing the generated command
- **Docs-first grounding:** Tool `--help` output captured, cached, and injected into the LLM prompt to prevent hallucination
- **150+ built-in skills:** Expert knowledge (concepts, pitfalls, worked examples) for bioinformatics tools spanning alignment, variant calling, QC, RNA-seq, epigenomics, metagenomics, single-cell, and more
- **4 LLM providers:** GitHub Copilot (default), OpenAI, Anthropic, Ollama (local/air-gapped)
- **Ed25519 offline license verification:** Academic (free) and commercial licenses with tamper-proof offline validation
- **JSONL command history:** Every execution recorded with UUID, exit code, timestamp, and tool name
- **`config` command:** `config set/get/show/verify/path` for layered configuration with environment variable overrides
- **`skill` command:** `skill list/show/create/install` for managing built-in and user-defined skill TOML files
- **`history` command:** `history list/show/clear` for reviewing past commands
- **`index` / `docs` commands:** Documentation index management
- **`license` command:** `license verify/show` for license inspection
- **Cross-platform binaries:** Linux (x86_64/aarch64, glibc/musl), macOS (Intel/Apple Silicon), Windows (x86_64/aarch64), WASM (wasm32-wasip1)
- **Automatic SHA256 checksums:** `SHA256SUMS.txt` published with each GitHub Release
- **CITATION.cff:** CFF v1.2.0 metadata for academic citation

### CI/CD

- Cross-platform release builds triggered on version tags ([#5](https://github.com/Traitome/oxo-call/pull/5))
- WebAssembly build target (`wasm32-wasip1`) added to CI and GitHub Pages landing page ([#7](https://github.com/Traitome/oxo-call/pull/7))
- GitHub Actions updated to Node.js 24-compatible versions ([#8](https://github.com/Traitome/oxo-call/pull/8))

### Bug Fixes

- Non-atomic writes to shared files causing flaky tests fixed ([#4](https://github.com/Traitome/oxo-call/pull/4))
- macOS CI runner updated from deprecated `macos-13` to `macos-latest` ([#9](https://github.com/Traitome/oxo-call/pull/9))

<!-- next-url -->
[Unreleased]: https://github.com/Traitome/oxo-call/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Traitome/oxo-call/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Traitome/oxo-call/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/Traitome/oxo-call/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Traitome/oxo-call/releases/tag/v0.1.1
