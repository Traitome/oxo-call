---
name: pip
category: package-management
description: Python package installer — install and manage packages from PyPI and other sources
tags: [pip, python, package, install, pypi, dependencies, virtualenv]
author: oxo-call built-in
source_url: "https://pip.pypa.io/en/stable/cli/"
---

## Concepts

- pip installs Python packages from PyPI (Python Package Index) and other sources. Always use 'pip install' inside an activated virtual environment (venv, conda) to avoid modifying the system Python.
- Virtual environment workflow: 'python -m venv .venv' creates a venv; 'source .venv/bin/activate' activates it (Linux/macOS) or '.venv\Scripts\activate' (Windows); then 'pip install' installs into the venv.
- requirements.txt: 'pip install -r requirements.txt' installs all packages listed in the file. 'pip freeze > requirements.txt' captures current environment. Pin versions for reproducibility: 'package==1.2.3'.
- pip install --upgrade upgrades a package to the latest version. pip install package==X.Y.Z installs a specific version. pip install 'package>=1.0,<2.0' installs within a version range.
- pip uninstall removes packages. Use -y to skip confirmation. 'pip uninstall -r requirements.txt -y' removes all packages listed in a requirements file.
- Use 'pip show package' to see a package's version, dependencies, and install location. 'pip list --outdated' shows packages with newer versions available.

## Pitfalls

- DANGER: 'pip install' without a virtual environment modifies the system Python, which can break system tools that depend on specific package versions. Always use a venv or conda environment.
- DANGER: 'pip uninstall package -y' removes the package without confirmation and without removing packages that depended on it, potentially breaking other tools.
- Avoid 'sudo pip install' — it installs into system Python and can break OS tools. Use --user or a virtual environment instead.
- pip install does NOT check for circular dependencies or conflicts comprehensively. If you see version conflicts, consider using pip-tools or poetry for dependency management.
- 'pip freeze' captures exact versions including sub-dependencies, making the file less portable. Consider 'pip-compile' (pip-tools) to maintain a clean top-level requirements.in.
- On systems with both Python 2 and 3, 'pip' may point to Python 2's pip. Use 'pip3' or 'python3 -m pip' to ensure you're using the Python 3 package manager.

## Examples

### install a package from PyPI
**Args:** `install requests`
**Explanation:** installs the latest version of requests; always run inside an activated virtual environment

### install a specific version of a package
**Args:** `install numpy==1.26.4`
**Explanation:** == pins to an exact version; use >= for minimum version, ~= for compatible releases

### install all packages from a requirements file
**Args:** `install -r requirements.txt`
**Explanation:** -r reads package list from file; use for reproducible installs across environments

### upgrade a package to the latest version
**Args:** `install --upgrade pandas`
**Explanation:** --upgrade fetches and installs the newest available version

### uninstall a package with confirmation skip
**Args:** `uninstall -y scipy`
**Explanation:** -y skips the confirmation prompt; removes scipy from the current environment

### save current environment packages to requirements.txt
**Args:** `freeze > requirements.txt`
**Explanation:** freeze outputs all installed packages with pinned versions; redirect to save as requirements file

### list all installed packages
**Args:** `list`
**Explanation:** shows all packages with their versions in the active environment

### show outdated packages
**Args:** `list --outdated`
**Explanation:** --outdated shows installed packages with newer versions available on PyPI

### install a package from a local directory or git repository
**Args:** `install -e git+https://github.com/user/repo.git#egg=packagename`
**Explanation:** -e installs in editable/development mode; git+ prefix tells pip to clone from git

### install package without build isolation (for packages needing system libs)
**Args:** `install --no-build-isolation pysam`
**Explanation:** --no-build-isolation lets the package use system-installed libraries during build (e.g., htslib)
