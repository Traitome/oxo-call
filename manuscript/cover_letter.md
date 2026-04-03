Dear Editors,

We are pleased to submit our manuscript entitled "oxo-call: Documentation-grounded skill augmentation for accurate bioinformatics command-line generation with large language models" for consideration as a Software article in *Genome Biology*.

**The problem.** Bioinformatics research relies on hundreds of command-line tools, each with distinct flag vocabularies and version-specific behaviors. Large language models (LLMs) can generate command-line invocations from natural language, yet without domain-specific grounding they frequently hallucinate flags or confuse options across tool versions—errors that silently corrupt downstream analyses.

**Our solution.** oxo-call is a Rust-based command-line assistant that addresses this challenge through two complementary strategies: (i) *documentation-first grounding*, which supplies the LLM with the complete, version-current help text of each target tool rather than fragmentary RAG chunks; and (ii) *curated skill augmentation*, which primes the model with domain-expert concepts, common pitfalls, and worked examples via 159 hand-curated skill files spanning 44 analytical categories. Every generated command is logged with comprehensive provenance metadata—including documentation hash, skill identifier, and model name—supporting the reproducibility standards increasingly demanded in genomic research.

**Key results.** In systematic evaluation across 286,200 deterministic trials (143,100 enhanced, 143,100 baseline) spanning three LLM backends (GPT-4o, Claude 3.5 Sonnet, GPT-4o-mini), documentation-plus-skill grounding improved exact-match accuracy by 25–47 percentage points relative to unassisted models, raising all backends above 99.5% exact match. Error analysis reveals that grounding reduces total errors by 78–97-fold compared to bare LLM generation.

**Why *Genome Biology*.** We believe this work is well suited for the Software category in *Genome Biology* for the following reasons:

1. **Broad utility.** oxo-call covers 159 tools across 44 analytical domains—from variant calling and genome assembly to single-cell transcriptomics and metagenomics—serving researchers working with RNA-seq, WGS, ATAC-seq, ChIP-seq, bisulfite sequencing, and other assay types.

2. **Principled advance.** Rather than applying retrieval-augmented generation at the chunk level, oxo-call loads the complete tool documentation alongside structured skill files, combining authoritative flag vocabularies with expert knowledge. This documentation-first strategy represents a qualitative shift from unreliable to near-perfect command construction.

3. **Reproducibility.** Provenance metadata recorded for every invocation enables exact reconstruction of the generation context, addressing a gap in current computational reproducibility practices.

4. **Practical deployment.** oxo-call is distributed as a single statically linked binary with no runtime dependencies. It supports multiple LLM backends, including local models (Ollama, llama.cpp) for privacy-sensitive environments, and includes a native DAG workflow engine with Snakemake and Nextflow export.

5. **Extensibility.** Users can create custom skills, share them through the community repository, or host organizational skill libraries via the Model Context Protocol (MCP). This architecture ensures the system grows with the bioinformatics ecosystem.

6. **Open availability.** The software, all 159 skill files, the complete benchmark dataset (1,590 reference scenarios, 15,900 natural-language descriptions), and the evaluation framework are publicly available at https://github.com/Traitome/oxo-call.

The manuscript has not been published elsewhere and is not under consideration at another journal. All authors have read and approved the manuscript.

We appreciate your consideration of this work and look forward to your response.

Sincerely,

Shixiang Wang, Ph.D.
School of Medicine
Southern University of Science and Technology
Shenzhen 518055, China
Email: w_shixiang@163.com
