---
name: kubectl
category: hpc
description: Kubernetes command-line tool for deploying, managing, and debugging containerized applications on clusters
tags: [kubernetes, k8s, kubectl, cluster, container, pod, deployment, job, orchestration, computing]
author: oxo-call built-in
source_url: "https://kubernetes.io/docs/reference/kubectl/"
---

## Concepts

- kubectl controls Kubernetes clusters. Core resource types: Pod (single container group), Deployment (managed replicas), Service (network endpoint), Job (batch execution), CronJob (scheduled tasks), Namespace (isolation boundary).
- Common operations: `kubectl get` (list), `kubectl describe` (details), `kubectl create` (create), `kubectl apply -f file.yaml` (declarative create/update), `kubectl delete` (remove), `kubectl logs` (view logs), `kubectl exec` (run commands in containers).
- Namespace isolation: use `-n NAMESPACE` or `--all-namespaces` (`-A`) to target resources. Set default namespace: `kubectl config set-context --current --namespace=NAMESPACE`.
- For bioinformatics batch computing, use Kubernetes Jobs: `kubectl create job NAME --image=IMAGE -- command args`. Jobs run to completion and can be parallelized with `.spec.parallelism` and `.spec.completions`.
- Resource requests and limits control scheduling: `resources.requests.cpu` (guaranteed), `resources.limits.cpu` (maximum), `resources.requests.memory`, `resources.limits.memory`. Always set both for production workloads.
- Use `kubectl top nodes` and `kubectl top pods` to monitor CPU/memory usage. Requires metrics-server installed on the cluster. Use `kubectl describe node NODENAME` for detailed capacity info.
- ConfigMaps and Secrets manage configuration: `kubectl create configmap NAME --from-file=FILE`, `kubectl create secret generic NAME --from-literal=key=value`. Mount them as volumes or environment variables in pods.

## Pitfalls

- `kubectl apply -f` is declarative and idempotent (preferred); `kubectl create -f` is imperative and fails if the resource already exists. Always prefer `apply` for reproducible deployments.
- Memory limits use binary suffixes: `Mi` (mebibytes), `Gi` (gibibytes). Using `M` or `G` gives decimal units. `128Mi` ≠ `128M`. Be consistent with request/limit units.
- Pods in CrashLoopBackOff: check logs with `kubectl logs POD --previous` to see the crash output. Common causes: wrong image, missing config, OOM kill (check `kubectl describe pod POD` for OOMKilled events).
- `kubectl exec -it POD -- bash` requires the container to have bash installed. Use `sh` as fallback. The `--` separator is required to prevent kubectl from interpreting container arguments.
- Deleting a Deployment deletes all its Pods. Deleting a Pod managed by a Deployment causes automatic recreation. To stop all replicas: `kubectl scale deployment NAME --replicas=0`.
- YAML indentation must be spaces, never tabs. Incorrect indentation in manifests causes cryptic parse errors. Use `kubectl apply --dry-run=client -f file.yaml` to validate before applying.
- Jobs don't clean up completed pods by default. Set `.spec.ttlSecondsAfterFinished` to auto-delete, or manually clean with `kubectl delete job NAME`.

## Examples

### list all pods in the current namespace
**Args:** `get pods -o wide`
**Explanation:** -o wide shows additional columns including node name and IP; omit -o for compact view

### view logs from a specific pod
**Args:** `logs pod-name --tail=100 -f`
**Explanation:** --tail=100 shows last 100 lines; -f follows/streams new log output in real-time

### run a one-time bioinformatics job
**Args:** `create job fastp-qc --image=biocontainers/fastp:0.23.4 -- fastp -i /data/reads_R1.fq.gz -I /data/reads_R2.fq.gz -o /output/clean_R1.fq.gz -O /output/clean_R2.fq.gz`
**Explanation:** creates a Job that runs fastp in a container; mount data volumes via YAML for real workflows

### apply a YAML manifest to create or update resources
**Args:** `apply -f alignment-job.yaml`
**Explanation:** declarative create/update; use --dry-run=client to preview changes without applying

### execute a command inside a running pod
**Args:** `exec -it pod-name -- bash`
**Explanation:** -it enables interactive terminal; -- separates kubectl args from container command; use sh if bash unavailable

### get detailed information about a pod (including events and status)
**Args:** `describe pod pod-name`
**Explanation:** shows container status, events (scheduling, pulling, started), resource usage, and any error messages

### check cluster node resources and capacity
**Args:** `top nodes`
**Explanation:** shows CPU and memory usage per node; requires metrics-server; use 'describe node NODE' for detailed capacity

### scale a deployment up or down
**Args:** `scale deployment alignment-workers --replicas=10`
**Explanation:** sets desired pod count to 10; Kubernetes manages creation/deletion to reach target

### delete completed jobs and their pods
**Args:** `delete jobs --field-selector status.successful=1`
**Explanation:** removes all successfully completed jobs; use status.failed=1 for failed jobs

### view resource usage of pods sorted by CPU
**Args:** `top pods --sort-by=cpu -A`
**Explanation:** -A shows all namespaces; --sort-by=cpu sorts by CPU usage; useful for finding resource-heavy pods

### create a configmap from a reference genome index config file
**Args:** `create configmap genome-config --from-file=genome.conf=/path/to/genome.conf`
**Explanation:** creates a ConfigMap that can be mounted as a volume in pods; key=genome.conf maps to the file content
