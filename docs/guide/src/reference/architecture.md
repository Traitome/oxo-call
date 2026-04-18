# System Architecture

## Overview

oxo-call is a Rust workspace with three crates:

| Crate | Purpose | Published |
|-------|---------|-----------|
| `oxo-call` (root) | End-user CLI | Yes (crates.io) |
| `crates/license-issuer` | Maintainer-only license signing tool | No |
| `crates/oxo-bench` | Benchmarking and evaluation suite | No |

The architecture is designed around a layered system that makes command generation usable in production science and engineering workflows. The core idea: **Describe your task in plain language — oxo-call fetches the tool's documentation, asks your LLM backend to generate the exact flags you need.**

## Layered Architecture

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          User Interface Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  CLI Client  │  │  Chat Mode   │  │  Web API     │  │  SDK/API   │ │
│  │  (cli.rs)    │  │  (chat.rs)   │  │  (server.rs) │  │  (lib.rs)  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     Language Processing Layer                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Universal Task Translator (Any Language → Optimized English)   │  │
│  │  • task_normalizer.rs  • task_complexity.rs  • sanitize.rs      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     AI Orchestration Layer                              │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Runner Pipeline (runner/)                                      │  │
│  │  • core.rs (orchestration)  • batch.rs (parallel execution)     │  │
│  │  • retry.rs (error recovery)  • utils.rs (tool detection)       │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  LLM Integration (llm/)                                         │  │
│  │  • provider.rs (multi-provider support)  • types.rs (traits)    │  │
│  │  • Copilot / OpenAI / Anthropic / Ollama / DeepSeek / etc.      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Command Generation (generator.rs)                              │  │
│  │  • LLM-based  • Rule-based  • Composite strategies              │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     Knowledge Enhancement Layer                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Documentation System                                           │  │
│  │  • docs.rs (resolver + caching)  • doc_processor.rs (extraction)│  │
│  │  • doc_summarizer.rs (compression)  • index.rs (search index)   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  Skill       │  │  MCP Skill   │  │  Mini Skill  │  │  Context   │ │
│  │  Manager     │  │  Provider    │  │  Cache       │  │  Builder   │ │
│  │  (skill.rs)  │  │  (mcp.rs)    │  │              │  │ (context.rs│ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     Execution & Monitoring Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  Workflow     │  │  DAG         │  │  History     │  │  Job       │ │
│  │  Templates    │  │  Engine      │  │  Tracker     │  │  Manager   │ │
│  │ (workflow.rs) │  │ (engine.rs)  │  │ (history.rs) │  │  (job.rs)  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Workflow Graph Visualization (workflow_graph.rs)                │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     Infrastructure Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  LLM Backend │  │  Cache Layer │  │  Config      │  │  Remote    │ │
│  │  (Multiple   │  │  (cache.rs)  │  │  Management  │  │  Execution │ │
│  │   Providers) │  │              │  │ (config.rs)  │  │ (server.rs)│ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │  License     │  │  Error       │  │  Copilot     │                  │
│  │  Verifier    │  │  Handling    │  │  Auth        │                  │
│  │ (license.rs) │  │ (error.rs)   │  │(copilot_auth)│                  │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Layer Descriptions

**User Interface Layer** — Multiple entry points for interacting with oxo-call:
- **CLI Client** (`cli.rs`, `main.rs`): Primary command-line interface with Clap-based argument parsing
- **Chat Mode** (`chat.rs`): Interactive conversational AI for bioinformatics tool guidance
- **Web API** (`server.rs`): Remote server management for SSH/HPC execution
- **SDK/API** (`lib.rs`): Programmatic Rust API for downstream crates and integrations

**Language Processing Layer** — Normalizes and analyzes user input before LLM processing:
- **Task Normalizer** (`task_normalizer.rs`): Translates natural-language tasks into optimized prompts
- **Task Complexity** (`task_complexity.rs`): Estimates task complexity for adaptive prompt tier selection
- **Sanitizer** (`sanitize.rs`): Anonymizes sensitive data before sending to LLM

**AI Orchestration Layer** — Core intelligence pipeline:
- **Runner Pipeline** (`runner/`): Orchestrates the full docs→skill→LLM→execute flow
- **LLM Integration** (`llm/`): Multi-provider abstraction (GitHub Copilot, OpenAI, Anthropic, Ollama, DeepSeek, and more)
- **Command Generator** (`generator.rs`): Extensible generation strategies via the `CommandGenerator` trait

**Knowledge Enhancement Layer** — Grounds LLM calls in real documentation and domain expertise:
- **Documentation System** (`docs.rs`, `doc_processor.rs`, `doc_summarizer.rs`): Fetches, parses, and caches tool documentation
- **Skill System** (`skill.rs`): Domain-specific knowledge injection (user → community → MCP → built-in)
- **MCP Provider** (`mcp.rs`): Model Context Protocol for external skill servers
- **Context Builder** (`context.rs`): Assembles enriched context for LLM prompts

**Execution & Monitoring Layer** — Runs commands and tracks results:
- **Workflow Engine** (`engine.rs`): DAG-based parallel workflow execution with tokio
- **Workflow Templates** (`workflow.rs`): Pre-built bioinformatics pipelines (RNA-seq, WGS, etc.)
- **History Tracker** (`history.rs`): JSONL command history with full provenance (UUID, model, exit code)
- **Job Manager** (`job.rs`): Background job tracking and management

**Infrastructure Layer** — Platform services and configuration:
- **LLM Backend**: Multi-provider support with adaptive prompt tiers
- **Cache Layer** (`cache.rs`): Semantic hash-based response caching to reduce API costs
- **Config Management** (`config.rs`): TOML-based configuration with environment variable overrides
- **License Verifier** (`license.rs`): Ed25519 offline license verification

## Module Structure

```text
main.rs             — Command dispatcher & license gate
  ├─→ cli.rs        — Command definitions (Clap)
  ├─→ handlers.rs   — Extracted command-handler helpers (formatting, suggestions)
  ├─→ license.rs    — Ed25519 offline verification
  ├─→ runner/       — Core orchestration pipeline + provenance tracking
  │     ├─→ core.rs            — Main runner logic
  │     ├─→ batch.rs           — Batch/parallel execution
  │     ├─→ retry.rs           — Auto-retry with error recovery
  │     └─→ utils.rs           — Tool detection & spinner utilities
  ├─→ docs.rs                  — Documentation resolver
  ├─→ doc_processor.rs         — Structured doc extraction (flag catalog, examples)
  ├─→ doc_summarizer.rs        — Documentation compression
  ├─→ skill.rs                 — Skill loading system + depth validation
  │     └─→ mcp.rs             — MCP skill provider (JSON-RPC / HTTP)
  ├─→ llm/                     — LLM integration
  │     ├─→ provider.rs        — Multi-provider client
  │     └─→ types.rs           — LlmProvider trait & types
  ├─→ llm_workflow.rs          — Fast/Quality workflow executor
  ├─→ generator.rs             — CommandGenerator trait (extensible strategies)
  ├─→ cache.rs                 — LLM response cache with semantic hash
  ├─→ history.rs               — Command history tracker with provenance
  ├─→ chat.rs                  — Interactive AI chat mode
  ├─→ sanitize.rs              — Data anonymization for LLM contexts
  ├─→ server.rs                — Remote server management (SSH / HPC)
  ├─→ workflow.rs              — Templates & registry
  │     └─→ engine.rs          — DAG execution engine
  ├─→ workflow_graph.rs        — DAG visualization
  ├─→ task_normalizer.rs       — Task normalization
  ├─→ task_complexity.rs       — Complexity estimation
  ├─→ context.rs               — Context assembly
  ├─→ config.rs                — Configuration management
  ├─→ index.rs                 — Documentation index
  ├─→ job.rs                   — Job management
  ├─→ format.rs                — Output formatting
  ├─→ mini_skill_cache.rs      — Lightweight skill caching
  ├─→ copilot_auth.rs          — GitHub Copilot authentication
  └─→ error.rs                 — Error type definitions
lib.rs              — Programmatic API surface (re-exports all modules)
```

## Execution Flow

### Command Generation (run/dry-run)

```text
1. License verification (Ed25519 signature check)
2. Documentation fetch (cache → --help → local files → remote URLs)
3. Structured doc extraction (flag catalog + command examples, deterministic)
4. Skill loading (user → community → MCP → built-in)
5. Doc-enriched prompt construction:
   - Flag catalog → "Valid Flags" section (prevents hallucination)
   - Doc-extracted examples → few-shot demonstrations (critical for ≤3B)
   - Skill knowledge → expert grounding (when available)
   - Task + context → user message
6. LLM API call (single call: GitHub Copilot / OpenAI / Anthropic / Ollama)
7. Response parsing (extract ARGS: and EXPLANATION: lines)
8. Flag validation against doc catalog (post-processing)
9. Command execution (run) or display (dry-run)
10. History recording (JSONL with UUID, exit code, timestamp)
```

### Workflow Execution

```text
1. Parse .oxo.toml workflow definition
2. Expand wildcards ({sample}, {params.*})
3. Build dependency DAG
4. Topological sort for execution order
5. Execute with tokio parallelism (JoinSet)
6. Skip steps with fresh outputs (output-freshness caching)
```

## Design Principles

1. **License-first**: Core commands require valid Ed25519 signature
2. **Docs-first grounding**: Documentation fetched before LLM call to prevent hallucination
3. **Offline-first**: Cached docs, no license server, optional remote fetching
4. **Skill-augmented prompting**: Domain knowledge injected without code changes
5. **Native performance**: Direct native compilation for all major platforms (Linux, macOS, Windows)
6. **Strict LLM contract**: ARGS:/EXPLANATION: format with retry on invalid response
7. **Adaptive prompt compression**: Three prompt tiers (Full/Medium/Compact) auto-selected by model size and context window, ensuring reliable output from 0.5B to 200B+ parameter models
8. **Extensible generation**: CommandGenerator trait enables multiple generation strategies (LLM, rule-based, composite) with chain-of-responsibility pattern
9. **Response caching**: Optional LLM cache reduces API costs for repeated tasks via semantic hash (tool + task + docs + skill + model)

## Why This Matters In Practice

- **Usability**: users can stay in natural language longer and only inspect flags when it matters
- **Reliability**: docs-first grounding and a strict response contract reduce free-form model drift
- **Scientific reproducibility**: provenance-rich history preserves the command, model, and context that produced each result
- **Engineering extensibility**: skills, MCP providers, and workflow export let teams expand capability without rewriting the core pipeline
