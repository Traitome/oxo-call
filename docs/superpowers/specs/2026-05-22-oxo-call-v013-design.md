# oxo-call v0.13 Design Spec

## Goal

Single LLM call → reliable CLI command for ANY tool, production-ready for research and industry.
Target: 100% structural correctness (zero hallucinated flags), high semantic accuracy.

## Architecture: 3-Stage Pipeline

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│ EXTRACT  │ →  │ GENERATE │ →  │VALIDATE  │
│ --help   │    │ 1 LLM    │    │ strip    │
│ +skill   │    │ call      │    │ +add     │
│ → schema │    │           │    │ +enforce │
└──────────┘    └──────────┘    └──────────┘
```

### Stage 1: EXTRACT

Parse `--help` output into `StructuredDoc` with flag catalog, subcommand list, usage pattern, examples. If a skill file exists, merge its flags (mark required), examples, and pitfalls as flag descriptions. Output is unified schema regardless of skill presence.

### Stage 2: GENERATE

Single LLM call with unified prompt. Two key additions:

**Cross-domain few-shot examples**: Show 1 example from a DIFFERENT CLI category before the tool-specific example. Categories: file ops (cp/mv), text processing (grep/sort), version control (git), system (ps/df). This teaches the general CLI pattern: subcommand → flags → files.

**Chain-of-thought in output**: Ask model to reason briefly before output. Format:
```
think: sort is the subcommand, -@ for threads (value 4), -o for output (value sorted.bam), input.bam is positional
ARGS: sort -@ 4 -o sorted.bam input.bam
```
The `think:` line is parsed and discarded; it helps the LLM self-regulate.

Prompt template (~200 tokens):
```
Tool: {tool}
Subcommand: {sub}
Task: {task}

{flag_list}

Cross-example: {cross_domain_example}
Tool-example: {matched_example}

Think step by step, then output:
think: <reasoning>
ARGS: <arguments>
```

### Stage 3: VALIDATE

Deterministic 5-step pipeline, never calls LLM:
1. STRIP: Remove flags not in unified schema (guarantees zero hallucination)
2. ENFORCE: Force pre-processor best subcommand match
3. ADD: Insert missing required flags from schema
4. REPLACE: Substitute generic filenames with task-specific ones
5. SAFETY: Reject injection, traversal, destructive commands

## Skill Integration

Skills SUPPLEMENT but never REPLACE. Pipeline is identical with or without skill. Skill provides: richer flag catalog (marks required), better examples (task-matched), pitfalls as context. No skill → works the same, just less complete schema.

## What Gets Removed

- `build_prompt_full/medium/compact` → single `build_unified_prompt`
- `suggest_command_two_step` → not needed
- Phase B template/rule bypass → validate stage handles correctness
- LLM-within-postprocessing → validate is deterministic only
- `TOOL_DEFAULT_FEW_SHOT` (229 bioinfo-only entries) → 25 cross-domain + bioinfo
- 50 tool-specific `if` blocks in prompts → schema-driven

## Evaluation: 5 Modes

| Mode | Skill | Doc | System Prompt | Purpose |
|------|-------|-----|---------------|---------|
| bare | ✗ | ✗ | ✗ | LLM raw ability baseline |
| prompt | ✗ | ✗ | ✓ | System prompt contribution |
| doc | ✗ | ✓ | ✓ | Doc-only capability |
| skill | ✓ | ✗ | ✓ | Skill-only capability |
| full | ✓ | ✓ | ✓ | Production mode |

## Scoring: Multi-Metric

Primary: flag_group_recall + flag_group_precision + flag_group_jaccard + subcommand_match + format_valid (weighted composite).

Supplementary:
- **ROUGE-L**: Longest common subsequence between generated and reference. Rewards partial structural correctness.
- **BertScore**: Optional feature gate (`--features bertscore`). Semantic similarity via embedding.
- **Custom hallucination score**: `hallucinated_flags / total_flags`. Penalizes invented flags.

## Prompt Design Principles

1. Single format for both doc and skill modes
2. Plain text ARGS: format (not JSON — less syntax overhead for small models)
3. Cross-domain few-shot teaches general CLI structure
4. Chain-of-thought reduces hallucination via self-regulation
5. Values pre-assigned to flags where possible (e.g., `-@ 4` not `-@ <INT>`)
6. Max 5 flags shown, prioritized by required → task-relevant → output/thread

## Implementation Sequence

1. Rewrite `prompt.rs`: unified prompt builder, cross-domain few-shot, CoT support
2. Simplify `provider.rs`: ~100-line suggest_command, single call_api path
3. Rewrite `postprocess.rs`: 5-step validate pipeline
4. Update `response.rs`: ARGS: line + think: line parsing
5. Simplify config: remove PromptTierConfig
6. Update runner/core.rs: integrate validate stage
7. Update benchmarks: 5-mode evaluation, ROUGE-L, BertScore
8. Clean up dead code, ensure make ci passes
9. Test with qwen2.5-coder:7b, deepseek-coder-v2:16b, DeepSeek-V4-Flash
10. Iterate on prompt/validate based on results

## Success Criteria

- make ci passes (fmt, clippy, build, test)
- All 2,211 existing tests pass
- 5-mode benchmark runs complete
- Subcommand match rate > 95%
- Zero hallucinated flags in final output (validate step 1 guarantees this)
- Format validity 100%
- System works for any CLI tool (not just bioinformatics)
