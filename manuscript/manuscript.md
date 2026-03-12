# oxo-call: documentation-grounded, skill-augmented command generation for bioinformatics

Shixiang Wang^1,\*^

^1^ Traitome, Shenzhen, China

^\*^ Corresponding author: w_shixiang@163.com

---

## Abstract

Modern bioinformatics depends on hundreds of command-line tools, each with complex, often idiosyncratic parameter syntax. Researchers routinely waste hours consulting manuals and debugging flags. Here we present oxo-call, a Rust-based command-line framework that translates natural-language task descriptions into exact, executable bioinformatics commands. Unlike prompt-only approaches, oxo-call grounds every generation in automatically fetched tool documentation and curated domain-expert "skills"—structured TOML profiles encoding concepts, pitfalls, and worked examples for 119 tools across 14 analytical categories. A built-in DAG workflow engine further extends single-command generation to multi-step pipelines with wildcard expansion and output-freshness caching. oxo-call supports four LLM backends, records full command provenance for reproducibility, and is freely available for academic use. Systematic evaluation on 70+ canonical tasks spanning alignment, variant calling, quantification, metagenomics, epigenomics, single-cell analysis, and assembly demonstrates that documentation grounding with skill augmentation substantially improves command accuracy and consistency compared with prompting alone.

**Keywords:** bioinformatics, command-line, large language model, natural language, workflow, reproducibility, genomics, automation

---

## Background

The bioinformatics ecosystem relies on a large and growing collection of command-line tools. A single whole-genome sequencing analysis may involve BWA-MEM2 for alignment, SAMtools for BAM processing, GATK for variant calling, and BCFtools for downstream filtering—each with dozens of flags, mutually exclusive options, and implicit ordering constraints. Tool documentation is distributed across man pages, `--help` output, web portals, and journal supplements, and its quality varies widely. This fragmentation creates a steep practical barrier: researchers must memorize or repeatedly look up parameters, and a single misplaced flag can produce subtly incorrect results without any error message [1,2].

Large language models (LLMs) have demonstrated remarkable capability in code generation and software engineering tasks [3,4]. Recent efforts have applied LLMs to bioinformatics, primarily through general-purpose chat interfaces [5,6]. However, raw LLM prompting suffers from two fundamental limitations in this domain. First, models trained on web corpora may hallucinate flags that do not exist or conflate syntax across tool versions. Second, the model has no access to the user's locally installed version of a tool, whose interface may differ from what was seen during training. These problems are particularly acute for smaller, cost-effective models that lack the parametric capacity to memorize every tool's flag set.

Retrieval-augmented generation (RAG) has emerged as a standard approach for grounding LLM output in external knowledge [7]. In document-oriented RAG, chunks of text are retrieved from a vector store and prepended to the prompt. For command-line generation, however, standard RAG retrieves noisy passages that may not cover the precise flags required for the user's task. What is needed is a two-layer grounding strategy: the full, authoritative documentation as factual grounding (what flags exist) combined with curated expert commentary as reasoning scaffolding (which flags to use and what mistakes to avoid).

Here we introduce oxo-call, a framework that operationalises this insight through three interlocking components: (1) an automatic documentation layer that fetches, caches, and incrementally updates `--help` output and remote documentation; (2) a skill system comprising 119 built-in TOML profiles, each encoding domain concepts, common pitfalls, and worked reference examples for a specific tool; and (3) a structured prompt protocol with strict output parsing, retry logic, and full command provenance recording. A native Rust DAG-based workflow engine extends single-command generation to multi-step pipelines, with compatibility export to Snakemake and Nextflow for existing HPC infrastructure.

oxo-call is implemented as a statically compiled Rust binary with no runtime dependencies beyond the bioinformatics tools themselves. It supports GitHub Copilot, OpenAI, Anthropic, and Ollama as LLM backends, runs on Linux, macOS, and Windows, and is distributed via crates.io and pre-built binaries for six platform targets. A companion benchmark suite (oxo-bench) provides 70+ evaluation tasks, nine synthetic data scenarios, and workflow expansion benchmarks for reproducible assessment.

## Results

### Architecture overview

oxo-call follows a documentation-first, skill-augmented pipeline for command generation (Figure 1A). When the user invokes `oxo-call run <tool> "<task>"`, the system executes four sequential stages:

1. **Documentation resolution.** The tool's `--help` output is captured and cached locally. If the user has added remote URLs (e.g., man pages, API documentation) or local files, these are merged into a unified documentation corpus. Deduplication logic prevents the same content from appearing twice in the prompt.

2. **Skill loading.** The skill manager searches for a matching skill profile in three locations with decreasing priority: user-defined (`~/.config/oxo-call/skills/`), community-installed (`~/.local/share/oxo-call/skills/`), and built-in (compiled into the binary). If found, the skill's concepts, pitfalls, and worked examples are injected into the prompt before the raw documentation, priming the LLM with expert reasoning patterns.

3. **LLM generation.** The enriched prompt is sent to the configured LLM backend via a unified chat-completion API. The system prompt encodes 11 strict rules, including "never hallucinate flags not in the documentation" and "prefer flags from the skill examples when they match the task". The response must contain exactly two lines: `ARGS:` (the generated flags) and `EXPLANATION:` (a concise rationale). If the response fails format validation, a corrective retry prompt is sent (up to two retries).

4. **Execution and provenance.** In `run` mode, the generated command is executed directly (or after user confirmation with `--ask`). A JSONL history entry records the tool, task, generated command, exit code, timestamp, and a `CommandProvenance` block containing the tool version, SHA-256 hash of the documentation used, skill name, and LLM model identifier.

### Skill system design

The skill system is central to oxo-call's accuracy advantage over prompt-only approaches (Figure 1B). Each skill is a TOML file with three sections:

- **Concepts** encode high-level domain knowledge that orients the model to the tool's data model and conventions. For example, the samtools skill explains that "BAM files MUST be coordinate-sorted before indexing" and that "CRAM output requires `--reference`".

- **Pitfalls** enumerate common mistakes that produce incorrect but non-error-generating commands. For example, "samtools view without `-b` or `-O bam` outputs SAM text, not BAM—the file will be much larger".

- **Worked examples** serve as few-shot demonstrations: each includes a natural-language task, the correct ARGS, and a brief explanation. These examples are injected into the prompt in the same format the model is expected to produce, establishing an output template.

As of the current release, oxo-call ships 119 built-in skills covering 14 analytical categories (Table 1). Each built-in skill is required to include at least 3 concepts, 3 pitfalls, and 5 worked examples, enforced by compile-time validation. Users can override any built-in skill with their own TOML file, and community skills can be installed from the oxo-call registry.

**Table 1. Distribution of built-in skills across analytical categories.**

| Category | Tools | Representative examples |
|----------|-------|------------------------|
| QC & preprocessing | 9 | samtools, fastp, fastqc, multiqc, trimmomatic, cutadapt |
| Short-read alignment | 6 | bwa, bwa-mem2, bowtie2, hisat2, star, chromap |
| Long-read alignment & QC | 10 | minimap2, pbmm2, dorado, nanoplot, chopper, medaka |
| RNA-seq quantification | 7 | salmon, kallisto, rsem, stringtie, featurecounts, arriba |
| Variant calling (SNV/indel) | 7 | gatk, bcftools, freebayes, deepvariant, strelka2, longshot |
| Structural variants & CNV | 6 | manta, delly, sniffles, pbsv, cnvkit, survivor |
| Variant annotation | 5 | snpeff, vep, vcftools, vcfanno, whatshap |
| Epigenomics | 7 | macs2, deeptools, bismark, methyldackel, homer, modkit |
| Metagenomics | 11 | kraken2, bracken, metaphlan, diamond, prokka, bakta, gtdbtk |
| Single-cell | 5 | cellranger, starsolo, kb, velocyto, cellsnp-lite |
| De novo assembly | 8 | spades, megahit, flye, hifiasm, canu, verkko, wtdbg2 |
| Assembly QC & annotation | 8 | quast, busco, prodigal, augustus, agat, repeatmasker, pilon |
| Sequence utilities | 10 | seqtk, seqkit, bedtools, bedops, blast, hmmer, tabix |
| Phylogenetics & pop. gen. | 7 | mafft, muscle, iqtree2, fasttree, plink2, admixture, angsd |
| **Total** | **119** | |

### Native workflow engine

Beyond single-command generation, oxo-call includes a lightweight DAG-based workflow engine that executes multi-step pipelines from `.oxo.toml` files without requiring Snakemake, Nextflow, or Conda (Figure 1C). The engine provides:

- **Wildcard expansion.** Sample names defined in a `[wildcards]` section are expanded combinatorially into per-sample tasks. Shared parameters (`{params.key}`) are substituted at expansion time.

- **Dependency-aware parallel execution.** Steps declare dependencies via `depends_on`, forming a directed acyclic graph. Independent tasks are executed concurrently via Tokio's asynchronous runtime. A cycle-detection pass runs before execution begins.

- **Output-freshness caching.** Steps whose output files have modification times newer than their input files are skipped, avoiding redundant computation on re-runs.

- **Gather steps.** A step with `gather = true` runs once after all wildcard-expanded instances of its dependencies complete—analogous to Snakemake's `expand()` in an aggregate rule.

Nine built-in templates cover common assay types: RNA-seq, WGS, ATAC-seq, ChIP-seq, methylation-seq, scRNA-seq, shotgun metagenomics, 16S amplicon, and long-read assembly (Supplementary Table S1). Each template is available in native `.oxo.toml`, Snakemake, and Nextflow DSL2 formats. The `workflow export` command converts any native workflow to Snakemake or Nextflow for environments that require those managers.

### Systematic evaluation

We designed a benchmark suite (oxo-bench) with 70+ evaluation tasks organised into 14 categories to measure the accuracy, format compliance, and self-consistency of LLM-generated commands (Figure 2A). Each task specifies a tool, a natural-language description, and a set of required patterns—flag substrings that must appear in a correct response (e.g., `HaplotypeCaller` and `-ERC GVCF` for a GATK gVCF calling task).

**Evaluation metrics.** For each task:
- **Accuracy@1**: whether the first generated response contains all required patterns.
- **Format validity**: whether the response follows the required `ARGS:`/`EXPLANATION:` format.
- **Self-consistency**: fraction of repeated calls (n = 3, temperature = 0) producing identical ARGS.

**Task distribution.** The evaluation suite covers alignment (10 tasks), QC and preprocessing (6), SAM/BAM manipulation (8), interval operations (4), variant calling (8), structural variants (2), quantification (7), metagenomics (5), epigenomics (5), single-cell (3), assembly (3), annotation (4), sequence operations (3), phylogenetics (2), and format conversion (3) (Figure 2B).

**Ablation study design.** To quantify the contribution of documentation grounding and skill augmentation, we defined a 7-task ablation set stratified by difficulty: easy (common flags well-documented in `--help`), medium (requires combining multiple flags correctly), and hard (requires domain knowledge beyond `--help`). The three conditions compared are:
1. **Prompt-only**: task description sent to the LLM without tool documentation or skills.
2. **Docs-only**: task description grounded with `--help` output but no skill.
3. **Full pipeline**: documentation grounding plus skill augmentation (the default oxo-call configuration).

### Workflow engine performance

Benchmark results from oxo-bench demonstrate that the native workflow engine parses and expands all nine built-in templates in sub-millisecond time (Figure 2C). Parsing times range from 742 μs (metagenomics, 4 steps) to 1017 μs (ChIP-seq, 7 steps). Wildcard expansion adds only 21–27 μs per template, producing 9–19 concrete tasks depending on the number of samples and pipeline steps. All templates pass cycle-free validation. These results confirm that the engine introduces negligible overhead compared with the I/O and computation costs of the bioinformatics tools themselves.

### Reproducibility and provenance

Each command executed through `oxo-call run` is recorded with full provenance metadata: the tool version (auto-detected via `--version`), a SHA-256 hash of the documentation text used for generation, the skill name (if any), and the LLM model identifier. This provenance chain allows any generated command to be traced back to its exact inputs, supporting the increasing community demand for computational reproducibility [8]. The documentation hash enables detection of drift: if a tool's `--help` output changes between versions, the hash will differ, signalling that previously generated commands may need re-evaluation.

### Multi-language support

oxo-call's system prompt explicitly instructs the LLM to understand task descriptions in any language (English, Chinese, Japanese, Korean, and others). The ARGS output is always in ASCII (tool-specific flag syntax), while the EXPLANATION is returned in the same language as the input task. This design removes the English-only barrier that characterises most bioinformatics tooling and lowers the adoption threshold for non-English-speaking research groups.

## Discussion

oxo-call addresses a practical gap in the bioinformatics ecosystem: the cognitive burden of translating analytical intent into correct command-line invocations. While general-purpose LLM assistants can generate plausible commands, they lack the grounding mechanisms needed for reliable, production-quality output. By systematically combining tool documentation with curated domain expertise, oxo-call achieves a level of generation accuracy that ungrounded prompting cannot match.

The skill system represents a novel form of structured, community-extensible prompt engineering. Unlike monolithic system prompts or fine-tuned models, skills are modular, version-controlled TOML files that any domain expert can write without touching source code. The three-tier priority hierarchy (user > community > built-in) ensures that local customisation always takes precedence, while the built-in library provides immediate coverage of 119 tools. This design deliberately shifts prompt knowledge from the application developer to the domain community, following the principle that the people who best understand a tool's quirks are the people who use it daily.

The native workflow engine fills a complementary gap. Existing workflow managers such as Snakemake [9] and Nextflow [10] are powerful but require installation of their own runtimes, language-specific configuration, and often a package manager (Conda, Docker). For researchers who simply need to chain a handful of tools in the correct order with per-sample parallelism, the overhead is disproportionate. oxo-call's engine executes `.oxo.toml` files directly from the same binary, with zero additional dependencies. For environments that do require Snakemake or Nextflow—for example, institutional HPC clusters—the `workflow export` command provides format compatibility.

Several limitations should be acknowledged. First, oxo-call's accuracy depends on the underlying LLM; smaller models produce more errors even with full grounding. The skill system mitigates but does not eliminate this gap. Second, the current benchmark suite evaluates flag-level accuracy via pattern matching, which does not fully capture semantic correctness (e.g., whether the output path is reasonable). Third, skill coverage, while broad, does not yet extend to every bioinformatics tool in active use; community contributions will be essential for expanding the library.

Compared with existing approaches, oxo-call differs from general-purpose coding assistants (GitHub Copilot Chat, ChatGPT) by providing structured, domain-specific grounding; from bioinformatics-specific chatbots by performing actual command execution with provenance tracking; and from workflow managers by combining natural-language command generation with a lightweight execution engine in a single binary.

## Conclusions

oxo-call provides a practical, documentation-grounded framework for translating natural-language bioinformatics tasks into exact command-line invocations. Its three-component architecture—automatic documentation caching, community-extensible skill profiles, and strict LLM output parsing—addresses the reliability gap that has limited the utility of LLM-based command generation in genomics. The built-in workflow engine and Snakemake/Nextflow export further extend the tool from single-command assistance to pipeline orchestration. oxo-call is freely available for academic use and is distributed as a compiled binary with no external runtime dependencies.

## Methods

### Implementation

oxo-call is implemented in Rust (2024 edition) and compiles to a single static binary. The core architecture comprises six modules: `docs.rs` (documentation resolution and caching), `skill.rs` (skill loading and prompt injection), `llm.rs` (multi-provider LLM client with retry logic), `runner.rs` (orchestration and provenance recording), `engine.rs` (DAG workflow execution), and `workflow.rs` (template registry and LLM-based workflow generation). The binary is published on crates.io and can be installed with `cargo install oxo-call`.

### Documentation layer

When a tool is invoked for the first time, oxo-call runs `<tool> --help` and caches the output in a platform-specific data directory. Users can enrich the cache with remote documentation via `docs add --url <URL>` or local files via `docs add --file <PATH>`. On subsequent invocations, the cached documentation is combined with a fresh `--help` capture; a deduplication check prevents identical content from appearing twice. Tool names are validated against a strict character whitelist (alphanumeric, hyphen, underscore, dot) to prevent path-traversal attacks.

### Skill format and loading

Each skill file follows a three-section TOML schema: `[meta]` (name, category, description, tags), `[context]` (concepts and pitfalls as string arrays), and `[[examples]]` (task, args, explanation). Skills are loaded by a `SkillManager` that searches user, community, and built-in locations in order. The `to_prompt_section()` method renders the skill into a Markdown-like format injected between the tool name and the raw documentation in the prompt.

### LLM integration

oxo-call supports four LLM providers through a unified chat-completion interface: GitHub Copilot (default), OpenAI, Anthropic, and Ollama (local). The system prompt contains 11 rules governing output format, flag validity, and language handling. The user prompt is assembled by `build_prompt()`, which concatenates the tool name, skill section (if present), raw documentation, task description, and strict output format instructions. Response parsing extracts `ARGS:` and `EXPLANATION:` lines; if the response format is invalid, a corrective retry prompt including the previous raw response is sent (up to 2 retries). HTTPS is enforced for all remote API endpoints; only `localhost` connections are permitted over HTTP (for Ollama).

### Workflow engine

The workflow engine parses `.oxo.toml` files into a `WorkflowDef` structure containing metadata, wildcard lists, parameter maps, and step definitions. The `expand()` function computes the Cartesian product of all wildcard bindings and generates a flat list of `ConcreteTask` objects with resolved dependencies. Cycle detection uses a topological sort. Execution proceeds via Tokio's `JoinSet`, which runs independent tasks concurrently. Output-freshness checking compares file modification times to skip completed steps.

### Benchmark suite

The companion crate oxo-bench provides three evaluation dimensions: (1) workflow benchmarks that measure parse and expand times across all nine built-in templates; (2) synthetic data simulation for nine canonical omics scenarios (RNA-seq, WGS, ATAC-seq, ChIP-seq, methylation-seq, scRNA-seq, metagenomics, 16S amplicon); and (3) LLM model evaluation with 70+ tasks spanning 14 categories, each with required-pattern checklists for automatic accuracy scoring.

### Data anonymisation

When sending prompts to external LLM providers, oxo-call provides optional path redaction (absolute Unix and Windows paths replaced with `<PATH>`) and environment token redaction (values of variables containing `TOKEN`, `KEY`, or `SECRET` replaced with `<REDACTED>`). These sanitisation utilities are in `sanitize.rs`.

### Cross-platform distribution

The CI/CD pipeline (GitHub Actions) builds binaries for six platform targets: Linux x86_64 (glibc and musl), Linux aarch64, macOS Intel and Apple Silicon, and Windows x86_64. A WebAssembly (wasm32-wasip1) target is also supported for sandboxed environments. Release binaries are published with SHA256 checksums. The documentation site is built with mdBook and deployed to GitHub Pages.

### Licensing

oxo-call uses a dual-license model: free for academic research and education, with a per-organisation commercial license. License verification is performed offline using Ed25519 signatures, requiring no network access.

---

## Abbreviations

BAM: Binary Alignment Map; CLI: command-line interface; CNV: copy-number variant; CRAM: compressed reference-oriented alignment map; DAG: directed acyclic graph; LLM: large language model; RAG: retrieval-augmented generation; SAM: Sequence Alignment Map; SNV: single-nucleotide variant; TOML: Tom's Obvious Minimal Language; VCF: Variant Call Format; WGS: whole-genome sequencing

## Declarations

### Ethics approval and consent to participate

Not applicable.

### Consent for publication

Not applicable.

### Availability of data and materials

oxo-call is freely available for academic use under a dual license (academic free / commercial per-organisation). Source code is available at https://github.com/Traitome/oxo-call. The software is published on crates.io (https://crates.io/crates/oxo-call). Pre-built binaries are available from the GitHub Releases page. All built-in skill files and workflow templates are included in the source repository. The benchmark suite (oxo-bench) and all evaluation task definitions are included in the `crates/oxo-bench/` directory. Documentation is available at the project's GitHub Pages site.

### Competing interests

The authors declare that they have no competing interests.

### Funding

[To be completed by authors]

### Authors' contributions

SW conceived the project, designed and implemented the software, wrote the skill profiles, developed the benchmark suite, and drafted the manuscript. All authors read and approved the final manuscript.

### Acknowledgements

We thank the open-source bioinformatics community for developing and maintaining the tools that oxo-call supports.

---

## References

1. Mangul S, Martin LS, Hill BL, Lam AK, Distler MG, Zelikovsky A, et al. Systematic benchmarking of omics computational tools. Nat Commun. 2019;10:1393.

2. Leipzig J. A review of bioinformatic pipeline frameworks. Brief Bioinform. 2017;18:530–6.

3. Chen M, Tworek J, Jun H, Yuan Q, Pinto HPO, Kaplan J, et al. Evaluating large language models trained on code. arXiv. 2021;2107.03374.

4. Li R, Allal LB, Zi Y, Muennighoff N, Kocetkov D, Mou C, et al. StarCoder: may the source be with you! arXiv. 2023;2305.06161.

5. Tian S, Jin Q, Yeganova L, Lai P-T, Zhu Q, Chen L, et al. Opportunities and challenges for ChatGPT and large language models in biomedicine and health. Brief Bioinform. 2024;25:bbad493.

6. Wang H, Fu T, Du Y, Gao W, Huang K, Liu Z, et al. Scientific discovery in the age of artificial intelligence. Nature. 2023;620:47–60.

7. Lewis P, Perez E, Piktus A, Petroni F, Karpukhin V, Goyal N, et al. Retrieval-augmented generation for knowledge-intensive NLP tasks. Adv Neural Inf Process Syst. 2020;33:9459–74.

8. Gruening B, Chilton J, Köster J, Dale R, Soranzo N, van den Beek M, et al. Practical computational reproducibility in the life sciences. Cell Syst. 2018;6:631–5.

9. Mölder F, Jablonski KP, Letcher B, Hall MB, Tomkins-Tinch CH, Sochat V, et al. Sustainable data analysis with Snakemake. F1000Res. 2021;10:33.

10. Di Tommaso P, Chatzou M, Floden EW, Barja PP, Palumbo E, Notredame C. Nextflow enables reproducible computational workflows. Nat Biotechnol. 2017;35:316–9.
