# oxo-call

**Model-intelligent orchestration for CLI bioinformatics**

oxo-call transforms the way bioinformaticians interact with command-line tools. Instead of memorizing hundreds of flags and parameters, you describe your task in plain language — oxo-call fetches the tool's documentation, grounds the request with built-in expert knowledge (skills), and asks your LLM backend to generate the exact flags you need.

## Key Features

- 🧠 **LLM-powered parameter generation** — describe what you want to do, get the right flags
- 📚 **Auto documentation** — `--help` output is cached transparently on first use; enrich with remote URLs, local files, or directories
- 🗂️ **Unified docs management** — `oxo-call docs add/list/show/remove/update` manages the documentation index
- 🔍 **Dry-run mode** — preview commands before executing
- 📜 **Command history** — track every run with exit codes and timestamps
- 🔧 **Flexible LLM backend** — GitHub Copilot (default), OpenAI, Anthropic, Ollama
- 🎯 **Skill system** — built-in expert knowledge for 150+ bioinformatics tools
- 🔄 **Workflow engine** — native DAG-based pipeline execution with Snakemake/Nextflow export

## Architecture Overview

```text
  Natural-language task
         │
         ▼
  ┌──────────────────────────────────────────────────────────┐
  │                      oxo-call CLI                        │
  │                                                          │
  │  ┌──────────────────────────────────────────────────┐   │
  │  │              Documentation Layer                  │   │
  │  │  --help output · local index cache · remote URL  │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │ combined docs + task          │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │                  Skill System                     │   │
  │  │       built-in → community registry → user       │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │ grounded prompt               │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │                  LLM Backend                      │   │
  │  │   GitHub Copilot · OpenAI · Anthropic · Ollama   │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │ ARGS: ...                     │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │              Command Execution                    │   │
  │  │        run · dry-run · run --ask                 │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │                               │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │            History & Output                       │   │
  │  │       JSONL log · exit code · timestamp          │   │
  │  └──────────────────────────────────────────────────┘   │
  └──────────────────────────────────────────────────────────┘
```

## Who is this for?

- **Bioinformaticians** who work with many CLI tools and want to avoid flag memorization
- **Researchers** who need reproducible, documented command pipelines
- **Core facility staff** who support multiple analysis workflows
- **Students** learning bioinformatics tools for the first time
