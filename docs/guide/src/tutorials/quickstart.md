# Quick Start

This guide walks you through your first oxo-call session in under 5 minutes, with the fastest path from installation to a safe, explainable command.

> **Test Data:** To follow along with real files, you can download small test datasets:
> - [samtools test data](https://github.com/samtools/samtools/tree/develop/test) — small BAM/SAM files
> - [nf-core test datasets](https://github.com/nf-core/test-datasets) — FASTQ, BAM, and reference files for various pipelines
> - Or create a minimal test BAM: `samtools view -b -h /dev/null -o test.bam` (empty BAM for testing command syntax)

## Step 1: Install oxo-call

Choose the path with the least friction for your environment:

```bash
# Option A: Download pre-built binary (recommended)
# Visit https://github.com/Traitome/oxo-call/releases

# Option B: Install via Bioconda
conda install oxo-call -c bioconda -c conda-forge
# or with mamba (faster)
mamba install oxo-call -c bioconda -c conda-forge

# Option C: Install via Cargo
cargo install oxo-call
```

See the [Installation guide](./installation.md) for detailed instructions.

## Step 2: Obtain a License

A signed license file is required for core commands and is free for academic use.

```bash
# Apply for a free academic license by emailing w_shixiang@163.com
# See the License Setup guide for details

# Place your license.oxo.json at the default path:
# Linux:   ~/.config/oxo-call/license.oxo.json
# macOS:   ~/Library/Application Support/io.traitome.oxo-call/license.oxo.json

# Or point to it at runtime:
export OXO_CALL_LICENSE=/path/to/license.oxo.json
```

## Step 3: Configure Your LLM

### GitHub Copilot (Default)

```bash
oxo-call config set llm.api_token <your-github-token>
```

### OpenAI

```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token <your-openai-key>
```

### Anthropic

```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token <your-anthropic-key>
```

### Ollama (Local, No Token Needed)

```bash
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2
```

Verify your configuration before you generate commands:

```bash
oxo-call config verify
```

## Step 4: Run Your First Command

> See the [completion guide](../commands/completion.md) for generating shell completion (e.g., `zsh`, `bash`) scripts for oxo-call.

### Preview a command (dry-run)

This is the recommended starting point because it shows the exact flags and explanation before anything runs.

```bash
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
```

Expected output:

```
Command: samtools sort -o sorted.bam input.bam
Explanation: Uses -o to specify the output file; coordinate sort is the default behavior.
```

### Execute a command

Once the preview looks right, run the same task for real.

```bash
oxo-call run samtools "index sorted.bam"
```

Expected output:

```
Command: samtools index sorted.bam
Explanation: Creates a .bai index file for random access to the sorted BAM.
→ Running: samtools index sorted.bam
✓ Exit code: 0
```

### Ask for confirmation before executing

Use this when commands are destructive, expensive, or still being reviewed by a teammate.

```bash
oxo-call run --ask bcftools "call variants from my.bam against ref.fa"
```

Expected output:

```
Command: bcftools mpileup -f ref.fa my.bam | bcftools call -mv -o variants.vcf
Explanation: mpileup generates genotype likelihoods; call -mv outputs variant sites only.
Execute this command? [y/N]
```

## Step 5: Explore More Features

### Check available skills

```bash
oxo-call skill list
```

### View cached documentation

```bash
oxo-call docs list
oxo-call docs show samtools
```

### Review command history

```bash
oxo-call history list
```

## What's Next?

- Learn about [Configuration](./configuration.md) options
- Explore the [Command Reference](../commands/run.md)
- Understand the [Skill System](../reference/skill-system.md)
- Try the [Workflow Engine](../commands/workflow.md) for pipeline automation
