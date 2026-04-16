---
name: singularity
category: containerization
description: Singularity/Apptainer HPC container runtime; runs Docker and SIF containers on HPC clusters without root privileges
tags: [singularity, apptainer, hpc, container, sif, docker, bind, overlay, cache, ngs]
author: oxo-call built-in
source_url: "https://docs.sylabs.io/guides/latest/user-guide/"
---

## Concepts
- Singularity (now Apptainer) is the standard container runtime for HPC clusters; runs as a normal user without a daemon.
- **SIF (Singularity Image Format)**: the container image file (`.sif`); a read-only compressed squashfs archive.
- **Singularity cache directory**: `~/.singularity/cache/` stores pulled images (layers and blobs); override with `SINGULARITY_CACHEDIR` (or `APPTAINER_CACHEDIR`).
- **SINGULARITY_CACHEDIR** / **APPTAINER_CACHEDIR**: set to a shared or scratch path on HPC to avoid re-pulling images; recommended to set in `~/.bashrc` or module.
- `singularity pull` downloads an image from a registry (Docker Hub, Singularity Hub, OCI registry); produces a `.sif` file in the CWD by default.
- `singularity exec image.sif command` runs a command inside the container; equivalent to `docker run --rm image command`.
- `singularity run image.sif` executes the default `%runscript` defined in the image definition.
- `singularity shell image.sif` opens an interactive shell inside the container; useful for debugging.
- **Bind mounts** (`--bind` / `-B`): mounts host paths into the container; default auto-binds: `$HOME`, `/tmp`, `/proc`, `/sys`, `/dev`.
- By default, `$HOME` is auto-mounted; use `--no-home` to suppress it when reproducibility requires a clean home.
- **Overlay images**: `--overlay overlay.img` mounts a writable ext3 image over the SIF; useful for installing packages into an otherwise read-only container.
- Singularity definition files (`.def`) describe how to build an image; `singularity build` creates a SIF from a def file.
- `apptainer` is the new command name (Linux Foundation fork); most `singularity` flags and images are fully compatible.
- `singularity instance` manages persistent background containers.
- `singularity push` uploads images to remote registries.

## Pitfalls
- `singularity build --sandbox` creates a writable directory container; changes are persistent but not reproducible — always `build` a final SIF from the sandbox.
- Not setting `SINGULARITY_CACHEDIR` on HPC: the default `~/.singularity/cache/` fills home quota quickly when pulling large biocontainers (multi-GB images).
- `--bind` paths must exist on the host; Singularity raises an error if a bind source does not exist; check paths before mounting.
- Network inside the container is inherited from the host by default; GPU access requires `--nv` (NVIDIA) or `--rocm` (AMD ROCm) flags.
- Building SIF images requires either root, `sudo singularity build`, or a `fakeroot` setup; on HPC, use `--fakeroot` if your site supports it, or build remotely with `singularity build --remote`.
- Security: containers run as the calling user (same UID/GID); files written inside bind-mounted directories are owned by the host user.
- Mixing Singularity versions between build host and run host can cause format incompatibilities; always match major SIF versions.

## Examples

### pull a Docker image and convert to SIF
**Args:** `pull --dir /scratch/sif/ docker://biocontainers/samtools:1.19`
**Explanation:** --dir writes the .sif to /scratch/sif/ instead of CWD; docker:// fetches from Docker Hub; converts to read-only SIF format

### run a bioinformatics tool inside a container
**Args:** `exec -B /data:/data samtools.sif samtools flagstat /data/input.bam`
**Explanation:** exec runs the command inside the SIF; -B mounts /data from host at /data inside the container; paths inside the container are resolved from there

### open an interactive shell for debugging
**Args:** `shell -B /scratch samtools.sif`
**Explanation:** opens bash inside the container with /scratch bind-mounted; useful for exploring the container environment and debugging tool issues

### run a container with GPU support (NVIDIA)
**Args:** `exec --nv cuda.sif python train.py`
**Explanation:** --nv mounts NVIDIA drivers and CUDA libraries from the host into the container; required for GPU-accelerated tools

### build a SIF from a definition file
**Args:** `build myenv.sif myenv.def`
**Explanation:** compiles the definition file into a read-only SIF; requires root or --fakeroot; use --remote for unprivileged HPC builds

### pull an image with a custom cache directory
**Args:** `pull --dir /project/containers docker://nfcore/rnaseq:3.14`
**Explanation:** downloads the nf-core rnaseq image to /project/containers; share this path with team members to avoid redundant pulls

### run a Nextflow pipeline using local Singularity images
**Args:** `exec -B $PWD:/mnt nfcore_rnaseq.sif nextflow run nf-core/rnaseq --input samplesheet.csv`
**Explanation:** runs Nextflow inside a container with the CWD bind-mounted; useful for reproducible pipeline execution

### cache image to a shared HPC location (bash)
**Args:** `exec -B /dev/null:/dev/null docker://ubuntu:22.04 echo 'cached'`
**Explanation:** pulling any image to the path set in SINGULARITY_CACHEDIR (or APPTAINER_CACHEDIR) caches layers; set the env var to a shared path before pulling

### inspect the contents and metadata of a SIF image
**Args:** `inspect --labels myenv.sif`
**Explanation:** --labels shows the OCI/build labels embedded in the SIF; use --deffile or --runscript to view the definition file or run script

### list all cached Singularity images
**Args:** `cache list`
**Explanation:** lists all images in the SINGULARITY_CACHEDIR (or ~/.singularity/cache/); shows file sizes and allows targeted cleaning with singularity cache clean

### start a persistent container instance
**Args:** `instance start mycontainer.sif myinstance`
**Explanation:** starts a named instance in the background; use `singularity instance list` to view and `singularity exec instance://myinstance` to run commands

### push an image to a remote registry
**Args:** `push myimage.sif library://user/collection/myimage:tag`
**Explanation:** uploads SIF to Sylabs Library or other compatible registry; requires authentication

### run a container with writable overlay
**Args:** `exec --overlay writable_overlay.img mycontainer.sif touch /newfile`
**Explanation:** --overlay mounts a writable ext3 image; changes persist in overlay.img while SIF remains read-only
