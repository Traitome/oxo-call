---
name: bio_hansel
category: Pathogen Subtyping / Genomics
description: A k-mer based bioinformatics tool for subtyping bacterial pathogens from high-throughput sequencing reads. Designed for foodborne pathogen surveillance, it identifies subtypes by matching read k-mers against pre-built scheme databases.
tags: [pathogen-typing, k-mers, wgs, surveillance, food-safety, bacterial-subtyping, illumina]
author: AI-generated
source_url: https://github.com/biohansel/bio_hansel
---

## Concepts

- **K-mer Subtyping Algorithm:** bio_hansel extracts k-mers from input FASTQ reads and matches them against a pre-built scheme database containing k-mers unique to specific pathogen subtypes. The subtype with the most matching k-mers above the minimum coverage threshold is reported.

- **Scheme Databases:** Subtyping requires a scheme (built with `bio_hansel-build`) containing known subtype k-mers. Each scheme is organism-specific (e.g., Salmonella, Listeria monocytogenes) and includes metadata such as subtype names, lineages, and reference sequences.

- **Input Formats:** Accepts single-end or paired-end Illumina FASTQ files (`.fq`, `.fastq`, optionally gzipped as `.fq.gz`, `.fastq.gz`). Read files can be specified with multiple `-1/-2` pairs for pooled samples.

- **Confidence Scoring:** Results include a log-likelihood ratio test comparing the best match to the second-best, plus a coverage percentage indicating what fraction of scheme k-mers were found in the sample.

- **Minimum K-mer Coverage:** The `-c/--min-cov` flag sets the minimum number of times a k-mer must be observed for it to be counted as present, reducing false positives from sequencing errors.

## Pitfalls

- **Wrong Scheme for Organism:** Using a scheme built for a different pathogen (e.g., Listeria scheme for Salmonella reads) will produce nonsensical or misleading subtype calls. Always verify the scheme organism matches your input sample.

- **Low Coverage Samples:** Samples with insufficient sequencing depth produce low k-mer coverage, leading to ambiguous or incorrect subtype calls. bio_hansel reports coverage percentage—values below 80% should be interpreted with caution.

- **Sequencing Technology Mismatch:** bio_hansel is optimized for Illumina short-read data. Using ONT or PacBio long reads may cause poor k-mer matching due to higher per-base error rates, resulting in false negatives.

- **Missing Required Options:** Forgetting to specify the scheme with `-s/--scheme` will cause the tool to fail. The scheme file is mandatory for subtyping operations.

- **Conflicting Read Inputs:** Mixing single-end and paired-end flags incorrectly (e.g., specifying `-1` without `-2` for paired data) can lead to partial analysis or errors.

## Examples

### Type a single FASTQ sample against a subtyping scheme

**Args:** `-s scheme.txt -1 reads.fq.gz -o output.tsv`
**Explanation:** Reads the specified scheme database and queries single-end FASTQ reads to identify the pathogen subtype, writing results to a tab-separated output file.

### Type paired-end FASTQ reads against a scheme

**Args:** `-s listeria_scheme.txt -1 read1.fq.gz -2 read2.fq.gz -o results.tsv`
**Explanation:** Uses both read files to increase k-mer detection sensitivity for paired-end data, providing more comprehensive subtype coverage than single-end alone.

### Set minimum k-mer coverage threshold to reduce false positives

**Args:** `-s scheme.txt -1 reads.fq.gz -c 5 -o output.tsv`
**Explanation:** Requires each k-mer to be observed at least 5 times before counting it as present, filtering out spurious k-mers from sequencing errors.

### Output detailed JSON results with all match information

**Args:** `-s scheme.txt -1 reads.fq.gz -o results.json --json`
**Explanation:** Produces machine-parseable JSON output containing all subtype candidates, k-mer matches, coverage stats, and log-likelihood scores for downstream analysis.

### Analyze multiple read files in a single run

**Args:** `-s scheme.txt -1 sample1_R1.fq.gz sample2_R1.fq.gz -2 sample1_R2.fq.gz sample2_R2.fq.gz -o combined.tsv`
**Explanation:** Allows batch processing of multiple samples simultaneously by specifying multiple files for each read direction, with results merged into a single output table.

### Specify a custom k-mer size for non-default schemes

**Args:** `-s custom_scheme.txt -1 reads.fq.gz -k 31 -o output.tsv`
**Explanation:** Uses a 31-base k-mer size instead of the default, which is required for schemes built with non-standard k-mer lengths during scheme creation.