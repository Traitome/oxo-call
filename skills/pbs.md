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
**Explanation:** submits job.pbs to the batch queue with 8 CPUs and 32GB RAM; -N sets job name; walltime is 4 hours

### submit a job script with Torque-style resource specification
**Args:** `qsub -N fastp_qc -q normal -l nodes=1:ppn=4 -l mem=16gb -l walltime=02:00:00 qc.pbs`
**Explanation:** Torque syntax uses nodes=N:ppn=M for processor allocation; check your system's PBS variant

### run an interactive session on a compute node
**Args:** `qsub -I -q interactive -l select=1:ncpus=4:mem=16gb -l walltime=02:00:00`
**Explanation:** -I starts an interactive session; useful for testing commands before creating batch scripts

### submit a sample-parallel array job
**Args:** `qsub -J 1-96 -N fastp_array -q batch -l select=1:ncpus=4:mem=8gb -l walltime=01:00:00 qc_array.pbs`
**Explanation:** PBS Pro array syntax; each task gets unique $PBS_ARRAY_INDEX; use -t 1-96 for Torque

### check your running and queued jobs
**Args:** `qstat -u $USER -a`
**Explanation:** -a shows extended info including requested resources; job states: Q=queued, R=running, H=held

### get detailed information about a specific job
**Args:** `qstat -f 12345.pbs-server`
**Explanation:** -f shows full job details including requested/used resources, submission time, and execution host

### delete a job or all your jobs
**Args:** `qdel 12345.pbs-server`
**Explanation:** cancels job 12345; some systems support 'qdel all' or use qselect: 'qdel $(qselect -u $USER)'

### list available queues and their resource limits
**Args:** `qstat -Q -f`
**Explanation:** shows all queues with their configuration including max walltime, max nodes, and access restrictions

### check node availability and resources
**Args:** `pbsnodes -a -F dsv`
**Explanation:** shows all nodes with state, properties, assigned jobs; -F dsv formats output as delimiter-separated values

### submit a job with dependency on a previous job
**Args:** `qsub -W depend=afterok:12345.pbs-server -N step2 -q batch -l walltime=02:00:00 step2.pbs`
**Explanation:** step2.pbs runs only after job 12345 completes successfully; use afterany for run-regardless

### submit a GPU job
**Args:** `qsub -N gpu_job -q gpu -l select=1:ncpus=8:mem=64gb:ngpus=1 -l walltime=24:00:00 gpu_job.pbs`
**Explanation:** ngpus=1 requests 1 GPU; queue name may vary (gpu, gpuq, etc.); check with qstat -Q

### check resource usage of completed jobs
**Args:** `qstat -x -f 12345.pbs-server`
**Explanation:** -x includes finished jobs in output; shows resources_used.walltime, resources_used.mem, etc.

### place a job on hold
**Args:** `qhold 12345.pbs-server`
**Explanation:** prevents job from starting; use qrls to release hold and allow job to run

### release a held job
**Args:** `qrls 12345.pbs-server`
**Explanation:** releases job from hold state; job becomes eligible to run according to queue priority

### requeue a job for re-execution
**Args:** `qrerun 12345.pbs-server`
**Explanation:** requeues running or completed job; useful for retrying failed jobs or re-running with same parameters

### move a job to a different queue
**Args:** `qmove priority 12345.pbs-server`
**Explanation:** moves job to the 'priority' queue; useful when current queue is busy or has different limits

### select jobs matching criteria for bulk operations
**Args:** `qselect -u $USER -s Q`
**Explanation:** selects all queued jobs for user; pipe to qdel for bulk deletion: qdel $(qselect -u $USER -s Q)
