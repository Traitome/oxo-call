# System Architecture

## Overview

oxo-call is a Rust workspace with three crates:

| Crate | Purpose | Published |
|-------|---------|-----------|
| `oxo-call` (root) | End-user CLI | Yes (crates.io) |
| `crates/license-issuer` | Maintainer-only license signing tool | No |
| `crates/oxo-bench` | Benchmarking and evaluation suite | No |

## Module Structure

The main CLI crate contains 13 modules with clear separation of concerns:

```text
main.rs (1089 lines)
  ├─→ cli.rs (385)      — Command definitions (Clap)
  ├─→ license.rs (531)  — Ed25519 offline verification
  ├─→ runner.rs (246)   — Core orchestration pipeline
  │     ├─→ docs.rs (695)     — Documentation resolver
  │     ├─→ skill.rs (495)    — Skill loading system
  │     ├─→ llm.rs (404)      — LLM client & prompt builder
  │     └─→ history.rs (66)   — Command history tracker
  ├─→ workflow.rs (773) — Templates & registry
  │     └─→ engine.rs (759)   — DAG execution engine
  ├─→ config.rs (383)   — Configuration management
  ├─→ index.rs (244)    — Documentation index
  └─→ error.rs (41)     — Error type definitions
```

## Execution Flow

### Command Generation (run/dry-run)

```text
1. License verification (Ed25519 signature check)
2. Documentation fetch (cache → --help → local files → remote URLs)
3. Skill loading (user → community → built-in)
4. Prompt construction (docs + skill + task → system + user message)
5. LLM API call (GitHub Copilot / OpenAI / Anthropic / Ollama)
6. Response parsing (extract ARGS: and EXPLANATION: lines)
7. Command execution (run) or display (dry-run)
8. History recording (JSONL with UUID, exit code, timestamp)
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
