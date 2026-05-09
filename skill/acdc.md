I need to first acknowledge that I don't have reliable, specific knowledge about a bioinformatics tool called "acdc" with a documented command-line interface, companion binaries, and established usage patterns. Creating an accurate skill file requires verified information about a tool's data model, I/O formats, and behaviors.

However, since the task explicitly requests this tool, I'll make a reasonable attempt based on potential bioinformatics tool patterns. If "acdc" refers to a specific tool you're working with, please verify or provide additional context, and I can revise accordingly.

Given the requirements, here's a generic skill-file structure that demonstrates the proper format:

---
name: acdc
category: bioinformatics
description: A tool for [specific bioinformatics task: e.g., sequence analysis, variant calling, alignment processing]. Handles [input format] and produces [output format].
tags: [bioinformatics, analysis, formatting]
author: AI-generated
source_url: [URL to official documentation]
---

## Concepts

- **Data Model**: [Description of the tool's internal data structures, e.g., sequences stored as strings with quality scores, genomic intervals as BED-style coordinates]
- **Input Formats**: [Common input file types, e.g., FASTA, FASTQ, VCF, BAM]
- **Output Formats**: [Generated output types, e.g., TSV, JSON, binary]
- **Key Behaviors**: [Core functionality, e.g., parallel processing, streaming, indexed access]
- **I/O Handling**: [How the tool manages file reading/writing, buffering, compression]

## Pitfalls

- **Incorrect file format**: [Consequence: tool failure, silent errors, or corrupted output]
- **Missing index files**: [Consequence: performance degradation or complete failure]
- **Memory mismanagement**: [Consequence: out-of-memory errors, especially with large datasets]
- **Parameter misuse**: [Consequence: incorrect results or unexpected behavior]
- **Version incompatibilities**: [Consequence: broken workflows when tool versions change]

## Examples

### Process a single input file with default settings
**