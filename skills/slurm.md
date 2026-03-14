---
name: slurm
category: hpc
description: Slurm workload manager for HPC cluster job scheduling, resource management, and batch computing
tags: [slurm, hpc, cluster, job-scheduler, batch, sbatch, srun, squeue, bioinformatics, computing]
author: oxo-call built-in
source_url: "https://slurm.schedmd.com/documentation.html"
---

## Concepts

- Slurm manages jobs on HPC clusters. Submit batch jobs with `sbatch script.sh`, interactive jobs with `srun`, and allocate resources with `salloc`. Always specify partition (-p), time limit (-t), and resource requirements.
- Job scripts start with `#!/bin/bash` followed by `#SBATCH` directives: `#SBATCH --job-name=NAME`, `#SBATCH --partition=PARTITION`, `#SBATCH --nodes=N`, `#SBATCH --ntasks=N`, `#SBATCH --cpus-per-task=N`, `#SBATCH --mem=SIZE`, `#SBATCH --time=HH:MM:SS`.
- Use `sinfo` to check available partitions, node states, and resources. Use `sinfo -N -l` for per-node details or `sinfo -p PARTITION` for a specific partition.
- Use `squeue` to monitor jobs: `squeue -u $USER` shows your jobs; `squeue -j JOBID` shows a specific job. Job states: PD=pending, R=running, CG=completing, CD=completed.
- Use `sacct` to check completed job accounting: `sacct -j JOBID --format=JobID,JobName,MaxRSS,Elapsed,State` shows resource usage. Add `--starttime` to filter by date.
- For bioinformatics pipelines, use array jobs (`--array=1-100`) to process multiple samples in parallel. Each array task gets a unique `$SLURM_ARRAY_TASK_ID`.
- Resource query: `scontrol show partition` lists all partitions with limits; `sacctmgr show qos` shows available QoS settings; `sshare -u $USER` shows fairshare allocation.
- Environment variables set by Slurm in jobs: `$SLURM_JOB_ID`, `$SLURM_ARRAY_TASK_ID`, `$SLURM_NTASKS`, `$SLURM_CPUS_PER_TASK`, `$SLURM_MEM_PER_NODE`, `$SLURM_SUBMIT_DIR`.

## Pitfalls

- Always specify `--time` (wall clock limit); jobs without a time limit use the partition default which may be very short. Use `HH:MM:SS` or `D-HH:MM:SS` format.
- `--mem` is per-node by default; use `--mem-per-cpu` for per-CPU memory. Requesting too little causes OOM kills; too much wastes allocation and delays scheduling.
- `--ntasks` vs `--cpus-per-task`: use `--ntasks=N` for MPI (N separate processes), `--cpus-per-task=N` for multi-threaded programs (N threads in one process). Mixing them up wastes resources.
- Output files default to `slurm-%j.out` in the submission directory. Use `--output` and `--error` to control locations. For array jobs use `%A` (array master ID) and `%a` (task ID) in filenames.
- `srun` inside an `sbatch` script inherits the allocation; don't re-specify resources. But standalone `srun` on the login node creates a new allocation — never run computation directly on login nodes.
- `module load` commands in job scripts must come before the actual computation. Ensure the module environment is clean with `module purge` first if needed.
- Job dependencies (`--dependency=afterok:JOBID`) only work with valid job IDs. Chain pipeline steps: submit step1, capture its JOBID, then submit step2 with `--dependency=afterok:$JOBID1`.

## Examples

### submit a basic batch job script
**Args:** `sbatch --job-name=align --partition=compute --time=04:00:00 --cpus-per-task=8 --mem=32G job.sh`
**Explanation:** submits job.sh to the compute partition with 8 CPUs, 32GB RAM, and 4-hour time limit

### run an interactive session on a compute node
**Args:** `srun --partition=interactive --time=02:00:00 --cpus-per-task=4 --mem=16G --pty bash`
**Explanation:** --pty bash opens an interactive shell; useful for testing commands before scripting them

### submit a sample-parallel array job for bioinformatics
**Args:** `sbatch --array=1-96 --job-name=fastp_qc --partition=compute --cpus-per-task=4 --mem=8G --time=01:00:00 --output=logs/fastp_%A_%a.out qc_array.sh`
**Explanation:** submits 96 parallel tasks; each gets unique $SLURM_ARRAY_TASK_ID; %A is master job ID, %a is array index

### check your running and pending jobs
**Args:** `squeue -u $USER --format="%.10i %.20j %.8T %.10M %.6D %.4C %.10m %R"`
**Explanation:** custom format shows JobID, Name, State, Time, Nodes, CPUs, Memory, and Reason (why pending)

### cancel a specific job or all your jobs
**Args:** `scancel 12345`
**Explanation:** cancels job 12345; use 'scancel -u $USER' to cancel all your jobs or 'scancel --state=PENDING -u $USER' for pending only

### check resource usage of a completed job
**Args:** `sacct -j 12345 --format=JobID,JobName,Partition,AllocCPUS,MaxRSS,MaxVMSize,Elapsed,State,ExitCode`
**Explanation:** shows actual CPU, memory, and time usage for completed job; MaxRSS shows peak resident memory

### query available partitions and their limits
**Args:** `sinfo --format="%20P %5a %10l %6D %6t %N" --sort=P`
**Explanation:** shows Partition, Availability, TimeLimit, Nodes, State, NodeList; helps choose the right partition

### submit a job with dependency on a previous job
**Args:** `sbatch --dependency=afterok:12345 --job-name=step2 --partition=compute --time=02:00:00 step2.sh`
**Explanation:** step2.sh runs only after job 12345 completes successfully; use afterany for run-regardless

### submit a GPU job for deep learning or variant calling
**Args:** `sbatch --partition=gpu --gres=gpu:1 --cpus-per-task=8 --mem=64G --time=24:00:00 gpu_job.sh`
**Explanation:** --gres=gpu:1 requests 1 GPU; specify gpu:TYPE:N for specific GPU type (e.g., gpu:a100:2)

### check cluster-wide resource availability
**Args:** `sinfo -N -l --format="%20N %10P %6t %10c %10m %30f %20G"`
**Explanation:** per-node view showing NodeName, Partition, State, CPUs, Memory, Features, GRES (GPUs); helps find free resources

### generate a Slurm script template for RNA-seq alignment
**Args:** `sbatch --wrap="module load star/2.7.10b && STAR --runThreadN $SLURM_CPUS_PER_TASK --genomeDir /ref/star_index --readFilesIn R1.fq.gz R2.fq.gz --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample1_" --job-name=star_align --partition=compute --cpus-per-task=16 --mem=48G --time=06:00:00`
**Explanation:** --wrap runs inline command; uses $SLURM_CPUS_PER_TASK for thread count; suitable for quick single-sample runs

### check your fairshare and account allocation
**Args:** `sacctmgr show assoc user=$USER format=Account,User,Share,GrpTRES,MaxTRES,QOS`
**Explanation:** shows account associations, share allocations, resource limits, and available QoS; helps understand scheduling priority
