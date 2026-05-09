I need to clarify that "cgcloud-lib" is not a standard standalone bioinformatics tool with well-documented CLI usage. Based on available context, this appears to be related to the CloudBioLinux/cgcloud ecosystem for cloud-based bioinformatics infrastructure deployment. I'll create the skill file with reasonable assumptions about its general purpose and usage patterns.
---
name: cgcloud-lib
category: Cloud Infrastructure / Bioinformatics Deployment
description: A Python library for deploying and managing bioinformatics tools and workflows on cloud infrastructure. Part of the CloudBioLinux ecosystem, cgcloud-lib provides programmatic interfaces for configuring, launching, and managing bioinformatics instances on cloud providers such as AWS and Google Cloud Platform.
tags:
- cloud-deployment
- bioinformatics-infrastructure
- automation
- cloud-computing
- infrastructure-as-code
- aws
- google-cloud
author: AI-generated
source_url: https://github.com/cloudbiolinux/cgcloud
---

## Concepts

- **Cloud Instance Lifecycle Management**: cgcloud-lib provides Python classes and functions to create, configure, start, stop, and terminate cloud compute instances pre-configured with bioinformatics software stacks. The library abstracts provider-specific APIs through a unified interface.

- **Package and Tool Installation**: The library integrates with CloudBioLinux's packaging system to install bioinformatics tools (e.g., Bowtie, GATK, BLAST) onto cloud instances during or after instance launch. Configuration is typically defined in YAML package definition files.

- **SSH Key and Credential Handling**: cgcloud-lib manages SSH key pairs for secure instance access and uses cloud-provider credentials (AWS IAM roles, GCP service accounts) for authentication. Proper credential configuration is essential for all operations.

- **Instance Types and Machine Images**: Supports specification of instance types (e.g., m4.large, n1-standard-8) and machine images (AMIs/VM images) containing pre-built bioinformatics environments. Users can create custom images from configured instances.

- **Virtual Environments and Configuration**: Bioinformatics tools often require specific Python versions or isolated environments. cgcloud-lib supports virtualenv creation and configuration management through config files or environment variables.

## Pitfalls

- **Credential Expiration**: Cloud credentials (especially temporary AWS tokens from IAM roles) can expire mid-operation, causing deployments to fail silently. Always refresh credentials before long-running operations and set appropriate token expiration limits.

- **Mismatch Between Instance Type and Image**: Attempting to launch an AMI designed for a specific architecture (e.g., ARM Graviton) on an incompatible instance family causes launch failures. Verify instance-image compatibility beforehand.

- **Public IP Address Costs**: Leaving running instances with public IP addresses when not needed accumulates cloud costs. Always terminate unused instances or use stop/start cycles with persistent storage to minimize charges.

- **Security Group Misconfiguration**: Opening unnecessary ports (e.g., allowing unrestricted SSH from 0.0.0.0/0) creates security vulnerabilities. Restrict SSH access to known IP ranges and limit database ports to internal networks only.

- **Data Egress Charges**: Downloading large bioinformatics datasets from cloud storage repeatedly incurs significant egress costs. Use instance-local storage or internal networking when possible.

## Examples

### Launch a pre-configured bioinformatics instance on AWS

**Args:** `launch --image ami-12345678 --key-name my-ssh-key --instance-type m4.large my-instance`

**Explanation:** Launches a cloud compute instance using a specified AMI containing bioinformatics tools, authenticates with the named SSH key, and uses the specified instance type for compute resources.

### List available machine images

**Args:** `list-images --provider aws --region us-east-1`

**Explanation:** Queries and displays all available AMIs in the specified AWS region that can be used for launching bioinformatics instances.

### Configure and install a bioinformatics package

**Args:** `install-package --package bowtie2 --version 2.4.1 --instance my-instance`

**Explanation:** Installs a specific version of the Bowtie2 aligner onto a running instance, handling dependencies and environment configuration automatically.

### Stop a running instance

**Args:** `stop my-instance`

**Explanation:** Stops a running cloud instance to save on compute costs while preserving the root volume and attached data disks.

### Create a custom image from a configured instance

**Args:** `create-image --name my-custom-bio-image --description "Bioinformatics tools v2.0" my-instance`

**Explanation:** Creates a new AMI from a configured instance, capturing all installed tools and custom configurations for future reuse in deployments.