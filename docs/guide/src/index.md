# Introduction

**oxo-call** is an AI-powered command orchestration tool for bioinformatics. Instead of memorizing hundreds of flags across dozens of tools, you describe what you want to accomplish — and oxo-call translates that into a correct, grounded command.

```bash
# You write:
oxo-call dry-run samtools "sort input.bam by coordinate using 4 threads"

# oxo-call generates:
samtools sort -@ 4 -o sorted.bam input.bam
```

This is not a simple LLM chat. oxo-call fetches the tool's actual `--help` output, injects curated expert knowledge (called a *skill*), and only then asks the LLM — ensuring every generated command uses real flags, not hallucinations.

---

## How It Works

![Command Generation Pipeline](./images/command-flow.svg)

The docs answer *"what flags exist?"* The skill answers *"which flags should I use, and what mistakes should I avoid?"* Together, they produce commands that are both syntactically correct and semantically appropriate.

---

## Core Capabilities

| Capability | What it means for you |
|------------|----------------------|
| **150+ built-in skills** | Expert knowledge for samtools, STAR, BWA, GATK, bcftools, fastp, and 145+ more |
| **Auto documentation** | `--help` cached on first use — no setup required |
| **Dry-run mode** | Preview every command before it runs |
| **Workflow engine** | Native DAG pipelines with Snakemake/Nextflow export |
| **History with provenance** | Every command logged with tool version, model, docs hash |
| **Local LLM support** | Run with Ollama for air-gapped or sensitive data |

---

## Who Is This For?

**Bioinformaticians** working with many CLI tools daily — oxo-call handles flag lookup so you can focus on biology, not man pages.

**Researchers** who need reproducible pipelines — every generated command is logged with the documentation and model that produced it.

**Core facility staff** supporting multiple assay types — a single tool covers NGS, single-cell, metagenomics, and more.

**Students** learning bioinformatics for the first time — describe tasks in plain language and learn by reading the generated explanations.

---

## How to Use This Guide

This documentation is organized as a hands-on tutorial book:

### If you are new to oxo-call
Start with **Getting Started**:

1. [Installation](./tutorials/installation.md) — install the binary
2. [License Setup](./tutorials/license.md) — get your free academic license
3. [Configuration](./tutorials/configuration.md) — connect your LLM
4. [Your First Command](./tutorials/first-command.md) — run something real in 10 minutes

### If you want hands-on practice
Work through the **Tutorials** in order:

- [SAM/BAM Processing](./tutorials/bam-workflow.md) — complete BAM pipeline
- [RNA-seq Walkthrough](./tutorials/rnaseq-walkthrough.md) — end-to-end analysis
- [Workflow Builder](./tutorials/workflow-builder.md) — automate multi-sample runs

### If you need to accomplish a specific task
Jump to **How-to Guides**:

- [Add docs for a new tool](./how-to/add-tool-docs.md)
- [Switch LLM provider](./how-to/change-llm-provider.md)
- [Create a custom skill](./how-to/create-custom-skill.md)
- [Build a production pipeline](./how-to/build-pipeline.md)

### If you need the full details
See **Command Reference** and **Architecture & Design** for complete specifications.

---

## Quick Example

Here is what a five-minute oxo-call session looks like:

```bash
# 1. Install
cargo install oxo-call

# 2. Set up (license + LLM token assumed)
export OXO_CALL_LICENSE=~/.config/oxo-call/license.oxo.json
oxo-call config set llm.api_token ghp_...

# 3. Preview a command
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
# → samtools sort -o sorted.bam input.bam

# 4. Execute it
oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"

# 5. Review what ran
oxo-call history list
```

Ready to begin? → [Installation](./tutorials/installation.md)

---

## Join the Community

oxo-call is a **user-driven, feedback-driven project**. Every bug report, feature request, and real-world use case you share directly influences what gets built next.

We actively welcome early adopters and testers — from students running their first RNA-seq pipeline to seasoned bioinformaticians automating complex workflows.

| How to contribute | Link |
|-------------------|------|
| 🐛 Report a bug | [Bug report](https://github.com/Traitome/oxo-call/issues/new?template=bug_report.md) |
| 💡 Request a feature | [Feature request](https://github.com/Traitome/oxo-call/issues/new?template=feature_request.md) |
| 🎯 Request a skill for a new tool | [Skill request](https://github.com/Traitome/oxo-call/issues/new?template=skill_request.md) |
| 🤝 Contribute code or skills | [Contributing guide](./development/contributing.md) |

> **Try it, break it, and tell us what happened.** Even a one-line comment on what went wrong — or right — helps us improve the tool for the whole community.
