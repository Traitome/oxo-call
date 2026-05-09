---
name: anadama2
category: workflow_management
description: A reproducible bioinformatics workflow manager that automates pipeline creation, execution, and provenance tracking through declarative Python-based workflow definitions.
tags: [workflow, pipeline, automation, reproducibility, provenance, bioinformatics, task_scheduling]
author: AI-generated
source_url: https://bitbucket.org/jordan_r_anadama/anadama2
---

## Concepts

- **Workflow Definition**: Anadama2 uses Python decorators (`@task`, `@workflow`) to define workflows in a `.py` file, where tasks specify input files, output files, and the command to execute. Tasks are declared as functions with special decorators that enable automatic dependency resolution and parallel execution.

- **File-Based Dependency Tracking**: Tasks declare their dependencies using `name` (task name), `input_files`, `output_files`, and `vars` (variables). Anadama2 automatically determines execution order by analyzing file dependencies—tasks run only when their required input files exist from previous task outputs—without requiring explicit specification of execution order.

- **Provenance and Reproducibility**: The `anadama2 run` command creates a provenance database (SQLite by default) that records every task executed, including timestamps, full commands, environment variables, and file checksums. This enables complete reproducibility auditing and rerunning only changed tasks.

- **CLI Entry Points**: The main `anadama2` command accepts subcommands including `run` (execute a workflow), `graph` (generate workflow visualization), `init` (scaffold a new workflow), `validate` (check workflow syntax), and `list` (show available workflows).

## Pitfalls

- **Missing Input File Declarations**: If a task's `input_files` list is incomplete, Anadama2 may skip task execution thinking dependencies are satisfied when they are not. This causes stale outputs that don't reflect upstream changes, leading to incorrect results without error messages.

- **Relative Paths Without Workspace Definition**: Running Anadama2 from a directory other than the workflow's declared working directory causes file-not-found errors. Workflows define their `workdir` at the top level, and all file paths in task declarations should be relative to this directory.

- **Circular Task Dependencies**: Defining tasks where A depends on B and B depends on A (directly or indirectly) causes Anadama2 to fail with a dependency resolution error. The tool performs static analysis and cannot execute workflows with cycles.

- **Python Path Issues**: If the workflow Python file imports modules that are not installed in the environment, execution fails immediately. Ensure all required Python packages are available in the active environment before running `anadama2 run`.

- **Overwriting File Outputs Accidentally**: By default, Anadama2 removes output files before task execution if they already exist. Using the `remove_old_outputs=False` task argument preserves old outputs, but this can cause tasks to skip when upstream data changed, producing inconsistent results.

## Examples

### Initialize a new workflow project in the current directory
**Args:** `init`
**Explanation:** Creates a template workflow file (`workflow.py`) and a configuration file in the current directory, providing the basic structure with example tasks to help users start building their own pipeline.

### Run a workflow defined in a Python file
**Args:** `run workflow.py`
**Explanation:** Executes the workflow defined in `workflow.py`, automatically resolving task dependencies, running tasks in the correct order, and recording provenance to the default SQLite database.

### Run a workflow with a specific configuration file
**Args:** `run workflow.py -c config.yaml`
**Explanation:** Executes the workflow using variables and settings from `config.yaml` instead of the default configuration, allowing users to parameterize paths, tool versions, and other settings across multiple runs.

### Visualize the workflow as a graph image
**Args:** `graph workflow.png workflow.py`
**Explanation:** Generates a PNG image showing the workflow's task dependency graph, useful for understanding execution order and debugging workflow structure before running expensive analyses.

### Validate a workflow file without executing it
**Args:** `validate workflow.py`
**Explanation:** Checks the workflow Python file for syntax errors, import issues, and common mistakes in task definitions without actually running any tasks, serving as a fast debugging step.

### List all available workflow templates
**Args:** `list`
**Explanation:** Displays a list of built-in Anadama2 example workflows and any user-created templates installed in the Anadama2 templates directory, helping new users discover example patterns.

### Run a specific workflow from the template library
**Args:** `run --template bwa_mem`
**Explanation:** Executes a built-in template workflow for BWA-MEM alignment, demonstrating how Anadama2 ships with reusable patterns for common bioinformatics tools that users can customize.

### Specify a custom working directory for the workflow
**Args:** `run workflow.py --workdir /path/to/project`
**Explanation:** Runs the workflow with `/path/to/project` as the working directory, changing where input/output files are resolved and where the provenance database is stored.

### Force rerun all tasks regardless of existing outputs
**Args:** `run workflow.py -f`
**Explanation:** Forces complete rerun of the entire workflow by removing all task outputs before execution, useful when upstream data has changed and intermediate results need to be regenerated.

### Limit the number of parallel tasks
**Args:** `run workflow.py -j 4`
**Explanation:** Restricts Anadama2 to run a maximum of 4 tasks in parallel, preventing resource exhaustion on shared systems while still leveraging multicore processors for independent tasks.