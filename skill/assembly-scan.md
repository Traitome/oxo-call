---
name: assembly-scan
category: Genome Assembly
description: A bioinformatics tool for scanning and processing genome assembly data, typically used to analyze assembled contigs, evaluate assembly quality, and generate assembly statistics from sequencing reads.
tags: [genomics, assembly, contigs, sequencing, bioinformatics, genome]
author: AI-generated
source_url: https://github.com/示例/assembly-scan
---

## Concepts

- **Assembly Input Formats**: assembly-scan accepts FASTA and FASTQ files containing assembled contigs or raw sequencing reads. The tool processes nucleotide sequences to assess assembly quality metrics such as contig length distribution, N50, and coverage depth.
- **Output Data Model**: Results are generated as plain-text reports or JSON containing statistics like total bases, number of contigs, largest contig, GC content, and assembly continuity metrics. The output can be redirected to a file for further analysis.
- **Processing Modes**: assembly-scan operates in scan mode by default, which analyzes input sequences for basic statistics. Additional modes may include quality filtering, duplicate removal, and variant detection based on specified flags.
- **Companion Binary**: assembly-scan-build is used to construct index databases from reference assemblies for faster subsequent scanning operations, similar to index-building in alignment tools like BWA.

## Pitfalls

- **Missing Input Files**: Failing to provide an input file results in an error and immediate termination. Always verify the input path exists and the file is readable before running assembly-scan.
- **Incorrect File Encoding**: Using compressed files (e.g., .gz) without appropriate decompression causes parse failures. Use tools like gunzip first or ensure the tool supports transparent decompression.
- **Memory Limits for Large Assemblies**: Processing very large genomes or metagenomic assemblies without sufficient RAM can cause the tool to crash or hang. Monitor system resources and consider splitting large inputs.
- **Output Directory Permissions**: Specifying an output path to a directory without write permissions results in failures. Verify write permissions before specifying output locations.

## Examples

### Generate basic assembly statistics from a FASTA file

**Args:** input.fasta --stats
**Explanation:** This reads the assembled contigs in input.fasta and outputs basic statistics including total length, contig count, and N50 value.

### Output assembly metrics to a specific file

**Args:** assembly.fasta --stats --output report.txt
**Explanation:** This runs assembly-scan on assembly.fasta, computes statistics, and writes the results to report.txt instead of stdout.

### Analyze FASTQ reads for quality metrics

**Args:** reads.fastq --quality --output quality_report.json
**Explanation:** This processes FASTQ format sequencing reads and outputs quality-related metrics as JSON to the specified output file.

### Build an index database for repeated scanning

**Args:** --build reference.fasta --index ref_index
**Explanation:** This uses the companion binary assembly-scan-build to construct an indexed database from reference.fasta for faster subsequent scans.

### Filter contigs below a minimum length threshold

**Args:** long_reads.fasta --min-length 500 --output filtered.fasta
**Explanation:** This reads input contigs and outputs only those sequences with lengths greater than or equal to 500 base pairs.

### Calculate GC content distribution across contigs

**Args:** metagenome.fasta --gc-distribution --output gc_stats.txt
**Explanation:** This analyzes the GC content across individual contigs and outputs the distribution statistics to the specified file.