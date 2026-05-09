---
name: bioprov
category: Workflow Automation
description: A bioinformatics provenance tracking tool that records and visualizes data lineage, input/output relationships, and workflow execution metadata across bioinformatics pipelines. Supports export to standard provenance formats and integrates with common workflow managers.
tags:
- bioinformatics
- provenance
- lineage-tracking
- workflow
- reproducibility
- data-catalog
author: AI-generated
source_url: https://github.com/bioprov/bioprov
---

## Concepts

- bioprov operates on a graph-based data model where each file or data object becomes a node, and transformation operations become edges connecting inputs to outputs
- The tool accepts multiple input formats including SAM, BAM, VCF, FASTQ, and plain text manifest files, with each format parsed to extract standardized metadata
- Provenance records can be exported in PROV-N, JSON-LD, or RDF Turtle formats for compatibility with semantic web tools and workflow archiving systems
- Execution tracking requires specifying a workflow definition file (JSON or YAML) that maps each tool in the pipeline to its command-line arguments and dependencies

## Pitfalls

- Omitting the `--workflow` flag when running on directories containing multiple pipeline stages causes bioprov to treat each file independently, losing inter-stage relationships and producing incomplete lineage graphs
- Using relative paths for input files breaks provenance tracking when the working directory changes between recording and query time, leading to broken references in exported graphs
- Failing to specify `--output-format` defaults to PROV-N, which may be incompatible with downstream tools expecting JSON-LD, forcing re-export and wasting computation cycles

## Examples

### Record provenance for a single FASTQ file processed by BWA mem
**Args:** `--input data/sample1.fastq.gz --tool "bwa mem" --args "-R '@RG\tID:sample1\tSM:sample1' reference.fasta -" --output processed/sample1.sam`
**Explanation:** This tracks the transformation of FASTQ input through BWA mem, creating a directed edge from the input file to the generated SAM output.

### Track a multi-stage variant calling pipeline
**Args:** `--workflow pipelines/vc_pipeline.json --root-dir /project/nGS-analysis --recursive`
**Explanation:** This processes a complete workflow definition, recursively traversing all stages and automatically linking outputs from one stage to inputs of the next.

### Export provenance graph to JSON-LD for semantic web integration
**Args:** `--export provenance_output.jsonld --format json-ld --include-metadata`
**Explanation:** This outputs the complete provenance graph in JSON-LD format with embedded execution metadata for compatibility with RDF tooling.

### Query lineage for a specific output VCF file
**Args:** `--query-path results/variants.vcf --lineage-only`
**Explanation:** This returns only the direct ancestry chain leading to the specified VCF, omitting intermediate files and tool executions.

### Resume broken provenance recording using checkpoint files
**Args:** `--checkpoint bioprov.checkpoint --resume`
**Explanation:** This loads a previous checkpoint to continue provenance tracking from where interruption occurred, avoiding re-processing of completed steps.

### Generate provenance visualization as SVG graph
**Args:** `--visualize pipeline_graph.svg --format dot --layout twopi`
**Explanation:** This creates a visual representation of the entire provenance graph using the twopi layout algorithm, suitable for inclusion in documentation.

### Track provenance with custom metadata tags for sample attributes
**Args:** --input samples/*.bam --tool "samtools sort" --tags "project:ONCO001,patient:P123,tissue:tumor" --output-dir sorted/
**Explanation:** This associates custom key-value metadata with each tracked file, enabling filtered queries by project or patient identifiers during later analysis.