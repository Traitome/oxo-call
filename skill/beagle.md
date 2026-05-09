---
name: beagle
category: phylogenetics
description: A high-performance library and tool for phylogenetic analysis that accelerates maximum-likelihood tree reconstruction using CPU parallelism and GPU acceleration via the BEAGLE library.
tags: [phylogeny, evolutionary-biology, sequence-analysis, maximum-likelihood, tree-reconstruction, gpu-computing]
author: AI-generated
source_url: https://github.com/beagle-lib/beagle-lib
---

## Concepts

- Beagle accepts multiple sequence alignment formats (FASTA, NEXUS, PHYLIP) as input and outputs phylogenetic trees in Newick format, supporting models ranging from simple JC69 to complex GTR+Γ+I substitutions.
- The tool leverages the BEAGLE library to perform parallel computation across CPU cores via OpenMP and can offload likelihood calculations to GPUs using CUDA or OpenCL, dramatically speeding up analyses of large alignments.
- Beagle uses resource scaling flags (`-scaling`, `-threads`, `-precision`) to control computational behavior—dynamic scaling prevents numerical underflow but loses branch length information, while fixed scaling preserves it at the cost of potential overflow.

## Pitfalls

- Failing to specify an evolutionary model results in the default JC69 model being used, which may be unsuitable for most biological datasets with biased base composition or transition/transversion rate differences, leading to inaccurate tree topology.
- Using dynamic branch length scaling (`-scaling dynamic`) with large trees can cause numerical underflow in deep lineages, collapsing internal branch lengths to zero and making the tree unusable for downstream comparative analyses.
- Running beagle on systems without GPU drivers installed while specifying GPU resource flags will cause the process to abort silently or fall back to slow CPU-only mode without warning, yielding unexpectedly long runtimes.
- Misaligning sequence identifiers between alignment and constraint tree files causes beagle to either ignore constraints or terminate with a cryptic parse error, depending on whether the mismatch is detected as a format issue.

## Examples

### Compute a maximum-likelihood tree from a FASTA alignment

**Args:** input.fasta -o output.tre

**Explanation:** This runs phylogenetic analysis on the FASTA-formatted alignment using default settings and writes the resulting tree in Newick format to the specified output file.

### Apply the GTR+Γ substitution model with site heterogeneity

**Args:** input.nex -o output.tre -m GTR -g

**Explanation:** This uses the general time reversible model with gamma-distributed rate heterogeneity across sites, which is more realistic for DNA sequences with varying evolutionary rates.

### Enable GPU acceleration using CUDA

**Args:** input.fasta -o output.tre -P cuda:0

**Explanation:** This routes likelihood computations to the first CUDA-capable GPU, providing substantial speedup for large alignments compared to CPU-only computation.

### Use fixed scaling to preserve branch length values

**Args:** input.fasta -o output.tre -scaling fixed

**Explanation:** This uses fixed point scaling during likelihood calculations, which maintains accurate branch length estimates but requires manual monitoring for numerical overflow in very long trees.

### Specify 8 CPU threads for parallel analysis

**Args:** input.fasta -o output.tre -threads 8

**Explanation:** This utilizes 8 OpenMP threads for parallel computation across CPU cores, distributing the computational burden and reducing runtime on multi-core workstations.

### Use a constraint tree to guide tree topology search

**Args:** input.nex -o output.tre -t constraint.tre

**Explanation:** This incorporates a user-provided constraint tree topology that limits the search space, enforcing monophyly of specified groups while optimizing branch lengths elsewhere.

### Analyze a protein-coding alignment with empirical amino acid frequencies

**Args:** input.fasta -o output.tre -m WAG -f empirical

**Explanation:** This applies the WAG amino acid substitution model with empirical frequency estimates derived directly from the dataset, improving model fit for protein sequence analyses.

### Run with high precision double-precision floating point

**Args:** input.fasta -o output.tre -precision double

**Explanation:** This enables double-precision arithmetic throughout likelihood computations, reducing numerical error accumulation at the cost of slightly increased memory usage per site.