---
name: pbs
category: hpc
description: PBS (Portable Batch System) and PBS Pro job scheduler for HPC cluster batch computing and resource management
tags: [pbs, torque, pbspro, hpc, cluster, job-scheduler, batch, qsub, bioinformatics, computing]
author: oxo-call built-in
source_url: "https://www.openpbs.org/documentation/"
---

## Concepts
- PBS manages jobs on HPC clusters. Submit batch jobs with `qsub script.pbs`, check status with `qstat`, and delete jobs with `qdel`. PBS variants include PBS Pro (commercial/open-source), Torque, and OpenPBS.
- Job scripts start with `#!/bin/bash` followed by `#PBS` directives: `#PBS -N JOBNAME`, `#PBS -q QUEUE`, `#PBS -l nodes=N:ppn=M` (Torque) or `#PBS -l select=N:ncpus=M:mem=SIZE` (PBS Pro), `#PBS -l walltime=HH:MM:SS`.
- Resource specification differs between Torque and PBS Pro: Torque uses `nodes=N:ppn=M`, PBS Pro uses `select=N:ncpus=M:mem=SIZE:ngpus=G`. Always check your system's syntax with `qstat -Qf` or `pbsnodes -a`.
- Use `qstat` to monitor jobs: `qstat -u $USER` shows your jobs; `qstat -f JOBID` shows full details. Job states: Q=queued, R=running, E=exiting, H=held, C=completed.
- PBS environment variables in jobs: `$PBS_JOBID`, `$PBS_JOBNAME`, `$PBS_O_WORKDIR` (submission directory), `$PBS_NODEFILE` (allocated node list), `$PBS_ARRAY_INDEX` (array task index).
- Use `pbsnodes -a` to check node status and resources. Use `qstat -Q` to list all queues and their limits. Use `qstat -Qf QUEUE` for detailed queue information.
- Array jobs in PBS: `qsub -J 1-100 script.pbs` (PBS Pro) or `qsub -t 1-100 script.pbs` (Torque). Each task gets `$PBS_ARRAY_INDEX` (PBS Pro) or `$PBS_ARRAYID` (Torque).
- Always `cd $PBS_O_WORKDIR` at the start of your job script — PBS starts jobs in the user's home directory by default, not the submission directory.
- `qhold` and `qrls` place jobs on hold and release them; useful for managing job dependencies manually.
- `qrerun` requeues a running or completed job for re-execution.
- `qmove` moves a job to a different queue.

## Pitfalls
- Always specify walltime (`#PBS -l walltime=HH:MM:SS`); jobs without it use the queue default which may be very short. Format is always `HH:MM:SS`.
- PBS starts jobs in the home directory, NOT the submission directory. Always add `cd $PBS_O_WORKDIR` at the top of your script before any relative paths.
- Resource syntax differs between Torque and PBS Pro. Using `nodes=N:ppn=M` on PBS Pro will fail; use `select=N:ncpus=M:mem=SIZE` instead.
- `#PBS -l mem=16gb` sets total memory; `#PBS -l pmem=4gb` sets per-process memory. Exceeding these causes the job to be killed with no warning in the output.
- Job output/error files (`-o` and `-e`) default to `JOBNAME.oJOBID` and `JOBNAME.eJOBID` in the home directory, not the submission directory. Use full paths or add `cd $PBS_O_WORKDIR`.
- `qsub -I` for interactive jobs still requires resource specifications. Without them, you get minimal resources. Always include `-l walltime=` and `-l select=` or `-l nodes=`.
- Module loads must come after `cd $PBS_O_WORKDIR` and before computation. The module environment may differ between login and compute nodes.
- `qhold` prevents a job from starting but does not stop a running job; use `qdel` to terminate running jobs.
- `qrerun` may fail if the job's output files are locked or if resources are no longer available.

## Examples

### submit a basic batch job script
**Args:** `qsub -N alignment -q batch -l select=1:ncpus=8:mem=32gb -l walltime=04:00:00 job.pbs`
**Explanation:** qsub command; -N alignment job name; -q batch queue name; -l select=1:ncpus=8:mem=32gb resource specification; -l walltime=04:00:00 time limit; job.pbs job script; submits to the batch queue with 8 CPUs and 32GB RAM

### submit a job script with Torque-style resource specification
**Args:** `qsub -N fastp_qc -q normal -l nodes=1:ppn=4 -l mem=16gb -l walltime=02:00:00 qc.pbs`
**Explanation:** qsub command; -N fastp_qc job name; -q normal queue name; -l nodes=1:ppn=4 Torque processor allocation; -l mem=16gb memory; -l walltime=02:00:00 time limit; qc.pbs job script; Torque syntax uses nodes=N:ppn=M for processor allocation; check your system's PBS variant

### run an interactive session on a compute node
**Args:** `qsub -I -q interactive -l select=1:ncpus=4:mem=16gb -l walltime=02:00:00`
**Explanation:** qsub command; -I starts an interactive session; -q interactive queue name; -l select=1:ncpus=4:mem=16gb resource specification; -l walltime=02:00:00 time limit; useful for testing commands before creating batch scripts

### submit a sample-parallel array job
**Args:** `qsub -J 1-96 -N fastp_array -q batch -l select=1:ncpus=4:mem=8gb -l walltime=01:00:00 qc_array.pbs`
**Explanation:** qsub command; -J 1-96 PBS Pro array job range; -N fastp_array job name; -q batch queue name; -l select=1:ncpus=4:mem=8gb resource specification; -l walltime=01:00:00 time limit; qc_array.pbs job script; each task gets unique $PBS_ARRAY_INDEX; use -t 1-96 for Torque

### check your running and queued jobs
**Args:** `qstat -u $USER -a`
**Explanation:** qstat command; -u $USER filter by user; -a shows extended info including requested resources; job states: Q=queued, R=running, H=held

### get detailed information about a specific job
**Args:** `qstat -f 12345.pbs-server`
**Explanation:** qstat command; -f 12345.pbs-server job ID; shows full job details including requested/used resources, submission time, and execution host

### delete a job or all your jobs
**Args:** `qdel 12345.pbs-server`
**Explanation:** qdel command; 12345.pbs-server job ID; cancels job 12345; some systems support 'qdel all' or use qselect: 'qdel $(qselect -u $USER)'

### list available queues and their resource limits
**Args:** `qstat -Q -f`
**Explanation:** qstat command; -Q list queues; -f full details; shows all queues with their configuration including max walltime, max nodes, and access restrictions

### check node availability and resources
**Args:** `pbsnodes -a -F dsv`
**Explanation:** pbsnodes command; -a show all nodes; -F dsv delimiter-separated values format; shows all nodes with state, properties, assigned jobs

### submit a job with dependency on a previous job
**Args:** `qsub -W depend=afterok:12345.pbs-server -N step2 -q batch -l walltime=02:00:00 step2.pbs`
**Explanation:** qsub command; -W depend=afterok:12345.pbs-server dependency on job 12345; -N step2 job name; -q batch queue name; -l walltime=02:00:00 time limit; step2.pbs job script; step2.pbs runs only after job 12345 completes successfully; use afterany for run-regardless

### submit a GPU job
**Args:** `qsub -N gpu_job -q gpu -l select=1:ncpus=8:mem=64gb:ngpus=1 -l walltime=24:00:00 gpu_job.pbs`
**Explanation:** qsub command; -N gpu_job job name; -q gpu queue name; -l select=1:ncpus=8:mem=64gb:ngpus=1 GPU resource; -l walltime=24:00:00 time limit; gpu_job.pbs job script; ngpus=1 requests 1 GPU; queue name may vary (gpu, gpuq, etc.); check with qstat -Q

### check resource usage of completed jobs
**Args:** `qstat -x -f 12345.pbs-server`
**Explanation:** qstat command; -x includes finished jobs; -f 12345.pbs-server job ID; shows resources_used.walltime, resources_used.mem, etc. for completed jobs

### place a job on hold
**Args:** `qhold 12345.pbs-server`
**Explanation:** qhold command; 12345.pbs-server job ID; prevents job from starting; use qrls to release hold and allow job to run

### release a held job
**Args:** `qrls 12345.pbs-server`
**Explanation:** qrls command; 12345.pbs-server job ID; releases job from hold state; job becomes eligible to run according to queue priority

### requeue a job for re-execution
**Args:** `qrerun 12345.pbs-server`
**Explanation:** qrerun command; 12345.pbs-server job ID; requeues running or completed job; useful for retrying failed jobs or re-running with same parameters

### move a job to a different queue
**Args:** `qmove priority 12345.pbs-server`
**Explanation:** qmove command; priority target queue; 12345.pbs-server job ID; moves job to the 'priority' queue; useful when current queue is busy or has different limits

### select jobs matching criteria for bulk operations
**Args:** `qselect -u $USER -s Q`
**Explanation:** qselect command; -u $USER filter by user; -s Q filter by queued state; selects all queued jobs for user; pipe to qdel for bulk deletion: qdel $(qselect -u $USER -s Q)
