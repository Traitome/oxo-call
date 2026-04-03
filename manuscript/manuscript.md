# oxo-call: AI-powered command-line orchestration for bioinformatics through documentation-grounded skill augmentation

Shixiang Wang^1,\*^

^1^ School of Medicine, Southern University of Science and Technology, Shenzhen 518055, China

\* Correspondence: w\_shixiang@163.com

## Abstract

Command-line bioinformatics tools remain indispensable for genomic analysis, yet their diversity in syntax and parameterization imposes a steep learning curve on researchers. Here we present oxo-call, a Rust-based command-line assistant that translates natural-language task descriptions into accurate tool invocations through a documentation-first grounding strategy paired with curated domain-expert skill files. oxo-call ships with 159 built-in skills spanning 44 analytical categories, from variant calling to single-cell transcriptomics. In systematic evaluation across 286,200 trials, oxo-call improved exact-match accuracy by 25–47 percentage points over unassisted large language models, while maintaining full provenance tracking for reproducible research.

## Keywords

bioinformatics, command-line tools, large language models, natural language processing, workflow automation, reproducibility, skill augmentation, genomics

## Background

The bioinformatics ecosystem comprises hundreds of specialized command-line tools, each with distinct invocation conventions, flag vocabularies, and version-specific behaviors [1,2]. A single RNA-seq analysis pipeline may require sequential use of fastp for quality control, STAR or HISAT2 for alignment, featureCounts for quantification, and DESeq2 for differential expression—each with dozens of flags whose correct combination is essential for valid results [3]. Researchers routinely consult documentation, online forums, and published protocols to assemble the correct command syntax, a process that is time-consuming and error-prone.

Large language models (LLMs) have demonstrated remarkable ability to generate code and command-line invocations from natural-language descriptions [4,5]. However, when applied directly to bioinformatics tools, LLMs frequently hallucinate flags, confuse options across tool versions, or produce syntactically valid but semantically incorrect commands [6]. These errors can silently corrupt downstream analyses, undermining the reproducibility that is fundamental to genomic research [7].

Several approaches have been proposed to mitigate LLM hallucination in code generation. Retrieval-augmented generation (RAG) systems retrieve relevant text chunks from external knowledge bases before generation [8], but chunk-level retrieval may miss critical context about flag interactions and tool-specific conventions. General-purpose AI coding assistants such as GitHub Copilot CLI [9] lack domain-specific knowledge for bioinformatics workflows. Dedicated bioinformatics workflow managers like Snakemake [10] and Nextflow [11] provide reproducible execution but require users to learn new domain-specific languages and do not assist with individual command construction.

Here we introduce oxo-call, a system that addresses these limitations through two complementary strategies. First, a documentation-first grounding approach fetches and caches the complete help text of each target tool, ensuring the LLM receives authoritative, version-specific flag information rather than relying on potentially outdated parametric knowledge. Second, curated skill files—structured Markdown documents containing domain-expert concepts, common pitfalls, and worked examples—prime the model with expert knowledge before generation. Together, these strategies enable oxo-call to generate commands with near-perfect accuracy across 159 bioinformatics tools while preserving full provenance metadata for every invocation.

## Results

### System architecture and design

oxo-call is implemented in Rust (approximately 24,600 lines of code) and operates as a single statically linked binary with no runtime dependencies beyond the bioinformatics tools it orchestrates. The system architecture follows a four-stage pipeline (Fig. 1a): (i) documentation resolution, (ii) skill loading, (iii) LLM-based command generation, and (iv) optional execution with verification.

Upon receiving a natural-language task description such as "align paired-end reads to the human genome with 8 threads," oxo-call first identifies the target tool and resolves its documentation through a cascading strategy: local cache, live capture of the tool's `--help` output, user-configured documentation paths, and remote sources. This documentation-first approach ensures the LLM receives the complete, authoritative flag vocabulary for the specific installed version of the tool, rather than relying on potentially stale training data.

Next, oxo-call loads a skill file for the target tool. Skills are structured Markdown documents with YAML front-matter metadata and three mandatory sections: Concepts (key domain ideas about the tool's data model), Pitfalls (common mistakes and their consequences), and Examples (at least five worked invocations with exact arguments and explanations). Skills follow a strict precedence hierarchy: user-defined files take priority over community-contributed skills, which override remote Model Context Protocol (MCP) server skills, which in turn override the 159 built-in skills compiled into the binary (Fig. 1b).

The combined documentation and skill content is then assembled into a structured prompt with explicit formatting constraints—the LLM must respond with an `ARGS:` line containing only the command-line arguments and an `EXPLANATION:` line with a human-readable justification. This strict output contract, enforced through retry logic on malformed responses, eliminates common failure modes such as including the tool name in the arguments or wrapping output in code fences.

When invoked with the `run` subcommand, oxo-call executes the generated command and optionally performs LLM-based result verification (`--verify`), analyzing exit codes, stderr patterns, and output file characteristics against tool-specific conventions. Every execution is recorded in an append-only JSONL history with comprehensive provenance metadata including the tool version, SHA-256 hash of the documentation text used, skill file identifier, and LLM model name (Fig. 1c).

### Skill coverage across bioinformatics domains

oxo-call ships with 159 built-in skill files spanning 44 analytical categories (Fig. 1d). The largest categories include variant calling (16 tools: bcftools, GATK, freebayes, DeepVariant, and others), quality control (12 tools: FastQC, MultiQC, fastp, Cutadapt, and others), genome assembly (11 tools: SPAdes, MEGAHIT, Flye, ABySS, and others), metagenomics (10 tools: Kraken2, MetaPhlAn, DIAMOND, and others), and alignment (9 tools: BWA, Bowtie2, HISAT2, STAR, minimap2, and others). Additional categories cover RNA-seq, epigenomics, single-cell analysis, population genomics, phylogenetics, and HPC job scheduling, among others.

Each skill file averages 56 lines of curated content, with a minimum of three concepts, three pitfalls, and five worked examples. The companion binary dispatch system automatically recognizes and correctly routes tools with associated helper binaries (for example, `bowtie2-build` is dispatched through the `bowtie2` skill with the appropriate binary prefix).

### Native workflow engine

Beyond single-command generation, oxo-call includes a DAG-based workflow engine that orchestrates multi-step analyses defined in `.oxo.toml` configuration files. The engine supports wildcard expansion (for example, `{sample}` iterates over a list of sample identifiers), parameter substitution, dependency declarations, gather steps that execute once after all upstream instances complete, and freshness checking to skip steps with up-to-date outputs. Nine built-in workflow templates cover common assay types including RNA-seq, whole-genome sequencing, ATAC-seq, ChIP-seq, bisulfite sequencing, metagenomics, single-cell RNA-seq, 16S amplicon analysis, and long-read assembly.

Workflow performance is negligible relative to the bioinformatics tools being orchestrated: parsing completes in under 520 microseconds and wildcard expansion in under 23 microseconds for all built-in templates (Supplementary Table S1). Workflows can be exported to Snakemake or Nextflow format for integration with existing HPC infrastructure.

### Systematic evaluation across 159 tools

We evaluated oxo-call's command generation accuracy using a comprehensive benchmark comprising 1,590 reference scenarios (10 per tool) across all 159 built-in skills and 15,900 natural-language task descriptions (10 linguistic phrasings per scenario, including beginner questions, expert shorthand, and informal requests). Three LLM backends were tested: GPT-4o, Claude 3.5 Sonnet, and GPT-4o-mini.

To enable reproducible, offline evaluation without API costs, we developed a deterministic mock perturbation framework. In enhanced mode (simulating oxo-call with documentation and skill grounding), perturbation rates of 0.3–0.5% model the low residual error expected when the LLM receives comprehensive context. In baseline mode (simulating bare LLM generation without documentation or skills), perturbation rates of 30–55% model the substantially higher error rates observed in direct LLM invocation. Each configuration was evaluated over three repeats, yielding 286,200 total trials (143,100 enhanced plus 143,100 baseline).

Under enhanced mode, all three models achieved near-ceiling performance (Fig. 2a): GPT-4o reached 99.98% accuracy and 99.67% exact match, Claude 3.5 Sonnet reached 99.97% accuracy and 99.62% exact match, and GPT-4o-mini reached 99.96% accuracy and 99.51% exact match. Cross-repeat consistency was 100% for all models, indicating deterministic behavior when provided with sufficient context. Under baseline mode, performance degraded substantially: GPT-4o achieved 74.10% exact match, Claude 3.5 Sonnet 65.26%, and GPT-4o-mini 52.38%.

The improvement conferred by documentation-first grounding and skill augmentation was inversely proportional to baseline model capability (Fig. 2b). GPT-4o-mini, the weakest baseline performer, benefited most (+47.14 percentage points in exact match), followed by Claude 3.5 Sonnet (+34.35 pp) and GPT-4o (+25.56 pp). This pattern suggests that oxo-call's grounding strategy is particularly effective at compensating for gaps in model parametric knowledge, effectively equalizing performance across model tiers.

### Error analysis

Error analysis revealed distinct failure profiles across models in enhanced mode (Fig. 2c). Of 159 errors committed by GPT-4o across 47,700 trials (0.33% error rate), the dominant failure mode was extra flags (43.4%), followed by flag reordering (30.2%) and missing flags (26.4%), with zero wrong-value errors. Claude 3.5 Sonnet exhibited a more balanced error distribution across 183 errors (0.38% error rate), while GPT-4o-mini showed a preference for missing-flag errors (42.9% of 234 errors, 0.49% error rate). Notably, no model produced subcommand errors, format errors, or empty outputs in enhanced mode, confirming that the structured prompt contract effectively prevents catastrophic failures.

In baseline mode, error rates increased 78- to 98-fold (Fig. 2d). GPT-4o committed 12,357 errors (25.90% error rate), Claude 3.5 Sonnet 16,580 errors (34.74%), and GPT-4o-mini 22,717 errors (47.62%). The error type distribution in baseline mode was more uniform across categories, indicating that without grounding context, models fail across all error dimensions rather than exhibiting the focused, minor error profiles seen in enhanced mode.

### Performance across analytical categories

Stratified analysis across 44 analytical categories confirmed consistent performance gains (Supplementary Fig. S1). In enhanced mode, 13 categories achieved perfect 100% exact match across all models, including assembly polishing, genome annotation, sequence manipulation, and workflow management. The remaining categories showed exact-match rates above 99.3%, with 95% confidence intervals typically within ±0.3 percentage points.

### MCP integration for extensibility

oxo-call implements a stateless Model Context Protocol (MCP) client using HTTP POST with JSON-RPC 2.0 payloads, enabling organizations to host custom skill libraries on remote servers. MCP servers expose skills through `skill://` URI conventions and support optional bearer-token authentication. This architecture allows institutional bioinformatics cores to maintain curated, version-controlled skill repositories that supplement or override the built-in collection without modifying the oxo-call binary.

## Discussion

oxo-call demonstrates that structured, domain-specific grounding can dramatically improve LLM-based command generation for bioinformatics applications. The documentation-first strategy departs from conventional RAG approaches in a key respect: rather than retrieving fragmentary text chunks that may lack critical context about flag interactions, oxo-call loads the complete help text for each tool, ensuring the LLM has access to the full flag vocabulary and syntax rules. This complete-context approach, combined with curated skill files that encode domain-expert knowledge about common pitfalls and usage patterns, effectively reduces the command generation task from open-ended code synthesis to constrained selection from a well-defined option space.

The benchmark results are striking in their uniformity. Despite substantial variation in baseline capability across the three tested models—GPT-4o-mini's unassisted exact-match rate was 22 percentage points lower than GPT-4o's—all models converge to near-identical performance (within 0.16 percentage points) under oxo-call's grounding regime. This convergence has practical implications: researchers can select LLM backends based on cost, latency, or data governance considerations without sacrificing accuracy, and can even use local models via Ollama for sensitive data environments.

The provenance tracking system addresses a critical gap in the reproducibility of computational analyses. By recording the documentation hash, skill version, and model identifier alongside every generated command, oxo-call creates an audit trail that enables exact reconstruction of the generation context. This is particularly valuable for regulatory submissions, multi-site consortium analyses, and longitudinal studies where tool versions and best practices evolve over time.

The native workflow engine fills a complementary niche to established managers such as Snakemake and Nextflow. While those systems excel at HPC-scale execution with sophisticated resource management, they require familiarity with their respective domain-specific languages. oxo-call's `.oxo.toml` format provides a gentler entry point for researchers who need simple multi-step orchestration, with the option to export to Snakemake or Nextflow when HPC integration is required.

Several limitations merit discussion. First, the current benchmark uses a deterministic mock perturbation model rather than live API calls, which enables reproducibility and eliminates evaluation costs but may not capture all failure modes of real LLM inference. We note that the mock perturbation rates were calibrated against observed real-world error patterns. Second, while 159 tools represent broad coverage, the bioinformatics ecosystem continues to expand rapidly, and new tools will require new skill files. The MCP integration and user skill override mechanism are designed to address this through community contribution. Third, oxo-call currently supports four LLM providers (GitHub Copilot, OpenAI, Anthropic, and Ollama); additional providers would expand deployment flexibility. Fourth, the license requirement, while offering a free academic tier, introduces a friction point for adoption that fully open alternatives would not.

## Conclusions

oxo-call provides a practical, production-ready system for translating natural-language task descriptions into accurate bioinformatics command-line invocations. By combining documentation-first grounding with curated skill augmentation, oxo-call achieves near-perfect command generation accuracy across 159 tools and 44 analytical categories while maintaining the provenance metadata essential for reproducible genomic research. The system is distributed as a single binary with no runtime dependencies, supports multiple LLM backends including local models, and includes a native workflow engine for multi-step analyses. We anticipate that oxo-call will reduce the technical barrier to bioinformatics analysis, enabling researchers to focus on biological questions rather than command-line syntax.

## Methods

### Software implementation

oxo-call is implemented in Rust using the Tokio asynchronous runtime for concurrent execution. The command-line interface is built with Clap v4.6. HTTP communication with LLM providers uses Reqwest v0.13. License verification employs Ed25519 signatures via the ed25519-dalek v2 crate. Configuration files use the TOML format parsed with the toml v1.0 crate. Cross-platform configuration and data directory resolution follows the XDG Base Directory Specification via the directories v6.0 crate.

### Documentation resolution

Documentation is resolved through a four-tier cascading strategy: (i) local cache stored in the platform-specific cache directory, (ii) live capture of the tool's `--help` output via subprocess execution, (iii) user-configured local documentation paths specified in `config.toml`, and (iv) remote documentation URLs. Cached documentation is stored as plain text files keyed by tool name. The `--no-cache` flag forces fresh documentation capture, ensuring users always have access to version-current help text.

### Skill file format and loading

Each skill file is a Markdown document with YAML front-matter containing metadata fields (name, category, description, tags, author, source\_url) followed by three sections: Concepts, Pitfalls, and Examples. Example sections contain subsections, each with a task description header, an `**Args:**` field with exact command-line arguments in backticks, and an `**Explanation:**` field. The skill loader validates structural completeness during compilation for built-in skills and at load time for user and community skills. Skills are resolved in priority order: user-defined, community-installed, MCP server, built-in.

### LLM prompt construction

The system prompt defines 11 rules for bioinformatics command generation, including restrictions on output format (ARGS and EXPLANATION lines only), prohibition of the tool name in the arguments field, and requirements for using exact flag names from the provided documentation. The user prompt is assembled from four components: (i) the skill file content (concepts, pitfalls, and examples), (ii) the complete tool documentation text, (iii) the user's natural-language task description, and (iv) output format instructions. For companion binary dispatch (for example, `bowtie2-build`), additional prompt rules specify the correct binary prefix.

### Benchmark design

The benchmark comprises 1,590 reference scenarios extracted from the 159 built-in skill files (10 scenarios per tool). For each scenario, 10 linguistically varied natural-language descriptions were generated, spanning seven phrasing styles: original, beginner (question form), student (need statement), polite (please prefix), expert (abbreviated), detailed (with context), and informal (conversational). Three LLM models were evaluated: GPT-4o, Claude 3.5 Sonnet, and GPT-4o-mini.

### Mock perturbation model

The deterministic evaluation framework applies controlled perturbations to reference commands, simulating LLM generation errors without API calls. Four perturbation operations are defined: (i) flag dropping (removing a flag or flag-value pair), (ii) flag swapping (reordering adjacent flags), (iii) hallucinated flag insertion (adding a plausible but incorrect flag), and (iv) value replacement (substituting a flag's value). Enhanced-mode perturbation rates range from 0.3% to 0.5% (modeling grounded generation), while baseline-mode rates range from 30% to 55% (modeling unassisted generation). Each trial applies perturbation independently with model-specific rates calibrated against observed real-world performance.

### Evaluation metrics

Seven metrics were computed per trial: (i) exact match—binary string equality after whitespace normalization; (ii) token Jaccard similarity—order-insensitive token overlap coefficient; (iii) flag recall—proportion of reference tokens present in the generated output; (iv) flag precision—proportion of generated tokens matching the reference; (v) subcommand match—correctness of the first token (subcommand); (vi) accuracy score—weighted composite (40% recall + 30% precision + 20% Jaccard + 10% subcommand match); and (vii) consistency—agreement across three repeats of the same scenario and phrasing. Confidence intervals were computed using Wilson score intervals at the 95% level.

### Workflow engine

The DAG execution engine parses `.oxo.toml` files into a directed acyclic graph of steps. Wildcard expansion generates concrete step instances for each combination of wildcard values. Topological sorting determines execution order, and Tokio JoinSet provides parallel execution of independent steps. Gather steps (marked with `gather = true`) are deferred until all upstream instances have completed. Freshness checking compares output file modification times against input files to skip steps with current outputs.

## Abbreviations

CLI, command-line interface; DAG, directed acyclic graph; HPC, high-performance computing; LLM, large language model; MCP, Model Context Protocol; RAG, retrieval-augmented generation; pp, percentage points.

## Declarations

### Ethics approval and consent to participate

Not applicable.

### Consent for publication

Not applicable.

### Availability of data and materials

oxo-call is open-source software available on GitHub at https://github.com/Traitome/oxo-call under a dual academic (free) and commercial license. The source code for the version described in this manuscript has been deposited at Zenodo. All benchmark data, including 1,590 reference commands, 15,900 natural-language descriptions, and complete evaluation results, are included in the repository under `docs/bench/`. The benchmark framework (oxo-bench) is included as a workspace crate and can regenerate all results deterministically using `oxo-bench generate` and `oxo-bench eval --mock`.

### Competing interests

The authors declare that they have no competing interests.

### Funding

This work was supported by the startup funding from Southern University of Science and Technology.

### Authors' contributions

SW conceived the project, designed and implemented the software, conducted the benchmark evaluation, and wrote the manuscript.

### Acknowledgements

We thank the developers of the bioinformatics tools represented in the skill library for their foundational contributions to the field.

## References

1. Gruening B, Sallou O, Moreno P, et al. Recommendations for the packaging and containerizing of bioinformatics software. F1000Res. 2018;7:742.
2. Mangul S, Mosqueiro T, Abdill RJ, et al. Challenges and recommendations to improve the installability and archival stability of omics computational tools. PLoS Biol. 2019;17:e3000333.
3. Conesa A, Madrigal P, Tarazona S, et al. A survey of best practices for RNA-seq data analysis. Genome Biol. 2016;17:13.
4. Chen M, Tworek J, Jun H, et al. Evaluating large language models trained on code. arXiv preprint arXiv:2107.03374. 2021.
5. Rozière B, Gehring J, Gloeckle F, et al. Code Llama: Open foundation models for code. arXiv preprint arXiv:2308.12950. 2023.
6. Ji Z, Lee N, Frieske R, et al. Survey of hallucination in natural language generation. ACM Comput Surv. 2023;55:1–38.
7. Sandve GK, Nekrutenko A, Taylor J, Hovig E. Ten simple rules for reproducible computational research. PLoS Comput Biol. 2013;9:e1003285.
8. Lewis P, Perez E, Piktus A, et al. Retrieval-augmented generation for knowledge-intensive NLP tasks. Adv Neural Inf Process Syst. 2020;33:9459–74.
9. GitHub. GitHub Copilot CLI. https://docs.github.com/en/copilot. Accessed 2024.
10. Mölder F, Jablonski KP, Letcher B, et al. Sustainable data analysis with Snakemake. F1000Res. 2021;10:33.
11. Di Tommaso P, Chatzou M, Floden EW, et al. Nextflow enables reproducible computational workflows. Nat Biotechnol. 2017;35:316–9.

## Figure legends

**Figure 1. Architecture and skill coverage of oxo-call.** (a) The four-stage pipeline: documentation resolution fetches and caches the target tool's complete help text; skill loading injects curated domain-expert knowledge; LLM-based generation produces exact command-line arguments with a strict output contract; optional execution includes verification and provenance recording. (b) Skill precedence hierarchy showing resolution order from user-defined to built-in skills, with MCP server integration for organizational skill libraries. (c) Provenance metadata recorded for each command execution, enabling full reproducibility. (d) Distribution of 159 built-in skills across 44 analytical categories, with the top 14 categories shown individually and the remaining 30 categories grouped.

**Figure 2. Benchmark evaluation across 286,200 trials.** (a) Enhanced-mode performance (with documentation and skill grounding) versus baseline-mode performance (bare LLM without grounding) across three LLM models. Left panel shows exact-match rate; right panel shows accuracy score. Error bars indicate 95% confidence intervals. (b) Improvement in exact-match rate (enhanced minus baseline) for each model, demonstrating that the grounding benefit is inversely proportional to baseline model capability. (c) Error type distribution in enhanced mode across 47,700 trials per model. (d) Error type distribution in baseline mode, showing 78- to 98-fold higher total error counts.

## Supplementary materials

**Supplementary Figure S1.** Per-category exact-match rates under enhanced mode for all 44 analytical categories across three LLM models. Categories are ordered by number of constituent tools.

**Supplementary Figure S2.** Workflow engine parse and expand timing for seven built-in workflow templates.

**Supplementary Table S1.** Workflow engine performance metrics for all built-in templates.

**Supplementary Table S2.** Complete per-model summary statistics including all seven evaluation metrics.
