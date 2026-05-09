---
name: cgmlst-dists-py
category: Bioinformatics / Phylogenetics & Typing
description: Computes pairwise core-genome Multilocus Sequence Typing (cgMLST) distances between bacterial genome assemblies based on gene-by-gene allelic profiles.
tags: [cgmlst, core-genome, bacterial-typing, distance-matrix, allelic-profiling, phylogenetics, pubmlst]
author: AI-generated
source_url: https://github.com/tseemann/cgmlst-dists
---

## Concepts

- **Input accepts FASTA assemblies and GenBank files**: The tool scans genome assemblies for the presence of loci defined in a cgMLST scheme. Loci are identified by sequence similarity against known allele sequences; exact matches yield a distance of 0, while absent or divergent loci contribute to the pairwise distance.
- **Distance computation is allele-based, not SNP-based**: Each locus contributes at most 1 to the distance if alleles differ (including missing loci). The total distance is the sum of allelic differences across shared loci, divided by the number of comparable loci to produce a proportional distance metric.
- **A scheme file defines the target loci and their reference alleles**: Schemes (e.g., *Salmonella*.enterica, *Campylobacter* jejuni) are available from the PubMLST database. The scheme file contains locus identifiers and their canonical allele sequences; using the correct scheme is essential for accurate distance calculation.
- **Output formats include PHYLIP, CSV, and plain text**: The distance matrix can be produced in multiple formats compatible with downstream phylogenetic tools. PHYLIP format is suitable for PHYLIP or IQ-TREE; CSV is convenient for spreadsheets or R analysis.
- **Missing loci are handled explicitly**: Isolates missing a locus from the scheme contribute to the denominator (total comparable loci) but not the numerator (differences), ensuring proportional distances account for incomplete sampling.

## Pitfalls

- **Using the wrong or missing scheme file produces meaningless distances**: Without a scheme, the tool falls back to arbitrary locus calling, yielding distances that do not reflect established cgMLST standards. Always specify the correct scheme via `--scheme` or `--scheme-file` for reproducible, comparable results.
- **Input assemblies with excessive fragmentation inflate distances**: Highly fragmented assemblies (hundreds of contigs) may cause loci to be split across contigs or missed entirely, artificially increasing reported distances. Preprocess assemblies with quality trimming or assembly improvement tools before running cgMLST analysis.
- **Allelic mismatches from paralogous loci inflate distances**: Genes duplicated in some genomes but not others create false positive allelic differences. Visual inspection of high-distance pairs against reference annotations is recommended to identify paralogy issues.
- **Specifying an output format incompatible with downstream tools wastes analysis time**: If the distance matrix is destined for a specific tool (e.g., PhyML, IQ-TREE), ensure the chosen format is supported. Some tools require PHYLIP with specific formatting; manual reformatting after the fact is error-prone.
- **Omitting `--min-cover` threshold leads to overcounting of missing loci**: Isolates with many missing loci still contribute to the denominator, skewing proportional distances toward lower values. Setting a minimum locus coverage (e.g., 90%) via `--min-cover` ensures only well-sampled comparisons are included.

## Examples

### Compute cgMLST distances between two *Salmonella* assemblies using the S. enterica scheme

**Args:** `--scheme Salmonella enterica --fasta sample1.fasta sample2.fasta --output dist_matrix.phylip`
**Explanation:** Using the named PubMLST scheme, the tool identifies cgMLST loci in both FASTA assemblies and outputs a pairwise distance matrix in PHYLIP format.

### Calculate distances for a batch of assemblies in CSV format

**Args:** `--scheme "Campylobacter jejuni" --csv --output all_isolates.csv --input-dir ./assemblies/`
**Explanation:** Processing all FASTA files in a directory, the tool computes a full pairwise distance matrix and exports results as a comma-separated table for spreadsheet analysis.

### Use a custom local scheme file with threshold filtering

**Args:** `--scheme-file ./custom_scheme.tsv --min-cover 0.85 --phylip --output custom_distances.phy`
**Explanation:** A locally curated scheme file (TSV format) is used to call alleles, and only locus comparisons where at least 85% of loci are present in both isolates contribute to the final proportional distance.

### Compute distances ignoring genomes with excessive missing loci

**Args:** `--scheme "Escherichia coli" --min-loci 1500 --phylip --output ecoli_distances.phy --input assembly1.gbk assembly2.gbk assembly3.gbk`
**Explanation:** The tool discards any genome missing more than the specified minimum number of scheme loci before distance calculation, ensuring only high-quality, well-characterized isolates influence the matrix.

### Export distances for rapid neighbour-joining tree building

**Args:** `--scheme "Listeria monocytogenes" --phylip --output lm_dist.phy --input genomes/*.fasta`
**Explanation:** The PHYLIP-formatted output is directly compatible with neighbour-joining tree builders like RapidNJ or QuickTree, allowing immediate phylogenetic visualization without reformatting.

### Use a specific number of threads for faster computation on large datasets

**Args:** `--scheme "Neisseria meningitidis" --threads 8 --phylip --output nm_dist.phy --input-dir ./nga_isolates/`
**Explanation:** Parallelized allele calling across 8 threads reduces wall-clock time on large isolate sets, while the standard PHYLIP output preserves compatibility with cgMLST tree reconstruction pipelines.

### Generate a distance matrix while logging detailed allele mismatches

**Args:** `--scheme "Staphylococcus aureus" --verbose --csv --output sa_dist.csv --input genome1.fa genome2.fa genome3.fa`
**Explanation:** Verbose mode records per-locus allelic differences in the log, helping identify specific genes driving high distances between isolate pairs for downstream epidemiological investigation.