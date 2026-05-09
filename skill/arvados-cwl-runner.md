---
name: "arvados-cwl-runner"
category: "Workflow Execution"
description: "A Common Workflow Language (CWL) runner that executes bioinformatics workflows on Arvados clusters, submitting jobs to the Crunch dispatcher and managing data through Keep distributed storage."
tags: ["cwl", "workflow", "arvados", "bioinformatics", "distributed-computing", "job-submission"]
author: "AI-generated"
source_url: "https://doc.arvados.org/v2.13/user/cwl/cwl-runner.html"
---

## Concepts

- **CWL Workflow Execution**: `arvados-cwl-runner` parses CWL workflow definitions (`.cwl` or `.yaml` files) and converts them into Arvados container requests, executing each step as a separate job in the Crunch cluster.

- **Keep Storage Integration**: Input and output files are referenced using Arvados Keep persistent URLs (e.g., `keep:abc123/filename`), enabling distributed storage access without explicit file downloading, and all output collections are automatically stored in Keep.

- **API Authentication**: All operations require an API token (`--token`) and API server URL (`--api-server`) to authenticate with the Arvados API and access project metadata, workflow history, and container status.

- **Project-Based Organization**: Workflow outputs are stored in a specific Arvados project identified by its UUID (`--project-uuid`), which serves as the container for organizing related runs and their generated data collections.

- **Job Submission Modes**: The tool supports `--submit` to queue jobs asynchronously and `--wait` to execute synchronously, where `--wait` polls until all containers complete and reports their final status.

## Pitfalls

- **Missing API Credentials**: Running without both `--api-server` and `--token` results in authentication failures, causing the tool to abort before submitting any containers.

- **Invalid Project UUID**: Providing an incorrect or non-existent `--project-uuid` causes the run to fail during the output creation phase after all computation completes, wasting compute resources.

- **Forgetting --wait Flag**: Submitting with only `--submit` returns immediately after queuing jobs, making it appear the workflow completed successfully when containers are still running.

- **Incorrect CWL Syntax**: CWL workflows with syntax errors fail during parsing before reaching the Arvados system, producing unhelpful CWL validation errors that mask the actual syntax problem.

- **Keep URL Typoes**: Referencing input files with incorrect Keep URLs (e.g., missing the collection hash prefix) causes container initialization to fail with file-not-found errors.

## Examples

### Submit a Simple CWL Workflow for Async Execution

**Args:** `--api-server https://pipelines.arvadosapi.com /path/to/workflow.cwl --project-uuid xyzzy-todoz-1234567890 --submit`

**Explanation:** This submits a CWL workflow to the specified Arvados API server without waiting for completion, storing outputs in the designated project; useful for long-running analysis pipelines that will complete later.

### Run a CWL Workflow and Wait for Completion

**Args:** `--api-server https://pipelines.arvadosapi.com --token abcd1234efgh5678ijkl9012 --project-uuid xyzzy-todoz-1234567890 --wait /path/to/workflow.cwl`

**Explanation:** This submits the workflow and blocks until all steps finish, then reports the final status; suitable for automated pipelines where downstream steps depend on the workflow outputs.

### Run a CWL Workflow with Inline Workflow Arguments

**Args:** `--api-server https://pipelines.arvadosapi.com /path/to/workflow.cwl --project-uuid xyzzy-todoz-1234567890 --submit /path/to/input.yaml`

**Explanation:** This runs the workflow with a separate input YAML file containing parameter values, allowing reuse of the same workflow definition across different datasets without modification.

### Execute a CWL Tool (Not a Full Workflow) Against Keep Files

**Args:** `--api-server https://pipelines.arvadosapi.com /path/to/tool.cwl --project-uuid xyzzy-todoz-1234567890 --wait --output-name myresults`

**Explanation:** This runs a single CWL command-line tool rather than a multi-step workflow, storing the output collection with a descriptive name for easier identification.

### Run with Verbose Output for Debugging

**Args:** `--api-server https://pipelines.arvadosapi.com --token abcd1234efgh5678ijkl9012 --project-uuid xyzzy-todoz-1234567890 --wait --verbose /path/to/workflow.cwl /path/to/inputs.yaml`

**Explanation:** This enables verbose logging to show container request details and intermediate status information, essential for diagnosing submission failures or unexpected behavior.