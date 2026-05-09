---
name: bioconda-repodata-patches
category: Package Management
description: Tool for creating and applying incremental patches to Bioconda/Conda repodata, reducing bandwidth and update times for package channel metadata.
tags: [conda, bioconductor, package-manager, repodata, patches, biocloud]
author: AI-generated
source_url: https://github.com/bioconda/bioconda-repodata-patches
---

## Concepts

- **Repodata patches** are incremental JSON files that describe changes (additions, modifications, removals) between two versions of a conda channel's repodata.json, allowing clients to update only changed entries rather than downloading the full repodata each time.
- The tool operates on **subdir-specific repodata** (e.g., `linux-64`, `osx-64`, `noarch`): each conda environment targets a specific subdirectory, and patches are generated and applied per subdir to minimize download size.
- Patch format supports **semantic versioning and index compression**: patches include version ranges for packages, and the output can be compressed (zstd or gzip) to further reduce download size for the conda client.
- The tool expects **input as two repodata JSON files** (old and new) and outputs a patch file in the standard conda-repodata patch format understood by conda >= 4.6.
- Patch application is **transparent to the conda client**: when running `conda update`, conda automatically fetches and applies available patches if the channel provides them.

## Pitfalls

- **Mismatched subdir architecture** generates patches that will never be applied: if you generate a patch for `linux-64` but apply it to a `osx-64` environment, conda will reject it and fall back to the full repodata.
- **Corrupted or truncated patch files** cause conda to fall back to the full repodata without warning, leading to unexpectedly large downloads and longer solve times.
- **Creating patches too frequently** (e.g., after every single package upload) can cause patch chain overhead: conda may need to apply dozens of small patches sequentially, which can be slower than downloading one consolidated patch.
- **Forgetting to sign patches** with the channel's GPG key results in conda ignoring them for channels configured with `trusted` checks, causing silent fallback to full repodata.

## Examples

### Generate a repodata patch for the linux-64 subdir
**Args:** build --subdir linux-64 old_repodata.json new_repodata.json --output patches/linux-64.patch
**Explanation:** This creates an incremental patch file comparing the old and new repodata for the linux-64 architecture, which conda clients on Linux systems will fetch to update their local package index.

### Generate a compressed zstd patch to reduce download size
**Args:** build --subdir noarch old_repodata.json new_repodata.json --output patches/noarch.patch.zst --compression zstd
**Explanation:** This generates a patch file with zstd compression, which conda 4.9+ clients can decompress on-the-fly, reducing network transfer size by 30-50% for large channels.

### Apply an existing patch to a local repodata file
**Args:** apply --subdir osx-64 current_repodata.json patchfile.patch --output updated_repodata.json
**Explanation:** This manually applies a patch file to an existing repodata file to reconstruct the full state, useful for testing patch correctness before publishing.

### List all available patches in a patches directory
**Args:** list /var/conda/channels/mychannel/patches/
**Explanation:** This lists all patch files present in the channel's patches directory, showing which subdirs and version ranges are covered, helping diagnose missing patches.

### Verify patch integrity and consistency
**Args:** verify --subdir linux-64 patches/linux-64.patch repodata.json
**Explanation:** This checks that the patch applies cleanly to the base repodata and produces the expected result without errors, ensuring the patch won't cause client-side failures.

### Create patch with explicit version range constraints
**Args:** build --subdir linux-64 old_repodata.json new_repodata.json --output patch.json --min-version 4.8 --max-version 4.11
**Explanation:** This restricts the patch to be used only by conda clients version 4.8 through 4.11, avoiding compatibility issues with older or very new conda versions that handle patches differently.

### Generate patch excluding specific package removals
**Args:** build --subdir linux-64 old_repodata.json new_repodata.json --output patch.json --exclude-removed python=3.8
**Explanation:** This creates a patch but deliberately omits removal entries for python version 3.8 packages, which may be needed for environments that still require the old versions.