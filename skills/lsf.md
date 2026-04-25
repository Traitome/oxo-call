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
- `-W` wall time kills jobs exceeding the limit. Set generously or use `-We` for estimated time (soft limit).
- `-r` (rerunnable) jobs restart from scratch after node failure; ensure your script handles restarts idempotently.
- `-b` (begin time) delays job start but doesn't guarantee resources; job may still wait in queue after begin time.
- Array job dependencies: use `done(job_array[*])` for all tasks or `done(job_array[1])` for specific index.
- `bkill -s SIGTERM` sends graceful termination signal; `bkill` alone sends SIGKILL immediately.
- `bpeek -f` follows output like `tail -f`; useful for monitoring long-running jobs in real-time.

## Examples

### submit a basic batch job script
**Args:** `bsub -J align_job -q normal -n 8 -R "rusage[mem=4000] span[hosts=1]" -W 4:00 -o align_%J.out < job.lsf`
**Explanation:** bsub command; -J align_job job name; -q normal queue; -n 8 CPUs; -R "rusage[mem=4000] span[hosts=1]" memory and single node; -W 4:00 wall time; -o align_%J.out output; < job.lsf script input

### run an interactive session on a compute node
**Args:** `bsub -Is -q interactive -n 4 -R "rusage[mem=4000] span[hosts=1]" -W 2:00 bash`
**Explanation:** bsub command; -Is interactive shell; -q interactive queue; -n 4 CPUs; -R "rusage[mem=4000] span[hosts=1]" memory; -W 2:00 wall time; bash shell

### submit a sample-parallel array job
**Args:** `bsub -J "fastp_qc[1-96]%24" -q normal -n 4 -R "rusage[mem=2000]" -W 1:00 -o logs/fastp_%J_%I.out < qc_array.lsf`
**Explanation:** bsub command; -J "fastp_qc[1-96]%24" array job with 96 tasks max 24 concurrent; -q normal queue; -n 4 CPUs; -R "rusage[mem=2000]" memory; -W 1:00 wall time; -o logs/fastp_%J_%I.out output; < qc_array.lsf script

### check your running and pending jobs
**Args:** `bjobs -u $USER -w`
**Explanation:** bjobs command; -u $USER filter by user; -w wide format

### get detailed information about a specific job
**Args:** `bjobs -l 12345`
**Explanation:** bjobs command; -l detailed view; 12345 job ID

### check available queues and their limits
**Args:** `bqueues -l normal`
**Explanation:** bqueues command; -l detailed view; normal queue name

### check host status and available resources
**Args:** `bhosts -w`
**Explanation:** bhosts command; -w wide format

### delete a specific job or all your jobs
**Args:** `bkill 12345`
**Explanation:** bkill command; 12345 job ID to kill

### submit a job with dependency on a previous job
**Args:** `bsub -w "done(12345)" -J step2 -q normal -n 4 -W 2:00 < step2.lsf`
**Explanation:** bsub command; -w "done(12345)" wait for job completion; -J step2 job name; -q normal queue; -n 4 CPUs; -W 2:00 wall time; < step2.lsf script input

### check historical resource usage of completed jobs
**Args:** `bhist -l 12345`
**Explanation:** bhist command; -l detailed view; 12345 job ID

### modify a pending job's queue
**Args:** `bmod -q long 12345`
**Explanation:** bmod command; -q long target queue; 12345 pending job ID

### submit a rerunnable job for fault tolerance
**Args:** `bsub -J backup_job -q normal -n 4 -R "rusage[mem=4000]" -W 24:00 -r -o backup_%J.out < backup.lsf`
**Explanation:** bsub command; -J backup_job job name; -q normal queue; -n 4 CPUs; -R "rusage[mem=4000]" memory; -W 24:00 wall time; -r rerunnable; -o backup_%J.out output; < backup.lsf script

### delay job start until specific time
**Args:** `bsub -J nightly_job -q normal -n 8 -W 8:00 -b 22:00 -o nightly_%J.out < process.lsf`
**Explanation:** bsub command; -J nightly_job job name; -q normal queue; -n 8 CPUs; -W 8:00 wall time; -b 22:00 begin time delay; -o nightly_%J.out output; < process.lsf script

### view output of a running job
**Args:** `bpeek 12345`
**Explanation:** bpeek command; 12345 job ID

### follow output of a running job in real-time
**Args:** `bpeek -f 12345`
**Explanation:** bpeek command; -f follow mode; 12345 job ID

### submit job with combined stdout/stderr
**Args:** `bsub -J combined_job -q normal -n 4 -oo combined_%J.log < job.lsf`
**Explanation:** bsub command; -J combined_job job name; -q normal queue; -n 4 CPUs; -oo combined_%J.log combined output; < job.lsf script

### kill all jobs in a job array
**Args:** `bkill -J "array_job[*]"`
**Explanation:** bkill command; -J "array_job[*]" kill all tasks in array

### submit job with estimated wall time (soft limit)
**Args:** `bsub -J soft_limit -q normal -n 4 -We 4:00 -W 8:00 < job.lsf`
**Explanation:** bsub command; -J soft_limit job name; -q normal queue; -n 4 CPUs; -We 4:00 estimated time; -W 8:00 hard limit; < job.lsf script

### check load on all hosts
**Args:** `lsload`
**Explanation:** lsload command; shows host load information
