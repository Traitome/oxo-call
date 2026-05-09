---
name: argparse
category: Python CLI Library
description: Python's standard library for parsing command-line arguments and generating help messages. Used by most bioinformatics tools to define their CLI interfaces with positional arguments, optional flags, and automatic help generation.
tags: [python, cli, argument-parsing, bioinformatics-tooling, command-line]
author: AI-generated
source_url: https://docs.python.org/3/library/argparse.html
---

## Concepts

- **Argument Types**: argparse supports positional arguments (required, ordered) and optional arguments (flags starting with `-` or `--`). Positional arguments are accessed by name, optional arguments by their destination attribute.
- **Action Modes**: The `action` parameter defines behavior—`store_true` creates boolean flags, `append` allows multiple values, `count` tracks flag occurrences, and `store` (default) assigns a value.
- **Type Conversion**: The `type` parameter automatically converts string input (e.g., `int`, `float`, `open` for file objects). Custom callables can validate and transform input.
- **Mutual Exclusion**: `add_mutually_exclusive_group()` prevents certain combinations. Calling `--verbose` with `--quiet` in the same group raises an error.

## Pitfalls

- **Ignoring the Namespace**: Forgetting to use `parse_args()` and accessing raw `sys.argv` instead. The Namespace object holds all parsed values; accessing `sys.argv[1:]` directly bypasses validation and type conversion.
- **Misusing `action='append'`**: Defining `--input` with `append` but passing a comma-separated string (`--input file1.txt,file2.txt`) instead of separate flags (`--input file1.txt --input file2.txt`). The string becomes a single list element rather than multiple.
- **Confusing `nargs` Values**: Using `nargs='?'` incorrectly—it allows one optional value (omitting the flag is allowed), but using it with required arguments or forgetting it exists leads to unexpected behavior.
- **Shallow Argument Copying**: Copying an `argparse.Namespace` object with `new_ns = old_ns` creates a shallow reference, not an independent copy. Modifying `new_ns.prop` mutates `old_ns`.

## Examples

### Create a basic positional argument

**Args:** `prog.py input.txt`

**Explanation:** This passes `input.txt` as the first positional argument, which becomes `args.input` (or `args[0]` if using `parse_known_args`).

### Use an optional flag with a value

**Args:** `prog.py --output results.txt input.txt`

**Explanation:** The `--output` flag requires a value (default `nargs=None`), so `results.txt` is assigned to `args.output` and `input.txt` to `args.input`.

### Append multiple values to a list

**Args:** `prog.py --sample sample1.tsv --sample sample2.tsv --sample sample3.tsv`

**Explanation:** With `action='append'`, each `--sample` flag appends its value to a list: `args.sample == ['sample1.tsv', 'sample2.tsv', 'sample3.tsv']`.

### Use a boolean flag for verbosity

**Args:** `prog.py --verbose input.txt`

**Explanation:** With `action='store_true'`, the flag is `False` when omitted and `True` when present—no value needed on the command line.

### Convert input to integer

**Args:** `prog.py --threads 4 input.txt`

**Explanation:** The `type=int` converter parses the string `'4'` to integer `4` before assignment. Invalid strings like `'four'` raise a `SystemExit` error.

### Require at least one argument with nargs='+'

**Args:** `prog.py --files file1.txt file2.txt`

**Explanation:** `nargs='+'` requires one or more values; omitting the argument after `--files` raises an error. The result is `args.files == ['file1.txt', 'file2.txt']`.

### Set a default value for missing optionals

**Args:** `prog.py input.txt`

**Explanation:** With `default='output.tsv'`, the `--output` flag defaults to `'output.tsv'` when absent. Accessing `args.output` returns the default without requiring input.

### Use mutually exclusive options

**Args:** `prog.py --quiet input.txt`

**Explanation:** When grouped with `add_mutually_exclusive_group()`, using `--quiet` prevents `--verbose` in the same group; using both raises an error.