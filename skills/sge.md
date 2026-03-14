---
name: sge
category: hpc
description: Sun Grid Engine (SGE/UGE/OGS) cluster scheduler for HPC job submission, monitoring, and resource management
tags: [sge, uge, gridengine, hpc, cluster, job-scheduler, batch, qsub, bioinformatics, computing]
author: oxo-call built-in
source_url: "https://gridscheduler.sourceforge.net/htmlman/manuals.html"
---

## Concepts

- SGE (Sun/Univa/Open Grid Scheduler) manages jobs on HPC clusters. Submit with `qsub script.sh`, monitor with `qstat`, delete with `qdel`. Resource requests use `-l resource=value` syntax.
- Job scripts use `#$` directives: `#$ -N JOBNAME`, `#$ -q QUEUE`, `#$ -pe PARALLEL_ENV N` (parallel environment), `#$ -l h_rt=HH:MM:SS` (hard runtime limit), `#$ -l h_vmem=SIZE` (memory per slot).
- Parallel environments (PE) control multi-threading: `#$ -pe smp N` for shared-memory (threads), `#$ -pe mpi N` for distributed (MPI). Check available PEs with `qconf -spl`.
- Use `qstat -u $USER` to check your jobs; `qstat -j JOBID` for details; `qstat -f` for full cluster status showing all queues and nodes. Job states: qw=queued/waiting, r=running, Eqw=error.
- Array jobs: `qsub -t 1-100 script.sh`. Each task gets `$SGE_TASK_ID`. Use `-tc N` to limit concurrent tasks. Output files use `$JOB_ID.$TASK_ID` naming.
- Environment variables: `$JOB_ID`, `$SGE_TASK_ID`, `$NSLOTS` (allocated slots), `$TMPDIR` (local scratch per job), `$SGE_O_WORKDIR` (submission directory).
- Resource queries: `qhost` lists nodes and their resources; `qconf -sql` lists queues; `qconf -sq QUEUE` shows queue configuration; `qquota` shows your resource usage quotas.

## Pitfalls

- SGE does NOT change to the submission directory by default. Add `cd $SGE_O_WORKDIR` or use `#$ -cwd` to run in the current working directory.
- Memory requests (`-l h_vmem=4G`) are per-slot in most configurations. If you request 8 slots (`-pe smp 8`) with `h_vmem=4G`, total memory is 32G. Check your site's policy.
- `h_rt` (hard runtime) vs `s_rt` (soft runtime): jobs exceeding `h_rt` are killed immediately. Set `h_rt` with margin. Always use `h_rt`, not just `s_rt`.
- Jobs in `Eqw` (error-queued-waiting) state won't run until the error is fixed. Check with `qstat -j JOBID | grep error`. Common causes: missing script, wrong permissions, invalid resource request.
- Output/error files (`-o` and `-e`) default to the home directory unless `-cwd` is set. Combine stdout/stderr with `#$ -j y` (join=yes) to simplify log management.
- The `$NSLOTS` variable gives the number of allocated slots — use this in your commands (e.g., `-t $NSLOTS` for thread count) instead of hard-coding thread numbers.
- Array task IDs start at the value you specify. `qsub -t 0-99` gives IDs 0-99; `qsub -t 1-100` gives 1-100. Ensure your sample list indexing matches.

## Examples

### submit a basic batch job
**Args:** `qsub -N align_job -q all.q -l h_rt=04:00:00 -l h_vmem=4G -pe smp 8 -cwd job.sh`
**Explanation:** submits job.sh with 8 threads, 4G per slot (32G total), 4-hour limit; -cwd runs in submission directory

### submit a sample-parallel array job
**Args:** `qsub -t 1-96 -tc 24 -N fastp_array -q all.q -l h_rt=01:00:00 -l h_vmem=2G -pe smp 4 -cwd qc_array.sh`
**Explanation:** submits 96 tasks, max 24 concurrent (-tc 24); each gets $SGE_TASK_ID; 4 threads and 2G/slot per task

### run an interactive session on a compute node
**Args:** `qrsh -q interactive.q -l h_rt=02:00:00 -l h_vmem=4G -pe smp 4`
**Explanation:** qrsh opens interactive shell on compute node; equivalent to srun --pty or qsub -I in other schedulers

### check your running and queued jobs
**Args:** `qstat -u $USER -r`
**Explanation:** -r shows requested resources; job states: qw=waiting, r=running, Eqw=error, dr=deletion

### get detailed information about a specific job
**Args:** `qstat -j 12345`
**Explanation:** shows full job details including resource requests, submission time, scheduling info, and error messages

### check available queues and their resource limits
**Args:** `qconf -sql && qconf -sq all.q`
**Explanation:** first lists all queue names, then shows detailed config for all.q; check h_rt, slots, and access lists

### check cluster node status and available resources
**Args:** `qhost -q`
**Explanation:** -q includes queue instance info per host; shows load, memory, CPU count, and which queues each node serves

### delete a specific job or all your jobs
**Args:** `qdel 12345`
**Explanation:** cancels job 12345; use 'qdel -u $USER' to cancel all your jobs

### submit a job with dependency on a previous job
**Args:** `qsub -hold_jid 12345 -N step2 -q all.q -l h_rt=02:00:00 -cwd step2.sh`
**Explanation:** step2.sh starts only after job 12345 finishes; -hold_jid accepts job IDs or job names

### check accounting information for completed jobs
**Args:** `qacct -j 12345`
**Explanation:** shows resource usage (ru_wallclock, maxvmem, cpu) for completed jobs; useful for optimizing future resource requests
