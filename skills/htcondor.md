---
name: htcondor
category: hpc
description: HTCondor high-throughput computing system for managing and scheduling large numbers of batch jobs across distributed resources
tags: [htcondor, condor, hpc, cluster, job-scheduler, batch, high-throughput, computing, bioinformatics]
author: oxo-call built-in
source_url: "https://htcondor.readthedocs.io/"
---

## Concepts

- HTCondor specializes in high-throughput computing (HTC). Submit jobs with `condor_submit job.sub`, check status with `condor_q`, remove with `condor_rm`. Jobs are defined in submit description files (.sub).
- Submit files use `key = value` syntax: `executable = script.sh`, `arguments = arg1 arg2`, `request_cpus = N`, `request_memory = SIZE`, `request_disk = SIZE`, `queue N` (submit N copies).
- File transfer: HTCondor copies input files to compute nodes and output files back. Use `transfer_input_files = file1,file2`, `transfer_output_files = out1,out2`. Set `should_transfer_files = YES` and `when_to_transfer_output = ON_EXIT`.
- Use `condor_q -analyze JOBID` to understand why a job is idle (unmatched resources, requirements not met). `condor_q -better-analyze` gives more details about resource availability.
- Universe types: `vanilla` (most jobs), `docker` (containerized), `java` (JVM), `local` (run on submit node). Use `universe = docker` with `docker_image = IMAGE` for containerized bioinformatics.
- Environment variables: `$(Process)` (0-indexed task ID within a queue), `$(Cluster)` (job cluster ID), `$(Item)` (from queue each/matching). Use `$(Process)` to index into sample lists.
- Resource queries: `condor_status` shows available slots; `condor_status -avail` for available only; `condor_status -compact` for summary; `condor_config_val MAX_MEMORY` for config values.

## Pitfalls

- HTCondor does NOT use a shared filesystem by default; files must be explicitly transferred. If your cluster has a shared filesystem, set `should_transfer_files = NO` to avoid unnecessary copies.
- `queue N` submits N identical jobs (Process=0 to N-1); to submit jobs with different arguments, use `queue arguments from file.txt` or `queue matching files pattern`.
- Memory requests use MB by default: `request_memory = 4096` means 4GB. Exceeding memory causes eviction, not OOM kill. Check with `condor_q -l JOBID | grep MemoryUsage`.
- Log files (`log = file.log`) are mandatory for debugging. The log shows submission, execution start, eviction, and completion events. Without it, diagnosing failures is very difficult.
- `condor_rm` removes jobs from the queue; `condor_hold` pauses jobs (can resume with `condor_release`). Use hold for temporary suspension rather than removing and resubmitting.
- HTCondor may evict long-running jobs if higher-priority jobs arrive (preemption). Use `+LongRunningJob = True` or adjust priority with `condor_prio` if your site supports it.
- Output/error redirection: `output = out.$(Cluster).$(Process)` and `error = err.$(Cluster).$(Process)` — always include `$(Process)` for multi-job submissions to avoid overwriting.

## Examples

### submit a basic job from a submit file
**Args:** `condor_submit job.sub`
**Explanation:** reads the submit description file and queues jobs; returns cluster ID for tracking

### check your queued and running jobs
**Args:** `condor_q -nobatch`
**Explanation:** -nobatch shows individual jobs instead of grouped summary; shows JobID, Owner, Status, Runtime

### get detailed analysis of why a job is idle
**Args:** `condor_q -better-analyze 12345.0`
**Explanation:** explains what resources are missing for the job to run; helps identify misconfigured requirements

### remove a specific job or all your jobs
**Args:** `condor_rm 12345`
**Explanation:** removes all processes in cluster 12345; use 'condor_rm 12345.0' for a specific process

### check available compute resources in the pool
**Args:** `condor_status -compact`
**Explanation:** shows summary of total, claimed, and unclaimed slots; useful for estimating wait times

### view detailed job information including resource usage
**Args:** `condor_q -l 12345.0`
**Explanation:** -l shows all classad attributes including memory, CPU usage, and job requirements

### submit a parametric sweep (array-like) job
**Args:** `condor_submit -append "queue 100" job.sub`
**Explanation:** submits 100 copies; each gets unique $(Process) from 0-99; use to index into sample lists

### hold and release a job
**Args:** `condor_hold 12345.0`
**Explanation:** pauses the job; resume with 'condor_release 12345.0'; useful for adjusting job requirements

### check historical completed job information
**Args:** `condor_history -limit 20 -af ClusterId ProcId RemoteWallClockTime MemoryUsage ExitCode`
**Explanation:** shows last 20 completed jobs with selected attributes; -af selects specific fields for clean output

### submit a Docker-containerized bioinformatics job
**Args:** `condor_submit docker_job.sub`
**Explanation:** requires universe=docker and docker_image=IMAGE in submit file; runs containerized tools without local installation
