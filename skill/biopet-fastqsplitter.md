---
name: biopet-fastqsplitter
category: sequence-analysis/fastq-processing
description: A tool for splitting large FASTQ files into smaller, manageable parts for parallel processing or distribution across computational nodes. Part of the BIOPET bioinformatics pipeline toolkit.
tags:
  - fastq
  - ngs
  - sequencing
  - splitting
  - parallel-processing
  - bioinformatics
author: AI-generated
source_url: https://biopet.github.io/fastqsplitter/
---

## Concepts

- **Input/Output Model**: biopet-fastqsplitter reads one or more input FASTQ files and generates multiple output FASTQ chunks, each containing a subset of reads. Output files are typically numbered sequentially (e.g., sample_R1.part_001.fastq, sample_R1.part_002.fastq).
- **Splitting Strategies**: The tool supports multiple splitting modes including dividing by total number of output files (`--num-split` or `-n`), splitting by number of reads per chunk (`--reads-per-file` or `-r`), or splitting by byte size (`--bytes-per-file`). The appropriate strategy depends on downstream tool requirements.
- **Quality and Metadata Preservation**: When splitting FASTQ files, biopet-fastqsplitter preserves all quality score lines, read identifiers, and sequence annotations intact within each chunk. No filtering or trimming is performed during the split operation itself.
- **Paired-End Read Handling**: For paired-end sequencing data, provide both mate files (R1 and R2) to ensure synchronized splitting where corresponding read pairs are distributed to the same output chunk indices. This maintains pairing integrity for downstream analysis.
- **Compression Support**: The tool automatically handles gzipped (.gz) and bzipped (.bz2) input FASTQ files, producing output chunks with matching compression formats. Explicit format flags may be needed when input files lack standard extensions.

## Pitfalls

- **Mismatched Paired-End Splits**: Providing only one file of a paired-end set causes the tool to split reads independently, breaking read pair correspondence. Downstream aligners expecting mate pairs will fail or produce incorrect results. Always provide both R1 and R2 files when processing paired-end data.
- **Incorrect Split Count**: Specifying a split count larger than the number of reads in the input file results in empty output files, wasting storage and causing downstream tools to fail with empty input errors. Verify input file sizes before specifying split parameters.
- **Ignoring Compression Format**: Failing to specify compressed input format when FASTQ files lack .gz extensions causes the tool to read raw bytes incorrectly, producing corrupted output files with malformed quality strings and read sequences.
- **Output Directory Conflicts**: Splitting multiple input files into the same output directory without unique prefixes causes filename collisions where later splits overwrite earlier ones. Always use distinct output directories or filename prefixes for each input sample.
- **Memory Consumption on Large Files**: Specifying `--reads-per-file` with very small values creates an excessive number of tiny output files, consuming disk inodes and causing file system performance degradation. Choose split values that produce files of reasonable minimum size (typically >100MB).

## Examples

### Split a single uncompressed FASTQ file into 4 equal chunks
**Args:** `-i sample_R1.fastq -o ./output_dir -n 4`
**Explanation:** This splits the input FASTQ file into 4 roughly equal output files containing one-quarter of the total reads each, stored in the specified output directory.

### Split a gzipped FASTQ file by number of reads per chunk
**Args:** `-i sample_R1.fastq.gz --reads-per-file 1000000 -o ./output_dir`
**Explanation:** This creates output chunks containing exactly 1,000,000 reads each, which is useful when downstream tools have memory constraints or processing limits.

### Split paired-end FASTQ files while maintaining read pairing
**Args:** `-i sample_R1.fastq.gz -i sample_R2.fastq.gz -o ./paired_output -n 8`
**Explanation:** This splits both mate files into 8 synchronized chunks, ensuring that read pairs remain aligned by their chunk index for paired-end downstream analysis.

### Split multiple input files with a base output prefix
**Args:** `-i sample1_R1.fastq -i sample2_R1.fastq -o ./batch_output --prefix sample`
**Explanation:** This processes two input files, using the prefix option to generate uniquely named output files (sample_1_part_001.fastq, sample_2_part_001.fastq) avoiding filename conflicts.

### Split a large FASTQ file and specify output format as gzipped
**Args:** `-i large_sample.fastq --output-format gzip -o ./compressed_output -n 16`
**Explanation:** This splits the input file and compresses all output chunks as gzipped files, useful for reducing storage requirements when downstream tools support compressed input.