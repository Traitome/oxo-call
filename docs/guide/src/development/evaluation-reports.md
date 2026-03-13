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

### Resolution Status

✅ `CommandProvenance` struct added to `src/history.rs` with `tool_version`, `docs_hash` (SHA-256 of combined documentation), `skill_name`, and `model` fields — attached to every `HistoryEntry` in JSONL history. This directly addresses recommendation 1.

✅ `detect_tool_version()` implemented in `src/runner.rs` — runs `<tool> --version` and records the output in provenance, resolving recommendation 4.

⏳ **Deferred: `docs update --auto-version` (recommendation 2).** Automatic detection of tool version changes requires reliable version-string parsing for 150+ tools with inconsistent `--version` output formats (some print to stderr, some embed version in help text, some require subcommands). The current approach — `docs add <tool>` to re-index manually — is simple and predictable. Users can re-run `docs add` after upgrading a tool. Automatic version monitoring may be added in a future release if a reliable cross-tool version detection heuristic is found.

⏳ **Deferred: `--validate` flag (recommendation 3).** Generic semantic validation of generated commands (file existence, format compatibility, reference genome matching) is not feasible as a single feature across 150+ tools with fundamentally different argument semantics. For example, validating a `samtools sort` command requires checking BAM file existence, while validating a `STAR --genomeGenerate` command requires verifying genome FASTA and GTF paths. The `--ask` flag provides human-in-the-loop verification, and `dry-run` mode allows previewing commands before execution — these serve as practical alternatives to automated validation.

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

### Resolution Status

✅ `CommandProvenance` struct in `src/history.rs` captures tool version, docs hash (SHA-256), skill name, and LLM model — partially addressing recommendation 3. Provenance is automatically recorded with every history entry rather than requiring a separate `--provenance` flag.

⏳ **Deferred: Input file metadata (recommendation 1).** Recording file size, format, and sample count in history entries would require parsing tool-specific argument semantics to identify which arguments are input files (vs. output files, reference files, or parameter values). Each tool has a different argument convention — positional arguments, `-i` flags, `--input` flags, etc. The existing `CommandProvenance` already captures tool version, docs hash, skill name, and LLM model, which provides sufficient provenance for reproducibility audits. Input file metadata can be derived from the recorded command and the filesystem state at execution time.

⏳ **Deferred: Statistical context in skills (recommendation 2).** Adding statistical guidance (normalization method selection, multiple testing correction, batch effect handling) to quantitative tool skills requires domain-expert curation for each statistical tool. This is an ongoing effort — skills like DESeq2, edgeR, and limma could benefit from statistical context, but the guidance must be accurate and up-to-date with evolving statistical best practices. Community contributions of statistical context to existing skills are welcome via pull requests.

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

### Resolution Status

✅ Container image references (recommendation 2) are now included in workflow templates — Docker/BioContainers URIs are present in Snakemake (`.smk`) and Nextflow (`.nf`) export files.

✅ SHA256 checksums (recommendation 4) — `SHA256SUMS.txt` is generated alongside release binaries in the CI pipeline and published with each GitHub Release.

✅ Environment reproducibility (recommendation 2, extended) — the `env` field on workflow steps allows per-step conda environment, virtualenv, or PATH overrides, enabling reproducible runtime isolation without full containerization. This addresses the "Missing containerization" concern for lightweight deployments where Docker/Singularity is not available.

⏳ **Deferred: `--seed` parameter (recommendation 1).** LLM seed support is provider-dependent and does not guarantee determinism. OpenAI supports a `seed` parameter but explicitly states outputs are not guaranteed to be identical. Anthropic does not support seeds. Ollama's seed behavior varies by model backend. Since oxo-call already uses `temperature=0.0` (the strongest determinism guarantee available across all providers), adding `--seed` would provide marginal benefit with significant implementation complexity. The existing `CommandProvenance` (docs hash + skill name + model) provides a better reproducibility mechanism than relying on non-deterministic LLM seeds.

⏳ **Deferred: Skill versioning (recommendation 3).** Adding semantic versioning to all 150+ built-in skill `[meta]` sections would require a large migration and an ongoing versioning policy. The existing `CommandProvenance` captures the `skill_name` and `docs_hash` per execution, which together with the git commit of the oxo-call release provides sufficient traceability. Skill changes are tracked via git history, and the skill depth validation (`MIN_EXAMPLES=5`, `MIN_CONCEPTS=3`, `MIN_PITFALLS=3`) ensures quality consistency across releases.

⏳ **Deferred: `reproduce` command (recommendation 5).** Faithful command replay requires reconstructing the exact documentation cache, skill content, and LLM state from the original execution — a complex dependency-resolution problem. The existing `CommandProvenance` records enough metadata (tool version, docs hash, skill name, model) for manual reproducibility audits. Users can re-run the same task description with `dry-run` to compare outputs. A full `reproduce` command would need to snapshot and restore documentation caches, which adds significant storage and complexity without proportional benefit.

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

### Resolution Status

✅ Offline operation and Ed25519 license verification (noted as strengths) remain fully functional, supporting clinical environments with network restrictions.

⏳ **Deferred: User identity in history (recommendation 1).** Recording user identity by default raises privacy concerns for shared systems and multi-tenant environments. On shared HPC clusters, the system username may leak information about who ran which analyses. A privacy-preserving approach (opt-in user identity, hashed identifiers, or configurable anonymization) would be needed before enabling this feature. Users who need audit trails with user identity can wrap oxo-call invocations with their own logging that captures `$USER`.

⏳ **Deferred: `--strict` mode (recommendation 2).** Rejecting commands when the LLM response changes between retries requires multi-call comparison logic — calling the LLM multiple times per command request, comparing outputs, and rejecting on divergence. This would increase API costs and latency significantly. The existing `temperature=0.0` setting provides the strongest available determinism guarantee. For regulatory submissions, users should use `dry-run` to preview and record the exact command before execution.

⏳ **Deferred: Clinical vs. research classification (recommendation 3).** Classifying tools by clinical validation status (research-only, clinically validated, FDA-cleared) requires domain-expert curation and ongoing maintenance as tools receive new regulatory approvals. This classification is also jurisdiction-dependent (FDA vs. EMA vs. NMPA). The skill system's `category` and `tags` fields could carry this metadata in the future, but the classification criteria and maintenance policy need to be defined first.

⏳ **Deferred: LIMS integration (recommendation 4).** Structured output beyond JSONL history (e.g., JSON, TSV, HL7 FHIR) would require defining a schema for each output format and understanding the specific LIMS system's import requirements. The existing JSONL history is machine-readable and can be transformed to other formats with standard tools (`jq`, `python`). A plugin or export system for LIMS integration may be considered for a future release when concrete integration requirements are identified.

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

### Resolution Status

✅ The `oxo-bench` crate provides a formal benchmark suite with 50 canonical evaluation tasks across 15 categories (alignment, variant-calling, SAM/BAM, quantification, QC, metagenomics, epigenomics, etc.), addressing recommendation 1.

✅ Ablation studies (recommendation 2) are supported — `oxo-bench` includes 7 ablation tasks at easy/medium/hard difficulty levels to measure independent component contributions.

✅ Quantitative metrics (recommendation 4) — benchmark results are exported as CSV files (`bench_workflow.csv`, `bench_scenarios.csv`, `bench_eval_tasks.csv`) for analysis.

📋 **Out of scope: User study (recommendation 3).** Conducting a user study with 20+ bioinformaticians requires human participants, IRB/ethics approval, structured experimental design, and academic collaboration. This is an external academic activity that falls outside the scope of the software project itself. The `oxo-bench` benchmark suite provides automated evaluation of command generation accuracy, which can supplement future user studies.

📋 **Out of scope: Head-to-head comparison (recommendation 5).** Systematically benchmarking oxo-call against Galaxy, BioContainers, and direct LLM prompting requires access to these platforms and a standardized evaluation methodology. The `oxo-bench` benchmark infrastructure is in place and can be extended with comparative baselines when such an evaluation is conducted. Direct LLM prompting (ChatGPT/Claude without documentation grounding) can be informally compared using the same task descriptions in `oxo-bench`.

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

### Resolution Status

✅ Ollama local LLM support (recommendation 1) is fully implemented, enabling completely offline operation for privacy-sensitive environments with no external API calls.

✅ Community skill contributions (recommendation 2) are partially addressed — `CONTRIBUTING.md` includes a detailed skill authoring guide with PR guidelines, and `validate_skill_depth()` in `src/skill.rs` enforces minimum quality standards (5 examples, 3 concepts, 3 pitfalls).

✅ Data anonymization (recommendation 4) — `src/sanitize.rs` implements `redact_paths()` (strips absolute file paths) and `redact_env_tokens()` (redacts TOKEN=, KEY=, SECRET= values) for sensitive contexts.

📋 **Out of scope: Sustainability plan (recommendation 3).** A formal sustainability plan documenting long-term maintenance commitments is an organizational decision that depends on funding, team size, and institutional support. The dual-license model (academic-free / commercial-paid) provides a sustainable funding path. The open-source codebase ensures community continuity regardless of any single maintainer's availability.

📋 **Out of scope: Community governance model (recommendation 5).** Formal governance structures (RFC process, public roadmap, release schedule) are typically established as a project's community grows. The current project uses standard GitHub features (issues, pull requests, discussions) for community interaction. `CONTRIBUTING.md` provides contribution guidelines, and issue templates structure bug reports, feature requests, and skill requests. Formal governance may be introduced as the contributor community expands.

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

### Resolution Status

✅ Public benchmark dataset (recommendation 2) — the `oxo-bench` crate contains 50 canonical evaluation tasks with CSV exports (`bench_eval_tasks.csv`, `bench_scenarios.csv`, `bench_workflow.csv`) published under `docs/`.

✅ `CITATION.cff` (supporting citation strategy, recommendation 1) exists at the repository root with CFF v1.2.0 metadata, enabling proper academic citation.

📋 **Out of scope: Publication and protocol paper (recommendations 1, 4).** Publishing in a high-impact venue (Nature Methods, Bioinformatics) and writing a companion protocol paper (Nature Protocols) are external academic activities that depend on research timeline, co-author availability, and journal review cycles. The `CITATION.cff` at the repository root enables proper academic citation in the meantime, and the comprehensive documentation and benchmark suite provide the methodological basis for a future publication.

📋 **Out of scope: DOI via Zenodo (recommendation 3).** Zenodo DOI assignment requires configuring the Zenodo-GitHub integration and creating a release. This is a straightforward one-time setup that will be performed when the project reaches a stable release milestone suitable for archival citation. The `CITATION.cff` already provides citation metadata.

📋 **Out of scope: Adoption metric tracking (recommendation 5).** Tracking downloads, citations, and usage beyond GitHub's built-in statistics (stars, forks, traffic) would require integrating external analytics services. GitHub provides download counts for releases, and crates.io tracks download statistics for published crates. These built-in mechanisms are sufficient for the current project scale. A formal tracking system may be added when the project reaches broader adoption.

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

### Resolution Status

✅ Skill depth validation (recommendation 2) is enforced — `validate_skill_depth()` in `src/skill.rs` checks `MIN_EXAMPLES=5`, `MIN_CONCEPTS=3`, `MIN_PITFALLS=3` constants for all built-in skills.

✅ Skill coverage (recommendation 3) has grown to 119 built-in skills spanning alignment, variant calling, QC, RNA-seq, epigenomics, metagenomics, single-cell, and more.

⏳ **Planned: Spatial omics, proteomics, and multi-omics integration skills (recommendation 1).** Tools like Squidpy, Giotto, MaxQuant, MSFragger, DIA-NN, MOFA+, and Seurat v5 multimodal are not yet covered by built-in skills. Adding these requires domain expertise in spatial transcriptomics, mass spectrometry, and multi-omics integration — each tool has unique argument patterns and domain conventions. Community contributions of skills for these tools are welcome via pull requests. The skill authoring guide in [Create a Custom Skill](../how-to/create-custom-skill.md) provides the template and quality requirements.

⏳ **Planned: Domain-specific tutorial workflows (recommendation 4).** Tutorial workflows for spatial transcriptomics and proteomics pipelines require both the built-in skills (see above) and real-world pipeline designs validated by domain experts. These will be added as the corresponding tool skills are developed.

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

### Resolution Status

✅ Handler extraction (recommendation 1) — `src/handlers.rs` extracts formatting and display helpers (`with_source`, `print_index_table`, `config_verify_suggestions`) from `main.rs`.

✅ `lib.rs` programmatic API (recommendation 4) — `src/lib.rs` re-exports 13 modules (config, docs, engine, error, handlers, history, index, license, llm, runner, sanitize, skill, workflow) for programmatic embedding.

⏳ **Deferred: Error handling standardization (recommendation 2).** The codebase uses a mix of `anyhow` (for application-level error propagation) and custom `thiserror` types (for domain-specific errors in `src/error.rs`). This is a common and pragmatic pattern in Rust applications — `anyhow` for the CLI entry points, `thiserror` for library boundaries. A full migration to a single error strategy would require touching most modules and may not provide significant user-facing benefit. The current approach works correctly and the `lib.rs` API surface uses typed errors where appropriate.

✅ **Done: Plugin trait for LLM providers (recommendation 3).** The `LlmProvider` trait has been implemented in `src/llm.rs` with `chat_completion()` and `name()` methods. The built-in `OpenAiCompatibleProvider` covers OpenAI, GitHub Copilot, Anthropic, and Ollama. Custom implementations can override it for providers with different API shapes. Adding a new provider now requires implementing this trait rather than modifying the core `LlmClient` logic.

⏳ **Planned: Structured logging with `tracing` crate (recommendation 5).** The codebase currently uses `eprintln!` for diagnostic output and `--verbose` for debug-level information. Migration to the `tracing` crate would provide structured, leveled logging with span-based context. This is a worthwhile improvement but requires touching all diagnostic output sites across 13 modules. The `--verbose` flag already provides basic diagnostic output for debugging.

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

### Resolution Status

✅ Tool name validation (recommendation 1, partial) — `validate_tool_name()` in `src/docs.rs` rejects path traversal attempts, empty names, and invalid characters. Data sanitization via `src/sanitize.rs` provides `redact_paths()` and `redact_env_tokens()`.

✅ `cargo audit` in CI (recommendation 5) — security audit step added to `.github/workflows/ci.yml` for dependency vulnerability scanning.

⏳ **Deferred: API token file permission checks (recommendation 2).** Warning when config files are group/other-readable requires platform-specific permission checking — Unix uses `stat()` with mode bits, Windows uses ACLs, and WASM has no filesystem permissions. The cross-platform complexity (including handling macOS sandboxing, Linux containers, and WSL) makes this a non-trivial feature. Users handling sensitive tokens should use environment variables instead of config files in shared environments, as documented in the [Security Considerations](../reference/security-considerations.md) page.

⏳ **Deferred: SBOM generation (recommendation 4).** Generating a CycloneDX or SPDX Software Bill of Materials in CI requires adding a tool like `cargo-sbom` or `cargo-cyclonedx` to the build pipeline. While straightforward, the value is primarily for organizations with formal supply chain compliance requirements. The existing `cargo audit` in CI provides vulnerability scanning, and `Cargo.lock` serves as a de facto dependency manifest. SBOM generation can be added to the CI pipeline when compliance requirements demand it.

⏳ **Deferred: Sandboxed execution (recommendation 6).** Namespace/seccomp sandboxing for generated commands would require significant platform-specific infrastructure — Linux namespaces, macOS sandbox-exec, Windows AppContainers — with different capabilities and limitations on each platform. The current mitigation strategy is layered: `dry-run` for preview, `--ask` for human confirmation, tool name validation, and data sanitization. For high-security environments, users can wrap oxo-call output in their own sandboxing infrastructure (Docker, Firejail, etc.).

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

### Resolution Status

✅ `cargo audit` in CI (recommendation 1) — security audit step is part of the quality gate in `.github/workflows/ci.yml`.

✅ SHA256 checksums (recommendation 2) — `SHA256SUMS.txt` is generated alongside release binaries and published with each GitHub Release.

✅ mdBook build/deploy (recommendation 3) — mdBook documentation is built and deployed to GitHub Pages automatically in the CI pipeline.

✅ Code coverage (recommendation 4) — `cargo-tarpaulin` with Codecov upload is configured in CI.

⏳ **Deferred: Automated changelog generation (recommendation 5).** Tools like git-cliff or conventional-changelog can generate changelogs from commit messages. This requires adopting a commit message convention (e.g., Conventional Commits) and adding the tool to the CI pipeline. The current project uses descriptive commit messages and GitHub Release notes. Automated changelog generation can be added when the release cadence increases and manual release notes become burdensome.

✅ **Done: Smoke tests (recommendation 6).** The integration test suite in `tests/cli_tests.rs` includes multiple smoke tests that verify binary startup without LLM calls: `test_help_output`, `test_version_output`, `test_help_allowed_without_license`, `test_version_allowed_without_license`, `test_config_show`, `test_config_path`, `test_skill_list`, `test_docs_list_empty_or_filled`, and `test_completion_works_without_license`. These tests verify that the binary starts, processes arguments, and produces correct output across all major subcommands without requiring LLM API access.

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

### Resolution Status

✅ `CONTRIBUTING.md` (recommendation 1) — a comprehensive 355-line guide with development setup, skill authoring instructions, workflow template guidelines, PR guidelines, and issue guidelines.

✅ GitHub issue templates (recommendation 2) — three templates created in `.github/ISSUE_TEMPLATE/`: `bug_report.md`, `feature_request.md`, and `skill_request.md`.

✅ `CODE_OF_CONDUCT.md` (recommendation 3) — Contributor Covenant v2.1 adopted.

✅ `CITATION.cff` (recommendation 6) — CFF v1.2.0 metadata file at repository root for academic citation.

📋 **Out of scope: Public roadmap (recommendation 4).** A formal RFC process, GitHub Projects board, or roadmap document is an organizational decision that depends on project maturity and community size. The current project uses GitHub issues for tracking feature requests and bug reports, with issue templates for structured submissions. A public roadmap may be introduced as the project matures and the contributor community grows.

⏳ **Planned: Community skill registry (recommendation 5).** A central discovery and distribution mechanism for community-contributed skills (beyond sharing TOML files via git repositories and pull requests) is planned for a future release. The current approach — user skills in `~/.config/oxo-call/skills/`, community skills in `~/.local/share/oxo-call/skills/`, and `skill install --url` for remote installation — provides basic distribution. A registry with search, versioning, and quality metrics would enhance the skill ecosystem but requires infrastructure (hosting, API, review process) that is not yet justified by the current community size.

---

## Consolidated Action Items

The following prioritized action list synthesizes recommendations across all 12 evaluation reports:

### Priority 1 — Critical for Publication

| # | Action | Source Reports | Status |
|---|--------|---------------|--------|
| 1 | Design formal benchmark (100+ tasks, 20+ tools, accuracy metrics) | 5, 7, 8 | ✅ Done |
| 2 | Conduct ablation study (docs-only vs. docs+skills vs. full pipeline) | 5 | ✅ Done |
| 3 | Add command provenance (tool version + docs hash + skill version + model) | 1, 2, 3 | ✅ Done |
| 4 | Create public benchmark dataset for reproducible evaluation | 5, 7 | ✅ Done |
| 5 | Add CITATION.cff for academic citation | 7, 12 | ✅ Done |

### Priority 2 — Important for Quality & Security

| # | Action | Source Reports | Status |
|---|--------|---------------|--------|
| 6 | Add `cargo audit` to CI pipeline | 10, 11 | ✅ Done |
| 7 | Generate SHA256 checksums for release binaries | 3, 11 | ✅ Done |
| 8 | Add command sanitization layer | 10 | ✅ Done |
| 9 | Add mdBook documentation build/deploy to CI | 11 | ✅ Done |
| 10 | Add code coverage reporting | 11 | ✅ Done |
| 11 | Implement tool version tracking in history | 1, 4 | ✅ Done |

### Priority 3 — Enhances User Experience

| # | Action | Source Reports | Status |
|---|--------|---------------|--------|
| 12 | Add CONTRIBUTING.md | 12 | ✅ Done |
| 13 | Create GitHub issue templates | 12 | ✅ Done |
| 14 | Extend skill coverage (spatial omics, proteomics, multi-omics) | 8 | ⏳ Planned |
| 15 | Standardize minimum skill depth (5 examples, 3 concepts, 3 pitfalls) | 8 | ✅ Done |
| 16 | Refactor main.rs (extract command handlers) | 9 | ✅ Done |

### Priority 4 — Future Enhancements

| # | Action | Source Reports | Status |
|---|--------|---------------|--------|
| 17 | Add plugin trait for LLM providers | 9 | ✅ Done |
| 18 | Add lib.rs for programmatic API | 9 | ✅ Done |
| 19 | Community skill registry | 6, 12 | ⏳ Planned |
| 20 | Container image references in workflows | 3 | ✅ Done |
| 21 | Data anonymization for sensitive LLM contexts | 6 | ✅ Done |
| 22 | Structured logging with tracing crate | 9 | ⏳ Planned |
| 23 | Per-step environment management (`env` field) for Python 2/3, conda | 3, 9 | ✅ Done |
| 24 | Workflow engine reliability documentation (caching, error handling, DAG patterns) | 9 | ✅ Done |

---

## Workflow Accuracy Audit

A comprehensive audit of all built-in workflow templates was conducted to verify computation logic, flow logic, and documentation accuracy. The following corrections were made:

### Pipeline Flow Corrections

| Issue | Description | Resolution |
|-------|-------------|------------|
| RNA-seq description | `rnaseq.toml` description implied linear pipeline ending with MultiQC | Fixed to "fastp QC → STAR alignment / MultiQC (parallel) → featureCounts" |
| Methylseq description | `methylseq.toml` listed MultiQC as the final step | Fixed to show MultiQC as upstream QC step in parallel with Bismark alignment |
| engine.rs doc comment | Module-level doc example showed multiqc depending on `["fastp", "star"]` | Corrected to `["fastp"]` — MultiQC depends only on the QC step |
| Unit test | `test_compute_phases_complex_pipeline` modeled multiqc at end of pipeline | Corrected to upstream QC aggregation pattern: multiqc in same phase as STAR |
| Snakemake header | `rnaseq.smk` header omitted MultiQC parallel structure | Updated to reflect parallel QC aggregation |
| Nextflow header | `rnaseq.nf` header omitted MultiQC parallel structure | Updated to reflect parallel QC aggregation |

### Correct RNA-seq Pipeline DAG

```
Raw FASTQ reads
       │
  ▼ Quality Control (fastp)
Trimmed FASTQ + QC report
       │
  ┌────┴────┐
  ▼         ▼
STAR      MultiQC (gather)
(per-sample)  (aggregates QC)
  │
  ▼ samtools index
  │
  ▼ featureCounts
Count matrix (gene × sample)
```

Key design principle: **MultiQC is an upstream QC aggregation step that runs in parallel with STAR alignment**, not a final step that waits for all analysis to complete. This means QC reports are available while alignment is still running.

### MultiQC Positioning Verification

All 9 built-in templates were verified for correct MultiQC placement:

| Template | MultiQC depends on | Correct upstream placement | Verified |
|----------|-------------------|---------------------------|----------|
| rnaseq | fastp | ✅ Phase 2 (parallel with STAR) | ✅ |
| wgs | fastp | ✅ Phase 2 (parallel with BWA-MEM2) | ✅ |
| atacseq | fastp | ✅ Phase 2 (parallel with Bowtie2) | ✅ |
| chipseq | fastp | ✅ Phase 2 (parallel with Bowtie2) | ✅ |
| metagenomics | fastp | ✅ Phase 2 (parallel with host removal) | ✅ |
| methylseq | trim_galore | ✅ Phase 2 (parallel with Bismark) | ✅ |
| scrnaseq | fastp | ✅ Phase 2 (parallel with STARsolo) | ✅ |
| amplicon16s | fastp | ✅ Phase 2 (parallel with DADA2) | ✅ |
| longreads | nanostat | ✅ Phase 2 (parallel with Flye) | ✅ |

### Workflow Engine Improvements

The following engine improvements address reliability and flexibility concerns:

1. **`env` field** — Per-step shell preamble for conda environments, virtualenvs, PATH overrides, and module system integration. Enables pipelines that mix Python 2 and Python 3 tools, or different conda environments for different steps.

2. **Reliability documentation** — Workflow Engine reference now documents:
   - Output freshness caching semantics and edge cases
   - Error handling behavior (fail-fast with concurrent task completion)
   - Cycle detection at expansion and verification time
   - Concurrent execution safety guarantees
   - Complex DAG patterns (diamond, fan-out/fan-in, multi-gather)

3. **Step fields reference table** — Complete field reference with types, requirements, and descriptions for all `[[step]]` attributes.

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

#### Resolution Status

✅ **Done: Troubleshooting section (recommendation 1)** — a "Troubleshooting" section has been added to the [Configuration](../tutorials/configuration.md) page with examples of common first-run errors: wrong/missing license (with error message example), failed LLM connection (with diagnostic steps), and missing config file. A "CI / HPC Cluster Considerations" subsection covers SLURM job scripts, `GITHUB_TOKEN` absence, and shared Ollama deployment.

✅ **Done: Test data download links (recommendation 2)** — the [Quick Start](../tutorials/quickstart.md) tutorial now includes links to small test datasets: samtools test data (BAM/SAM files), nf-core test datasets (FASTQ, BAM, reference files), and a command to create a minimal test BAM.

✅ **Done: Expected output blocks (recommendation 3)** — the [Quick Start](../tutorials/quickstart.md) tutorial now includes expected output for all three Step 4 examples (dry-run, run, and --ask), showing the `Command:`, `Explanation:`, exit code, and confirmation prompt.

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

#### Resolution Status

✅ Skill depth requirements (recommendation 3) are enforced in the codebase — `validate_skill_depth()` in `src/skill.rs` checks `MIN_EXAMPLES=5`, `MIN_CONCEPTS=3`, `MIN_PITFALLS=3`. The validation is now explicitly documented in the [Create a Custom Skill](../how-to/create-custom-skill.md) how-to guide, including the "Debugging Skills" section that describes validation warnings.

✅ The Workflow Engine reference (recommendation related to gap 1) now documents complex DAG patterns including diamond dependencies, fan-out/fan-in, multiple gather points, and inter-phase dependencies. The reference also includes a step fields reference table, reliability considerations, and environment management.

✅ **Done: CI/cluster considerations (recommendation 1)** — a "CI / HPC Cluster Considerations" section has been added to the [Configuration](../tutorials/configuration.md) page with guidance on license setup via environment variables, API token management without config files, `GITHUB_TOKEN` alternatives, shared Ollama deployment, and a complete SLURM job script example.

✅ **Done: STAR two-pass mode note (recommendation 2)** — a callout about STAR two-pass mode has been added to the [RNA-seq Walkthrough](../tutorials/rnaseq-walkthrough.md) in the alignment section, explaining when to use `--twopassMode Basic` (tumor RNA-seq, rare transcripts) vs. standard one-pass mode (well-annotated genomes, standard differential expression).

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

#### Resolution Status

✅ The `sanitize.rs` module (recommendation 3) is documented — `redact_paths()` and `redact_env_tokens()` functionality is now described in the [Security Considerations](../reference/security-considerations.md) page, which explicitly documents what data is sent to the LLM API and what is anonymized.

✅ **Done: "Debugging skills" section (recommendation 1)** — a comprehensive "Debugging Skills" section has been added to the [Create a Custom Skill](../how-to/create-custom-skill.md) how-to guide. It documents using `--verbose` to see the full prompt sent to the LLM, common debugging steps (skill not loading, LLM ignoring skill, validation warnings), and how to test skills programmatically with `oxo-bench`.

✅ **Done: `oxo-bench` linked from contributing guide (recommendation 2)** — a "Benchmarking with oxo-bench" section has been added to [Contributing](../development/contributing.md) with usage examples for running the full benchmark suite, testing specific tools, running ablation tests, and exporting CSV results.

✅ **Done: Raw LLM prompt format (recommendation 4)** — the [LLM Integration](../reference/llm-integration.md) reference now includes a complete "Raw Prompt Example" showing the actual prompt structure sent to the LLM: tool header, skill knowledge injection (concepts, pitfalls, examples), tool documentation, task description, and strict output format instructions. The `--verbose` flag is documented for viewing the actual prompt for any command.

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

#### Resolution Status

✅ Ollama local LLM support enables fully air-gapped operation (recommendation 2) — the functionality is documented in the new "Air-Gapped / Offline Mode" section of the [Switch LLM Provider](../how-to/change-llm-provider.md) guide with a complete setup walkthrough and network requirements table.

✅ Data anonymization via `src/sanitize.rs` (recommendation 3) — `redact_paths()` and `redact_env_tokens()` are implemented, and the new [Security Considerations](../reference/security-considerations.md) page explicitly documents what data is sent to the LLM API and what is not (with a comparison table).

✅ **Done: Team Setup / Organizational Deployment (recommendation 1)** — a "Team Setup / Organizational Deployment" section has been added to the [Switch LLM Provider](../how-to/change-llm-provider.md) how-to guide, covering shared environment variables, shared skill directories, skill distribution via Git repositories, and multi-user license deployment.

✅ **Done: Air-gapped mode documentation (recommendation 2)** — a comprehensive "Air-Gapped / Offline Mode" section has been added to the [Switch LLM Provider](../how-to/change-llm-provider.md) how-to guide with a complete offline setup walkthrough (Ollama, pre-cached documentation, offline license), a feature-by-feature network requirements table, and verification steps.

✅ **Already resolved: Commercial license scope (recommendation 4)** — the [License Setup](../tutorials/license.md) page already documents: "Commercial licenses are **USD 200 per organization** — a single license covers all employees and contractors within your organization." This matches the README content and fully addresses the clarification request.

✅ **Done: Security considerations page (recommendation 5)** — a new [Security Considerations](../reference/security-considerations.md) reference page has been added to the Architecture & Design section, documenting the threat model, input validation mitigations, data anonymization (what is/isn't sent to LLM), dry-run mode, API token security, license security, supply chain security, and deployment recommendations for single-user, shared HPC, and clinical environments.

---

### Documentation Iteration Summary

Based on the four-role review above, all identified documentation issues have been addressed:

| Priority | Issue | Reviewer(s) | Status |
|----------|-------|-------------|--------|
| 🔴 High | Add troubleshooting examples with error messages for first-run failures | Student | ✅ Done |
| 🔴 High | Document what data is sent to LLM API (privacy/compliance) | Core Manager | ✅ Done |
| 🟡 Medium | Add team/organizational deployment how-to | Core Manager | ✅ Done |
| 🟡 Medium | Add air-gapped / offline mode documentation | Core Manager | ✅ Done |
| 🟡 Medium | Add test data download links to tutorials | Student | ✅ Done |
| 🟡 Medium | Document skill depth requirements explicitly in how-to | Experienced Bio | ✅ Done |
| 🟡 Medium | Document complex DAG patterns and step fields reference | Experienced Bio | ✅ Done |
| 🟡 Medium | Document workflow reliability (caching, error handling, env) | Experienced Bio | ✅ Done |
| 🟢 Low | STAR two-pass mode note in RNA-seq tutorial | Experienced Bio | ✅ Done |
| 🟢 Low | Show raw LLM prompt format in reference | Methods Developer | ✅ Done |
| 🟢 Low | Link oxo-bench from contributing guide | Methods Developer | ✅ Done |
