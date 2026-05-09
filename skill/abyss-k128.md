---
name: abyss-k128
category: sequence-analysis
description: ABySS k-mer size estimator that suggests an optimal k-mer length for de Bruijn graph genome assembly based on input read statistics, coverage depth, and sequence quality metrics.
tags:
  - genome-assembly
  - k-mer-analysis
  - de-bruijn-graph
  - paired-end
  - read-length
  - assembly-parameter
author: AI-generated
source_url: https://github.com/bcgsc/abyss
---

## Concepts

- **k-mer length determination**: abyss-k128 analyzes input sequences to compute an optimal odd k-mer length that balances assembly contiguity (larger k reduces branching in the de Bruijn graph) against connectivity (smaller k preserves read overlaps). The output is typically a single integer representing the recommended k value.

- **Input format compatibility**: The tool accepts FASTA, FASTQ, or ACE file formats as input. It can process either raw sequencing reads or already-assembled contigs. File input is specified via stdin or file arguments, with read length and coverage statistics derived automatically from the data.

- **Coverage-aware estimation**: The estimator considers expected k-mer coverage, which should approximate the raw sequencing coverage when k is sufficiently small. If input coverage is too low, the tool may recommend a smaller k value to maintain enough k-mer redundancy for reliable graph construction.

- **Relationship to read length**: The optimal k must be strictly less than the read length minus any required overlap. For paired-end data, the recommended k is often set to approximately (read_length / 3) or less to ensure sufficient overlap between read pairs during extension.

- **Companion binary dependency**: While abyss-k128 produces the k value, actual assembly requires abyss-pe (the main ABySS assembler) or abyss-k128-build for building k-mer indices. The estimated k value is passed directly to these tools via command-line flags.

## Pitfalls

- **Using k larger than read length minus overlap**: A k value approaching or exceeding the read length eliminates meaningful overlaps, causing the de Bruijn graph to fragment and producing poor N50 statistics. Always ensure k ≤ (read_length − overlap_requirement).

- **Ignoring odd k requirements**: ABySS and many de Bruijn assemblers require odd k values to avoid internal palindromic k-mers that complicate read pairing. If abyss-k128 outputs an even number, increment it by 1 rather than rounding down.

- **Misinterpreting output for different input types**: The k estimate is optimized for the input provided. Using the same k for raw reads versus assembled contigs may yield suboptimal results because contigs already have collapsed redundant sequences, altering the effective coverage.

- **Neglecting coverage in the final k selection**: A low-coverage genome with a recommended large k may fail to assemble due to insufficient k-mer redundancy. The tool outputs a statistical optimum, but practical assembly success depends on empirical validation with different k values.

- **Feeding corrupted or trimmed input without adjustment**: If adapters or low-quality bases have been trimmed from reads, the effective read length changes. Failing to account for this trimmed length when interpreting the k recommendation can result in k values that exceed usable overlap.

## Examples

### Estimate optimal k-mer size from paired-end FASTQ reads

**Args:** `reads.fastq -l 100 -e 0.98`
**Explanation:** Providing paired-end reads with an effective read length of 100 bp and error rate of 0.98 allows abyss-k128 to calculate a k value that maintains sufficient overlap between read pairs during assembly, yielding a recommendation typically in the range of 31–47.

### Estimate k using assembled contigs as input

**Args:** `contigs.fasta -e 0.995`
**Explanation:** When assembled contigs are provided, abyss-k128 derives the k estimate from contig statistics (length distribution, redundancy) rather than raw read overlap, useful for iterative parameter refinement in multi-pass assembly strategies.

### Generate k value with explicit minimum coverage threshold

**Args:** `raw_reads.fq -l 150 -c 10`
**Explanation:** Specifying a minimum coverage of 10 ensures the recommended k does not drop below a threshold that would cause excessive branching, while still respecting practical assembly constraints for the given read length.

### Use standard input with piped sequence data

**Args:** `-l 250`
**Explanation:** Accepting input from stdin via pipe (e.g., `cat reads.fq | abyss-k128 -l 250`) allows integration into bioinformatics pipelines where intermediate files are not written to disk, though the effective read length must still be specified explicitly.

### Verify k estimate before running full assembly

**Args:** `sample_reads.fasta -l 100`
**Explanation:** Running the estimator on a subset of reads before committing to full assembly avoids wasting compute resources on an unsuitable k value, particularly when experimenting with different sequencing technologies or library preparations.