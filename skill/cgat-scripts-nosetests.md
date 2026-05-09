---
name: cgat-scripts-nosetests
category: testing-and-validation
description: Test runner for the CGAT (Cancer Genome Analysis Toolkit) scripts using the nose testing framework. Discovers and executes Python unit tests, supports test filtering, verbose output, and coverage reporting.
tags:
  - testing
  - python
  - nose
  - cgat
  - bioinformatics
  - unit-tests
  - test-discovery
author: AI-generated
source_url: https://github.com/cgat-developers/cgat
---

## Concepts

- **Nose-based test discovery**: `cgat-scripts-nosetests` uses the nose testing framework to automatically discover and run Python test files matching the pattern `test_*.py` or `*_test.py` within specified directories or modules.
- **Module-level test execution**: Tests can be scoped to specific CGAT modules (e.g., `CGAT.FlowCell`, `CGAT.BamTools`) for targeted validation without running the entire test suite, reducing CI time and enabling focused debugging.
- **Coverage integration**: Supports `--with-coverage` and `--cover-package` flags to generate code coverage reports, essential for bioinformatics pipelines where reliability and correctness are critical for downstream analysis results.
- **Verbosity and output control**: The `-v` / `--verbose` flags control test output detail, while `--logging-config` allows fine-grained control over Python logging levels during test execution, useful for debugging complex I/O operations.

## Pitfalls

- **Incorrect test path specification**: Providing a non-existent module path or directory causes immediate failure with an `ImportError`, wasting CI cycles. Always verify module paths exist before specifying them.
- **Confusing pytest syntax with nose**: Options like `-k` (test filtering by keyword) are pytest-specific; nose uses `--tests=` or pattern matching differently, leading to silent test exclusion if misapplied.
- **Missing test dependencies**: Running tests without `--exe` on systems without executable permissions for test files causes silent failures where individual tests are skipped without warning, producing misleading empty test reports.
- **Verbose flag position sensitivity**: The `-v` flag must appear before the module path argument; placing it after causes the flag to be interpreted as a path, resulting in unrecognized argument errors.

## Examples

### Running all CGAT tests with verbose output
**Args:** `-v CGAT`
**Explanation:** Verbose mode (`-v`) is placed before the module path to enable detailed output for every test case executed in the CGAT module, helping identify which specific tests pass or fail.

### Running tests for a specific CGAT submodule
**Args:** `-v CGAT.Expression`
**Explanation:** Scoping test execution to the `CGAT.Expression` submodule runs only expression analysis tests, reducing execution time when debugging specific toolkit components without affecting unrelated tests.

### Using coverage reporting for a specific package
**Args:** `--with-coverage --cover-package=CGAT.Expression CGAT.Expression`
**Explanation:** The `--with-coverage` flag enables coverage tracking while `--cover-package=CGAT.Expression` limits the coverage report to the expression module, producing actionable code coverage metrics for that component.

### Filtering tests by name pattern
**Args:** `-v CGAT.FlowCell --tests=*test_align*`
**Explanation:** The `--tests=` argument filters execution to only test functions matching the `*test_align*` pattern within the FlowCell module, enabling precise debugging of alignment-related tests without running the full module suite.

### Generating a detailed test report to file
**Args:** `--verbose --logging-level=DEBUG CGAT.Expression`
**Explanation:** Redirecting verbose and debug logging output produces a detailed test execution report that is valuable for post-mortem analysis when tests fail in automated pipelines or remote environments.