# Reliable Behavioral Test Set

300 trials across 38 CLI tools and 14 categories. All tasks are realistic
natural language descriptions without embedded flag names.

## Design Principles

- **100% derivable from --help**: Every task can be correctly answered using
  only the tool's --help output. No hidden flags or domain-specific knowledge required.
- **Realistic tasks**: Natural language descriptions that a researcher might actually use
- **Cross-domain**: Covers alignment, variant calling, QC, assembly, RNA-seq,
  text processing, filesystem, compression, networking, and more.

## Files

- `reference_commands.csv` — tool, scenario_id, reference_args, task_description, category
- `usage_descriptions.csv` — tool, scenario_id, desc_id, user_level, description

## Usage

```bash
oxo-bench eval --config bench_config.toml --data-dir docs/bench/reliable-test
```
