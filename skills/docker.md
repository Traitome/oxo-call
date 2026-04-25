---
name: docker
category: containerization
description: Container platform for building, running, and managing containerized applications
tags: [docker, container, image, kubernetes, devops, build, compose, dockerfile, volume, network]
author: oxo-call built-in
source_url: "https://docs.docker.com/reference/cli/docker/"
---

## Concepts

- Docker commands use subcommands: 'docker <subcommand> [options]'. Main subcommands: run, build, pull, push, ps, images, exec, logs, stop, rm, rmi, compose.
- 'docker run' creates and starts a new container from an image. Key flags: -d (detach/background), -p host:container (port mapping), -v host:container (volume mount), --name (container name), --rm (auto-remove on exit).
- Images vs containers: images are immutable templates (built from Dockerfiles); containers are running instances of images. One image can run as many containers. Use 'docker images' to list images, 'docker ps' to list running containers.
- Dockerfile instructions: FROM (base image), RUN (build-time commands), COPY/ADD (files into image), ENV (environment variables), CMD/ENTRYPOINT (default run command), EXPOSE (document ports).
- docker exec -it runs an interactive command inside a running container. Use 'docker exec -it <container> bash' (or sh) to get a shell inside a running container.
- docker compose (or docker-compose) manages multi-container applications via a YAML config. 'docker compose up -d' starts all services; 'docker compose down' stops and removes containers.
- Resource limits: --memory limits RAM usage; --cpus limits CPU cores; --gpus adds GPU access; prevents one container from consuming all host resources.
- --user runs container as specific user (security best practice); --network connects to custom networks for container communication.
- docker cp copies files between host and container; docker inspect shows detailed container/image metadata in JSON.
- docker system prune removes unused images, containers, networks; docker volume manages persistent data storage.

## Pitfalls

- 'docker rm' removes containers, 'docker rmi' removes images. 'docker system prune -a' removes ALL unused images, containers, and volumes — data that is not in a volume is permanently lost.
- 'docker rm -f <container>' force-removes a running container without warning. All in-container data not stored in volumes is destroyed immediately.
- 'docker run' without --name generates a random container name. Always use --name for containers you intend to manage by name.
- Port binding syntax is -p HOST_PORT:CONTAINER_PORT (host first, container second). Reversing them will bind the wrong ports.
- Volume mounts with -v: use absolute paths for host directories (e.g., -v /home/user/data:/data), not relative paths, as relative paths may be interpreted as named volumes.
- Without --rm, stopped containers accumulate. Use 'docker ps -a' to see all containers including stopped ones, and clean up with 'docker container prune' to remove all stopped containers.
- Without resource limits (--memory, --cpus), a container can consume all host resources and cause system instability.
- --memory-swap must be >= --memory; setting --memory-swap equal to --memory disables swap.
- Exit code 137 means container was killed by OOM Killer (out of memory); increase --memory or fix memory leak.
- Running containers as root (--user not specified) is a security risk; use --user $(id -u):$(id -g) to run as current user.
- docker build cache can cause stale images; use --no-cache when dependencies change or for reproducible builds.

## Examples

### run a container in the background with port mapping and a name
**Args:** `run -d -p 8080:80 --name my-web nginx`
**Explanation:** run subcommand; -d detaches (background); -p 8080:80 maps host port 8080 to container port 80; --name my-web identifies the container; nginx image name

### build a Docker image from a Dockerfile in the current directory
**Args:** `build -t myapp:1.0 .`
**Explanation:** build subcommand; -t myapp:1.0 tags the image as myapp:1.0; '.' is the build context (current directory with Dockerfile)

### get an interactive shell inside a running container
**Args:** `exec -it my-web bash`
**Explanation:** exec subcommand; -it allocates a TTY and attaches stdin; my-web container name; 'bash' is the command to run (use 'sh' if bash is unavailable)

### list all containers including stopped ones
**Args:** `ps -a`
**Explanation:** ps subcommand; -a shows all containers; without -a only running containers are shown; add --format for custom output

### view logs of a container and follow new output
**Args:** `logs -f --tail 100 my-web`
**Explanation:** logs subcommand; -f follows log output in real time; --tail 100 shows only the last 100 lines first; my-web container name

### mount a local directory as a volume and set environment variables
**Args:** `run -d -v /data/app:/app -e APP_ENV=production --name app myimage:latest`
**Explanation:** run subcommand; -d detaches (background); -v /data/app:/app mounts host /data/app to container /app; -e APP_ENV=production sets an environment variable; --name app container name; myimage:latest image

### remove a stopped container
**Args:** `rm my-web`
**Explanation:** rm subcommand; my-web container name; removes the named stopped container; use -f to force-remove a running container (data loss risk)

### remove a Docker image
**Args:** `rmi myapp:1.0`
**Explanation:** rmi subcommand; myapp:1.0 image name:tag; removes the image by name:tag; fails if a container (even stopped) is still using it; use 'docker ps -a' to check

### remove all stopped containers to free disk space
**Args:** `container prune`
**Explanation:** container subcommand with prune; removes all stopped containers; prompts for confirmation; use -f to skip prompt

### pull the latest version of an image from Docker Hub
**Args:** `pull python:3.12-slim`
**Explanation:** pull subcommand; python:3.12-slim image name; downloads the image without running it; useful to pre-pull before deployment

### run container with memory and CPU limits
**Args:** `run -d --memory 4g --cpus 2 --name limited-app myimage`
**Explanation:** run subcommand; -d detaches (background); --memory 4g limits RAM to 4GB; --cpus 2 limits to 2 CPU cores; --name limited-app container name; myimage image; prevents resource exhaustion

### run container with GPU access
**Args:** `run -d --gpus all --name gpu-app nvidia/cuda:12.0-base`
**Explanation:** run subcommand; -d detaches (background); --gpus all provides access to all GPUs; --name gpu-app container name; nvidia/cuda:12.0-base image; requires NVIDIA Docker runtime installed

### run container as non-root user
**Args:** `run -d --user $(id -u):$(id -g) -v $(pwd):/data --name secure-app myimage`
**Explanation:** run subcommand; -d detaches (background); --user $(id -u):$(id -g) runs as current user/group; -v $(pwd):/data mounts current directory to /data; --name secure-app container name; myimage image; security best practice to avoid running as root

### copy files between host and container
**Args:** `cp host_file.txt my-web:/app/data/`
**Explanation:** cp subcommand; host_file.txt source file on host; my-web:/app/data/ destination path in container; copies host_file.txt to /app/data/ inside my-web container; reverse order to copy from container to host

### inspect container details in JSON
**Args:** `inspect my-web`
**Explanation:** inspect subcommand; my-web container/image name; outputs detailed container configuration and state in JSON; useful for scripting and debugging

### create and use a custom network
**Args:** `network create my-network && docker run -d --network my-network --name web nginx`
**Explanation:** network subcommand with create; my-network network name; docker run -d --network my-network --name web nginx runs container on custom network; containers on same network can communicate by name

### run multi-container application with compose
**Args:** `compose -f docker-compose.yml up -d`
**Explanation:** compose subcommand; -f docker-compose.yml specifies compose file; up starts services; -d detached mode; starts all services in detached mode

### stop and remove compose services
**Args:** `compose down -v`
**Explanation:** compose subcommand; down stops and removes containers; -v also removes volumes (data loss warning)

### clean up unused Docker resources
**Args:** `system prune -a -f`
**Explanation:** system subcommand with prune; -a removes all unused images; -f skips confirmation; removes all unused images, containers, networks; use with caution
