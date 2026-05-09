---
name: clusterone
category: Job Scheduler / HPC
description: A command-line tool for managing and executing jobs on Sun Grid Engine (SGE) clusters and compatible job schedulers. Allows users to submit batch jobs, monitor queue status, manage parallel environments, and control job lifecycle operations.
tags:
  - sge
  - hpc
  - cluster
  - job-scheduler
  - grid-engine
  - torque
  - parallel-computing
  - qsub
  - qstat
author: AI-generated
source_url: https://github.com/_gridengine/gridengine
---

## Concepts

- **Job Submission Model**: ClusterOne uses `qsub`-style job scripts containing `#$` directive comments for specifying queue (-q), project (-P), parallel environment (-pe), and resource requirements. Job scripts are executed on compute nodes while output streams are written to files in the submission directory by default.
- **Queue and Parallel Environment Hierarchy**: Jobs are routed to queues (also called clusters) based on ACLs and resource availability. Parallel jobs require a specific parallel environment (PE) configured with allocation rules (fillup, round_robin, pe_slots) and minimum/maximum slot counts.
- **Status Lifecycle and Exit Codes**: Jobs transition through states: pending (qw), running (r), suspended (s), transferred (t), or completed (done). The exit code in `$HOME/.pid/$JOB_ID.exit` determines success (0) or failure (non-zero), separate from job arrays which track each element's status independently.
- **Array Jobs and Dependency Scheduling**: Array jobs (`-t 1-10`) launch replicated tasks with `$SGE_TASK_ID` set per task. Job dependencies use `-hold_jid` to enforce ordering, where dependent jobs wait until all specified jobs reach a terminal state.
- **Integration with SGE Compatible Schedulers**: ClusterOne supports TORQUE, Openlava, and Son of Grid Engine through the `-scheduller` flag, adapting resource string formats while maintaining compatible job control semantics.

## Pitfalls

- **Omitting the executable permission in job scripts**: Running `qsub script.sh` without setting execute permissions (`chmod +x script.sh`) causes the job to fail immediately with "permission denied" because the SGE execution daemon cannot invoke the interpreter.
- **Specifying incorrect parallel environment names**: Using `-pe mpi 8` when no PE named "mpi" exists results in "Unable to find PE" error; verify available PEs with `qconf -spl` before requesting slots.
- **Conflicting stdout/stderr redirection**: Using both `-o` / `-e` and redirecting inside the script (e.g., `> output.log 2>&1`) creates duplicate output files—the SGE directives handle file placement separately from script-level redirection.
- **Ignoring default working directory behavior**: Submitted jobs run in the submission directory unless `-cwd` is specified, which can cause path resolution failures for relative file paths if the job script expects to run from a different location.
- **Job timeout not enforced by default**: Without `-l h_rt=HH:MM:SS`, jobs may run indefinitely and occupy cluster resources; administrators may kill unconstrained jobs preemptively, resulting in incomplete results.

## Examples

### Run a simple single-node job with email notification
**Args:** `myjob.sh -m abe -M user@example.com`
**Explanation:** Submits the job script and sends emailnotifications when the job begins (b), ends (e), or aborts (a), directing status to the specified address.

### Request 16 slots across 2 nodes using the mpi parallel environment
**Args:** `program.sh -pe mpi 16 -l hostname='node[1-2]'`
**Explanation:** Allocates 16 slots on exactly two specified compute nodes, binding MPI ranks to specific hosts for NUMA-aware or topology-optimized execution.

### Submit a job array processing 100 input files
**Args:** `process_array.sh -t 1-100 -tc 10 -q short -N array_job`
**Explanation:** Creates 100 task variations where `$SGE_TASK_ID` maps to input file indices, limitsconcurrent tasks to 10 to prevent I/O saturation, queues to the short queue, and names the job for easier identification.

### Hold a dependent job until a previous job completes successfully
**Args:** `dependent.sh -hold_jid 12345 -N dep_job`
**Explanation:** Queues the dependent job in pending state until job ID 12345 finishes with exit code 0, enablingworkflows where downstreamanalysis depends on upstream results.

### Set a hard runtime limit and request specific memory per slot
**Args:** `mem_intensive.sh -l h_rt=02:00:00 -l mem_free=4G`
**Explanation:** Terminates the job automatically after 2 hours if it exceeds the limit, and reserves 4GB of physical memory per slot to prevent OOM kills on nodes with insufficient RAM.

### Combine error and output into a single log file
**Args:** `logs_merged.sh -o job.out -e job.out`
**Explanation:** Directs both stdout and stderr to the same file in the submission directory, preserving output order for easier debugging when redirection inside the script is unavailable.

### Submit to a specific project and specify a deadline reservation
**Args:** `-P myproject -dl 2024-12-31-23:59:00 run.sh`
**Explanation:** Charges job resources to the myproject account and ensures the job starts by the specified deadline; if insufficient resources exist, the reservation may fail.

### Query status of all jobs owned by a specific user
**Args:** `-u username`
**Explanation:** Displays queue information for all jobs belonging to the specified user, filtering the default all-user view to focus on specific user activity.

### Delete a running or pending job by ID
**Args:** `-d 67890`
**Explanation:** Removes job 67890 from the queue, freeing allocated resources immediately; the job terminates with aborted status if currently running.

### Run an interactive job on a compute node with X11 forwarding
**Args:** `-interactive -pe shell 1 -v DISPLAY run.sh`
**Explanation:** Requests an interactive shell on a compute node with X11DISPLAY variable exported, enabling graphical applications to forward to the login node's display.