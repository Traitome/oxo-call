---
name: cassis
category: CRISPR sgRNA Design
description: CASSIS predicts CRISPR/Cas9 target sites and ranks guide RNAs by efficiency and specificity from genomic DNA input sequences.
tags:
  - crispr
  - sgRNA design
  - guide RNA
  - off-target prediction
  - cas9
  - genome editing
author: AI-generated
source_url: https://github.com/rkitchen/cassis
---
```

## Concepts

- CASSIS analyzes input DNA sequences (FASTA or plain sequence) to identify all 20-nt protospacer sequences immediately preceding a 3′ NGG PAM motif compatible with Cas9, then ranks them by a composite score weighting both on-target efficiency and off-target specificity.
- The tool requires a pre-built genomic index (via `cassis-build`) to perform off-target specificity checks against the reference genome; without this index, specificity scores are omitted or set to a minimal placeholder.
- Output is produced in structured text or tab-delimited format depending on flags, reporting fields such as chromosome, start, end, strand, sgRNA sequence, PAM, efficiency score, specificity score, and genomic off-target count.
- CASSIS supports `--species` to load pre-configured PAM rules (e.g., human, mouse, zebrafish) and scoring model parameters trained on organism-specific datasets; mismatched species settings degrade prediction accuracy.
- The core pipeline operates in two phases: target enumeration (scanning for NGG PAM matches) followed by scoring (applying machine-learning efficiency and alignment-based specificity models); disabling either phase with a flag changes the output accordingly.

## Pitfalls

- Providing a raw sequence file with headers or whitespace formatting that differs from the expected input type causes the scanner to miss all target sites silently, producing an empty output file with no error.
- Running CASSIS without first executing `cassis-build` to generate a genomic index results in all specificity scores defaulting to zero, making guides appear equally safe when in reality some have significant off-target hits.
- Specifying the wrong `--species` value leads to CASSIS applying mismatched PAM rules or a scoring model trained on a different organism, which can cause false-positive PAM calls (missing valid targets) or incorrect efficiency predictions.
- Using an outdated genome assembly for `cassis-build` while annotating guides for a different genome version creates coordinate mismatches, rendering off-target reports non-comparable to experimental mapping data.
- Requesting only efficiency ranking without specificity checks by omitting `--score-specificity` can select guides that are highly efficient but genomically promiscuous, increasing the risk of unintended editing events in the lab.

## Examples

### Predict CRISPR target sites from a human DNA sequence file

**Args:** `predict --sequence input.fa --species human --output predictions.txt`
**Explanation:** This scans the FASTA input for all Cas9 NGG PAM targets in the human genome context and writes ranked results to the output file.

### Build a genomic index for off-target specificity scoring

**Args:** `build --genome hg38.fa --index-dir ./cassis_index --threads 8`
**Explanation:** This constructs the reference index needed later by the specificity scoring engine against the hg38 genome assembly.

### Rank predicted guides by combined efficiency and specificity score

**Args:** `predict --sequence input.fa --species human --score-efficiency --score-specificity --output ranked.txt`
**Explanation:** This enables both scoring modules so that CASSIS computes a composite rank reflecting both guide activity and off-target risk.

### Limit output to guides with PAM sequence immediately downstream of the target

**Args:** `predict --sequence input.fa --species human --require-pam --output pam_filtered.txt`
**Explanation:** This restricts output to only those guides where the canonical NGG PAM is confirmed, reducing false positives from non-canonical PAM predictions.

### Run predictions using a pre-built index stored in a custom directory

**Args:** `predict --sequence query.fa --species human --index-dir /data/cassis_idx --output results.txt`
**Explanation:** This directs CASSIS to the previously built index directory rather than searching for a default location, avoiding re-indexing on every run.