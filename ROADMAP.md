# oxo-call 1.0 Roadmap: Unix-First Command Intelligence

> **Status:** Product boundary, engineering invariants, and acceptance plan for the 1.0 release.
>
> **This is not a twenty-year prediction; it defines the auditable conditions that can keep the project worth adopting over time.**

## 1. Product decision

**oxo-call solves one reviewable tool invocation.** Its input is a tool name and human intent; its output is an inspectable command, explanation, and reproducible evidence. It may execute that command, but it no longer creates, persists, schedules, resumes, exports, or visualizes multi-step DAGs.

This is not a reduction in ambition. It is a deliberate reduction in responsibility: reliability comes from a small, clear, composable, and verifiable boundary, rather than duplicating another mature system's responsibilities in one binary.

| Question | oxo-call | [oxo-flow](https://github.com/Traitome/oxo-flow) |
|---|---|---|
| “How should I invoke this tool?” | Generate, explain, review, and run one command | Consume shell calls in rules or commands |
| Multi-sample dependencies, DAGs, caching, retries, and resource scheduling | Does not own | Owns |
| Environment isolation, cluster backends, run reports, and resumable execution | Does not own | Owns |
| Interactive command knowledge, `--help`, skills, and command provenance | Owns | May supplement at the rule layer |

`oxo-call workflow`, `.oxo.toml`, built-in DAG templates, and their Snakemake/Nextflow exports are removed legacy interfaces. This project will **not** pretend to continue supporting them through silent forwarding or compatibility shims. Users who need DAGs should preserve old files before migration, then model them again using oxo-flow's `.oxoflow` format, validation, and run documentation. Format conversion requires human review and must not be represented as a lossless automatic migration.

## 2. Long-lived engineering invariants

These invariants take precedence over feature work. Every proposal must state how it preserves them.

1. **One request, one command contract.** oxo-call does not maintain task graphs, cross-step state, dependency resolution, or scheduling semantics. An explicit shell pipeline remains user-reviewable Unix composition text; it does not become an oxo-call execution graph.
2. **Generation and execution are separable.** `dry-run` text and JSON output must be consumable by people, scripts, and other tools. `run` must not hide the generated command, risks, or execution result.
3. **Documentation first.** `Runner::prepare()` must obtain tool documentation before loading a skill and before requesting a model. Model output is always a proposal to validate, never an authority.
4. **Provenance rather than apparent intelligence.** Each invocation should connect intent, generated command, tool version, documentation hash, skill, model/configuration, timestamp, execution result, and policy decision.
5. **People retain final control.** Commands with high risk, destructive side effects, or insufficient evidence must be clearly marked and require explicit confirmation. Least privilege, minimum environment exposure, and default denial take priority over automation convenience.
6. **The interface outlives models.** Providers, models, and prompts may be replaced. The CLI, machine-readable output, error codes, provenance schema, and migration guidance need a versioned compatibility policy.
7. **Offline, portable, and maintainable.** Local models and local documentation caches are first-class paths. Releases, skills, and evaluation data must be locatable, replayable, and governed by maintainers.

## 3. Adversarial design review

| Attractive direction | Failure mode | Decision and guardrail |
|---|---|---|
| Re-add DAGs, caching, and cluster scheduling to oxo-call | Two incompatible workflow semantics, a larger security surface, and diluted maintenance | DAG lifecycle remains in oxo-flow; collaborate through links and a clear migration boundary |
| “Natural language to automatic execution” | Hallucination, prompt injection, file loss, or expensive cluster jobs | Documentation/skill grounding, deterministic preflight checks, risk classification, `dry-run`, and confirmation gates |
| Depend on one cloud model for quality | Data-governance risk, cost, vendor withdrawal, and irreproducible experiments | Provider-independent configuration, local-model paths, and recording of actual model and parameters |
| Claim command correctness through exact string matching | Equivalent flags, order, and environment differences create false negatives; semantic faults remain undetected | Keep exact matching as a regression signal; add functional-equivalence, flag, and side-effect evaluation in isolated environments |
| Substitute ever-larger prompts for engineering | Opaque behavior, increased cost and latency, and poor auditability | Structured documentation, versioned skills, deterministic checks, and the minimum necessary LLM calls |
| Retain deprecated workflow entry points for “compatibility” | Users infer production scheduling guarantees that do not exist and labs inherit the risk | Make the breaking change explicit, provide no silent fallback, and make the oxo-flow handoff visible |

## 4. Roadmap and release gates

Milestones are ordered by **acceptance evidence**, not dates. Marketing must not substitute for a missed gate.

### M0 — Scope reset (this change)

- Remove the public DAG CLI, native format, templates, executor, graph visualization, exports, and associated benchmarks.
- Rename internal single-command `Fast`/`Quality` inference to `GenerationMode`/`CommandGenerationPipeline`, so multiple LLM calls are not misrepresented as a workflow.
- Remove legacy history learning for multi-tool sequences; retain individual command history and provenance.
- State the oxo-flow handoff in the README, documentation site, help text, and contributor guidance.

**Exit gate:** `oxo-call --help` does not expose `workflow`; `oxo-call workflow --help` fails; no compilable `.oxo.toml`/DAG execution path remains; core commands, documentation build, and workspace tests pass.

### M1 — Stable command contract

- Publish explicit schemas, field meanings, exit codes, and compatibility windows for command text, JSON, and JSONL output.
- Give `dry-run` and `run` consistent request/result/provenance correlation IDs so scripts do not need to parse terminal colors.
- Extend the existing JSONL history into versioned W3C PROV-aligned records and provide lossless export without requiring a new database.
- Establish SemVer rules, deprecation process, migration guidance, and contract tests for the CLI and public Rust API.

**Exit gate:** Every stable field, error code, and deprecation has tests; breaking changes have a major version or explicit migration period; a generation record independently explains who generated which command, from which configuration, and grounded in which documentation.

### M2 — Scientific correctness and reproducibility

- Include documentation snapshots, skill hashes, tool versions, model identity, sampling parameters, and execution environment in exportable provenance.
- Center `oxo-bench` on a real, versioned reference corpus. Simulation may validate the benchmark harness but must not support scientific performance claims.
- Add functional-equivalence, invalid-flag, input/output-contract, idempotence, and side-effect tests in isolated environments. Exact string matching remains only one metric.
- Archive a rerunnable evaluation manifest, failure taxonomy, and known limitations for each formal release.

**Exit gate:** Every public performance claim is traceable to a version, dataset, model, run configuration, and raw results. Unvalidated model/tool combinations are presented as unknown, not reliable.

### M3 — Safe generation-to-execution boundary

- Build two pre-execution layers: deterministic syntax, path, overwrite, network, and high-risk-operation rules first; escalate only indeterminate cases to model review or human confirmation.
- Provide configurable but safe-by-default policies for allowed executables, working directories, network access, output overwrites, remote hosts, and batch fan-out.
- Treat input text, documentation, and model output as untrusted data; continuously test command injection, path traversal, secret exposure, and prompt injection.
- Apply least privilege, explicit targets, and audit records to local, remote, and HPC single-command execution without turning them into a DAG scheduler.

**Exit gate:** Every high-risk rule is explainable, unit-tested, and observable in JSON results. The default path cannot bypass confirmation or execution policy because of model output.

### M4 — Reliable 1.0 release

- Publish a support matrix for platforms, shells, providers, local models, stable CLI/API surfaces, and withdrawal/deprecation policy.
- Reproduce the release build, tests, documentation, and benchmark manifest in a clean environment; publish supply-chain provenance for release binaries.
- Establish durable archives and citable identifiers for releases, skills, and evaluation data, plus security response, compatibility, and maintainer-rotation policies.
- Prioritize failures reported by real laboratories, core facilities, and teaching users rather than merely adding more commands.

**Exit gate:** A release candidate has reproducible build evidence, no known unaddressed high-severity security findings, stable contract tests, a public limitations list, and accountable maintainers.

### M5 — Long-term maintenance, not feature expansion

- Classify every capability under command intelligence, provenance, safety, interoperability, or sustainability. Work that fits none of these does not enter the core by default.
- Regularly retire invalid providers, stale skills, and irreproducible benchmarks while recording the reason.
- Keep AI components replaceable while making history/provenance and the CLI protocol resilient across models, platforms, and organizations.
- Measure utility through public issues, reproduction packages, independent evaluation, and citation archives; do not substitute a “twenty-year guarantee” for evidence.

## 5. Near-term execution plan

1. Turn M1–M4 into issues with acceptance conditions; each issue links to the relevant CLI contract, schema, test, or safety policy.
2. Freeze and test `dry-run`/`run` machine-readable output before extending provenance. Define the user contract before designing a database.
3. Implement deterministic risk rules and regression corpora before researching an LLM judge. Any LLM judge must be disableable, recorded, and unable to exceed policy.
4. Build functional evaluation from real tool documentation and minimal isolated fixtures. Every new accuracy claim must include a complete reproducibility package.
5. Maintain reciprocal boundary guidance in oxo-flow and oxo-call documentation instead of duplicating one another's scheduling or command-generation logic.

## 6. Research and standards basis

- Wilkinson et al., *The FAIR Guiding Principles for scientific data management and stewardship* (2016), [DOI: 10.1038/sdata.2016.18](https://doi.org/10.1038/sdata.2016.18) — long-term findability, accessibility, interoperability, and reusability.
- Sandve et al., *Ten Simple Rules for Reproducible Computational Research* (2013), [DOI: 10.1371/journal.pcbi.1003285](https://doi.org/10.1371/journal.pcbi.1003285) — recording versions, processes, and provenance for each result.
- Wilson et al., *Good Enough Practices in Scientific Computing* (2017), [DOI: 10.1371/journal.pcbi.1005510](https://doi.org/10.1371/journal.pcbi.1005510) — practical, executable, maintainable foundations over unsustainable perfection.
- Martín del Pico, Gelpí & Capella-Gutierrez, *FAIRsoft* (2024), [DOI: 10.1093/bioinformatics/btae464](https://doi.org/10.1093/bioinformatics/btae464); [FAIR4RS](https://doi.org/10.15497/RDA00068) — assessable FAIR principles for research software.
- [W3C PROV](https://www.w3.org/TR/prov-dm/) and [RO-Crate](https://www.researchobject.org/ro-crate/) — exchangeable provenance foundations rather than an isolated log format.
- [NIST AI RMF 1.0](https://doi.org/10.6028/NIST.AI.100-1), the [NIST Generative AI Profile](https://doi.org/10.6028/NIST.AI.600-1), and [OWASP LLM06: Excessive Agency](https://genai.owasp.org/llmrisk/llm062025-excessive-agency/) — human control, risk measurement, and least privilege.
- [SLSA v1.2](https://slsa.dev/spec/v1.2/) and [Software Citation Principles](https://doi.org/10.7717/peerj-cs.86) — verifiable releases and durable citation.
- [CLI Guidelines](https://clig.dev/), [Semantic Versioning](https://semver.org/spec/v2.0.0.html), and [Traitome/oxo-flow](https://github.com/Traitome/oxo-flow) — composable CLI contracts, compatibility, and a clear DAG responsibility boundary.

## 7. Definition of success

oxo-call is worth using over the long term not because it claims to replace every piece of analysis infrastructure, but because years later a researcher can still answer: **what was invoked, why it was invoked, what grounded it, how risks were handled, and how it can be replayed or composed elsewhere.**
