# job

Manage named jobs — a personal library of command shortcuts with full lifecycle
support: scheduling, execution history, status tracking, and LLM generation.

## Synopsis

```
oxo-call job add       <NAME> <COMMAND> [--description <DESC>] [--tag <TAG>]... [--schedule <CRON>]
oxo-call job remove    <NAME>
oxo-call job list      [--tag <TAG>] [--builtin]
oxo-call job show      <NAME>
oxo-call job run       <NAME> [--server <SERVER>] [--dry-run]
oxo-call job edit      <NAME> [--command <CMD>] [--description <DESC>] [--tag <TAG>]...
                              [--schedule <CRON>] [--clear-schedule]
oxo-call job rename    <FROM> <TO>
oxo-call job status    [<NAME>]
oxo-call job history   <NAME> [-n <LIMIT>]
oxo-call job schedule  <NAME> [<CRON>]
oxo-call job generate  <DESCRIPTION> [--name <NAME>] [--tag <TAG>]... [--dry-run]
oxo-call job import    <BUILTIN-NAME> [--as-name <NAME>]
```

Aliases: `j` (short alias), `cmd` (backward-compatible alias)

## Description

The `job` command is your personal library of named shell commands — similar to
shell aliases but with rich metadata, scheduling, execution history, and remote
execution support.

Jobs are stored in `~/.local/share/oxo-call/jobs.toml`.  
Run history is stored in `~/.local/share/oxo-call/job_runs.jsonl`.

### Key features

| Feature | Description |
|---------|-------------|
| Named shortcuts | Save any shell command with a memorable name |
| Tags & organisation | Group jobs by tags for easy filtering |
| Execution history | Every run is recorded with timestamps and exit codes |
| Status tracking | See at a glance which jobs succeeded, failed, or never ran |
| Cron scheduling | Attach a schedule expression to document intended frequency |
| Remote execution | Run any job on a registered SSH server |
| LLM generation | Describe a task in plain English → job is generated and saved |
| Built-in templates | 25+ pre-defined jobs for ops, cluster management, and bioinformatics |

## Subcommands

### `job add`

Save a new named job:

```bash
oxo-call job add gpu-check 'nvidia-smi'
oxo-call job add squeue-me 'squeue -u $USER' --tag slurm --tag hpc \
    --description 'Show my SLURM jobs'
oxo-call job add disk-check 'df -h' --schedule '0 * * * *'
```

#### Options

| Flag | Description |
|------|-------------|
| `--description` / `-d` | Brief description of what the job does |
| `--tag` / `-t` | Tag for organisation (repeatable) |
| `--schedule` | Cron expression (e.g. `"0 * * * *"`) |

---

### `job list`

List saved jobs or browse built-in templates:

```bash
oxo-call job list                    # all user jobs
oxo-call job list --tag slurm        # filter by tag
oxo-call job list --builtin          # browse built-in templates
oxo-call job list --builtin --tag k8s  # filter built-ins by tag
```

The `--builtin` flag shows the pre-defined job templates shipped with
oxo-call. Use `job import` to copy one into your personal library.

---

### `job show`

Show full details of a job, including last run status and schedule:

```bash
oxo-call job show squeue-me
```

---

### `job run`

Execute a job locally or on a registered remote server:

```bash
oxo-call job run squeue-me
oxo-call job run gpu-check --server mycluster  # SSH execution
oxo-call job run disk-check --dry-run          # preview only
```

Every execution is recorded in `job_runs.jsonl`. View history with
`job history <name>` or `job status <name>`.

---

### `job edit`

Update an existing job:

```bash
# Update the command
oxo-call job edit squeue-me --command 'squeue -u $USER -o "%.18i %.9P %.8j %.8u %.2t"'
# Add a schedule
oxo-call job edit disk-check --schedule '*/30 * * * *'
# Remove a schedule
oxo-call job edit disk-check --clear-schedule
```

---

### `job rename`

Rename a job:

```bash
oxo-call job rename old-name new-name
```

---

### `job status`

Show execution status for one job or a summary table of all jobs:

```bash
oxo-call job status            # all jobs summary
oxo-call job status squeue-me  # single job with recent run history
```

Output includes run count, last run time, last exit code, and schedule.

---

### `job history`

Show the execution run log for a job:

```bash
oxo-call job history squeue-me
oxo-call job history squeue-me -n 20    # last 20 runs
```

Each entry shows timestamp, exit code, duration, and the server if remote.

---

### `job schedule`

Set or clear a cron schedule on a job:

```bash
oxo-call job schedule disk-check '0 * * * *'   # every hour
oxo-call job schedule disk-check               # clear schedule
```

The schedule is metadata only — it does **not** register a system cron job
automatically. To run a job on a cron schedule, add it to your crontab:

```cron
0 * * * *  oxo-call job run disk-check
```

---

### `job generate`

Use the LLM to create a job from a plain-English description:

```bash
oxo-call job generate 'check disk usage and list top 10 largest directories'
oxo-call job generate 'show all failed SLURM jobs in the last 24 hours' \
    --name slurm-failures --tag slurm
oxo-call job generate 'list all kubernetes pods in error state' --dry-run
```

The generated command is saved to your library unless `--dry-run` is given.

#### Options

| Flag | Description |
|------|-------------|
| `--name` / `-n` | Override the auto-derived job name |
| `--tag` / `-t` | Tags to assign to the generated job |
| `--dry-run` | Print the generated command without saving |

> **Requires** a configured LLM provider. See `oxo-call config set llm.api_token`.

---

### `job import`

Copy a built-in job template into your personal library:

```bash
oxo-call job import squeue-me
oxo-call job import disk --as-name my-disk    # save with a different name
```

Use `oxo-call job list --builtin` to see all available templates.

---

### `job remove`

Delete a job from your library:

```bash
oxo-call job remove gpu-check
```

---

## Built-in job templates

oxo-call ships 25+ pre-defined jobs for common operations. Browse them with
`oxo-call job list --builtin`.

### System / ops

| Name | Command | Tags |
|------|---------|------|
| `disk` | `df -h` | system, ops |
| `mem` | `free -h` | system, ops |
| `cpu` | `top -bn1 \| head -20` | system, ops |
| `gpu` | `nvidia-smi` | gpu, ops |
| `ps-me` | `ps aux \| grep $USER` | system, ops |
| `ports` | `ss -tulnp` | network, ops |
| `iface` | `ip addr show` | network, ops |
| `big-files` | `du -sh * … \| sort -rh \| head -20` | fs, ops |

### SLURM

| Name | Description | Tags |
|------|-------------|------|
| `squeue-me` | My SLURM jobs | slurm, hpc |
| `squeue-all` | All queue entries | slurm, hpc |
| `sacct-me` | Accounting records | slurm, hpc |
| `sinfo` | Partition/node status | slurm, hpc |
| `scancel-me` | Cancel all my jobs | slurm, hpc |

### PBS / Torque

| Name | Description | Tags |
|------|-------------|------|
| `qstat-me` | My PBS jobs | pbs, hpc |
| `pbsnodes` | Node status | pbs, hpc |

### LSF

| Name | Description | Tags |
|------|-------------|------|
| `bjobs-me` | My LSF jobs | lsf, hpc |
| `bhosts` | LSF host status | lsf, hpc |

### Kubernetes

| Name | Description | Tags |
|------|-------------|------|
| `k8s-pods` | All pods across namespaces | k8s, cluster |
| `k8s-nodes` | Node status with IPs | k8s, cluster |
| `k8s-events` | Recent events by time | k8s, cluster |

### Docker

| Name | Description | Tags |
|------|-------------|------|
| `docker-ps` | Running containers table | docker, ops |
| `docker-clean` | Prune stopped containers | docker, ops |

### Git

| Name | Description | Tags |
|------|-------------|------|
| `git-log` | Last 20 commits as graph | git, dev |
| `git-stash-list` | All stash entries | git, dev |

### Bioinformatics

| Name | Description | Tags |
|------|-------------|------|
| `count-reads` | Count reads in FASTQ files | bioinformatics, data |
| `bam-stats` | `samtools flagstat` on all BAMs | bioinformatics, samtools |

## Data storage

| File | Location | Contents |
|------|----------|----------|
| `jobs.toml` | `$OXO_CALL_DATA_DIR/jobs.toml` | Job library (TOML) |
| `job_runs.jsonl` | `$OXO_CALL_DATA_DIR/job_runs.jsonl` | Run history (JSONL) |

> If `$OXO_CALL_DATA_DIR` is not set, the default platform data directory is
> used (e.g. `~/.local/share/oxo-call/` on Linux).

### Legacy migration

If you have a `cmds.toml` file from an older version of oxo-call, it is
automatically migrated to `jobs.toml` on first use. The original `cmds.toml`
is kept as a backup.

## Examples

```bash
# Quick ops check
oxo-call job import disk
oxo-call job run disk

# SLURM workflow
oxo-call job import squeue-me
oxo-call job import sacct-me
oxo-call job run squeue-me

# Schedule disk check and wire it to cron
oxo-call job add disk-alert 'df -h | awk "NR>1 && $5+0>90 {print \"WARN: \"$0}"'
oxo-call job schedule disk-alert '*/15 * * * *'
echo '*/15 * * * *  oxo-call job run disk-alert' | crontab -

# LLM-generated job
oxo-call job generate 'show Docker containers consuming over 1 GB memory'

# Remote execution
oxo-call job run squeue-me --server mycluster
```
