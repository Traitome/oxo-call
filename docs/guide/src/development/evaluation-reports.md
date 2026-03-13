# Expert Evaluation Reports

This document presents a multi-perspective evaluation of the oxo-call project from 12 expert roles spanning research methodology, scientific significance, and software engineering. Each evaluation identifies strengths, concerns, and actionable recommendations.

---

## Evaluation Methodology

Twelve independent expert roles were designed to cover three evaluation dimensions:

| Dimension | Roles |
|-----------|-------|
| **Research & Methodology** | Computational Biologist, Biostatistician, Reproducibility Scientist, Clinical Bioinformatician |
| **Scientific Significance** | Journal Reviewer, Funding Agency Evaluator, Scientific Impact Analyst, Domain Expert (Omics) |
| **Software Engineering** | Systems Architect, Security Engineer, DevOps/CI Engineer, Open-Source Community Manager |

---

## Report 1: Computational Biologist

**Role**: Senior researcher using 20+ CLI tools daily for NGS analysis.

### Strengths
- The docs-first grounding approach is scientifically sound — it prevents LLM hallucination of flags that don't exist
- 150+ built-in skills cover the vast majority of tools in a typical bioinformatics workflow
- The skill system's concept/pitfall/example structure mirrors how domain experts actually reason about tool usage
- Natural language task descriptions lower the barrier for researchers who are not command-line experts

### Concerns
- **Reproducibility**: Generated commands depend on LLM state — the same prompt may produce different commands across runs, even with temperature=0.0
- **Version sensitivity**: Bioinformatics tools change flags between versions; cached `--help` output may become stale
- **Validation gap**: No built-in verification that generated commands are semantically correct (e.g., checking that reference genome matches the organism)

### Recommendations
1. Add command fingerprinting: hash the generated ARGS with the tool version and documentation hash to enable reproducibility audits
2. Implement `docs update --auto-version` to detect tool version changes and refresh cache automatically
3. Add a `--validate` flag that performs basic semantic checks (e.g., file existence, format compatibility)
4. Include tool version in history entries for provenance tracking

---

## Report 2: Biostatistician

**Role**: Statistician focusing on experimental design and analytical reproducibility.

### Strengths
- JSONL history with exit codes provides an audit trail for computational experiments
- Dry-run mode enables verification before execution — critical for statistical pipelines
- Deterministic LLM settings (temperature=0.0) are the correct default for scientific computing

### Concerns
- **Statistical validation**: No mechanism to verify that parameter choices are statistically appropriate (e.g., multiple testing correction, normalization methods)
- **Batch effect awareness**: Skills don't encode experimental design considerations
- **Provenance chain**: History records commands but not the input data characteristics that influenced parameter choices

### Recommendations
1. Extend history entries with input file metadata (size, format, sample count) for provenance
2. Add statistical context to skills for quantitative tools (e.g., DESeq2 normalization guidance)
3. Consider adding a `--provenance` flag that generates a complete reproducibility manifest (tool version + docs hash + skill version + LLM model + parameters)

---

## Report 3: Reproducibility Scientist

**Role**: Researcher specializing in computational reproducibility and FAIR principles.

### Strengths
- Offline license verification (no network dependency for core functionality)
- Local documentation caching enables air-gapped operation
- TOML-based configuration is human-readable and version-controllable
- Workflow engine with `.oxo.toml` format is declarative and reproducible

### Concerns
- **LLM non-determinism**: Even with temperature=0.0, different LLM providers/versions may produce different outputs for the same prompt
- **Missing containerization**: No built-in support for Docker/Singularity to ensure environment reproducibility
- **Skill versioning**: No version tracking for built-in skills — changes between releases may silently alter behavior
- **Missing checksums**: No SHA256 checksums for release binaries

### Recommendations
1. Add a `--seed` parameter for LLM calls to improve reproducibility across sessions
2. Include container image references in workflow templates (Docker/Singularity URIs)
3. Version built-in skills (semantic versioning in `[meta]` section) and log skill version in history
4. Publish SHA256 checksums alongside release binaries
5. Add a `reproduce` command that replays a history entry with the same prompt/docs/skill context

---

## Report 4: Clinical Bioinformatician

**Role**: Bioinformatician in a clinical genomics laboratory with regulatory requirements.

### Strengths
- License system provides organizational accountability
- Offline operation is critical for clinical environments with network restrictions
- Ed25519 signature verification is cryptographically robust

### Concerns
- **Audit compliance**: Clinical genomics requires complete audit trails with user identity, timestamp, and input/output provenance
- **Deterministic output**: Regulatory submissions require bit-identical reproducibility
- **Access control**: No role-based access control or multi-user support
- **Validation status**: Tools should be distinguished by validation status (research-only vs. clinically validated)

### Recommendations
1. Add user identity to history entries (configurable, default to system username)
2. Implement `--strict` mode that rejects commands if the LLM response changes between retries
3. Add clinical vs. research classification to skill metadata
4. Support LIMS integration via structured output formats (JSON, TSV)

---

## Report 5: Journal Reviewer (Bioinformatics Methods)

**Role**: Peer reviewer for a computational biology methods journal.

### Strengths
- **Novel contribution**: The combination of docs-first grounding + skill-augmented prompting is a genuine methodological advance over naive LLM wrappers
- **Comprehensive scope**: 150+ tools across all major omics domains demonstrates broad applicability
- **Practical validation**: The architecture diagram and execution flow are clearly documented
- **Dual licensing**: Academic-free model removes adoption barriers for the research community

### Concerns
- **Benchmark rigor**: Need systematic comparison against manual command construction (accuracy, time savings, error rates)
- **Ablation studies**: What is the independent contribution of documentation grounding vs. skill injection?
- **Baseline comparisons**: How does oxo-call compare to ChatGPT/Claude direct prompting, BioContainers, Galaxy GUI?
- **User study**: No empirical evidence of user experience improvements

### Recommendations
1. Design a formal benchmark with 100+ tasks across 20+ tools, comparing: (a) LLM-only, (b) LLM+docs, (c) LLM+docs+skills, (d) expert manual construction
2. Conduct ablation studies removing each component to measure independent contribution
3. Perform a user study with 20+ bioinformaticians at different skill levels
4. Add quantitative metrics: flag accuracy (%), argument order correctness, semantic validity, time-to-command
5. Compare against existing approaches (direct LLM prompting, Galaxy, BioContainers)

---

## Report 6: Funding Agency Evaluator

**Role**: Grant reviewer assessing scientific impact and sustainability.

### Strengths
- **Clear problem statement**: CLI complexity is a well-recognized barrier in bioinformatics
- **Broad impact potential**: Affects every bioinformatician who uses command-line tools
- **Sustainability model**: Dual licensing (academic-free/commercial) provides a viable funding path
- **Open-source core**: Source code availability enables community contribution and verification

### Concerns
- **Dependency risk**: Heavy reliance on external LLM providers (API availability, cost, privacy)
- **Long-term maintenance**: 150+ skills require ongoing curation as tools evolve
- **Community adoption**: No community governance model or contribution guidelines beyond basic PR workflow
- **Data privacy**: User tasks are sent to external LLM APIs — may contain sensitive data (patient identifiers, proprietary sequences)

### Recommendations
1. Emphasize Ollama (local LLM) for privacy-sensitive environments
2. Establish a community skill contribution program with automated validation
3. Create a sustainability plan documenting maintenance commitments
4. Add data anonymization/scrubbing before LLM submission for sensitive contexts
5. Develop a community governance model (RFC process, roadmap, release schedule)

---

## Report 7: Scientific Impact Analyst

**Role**: Bibliometrics and research impact assessment specialist.

### Strengths
- **Addresses a universal pain point**: Every bioinformatician struggles with CLI complexity
- **Citable methodology**: The docs-first + skill-augmented approach is publishable as a methods paper
- **Cross-domain applicability**: Architecture could be adapted beyond bioinformatics (DevOps, scientific computing, system administration)
- **Quantifiable impact**: Time savings and error reduction are directly measurable

### Concerns
- **Citation strategy**: Need a clear publication venue and citation mechanism (JOSS, Bioinformatics, Nature Methods)
- **Benchmark dataset**: No public benchmark dataset for others to reproduce and cite
- **Comparison baseline**: No systematic comparison with existing approaches

### Recommendations
1. Publish in a high-impact venue (Nature Methods for the methodology, Bioinformatics for the tool)
2. Create a public benchmark dataset (`oxo-bench-tasks.json`) for reproducible evaluation
3. Assign a DOI via Zenodo for each release
4. Write a companion protocol paper for Nature Protocols or Current Protocols
5. Track adoption metrics (downloads, GitHub stars, citations)

---

## Report 8: Domain Expert (Multi-Omics)

**Role**: Senior scientist working across genomics, transcriptomics, epigenomics, and metagenomics.

### Strengths
- **Comprehensive tool coverage**: Skills span all major omics domains
- **Workflow templates**: Built-in pipelines for RNA-seq, WGS, ATAC-seq, metagenomics, etc.
- **Cross-domain consistency**: Same interface for all tools regardless of domain
- **Skill quality**: Built-in skills encode real domain expertise (not just flag descriptions)

### Concerns
- **Spatial omics gap**: No skills for spatial transcriptomics tools (e.g., Squidpy, Giotto, MERFISH)
- **Proteomics gap**: No skills for mass spectrometry tools (e.g., MaxQuant, MSFragger, DIA-NN)
- **Multi-omics integration**: No skills for integrative analysis tools (e.g., MOFA+, Seurat v5 multimodal)
- **Skill depth**: Some skills are shallow (few examples) compared to samtools/bcftools

### Recommendations
1. Add skills for spatial omics, proteomics, and multi-omics integration tools
2. Standardize skill depth: minimum 5 examples, 3 concepts, 3 pitfalls per skill
3. Add skill quality metrics and display coverage in documentation
4. Create domain-specific tutorial workflows (spatial transcriptomics, proteomics)

---

## Report 9: Systems Architect

**Role**: Senior software architect evaluating system design and scalability.

### Strengths
- **Clean module separation**: 13 modules with clear responsibilities and minimal coupling
- **Layered architecture**: docs → skills → LLM → execution is a well-designed pipeline
- **Platform abstraction**: Cross-platform support via `directories::ProjectDirs` and conditional WASM compilation
- **Workflow engine**: Native DAG execution with `tokio` parallelism is well-engineered
- **Strict LLM contract**: ARGS:/EXPLANATION: format with retry prevents malformed output

### Concerns
- **`main.rs` complexity**: 1089 lines in `main.rs` — too much logic for a command dispatcher
- **Error handling inconsistency**: Mix of `anyhow` and custom `thiserror` types
- **No plugin architecture**: Adding new LLM providers or documentation sources requires code changes
- **No API/library mode**: oxo-call is CLI-only; no programmatic API for integration

### Recommendations
1. Extract command handlers from `main.rs` into separate handler modules (reduce to ~200 lines)
2. Standardize error handling: use `thiserror` for domain errors, `anyhow` only in main
3. Design a plugin trait for LLM providers and documentation sources
4. Consider adding a `lib.rs` with public API for embedding in other tools
5. Add structured logging (tracing crate) for debugging and performance analysis

---

## Report 10: Security Engineer

**Role**: Application security specialist reviewing the threat model.

### Strengths
- **Ed25519 license verification**: Cryptographically sound, offline, tamper-resistant
- **Input validation**: Tool names sanitized against path traversal, URLs restricted to HTTP/HTTPS
- **No credential storage in code**: API tokens in config file or environment variables only
- **Offline license model**: No network dependency for license verification

### Concerns
- **Command injection**: Generated commands are executed via shell — potential for injection if LLM output is malicious
- **API token exposure**: Tokens in config files may be readable by other users on shared systems
- **LLM prompt injection**: Malicious documentation content could manipulate LLM behavior
- **Supply chain**: Dependencies (ed25519-dalek, reqwest, tokio) need regular security audits
- **Missing SBOM**: No Software Bill of Materials for supply chain transparency

### Recommendations
1. Implement command sanitization: validate generated args against the tool's known flag set
2. Add file permission checks for config files (warn if group/other readable)
3. Implement documentation content sanitization (strip potential prompt injection patterns)
4. Generate SBOM (CycloneDX or SPDX format) in CI pipeline
5. Add `cargo audit` to CI for dependency vulnerability scanning
6. Consider sandboxed execution (namespace, seccomp) for generated commands

---

## Report 11: DevOps / CI Engineer

**Role**: Build and deployment specialist evaluating the CI/CD pipeline.

### Strengths
- **Multi-platform builds**: Linux (x86_64/aarch64, glibc/musl), macOS (Intel/Apple Silicon), Windows, WASM
- **Automated releases**: Tag-triggered builds with GitHub Release artifact upload
- **crates.io publishing**: Automated version verification and publish
- **GitHub Pages deployment**: Landing page auto-deployed on push to main

### Concerns
- **Missing security scanning**: No `cargo audit`, no SAST/DAST in CI
- **No release checksums**: Binary releases lack SHA256 checksums
- **No integration tests in CI**: Only unit tests run; no end-to-end tests with real LLM calls
- **No documentation build in CI**: mdBook not built/deployed automatically
- **Missing changelog**: No automated changelog generation from commits/PRs
- **No code coverage**: No coverage reporting or minimum threshold

### Recommendations
1. Add `cargo audit` step to CI pipeline
2. Generate and publish SHA256 checksums with each release
3. Add mdBook build and deploy step to the GitHub Pages workflow
4. Add code coverage reporting (tarpaulin or llvm-cov)
5. Implement automated changelog generation (git-cliff or similar)
6. Add smoke tests that verify binary startup without LLM calls

---

## Report 12: Open-Source Community Manager

**Role**: Community builder evaluating project governance and contributor experience.

### Strengths
- **Clear README**: Comprehensive with architecture diagram, quick start, and command reference
- **Dual licensing**: Academic-free model encourages adoption
- **Built-in skills**: Community can contribute skills without touching Rust code
- **Integration tests**: Clear test patterns for contributors to follow

### Concerns
- **Missing CONTRIBUTING.md**: No top-level contribution guide
- **No issue templates**: No structured issue/bug report templates
- **No code of conduct**: Missing community standards
- **No roadmap**: No public roadmap or RFC process
- **Limited documentation**: No comprehensive docs site (addressed by this PR)
- **No community registry**: Skill sharing requires manual file exchange

### Recommendations
1. Add CONTRIBUTING.md with development setup, PR guidelines, and skill contribution guide
2. Create GitHub issue templates (bug report, feature request, skill request)
3. Add CODE_OF_CONDUCT.md
4. Publish a public roadmap (GitHub Projects or docs page)
5. Create a community skill registry (GitHub-based or standalone)
6. Add a CITATION.cff file for academic citation

---

## Consolidated Action Items

The following prioritized action list synthesizes recommendations across all 12 evaluation reports:

### Priority 1 — Critical for Publication

| # | Action | Source Reports |
|---|--------|---------------|
| 1 | Design formal benchmark (100+ tasks, 20+ tools, accuracy metrics) | 5, 7, 8 |
| 2 | Conduct ablation study (docs-only vs. docs+skills vs. full pipeline) | 5 |
| 3 | Add command provenance (tool version + docs hash + skill version + model) | 1, 2, 3 |
| 4 | Create public benchmark dataset for reproducible evaluation | 5, 7 |
| 5 | Add CITATION.cff for academic citation | 7, 12 |

### Priority 2 — Important for Quality & Security

| # | Action | Source Reports |
|---|--------|---------------|
| 6 | Add `cargo audit` to CI pipeline | 10, 11 |
| 7 | Generate SHA256 checksums for release binaries | 3, 11 |
| 8 | Add command sanitization layer | 10 |
| 9 | Add mdBook documentation build/deploy to CI | 11 |
| 10 | Add code coverage reporting | 11 |
| 11 | Implement tool version tracking in history | 1, 4 |

### Priority 3 — Enhances User Experience

| # | Action | Source Reports |
|---|--------|---------------|
| 12 | Add CONTRIBUTING.md | 12 |
| 13 | Create GitHub issue templates | 12 |
| 14 | Extend skill coverage (spatial omics, proteomics, multi-omics) | 8 |
| 15 | Standardize minimum skill depth (5 examples, 3 concepts, 3 pitfalls) | 8 |
| 16 | Refactor main.rs (extract command handlers) | 9 |

### Priority 4 — Future Enhancements

| # | Action | Source Reports |
|---|--------|---------------|
| 17 | Add plugin trait for LLM providers | 9 |
| 18 | Add lib.rs for programmatic API | 9 |
| 19 | Community skill registry | 6, 12 |
| 20 | Container image references in workflows | 3 |
| 21 | Data anonymization for sensitive LLM contexts | 6 |
| 22 | Structured logging with tracing crate | 9 |

---

## Running Evaluations

The `oxo-bench` crate provides automated evaluation capabilities:

```bash
# Run the full benchmark suite
cargo run -p oxo-bench -- evaluate

# Run for a specific tool
cargo run -p oxo-bench -- evaluate --tool samtools

# Export benchmark data
cargo run -p oxo-bench -- export-csv --output docs/
```

Benchmark results are stored in CSV files under `docs/`:
- `bench_workflow.csv` — Workflow execution metrics
- `bench_scenarios.csv` — Scenario configurations
- `bench_eval_tasks.csv` — Evaluation task results

---

## Documentation Review: Multi-Role Perspectives

The following section presents a structured review of this documentation guide from the perspective of four key user roles. Each reviewer read through the guide as a new user and provided feedback on usability, completeness, and clarity.

---

### Documentation Reviewer 1: New PhD Student

**Role**: First-year graduate student, bioinformatics. Has basic Linux skills; knows what FASTQ, BAM, and RNA-seq are; never used oxo-call before.

#### Positive Findings

- The **Introduction** is clear about what oxo-call is and why it is useful. The architecture diagram with plain-language labels helps.
- **Your First Command** tutorial is the right entry point — the 5-step structure with expected output examples is very helpful. The "what happened behind the scenes" callout boxes explain the *why*, not just the *how*.
- The dry-run → run → ask pattern is simple enough to learn in one session.
- The "What You Learned" summary at the end of each tutorial helps consolidate knowledge.

#### Gaps Found

- The **License Setup** page does not explain what happens if the license is wrong — what error do I see? Add an error example and how to fix it.
- **Configuration** mentions `oxo-call config verify` but does not show a failed verification example. What does a failed LLM connection look like?
- The RNA-seq tutorial assumes the user has a STAR genome index. Add a note about where to download pre-built indices (e.g., ENCODE, GENCODE).
- The Ollama section in the how-to guide assumes Ollama is already installed. Add the install command explicitly.

#### Recommendations

1. Add a "Troubleshooting" section to the Getting Started pages with common first-run errors
2. Add download links for test data (e.g., a small BAM file to follow the tutorials)
3. Add an "Expected output" block to every `oxo-call run` example, even if approximate

---

### Documentation Reviewer 2: Experienced Bioinformatician

**Role**: Staff scientist at a genomics core, 7 years of experience, runs pipelines for 20+ PIs. Uses Snakemake daily. Evaluating oxo-call for adoption across the core.

#### Positive Findings

- The **BAM workflow tutorial** covers exactly the operations we perform daily (sort → index → filter → stat). The `-F 0x904` explanation is correct and thorough.
- The **Workflow Builder tutorial** correctly explains `gather = true` for MultiQC — this is a non-obvious but critical concept.
- The pipeline design checklist in the how-to guide is production-quality.
- HPC export (Snakemake + Nextflow) is documented and the step-by-step is complete.

#### Gaps Found

- The **Workflow Engine reference** should document whether `depends_on` supports inter-phase dependencies (e.g., can a gather step depend on another gather step?).
- The RNA-seq tutorial should mention STAR two-pass mode — it is the standard for novel splice junction discovery. Currently the alignment step uses basic one-pass.
- The how-to guide for custom skills does not mention the minimum skill requirements (5 examples, 3 concepts, 3 pitfalls). This is validated by the engine — users need to know.
- There is no documentation on how to run oxo-call in a SLURM job script environment where `GITHUB_TOKEN` may not be set.

#### Recommendations

1. Add a "CI/cluster considerations" section to the configuration page
2. Add STAR two-pass mode as a note in the RNA-seq tutorial
3. Explicitly document skill depth requirements in the custom skill how-to
4. Add a workflow troubleshooting table to the workflow builder tutorial (already done — this is good)

---

### Documentation Reviewer 3: Computational Biologist / Methods Developer

**Role**: Postdoc developing new analysis methods. Writes Rust and Python. Wants to extend oxo-call with custom skills and possibly contribute to the codebase.

#### Positive Findings

- The skill TOML format is well-documented with a complete working example (kallisto). The good/bad examples in the "Writing Good Skills" section are exactly the right teaching pattern.
- The `skill create` → `skill show` → test flow is clear.
- The contributing guide in Development explains how to add built-in skills to the Rust binary.
- The architecture module graph in the reference section gives enough context to navigate the codebase.

#### Gaps Found

- The skill how-to mentions "minimum requirements" (5 examples, 3 concepts) but the validation error messages are not shown. What does the LLM prompt injection look like when a skill is too thin?
- The **LLM Integration reference** should document the exact prompt format sent to the LLM — this is important for debugging and for evaluating skill effectiveness.
- There is no guidance on testing skills programmatically with `oxo-bench`. The bench crate is mentioned at the end of the evaluation reports but not linked from the contributing guide.
- The `sanitize.rs` module (path/token redaction) is mentioned in architecture but not explained. Users handling sensitive data need to know how this works.

#### Recommendations

1. Add a "Debugging skills" section to the custom skill how-to: how to see what the LLM actually received
2. Link `oxo-bench` from the contributing guide with usage examples
3. Add a note in the configuration guide about `sanitize.rs` and what data is anonymized before LLM calls
4. Show the raw prompt format in the LLM Integration reference

---

### Documentation Reviewer 4: Bioinformatics Core Manager

**Role**: Manages a team of 8 bioinformaticians, responsible for adopting and standardizing tools across the organization. Focuses on onboarding experience, cost, licensing, and institutional concerns.

#### Positive Findings

- The **License page** is clear about free vs. commercial use. The offline verification model is a major plus for air-gapped or data-sovereignty-constrained environments.
- The Ollama section in the how-to addresses our primary concern about data privacy for patient data.
- The history with provenance metadata (tool version, docs hash, model) directly addresses our reproducibility requirements.
- The How-to Guides section is well-organized for the types of questions we receive from new team members.

#### Gaps Found

- There is no documentation on **team-wide configuration** — how do we share a common `config.toml` or skills directory across a team? Environment variables are mentioned but the multi-user scenario is not addressed.
- The **Commercial license** section says "USD 200" but should clarify: one license covers all users in the organization? (This is stated in the README but not in the documentation guide.)
- There is no discussion of **audit and compliance** — what data does oxo-call send to the LLM API? How is patient data handled? The sanitize module should be explicitly documented.
- No mention of how to air-gap the tool completely — can oxo-call run with local documentation and Ollama, with no external network calls at all?

#### Recommendations

1. Add a "Team Setup" or "Organizational Deployment" how-to guide
2. Add an "Air-gapped / Offline Mode" section to the configuration page
3. Document explicitly what data is sent to the LLM API (and what is NOT sent — e.g., actual file contents)
4. Clarify commercial license scope (one license = whole organization) in the documentation guide
5. Add a security considerations page to the architecture reference section

---

### Documentation Iteration Summary

Based on the four-role review above, the following issues are prioritized for the next iteration:

| Priority | Issue | Reviewer(s) |
|----------|-------|-------------|
| 🔴 High | Add troubleshooting examples with error messages for first-run failures | Student |
| 🔴 High | Document what data is sent to LLM API (privacy/compliance) | Core Manager |
| 🟡 Medium | Add team/organizational deployment how-to | Core Manager |
| 🟡 Medium | Add air-gapped / offline mode documentation | Core Manager |
| 🟡 Medium | Add test data download links to tutorials | Student |
| 🟡 Medium | Document skill depth requirements explicitly in how-to | Experienced Bio |
| 🟢 Low | STAR two-pass mode note in RNA-seq tutorial | Experienced Bio |
| 🟢 Low | Show raw LLM prompt format in reference | Methods Developer |
| 🟢 Low | Link oxo-bench from contributing guide | Methods Developer |
