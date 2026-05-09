---
name: aspera-cli
category: File Transfer / Bioinformatics
description: Command-line interface for IBM Aspera high-speed file transfers using the FASP (Fast and Secure Protocol) technology, commonly used for moving large genomic datasets, NGS data, and research files to/from cloud storage.
tags:
  - bioinformatics
  - file-transfer
  - FASP-protocol
  - high-speed-transfer
  - cloud-storage
  - NGS-data
  - genomic-data
author: AI-generated
source_url: https://www.ibm.com/docs/en/aspera-fasp
---

## Concepts

- **FASP Protocol**: Aspera uses the proprietary FASP protocol which bypasses TCP congestion control, enabling consistent high-speed transfers regardless of network latency—critical for multi-GB genomic files.
- **Authentication Methods**: Supports both password-based and SSH public key authentication. SSH keys are strongly recommended for automated pipelines as they avoid interactive password prompts and enable scripted transfers.
- **Transfer Specs and Arguments**: Transfers can be specified using either direct source/destination paths or transfer specification files (.tspec) that define source, destination, and transfer options separately.
- **Bandwidth Control**: The `——bandwidth` flag controls transfer speed (e.g., `——bandwidth 10G` for 10 Gbps). Without specification, Aspera attempts to use all available bandwidth, which may impact other network operations.
- **Resume Capability**: Aspera automatically resumes interrupted transfers from checkpoints. Use `——resume` to control resume behavior (never, differ, or always).

## Pitfalls

- **Incorrect Credential Paths**: Specifying an invalid or unreadable SSH private key path causes immediate authentication failure; always verify key file permissions (400 or 600) and existence before running transfers.
- **Missing Trailing Slashes**: Using `——path /data/bam` treats `/data/bam` as a rename operation if it exists as a file, but as a directory move if it contains files; adding trailing slashes (`/data/bam/`) ensures directory contents are transferred correctly.
- **Firewall and Port Blocking**: FASP uses TCP port 33001 (control channel) and dynamic UDP ports (30000-33000) for data transfer; blocked UDP ports result in silent transfer failures with no data copied.
- **Bandwidth Oversubscription**: Running multiple Aspera transfers without bandwidth limits can saturate network links, causing congestion for other services and potentially triggering network alerts.

## Examples

### Upload a directory of BAM files to Aspera server

**Args:** --user myuser --host transfer.example.com --password "mypassword" /local/bam-files/ /remote/bam-files/

**Explanation:** This uploads the local directory to a remote Aspera server using password authentication; the trailing slash on the source ensures all contents are transferred recursively.

### Download a whole-genome VCF file using SSH key authentication

**Args:** --user myuser --host transfer.example.com --priv-key ~/.ssh/aspera_key /remote/genomes/sample.vcf.gz /local/data/

**Explanation:** Downloads a large VCF file using SSH key authentication, eliminating interactive password entry; the private key should have corresponding public key registered on the server.

### Transfer with limited bandwidth to avoid network saturation

**Args:** --user myuser --host transfer.example.com --password "mypassword" --bandwidth 1G /local/large-files/ /remote/backup/

**Explanation:** Limits transfer speed to 1 Gbps to preserve bandwidth for other network operations; essential when running Aspera alongside critical services on shared network infrastructure.

### List all files in a remote Aspera directory

**Args:** --user myuser --host transfer.example.com --password "mypassword" --list /remote/data/目录/*

**Explanation:** Lists files in a remote directory without performing a transfer; useful for verifying file existence and checking remote directory structures before actual transfers.

### Resume an interrupted whole-genome sequencing transfer

**Args:** --user myuser --host transfer.example.com --password "mypassword" --resume differ /local/wgs-data/ /remote/wgs-backup/

**Explanation:** Resumes a partially transferred WGS dataset, comparing file checksums and continuing only for changed or incomplete files; the FASP protocol handles this automatically but the flag ensures proper behavior.

### Create a remote directory before transferring

**Args:** --user myuser --host transfer.example.com --password "mypassword" --mkdir /remote/project-outputs/run-123/

**Explanation:** Creates a remote directory on the Aspera server before transferring files; avoids failed transfers due to non-existent destination paths, which would otherwise require manual server-side setup.

### Mirror a local analysis results directory with delete mode

**Args:** --user myuser --host transfer.example.com --password "mypassword" --mode mirror --delete /local/results/ /remote/backup/results/

**Explanation:** Mirrors local directory to remote, deleting files in destination that no longer exist locally; ensures exact synchronization but must be used carefully to avoid accidental data loss.

### Encrypt transfer with AES-256 for sensitive genomic data

**Args:** --user myuser --host transfer.example.com --password "mypassword" --crypto aes-256 /local/sensitive-data/ /remote/encrypted-store/

**Explanation:** Encrypts data in transit using AES-256 encryption for regulatory compliance; required when transferring PHI or other sensitive healthcare data over untrusted networks.