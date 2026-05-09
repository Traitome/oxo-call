---
name: ansible
category: IT Automation / Configuration Management
description: Ansible is an open-source automation tool for configuration management, application deployment, orchestration, and task automation. It uses a declarative YAML-based language (Playbooks) to define automated workflows across multiple managed hosts without requiring agent software on target systems.
tags: [automation, devops, configuration-management, orchestration, deployment, ssh-based, idempotent]
author: AI-generated
source_url: https://docs.ansible.com/
---

## Concepts

- **Agentless Architecture**: Ansible operates over SSH or WinRM connections, requiring no daemon or agent installed on managed nodes—just Python and SSH access.
- **Playbook Format**: Playbooks are written in YAML and define a list of plays (tasks) to be executed on specific hosts or groups, using declarative state-based declarations.
- **Inventory Management**: Hosts are defined in inventory files (INI or YAML format) with grouping, variables, and connection parameters, enabling targeted automation across subsets.
- **Modules as Units**: Ansible ships with thousands of built-in modules (apt, yum, docker, git, shell, file, template, etc.) that execute on managed nodes and return JSON results for conditionals.
- **Idempotency Principle**: Well-written playbooks are idempotent—running them multiple times produces the same end state without unintended side effects.

## Pitfalls

- **Using shell/command Instead of Native Modules**: Invoking shell or command modules bypasses idempotency checks and can cause drift or incomplete state reconciliation on re-runs.
- **Mixing Tabs and Spaces in YAML Playbooks**: YAML is whitespace-sensitive; inconsistent indentation with tabs causes parse errors that are hard to debug.
- **Not Using check Mode (--check) Before Production Runs**: Skipping dry-run validation can result in destructive changes propagating to production before validation.
- **Hardcoding Credentials or Secrets in Playbooks**: Storing passwords, API keys, or tokens directly in playbooks creates security vulnerabilities; use Ansible Vault or environment variables instead.
- **Ignoring Host Key Checking**: Disabling host key checking globally (e.g., via environment variables) without context exposes the workflow to man-in-the-middle attacks.

## Examples

### Install a package using the apt module

**Args:** `localhost -m apt -a "name=git state=present" -c become`
**Explanation:** Installs git on localhost using the apt module and elevated privileges; the `become` flag enables sudo.

### Create a directory if it does not exist

**Args:** `all -m file -a "path=/opt/app config state=directory mode=0755" -c become`
**Explanation:** Creates the directory /opt/app with mode 0755 on all hosts; the module is idempotent—running again produces no change.

### Restart a service only if it is running

**Args:** `webservers -m service -a "name=nginx state=restarted" -c become`
**Explanation:** Restarts nginx on hosts in the webservers group; Ansible checks current state before acting, reducing unnecessary restarts.

### Copy a template file with variable substitution

**Args:** `dbservers -m template -a "src=/templates/db.conf.j2 dest=/etc/db.conf mode=0644" -c become`
**Explanation:** Copies the Jinja2 template db.conf.j2 to /etc/db.conf on dbservers, substituting variables defined in inventory or group_vars.

### Synchronize a local directory to remote hosts

**Args:** `appservers -m synchronize -a "src=/build/app dest=/opt/app delete=no recursive=yes"`
**Explanation:** Uses the synchronize module (rsync wrapper) to push the build directory to appservers; more efficient than copy for large files.

### Define a playbook to deploy a web application

**Args:** `--playbook-file deploy.yml --check`
**Explanation:** Runs the deploy.yml playbook in check (dry-run) mode to validate tasks and notify about changes before applying.

### Fetch a file from remote to control node

**Args:** `dbserver -m fetch -a "src=/var/log/app.log dest=/logs/{{ inventory_hostname }}/flat=yes"`
**Explanation:** Fetches app.log from dbserver into local /logs directory organized by hostname; flat=yes preserves single-file structure.

### Run a playbook limiting execution to a specific tag

**Args:** `--tags "install,configure" --playbook-file main.yml`
**Explanation:** Executes only roles and tasks tagged install or configure, skipping other tagged sections in the playbook.

### Gather facts about remote hosts before tasks

**Args:** `gather_facts: true`
**Explanation:** Enables fact gathering before playbook execution; allows use of ansible_* variables (OS version, memory, network, etc.) in conditionals.