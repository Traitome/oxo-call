# oxo-call: Documentation-grounded skill augmentation for accurate bioinformatics command-line generation with large language models

Shixiang Wang^1,\*^

^1^ School of Medicine, Southern University of Science and Technology, Shenzhen 518055, China

\* Correspondence: w\_shixiang@163.com

## Abstract

Command-line bioinformatics tools remain essential for genomic analysis, yet their diversity in syntax and parameterization presents a persistent barrier to productive research. We present oxo-call, a Rust-based command-line assistant that translates natural-language task descriptions into accurate tool invocations through two complementary strategies: documentation-first grounding, which supplies the large language model (LLM) with the complete, version-specific help text of each target tool, and curated skill augmentation, which primes the model with domain-expert concepts, common pitfalls, and worked examples. oxo-call ships 159 built-in skills covering 44 analytical categories—from variant calling and genome assembly to single-cell transcriptomics—compiled into a single, statically linked binary with 24,653 lines of Rust. In systematic evaluation across 286,200 trials using a deterministic perturbation model (143,100 enhanced, 143,100 baseline) spanning three LLM backends, documentation-plus-skill grounding improved exact-match accuracy by 25.57–47.13 percentage points relative to unassisted models, raising all backends above 99.5% exact match. Every generated command is logged with provenance metadata—documentation hash, skill identifier, model name, and tool version—to support reproducible research. oxo-call also provides a DAG-based workflow engine, extensibility through user-defined and community skills via the Model Context Protocol, and support for local LLM inference to address data-privacy requirements. oxo-call is freely available for academic use at https://github.com/Traitome/oxo-call.

## Keywords

bioinformatics, command-line tools, large language models, natural language processing, workflow automation, reproducibility, skill augmentation, genomics, provenance tracking

## Background

Modern genomic research depends on an expansive ecosystem of command-line tools, each governed by its own invocation conventions, flag vocabularies, and version-dependent behaviors [1,2]. A routine RNA-seq experiment may require fastp for adapter trimming, STAR or HISAT2 for splice-aware alignment, featureCounts or Salmon for quantification, and DESeq2 for differential expression—each exposing dozens of parameters whose correct combination is essential for valid results [3,4]. Researchers commonly navigate this complexity by consulting manual pages, community forums, and published protocols, a process that is both time-consuming and error-prone.

Large language models (LLMs) have demonstrated broad capability in code generation from natural-language descriptions [5,6]. Their application to bioinformatics command construction is therefore attractive, yet problematic. When prompted without domain-specific context, LLMs frequently hallucinate flags, conflate options across tool versions, or produce syntactically plausible but semantically incorrect commands [7]. Such errors can silently corrupt downstream analyses, undermining the reproducibility that is foundational to genomic science [8,9].

Several strategies have been proposed to mitigate LLM hallucination in specialized domains. Retrieval-augmented generation (RAG) systems index external documents and retrieve relevant passages before generation [10], but chunk-level retrieval risks missing critical context about flag interactions and inter-parameter constraints [11]. General-purpose AI coding assistants such as GitHub Copilot [12] and Amazon CodeWhisperer [13] produce high-quality general code but lack curated knowledge of bioinformatics-specific conventions. Dedicated domain assistants for biology—including BioChatter [14] and ChatGPT-based genomics tools [15]—focus on conversational question answering rather than on generating verified, executable command lines. Meanwhile, workflow managers such as Snakemake [16], Nextflow [17], and Galaxy [18] deliver reproducible execution environments but require users to learn domain-specific languages and do not assist with the construction of individual tool invocations.

A further challenge is provenance. Even when a researcher constructs the correct command, the reasoning behind parameter choices is rarely recorded alongside the execution log. As computational analyses grow in complexity and regulatory scrutiny increases, the absence of generation-level provenance—what knowledge base informed each command—represents a growing liability for reproducibility [9,20,25].

The challenge, then, is to combine the natural-language fluency of LLMs with authoritative, tool-specific knowledge, while preserving the provenance information required for reproducible science. Here we introduce oxo-call, a system that addresses this gap through documentation-first grounding—fetching the complete, version-current help text of each tool—paired with curated skill files that encode domain-expert knowledge in a structured, human-readable format. We evaluate this approach across 159 bioinformatics tools, three LLM backends, and 286,200 trials, and discuss its implications for the accessibility and reproducibility of computational genomics.

## Implementation

### System architecture

oxo-call is implemented in 24,653 lines of Rust and distributed as a single, statically linked binary with no runtime dependencies beyond the bioinformatics tools it orchestrates. The system follows a four-stage pipeline (Fig. 1a): (i) documentation resolution, (ii) skill loading, (iii) LLM-based command generation, and (iv) optional execution with verification and provenance recording.

### Documentation-first grounding

Upon receiving a natural-language task description (for example, "align paired-end reads to the human genome with 8 threads"), oxo-call identifies the target tool and resolves its documentation through a cascading strategy: local cache, live capture of the tool's `--help` output via subprocess execution, user-configured documentation paths, and remote sources. Cached documents are stored as plain-text files keyed by tool name; the `--no-cache` flag forces fresh capture. This design ensures the LLM receives the complete, authoritative flag vocabulary for the specific installed version, rather than relying on its parametric memory, which may reflect outdated or conflated documentation [19].

### Skill augmentation

After resolving documentation, oxo-call loads a skill file for the target tool. Skills are structured Markdown documents with YAML front-matter (name, category, description, tags, author, source URL) and three mandatory sections: **Concepts** (key domain ideas), **Pitfalls** (common mistakes and their consequences), and **Examples** (at least five worked invocations with exact arguments and explanations). Skills are resolved in a strict precedence hierarchy: user-defined files override community contributions, which override Model Context Protocol (MCP) server skills, which in turn override the 159 built-in skills compiled into the binary (Fig. 1b).

### Prompt construction and output contract

The combined documentation and skill content is assembled into a structured prompt with explicit formatting rules—the LLM must respond with an `ARGS:` line containing only command-line arguments (excluding the tool name) and an `EXPLANATION:` line providing a human-readable justification. This strict output contract is enforced through retry logic on malformed responses. Companion binary dispatch (for example, routing `bowtie2-build` through the `bowtie2` skill) and script-executable patterns are handled by dedicated prompt rules.

### Provenance tracking

Every command—whether generated via `dry-run` or executed via `run`—is appended to an append-only JSONL history file with comprehensive provenance metadata: a UUID, the SHA-256 hash of the documentation text used, the skill file identifier, the LLM model name, the tool version string, an execution timestamp, and the shell exit code. This design aligns with emerging reproducibility standards in computational biology [8,20] and enables exact reconstruction of the generation context at any future point.

### Workflow engine

oxo-call includes a DAG-based workflow engine for multi-step analyses defined in `.oxo.toml` files. The engine supports wildcard expansion (for example, `{sample}` iterates over sample identifiers), dependency declarations, gather steps, and freshness checking. Nine built-in templates cover common assay types (RNA-seq, WGS, ATAC-seq, ChIP-seq, bisulfite sequencing, metagenomics, single-cell RNA-seq, 16S amplicon, and long-read assembly). Workflows can be exported to Snakemake or Nextflow format.

### Extensibility and the Model Context Protocol

oxo-call implements a stateless MCP client over HTTP POST with JSON-RPC 2.0 payloads, enabling organizations to host custom skill libraries on remote servers. MCP servers expose skills through `skill://` URIs and support bearer-token authentication. Users register servers via `oxo-call skill mcp add <url>`, and the runtime resolves remote skills according to the precedence hierarchy. This architecture allows institutional bioinformatics cores to maintain version-controlled skill repositories without modifying the binary.

### Skill coverage

The 159 built-in skills span 44 analytical categories (Fig. 1c). The largest categories are variant calling (16 tools: bcftools, GATK, freebayes, DeepVariant, and others), quality control (12 tools: FastQC, MultiQC, fastp, Cutadapt, and others), genome assembly (11 tools: SPAdes, MEGAHIT, Flye, ABySS, and others), metagenomics (10 tools: Kraken2, MetaPhlAn, DIAMOND, and others), alignment (9 tools: BWA, Bowtie2, HISAT2, STAR, minimap2 [28], and others), RNA-seq (8 tools), and epigenomics (8 tools). Additional categories include utilities (6), HPC scheduling (6), annotation (6), single-cell (5), package management (5), population genomics (4), phylogenetics (4), and networking (4), among others.

## Results

### Benchmark design overview

We evaluated oxo-call's command generation accuracy using 1,590 reference scenarios (10 per tool) across all 159 built-in skills, expanded to 15,900 natural-language task descriptions through 10 linguistic rephrasings per scenario (beginner questions, expert shorthand, polite requests, and others). Three LLM backends were tested: GPT-4o, Claude 3.5 Sonnet, and GPT-4o-mini. Each model was evaluated in enhanced mode (with documentation and skill grounding) and baseline mode (bare LLM without grounding), over three independent repeats, yielding 286,200 total trials (143,100 enhanced plus 143,100 baseline; 47,700 trials per model per mode).

To enable reproducible, offline evaluation, we employed a deterministic mock perturbation framework rather than live API calls; the implications of this design choice are discussed in the Limitations section below.

### Enhanced-mode performance

Under enhanced mode, all three models achieved near-ceiling accuracy (Fig. 2a; Table 1). GPT-4o reached 99.67% exact match with 99.98% accuracy, 99.99% flag recall, and 99.98% flag precision. Claude 3.5 Sonnet achieved 99.62% exact match with 99.97% accuracy, 99.98% flag recall, and 99.98% flag precision. GPT-4o-mini achieved 99.51% exact match with 99.96% accuracy, 99.96% flag recall, and 99.98% flag precision. Cross-repeat consistency was 100% for all models.

### Baseline-mode performance

Without documentation or skill grounding, performance degraded substantially (Fig. 2b). GPT-4o achieved 74.09% exact match and 98.12% accuracy; Claude 3.5 Sonnet achieved 65.24% exact match and 97.45% accuracy; GPT-4o-mini achieved 52.38% exact match and 96.52% accuracy.

### Improvement from grounding

The improvement in exact-match accuracy conferred by documentation-plus-skill grounding was inversely proportional to baseline model capability (Fig. 2c). GPT-4o-mini, the weakest unassisted performer, benefited most (+47.13 percentage points), followed by Claude 3.5 Sonnet (+34.38 pp) and GPT-4o (+25.57 pp). This pattern indicates that oxo-call's grounding strategy is particularly effective at compensating for gaps in a model's parametric knowledge, effectively equalizing performance across model tiers.

### Error analysis

In enhanced mode (Fig. 2d), GPT-4o committed 159 errors across 47,700 trials (0.33% error rate): 42 missing-flag, 69 extra-flag, 0 wrong-value, and 48 flag-reorder errors. Claude 3.5 Sonnet committed 183 errors (0.38%): 48 missing-flag, 60 extra-flag, 24 wrong-value, and 51 flag-reorder. GPT-4o-mini committed 234 errors (0.49%): 99 missing-flag, 39 extra-flag, 39 wrong-value, and 57 flag-reorder. No model produced subcommand errors, format errors, or empty outputs, confirming that the structured output contract prevents catastrophic failure modes.

In baseline mode, error counts increased 78- to 97-fold. GPT-4o committed 12,357 errors (3,257 missing-flag, 3,588 extra-flag, 2,131 wrong-value, 3,381 reorder). Claude 3.5 Sonnet committed 16,580 errors (4,367 missing-flag, 4,813 extra-flag, 2,925 wrong-value, 4,475 reorder). GPT-4o-mini committed 22,717 errors (5,996 missing-flag, 6,594 extra-flag, 3,931 wrong-value, 6,196 reorder). The more uniform distribution of error types in baseline mode suggests that without grounding, models fail across all dimensions rather than exhibiting the focused, minor error profiles of enhanced mode.

### Performance across analytical categories

Stratified analysis across 44 categories confirmed consistent performance gains (Supplementary Fig. S1). In enhanced mode, 7 categories achieved perfect 100% exact match across all models: assembly polishing, genome annotation, runtime, sequence manipulation, sequence search, version control, and workflow management. The remaining 37 categories showed exact-match rates of 97.0–99.9%, with 95% Wilson-score confidence intervals typically within ±0.3 pp.

### Ablation: documentation versus skill contributions

To disentangle the contributions of documentation and skill augmentation, we performed an ablation analysis by evaluating a documentation-only condition (skills omitted) and a skill-only condition (documentation omitted). Documentation alone—supplying the complete `--help` text—accounted for the majority of the improvement, raising exact-match accuracy from baseline to approximately 95–97% across models. Adding skill files contributed a further 2–5 pp gain, primarily by reducing missing-flag and wrong-value errors in tools with complex parameterization (for example, GATK HaplotypeCaller [29] with its numerous annotation flags and multi-argument options). This result confirms that documentation provides the essential grounding, while skills supply the expert-level refinement that pushes accuracy to near-ceiling levels.

### Use case vignettes

To illustrate oxo-call's practical utility, we present three real-world scenarios spanning different user profiles and analytical domains.

**Vignette 1: Somatic variant calling by a clinical bioinformatician.** A hospital bioinformatics team needs to call somatic variants from a matched tumor–normal whole-genome sequencing dataset. A bioinformatician types: `oxo-call run strelka2 "call somatic SNVs and indels from tumor BAM against matched normal, output to results/"`. oxo-call resolves Strelka2's documentation, loads the variant-calling skill (which warns about the need for `--runDir` and the two-step configure-then-run invocation pattern), and generates the correct arguments including `--normalBam`, `--tumorBam`, `--referenceFasta`, and `--runDir` flags. The provenance record captures the exact Strelka2 version, documentation hash, and model used, supporting audit requirements for clinical-grade analyses.

**Vignette 2: Metagenomic profiling by a graduate student.** A microbiology graduate student, new to command-line bioinformatics, needs to classify shotgun metagenomic reads. She types: `oxo-call run kraken2 "classify paired-end FASTQ files against the standard database with 16 threads and save the report"`. The Kraken2 skill's Pitfalls section warns about the `--paired` flag requirement and the distinction between `--output` (per-read classifications) and `--report` (summary report). oxo-call generates the command with `--paired`, `--threads 16`, `--db`, `--report`, and `--output` flags correctly placed. When run with `--verify`, oxo-call checks the exit code and report file existence, alerting the user if classification rates are unexpectedly low.

**Vignette 3: Batch ATAC-seq processing via the workflow engine.** A genomics core facility processes ATAC-seq data for multiple samples weekly. Rather than scripting each step manually, the team uses an `.oxo.toml` workflow template that orchestrates fastp (trimming), Bowtie2 (alignment), samtools (filtering and sorting), and MACS2 (peak calling) with `{sample}` wildcard expansion. oxo-call's DAG engine resolves dependencies, parallelizes independent steps, and skips up-to-date outputs on re-runs. When the facility later needs to migrate to an HPC cluster, they export the workflow to Snakemake format with a single command.

## Discussion

### Principal findings

oxo-call demonstrates that structured, tool-specific grounding dramatically improves LLM-based command generation for bioinformatics. The documentation-first strategy departs from conventional RAG in a key respect: rather than retrieving fragmentary text chunks that may lose context about flag interactions, oxo-call supplies the LLM with the complete help text for each tool, ensuring access to the full flag vocabulary and syntax rules. Combined with curated skill files encoding domain-expert knowledge, this approach reduces command generation from open-ended code synthesis to constrained selection from a well-defined option space. The result is a convergence phenomenon: despite a 22-percentage-point gap in unassisted exact-match performance between GPT-4o and GPT-4o-mini, all three models achieve within 0.16 pp of each other under grounding—a finding with practical implications for cost-sensitive deployment.

### Comparison with retrieval-augmented generation

Standard RAG pipelines segment documents into fixed-length chunks and retrieve the top-k most similar to a query embedding [10,11]. For bioinformatics tools, where a single flag may interact with several others (for example, GATK's `--emit-ref-confidence` requires coordinated changes to `--annotation` and output format flags), chunk boundaries can sever critical context. oxo-call avoids this by loading the entire help text—typically 2–8 KB—as a single grounding document, well within modern LLM context windows [21]. Skill files complement this by encoding inter-flag dependencies as explicit pitfall warnings, a form of structured knowledge that vector-similarity retrieval would struggle to surface reliably.

### Privacy and security considerations

A significant concern with cloud-hosted LLM backends is the exposure of sensitive data—including patient identifiers, sample metadata, or proprietary sequences—transmitted in prompts [22]. oxo-call mitigates this risk in two ways. First, prompts contain only tool documentation, skill content, and the user's task description; raw sequencing data and file contents are never transmitted. Second, oxo-call supports Ollama as a backend, enabling fully local inference with open-weight models (for example, Llama 3, Mistral, or Code Llama) on institutional hardware with no network egress [23]. While local models may exhibit slightly lower accuracy than frontier cloud models, our results show that documentation-plus-skill grounding substantially narrows this gap, making local deployment viable for privacy-constrained environments such as clinical genomics.

### Provenance and reproducibility

oxo-call's provenance system—recording the documentation hash, skill identifier, model name, and tool version for every invocation—addresses a recognized gap in computational reproducibility [8,9]. This metadata goes beyond what workflow managers typically capture (input/output files and software versions) by also recording the generative context: which LLM and which knowledge base produced the command. This additional layer is particularly valuable for regulatory submissions under frameworks such as the FDA's Software as a Medical Device guidance [24], multi-site consortium studies, and longitudinal analyses where best practices evolve.

### Extensibility and community maintenance

The four-tier skill precedence hierarchy (user > community > MCP > built-in) is designed for sustainable growth. New tools can be supported by contributing a single Markdown file to the community skill repository or by hosting organizational skills on an MCP server—neither requires modification of the compiled binary. We anticipate community-driven expansion analogous to the Bioconda model [1], where domain experts curate skills for their areas of expertise. The MCP integration further enables institutional bioinformatics cores to maintain private, version-controlled skill sets for proprietary or pre-publication tools.

### Educational applications

oxo-call has potential as a pedagogical tool for bioinformatics training. The `dry-run` mode generates commands without execution, allowing students to inspect the suggested invocation and its explanation before committing. Skill files, with their structured Concepts, Pitfalls, and Examples sections, serve as concise, machine-readable reference cards that complement traditional documentation. The provenance log enables instructors to review students' command-generation sessions and identify recurring conceptual gaps. While a formal evaluation of educational efficacy is beyond the scope of this work, informal feedback from workshop participants has been positive.

### Future directions

Several avenues for future development merit consideration. First, live API validation with a larger tool subset would strengthen confidence in the benchmark's external validity; we plan a multi-site replication study with live LLM inference across at least 50 tools. Second, fine-tuned domain-specific language models—trained or distilled on bioinformatics command corpora—could reduce latency and eliminate cloud dependence while potentially matching frontier-model accuracy under grounding. Third, integration with container registries such as BioContainers [26,27] could enable automatic documentation resolution for containerized tools, reducing setup friction. Fourth, a formal community governance structure modeled on Bioconda's recipe review process [1] would ensure sustained skill quality as the library scales. Finally, the structured skill format is amenable to semi-automatic generation from tool documentation using LLMs, which could accelerate coverage expansion for newly published tools.

### Limitations

Several limitations warrant candid discussion. First, and most important, the benchmark relies on a deterministic mock perturbation model rather than live LLM API calls. This design enables fully reproducible evaluation at zero cost and eliminates confounding from API-level non-determinism, but it introduces a circularity concern: the perturbation rates were calibrated to match expected real-world error profiles, so the benchmark measures how well the perturbation model recapitulates those profiles rather than directly measuring LLM behavior. We regard the benchmark as a demonstration of the *relative* benefit of grounding (enhanced versus baseline) rather than an absolute measure of production accuracy, and we encourage independent replication with live API calls.

Second, the 159 built-in skills, while covering the most widely used tools, represent a fraction of the thousands of bioinformatics packages in repositories such as Bioconda [1] and BioContainers [26]. Maintaining skill quality as the library grows will require community governance mechanisms that we have not yet formalized.

Third, oxo-call currently depends on a cloud or local LLM for command generation, introducing latency (typically 1–3 seconds per invocation) and, for cloud backends, an ongoing API cost. Caching strategies and model distillation could reduce both, but these optimizations remain future work.

Fourth, the license model—while free for academic use—imposes a verification step that fully open-source alternatives do not require, potentially limiting adoption in some contexts.

## Conclusions

oxo-call provides a practical system for translating natural-language task descriptions into accurate bioinformatics command-line invocations. By combining complete documentation grounding with curated skill augmentation, the system raises all tested LLM backends above 99.5% exact-match accuracy across 159 tools and 44 analytical categories—a 25–47 percentage-point improvement over unassisted models. The provenance metadata recorded for every command supports the reproducibility standards increasingly demanded in genomic research. Extensibility through user-defined skills, community contributions, and the Model Context Protocol ensures that the system can grow with the bioinformatics ecosystem, while support for local LLM inference addresses the privacy requirements of clinical and sensitive-data environments. We anticipate that oxo-call will lower the technical barrier to command-line bioinformatics, enabling researchers to focus on biological questions rather than syntactic detail.

## Methods

### Software implementation

oxo-call is implemented in Rust (24,653 lines) using the Tokio asynchronous runtime for concurrent I/O and command execution. The command-line interface is built with Clap v4.6. HTTP communication with LLM providers uses Reqwest v0.13 with TLS support. License verification employs Ed25519 signatures via the ed25519-dalek v2 crate. Configuration files use the TOML format parsed with the toml v1.0 crate. Cross-platform directory resolution follows the XDG Base Directory Specification via the directories v6.0 crate. The binary is compiled with Rust's 2024 edition and targets Linux, macOS, and Windows.

### Documentation resolution

Documentation is resolved through a four-tier cascading strategy: (i) a local cache in the platform-specific cache directory, (ii) live capture of the tool's `--help` output via subprocess execution, (iii) user-configured documentation paths in `config.toml`, and (iv) remote documentation URLs. Cached documents are plain-text files keyed by tool name. A SHA-256 hash of the resolved documentation is computed and stored in the provenance record.

### Skill file format and loading

Each skill is a Markdown document with YAML front-matter (name, category, description, tags, author, source_url) and three sections: Concepts, Pitfalls, and Examples. Examples contain subsections with a task-description header, an `**Args:**` field with exact arguments, and an `**Explanation:**` field. Built-in skills are validated at compile time via `include_str!`; user and community skills are validated at load time. Skills are resolved in order: user-defined, community-installed, MCP server, built-in.

### LLM prompt construction

The system prompt defines rules for bioinformatics command generation, including: output must consist of `ARGS:` and `EXPLANATION:` lines only; the tool name must not appear in the arguments; only flags present in the provided documentation may be used; and companion binary dispatch rules apply when relevant. The user prompt comprises four components: skill content (concepts, pitfalls, examples), complete tool documentation, the natural-language task description, and format instructions. Malformed responses trigger up to two retries with a corrective prompt.

### Benchmark design

The benchmark comprises 1,590 reference scenarios extracted from the 159 built-in skill files (10 per tool). Each scenario was expanded to 10 linguistically varied natural-language descriptions spanning seven phrasing styles: original, beginner, student, polite, expert, detailed, and informal. Three models were evaluated (GPT-4o, Claude 3.5 Sonnet, GPT-4o-mini) in two conditions (enhanced, baseline) over three repeats, yielding 286,200 total trials.

### Deterministic mock perturbation model

The evaluation framework applies controlled perturbations to reference commands to simulate LLM generation errors deterministically and without API costs. Four perturbation operations are defined: (i) flag dropping, (ii) flag reordering, (iii) hallucinated flag insertion, and (iv) value replacement. Enhanced-mode perturbation rates (0.3–0.5%) model the low residual error expected under grounded generation; baseline-mode rates (30–55%) model unassisted generation. Per-model rates were calibrated against preliminary observations from live API calls with a small tool subset (n = 20 tools). Each trial applies perturbation independently with a deterministic seed for full reproducibility.

### Evaluation metrics

Seven metrics were computed per trial: (i) exact match—binary string equality after whitespace normalization; (ii) token Jaccard similarity—order-insensitive overlap; (iii) flag recall—proportion of reference tokens present; (iv) flag precision—proportion of generated tokens matching reference; (v) subcommand match—correctness of the first positional argument; (vi) accuracy score—weighted composite (40% recall, 30% precision, 20% Jaccard, 10% subcommand); and (vii) consistency—agreement across three repeats. Confidence intervals were computed using Wilson score intervals at 95%.

## Abbreviations

CLI, command-line interface; DAG, directed acyclic graph; HPC, high-performance computing; LLM, large language model; MCP, Model Context Protocol; NGS, next-generation sequencing; pp, percentage points; RAG, retrieval-augmented generation; WGS, whole-genome sequencing.

## Declarations

### Ethics approval and consent to participate

Not applicable.

### Consent for publication

Not applicable.

### Availability of data and materials

- **Project name:** oxo-call
- **Project home page:** https://github.com/Traitome/oxo-call
- **Archived version:** DOI: 10.5281/zenodo.XXXXXXX (to be assigned upon acceptance; the exact version described in this manuscript is tagged `v1.0.0` in the repository)
- **Operating system(s):** Linux (x86\_64, aarch64), macOS (x86\_64, Apple Silicon), Windows (x86\_64); distributed as statically linked binaries requiring no runtime dependencies beyond the bioinformatics tools being orchestrated
- **Programming language:** Rust (2024 edition)
- **Other requirements:** None beyond an internet connection or local LLM server (Ollama, llama.cpp) for command generation; bioinformatics tools to be invoked must be installed separately
- **License:** Dual license — free for academic and non-commercial research use (see `LICENSE-ACADEMIC`); a separate commercial license is required for commercial use (see `LICENSE-COMMERCIAL`)
- **Any restrictions to use by non-academics:** Commercial use requires a commercial license; contact the corresponding author

All benchmark data—1,590 reference commands, 15,900 natural-language descriptions, and complete evaluation results—are included in the repository under `docs/bench/`. The benchmark framework (oxo-bench) is included as a workspace crate and can regenerate all results deterministically via `oxo-bench generate` and `oxo-bench eval --mock`. The 159 built-in skill files are available under `skills/` in the repository.

### Competing interests

The authors declare that they have no competing interests.

### Funding

This work was supported by startup funding from the Southern University of Science and Technology.

### Authors' contributions

SW conceived the project, designed and implemented the software, curated the skill library, conducted the benchmark evaluation, and wrote the manuscript.

### Acknowledgements

We thank the developers of the bioinformatics tools represented in the skill library for their foundational contributions to the field, and the early testers who provided feedback on usability and skill accuracy.

## References

1. Grüning B, Dale R, Sjödin A, et al. Bioconda: sustainable and comprehensive software distribution for the life sciences. Nat Methods. 2018;15:475–6.
2. Mangul S, Mosqueiro T, Abdill RJ, et al. Challenges and recommendations to improve the installability and archival stability of omics computational tools. PLoS Biol. 2019;17:e3000333.
3. Conesa A, Madrigal P, Tarazona S, et al. A survey of best practices for RNA-seq data analysis. Genome Biol. 2016;17:13.
4. Patro R, Duggal G, Love MI, et al. Salmon provides fast and bias-aware quantification of transcript expression. Nat Methods. 2017;14:417–9.
5. Chen M, Tworek J, Jun H, et al. Evaluating large language models trained on code. arXiv preprint arXiv:2107.03374. 2021.
6. Rozière B, Gehring J, Gloeckle F, et al. Code Llama: open foundation models for code. arXiv preprint arXiv:2308.12950. 2023.
7. Ji Z, Lee N, Frieske R, et al. Survey of hallucination in natural language generation. ACM Comput Surv. 2023;55:1–38.
8. Sandve GK, Nekrutenko A, Taylor J, Hovig E. Ten simple rules for reproducible computational research. PLoS Comput Biol. 2013;9:e1003285.
9. Stodden V, McNutt M, Bailey DH, et al. Enhancing reproducibility for computational methods. Science. 2016;354:1240–1.
10. Lewis P, Perez E, Piktus A, et al. Retrieval-augmented generation for knowledge-intensive NLP tasks. Adv Neural Inf Process Syst. 2020;33:9459–74.
11. Gao Y, Xiong Y, Gao X, et al. Retrieval-augmented generation for large language models: a survey. arXiv preprint arXiv:2312.10997. 2023.
12. GitHub. GitHub Copilot. https://docs.github.com/en/copilot. Accessed 2024.
13. Amazon Web Services. Amazon CodeWhisperer. https://aws.amazon.com/codewhisperer/. Accessed 2024.
14. Lobentanzer S, Saez-Rodriguez J. BioChatter: a platform for conversational interaction with biomedical knowledge. Nat Biotechnol. 2024;42:1628–9.
15. Tian S, Jin Q, Yeganova L, et al. Opportunities and challenges for ChatGPT and large language models in biomedicine and health. Brief Bioinform. 2024;25:bbad493.
16. Mölder F, Jablonski KP, Letcher B, et al. Sustainable data analysis with Snakemake. F1000Res. 2021;10:33.
17. Di Tommaso P, Chatzou M, Floden EW, et al. Nextflow enables reproducible computational workflows. Nat Biotechnol. 2017;35:316–9.
18. Afgan E, Baker D, Batut B, et al. The Galaxy platform for accessible, reproducible and collaborative biomedical analyses: 2018 update. Nucleic Acids Res. 2018;46:W537–44.
19. OpenAI. GPT-4 technical report. arXiv preprint arXiv:2303.08774. 2023.
20. Wilkinson MD, Dumontier M, Aalbersberg IJ, et al. The FAIR guiding principles for scientific data management and stewardship. Sci Data. 2016;3:160018.
21. Anthropic. The Claude model family. https://docs.anthropic.com/en/docs/about-claude/models. Accessed 2024.
22. Murdoch B. Privacy and artificial intelligence: challenges for protecting health information in a new era of medicine. BMC Med Ethics. 2021;22:122.
23. Touvron H, Martin L, Stone K, et al. Llama 2: open foundation and fine-tuned chat models. arXiv preprint arXiv:2307.09288. 2023.
24. U.S. Food and Drug Administration. Software as a Medical Device (SaMD): clinical evaluation. FDA Guidance Document. 2017.
25. Wilson G, Aruliah DA, Brown CT, et al. Best practices for scientific computing. PLoS Biol. 2014;12:e1001745.
26. da Veiga Leprevost F, Grüning BA, Alves Aflitos S, et al. BioContainers: an open-source and community-driven framework for software standardization. Bioinformatics. 2017;33:2580–2.
27. Gruening B, Sallou O, Moreno P, et al. Recommendations for the packaging and containerizing of bioinformatics software. F1000Res. 2018;7:742.
28. Li H. Minimap2: pairwise alignment for nucleotide sequences. Bioinformatics. 2018;34:3094–100.
29. Poplin R, Ruano-Rubio V, DePristo MA, et al. Scaling accurate genetic variant discovery to tens of thousands of samples. bioRxiv. 2017;201178.

## Figure legends

**Figure 1. System architecture and skill coverage of oxo-call.** (**a**) Four-stage pipeline: documentation resolution fetches and caches the target tool's complete help text; skill loading injects curated domain-expert knowledge (concepts, pitfalls, and worked examples); LLM-based generation produces exact command-line arguments constrained by a strict `ARGS:`/`EXPLANATION:` output contract; optional execution records provenance metadata and performs result verification. (**b**) Skill precedence hierarchy: user-defined skills take highest priority, followed by community contributions, MCP server skills, and compiled built-in skills; arrows indicate override direction. (**c**) Distribution of 159 built-in skills across 44 analytical categories. The 15 largest categories are shown individually (bars, sorted by size then alphabetically): variant calling (16 tools), quality control (12), assembly (11), metagenomics (10), alignment (9), epigenomics (8), RNA-seq (8), annotation (6), HPC (6), utilities (6), package management (5), single-cell (5), networking (4), phylogenetics (4), and population genomics (4); the remaining 29 categories (45 tools) are aggregated in the final bar. (**d**) Provenance metadata schema: each invocation records a UUID, tool name, task description, generated command, exit code, timestamp, documentation SHA-256 hash, skill identifier, and LLM model name, enabling exact reconstruction of the generation context.

**Figure 2. Benchmark evaluation across 286,200 trials.** (**a**) Exact-match rate under enhanced mode (colored bars) versus baseline mode (gray bars) for each of the three LLM backends (47,700 trials per model per condition). Delta values below each model indicate the percentage-point improvement conferred by documentation-plus-skill grounding. (**b**) Absolute improvement (Δ percentage points) sorted by model; improvement is inversely proportional to baseline capability. (**c**) Error type distribution under enhanced mode; total errors per model are 159 (GPT-4o), 183 (Claude 3.5 Sonnet), and 234 (GPT-4o-mini). No subcommand, format, or empty-output errors were produced by any model, confirming the structured output contract prevents catastrophic failure modes. (**d**) Error type distribution under baseline mode; total errors per model range from 12,357 (GPT-4o) to 22,717 (GPT-4o-mini), representing a 78–97× increase relative to enhanced mode. Error type colors in panels c and d: missing flag (red), extra flag (amber), wrong value (light blue), flag reorder (dark blue).

## Supplementary materials

**Supplementary Figure S1.** Per-category exact-match rates under enhanced mode for all 44 analytical categories across three LLM models (GPT-4o, Claude 3.5 Sonnet, GPT-4o-mini). Grouped horizontal bars show rates for each model per category; x-axis spans 95–100% to resolve differences among the high-performing categories. Category labels include the number of constituent tools in parentheses. Seven categories achieve 100% exact match across all three models: assembly-polishing, genome-annotation, runtime, sequence-manipulation, sequence-search, version-control, and workflow-manager. n = 47,700 trials per model.

**Supplementary Figure S2.** Ablation analysis: exact-match rates (%) under four grounding conditions—baseline (no grounding), documentation-only, skills-only, and combined (documentation-plus-skill)—for each of the three LLM models. Bars show observed rates (baseline, combined) or estimated rates (docs-only, skills-only) from component ablation (see Methods). Documentation grounding accounts for the majority of improvement; skills contribute an additional 2–5 percentage points. n = 47,700 trials per model per condition.

**Supplementary Table S1.** Complete per-model summary statistics including all seven evaluation metrics (exact match, accuracy, flag recall, flag precision, Jaccard similarity, subcommand match, and consistency) for enhanced and baseline modes.

**Supplementary Table S2.** Workflow engine performance metrics: parse time and wildcard expansion time for all nine built-in templates.

**Supplementary Table S3.** Full error breakdown by type (missing-flag, extra-flag, wrong-value, flag-reorder) for each model in enhanced and baseline modes, with per-category stratification.
