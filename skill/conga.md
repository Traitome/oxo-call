---
name: conga
category: computational biology
description: Design orthogonal CRISPR guide RNAs for cellular multiplexing. ConGA takes genomic sequences and returns ranked on-target guides filtered for cross-reactivity with existing CRISPR systems, predicting efficiency and off-target sites for each candidate.
tags: [crispr, guide-rna, orthogonality, gene-editing, multiplexing, thermodynamics]
author: AI-generated
source_url: https://github.com/johli/conga
---

## Concepts

- ConGA designs **orthogonal** guide RNAs (o-gRNAs) — guide sequences that do not cross-talk with existing CRISPR systems in the target cell line, enabling multiple CRISPR effectors (e.g., Cas9, Cas12a, dCas9-KRAB) to operate simultaneously without mutual interference.
- Input is a **genomic target sequence** (FASTA or plain text) plus a reference genome index for off-target scanning; output is a ranked list of candidate guides scored by on-target efficiency and filtered for orthogonal cross-reactivity.
- ConGA uses **thermodynamic folding models** (RNAcofold/NUPACK-style secondary structure predictions) to evaluate guide RNA architecture — the spacer, scaffold, and transcriptional terminator — ensuring the chosen guide folds correctly inside the cell.
- Guides are scored on two axes: **on-target efficiency** (positional features + thermodynamic stability) and **off-target penalty** (mismatches relative to the reference genome); the combined score ranks candidates for experimental pick.
- ConGA supports multiple **Cas orthologs** (e.g., SpyCas9, StCas9, Cas12a) and **organism-specific** reference genomes; the wrong genome build makes all off-target scores meaningless.
- Output formats include a **ranked text table** (by default), a **JSON** file for downstream pipelines, and an optional **BED** file for genome browser visualization of guide target loci.

## Pitfalls

- Using an outdated or mismatched **reference genome build** causes off-target scores to be computed against the wrong genomic coordinates, making all cross-reactivity predictions unreliable for the cell line being edited.
- Providing **fewer than 100 bp of flanking sequence context** on either side of the target site leads to guide candidates that may overlap promoter or regulatory elements, reducing efficiency and introducing unexpected regulatory effects.
- Accepting the **default 1-bp tolerance** for off-target scanning when designing guides for therapeutically relevant edits — for clinical applications, the off-target mismatch threshold should be tightened (0 mismatches allowed) at the cost of fewer candidates.
- Confusing **on-target efficiency rank** with orthogonal cross-reactivity rank: a guide with top efficiency can still cross-talk with the endogenous CRISPR system if it was not explicitly checked against the existing o-gRNA library.
- Running ConGA without specifying the **Cas subtype** (e.g., `--cas SpyCas9` vs `--cas Cas12a`) produces guides with the wrong PAM or spacer-length requirements, and the resulting sequences will not be compatible with the intended effector.

## Examples

### Design orthogonal guides for a human genomic target
**Args:** `input_seqs.txt --genome GRCh38 --cas SpyCas9 --org human --outfmt table --out conga_human_guides.tsv`
**Explanation:** Specifies the human reference genome and Cas9 subtype so ConGA evaluates off-target sites and PAM compatibility correctly for human cells.

### Generate a ranked list of orthogonal guides in JSON format for pipeline integration
**Args:** `target_BRAF.fa --genome GRCh38 --cas SpyCas9 --org human --outfmt json --out conga_BRAF_ortho.json --score_metric combined`
**Explanation:** JSON output preserves per-guide score breakdowns for downstream pick lists in automated CRISPR design pipelines.

### Export guide target coordinates as a BED file for genome browser visualization
**Args:** `target_FAANGene.fasta --genome GRCh38 --cas SpyCas9 --org human --outfmt bed --out conga_FAANGene.bed`
**Explanation:** BED format allows direct loading into IGV or UCSC Genome Browser to visually inspect guide target loci alongside existing annotations.

### Enforce zero-mismatch off-target tolerance for a therapeutic editing context
**Args:** `target_clinical.fa --genome GRCh38 --cas SpyCas9 --org human --ot-mismatches 0 --out conga_strict.tsv --outfmt table`
**Explanation:** Setting `--ot-mismatches 0` discards any guide with a single off-target match, producing a shorter but safer candidate list for clinical applications.

### Design orthogonal guides using the Cas12a (cpf1) system for a different PAM requirement
**Args:** `target_Ngene.fa --genome GRCh38 --cas Cas12a --org human --out conga_cas12a.tsv --outfmt table`
**Explanation:** Specifying `--cas Cas12a` switches PAM and spacer-length rules to TTTV rather than NGG, and outputs guides compatible with Cas12a expression constructs.