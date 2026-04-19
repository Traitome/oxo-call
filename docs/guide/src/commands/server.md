# server

Manage remote servers for SSH-based command execution on workstations and HPC clusters.

## Synopsis

```
oxo-call server add      <NAME> --host <HOST> [--user <USER>] [--port <PORT>]
                          [--identity-file <PATH>] [--type <TYPE>] [--scheduler <SCHED>]
                          [--work-dir <DIR>]
oxo-call server remove   <NAME>
oxo-call server list
oxo-call server status   <NAME>
oxo-call server ssh-config
oxo-call server run      <SERVER> <TOOL> <TASK> [--model <MODEL>]
oxo-call server dry-run  <SERVER> <TOOL> <TASK> [--model <MODEL>]
```

## Description

The `server` command manages SSH-connected remote servers. Register workstations
or HPC cluster login nodes, then use `server run` / `server dry-run` to generate
and preview commands for remote execution.

### Server types

| Type | Description |
|------|-------------|
| `workstation` | Standalone server — commands run directly |
| `hpc` | HPC cluster login node — commands should be submitted via a scheduler (Slurm, PBS, SGE, LSF, HTCondor) |

For **HPC clusters**, oxo-call detects compute-intensive commands and warns
against direct execution on login nodes. It suggests wrapping commands with the
appropriate scheduler (e.g., `sbatch --wrap="..."` for Slurm).

### SSH configuration

Servers are stored in the oxo-call config file (`config.toml`) under
`[[server.hosts]]`. You can also import hosts from `~/.ssh/config`.

## Subcommands

### `server add`

Register a new remote server:

```bash
# Add an HPC cluster login node
oxo-call server add mycluster --host login.hpc.edu --user alice --type hpc --scheduler slurm

# Add a standalone workstation
oxo-call server add workbox --host 10.0.0.5 --type workstation

# Add with custom SSH port and key
oxo-call server add dev --host dev.example.com --port 2222 --identity-file ~/.ssh/id_dev
```

#### Options

| Flag | Description |
|------|-------------|
| `--host <HOST>` | SSH hostname or IP address (required) |
| `--user <USER>` | SSH username (defaults to current user) |
| `--port <PORT>` | SSH port (defaults to 22) |
| `--identity-file <PATH>` | Path to SSH private key file |
| `--type <TYPE>` | `workstation` (default) or `hpc` |
| `--scheduler <SCHED>` | Job scheduler: `slurm`, `pbs`, `sge`, `lsf`, `htcondor` |
| `--work-dir <DIR>` | Default working directory on remote |

For HPC nodes, oxo-call automatically tries to detect the scheduler.

### `server remove`

Remove a registered server:

```bash
oxo-call server remove mycluster
```

### `server list`

List all registered servers:

```bash
oxo-call server list
```

Displays server name, SSH destination, type, scheduler, and user.

### `server status`

Check SSH connectivity to a registered server:

```bash
oxo-call server status mycluster
```

Returns whether the server is reachable and, for HPC nodes, the detected scheduler.

### `server ssh-config`

Import and display hosts from `~/.ssh/config`:

```bash
oxo-call server ssh-config
```

Lists all non-wildcard hosts with their hostname, user, and port.

### `server run`

Generate an LLM-powered command and display it for remote execution:

```bash
oxo-call server run mycluster samtools "sort input.bam by coordinate"
oxo-call server run mycluster samtools "sort bam" --no-stream  # disable streaming output
```

For HPC login nodes, warns about compute-intensive commands and suggests
scheduler submission.

### `server dry-run`

Preview a command for remote execution without connecting:

```bash
oxo-call server dry-run mycluster bwa "align reads.fq to reference.fa with 8 threads"
oxo-call server dry-run mycluster bwa "align reads" --no-stream  # disable streaming output
```

Shows the SSH command to execute and, for HPC nodes, the scheduler-wrapped version.

## HPC login node safety

When running commands on HPC login nodes, oxo-call:

1. **Detects compute-intensive commands** (samtools, bwa, STAR, Python, etc.)
2. **Warns** that direct execution on login nodes is discouraged
3. **Suggests scheduler submission** using the configured scheduler (sbatch, qsub, etc.)

This helps prevent accidental resource-intensive jobs on shared login nodes.

## Examples

```bash
# Register and use an HPC cluster
oxo-call server add hpc1 --host login.hpc.edu --user alice --type hpc --scheduler slurm
oxo-call server status hpc1
oxo-call server dry-run hpc1 samtools "sort input.bam by coordinate"

# Import SSH config hosts and register one
oxo-call server ssh-config
oxo-call server add myhost --host 10.0.0.1 --type workstation

# List all servers
oxo-call server list
```
