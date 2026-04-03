Dear Editors,

We are pleased to submit our manuscript entitled "oxo-call: AI-powered command-line orchestration for bioinformatics through documentation-grounded skill augmentation" for consideration as a Software article in *Genome Biology*.

Bioinformatics analyses depend on hundreds of specialized command-line tools, each with idiosyncratic syntax, flag conventions, and version-specific behaviors. While large language models have shown promise for code generation, their direct application to bioinformatics command construction suffers from flag hallucination and cross-version confusion—errors that can silently corrupt downstream analyses.

oxo-call addresses this challenge through a documentation-first grounding strategy combined with curated domain-expert skill files. Rather than retrieving fragmentary text chunks as in conventional RAG systems, oxo-call loads the complete help text of each target tool, ensuring the LLM receives authoritative, version-specific information. This documentation is supplemented by 159 hand-curated skill files spanning 44 analytical categories—from variant calling and genome assembly to single-cell transcriptomics and metagenomics—each encoding concepts, common pitfalls, and worked examples distilled from domain expertise.

We believe this manuscript is well-suited for the Software category in *Genome Biology* for the following reasons:

1. **Broad utility across genomics.** oxo-call covers 159 tools across 44 analytical domains, serving researchers working with diverse assay types including RNA-seq, whole-genome sequencing, ATAC-seq, ChIP-seq, bisulfite sequencing, metagenomics, and single-cell technologies.

2. **Clear advance over existing approaches.** Systematic evaluation across 286,200 trials demonstrates that documentation-first grounding with skill augmentation improves exact-match accuracy by 25–47 percentage points over unassisted LLMs, effectively equalizing performance across model tiers. This represents a qualitative shift from unreliable generation to near-perfect command construction.

3. **Reproducibility and provenance.** Every generated command is recorded with comprehensive metadata including documentation hash, skill version, and model identifier—addressing the reproducibility demands of genomic research.

4. **Practical deployment.** oxo-call is distributed as a single statically linked binary requiring no runtime dependencies, supports multiple LLM backends including local models for data-sensitive environments, and includes a native DAG workflow engine with Snakemake and Nextflow export.

5. **Open source.** The software, all 159 skill files, the complete benchmark dataset, and the evaluation framework are publicly available on GitHub.

The manuscript has not been published elsewhere and is not under consideration at another journal. All authors have read and approved the manuscript.

We appreciate your consideration of this work and look forward to your response.

Sincerely,

Shixiang Wang, Ph.D.
School of Medicine
Southern University of Science and Technology
Shenzhen 518055, China
Email: w_shixiang@163.com
