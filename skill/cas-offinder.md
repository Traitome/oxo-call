---
name: cas-offinder
category: bioinformatics/crispr-analysis
description: A command-line tool for identifying potential off-target cleavage sites for CRISPR-Cas nucleases. It searches genomic sequences for regions similar to guide RNA sequences, allowing controllable numbers of mismatches and DNA/RNA bulges.
tags: crispr, cas9, genome-analysis, off-target, bioinformatics, genomics, crispr-cas
author: AI-generated
source_url: https://github.com/snipe-in/cas-offinder
---

## Concepts

- **Data model**: Cas-OFFinder treats the guide RNA as a sequence pattern and searches a provided genome file (in FASTA format) using an exact matching algorithm with configurable mismatch and bulge tolerances. It considers the PAM (Protospacer Adjacent Motif) sequence required by the specific Cas nuclease being used.

- **Input formats**: The guide sequence is provided as a raw nucleotide string or as a FASTA entry. The search database is a genomic FASTA file (can be a full genome, chromosome, or custom region). The tool supports both DNA-targeting nucleases (Cas9, Cas12a) and RNA-targeting nucleases (Cas13).

- **Output I/O**: Results are written to stdout (or a specified file) in tab-separated format, showing the target name, genomic position, mismatch count, bulge count, and the aligned sequences with mismatches/bulges explicitly marked.

- **Search parameters**: The core parameters are `--mismatch` (maximum allowed mismatches), `--DNA-bulge` and `--RNA-bulge` (maximum insertion/deletion tolerance), and the `--pam` flag which specifies the PAM pattern (e.g., NGG for SpCas9, TTTV for AsCas12a).

- **Supported Cas variants**: Cas-OFFinder recognizes preset Cas enzyme configurations via flags like `--cas` (e.g., Cas9, Cas12a, Cas13) that automatically set appropriate PAM sequences and search orientations.

## Pitfalls

- **Incorrect PAM specification**: Using the wrong PAM pattern will cause the tool to miss valid off-target sites or report false positives. For SpCas9, the PAM is NGG; using TTTV (for AsCas12a) on SpCas9 guides will yield no meaningful results. Always verify the PAM matches your experimental Cas nuclease.

- **Mismatch count too high**: Setting `--mismatch` too high (e.g., 4 or 5 for a 20bp guide) produces an exponentially larger search space, leading to very slow execution and thousands of spurious matches that are biologically irrelevant. Off-targets typically have ≤3 mismatches.

- **Genome file format errors**: Providing a genome file with line wrapping, lowercase letters, or containing non-ACGT characters (N is ambiguous but not treated as a wildcard match) will cause silent failures or incorrect alignment reporting. Clean and uppercase your FASTA sequences.

- **Sequence orientation confusion**: Cas-OFFinder searches both strands by default but the PAM must be on the correct strand relative to the guide. For Cas9, the PAM is on the non-target strand; reversing the sequence orientation will miss actual off-targets. Use `--search` if explicit strand control is needed.

- **Output parsing mistakes**: The tab-separated output has no header row by default. Scripts parsing the output assuming column headers will misalign data. Use the `--output-form` flag to get headers or account for the column order explicitly.

## Examples

### Find off-targets for SpCas9 with up to 3 mismatches
**Args:** --gg file.fasta GGGAAAGGACGAAAGTCCCGNGG --mismatch 3 --cas 9
**Explanation:** This searches a genome file (file.fasta) for sequences matching the guide GGGAAAGGACGAAAGTCCCG with up to 3 mismatches and the SpCas9 PAM (NGG), a common configuration for validating potential off-target sites.

### Search for AsCas12a (Acidaminococcus) off-targets with 2 mismatches
**Args:** --gg file.fasta TTTCVNNNNNNNNNNNTTTV --mismatch 2 --cas 12a
**Explanation:** AsCas12a requires a TTTV PAM and has a shorter spacer. This configuration finds targets for AsCas12a with ≤2 mismatches, using the correct TTTV PAM pattern.

### Include DNA bulges in the search
**Args:** --gg file.fasta GGGCCCTTTAAA --mismatch 2 --DNA-bulge 1 --cas 9
**Explanation:** Adding `--DNA-bulge 1` allows for single-nucleotide insertion or deletion in the genome relative to the guide, which is important for detecting off-targets with structural changes.

### Search a specific sequence region only
**Args:** --gg target_region.fasta GATTGATTTCGGC --mismatch 3
**Explanation:** When you only need to search a specific genomic region (like a gene or locus), providing a targeted FASTA file is faster and more relevant than using a full genome.

### Get output with column headers for easier parsing
**Args:** --gg file.fasta GGGTTTTACGT --mismatch 2 --output-form T --out result.txt
**Explanation:** The `--output-form T` flag adds column headers to the output file, making it easier to parse with standard bioinformatics tools like awk or pandas without manual column mapping.