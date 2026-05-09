---
name: colabfold
category: protein-structure-prediction
description: Protein structure prediction tool combining AlphaFold2 and RoseTTAFold algorithms for accurate 3D protein structure prediction from amino acid sequences
tags: protein-folding, structural-biology, alphafold, rosettafold, deep-learning, molecular-modeling
author: AI-generated
source_url: https://github.com/sokrypton/ColabFold
---

## Concepts

- ColabFold accepts FASTA files containing protein sequences and produces 3D structure predictions in PDB format, along with confidence metrics including pLDDT scores and PAE (predicted alignment error) matrices
- The tool automatically generates Multiple Sequence Alignments (MSAs) from query sequences by default, which significantly improves prediction accuracy for proteins with detectable homology; this behavior can be controlled with --use-precomputed-msa
- Template searching against the PDB is enabled by default and can be configured using --template-max-templates and --template-hit-cache to improve predictions for proteins with known structural homologs
- Recycling iterations, controlled via --num-recycle, allow the model to refine predictions by re-passing predicted structures back through the network; values between 1-20 are supported with 3 being a common default
- ColabFold outputs all predicted structures ranked by their PAE scores, with rank 1 being the top prediction; additional ranked structures are available for comparative analysis

## Pitfalls

- Submitting very long sequences exceeding the maximum supported length without truncating or adjusting parameters will cause the prediction to fail silently or produce unreliable structures
- Using --disable-template-search when the target protein has available structural homologs results in lower prediction accuracy, as templates provide crucial structural constraints
- Running with insufficient recycling iterations (below 3) produces poorly converged structures with inconsistent pLDDT scores across residues
- Not specifying --output-dir results in output files being written to the current working directory, potentially overwriting previous predictions with identical filenames
- Failing to properly configure database paths with --bfd-database, --uniref-database, and --mgnify-database causes the tool to download databases on first run, which can take hours depending on network conditions

## Examples

### Predict protein structure from a single FASTA file

**Args:** --input input_sequence.fasta --output-dir predictions_output
**Explanation:** This command reads the protein sequence from the provided FASTA file and saves all output structures and confidence matrices to the specified output directory.

### Disable template searching for a novel protein with no structural homologs

**Args:** --input novel_protein.fasta --template-max-templates 0 --output-dir no_template_predictions
**Explanation:** Disabling template searches is appropriate when predicting structures for proteins with no detectable structural homologs in the PDB, avoiding potential false template alignments.

### Use a pre-computed MSA file to speed up repeated predictions

**Args:** --input protein.fasta --use-precomputed-msa msa_a3m --output-dir cached_predictions
**Explanation:** Providing a pre-computed MSA in A3M format eliminates the time-consuming alignment generation step for subsequent predictions of the same protein or closely related sequences.

### Control prediction accuracy with recycling iterations

**Args:** --input protein.fasta --num-recycle 10 --output-dir high_accuracy_predictions
**Explanation:** Increasing the number of recycling iterations to 10 allows the model more opportunities to refine atomic positions, producing higher-quality structures at the cost of longer computation time.

### Predict structures for multiple sequences in a batch FASTA file

**Args:** --input batch_proteins.fasta --num-recycle 3 --output-dir batch_predictions
**Explanation:** ColabFold processes all sequences in the provided FASTA file sequentially, generating independent structure predictions for each protein with the specified recycling configuration.