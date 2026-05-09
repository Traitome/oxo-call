---
name: azure-cli
category: cloud-management
description: Command-line interface for Microsoft Azure cloud services. A unified CLI tool for managing Azure resources including virtual machines, web apps, databases, containers, Kubernetes clusters, and serverless functions.
tags: azure, cloud, microsoft, management, devops, infrastructure
author: AI-generated
source_url: https://docs.microsoft.com/en-us/cli/azure/
---
## Concepts
- Azure CLI uses a command structure where `az` is followed by the resource type and then an action (e.g., `az vm list`, `az storage account create`, `az aks cluster create`)
- All operations require authentication via `az login` (interactive) or environment variables (service principal) before executing commands; unauthenticated requests return "Please run 'az login' first" error
- Subscriptions organize billing and resource management; use `az account set --subscription` to switch between tenants before creating or querying resources
- Output formatting is controlled by `--output` flag with values `json`, `table`, or `tsv`; JSON is default and provides complete data for scripting
- Resource groups serve as logical containers for related Azure resources; most resource creation commands require `--resource-group` or `-g` flag to specify the target group

## Pitfalls
- Running commands without first executing `az login` results in authentication errors and prevents any resource operations from completing
- Operating on the wrong subscription (due to not setting `az account set`) causes resources to be created in the wrong tenant, leading to billing confusion and access issues
- Specifying a non-unique storage account name (e.g., "storage" or "mybackup") fails with "Storage account name already taken" errors because all storage account names must be globally unique across Azure
- Using `--location` with an invalid or unsupported region name causes deployment failures with "Location not supported" errors for the specific resource provider
- Forgetting to specify `--name` when required (for operations like `az vm restart`) returns "missing required argument" errors and the command aborts

## Examples

### Log in to Azure interactively via browser
**Args:** `login`
**Explanation:** Opens a browser window for device code flow authentication, establishing a session for all subsequent `az` commands. Required first step before any resource management.

### List all virtual machines in the current subscription
**Args:** `vm list --output table`
**Explanation:** Queries all VMs in the active subscription and displays them in human-readable table format with name, resource group, power state, and location columns.

### Create a new resource group in East US region
**Args:** `group create --name myResourceGroup --location eastus`
**Explanation:** Creates a logical container for organizing related resources in the East US Azure region; resource groups are mandatory for most resource deployments.

### Create a general-purpose v2 storage account
**Args:** `storage account create --name uniqueStorageAcct123 --resource-group myResourceGroup --location eastus --sku Standard_LRS --kind StorageV2`
**Explanation:** Creates a locally redundant storage account with the specified globally unique name in the designated resource group; suitable for blobs, files, queues, and tables.

### Get the current subscription details
**Args:** `account show`
**Explanation:** Displays JSON output with subscription ID, tenant ID, subscription name, and user information for the currently active subscription context.

### Deploy an Nginx container to Azure Container Instances
**Args:** `container create --name nginx-container --resource-group myResourceGroup --image nginx --location eastus --ip-address public`
**Explanation:** Deploys an Nginx Docker container to Azure Container Instances with a public IP address, enabling external HTTP access to the running container.

### Scale an Azure Kubernetes Service cluster
**Args:** `aks scale --name myAKSCluster --resource-group myResourceGroup --agent-count 5`
**Explanation:** Scales the node pool in an existing AKS cluster from the current count to 5 agent nodes, provisioning additional compute capacity for workloads.