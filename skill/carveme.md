---
name: carveme
category: metabolic-modeling
description: Carveme reconstructs genome-scale metabolic models (GEMs) from genome annotations by mapping predicted protein sequences to metabolic reactions. It integrates gene calls from Prodigal, homology searches from DIAMOND or HMMER, and predefined model templates to generate SBML-formatted metabolic networks suitable for constraint-based modeling in COBRApy or similar tools.
tags:
  - metabolic-reconstruction
  - genome-annotation
  - constraint-based-modeling
  - SBML
  - systems-biology
author: AI-generated
source_url: https://github.com/cdanielmachado/carveme
---

## Concepts

- **Gene-to-Reaction Mapping**: Carveme maps protein sequences identified by annotation tools (Prodigal, DIAMOND, HMMER) to reactions in metabolic databases like BiGG, KEGG, or MetaCyc. The quality of the input annotations directly determines model completeness.
- **Model Templates**: Carveme uses predefined "draft" templates (e.g., BiGG universal model) to guide reconstruction. Templates specify reaction confidence thresholds and include reference metabolic networks against which genome-derived genes are matched.
- **SBML Output Format**: The final reconstructed model is exported as an SBML (Systems Biology Markup Language) file, compatible with constraint-based modeling tools like COBRApy, libSBML, or FBA solvers.
- **Evidence Scores**: Reactions receive evidence scores based on the number and quality of supporting gene annotations. Reactions with multiple independent gene supports are ranked higher, while hypothetical or low-evidence reactions receive lower scores.
- **Input Compatibility**: Carveme accepts standard annotation outputs including DIAMOND BLAST tab-format, HMMER domain tables, and Prodigal gene calls in GFF or TSV format.

## Pitfalls

- **Incomplete Genome Annotations**: Providing partial or low-coverage genome annotations results in draft models with many missing reactions, producing biologically implausible models with numerous blocked reactions in subsequent FBA simulations.
- **Mismatched Template Database**: Using a template database that does not align with the organism's taxonomy (e.g., using plant templates for bacterial genomes) leads to irrelevant or erroneous reaction inclusion, reducing model accuracy.
- **Threshold Misconfiguration**: Setting reaction confidence thresholds too low (e.g., threshold 0) floods the model with low-evidence reactions, causing false positives and inflated metabolic capabilities that do not reflect real physiology. Setting thresholds too high prunes legitimate reactions.
- **Missing or Malformed Input Files**: Supplying input annotation files in unsupported formats or with missing required fields (e.g., lacking sequence identifiers that map to the template database) causes silent failures where reactions are not matched at all.
- **Ignoring Gaps and Blocked Reactions**: Reconstructed models frequently contain reactions with no computed flux under any environment (blocked reactions). Failing to identify and address these gaps leads to models that fail to produce realistic growth predictions in FBA simulations.

## Examples

### Reconstruct a draft metabolic model from DIAMOND BLAST annotations against BiGG database

**Args:** reproduce --input-annotations my_genome_diamond.tsv --algorithm diamond --template大肠杆菌 --output my_draft_model.xml
**Explanation:** This reconstructs a draft metabolic model for E. coli using DIAMOND BLAST annotations matched against the BiGG universal template, outputting an SBML file for downstream FBA analysis.

### Reconstruct a model with custom reaction confidence thresholds

**Args:** reproduce --input-annotations prodigal_genes.tsv prodigal --input-annotations hmmer_domains.tsv hmmers --template universal --threshold 2 --output strict_model.xml
**Explanation:** This combines Prodigal gene calls and HMMER domain predictions with a confidence threshold of 2, only including reactions supported by at least two independent gene or domain annotations.

### List available metabolic templates and their associated reaction databases

**Args:** list_templates
**Explanation:** This displays all predefined model templates (e.g., BiGG, MetaCyc, KEGG) that carveme can use for reconstruction, showing their source databases and version information.

### Validate a reconstructed SBML model for consistency and blocked reactions

**Args:** validate my_model.xml
**Explanation:** This checks the reconstructed SBML model for internal consistency, identifies blocked reactions with zero flux under any tested condition, and reports statistics on model completeness.

### Perform iterative model refinement by identifying and filling gaps

**Args:** recycle my_model.xml --method fill_gaps --template universal --max-reactions 50
**Explanation:** This iteratively refines an existing model by identifying metabolic gaps and attempting to fill them using reactions from the universal template, limiting the addition to 50 new reactions per iteration.