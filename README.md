<div align="center">

# oxo-call

**Reliable natural-language orchestration for CLI bioinformatics**

[![CI](https://github.com/Traitome/oxo-call/actions/workflows/ci.yml/badge.svg)](https://github.com/Traitome/oxo-call/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/oxo-call.svg)](https://crates.io/crates/oxo-call)
[![install with bioconda](https://img.shields.io/badge/install%20with-bioconda-brightgreen.svg?style=flat)](http://bioconda.github.io/recipes/oxo-call/README.html)
[![License](https://img.shields.io/badge/license-Academic%20%7C%20Commercial-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](#data-storage)
[![codecov](https://codecov.io/gh/Traitome/oxo-call/graph/badge.svg?token=HDGVNW96XB)](https://codecov.io/gh/Traitome/oxo-call)
[![Docs](https://img.shields.io/badge/docs-guide-blue.svg)](https://traitome.github.io/oxo-call/documentation/)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Traitome/oxo-call)
[![Conda](https://img.shields.io/conda/dn/bioconda/oxo-call.svg)](https://anaconda.org/bioconda/oxo-call/files)
[![GitHub Releases](https://img.shields.io/github/downloads/Traitome/oxo-call/total.svg)](https://github.com/Traitome/oxo-call/releases)
[![Crates.io Downloads](https://img.shields.io/crates/d/oxo-call.svg)](https://crates.io/crates/oxo-call)

Describe your task in plain language — `oxo-call` fetches the tool's documentation, grounds the request with a built-in skill, and asks your LLM backend to generate the exact flags you need with auditable, benchmarked context.

**→ Full documentation, tutorials, and how-to guides: [traitome.github.io/oxo-call/documentation](https://traitome.github.io/oxo-call/documentation/)**

</div>

---

## What is oxo-call?

oxo-call is an AI-powered CLI assistant for bioinformatics. Instead of memorizing hundreds of flags across dozens of tools, you describe what you want to accomplish — oxo-call translates that into a correct, grounded command you can inspect, explain, and reproduce.

```
  Natural-language task  ──▶  Documentation + Skill  ──▶  LLM  ──▶  Exact command
```

- 🧠 **LLM-powered** — GitHub Copilot, OpenAI, Anthropic, or local Ollama
- 📚 **Docs-grounded** — tool `--help` output is cached and injected before every LLM call
- 🎯 **Skill system** — built-in expert knowledge for 159 bioinformatics tools across 44 domains
- 🔄 **Workflow engine** — native DAG-based pipelines with Snakemake/Nextflow export
- 🔍 **Dry-run mode** — preview every command before it runs
- 📜 **History** — every execution is logged with provenance metadata
- 📋 **Job library** — save named command shortcuts with scheduling, history, and LLM generation (`oxo-call job`)
- 📊 **Benchmarked** — [Systematic evaluation framework](docs/bench/BENCHMARK.md) across 143 bioinformatics tools with 14,300 descriptions; run `oxo-bench eval` with your own API key to measure real LLM accuracy

## Why teams adopt oxo-call

- **Usable immediately** — install, connect an LLM, and get a first working command in minutes
- **Safer than prompt-only command generation** — oxo-call grounds every request in real docs plus tool-specific pitfalls and examples
- **Easy to learn from** — dry-run output and explanations help students and new lab members understand unfamiliar tools
- **Built for reproducible science** — history records the generated command, model, tool version, and docs provenance
- **Engineered for real labs** — supports local models, HPC environments, workflow export, and a growing cross-domain skill library

---

## Quick Start

### 1. Install (Recommended: Pre-built Binaries)

The easiest way to install oxo-call is to download pre-built binaries from GitHub Releases:

```bash
# Linux/macOS
curl -fsSL https://github.com/Traitome/oxo-call/releases/latest/download/oxo-call-linux-x86_64 -o oxo-call
chmod +x oxo-call
sudo mv oxo-call /usr/local/bin/

# Or macOS (Apple Silicon)
curl -fsSL https://github.com/Traitome/oxo-call/releases/latest/download/oxo-call-macos-aarch64 -o oxo-call
chmod +x oxo-call
sudo mv oxo-call /usr/local/bin/

# Or Windows
# Download from: https://github.com/Traitome/oxo-call/releases
```

See the [Installation guide](https://traitome.github.io/oxo-call/documentation/tutorials/installation/) for all platforms and options.

### Alternative: Install via Conda (Bioconda)

If you use conda, you can install oxo-call from Bioconda:

```bash
# Install bioconda if you haven't already
# conda config --add channels conda-forge
# conda config --add channels bioconda
# conda config --set channel_priority strict

conda install oxo-call -c bioconda -c conda-forge
```

Or with mamba (faster):

```bash
mamba install oxo-call -c bioconda -c conda-forge
```

> **Note:** Bioconda support is experimental. Please report any issues at https://github.com/Traitome/oxo-call/issues.

### Alternative: Install via Cargo

If you have Rust installed, you can also install via cargo:

```bash
cargo install oxo-call
```

### 2. Get a license

A signed license is required for core commands. Academic licenses are **free**.

```bash
# Apply at: https://github.com/Traitome/oxo-call#license
# Then place your license file or set the environment variable:
export OXO_CALL_LICENSE=/path/to/license.oxo.json
```

### 3. Configure your LLM and run

```bash
# GitHub Copilot (default) - Interactive OAuth login
oxo-call config login

# Or set manually with a GitHub token
oxo-call config set llm.api_token <your-github-token>

# Preview a command without running it
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
# → samtools sort -o sorted.bam input.bam

# Execute a command
oxo-call run bwa "align reads.fastq to reference.fa using 8 threads"

# Run a built-in workflow pipeline
oxo-call workflow dry-run rnaseq
```

For OpenAI, Anthropic, Ollama, and full configuration details, see the [Configuration guide](https://traitome.github.io/oxo-call/documentation/tutorials/configuration/).

---

## Documentation

The full documentation is organized so new users can get value quickly while advanced users can dig into architecture and workflow automation:

| Section | What you'll learn |
|---------|-------------------|
| [Getting Started](https://traitome.github.io/oxo-call/documentation/tutorials/installation/) | Install quickly, configure once, and reach your first useful command fast |
| [Tutorials](https://traitome.github.io/oxo-call/documentation/tutorials/first-command/) | Learn by doing: command preview, BAM processing, RNA-seq, workflows |
| [How-to Guides](https://traitome.github.io/oxo-call/documentation/how-to/add-tool-docs/) | Solve targeted problems such as adding docs, switching providers, or building pipelines |
| [Command Reference](https://traitome.github.io/oxo-call/documentation/commands/run/) | Look up exact flags, options, and command behavior |
| [Architecture](https://traitome.github.io/oxo-call/documentation/reference/architecture/) | Understand the engineering decisions behind reliability, provenance, and extensibility |

---

## Community & Feedback

oxo-call is a **user-driven project** — your real-world usage and feedback directly shape what gets built next.

We are actively looking for early adopters and testers across all stages of bioinformatics work. The more you use it and report back, the better it gets for everyone.

- 🐛 **Found a bug?** [Open a bug report](https://github.com/Traitome/oxo-call/issues/new?template=bug_report.md)
- 💡 **Have a feature idea?** [Request a feature](https://github.com/Traitome/oxo-call/issues/new?template=feature_request.md)
- 🎯 **Missing a skill for your tool?** [Request a skill](https://github.com/Traitome/oxo-call/issues/new?template=skill_request.md)
- 📣 **Share how you use it** — real-world use cases help prioritize development

> We especially welcome feedback from students, researchers, and core facility staff who run oxo-call on real data. Every issue filed and every comment left makes the tool better!

---

## License

**Dual License — Academic Free / Commercial Per-Organization**

| Use case | License | Cost |
|----------|---------|------|
| Academic research, education, personal non-commercial | [LICENSE-ACADEMIC](LICENSE-ACADEMIC) | **Free** — signed license file required |
| Commercial / production | [LICENSE-COMMERCIAL](LICENSE-COMMERCIAL) | **USD 200** — per-org, one-time fee |

Apply for an academic license or contact `w_shixiang@163.com` for commercial licensing.

A public test license for evaluation is available at `docs/public-academic-test-license.oxo.json`.

> Skill files contributed to the community registry are licensed under **CC-BY-4.0**.
