---
name: arv
category: Bioinformatics Data Management
description: Arv is the command-line interface for Arvados, a platform for managing, processing, and sharing large scientific data including genomic data. It provides commands to list, upload, download, and manage Keep collections, run workflows, and query the Arvados API.
tags:
  - arvados
  - keep-storage
  - collection
  - workflow
  - crunch
  - pipeline
  - api-client
  - genomics
  - data-management
author: AI-generated
source_url: https://arvados.org/ | https://github.com/arvados/arvados
---

## Concepts

- **Keep Collections**: Arvados stores data as immutable collections in Keep, a distributed content-addressed storage system. Collections are identified by a UUID (e.g., ` xxxxx-xxxxx-xxxxxxx`) and referenced by Portable Data Hashes (PDH). The `arv collection` subcommand is the primary interface for creating, listing, querying, and managing these collections.

- **API Client and Authentication**: The `arv` CLI authenticates using an API token stored in the `ARVADOS_API_TOKEN` environment variable and connects to the host specified by `ARVADOS_API_HOST`. Most commands require read or write permission scopes on the API token depending on the operation. Token permissions directly control access to collections, workflows, and other resources.

- **Workflow Execution (Crunch Run)**: Arvados workflows are executed via the `arv workflow run` or `arv crunch run` commands, which submit a Docker-based Crunch job to the Arvados Crunch compute cluster. Input/output collections are passed as Keep references using the PDH format. The workflow must be pre-registered in the Arvados system.

- **Collection Transfer Formats**: Collections can be referenced by UUID, PDH (content hash), or name tag. The `arv keep get` command downloads collection contents to local files or streams to stdout, using the collection reference as the argument. The PDH format is portable across Arvados installations because it is based on content hash.

## Pitfalls

- **Wrong Environment Variable Names**: Using `ARVADOS_TOKEN` instead of `ARVADOS_API_TOKEN` results in "Permission denied" errors because the token is not found. The correct variable name must be used or specified explicitly with `--api-token`.

- **Confusing UUID and PDH**: Attempting to download a collection using a name tag (e.g., `--collection project1.results`) without first resolving it to a PDH causes errors because Keep storage is content-addressed and does not index by name. Use `arv collection list` with `--filters` to resolve the PDH first.

- **Missing Read Permission Scope**: Running `arv keep get` on a collection succeeds but returns empty output if the API token lacks the read permission scope, without any error message. Always verify token permissions with `arv user info` before data access operations.

- **Workflow Container Image Unavailable**: Submitting a workflow with `arv workflow run` fails if the Docker image specified in the workflow is not accessible to the Crunch dispatcher. The error message is vague ("container not found") and does not indicate that the image must be pre-loaded into the Arvados registry.

- **Timeout on Large Collections**: Using `arv keep get` on very large collections without the `--collection` streaming mode causes local disk space exhaustion. Always estimate collection size first with `arv collection list` and use piped streaming output for collections larger than available disk space.

## Examples

### List all collections in the current project
**Args:** `collection list --limit 20`
**Explanation:** Lists the 20 most recent collections in the authenticated user's current project, showing UUID, name, PDH, and modified timestamps.

### Download a collection to a local directory
**Args:** `keep get xxxxx:sample.bam --collection /output_dir/`
**Explanation:** Downloads the collection identified by portable data hash `xxxxx` (or UUID) to the local path `/output_dir/`, preserving the directory structure within the collection.

### Upload a local directory as a new collection
**Args:** `collection create --name "RNA-seq Run 42" --properties '{"sample":"SRR001"}' /data/input/`
**Explanation:** Uploads the local directory `/data/input/` as a new immutable collection with a human-readable name and custom metadata properties stored in Arvados.

### Run a registered workflow with input collections
**Args:** `workflow run xxxxx-xxxxx-workflow001 --input-collection xxxxx:reference_v38 --input-fastq xxxxx:sample_01.fastq`
**Explanation:** Submits a Crunch job for the workflow UUID using the reference collection as a parameter and the sample FASTQ as another input parameter.

### Filter collections by name tag and date range
**Args:** `collection list --filters '[["name","like","%control%"]]' --order "created_at desc" --limit 50`
**Explanation:** Returns up to 50 collections whose name contains "control", ordered by creation date from newest to oldest, using the Arvados Python filter syntax.

### Stream a large collection to stdout without writing to disk
**Args:** `keep get xxxxx:large_dataset.tar.gz --stream`
**Explanation:** Streams the collection content directly to stdout instead of writing to disk, which is required when piping to another tool or when local disk space is insufficient.

### Update collection metadata after creation
**Args:** `collection update xxxxx-xxxxx-xxxxxxx --name "Validated Run 42" --properties '{"validated":true}'`
**Explanation:** Updates the mutable metadata of an existing collection (name and properties) without modifying the immutable content in Keep.

### Query collection size and checksum before download
**Args:** `collection list --filters '[["uuid","=","xxxxx-xxxxx-xxxxxxx"]]' --include-rich`
**Explanation:** Returns collection metadata including byte size, manifest digest, and file count so you can verify the collection before downloading or estimating transfer time.