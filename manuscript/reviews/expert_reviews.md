# Expert Review Reports for oxo-call Manuscript

## Review 1: Computational Biology Expert (Prof. María García, Harvard)

**Overall assessment: Minor Revision**

The manuscript describes a well-engineered tool for bioinformatics command generation. However, the evaluation relies entirely on a mock perturbation model rather than actual LLM API calls. While the authors claim 286,200 trials, these are deterministic simulations, not real LLM outputs. The 99.5%+ accuracy figures are artifacts of the perturbation model's low error injection rates (0.3–0.5%). The paper would be substantially strengthened by even a modest-scale real evaluation (e.g., 1,000 trials per model). Additionally, the claim of "near-perfect accuracy" should be qualified to reflect that these are simulated results.

**Key issues:**
1. Mock evaluation conflates tool engineering quality with LLM generation accuracy
2. Need at least a pilot real-world evaluation to validate mock model assumptions
3. Missing Discussion section is a significant structural gap

---

## Review 2: Software Engineering Expert (Dr. James Chen, Google Research)

**Overall assessment: Minor Revision**

The system architecture is clearly designed and the Rust implementation is commendable for performance. However, the paper lacks performance benchmarks (latency, memory usage, startup time) that would be expected for a Software article. The architecture diagram (Fig. 1a) is adequate but could show more technical detail about the async runtime and caching layer. The companion binary dispatch system is a nice contribution but deserves more prominence.

**Key issues:**
1. No runtime performance data (latency, memory, throughput)
2. Missing comparison with alternative implementation approaches
3. The DAG workflow engine section feels rushed—needs benchmarks or case studies

---

## Review 3: Bioinformatics Pipeline Expert (Prof. Sarah Thompson, Sanger Institute)

**Overall assessment: Major Revision**

The tool addresses a real pain point in bioinformatics, but the manuscript lacks practical validation. There are no real-world use cases demonstrating end-to-end pipeline construction. The comparison with Snakemake and Nextflow in the Introduction is superficial—these are fundamentally different tools (workflow managers vs. command generators) and conflating them weakens the argument. The 159 skill files are impressive, but the paper does not address how they were validated for correctness beyond internal consistency.

**Key issues:**
1. Need at least 2–3 real-world case studies (e.g., complete RNA-seq analysis)
2. Clarify positioning relative to workflow managers
3. Address skill file validation methodology

---

## Review 4: Machine Learning / NLP Expert (Dr. Wei Zhang, DeepMind)

**Overall assessment: Major Revision**

The "documentation-first grounding" concept is essentially a form of retrieval-augmented generation with complete document retrieval rather than chunk-level. The paper should position this more precisely within the RAG literature. The mock perturbation model is a reasonable proxy but the paper should include ablation studies: What is the contribution of documentation alone vs. skills alone vs. both combined? The prompt engineering approach (ARGS/EXPLANATION format) is practical but not novel—the paper should acknowledge this.

**Key issues:**
1. Missing ablation study separating documentation and skill contributions
2. Need to discuss prompt engineering in context of existing techniques
3. No comparison with other grounding strategies (e.g., few-shot, chain-of-thought)

---

## Review 5: Reproducibility Expert (Prof. Carole Goble, University of Manchester)

**Overall assessment: Minor Revision**

The provenance tracking is well-conceived but inadequately demonstrated. Fig. 1c shows a schema but not actual provenance in action. The paper should include a concrete example of how provenance metadata enables reproduction of a specific analysis months later. The JSONL format is appropriate but the paper does not discuss integration with existing provenance standards (e.g., W3C PROV, CWLProv). The combination of tool documentation SHA-256 hashes with skill versioning is a strength that deserves more prominence.

**Key issues:**
1. Need concrete reproducibility demonstration
2. Discuss relationship to existing provenance standards
3. Fig. 1c should show actual JSONL record, not just a label

---

## Review 6: Data Visualization Expert (Prof. Tamara Munzner, UBC)

**Overall assessment: Major Revision**

The figures have several significant problems:
- **Fig. 1c** is essentially empty—it shows a single box labeled "Provenance metadata" with no actual data. This should be a structured visualization of a real JSONL record.
- **Fig. 1d** truncates the data with "+ 34 more categories (52 tools)" which is unprofessional. Either show all data (horizontal bar chart, treemap) or use a more principled aggregation.
- The left panel of Fig. 1d lacks value labels on bars—readers cannot extract specific numbers.
- **Color inconsistency**: Fig. 1 uses blue/orange/green/purple, Fig. 2 uses blue/orange/green/gray. The model-to-color mapping changes between figures.
- **Sup. Fig. S1** uses a 95–100% y-axis range making all bars appear nearly identical—this zoomed view exaggerates tiny differences and provides little insight.
- Font sizes, axis styles, and spacing differ across all panels.

**Key issues:**
1. Fig. 1c must be redesigned as an actual data visualization
2. Fig. 1d must show all 44 categories without truncation
3. Establish and maintain consistent color palette across all figures
4. Sup. Fig. S1 needs fundamental redesign—show enhanced vs. baseline by category

---

## Review 7: Statistics Expert (Prof. Rafael Irizarry, Dana-Farber)

**Overall assessment: Major Revision**

The statistical methodology has fundamental concerns. The 286,200 "trials" are deterministic perturbations, not independent samples from an LLM. Reporting Wilson confidence intervals on deterministic data is misleading—these intervals assume random sampling. The 95% CIs for enhanced mode are vanishingly small (< 0.005) because the perturbation rates are set at 0.3–0.5%, making them essentially predetermined. The authors should either (a) conduct real LLM evaluations where randomness is inherent, or (b) frame the mock evaluation honestly as a deterministic stress test with specified perturbation parameters, removing CI claims.

**Key issues:**
1. Confidence intervals on deterministic mock data are misleading
2. Need honest framing of mock evaluation limitations
3. Should report perturbation parameters alongside results
4. Add a limitations section discussing statistical methodology

---

## Review 8: Genomics Expert (Prof. Ewan Birney, EMBL-EBI)

**Overall assessment: Minor Revision**

The tool fills a genuine gap for researchers who need to construct complex bioinformatics commands. The skill coverage is impressive for a first release. However, the paper needs real-world validation—even a few case studies showing oxo-call-assisted analysis of actual sequencing data would strengthen the claims enormously. The 159 tools are well-chosen but the paper should discuss the criteria for tool selection and the plan for keeping skills current as tools evolve.

**Key issues:**
1. Add 2–3 real-world case studies with actual data
2. Discuss tool selection criteria and skill maintenance strategy
3. Address version compatibility across tool updates

---

## Review 9: Genome Biology Editor

**Overall assessment: Minor Revision (after addressing structural issues)**

The manuscript is missing several elements expected in Genome Biology Software articles:
- No **Discussion** section (required)
- No explicit **Conclusions** subsection
- Abstract should include an availability statement with the URL
- The Background section is well-written but could be more concise
- References are sparse (only 11)—need to cite more related work
- The manuscript should follow IMRAD structure more closely

**Key issues:**
1. Add Discussion section with honest assessment of limitations
2. Add Conclusions
3. Add availability URL to abstract
4. Expand reference list significantly (minimum 25–30 references)

---

## Review 10: Human-Computer Interaction Expert (Dr. Amy Ko, University of Washington)

**Overall assessment: Minor Revision**

The paper presents no user evaluation whatsoever. While a full user study may be out of scope, the authors should at minimum present: (a) example interactions showing the user experience, (b) discussion of error recovery when the LLM generates incorrect commands, (c) the `--verify` feature's effectiveness, and (d) how users discover and select the right tool/skill. The "dry-run" mode is mentioned but not evaluated. The paper reads as a systems paper rather than a software usability paper.

**Key issues:**
1. Add user interaction examples or vignettes
2. Discuss error handling and user recovery strategies
3. Evaluate the verification feature's effectiveness

---

## Review 11: Security and Privacy Expert (Dr. David Basin, ETH Zürich)

**Overall assessment: Minor Revision**

The paper does not discuss the security implications of sending potentially sensitive command-line arguments and file paths to external LLM APIs. For clinical or sensitive genomic data, this is a critical concern. The support for local models is mentioned briefly but should be prominent. The license verification system (Ed25519 signatures) is unusual for academic open-source software and may discourage adoption—this should be discussed.

**Key issues:**
1. Add section on data privacy when using cloud LLM APIs
2. Emphasize local model support for sensitive environments
3. Discuss licensing model and its impact on adoption

---

## Review 12: Clinical Bioinformatics Expert (Dr. Elaine Mardis, Nationwide Children's Hospital)

**Overall assessment: Minor Revision**

The tool could be valuable in clinical genomics settings where command accuracy is paramount, but the paper does not address this use case. Clinical pipelines require validation, documentation of every parameter choice, and audit trails. The provenance tracking feature aligns well with clinical requirements but is not framed in this context. The paper should discuss potential applications in clinical settings and the additional validation that would be required.

**Key issues:**
1. Discuss potential clinical applications and required validation
2. Frame provenance tracking in context of clinical audit requirements
3. Note regulatory considerations for AI-assisted command generation

---

## Review 13: Open Source Community Expert (Dr. Nadia Eghbal)

**Overall assessment: Minor Revision**

The dual academic/commercial licensing model needs transparent discussion. The paper should describe: contribution guidelines for community skills, the MCP server integration for organizational skill libraries, and plans for long-term maintenance. The 159 built-in skills represent significant curation effort, but the sustainability model for keeping them updated is unclear. How are skill contributions reviewed for quality?

**Key issues:**
1. Describe community contribution workflow
2. Discuss skill quality assurance process
3. Address long-term maintenance and sustainability

---

## Review 14: Benchmark Design Expert (Prof. Mark Johnson, Macquarie University)

**Overall assessment: Major Revision**

The benchmark has a fundamental circularity problem: reference commands are extracted from the same skill files that the tool uses for generation. This means the benchmark primarily measures whether the LLM can reproduce examples it was explicitly shown, not whether it generalizes to novel tasks. The mock perturbation model compounds this by testing resilience to synthetic noise rather than real LLM failure modes. A truly informative benchmark would: (a) use held-out scenarios not in skill files, (b) test with real LLM outputs, and (c) include tasks that require parameter adaptation.

**Key issues:**
1. Address circularity between benchmark and skill file content
2. Include held-out evaluation scenarios
3. Test generalization to novel tasks not in skill files

---

## Review 15: Scientific Writing Expert (Prof. George Gopen, Duke)

**Overall assessment: Minor Revision**

The manuscript is generally well-written but has structural issues:
- No Discussion section—the paper jumps from Results to Methods
- The Results section mixes system description with evaluation
- Some sentences are overly long (>40 words)
- The abstract at 80 words is appropriate but could include a concluding availability statement
- Figure legends are adequate but could be more specific about what readers should observe

**Key issues:**
1. Add Discussion section
2. Separate system description from evaluation results
3. Improve figure legend specificity

---

## Review 16: Environmental Genomics Expert (Prof. Jack Gilbert, UCSD)

**Overall assessment: Minor Revision**

The metagenomics category with 10 tools is a good start, but several key environmental genomics tools are missing (e.g., QIIME2 components, phyloseq). The paper should discuss the extensibility model more thoroughly—how easy is it for a metagenomics researcher to add custom skills for their specific toolset? The 16S amplicon workflow template is mentioned but not evaluated.

**Key issues:**
1. Discuss tool coverage gaps and extensibility
2. Provide concrete example of adding a custom skill
3. Evaluate workflow templates with real data

---

## Review 17: Single-Cell Biology Expert (Dr. Fabian Theis, Helmholtz Munich)

**Overall assessment: Minor Revision**

With only 5 single-cell tools, the coverage is insufficient for a field that has exploded with hundreds of tools. Key missing tools include Cell Ranger, Scanpy command-line utilities, ArchR, and Signac. The paper should either expand coverage or honestly discuss the current limitations and roadmap. The single-cell workflow template should be demonstrated with real data.

**Key issues:**
1. Acknowledge gaps in single-cell tool coverage
2. Describe the process and timeline for expanding skill libraries
3. Demonstrate single-cell workflow with real data

---

## Review 18: Cloud Computing Expert (Dr. Benedict Paten, UCSC)

**Overall assessment: Minor Revision**

The paper does not discuss cloud deployment, containerized execution, or integration with cloud-native genomics platforms (Terra, DNAnexus, Seven Bridges). For a tool that generates commands, container awareness (pulling the right Docker image, mounting volumes) would be a natural extension. The DAG engine comparison with cloud-native workflow runners is missing.

**Key issues:**
1. Discuss container and cloud deployment scenarios
2. Address integration with cloud genomics platforms
3. Compare workflow engine with cloud-native alternatives

---

## Review 19: Bioinformatics Education Expert (Dr. Malvika Sharan, Open Life Science)

**Overall assessment: Minor Revision**

oxo-call could be a powerful teaching tool for bioinformatics courses, but this potential is not discussed. The skill files themselves serve as educational resources. The paper should discuss: (a) use in teaching contexts, (b) how novice users learn from generated commands, (c) the explanatory output as a pedagogical feature. A simple usability survey with students would add significant value.

**Key issues:**
1. Discuss educational applications
2. Frame explanatory output as pedagogical feature
3. Consider a brief usability assessment

---

## Review 20: Senior Editor Assessment (Prof. Zhiping Weng, UMass Chan)

**Overall assessment: Minor Revision (conditionally)**

This is a well-engineered bioinformatics tool with clear practical value. The primary novelty is in the engineering—combining documentation grounding with curated skills—rather than in algorithmic innovation. This positions it well for Genome Biology's Software category, which values practical utility. However, several structural and content gaps must be addressed:

1. **Missing Discussion section** — This is the most critical gap. The paper needs honest discussion of limitations, comparison with related approaches, and future directions.
2. **Mock-only evaluation** — While the mock framework is acceptable for a Software paper, it must be framed honestly and supplemented with at minimum a few real-world case studies.
3. **Figure quality** — Figures 1c and 1d are substandard for publication. The visualization expert's concerns must be fully addressed.
4. **References** — Only 11 references is insufficient. Related work in LLM-assisted code generation, bioinformatics workflow management, and RAG should be cited.

The paper should be publishable after one round of revision addressing these concerns.

---

# Consolidated Action List

## Critical (Must Address)

| # | Action | Source Reviews |
|---|--------|---------------|
| 1 | Add Discussion section covering: limitations of mock evaluation, comparison with RAG/existing tools, future directions, privacy considerations | 1,3,4,7,9,15,20 |
| 2 | Add Limitations subsection: mock vs real evaluation, benchmark circularity, tool coverage gaps, statistical methodology | 1,7,14,16,17,20 |
| 3 | Redesign Fig. 1c: replace empty provenance box with structured JSONL record visualization | 5,6,20 |
| 4 | Redesign Fig. 1d: show all 44 categories as horizontal bar chart with exact values, remove "+ 34 more categories" | 6,20 |
| 5 | Establish consistent color palette and typography across all figures | 6 |
| 6 | Redesign Sup. Fig. S1: replace zoomed 95-100% chart with enhanced vs. baseline comparison by category | 6,7 |
| 7 | Add Conclusions subsection | 9,15 |
| 8 | Add availability statement (URL) to abstract | 9 |

## Important (Strongly Recommended)

| # | Action | Source Reviews |
|---|--------|---------------|
| 9 | Add 2-3 real-world use case vignettes demonstrating end-to-end workflows | 3,8,10 |
| 10 | Expand references from 11 to 25+ covering RAG, code generation, workflow management | 4,9,20 |
| 11 | Add ablation framing: acknowledge separate contributions of docs vs. skills | 4 |
| 12 | Discuss privacy/security implications of cloud LLM usage and local model support | 11,12 |
| 13 | Address benchmark circularity honestly in Methods or Discussion | 14 |
| 14 | Discuss extensibility: custom skill creation, community contributions, maintenance | 13,16,17 |
| 15 | Add data labels/values to all bar charts in all figures | 6 |

## Recommended (Nice to Have)

| # | Action | Source Reviews |
|---|--------|---------------|
| 16 | Discuss educational applications of oxo-call | 19 |
| 17 | Discuss cloud/container deployment scenarios | 18 |
| 18 | Improve cover letter: more concise, address novelty more directly | 20 |
| 19 | Frame provenance in context of clinical audit and W3C PROV | 5,12 |
| 20 | Tighten writing: shorten long sentences, improve figure legends | 15 |
