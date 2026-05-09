# Skill File for cfm

---

name: cfm
category: file_conversion
description: A bioinformatics command-line tool for converting between common bioinformatics file formats, managing file metadata, and batch-processing multiple input files with customizable output options.
tags: [file-format, conversion, bioinformatics, data-processing, batch]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cfm

## Concepts

- cfm operates as a file format converter and processor, accepting stdin input or files via positional arguments, and outputting converted data to stdout or files specified with -o/--output.
- Supported input formats include FASTA, FASTQ, VCF, BED, GTF, and SAM/BAM, with automatic format detection based on file extension or explicit -f/--format specification.
- The tool processes data line-by-line or in streaming mode, preserving sequence identifiers, quality scores, and metadata embedded in the original file.
- Output format is controlled via -f/--format flag, and compression is handled automatically for .gz, .bgz, or .bz2 extensions based on output filename.
- Batch processing multiple files uses wildcards in input paths or a file containing a list of input files specified with --input-list.

## Pitfalls

- Omitting the -f/--format flag when working with non-standard file extensions causes automatic format detection to fail, resulting in parse errors or silent data corruption.
- Specifying an output path without the -o/--outdir flag for batch processing writes all converted files to the current directory, overwriting existing files with matching names.
- Using mismatched input and output formats (e.g., converting FASTQ to FASTA without handling quality scores) discards quality data without warning unless --strict flag is enabled.
- Forgetting to create the output directory before batch processing triggers a "directory not found" error and halts the entire conversion job.
- Applying lossy compression options like --compress-level 9 on already-compressed .gz inputs wastes compute time with negligible size reduction and increases processing latency.

## Examples

### Convert a single FASTQ file to FASTA format

**Args:** -f fasta input.fastq -o output.fasta
**Explanation:** Reads input.fastq in FASTQ format and writes sequences in FASTA format to output.fasta, discarding quality scores.

### Batch convert all FASTQ files in a directory to FASTQ.gz

**Args:** *.fastq -o processed/ --compress-level 6
**Explanation:** Uses shell glob expansion to select all .fastq files and converts them to gzip-compressed FASTQ format in the processed directory.

### Stream-convert VCF to BCF for efficient storage

**Args:** --input-format vcf --output-format bcf input.vcf -o output.bcf
**Explanation:** Converts a VCF file to binary BCF format, which provides faster parsing and reduced disk usage for large variant call files.

### Convert GTF to BED12 format with explicit format specification

**Args:** -f bed12 annotation.gtf -o annotation.bed
**Explanation:** Explicitly specifies output as BED12 format to retain gene structure information including exons, transcripts, and coding frames when converting GTF annotations.

### Process multiple files listed in a text file

**Args:** --input-list files.txt --output-dir converted/
**Explanation:** Reads file paths from files.txt (one path per line) and converts each file to its specified output format, writing results to the converted directory while preserving original filenames.