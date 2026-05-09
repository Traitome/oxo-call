---
name: cgat-pipelines-nosetests
category: Testing / Quality Assurance
description: Test runner for CGAT bioinformatics pipelines using the nose testing framework. Executes unit and integration tests for pipeline components, supporting test discovery, filtering, and reporting.
tags:
  - testing
  - bioinformatics
  - pipelines
  - nose
  - quality-assurance
  - cgat
author: AI-generated
source_url: https://github.com/cgat-developers/cgat-pipelines
---

## Concepts

- CGAT pipelines tests use the nose framework for discovery and execution; tests are typically defined in files matching `test_*.py` and organized under pipeline-specific `Tests/` subdirectories.
- Test execution requires the CGAT environment variables (`CGAT_HOME`, `DATADIR`) to be set so that fixture data and reference files can be located at runtime.
- The tool supports verbose output (`-v`), test filtering by pattern (`-m`), and selective test execution by name prefix (`-t`) to help isolate failures during development.
- Exit codes follow standard Unix conventions: `0` means all tests passed, `1` means test failures occurred, and `2` means a collection or setup error occurred.
- Test fixtures often require external data archives (e.g., BAM files, FASTA references) that must be pre-downloaded or linked via the CGAT configuration before running tests.

## Pitfalls

- Running tests from the wrong working directory causes fixture lookup failures because paths are resolved relative to the test module location, not the shell's current directory.
- Skipping the `-v` flag silently hides CGAT-specific diagnostic output; without verbose mode, a failing test may report only a generic assertion error with no context.
- Running only a subset of tests with `-t` can mask integration failures that only appear when all pipeline components are loaded together in a full suite.
- Modifying pipeline configuration files (e.g., `pipeline.yml`) between test runs without cleaning the cache can cause inconsistent state and spurious test failures.
- Missing the nose package dependency results in an import error before any tests are collected; ensure `nose` is installed in the active Python environment.

## Examples

### Run all tests with verbose output
**Args:** `-v`
**Explanation:** The `-v` flag enables verbose output, showing each test name as it runs and providing detailed tracebacks on failure for easier debugging.

### Run only tests matching a specific pattern
**Args:** `-v -m differential_expression`
**Explanation:** The `-m` flag filters tests to only those containing `differential_expression` in their name, useful for targeting tests during development of a specific feature.

### Run a specific test by its full name
**Args:** `-v -t test_load_bam_file`
**Explanation:** The `-t` flag runs only the test with the exact name `test_load_bam_file`, allowing isolated execution of a single failing test case.

### Capture output and stop on first failure
**Args:** `-v -x --nocapture`
**Explanation:** The `-x` flag stops execution at the first test failure, and `--nocapture` ensures printed output is shown immediately rather than buffered.

### Run tests and generate an XML report
**Args:** `--with-xunit --xunit-file=test_results.xml -v`
**Explanation:** The `--with-xunit` plugin outputs test results in JUnit XML format to the specified file, enabling integration with CI/CD systems like Jenkins or GitLab CI.