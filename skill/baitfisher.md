---
name: baitfisher
category: sequence_analysis
description: Baitfisher is a bioinformatics tool used for designing and analyzing oligonucleotide baits for targeted sequencing approaches, such as hybrid capture experiments. It aids in identifying optimal bait sequences by evaluating various parameters including specificity, GC content, and off-target binding potential.
tags: [targeted-sequencing, hybrid-capture, oligonucleotide-design, bait-design, sequence-analysis, genomics]
author: AI-generated
source_url: https://github.com/baitfisher/baitfisher
---

## Concepts

- Baitfisher operates on FASTA-formatted sequence input files containing target genomic regions or genes of interest. The tool accepts multiple input formats including plain FASTA, multi-FASTA, and optionally BED format for specifying genomic coordinates.
- The output consists of optimized bait sequences in FASTA format, often accompanied by a summary report (TSV or CSV) containing metrics such as GC content, melting temperature (Tm), uniqueness scores, and off-target hits.
- Baitfisher evaluates bait candidates based on a scoring algorithm that weighs parameters like sequence complexity, GC content (typically targeting 40-60% GC), self-complementarity (to avoid secondary structures), and cross-reactivity against a background genome index.

## Pitfalls

- Using input sequences with excessive N characters or ambiguous bases can lead to bait designs with unreliable binding predictions, resulting in poor capture efficiency during experimental validation.
- Failing to provide a complete background database for off-target assessment causes Baitfisher to generate baits with unanticipated cross-hybridization, reducing specificity and wasting experimental resources.
- Specifying an excessively short or long bait length (outside recommended 80-120 bp for most capture platforms) creates issues with solubility, binding kinetics, or library preparation compatibility.
- Not filtering duplicate or overlapping input regions before running Baitfisher results in redundant bait designs that waste sequencing library capacity and increase costs without improving coverage.
- Misconfiguring the scoring weights (e.g., over-emphasizing GC content at the expense of specificity) produces baits that may work in silico but perform poorly in actual hybrid capture reactions.

## Examples

### Design baits from a single gene sequence

**Args:** -i sequence.fasta -o output_baits.fasta -length 100
**Explanation:** This runs Baitfisher using the input gene sequence to generate 100 bp bait sequences, placing output files in the specified directory.

### Generate baits with a specific GC content range

**Args:** -i target_genes.fasta -o baits.gcfiltered.fasta -min-gc 40 -max-gc 60
**Explanation:** This command constrains the output baits to have GC content between 40% and 60%, optimizing for stable hybrid binding without extreme AT/GC bias.

### Include off-target screening against a reference genome

**Args:** -i capture_regions.fasta -o baits_filtered.fasta -bg genome_index -max-offtarget 2
**Explanation:** This runs Baitfisher with background genome screening to filter out baits with more than 2 off-target binding sites, improving capture specificity.

### Adjust the number of bait tiles per target region

**Args:** -i exome_targets.fasta -o exome_baits.fasta -tiling 3 -overlap 20
**Explanation:** This generates overlapping bait tiles (3 per target) with 20 bp overlap to ensure complete coverage of the target exome regions.

### Export detailed metrics for downstream analysis

**Args:** -i input.fasta -o baits.fasta -stats bait_metrics.tsv -format detailed
**Explanation:** This produces both the bait FASTA file and a detailed TSV report containing per-bait metrics for quality assessment and selection criteria.

### Filter baits for self-complementarity to avoid secondary structures

**Args:** -i genes.fasta -o clean_baits.fasta -max-self-dimer 8 -hairpin-size 6
**Explanation:** This removes bait sequences predicted to form self-dimers with delta G exceeding 8 kcal/mol or hairpin structures larger than 6 bp.