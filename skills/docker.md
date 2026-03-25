---
name: docker
category: containerization
description: Container platform for building, running, and managing containerized applications
tags: [docker, container, image, kubernetes, devops, build, compose]
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

## Pitfalls

- 'docker rm' removes containers, 'docker rmi' removes images. 'docker system prune -a' removes ALL unused images, containers, and volumes — data that is not in a volume is permanently lost.
- 'docker rm -f <container>' force-removes a running container without warning. All in-container data not stored in volumes is destroyed immediately.
- 'docker run' without --name generates a random container name. Always use --name for containers you intend to manage by name.
- Port binding syntax is -p HOST_PORT:CONTAINER_PORT (host first, container second). Reversing them will bind the wrong ports.
- Volume mounts with -v: use absolute paths for host directories (e.g., -v /home/user/data:/data), not relative paths, as relative paths may be interpreted as named volumes.
- Without --rm, stopped containers accumulate. Use 'docker ps -a' to see all containers including stopped ones, and clean up with 'docker container prune' to remove all stopped containers.

## Examples

### run a container in the background with port mapping and a name
**Args:** `run -d -p 8080:80 --name my-web nginx`
**Explanation:** -d detaches (background); -p 8080:80 maps host port 8080 to container port 80; --name identifies the container

### build a Docker image from a Dockerfile in the current directory
**Args:** `build -t myapp:1.0 .`
**Explanation:** -t tags the image as myapp:1.0; '.' is the build context (current directory with Dockerfile)

### get an interactive shell inside a running container
**Args:** `exec -it my-web bash`
**Explanation:** -it allocates a TTY and attaches stdin; 'bash' is the command to run (use 'sh' if bash is unavailable)

### list all containers including stopped ones
**Args:** `ps -a`
**Explanation:** -a shows all containers; without -a only running containers are shown; add --format for custom output

### view logs of a container and follow new output
**Args:** `logs -f --tail 100 my-web`
**Explanation:** -f follows log output in real time; --tail 100 shows only the last 100 lines first

### mount a local directory as a volume and set environment variables
**Args:** `run -d -v /data/app:/app -e APP_ENV=production --name app myimage:latest`
**Explanation:** -v mounts host /data/app to container /app; -e sets an environment variable

### remove a stopped container
**Args:** `rm my-web`
**Explanation:** removes the named stopped container; use -f to force-remove a running container (data loss risk)

### remove a Docker image
**Args:** `rmi myapp:1.0`
**Explanation:** removes the image by name:tag; fails if a container (even stopped) is still using it; use 'docker ps -a' to check

### remove all stopped containers to free disk space
**Args:** `container prune`
**Explanation:** removes all stopped containers; prompts for confirmation; use -f to skip prompt

### pull the latest version of an image from Docker Hub
**Args:** `pull python:3.12-slim`
**Explanation:** downloads the image without running it; useful to pre-pull before deployment
