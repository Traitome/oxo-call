---
name: avro-python2
category: Data Serialization / Bioinformatics IPC
description: Python implementation of Apache Avro for serialization and RPC in bioinformatics pipelines. Handles Avro schema definitions, binary data encoding/decoding, and inter-process communication between analysis tools.
tags:
  - avro
  - serialization
  - bioinformatics
  - data-format
  - rpc
  - schema
  - python
author: AI-generated
source_url: https://pypi.org/project/avro-python2/
---

## Concepts

- **Avro Schema Definition**: Avro uses JSON-based schemas to define data structures. Schemas specify field names, types (primitive like int, string, float, or complex like record, array, map), and optional/default values. The schema must be provided when reading or writing Avro files.
- **Binary Data Encoding**: Avro encodes data in a compact binary format that includes a schema fingerprint (sync marker) for quick validation. Each record is prefixed with a field count and uses type-specific encoding (e.g., varint for integers, UTF-8 for strings), making files significantly smaller than JSON equivalents.
- **Schema Evolution & Compatibility**: Avro supports schema evolution through three compatibility types—BACKWARD (new schema can read old data), FORWARD (old schema can read new data with defaults), and FULL (bidirectional). This is critical in pipelines where tool versions change but data must remain readable.
- **Data File Structure**: Avro data files (.avro) consist of a header (magic, schema, sync marker) followed by data blocks. The sync marker enables file splitting for MapReduce processing, and each file includes a codec (snappy, deflate, null) for compression.

## Pitfalls

- **Schema Mismatch Errors**: Writing data with one schema and attempting to read with another (even if semantically identical) will fail unless the reader schema explicitly uses the writer schema or schema resolution is configured. Consequence: crashes at runtime with cryptic "schema mismatch" messages.
- **Missing Sync Markers in Streaming**: When piping Avro data through Unix pipes without proper block boundaries, the reader may hang waiting for sync markers that never arrive. Consequence: pipeline deadlocks where `avro cat` or similar tools wait indefinitely for EOF that never comes.
- **Incorrect Codec Selection for Bioinformatics**: Using "null" codec (no compression) on large BAM/VCF-derived datasets creates massive files. Conversely, using "deflate" on already-compressed data provides negligible benefit while increasing CPU usage. Consequence: wasted storage or CPU cycles with no practical gain.
- **Python 2 vs Python 3 Encoding Differences**: Text fields in Python 2 (bytes) vs Python 3 (unicode) require explicit encoding/decoding. Consequence: strings decode as lists of integers instead of text, corrupting downstream analysis.

## Examples

### Validate an Avro schema JSON file
**Args:** `--schema file://path/to/schema.avsc`
**Explanation:** Verifies that the specified Avro schema file is valid JSON and conforms to Avro schema specification without performing any data operations.

### Read an Avro data file with a specific reader schema
**Args:** `data.avro --reader-schema file://updated_schema.avsc`
**Explanation:** Reads an Avro data file using the provided reader schema, enabling schema evolution where the file was written with an older schema but a new schema is available.

### Extract records from an Avro file as JSON
**Args:** `data.avro --to-json`
**Explanation:** Converts binary Avro records to JSON format on standard output, useful for debugging or piping into other JSON-processing tools in analysis pipelines.

### Count records in an Avro file without fully decoding
**Args:** `data.avro --head 0`
**Explanation:** Returns only the file header metadata (schema, codec, sync marker) without processing data blocks, useful for quick schema inspection.

### Convert CSV data to Avro using a schema
**Args:** `--from-json --schema file://schema.avsc input.json`
**Explanation:** Creates an Avro data file from JSON input records, applying the schema validation and binary encoding, useful for converting tabular bioinformatics exports to Avro format.

### List schema fields from an Avro file
**Args:** `data.avro --schema-def`
**Explanation:** Extracts and displays the Avro schema definition embedded in the file header, useful for understanding the data structure without loading any records.

### Use deflate compression for large datasets
**Args:** `data.avro --codec deflate output.avro`
**Explanation:** Writes Avro data with deflate compression enabled, reducing file size for large variant call or alignment datasets at the cost of some CPU during read/write operations.