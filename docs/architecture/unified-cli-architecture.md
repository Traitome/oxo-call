# Hierarchical Deterministic Architecture (HDA) for oxo-call

## Overview

HDA (Hierarchical Deterministic Architecture) is the core architecture of oxo-call, designed to achieve high accuracy in CLI command generation for bioinformatics tools, especially on small models (3B parameters).

## Target Metrics

- **doc mode accuracy**: ≥ 0.80
- **skill mode accuracy**: ≥ 0.95
- **Model scale**: 3B+ models should achieve stable, reliable results

## Core Theory

HDA separates LLM generation into two categories:

1. **Deterministic operations** — can be completed with 100% reliability through code, no LLM needed:
   - Intent parsing (keyword matching)
   - Schema matching (subcommand selection)
   - Flag whitelisting (schema-based validation)
   - Format validation (syntactic checking)

2. **Probabilistic operations** — require LLM inference:
   - Value filling (file paths, parameters)
   - Fuzzy matching (intent → subcommand mapping when ambiguous)

By maximizing deterministic operations and minimizing probabilistic ones, HDA reduces hallucination at the architecture level.

## Architecture Layers

### Layer 1: Schema IR (Intermediate Representation)

**Module**: `src/schema/`

The Schema IR is a unified formal model for representing CLI tool interfaces. It provides:

- **CliSchema**: Complete representation of a tool's interface
  - `cli_style`: Subcommand / Direct / IndexAction / LongOption
  - `flags`: Tool-level flags with types, descriptions, defaults
  - `subcommands`: Subcommand definitions with their own flags
  - `global_flags`: Flags that apply to all subcommands
  - `positionals`: Positional argument definitions
  - `constraints`: Mutual exclusion, dependency rules

- **FlagSchema**: Individual flag definition
  - Names (short + long), param_type, required, description, default
  - `validate_value()`: Type-aware value validation
  - `matches_name()`: Flexible name matching (short/long)

- **ValidationResult**: Schema-based validation output
  - `is_valid`, `errors`, `warnings`, `used_flags`, `valid_flags`
  - `validate_command()`: Validate (flag, value) pairs against schema

**Schema Parsers** (`src/schema/parser/`):
- `generic.rs`: Parse any CLI tool's `--help` output
- `python.rs`: Parse Python argparse-based tools

**Schema Generator** (`src/schema/generator.rs`):
- `build_schema_prompt_section()`: Generate constrained prompt from schema
- `suggest_subcommand_for_task()`: Deterministic subcommand selection
- `suggest_flags_for_task()`: Keyword-based flag suggestion
- `validate_command_against_schema()`: Post-generation validation

### Layer 2: Confidence-Driven Workflow

**Module**: `src/confidence/`

Replaces the heuristic Fast/Quality mode with a scientific confidence estimation system.

**ConfidenceResult** components:
- `schema_coverage` (weight 0.4): How much of the tool interface is captured
- `intent_clarity` (weight 0.3): How clearly the task specifies the operation
- `model_capability` (weight 0.2): Model's instruction-following ability
- `skill_availability` (weight 0.1): Whether a skill file is available

**Confidence levels and strategies**:

| Level | Score | Strategy |
|-------|-------|----------|
| High | ≥ 0.7 | SingleCall — one LLM call with schema constraints |
| Medium | 0.4–0.7 | ValidationRetry — single call + validation + up to 2 retries |
| Low | < 0.4 | ThinkingMode — multi-stage reasoning with intent parsing |

**Key function**: `estimate_confidence(schema_flags, desc_coverage, keyword_match, file_mentions, model_params, has_skill)`

### Layer 3: Schema-Guided Prompt Generation

**Module**: `src/llm/prompt.rs`

When a schema is available, the prompt includes:

1. **Subcommand Selection** — lists valid subcommands with recommended one
2. **Global Flags** — complete list with type hints and descriptions
3. **Subcommand-specific Flags** — only flags for the selected subcommand
4. **Positional Arguments** — required/optional with descriptions
5. **Flag Constraints** — mutual exclusion, dependency rules

This replaces heuristic flag extraction with formal Schema IR, providing:
- Complete whitelist of valid flags (prevents hallucinated flags)
- Type hints for each flag (⟨integer⟩, ⟨file⟩, ⟨text⟩, ⟨a|b|c⟩)
- Required flag indicators ([REQUIRED])
- Constraint enforcement (mutual exclusion, dependencies)

### Layer 4: Validation-Reflection Loop

**Modules**: `src/validation_loop.rs`, `src/reflection_engine.rs`, `src/command_validator.rs`

Post-generation validation and correction:

**CommandValidator** (`src/command_validator.rs`):
- Schema-based validation: checks flags against whitelist
- Subcommand detection and validation
- Auto-fix: remove invalid flags, add missing required flags
- Levenshtein distance for close-match suggestions

**ReflectionEngine** (`src/reflection_engine.rs`):
- Rule-based reflection using ValidationError types
- Approach classification: RemoveHallucinations, UseWhitelistFlags, AddSubcommand
- Retry decision based on error severity and iteration count

**ValidationReflectionLoop** (`src/validation_loop.rs`):
- Iterative validation → reflection → fix cycle
- Auto-fix first attempt, then reflection-guided fixes
- Configurable max iterations

**AutoFixer** (`src/auto_fixer.rs`):
- Priority-based fix application
- Fix types: ReplaceFlag, RemoveFlag, AddFlag, AddSubcommand
- Aggressive mode for constraint violations

### Layer 5: Orchestration

**Module**: `src/orchestrator/`

- **Supervisor** (`supervisor.rs`): Confidence-based orchestration decisions
  - SupervisorDecision includes confidence result and enrichment hints
  - OrchestrationMode selection based on confidence level

- **Planner** (`planner.rs`): Task decomposition
- **Executor** (`executor.rs`): Step execution
- **Validator** (`validator.rs`): Result verification

### Layer 6: Workflow Graph

**Module**: `src/workflow_graph.rs`

Scenario-based workflow with confidence integration:
- WorkflowState tracks normalized task, confidence, mode, scenario
- Confidence-based mode override (upgrade/downgrade)
- Adaptive workflow adjustment based on confidence level

## Data Flow

```
User Task
    │
    ▼
TaskNormalizer ──► NormalizedTask
    │
    ▼
Schema Parser ──► CliSchema (from docs/skill)
    │
    ▼
Confidence Estimator ──► ConfidenceResult
    │                         │
    ▼                         ▼
Workflow Decision ◄──── Strategy
    │
    ├── High ──► Single LLM Call (schema-guided prompt)
    ├── Medium ─► LLM Call + Validation + Retry
    └── Low ──► Multi-stage: IntentParse → SchemaPrompt → ValueFill → Validate
    │
    ▼
Schema-Guided Prompt Generation
    │
    ▼
LLM Generation
    │
    ▼
Validation-Reflection Loop
    │
    ├── Valid ──► Output
    └── Invalid ──► Reflect → Fix → Re-validate (up to max_iterations)
```

## CLI Pattern Taxonomy

Analysis of 6000+ bioconda CLI tools reveals 5 distinct patterns, captured in `CliStyle`:

| Pattern | CliStyle | Example Tools | Structure |
|---------|----------|---------------|-----------|
| A | Subcommand | samtools, bcftools, gatk | `tool subcommand -flags args` |
| B | Direct | fastp, minimap2, seqkit | `tool -flags args` |
| C | IndexAction | bwa, bowtie2, hisat2 | `tool-index ref.fa && tool mem ref.fa reads.fq` |
| D | LongOption | STAR, featureCounts | `tool --option=value args` |
| E | Subcommand | salmon, kallisto | `tool index -i idx ref.fa && tool quant -i idx reads.fq` |

## Module Map

| Module | HDA Layer | Purpose |
|--------|-----------|---------|
| `schema/` | Layer 1 | Schema IR definition, parsing, generation |
| `confidence/` | Layer 2 | Confidence estimation and workflow strategy |
| `llm/prompt.rs` | Layer 3 | Schema-guided prompt construction |
| `validation_loop.rs` | Layer 4 | Validation-reflection iteration |
| `reflection_engine.rs` | Layer 4 | Error analysis and guidance |
| `command_validator.rs` | Layer 4 | Schema-based command validation |
| `auto_fixer.rs` | Layer 4 | Priority-based command fixing |
| `orchestrator/` | Layer 5 | Confidence-based orchestration |
| `workflow_graph.rs` | Layer 6 | Scenario-based workflow with confidence |
| `llm_workflow.rs` | Core | Workflow execution with confidence integration |
| `runner/` | Core | Main execution pipeline |
| `skill.rs` | Data | Skill file management and rendering |
| `doc_summarizer.rs` | Data | Documentation processing |
| `sanitize.rs` | Utility | Output sanitization and flag deduplication |
| `task_normalizer.rs` | Utility | Task normalization |

## Key Design Decisions

1. **Schema over heuristics**: CliSchema provides a formal, machine-readable representation of tool interfaces, replacing heuristic-based flag extraction
2. **Confidence over complexity**: Workflow decisions are driven by measurable confidence scores, not heuristic complexity estimates
3. **Deterministic first**: Every operation that can be done deterministically (keyword matching, schema validation, flag whitelisting) is done in code, not by LLM
4. **Constrained generation**: Schema-guided prompts provide explicit whitelists and type hints, reducing the LLM's freedom to hallucinate
5. **Validation loop**: Post-generation validation catches and fixes errors before returning to the user
