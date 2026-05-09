---
name: altamisa
category: bioinformatics/microarray
description: AltamIS (Annotation Library for Microarray and Sequencing) parses and processes ISA-Tab format files used in metabolomics, proteomics, and transcriptomics studies. Converts between ISA-Tab, JSON, and RDF formats for systems biology data integration.
tags:
  - isa-tab
  - metabolomics
  - proteomics
  - transcriptomics
  - metadata
  - annotation
  - data-integration
  - systems-biology
author: AI-generated
source_url: https://github.com/ISA-tools/AltamIS
---

## Concepts

- AltamIS operates on the ISA (Investigation-Study-Assay) data model, which organizes experimental metadata into three interconnected tab-delimited files: investigation.txt (overall study metadata), study.txt (sample and protocol details), and assay.txt (measurement data like expression or metabolite levels).

- The tool accepts ISA-Tab input files and can output them as ISA-Tab (preservation), JSON (programmatic access), or RDF/TTL (semantic web integration). The `--output-format` flag controls the serialization target for downstream data pipelines.

- Column relationships in ISA-Tab files are defined by "Characteristic" and "Factor" fields that link assay measurements to study variables. AltamIS validates these referential integrity constraints during parsing and reports missing or duplicate links as warnings.

- The software maintains an in-memory graph representation of all ISA entities during processing, which enables query operations like retrieving all assays belonging to a specific study or extracting samples with particular metadata attributes.

## Pitfalls

- Specifying incorrect column headers in ISA-Tab files (e.g., "Characterstic" instead of "Characteristic") causes AltamIS to silently ignore those columns, resulting in incomplete metadata in output files and lost annotation data in downstream analyses.

- Mixing DOS-style (CRLF) and Unix-style (LF) line endings in a single ISA-Tab file breaks parsing and produces truncated output or throws "unexpected end of file" errors, especially when the file is edited across different operating systems.

- Using relative file paths for output (`-o output/`) when running AltamIS from different working directories results in files being written to unexpected locations or permission denied errors, as the directory may not exist or may be on a read-only filesystem.

- Failing to specify the `--compression gzip` flag when processing large ISA-Tab files wastes disk space and I/O bandwidth; output files can be 5-10x larger than necessary for text-based tabular data.

- ISA-Tab files with duplicate sample names in the same study file cause AltamIS to create ambiguous entity references in JSON/RDF output, which breaks tools that rely on unique identifiers for sample tracking in data integration workflows.

## Examples

### Validate an ISA-Tab directory structure
**Args:** `validate -i /data/experiment/isa-tab/`
**Explanation:** The validate command checks that all three required ISA files exist and that referential links between investigation, study, and assay files are consistent before any conversion or export operations.

### Convert ISA-Tab to JSON for programmatic access
**Args:** `convert -i /data/experiment/isa-tab/ -o /data/output/study.json --to json --pretty`
**Explanation:** The convert command transforms the hierarchical ISA-Tab files into a flat JSON structure suitable for scripting environments, with `--pretty` enabling human-readable formatting for debugging.

### Export ISA-Tab as RDF for semantic data integration
**Args:** `export -i /data/metabolomics/ -o study.ttl --format rdf --base-uri http://example.org/ontologies/`
**Explanation:** The export command serializes ISA metadata as RDF triples using the provided base URI namespace, enabling Linked Data compatibility and SPARQL queries across distributed datasets.

### Extract assay files for a specific study factor
**Args:** `extract -i /data/experiment/ --filter "FactorValue[Treatment]=Control" -o control-assays/`
**Explanation:** The extract command subsets assay rows matching the specified factor criteria, creating a filtered ISA-Tab directory containing only control samples for differential analysis pipelines.

### Compress output files to save storage space
**Args:** `convert -i /data/large-study/ -o archive/study.json --compression gzip --level 9`
**Explanation:** The compression flag creates gzipped output at maximum compression level, reducing storage requirements by 70-90% for large multi-assay studies while maintaining full decompressibility.