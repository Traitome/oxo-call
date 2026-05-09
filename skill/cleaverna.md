---
name: cleaverna
category: Genomics / Genome Editing
description: A bioinformatics tool for CRISPR guide RNA design, analysis, and optimization. Cleaverna identifies and ranks candidate guide sequences (gRNAs/sgRNAs) for CRISPR-Cas9 and related genome editing systems, providing efficiency scores, off-target analysis, and specificity metrics.
tags: [crispr, grna, genome-editing, sirna, gene-editing, bioinformatics, sequence-analysis]
author: AI-generated
source_url: https://github.com/cleaverna/cleaverna
---

## Concepts

- **Input Formats**: Cleaverna accepts target sequences in FASTA format (single sequences or multi-FASTA), GenBank files, or raw text with genomic coordinates (chromosome:start-end). Sequences must be provided in 5'→3' orientation relative to the PAM site for accurate scoring.
- **Output Formats**: Primary output is a tab-delimited table with columns for guide sequence, start position, strand, PAM motif, GC content, efficiency score, specificity score, and off-target count. Optional JSON and BED exports are available for downstream analysis.
- **Scoring Model**: Cleaverna calculates efficiency scores (0-100 scale) using a trained machine learning model considering position-specific nucleotide weights, GC content, and secondary structure predictions. Specificity scores reflect genome-wide homology analysis against a reference database.
- **PAM Recognition**: The tool supports multiple PAM variants including NGG (SpCas9), NNNNGG (SpCas9 variants), TTTV (Cas12a), and TGG (Cas9 from S. thermophilus). PAM must be specified at the 3' end of the target sequence.
- **Off-Target Analysis**: Guides are mapped against the provided reference genome using exact matching (by default) or allowing up to 2 mismatches. Off-target hits are reported with genomic coordinates and mismatch positions.

## Pitfalls

- **Incorrect Sequence Orientation**: Providing the anti-complementary strand instead of the target strand results in guides binding the wrong DNA region. The returned coordinates will be on the opposite strand, leading to failed experiments.
- **Ignoring Off-Target Scores**: Selecting guides solely based on high efficiency scores without considering specificity can result in unintended genome modifications at similar genomic loci, causing confounding phenotypes in downstream assays.
- **PAM Site Mismatch**: Specifying the wrong PAM variant (e.g., NGG when analyzing for Cas12a targets) produces no valid guides or guides that will not be recognized by the intended Cas enzyme, wasting experimental resources.
- **Low GC Content Guides**: Guides with GC content below 30% or above 80% often show poor cutting efficiency due to poor DNA binding kinetics. This leads to reduced edit rates and inconsistent experimental results.
- **Targeting Repetitive Regions**: Failing to filter guides in repetitively-mapped genomic regions causes ambiguous mapping and potential off-target effects, as these regions share high homology with multiple genome locations.

## Examples

### Design CRISPR guides for a specific gene sequence
**Args:** `-seq ATGGCGTACGACCTGGACTACATCGTGGGCATCGTGATCGGCGTCACCAACTTC -pam NGG -output guides.txt`
**Explanation:** Given a target DNA sequence, this command identifies all possible 20bp guide windows flanked by NGG PAM sites and outputs candidate guides with their genomic positions and orientation.

### Analyze off-target potential for a single guide
**Args:** `-guide GATCGGCGTCACCAACTTC -genome hg38 -mismatches 2 -offtarget_out off_targets.txt`
**Explanation:** This maps the specified guide against the human genome reference (hg38), allowing up to 2 mismatches, and reports all genomic hits to assess potential off-target cleavage sites.

### Filter guides by minimum efficiency threshold
**Args:** `-input guides.txt -min_score 60 -output high_efficiency.txt`
**Explanation:** Using a pre-computed guide list as input, this filters candidates to retain only those with efficiency scores of 60 or higher, ensuring higher likelihood of successful genome editing.

### Export results in JSON format for programmatic parsing
**Args:** `-seq ATGGCGTACGACCTGGACTAC -pam NGG -format json -output results.json`
**Explanation:** This outputs the guide design results in JSON format rather than the default tabular format, facilitating integration with pipeline scripts and automated workflows.

### Batch process multiple sequences from a multi-FASTA file
**Args:** `-input sequences.fasta -pam NGG -score -output batch_results.txt`
**Args:** `**Explanation:** This processes all sequences in a multi-FASTA file, computing efficiency and specificity scores for each candidate guide across every input sequence in a single run.

### Generate BED file for guide genomic visualization
**Args:** `-input guides.txt -bed -output guides.bed`
**Explanation:** This converts the guide output table to BED format, enabling visualization of candidate guide positions in genome browsers like IGV for genomic context assessment.