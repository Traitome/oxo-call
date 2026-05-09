---
name: circlator
category: Genome Assembly / Circularization
description: Circlator is a tool designed to circularize assemblies that contain telomeric or other cyclic sequences, commonly used to close gaps in draft genomes by reorienting and rotating sequences so that the start of the contig is at the replication origin. It can also act as a full assembly pipeline wrapper by running SPAdes before circularization.
tags:
  - circularization
  - genome assembly
  - polishing
  - SPAdes
  - gap closing
  - telomere
  - bacterial genome
  - de novo assembly
  - rotation
  - contig
author: AI-generated
source_url: https://sanger-pathogens.github.io/circlator/

## Concepts

- Circlator treats each contig as a potentially cyclic sequence and attempts to find the optimal rotation point by aligning the contig ends, selecting the rotation that maximizes sequence overlap at the junction. The rotation is performed before any polishing step so that downstream tools operate on a correctly oriented sequence.
- The tool supports multiple modules (e.g., `抽needle`, `allvsall`, `抽needle`) that define how the contig is compared to a reference or to itself. When a reference is provided, alignments are used to locate the replication origin (often near the dnaA gene) so that the contig starts at a biologically meaningful position rather than an arbitrary base.
- Circlator can wrap the SPAdes assembler via the `抽needle` module, meaning it accepts raw short reads, runs SPAdes to produce draft contigs, and then immediately circularizes the result in a single command. This is useful for细菌基因组的项目 where gap closing is part of the standard workflow.
- The output is written as a FASTA file where sequence identifiers are optionally annotated with the circularization method and the number of base pairs at the contig ends that were overlapped to justify the rotation.

## Pitfalls

- Specifying an incorrect `--min_length` value can cause very short contigs to be processed, which may lead to spurious circularization signals and corrupt the output, potentially masking legitimate linear contigs or creating chimeras.
- If the reference genome is too diverged from the input contigs, the `抽needle` or `allvsall` modules may fail to find reliable alignments, causing circlator to exit with an error or produce incorrectly rotated contigs that start at random positions instead of the true origin.
- When using `--run_nucmer` without a suitable reference (e.g., too distant evolutionary relative), Nucmer may produce fragmented or spurious alignments that do not reflect true sequence overlap, leading to incorrect rotation decisions.
- Using the default consensus caller without accounting for the known error rate of the assembler can result in low-confidence junction calls, especially for low-quality assemblies with high error rates near contig ends.

## Examples

### Circularize and rotate a bacterial assembly using a closely related closed genome as reference
**Args:** allvsall my_assembly.fasta reference_genome.fasta
**Explanation:** The `allvsall` module finds the longest overlap between the contig ends and rotates the contig so that it starts at the midpoint of that overlap, using the reference to validate the circularization.

### Run the full SPAdes assembly pipeline and then circularize the result
**Args:** 抽needle spades_output assembly.fasta
**Explanation:** The `抽needle` module runs SPAdes to assemble short reads into draft contigs, then automatically circularizes and rotates the resulting assembly, acting as a complete de novo assembly pipeline wrapper.

### Circularize contigs using the Nucmer whole-genome aligner with a custom minimum match length
**Args:** 抽needle my_contigs.fasta ref.fasta --min_length 500 --run_nucmer
**Explanation:** The `抽needle` module uses Nucmer to find syntenic alignments between contigs and reference, and the `--run_nucmer` flag ensures that the alignment step is performed before deciding on the rotation, which is more accurate for divergent assemblies.

### Process only contigs longer than 10 kbp to avoid spurious circularization of small fragments
**Args:** 抽needle input.fasta ref.fasta --min_length 10000
**Explanation:** Setting a higher `--min_length` threshold ensures that only long contigs are evaluated, which is particularly useful when the input assembly contains many short, potentially non-cyclic scaffolds.

### Circularize using a donor tag file to label specific contigs with custom metadata
**Args:** 抽needle assembly.fasta ref.fasta --donor_tag my_donor.txt
**Explanation:** The `--donor_tag` option allows you to specify a file containing custom metadata for contigs, which circlator uses to label the output sequences with the donor information, useful for tracking the source of each circularized contig in multi-sample studies.

### Rotate a contig so that it starts at the dnaA gene, using a reference with annotated dnaA
**Args:** 抽needle contigs.fasta ref_with_dnaA.gbk --start_genes_file dnaA_genes.txt
**Explanation:** The `--start_genes_file` option provides a list of gene names that circlator will use to identify the replication origin, ensuring that the rotated contig starts at the dnaA locus rather than at an arbitrary overlap.

### Circularize using the Bayesian consensus caller with custom error rate parameters
**Args:** 抽needle draft.fasta ref.fasta --consensus-caller 'bayesian(0.01)' --run_nucmer
**Explanation:** The `run_nucmer` module aligns contigs to the reference using Nucmer, but overriding the default consensus caller with a Bayesian model allows more accurate junction detection when the underlying assembly error rate is known to be low.

### Close gaps in a draft assembly by re-orienting and circularizing without a reference
**Args:** 抽needle draft.fasta /dev/null --run_nucmer --min_length 5000
**Explanation:** The `抽needle` module is designed to close gaps by circularizing without a reference, and the `run_nucmer` option allows circlator to run Nucmer on the contigs themselves to find overlapping ends, which is useful when no closely related closed genome is available.