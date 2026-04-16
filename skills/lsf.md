---
name: lsf
category: hpc
description: IBM Spectrum LSF (Load Sharing Facility) workload manager for HPC cluster job scheduling and resource management
tags: [lsf, hpc, cluster, job-scheduler, batch, bsub, bjobs, bioinformatics, computing]
author: oxo-call built-in
source_url: "https://www.ibm.com/docs/en/spectrum-lsf"
---

## Concepts
- LSF manages jobs on HPC clusters. Submit batch jobs with `bsub < script.lsf` or `bsub -I command` (interactive). Check status with `bjobs`, delete with `bkill`. LSF uses queues and host groups for resource management.
- Job scripts use `#BSUB` directives: `#BSUB -J JOBNAME`, `#BSUB -q QUEUE`, `#BSUB -n NCPUS`, `#BSUB -R "rusage[mem=SIZE]"` (memory per core), `#BSUB -W HH:MM` or `#BSUB -W DD:HH:MM` (wall time).
- Resource requirements use `-R` with `rusage[]` for consumable resources: `-R "rusage[mem=4000]"` requests 4GB per core, `-R "rusage[ngpus_physical=1]"` requests GPU. Memory unit is MB by default.
- Use `bjobs -u $USER` to check your jobs; `bjobs -l JOBID` for detailed info. Job states: PEND=pending, RUN=running, DONE=completed, EXIT=failed, SUSP=suspended.
- Array jobs: `bsub -J "array[1-100]" < script.lsf`. Each task gets `$LSB_JOBINDEX`. Limit concurrent tasks: `bsub -J "array[1-100]%20"` (max 20 at a time).
- Environment variables: `$LSB_JOBID`, `$LSB_JOBINDEX` (array index), `$LSB_DJOB_NUMPROC` (allocated processors), `$LSB_SUBCWD` (submission directory), `$LSB_HOSTS` (allocated hosts).
- Resource queries: `bqueues` lists all queues; `bqueues -l QUEUE` shows queue details; `bhosts` shows host status; `lsload` shows current load; `busers -w` shows your job limits.
- Job dependencies: `-w "done(JOBID)"` waits for completion; conditions: done(), ended(), exit(), started(), done(job_array[*]).
- Rerunnable jobs: `-r` makes jobs restartable after node failure; `-rn` disables rerun for that job.
- Job modification: `bmod` changes pending job parameters; cannot modify running jobs.
- Output control: `-o` stdout file, `-e` stderr file, `-oo` combined output; `%J` for job ID, `%I` for array index.
- Begin/end times: `-b HH:MM` delays job start; `-t HH:MM` sets termination time.
- `bpeek` views stdout/stderr of running jobs; `bhist` shows historical resource usage.

## Pitfalls
- LSF runs jobs in the submission directory by default (unlike PBS/SGE). If you need a different directory, use `#BSUB -cwd /path/to/dir`.
- `-R "rusage[mem=SIZE]"` requests memory per core. With `-n 8` and `mem=4000`, total memory is 32GB. Use `-R "rusage[mem=4000] span[hosts=1]"` to ensure single-node allocation.
- `bsub < script.lsf` reads the script from stdin; `bsub script.lsf` tries to run `script.lsf` as a command. Always use input redirection `<` for script submission.
- Wall time format varies: `-W 4:00` means 4 hours, `-W 240` means 240 minutes. Be explicit with `HH:MM` format to avoid confusion.
- Job output defaults to `JOBNAME.oJOBID` and `JOBNAME.eJOBID`. Use `#BSUB -o output.%J.log` and `#BSUB -e error.%J.log` with `%J` for job ID substitution.
- `bmod` can modify pending jobs but not running jobs. Once a job starts running, you cannot change its resource requests.
- LSF uses `span[hosts=1]` to ensure a job runs on a single node — important for multi-threaded (non-MPI) bioinformatics tools.
- CRITICAL: `-W` wall time kills jobs exceeding the limit. Set generously or use `-We` for estimated time (soft limit).
- `-r` (rerunnable) jobs restart from scratch after node failure; ensure your script handles restarts idempotently.
- `-b` (begin time) delays job start but doesn't guarantee resources; job may still wait in queue after begin time.
- Array job dependencies: use `done(job_array[*])` for all tasks or `done(job_array[1])` for specific index.
- `bkill -s SIGTERM` sends graceful termination signal; `bkill` alone sends SIGKILL immediately.
- `bpeek -f` follows output like `tail -f`; useful for monitoring long-running jobs in real-time.

## Examples

### submit a basic batch job script
**Args:** `bsub -J align_job -q normal -n 8 -R "rusage[mem=4000] span[hosts=1]" -W 4:00 -o align_%J.out < job.lsf`
**Explanation:** submits job.lsf with 8 CPUs, 4GB/core (32GB total), 4-hour limit on a single node; %J is job ID

### run an interactive session on a compute node
**Args:** `bsub -Is -q interactive -n 4 -R "rusage[mem=4000] span[hosts=1]" -W 2:00 bash`
**Explanation:** -Is opens interactive shell with pseudo-terminal; useful for testing before scripting

### submit a sample-parallel array job
**Args:** `bsub -J "fastp_qc[1-96]%24" -q normal -n 4 -R "rusage[mem=2000]" -W 1:00 -o logs/fastp_%J_%I.out < qc_array.lsf`
**Explanation:** submits 96 tasks, max 24 concurrent; %I is array index in output filename; each gets $LSB_JOBINDEX

### check your running and pending jobs
**Args:** `bjobs -u $USER -w`
**Explanation:** -w wide format shows full job name and queue; states: PEND=pending, RUN=running, DONE/EXIT=finished

### get detailed information about a specific job
**Args:** `bjobs -l 12345`
**Explanation:** shows full details including resource requests, execution host, pending reasons, and resource usage

### check available queues and their limits
**Args:** `bqueues -l normal`
**Explanation:** shows queue limits (RUNLIMIT, MEMLIMIT, PROCLIMIT), access control, and scheduling parameters

### check host status and available resources
**Args:** `bhosts -w`
**Explanation:** shows each host's status, max slots, running jobs, and available resources; -w for wide output

### delete a specific job or all your jobs
**Args:** `bkill 12345`
**Explanation:** kills job 12345; use 'bkill 0' to kill all your jobs or 'bkill -J JOBNAME' by job name

### submit a job with dependency on a previous job
**Args:** `bsub -w "done(12345)" -J step2 -q normal -n 4 -W 2:00 < step2.lsf`
**Explanation:** step2 runs after job 12345 completes; -w accepts conditions: done(), ended(), exit(), started()

### check historical resource usage of completed jobs
**Args:** `bhist -l 12345`
**Explanation:** shows detailed resource usage history including CPU time, memory, and runtime for completed jobs

### modify a pending job's queue
**Args:** `bmod -q long 12345`
**Explanation:** moves pending job 12345 to 'long' queue; only works for pending jobs, not running ones

### submit a rerunnable job for fault tolerance
**Args:** `bsub -J backup_job -q normal -n 4 -R "rusage[mem=4000]" -W 24:00 -r -o backup_%J.out < backup.lsf`
**Explanation:** -r makes job rerunnable after node failure; job restarts from scratch; ensure script handles restarts

### delay job start until specific time
**Args:** `bsub -J nightly_job -q normal -n 8 -W 8:00 -b 22:00 -o nightly_%J.out < process.lsf`
**Explanation:** -b 22:00 delays start until 10 PM; job enters queue immediately but waits until begin time

### view output of a running job
**Args:** `bpeek 12345`
**Explanation:** displays stdout/stderr of running job 12345; useful for checking progress without waiting for completion

### follow output of a running job in real-time
**Args:** `bpeek -f 12345`
**Explanation:** -f follows output like tail -f; continuously shows new output until job completes

### submit job with combined stdout/stderr
**Args:** `bsub -J combined_job -q normal -n 4 -oo combined_%J.log < job.lsf`
**Explanation:** -oo combines stdout and stderr into single file; useful for simpler log management

### kill all jobs in a job array
**Args:** `bkill -J "array_job[*]"`
**Explanation:** kills all tasks in job array; use specific index like [1] for single task, [*] for all tasks

### submit job with estimated wall time (soft limit)
**Args:** `bsub -J soft_limit -q normal -n 4 -We 4:00 -W 8:00 < job.lsf`
**Explanation:** -We 4:00 is estimated time (soft limit), -W 8:00 is hard limit; scheduler uses estimates for optimization

### check load on all hosts
**Args:** `lsload`
**Explanation:** shows current load, CPU usage, memory usage per host; helps identify busy vs idle nodes
