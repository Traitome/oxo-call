---
name: admixture
category: population-genomics
description: Fast maximum likelihood estimation of individual ancestry proportions from multilocus SNP genotype datasets
tags: [population-genetics, ancestry, structure, gwas, admixture, plink, k-value, q-matrix, cross-validation]
author: oxo-call built-in
source_url: "https://dalexander.github.io/admixture/"
---

## Concepts

- ADMIXTURE estimates ancestry proportions (Q-matrix) and allele frequencies (P-matrix) for a specified K (number of ancestral populations).
- ADMIXTURE uses PLINK binary format (.bed/.bim/.fam) as primary input; PLINK "12" coded .ped/.map and EIGENSTRAT .geno formats are also supported.
- Input file MUST include the full extension (e.g., `data.bed`, not `data`). ADMIXTURE uses the extension to detect the file format.
- Specify K (number of ancestral populations) as a positional argument: `admixture data.bed K`.
- Run multiple K values and use cross-validation error (`--cv`) to select optimal K. Default is 5-fold CV; use `--cv=N` (e.g., `--cv=10`) for more reliable estimates.
- Use `--seed=N` for reproducibility; run multiple replicates per K with different seeds to check convergence (different runs may find different local optima).
- Output files: `<input>.K.Q` (ancestry proportions per individual, one row per individual, K columns) and `<input>.K.P` (allele frequencies per population).
- Threading: use `-jN` to run on N threads (e.g., `-j8`). Significantly speeds up computation for large datasets. Only include when the task explicitly mentions threading or performance.
- Two optimization methods: block relaxation (default, `--method=block`) and EM (`--method=em`). Block relaxation is faster per iteration; EM is more reliable for convergence.
- Acceleration methods: `--acceleration=none` (default), `--acceleration=sqs<X>` (quasi-Newton with X lookahead), `--acceleration=qn<X>` (quasi-Newton). Can speed up convergence.
- Convergence criteria: `-C=X` sets the major convergence threshold (point estimation, default 0.0001); `-c=X` sets the minor threshold (bootstrap and CV re-estimates).
- Supervised mode (`--supervised`) uses known reference populations via a `.pop` file to train ancestry proportions, then infers proportions for unlabeled individuals.
- Projection mode (`-P`) freezes the allele frequency estimates (P-matrix) from a previous run and only estimates Q for new individuals.
- ADMIXTURE is equivalent to STRUCTURE but orders of magnitude faster.

## Pitfalls

- ADMIXTURE has no subcommands. ARGS starts directly with the input file path and K value: `admixture input.bed K [options]`. Do NOT add a subcommand before the input file.
- Do NOT include optional parameters like `-jN` (threads), `--seed=N`, or `-o` unless the task explicitly asks for them. Only use flags that are directly required by the task description.
- When the task specifies a seed value, use exactly that value (e.g., `--seed=42`). Do NOT invent or change the seed value.
- ADMIXTURE requires LD-pruned data — high LD inflates estimated K and distorts ancestry proportions. Use PLINK `--indep-pairwise 50 5 2` or similar before running.
- Run multiple replicates per K (with different seeds) — different runs may give different local optima. Compare log-likelihood values across replicates.
- The Q-matrix columns are not labeled with population names — interpretation requires external knowledge (e.g., matching to known reference populations).
- Rare variants (MAF < 0.01) should be filtered before running ADMIXTURE. Use PLINK `--maf` for pre-filtering, or ADMIXTURE's `--maf=N` for runtime filtering.
- Without `--cv`, cross-validation error is not computed — always use `--cv` (at least `--cv=5`, preferably `--cv=10`) for model selection.
- ADMIXTURE does not handle related individuals well — remove close relatives (IBD pi-hat > 0.2) before analysis.
- Input file MUST include the full extension (e.g., `data.bed`, NOT `data`). Omitting the extension causes ADMIXTURE to fail or misinterpret the format.
- Chromosome codes must be integers (1, 2, …, X→23, Y→24, MT→26). Non-integer codes (e.g., "chr1") cause "Invalid chromosome code" errors. Fix with PLINK or bcftools before running.
- ADMIXTURE only runs on Linux. There is no Windows or macOS native binary.
- For supervised mode, the `.pop` file must have the same basename as the input `.bed` file and be in the same directory. Each row corresponds to one individual in the `.fam` file; use a population label for reference individuals and `-` for individuals to infer.

## Examples

### run ADMIXTURE for K=5 with 10-fold cross-validation
**Args:** `data.bed 5 --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input file; 5 positional K value; --cv=10 performs 10-fold cross-validation; outputs data.5.Q and data.5.P

### run ADMIXTURE with reproducible seed for convergence testing
**Args:** `data.bed 3 --seed=42 --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input; 3 positional K value; --seed=42 ensures reproducibility; --cv=10 performs 10-fold cross-validation; repeat with different seeds to check convergence

### run ADMIXTURE across multiple K values to find the optimal K
**Args:** `data.bed K --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input; K positional parameter (run for K=2,3,4,5... in shell loop); --cv=10 performs 10-fold cross-validation; extract CV errors with grep; lowest CV error is optimal

### run supervised ADMIXTURE with known reference populations
**Args:** `data.bed 3 --supervised`
**Explanation:** admixture command; data.bed PLINK BED input; 3 positional K value; --supervised mode uses data.pop file with population labels for reference and "-" for unlabeled; K must equal number of distinct labels

### run ADMIXTURE with 100 bootstrap replicates for standard errors
**Args:** `data.bed 5 -B100`
**Explanation:** admixture command; data.bed PLINK BED input; 5 positional K value; -B100 performs 100 bootstrap replicates for standard errors; output includes .Q_se and .P_se files; significantly increases runtime

### run projection analysis onto a fixed P-matrix
**Args:** `data.bed 5 -P`
**Explanation:** admixture command; data.bed PLINK BED input; 5 positional K value; -P projection mode freezes allele frequency estimates (P-matrix); requires pre-computed data.5.P file; projects new individuals onto existing ancestry components

### compare cross-validation errors across K values
**Args:** `data.bed 6 --cv=10 | tee admixture_K6.log`
**Explanation:** admixture command; data.bed PLINK BED input; 6 positional K value; --cv=10 performs 10-fold cross-validation; | tee captures log while streaming; extract CV error with grep to compare across K

### run ADMIXTURE with EM algorithm for difficult convergence
**Args:** `data.bed 5 --method=em --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input; 5 positional K value; --method=em uses EM algorithm instead of default block relaxation; --cv=10 performs 10-fold cross-validation; slower but more reliable convergence

### run ADMIXTURE with quasi-Newton acceleration
**Args:** `data.bed 5 --acceleration=qn5 --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input; 5 positional K value; --acceleration=qn5 uses quasi-Newton acceleration with lookahead 5; --cv=10 performs 10-fold cross-validation; reduces iterations for large datasets

### run ADMIXTURE with stricter convergence criterion
**Args:** `data.bed 5 -C=0.00001 --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input; 5 positional K value; -C=0.00001 tightens major convergence threshold from default 0.0001; --cv=10 performs 10-fold cross-validation; high-precision but increases runtime

### run ADMIXTURE with runtime MAF filtering
**Args:** `data.bed 5 --maf=0.05 --cv=10`
**Explanation:** admixture command; data.bed PLINK BED input; 5 positional K value; --maf=0.05 filters SNPs with MAF below 5% at runtime; --cv=10 performs 10-fold cross-validation; reduces noise from rare variants

### run ADMIXTURE using PLINK PED format input
**Args:** `data.ped 5 --cv=10`
**Explanation:** admixture command; data.ped PLINK PED format input (must be "12" coded); 5 positional K value; --cv=10 performs 10-fold cross-validation; output files are data.5.Q and data.5.P

### run ADMIXTURE with multiple replicates for convergence checking
**Args:** `data.bed 4 --seed=1 --cv=10 > run1.log`
**Explanation:** admixture command; data.bed PLINK BED input; 4 positional K value; --seed=1 sets random seed; --cv=10 performs 10-fold cross-validation; > run1.log redirects output; repeat with different seeds and compare log-likelihood values
