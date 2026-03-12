Dear Editors,

We are pleased to submit our manuscript entitled "oxo-call: documentation-grounded, skill-augmented command generation for bioinformatics" for consideration as a Software article in *Genome Biology*.

Bioinformatics analyses critically depend on command-line tools, yet the ecosystem presents a persistent usability challenge: researchers must navigate hundreds of tools, each with its own parameter syntax, undocumented conventions, and version-specific behaviours. While large language models (LLMs) show promise for code generation, direct prompting alone produces unreliable commands—models may hallucinate flags or conflate syntax between tools and versions. This reliability gap has limited the practical adoption of LLM-based approaches in production genomics workflows.

oxo-call addresses this gap through a principled, three-component design. First, an automatic documentation layer fetches, caches, and deduplicates each tool's `--help` output and optionally remote documentation, ensuring that the LLM always sees the flags available in the user's locally installed version. Second, a curated skill system provides structured domain expertise—concepts, common pitfalls, and worked examples—for 119 bioinformatics tools across 14 analytical categories, dramatically improving generation accuracy even with smaller LLMs. Third, a strict output protocol with retry logic and full command provenance (tool version, documentation hash, skill name, model identifier) ensures both reliability and reproducibility.

Beyond single-command generation, oxo-call ships a native Rust DAG-based workflow engine that executes multi-step pipelines from a simple TOML format, with nine built-in templates (RNA-seq, WGS, ATAC-seq, and others) and compatibility export to Snakemake and Nextflow. The tool requires no runtime dependencies beyond the bioinformatics tools themselves and compiles to a static binary for Linux, macOS, and Windows.

We believe oxo-call is well suited for *Genome Biology*'s Software category for several reasons:

1. **Broad utility.** The tool covers the major analytical domains of genomics, transcriptomics, epigenomics, metagenomics, single-cell biology, and assembly—spanning the readership of *Genome Biology*.

2. **Clear advance over existing approaches.** Unlike general-purpose LLM chat interfaces, oxo-call provides structured grounding in tool documentation and domain knowledge, with measurable accuracy improvements demonstrated through a systematic benchmark suite of 70+ evaluation tasks.

3. **Open and reproducible.** The source code, all 119 skill files, workflow templates, and the complete benchmark suite are publicly available on GitHub (https://github.com/Traitome/oxo-call). The tool is published on crates.io for straightforward installation.

4. **Community-extensible design.** The skill system is deliberately designed for community contribution: any researcher can write a TOML file encoding their expertise about a tool, without modifying any Rust code.

The software is freely available for academic use under a dual-license model. We confirm that this manuscript has not been published elsewhere and is not under consideration by another journal.

We suggest the following potential reviewers with relevant expertise in bioinformatics tool development and workflow systems:

- [Reviewer suggestions to be added by authors]

Thank you for your consideration. We look forward to hearing from you.

Sincerely,

Shixiang Wang
Traitome, Shenzhen, China
Email: w_shixiang@163.com
