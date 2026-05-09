---
name: avro-python3
category: Data Serialization / Bioinformatics Utilities
description: Python library for reading, writing, and manipulating Apache Avro data files and schemas. Provides functionality for schema parsing, data validation, and interoperation with Hadoop ecosystems in bioinformatics pipelines.
tags:
  - avro
  - serialization
  - bioinformatics
  - data-format
  - schema
  - hadoop
author: AI-generated
source_url: https://pypi.org/project/avro-python3/
---

## Concepts

- **Avro Schema Model**: Avro uses JSON-based schemas to define data structure. Schemas can be embedded in data files (for self-describing records) or provided separately. The schema defines fields, types (primitive: null, boolean, int, long, float, double, bytes, string; complex: record, enum, array, map, union, fixed), and optional/default values.
- **Data File I/O**: Avro data files are binary containers that store serialized Avro records along with a schema. Files begin with a sync marker and header, enabling fast splitting for distributed processing. Use `avro.datafile.DataFileReader` and `DataFileWriter` for controlled reading/writing with specific schema versions.
- **Schema Resolution and Aliases**: When reading data written with a different schema, Avro uses field names and aliases to map between writer and reader schemas. Reader schema fields can have defaults to fill missing writer fields; extra writer fields are silently ignored. This enables schema evolution in production pipelines.
- **Codec and Compression**: Avro supports optional compression codecs (null, deflate, snappy). The deflate codec uses zlib compression; snappy provides faster compression with slightly larger output. Codec is specified at write time and auto-detected at read time.

## Pitfalls

- **Mismatched Schema Versions**: Attempting to read Avro files with a reader schema that lacks required fields defined in the writer schema causes `avro.schema.SchemaResolutionException`. Always ensure reader schema includes all fields with no default in the writer schema, or those fields will be silently dropped.
- **Unicode Handling in String Fields**: Avro Python treats string fields as unicode by default, but when writing to binary Avro files, encoding can cause issues if the input contains non-ASCII characters improperly encoded. Always ensure input data is properly unicode-normalized before writing.
- **File Sync Marker Corruption**: Writing Avro files with a non-seekable stream (e.g., pipe or socket) without proper buffering can result in corrupted sync markers. Use `BufferedWriter` or `DatumWriter` with a `SeekableByteChannelWriter` when random-access isn't available.
- **Invalid Schema JSON**: Providing malformed JSON to schema parsing functions (e.g., missing quotes around field names, invalid type references) results in `avro.schema.SchemaParseException`. Use `avro.schema.parse_schema` to validate before use.

## Examples

### Read records from an Avro data file with embedded schema
**Args:** `--fromfile data.avro`
**Explanation:** When reading an Avro file that already contains the schema in its header, use the `--fromfile` flag to automatically load both data and schema. The reader infers the schema from file metadata.

### Write Avro records using a JSON schema file
**Args:** `--schema-file schema.avsc --record-output output.avro`
**Explanation:** Specify an external Avro schema file (JSON format `.avsc`) to validate and serialize records. The schema is used to encode the data properly; the schema itself is written to the output file header.

### Validate a JSON schema against the Avro specification
**Args:** --schema-parse schema.avsc
**Explanation:** Parse and validate a JSON schema file without reading or writing data. Useful for checking schema syntax before integrating into a pipeline; prints any parsing errors to stderr.

### List schema fields from an Avro file
**Args:** --schema-fromfile data.avro --print-schema
**Explanation:** Extract and display the embedded schema from an Avro data file. Prints the schema in JSON format to stdout, useful for understanding data structure in existing files.

### Compress output Avro file using deflate codec
**Args:** --record-output compressed.avro --codec deflate
**Explanation:** Write Avro records with zlib compression to reduce file size. Specifying `--codec deflate` explicitly uses the deflate codec; files written with this flag are larger than snappy but more widely compatible.

### Read Avro file with a different reader schema for schema evolution
**Args:** --schema-file reader.avsc --fromfile legacy_data.avro --record-output migrated.avro
**Explanation:** Use a newer reader schema to read data written with an older schema. Fields with defaults fill in missing values from the old schema. This pattern upgrades legacy Avro files to new schema versions in bioinformatics workflows.

### Create Avro records from JSON lines with schema validation
**Args:** --schema-file schema.avsc --fromjson records.jsonl --record-output validated.avro
**Explanation:** Convert newline-delimited JSON records to Avro format while validating against the provided schema. Records failing schema validation are skipped with an error message; successful records are written to the output file.

### Show metadata headers from an Avro file
**Args:** --fromfile data.avro --metadata
**Explanation:** Display all metadata stored in the Avro file header, including schema, codec, and sync marker. Useful for debugging file format issues or checking file provenance in data release archives.