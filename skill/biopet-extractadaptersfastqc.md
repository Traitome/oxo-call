---
name: biopet-extractadaptersfastqc
category: bioinformatics/sequence-preprocessing
description: Extracts detected adapter sequences from FASTQC analysis reports for use in downstream quality trimming tools. Parses FASTQC overrepresented sequences, adapter content, and reports identified library adapters in standard formats.
tags:
  - fastqc
  - adapters
  - trimming
  - preprocessing
  - ngs
  - sequence-quality
author: AI-generated
source_url: https://github.com/biopet/biopet
---

## Concepts

- **Input Formats**: The tool accepts FASTQC output in multiple forms—FASTQC zip archives (containing the `fastqc_data.txt` and `adapter_content.txt`), the raw FASTQC data directory, or FASTQC JSON/HTML reports. The most reliable input is the FASTQC zip file or directory containing the full analysis results.
- **Output Formats**: Extracted adapter sequences are output as a FASTA file listing all detected adapter sequences with their names and a text summary. The FASTA format includes the adapter name in the header (e.g., `>Illumina Universal Adapter`) followed by the sequence on the next line.
- **Detection Logic**: The tool parses FASTQC's "Overrepresented Sequences" and "Adapter Content" modules to identify adapter sequences that appear above the detection threshold. It categorizes adapters by library type (Illumina, TruSeq, Nextera, Polya) and reports sequences with their abundance metrics.
- **Companion Binary**: `biopet-extractadaptersfastqc` is typically used in pipelines that combine FASTQC analysis followed by adapter trimming with tools like Cutadapt or Trimmomatic. The extracted adapter sequences can be piped directly to trimming tools.

## Pitfalls

- **Using FASTQC reports from incompatible sequencing chemistries**: Adapter sequences detected in RNA-seq libraries (poly-A tails, rRNA depletion adapters) differ from DNA-seq adapters. Extracting adapters from the wrong library type will introduce incorrect sequences into trimming, leading to either over-trimming (removing valid reads) or under-trimming (leaving adapter contamination).
- **Ignoring the minimum abundance threshold**: FASTQC only flags sequences as "overrepresented" if they meet a minimum percentage (default 0.1% of total reads). Rare adapter contamination below this threshold will not appear in the extracted list, potentially leaving undetected adapters in final outputs.
- **Mixing samples from different library preparation runs**: Running the tool on a combined FASTQC report or using a single output for multiple samples prepared with different kits will produce a hybrid adapter list that may not match any individual sample, reducing trimming efficiency.
- **Relying solely on FASTQC without verification**: FASTQC's adapter detection uses k-mer matching and may miss novel or custom adapters. Always visually inspect the Adapter Content plot in the FASTQC report before assuming all adapters are correctly identified.

## Examples

### Extract adapter sequences from a single FASTQC zip file

**Args:** `--input sample1_fastqc.zip --output adapters.fasta`
**Explanation:** This command reads the FASTQC zip archive for sample1 and extracts all detected adapter sequences into a FASTA file named adapters.fasta. The zip file contains all necessary modules including adapter_content.txt.

### Extract adapters from multiple FASTQC directories into a combined list

**Args:** `--input sample1_fastqc --input sample2_fastqc --output combined_adapters.fasta --merge`
**Explanation:** Processes two separate FASTQC output directories and merges the detected adapter sequences into a single output file, removing duplicates. Use this when multiple samples share the same library preparation kit.

### Specify a minimum sequence abundance threshold

**Args:** `--input sample_fastqc.zip --output adapters.fasta --threshold 0.5`
**Explanation:** Only extracts adapter sequences that represent at least 0.5% of total reads, filtering out low-abundance hits that may be sequencing artifacts rather than true adapters.

### Output adapters in text format instead of FASTA

**Args:** `--input sample_fastqc.zip --output adapter_list.txt --format txt`
**Explanation:** Writes the extracted adapter sequences as plain text (one sequence per line) rather than FASTA format, suitable for use with command-line tools that accept simple text lists.

### Extract adapters for a specific library type

**Args:** `--input sample_fastqc.zip --output truseq_adapters.fasta --library-type truseq`
**Explanation:** Filters the output to only include TruSeq adapter sequences, ignoring other detected adapters. Useful when you know the exact library kit used and only want relevant sequences.