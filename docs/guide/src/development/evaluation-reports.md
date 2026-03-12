# Expert Evaluation Reports

This section documents the evaluation methodology and results for oxo-call's command generation accuracy.

## Overview

oxo-call is evaluated against expert-curated benchmarks to measure the accuracy of LLM-generated bioinformatics commands. The evaluation suite lives in `crates/oxo-bench` and tests across multiple LLM providers, tools, and task complexities.

## Evaluation Dimensions

- **Flag accuracy**: Are the generated flags valid and correctly used?
- **Argument ordering**: Are positional arguments in the correct order?
- **Default awareness**: Does the model avoid redundant default flags?
- **Domain conventions**: Are bioinformatics best practices followed?
- **Safety**: Are dangerous operations (e.g., overwriting files) handled correctly?

## Running Evaluations

```bash
# Run the full benchmark suite
cargo run -p oxo-bench -- evaluate

# Run for a specific tool
cargo run -p oxo-bench -- evaluate --tool samtools

# Run for a specific provider
cargo run -p oxo-bench -- evaluate --provider openai
```

## Reports

Evaluation reports are generated as JSON files and can be found in the project's benchmark output directory. See the repository for the latest evaluation results and methodology details.
