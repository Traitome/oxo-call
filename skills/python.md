---
name: python
category: programming
description: Python interpreter — run scripts, one-liners, and interactive REPL for data science and scripting
tags: [python, scripting, data-science, programming, repl, jupyter]
author: oxo-call built-in
source_url: "https://docs.python.org/3/using/cmdline.html"
---

## Concepts
- python runs scripts, one-liners, and modules. Key invocations: 'python script.py' (run file), 'python -c "code"' (one-liner), 'python -m module' (run module as script, e.g., python -m http.server).
- Always use 'python3' or 'python' inside an activated virtual environment to target the correct interpreter. 'which python' shows the active interpreter path.
- Module execution with -m: 'python -m venv .venv' creates a virtual environment; 'python -m pip install pkg' ensures pip for the current interpreter; 'python -m pytest' runs tests.
- Python -c for one-liners: use semicolons or \n for multiple statements. -c is great for quick data manipulation: 'python -c "import json,sys; data=json.load(sys.stdin); print(data['key'])"'.
- Environment variables: PYTHONPATH adds directories to the module search path; PYTHONDONTWRITEBYTECODE=1 prevents .pyc file creation; PYTHONUNBUFFERED=1 forces stdout/stderr to be unbuffered (useful in containers).
- Profiling and debugging: 'python -m cProfile -s cumtime script.py' for profiling; 'python -m pdb script.py' for interactive debugging; 'python -W all script.py' to show all warnings.
- -O enables basic optimizations; -OO removes docstrings for smaller bytecode.
- -S disables site module import; useful for minimal environments or testing.
- -v enables verbose mode; shows imported modules and their locations.

## Pitfalls
- On systems with both Python 2 and Python 3, 'python' may invoke Python 2. Use 'python3' explicitly, or check with 'python --version'. Python 2 reached end-of-life in 2020.
- python -c with double quotes inside the code string requires escaping on some shells. Use single-quoted strings inside the code, or use a heredoc.
- Running 'python script.py' from the wrong directory affects relative imports and file paths. Use 'cd /correct/dir && python script.py' or specify absolute paths.
- Modifying sys.path or using PYTHONPATH can shadow standard library modules if directory names collide (e.g., having a local 'json.py' breaks 'import json').
- 'python -m module' looks for the module in sys.path. If the module is not installed or not on the path, it will fail with 'No module named module'.
- Unbuffered output: by default Python buffers stdout. In pipelines or containers, use 'python -u script.py' or set PYTHONUNBUFFERED=1 to see output immediately.
- -OO removes docstrings which may break code that relies on __doc__ attributes.
- -S disables site module which may prevent import of site-packages.

## Examples

### run a Python script
**Args:** `script.py`
**Explanation:** executes script.py with the current Python interpreter; use python3 if python points to Python 2

### run a Python one-liner
**Args:** `-c "print('Hello, World!')"`
**Explanation:** -c executes the quoted string as Python code; useful for quick tests

### run a module as a script (e.g., start an HTTP server)
**Args:** `-m http.server 8080`
**Explanation:** -m runs the named module; http.server starts a simple HTTP server on port 8080

### create a virtual environment
**Args:** `-m venv .venv`
**Explanation:** -m venv creates a virtual environment in .venv/; activate with 'source .venv/bin/activate'

### run a script with an additional module search path
**Args:** `-m pytest tests/ -v`
**Explanation:** -m pytest runs the pytest module; -v verbose output; tests/ is the test directory

### process JSON from stdin with a one-liner
**Args:** `-c "import json,sys; data=json.load(sys.stdin); [print(r['name']) for r in data]"`
**Explanation:** reads JSON from stdin, parses it, and prints the 'name' field of each element

### run a script with unbuffered output (for pipelines)
**Args:** `-u pipeline_script.py`
**Explanation:** -u forces unbuffered stdout/stderr; ensures real-time output in pipelines and containers

### profile a script and show cumulative time
**Args:** `-m cProfile -s cumtime slow_script.py`
**Explanation:** -m cProfile runs the profiler; -s cumtime sorts output by cumulative time

### check Python version
**Args:** `--version`
**Explanation:** prints the Python version; use python3 --version if python is aliased to Python 2

### run a script with a warning filter to show all deprecation warnings
**Args:** `-W all script.py`
**Explanation:** -W all shows all warning categories including DeprecationWarning; useful when upgrading code

### run with optimizations enabled
**Args:** `-O script.py`
**Explanation:** -O enables basic optimizations; removes assert statements and sets __debug__ to False

### run with maximum optimization
**Args:** `-OO script.py`
**Explanation:** -OO removes docstrings in addition to -O optimizations; produces smaller bytecode but breaks code using __doc__

### run in verbose mode to see imports
**Args:** `-v script.py`
**Explanation:** -v enables verbose mode; prints each module as it's imported; useful for debugging import issues

### disable site module for minimal environment
**Args:** `-S -c "import sys; print(sys.path)"`
**Explanation:** -S prevents automatic import of site module; sys.path contains only standard library paths

### run script with custom module search path
**Args:** `-m site --user-site`
**Explanation:** -m site runs the site module; --user-site prints user site-packages directory path
