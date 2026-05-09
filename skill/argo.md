---
name: Argo CLI
category: Workflow Orchestration / Kubernetes
description: A command-line interface for Argo Workflows, a container-native workflow engine for orchestrating parallel jobs on Kubernetes. Manages workflow creation, execution, monitoring, and cleanup through Kubernetes CRDs.
tags:
  - kubernetes
  - workflow
  - container-orchestration
  - dag
  - ci/cd
  - cloud-native
  - containerization
author: AI-generated
source_url: https://argo-workflows.readthedocs.io/
---

## Concepts

- **Workflow Definition Format**: Argo workflows are defined in YAML using Kubernetes-like syntax with `apiVersion: argoproj.io/v1alpha1`, `kind: Workflow`, and a `spec` containing `templates` (steps, DAG, or script). Each template defines a container, script, or resource operation.

- **Template Types**: Three primary template types exist — `container` (runs a container image), `script` (runs an embedded script inside a container), and `resource` (creates/manages Kubernetes resources). Templates can be nested and reused across workflows.

- **Artifact Management**: Workflows support input/output artifacts stored in external storage backends (S3, GCS, Azure Blob, Artifactory). Artifacts are referenced using `artifact` fields inside template `inputs`/`outputs`, enabling data passing between workflow steps.

- **DAG and Steps Syntax**: Parallel execution is defined using either `dag` (Directed Acyclic Graph) with explicit `dependencies` for each task, or `steps` with `- -` (two dashes) indicating sequential execution and `-|` indicating parallel execution of sibling steps.

- **Workflow Execution Context**: The CLI interacts with a Kubernetes cluster via kubeconfig. All workflow submissions, queries, and deletions operate on workflows defined as Custom Resource Definitions (CRDs) in the `argo` namespace.

## Pitfalls

- **Namespace Misconfiguration**: Forgetting to specify `--namespace` or `-n` causes the CLI to default to the `default` namespace, which may lack workflow permissions or the proper CRD installation, resulting in "workflows.argoproj.io not found" errors.

- **Cron Workflow Syntax Errors**: Defining a cron workflow without the `spec.schedule` field or with invalid cron syntax (not using standard cron format) causes the cron workflow to be rejected at submission, with no error message about the schedule field.

- **Artifact Path Misreferences**: Specifying incorrect artifact paths (e.g., `path: /outputs/data.txt` instead of `path: data.txt`) leads to empty artifacts being saved, because the artifact path is interpreted relative to the container's working directory, causing downstream steps to fail with empty data.

- **Template Name Case Sensitivity**: Template and workflow names are case-sensitive in Kubernetes. Using `Template: "My-Task"` in the `spec` but defining `name: my-task` in the template body causes a "template not found" error because the names must match exactly.

- **DAG Dependency Cycles**: Defining circular dependencies in a DAG (Task A depends on B, B depends on A) causes the workflow to be stuck in a `Pending` state indefinitely, consuming cluster resources with no error logged until manual intervention.

## Examples

### Submit a simple workflow from a YAML file

**Args:** `submit hello-world.yaml`

**Explanation:** Submits a workflow definition from the specified YAML file to the default namespace, creating a workflow resource that the Argo controller picks up and executes.

### List all workflows in a specific namespace

**Args:** `list -n argo`

**Explanation:** Lists all workflow resources currently in the `argo` namespace, showing their names, status, duration, and creation time for monitoring.

### Watch the real-time status of a running workflow

**Args:** `watch @latest`

**Explanation:** Monitors the most recently submitted workflow in the current namespace, displaying a live-updating table of step statuses until the workflow completes or fails.

### Retrieve logs from a specific step in a workflow

**Args:** `logs my-workflow-xyz12 --step my-step`

**Explanation:** Fetches the log output from the container(s) that executed the template named `my-step` within workflow `my-workflow-xyz12`, useful for debugging individual tasks.

### Delete a completed workflow to free cluster resources

**Args:** `delete my-workflow-xyz12`

**Explanation:** Removes the specified workflow resource from Kubernetes, cleaning up its pods and associated resources to prevent accumulation and cluster resource exhaustion.

### Submit a workflow with custom parameters

**Args:** `submit workflow.yaml -p image=nginx:latest -p replicas=3`

**Explanation:** Submits a parameterized workflow, passing `image=nginx:latest` and `replicas=3` as parameter values that the workflow templates reference using `{{inputs.parameters.image}}` syntax.

### Resubmit a failed workflow with the same arguments

**Args:** `resubmit my-workflow-failed123`

**Explanation:** Re-submits a copy of the specified failed workflow using the same parameter values and template, useful for retrying transient failures without redefining the workflow.

### Get detailed information about a workflow including node status

**Args:** `get my-workflow-xyz12 -o yaml`

**Explanation:** Retrieves the complete workflow resource in YAML format, including metadata, spec, and node status, enabling inspection of the full execution DAG and individual node states.

### Suspend a running workflow temporarily

**Args:** `suspend my-workflow-xyz12`

**Explanation:** Pauses the execution of a running workflow by setting its phase to `Suspended`, allowing administrators to stop the workflow without deleting it for later resumption.

### Resume a suspended workflow

**Args:** `resume my-workflow-xyz12`

**Explanation:** Resumes execution of a previously suspended workflow, continuing from the point where it was paused, useful for troubleshooting without losing progress.