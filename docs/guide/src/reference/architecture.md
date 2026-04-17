# System Architecture

![System Architecture Diagram](../images/architecture.svg)

## Overview

oxo-call is a Rust workspace with three crates:

| Crate | Purpose | Published |
|-------|---------|-----------|
| `oxo-call` (root) | End-user CLI | Yes (crates.io) |
| `crates/license-issuer` | Maintainer-only license signing tool | No |
| `crates/oxo-bench` | Benchmarking and evaluation suite | No |

The architecture is designed to make command generation usable in production science and engineering workflows, not just impressive in a demo. The key idea is that `oxo-call` reduces ambiguity before the model answers, then records enough provenance afterward for users to trust and reproduce the result.

## Module Structure

The main CLI crate contains the following modules with clear separation of concerns:

```text
main.rs             — Command dispatcher & license gate
  ├─→ cli.rs        — Command definitions (Clap)
  ├─→ handlers.rs   — Extracted command-handler helpers (formatting, suggestions)
  ├─→ license.rs    — Ed25519 offline verification
  ├─→ runner.rs     — Core orchestration pipeline + provenance tracking
  │     ├─→ docs.rs            — Documentation resolver
  │     ├─→ doc_processor.rs   — Structured doc extraction (flag catalog, examples, quality)
  │     ├─→ skill.rs           — Skill loading system + depth validation
  │     │     └─→ mcp.rs       — MCP skill provider (JSON-RPC / HTTP)
  │     ├─→ llm.rs             — LLM client, prompt builder & provider trait
  │     ├─→ llm_workflow.rs    — Fast/Quality workflow executor
  │     ├─→ cache.rs           — LLM response cache with semantic hash
  │     ├─→ generator.rs       — CommandGenerator trait (extensible strategies)
  │     └─→ history.rs         — Command history tracker with provenance
  ├─→ chat.rs       — Interactive chat with AI about bioinformatics tools
  ├─→ sanitize.rs   — Data anonymization for LLM contexts
  ├─→ server.rs     — Remote server management (SSH / HPC)
  ├─→ workflow.rs   — Templates & registry
  │     └─→ engine.rs      — DAG execution engine
  ├─→ config.rs     — Configuration management
  ├─→ index.rs      — Documentation index
  └─→ error.rs      — Error type definitions
lib.rs              — Programmatic API surface (re-exports all modules)
```

## Execution Flow

![Command Generation Flow](../images/command-flow.svg)

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
5. **Platform independence**: WASM conditional compilation, cross-platform config dirs
6. **Strict LLM contract**: ARGS:/EXPLANATION: format with retry on invalid response
7. **Adaptive prompt compression**: Three prompt tiers (Full/Medium/Compact) auto-selected by model size and context window, ensuring reliable output from 0.5B to 200B+ parameter models
8. **Extensible generation**: CommandGenerator trait enables multiple generation strategies (LLM, rule-based, composite) with chain-of-responsibility pattern
9. **Response caching**: Optional LLM cache reduces API costs for repeated tasks via semantic hash (tool + task + docs + skill + model)

## Why This Matters In Practice

- **Usability**: users can stay in natural language longer and only inspect flags when it matters
- **Reliability**: docs-first grounding and a strict response contract reduce free-form model drift
- **Scientific reproducibility**: provenance-rich history preserves the command, model, and context that produced each result
- **Engineering extensibility**: skills, MCP providers, and workflow export let teams expand capability without rewriting the core pipeline
