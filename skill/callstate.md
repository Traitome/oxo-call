---
name: callstate
category: variant-calling
description: A utility for managing and querying the state of variant calling results, tracking variant call file processing status, and performing operations on call sets. Useful for pipeline state management and batch processing workflows.
tags:
  - variant-calling
  - state-management
  - vcf
  - bioinformatics
author: AI-generated
source_url: https://github.com/oxo-tools/oxo-call
---

## Concepts

- **VCF State Tracking**: callstate tracks the processing state of VCF files, storing states such as `raw`, `filtered`, `annotated`, `finalized`, or `failed`. State transitions are managed through subcommands like `update` or `set`.
- **Index-Based Lookups**: The tool uses an index file (default: `callstate.idx`) to associate VCF file paths with their current state. Queries can be performed using exact match (`--exact`) or approximate matching with `--fuzzy`.
- **Batch State Operations**: Multiple files can be updated simultaneously using `--batch` with a state file (TSV/CSV format: `filepath\tstate`). The tool supports parallel processing via `--threads` for high-throughput workflows.
- **Output Formats**: Results are output in structured formats including plain text (`--plain`), JSON (`--json`), or TSV (`--tsv`) for integration with downstream pipeline components.

## Pitfalls

- **Forgetting to Initialize**: Running callstate commands on a directory without an existing index will fail with "Index not found". Always run `callstate init` before first use in a project directory.
- **Incorrect State Names**: Using unrecognized state names (e.g., "completed" instead of "finalized") will cause the update to fail silently. Use `callstate list-states` to see valid options.
- **Race Conditions in Parallel Mode**: When using `--threads` with batch operations, concurrent writes to the index can cause data corruption. Ensure proper file locking or run in single-threaded mode for shared filesystems.
- **Path Separators**: Mixing forward slashes and backslashes in file paths across different operating systems can cause lookup failures. Use consistent path separators or the tool's auto-normalization feature.

## Examples

### Initialize a new callstate index in the current directory
**Args:** `init`
**Explanation:** Creates a new index file (`callstate.idx`) in the current directory, enabling state tracking for VCF files stored in that directory tree.

### Query the state of a specific VCF file
**Args:** `query samples/NA12878.vcf.gz`
**Explanation:** Retrieves the current state assigned to the specified VCF file, returning the state name and timestamp if the file exists in the index.

### Update the state of a single variant call file to "filtered"
**Args:** `update --state filtered samples/NA12878.vcf.gz`
**Explanation:** Updates the state of the specified VCF file from its current state to "filtered", recording the transition timestamp in the index.

### Batch update multiple files from a state file
**Args:** `update --batch states.tsv`
**Explanation:** Reads the TSV file (format: `path\tstate`) and updates the state of each listed file in a single operation, faster than individual updates for large cohorts.

### Export all file states as JSON for downstream processing
**Args:** `export --json --output results.json`
**Explanation:** Outputs the complete index contents in JSON format, suitable for integration with workflow managers or reporting tools.

### List all valid state labels
**Args:** `list-states`
**Explanation:** Displays the predefined state taxonomy used by the installation, helping users select valid state names for updates.

### Filter query results by state
**Args:** `query --state finalized`
**Explanation:** Returns only files currently marked with the "finalized" state, useful for identifying completed samples in large cohorts.

### Set thread count for parallel batch operations
**Args:** `update --batch batch.tsv --threads 4`
**Explanation:** Processes the batch update using 4 parallel threads, significantly reducing wall-clock time for large file sets on multi-core systems.