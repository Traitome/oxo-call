---
name: arvados-cli
category: Bioinformatics/Distributed Computing
description: Command-line interface for interacting with the Arvados platform - manages Keep storage collections, runs containerized workflows (Crunch), and interfaces with the Arvados API server for bioinformatics pipelines.
tags: bioinformatics, workflows, distributed-computing, containers, keep-storage, cwl, pipeline, crunch
author: AI-generated
source_url: https://doc.arvados.org/v2.12/reference/arvados-cli.html
---

## Concepts

- **Collections and Keep Storage**: All data in Arvados is stored in collections — immutable, versioned sets of files stored in Keep (the distributed storage system). Use `arv ls` and `arv get` to browse and retrieve collection contents; use `arv put` to upload new files or directories into a collection.

- **Container Execution via Crunch**: The `arvcr-runner` command (invoked through `arv workflow run` or directly) executes CWL (Common Workflow Language) workflows inside isolated containers. You must specify a Docker image, CPU/memory constraints, and the workflow definition file; the runner submits the job to the API server for scheduling on compute nodes.

- **API Server Authentication**: Every command communicates with the Arvados API server using an API token (set via `ARVADOS_API_TOKEN` environment variable or `--api-token` flag). Without valid authentication, commands return 401/403 errors; tokens are scoped to a specific user and project.

- **Project Hierarchy with UUIDs**: Collections, workflows, and containers are organized under projects (containers/projects) identified by UUIDs (e.g., `wzzzz-xxxxx-yyyyy`). Use `arv ls` with project UUIDs to navigate the hierarchy; omitting a project path defaults to the user's home project.

## Pitfalls

- **Omitting the Collection UUID from `arv get`**: Running `arv get` without specifying a collection UUID (or portable data hash) downloads nothing — the command requires a target identifier. This silently fails, leaving users unsure where their output went.

- **Mismatching API Host in Development vs Production**: If `ARVADOS_API_HOST` points to the wrong server (e.g., staging instead of production), commands succeed but operate on the wrong project/collection data. Consequences include data loss or publishing to the wrong environment.

- **Not Using `--wait` When Monitoring Workflows**: Submitting a workflow with `arv workflow run` without `--wait` returns immediately, giving no feedback on job status. Users assume the workflow failed because they never see output; the job actually runs asynchronously.

- **Confusing Collections vs Containers**: Collections store immutable data; containers store workflow runs (logs, output references). Mixing them up when using `arv ls` leads to unexpected directory listings, as they are separate API endpoints with different fields.

## Examples

### List collections in a project
**Args:** `ls --uuid wzzzz-xxxxx-yyyyy`
**Explanation:** Lists all collection objects under the specified project UUID, enabling users to find existing datasets before creating new ones or copying data.

### Upload a directory to Keep storage
**Args:** `put --collection my-project/path/to/files/`
**Explanation:** Recursively uploads a local directory into a collection; the `--collection` flag creates or appends to an existing collection, storing all files in Keep with integrity checking.

### Run a CWL workflow
**Args:** `workflow run workflow.cwl --job job.yml --wait`
**Explanation:** Submits a Common Workflow Language definition with input parameters, blocks until completion (via `--wait`), and streams logs — essential for monitoring pipeline execution.

### Download files from a collection
**Args:** `get wzzzz-xxxxx-yyyyy --output-dir ./output/`
**Explanation:** Downloads all files from a specific collection (identified by UUID or portable data hash) into a local directory, preserving the Keep-stored structure.

### Check container status
**Args:** `containers wzzzz-xxxxx-yyyyy --long`
**Explanation:** Retrieves detailed runtime information for a container (job) including exit code, runtime duration, and node assignment — used for debugging failed workflow executions.

### Create a new project for organizing workflows
**Args:** `create project --name "RNA-seq Analysis" --description "Variant calling pipelines"`
**Explanation:** Creates a new project container under the user's home project to organize related workflows, providing isolation and access control for team pipelines.

### Use a different API token for testing
**Args:** `--api-token your-test-token --host api-staging.example.com:443 ls`
**Args:** (alternative form using environment): `ARVADOS_API_TOKEN=your-test-token ARVADOS_API_HOST=api-staging.example.com:443 arv ls`
**Explanation:** Overrides the default API credentials to interact with a staging or test instance without modifying the global environment, useful for CI/CD testing.

### Get a collection's portable data hash
**Args:** `collections wzzzz-xxxxx-yyyyy --filters "uuid=wzzzz-xxxxx-yyyyy" --format=uuid`
**Explanation:** Retrieves the portable data hash (PDH) for a collection, which is a content-addressable identifier useful for sharing immutable dataset references.