---
name: avro-cwl
category: Bioinformatics / Data Serialization
description: A tool for converting and validating Apache Avro schemas against Common Workflow Language (CWL) type definitions. Used in CWL-based bioinformatics workflows to ensure type compatibility between workflow inputs/outputs and Avro-serialized data files.
tags:
  - avro
  - cwl
  - schema-validation
  - bioinformatics
  - data-serialization
  - workflow
author: AI-generated
source_url: https://github.com/common-workflow-language/avro-cwl
---

## Concepts

- **Avro Schema Input**: The tool accepts Avro schema files (`.avsc` format in JSON) defining the structure of bioinformatics data records such as genomic variants, read alignments, or expression matrices. Schemas must follow Avro's specification with named types, fields, and optional/default values.
- **CWL Type Output**: Generates corresponding CWL type definitions using CWL's `Record` type with named fields matching the Avro schema fields. This enables CWL workflow steps to declare inputs/outputs that are valid Avro-serialized data containers.
- **Bidirectional Conversion**: Supports both Avro-to-CWL conversion (generating CWL type definitions from existing Avro schemas) and CWL-to-Avro conversion (generating Avro schemas from CWL `CommandLineTool` input/output declarations). Field types are mapped according to the Avro-CWL type correspondence table.
- **Schema Validation**: Validates that Avro schemas conform to CWL's type system constraints, rejecting schemas with complex unions, recursive types, or logical types not representable in CWL without explicit mapping options.

## Pitfalls

- **Missing Schema File**: Forgetting to specify the input schema file with the appropriate flag causes the tool to fail with an ambiguous error. Without `--schema`, the tool cannot determine what Avro type to convert or validate.
- **Type Mismatch in Field Mapping**: Using CWL type names that don't have a direct Avro equivalent (like `File` or `Directory` without proper binding) results in silent data loss or runtime errors when the generated workflow attempts to deserialize files.
- **Incompatible Union Types**: Avro schemas with union types containing more than two members cannot be directly represented in CWL without explicit disambiguation. The tool may reject these or generate invalid CWL that fails validation.
- **Version Mismatch**: Using an Avro schema written for a newer Avro specification with an older version of avro-cwl causes parsing errors. Ensure the tool version matches the schema specification version.

## Examples

### Convert an Avro schema to a CWL type definition
**Args:** `--schema variants.avsc --output variants.cwl`
**Explanation:** Reads the Avro schema file `variants.avsc` containing a genetic variant record definition and outputs an equivalent CWL type definition in `variants.cwl` for use in CWL workflow inputs.

### Validate an Avro schema against CWL requirements
**Args:** `--schema readalignment.avsc --validate-only`
**Explanation:** Checks if the Avro schema `readalignment.avsc` can be correctly represented in CWL without generating output, reporting any incompatibility issues.

### Generate CWL type from Avro schema with specific namespace
**Args:** `--schema expression.avsc --namespace http://example.org/expression --output expression.cwl`
**Explanation:** Converts the expression matrix Avro schema to CWL while applying the specified namespace URI to the generated CWL type for proper namespacing.

### Convert CWL type back to Avro schema
**Args:** `--cwl-type GeneAnnotation.cwl --output-to-avro`
**Explanation:** Takes a CWL Record type definition and generates an equivalent Avro schema file, useful for round-trip conversions between schema formats.

### Specify output format as JSON
**Args:** `--schema sample.avsc --output-format json --output sample.cwl`
**Explanation:** Generates the CWL type definition in JSON format (the default), ensuring compatibility with CWL parsers that require JSON rather than YAML representation.