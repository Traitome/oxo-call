<div align="center">

# oxo-call

**Model-intelligent orchestration for CLI bioinformatics**

[![CI](https://github.com/Traitome/oxo-call/actions/workflows/ci.yml/badge.svg)](https://github.com/Traitome/oxo-call/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/oxo-call.svg)](https://crates.io/crates/oxo-call)
[![License](https://img.shields.io/badge/license-Academic%20%7C%20Commercial-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](#data-storage)

Describe your task in plain language — `oxo-call` fetches the tool's documentation, grounds the request with a built-in skill, and asks your LLM backend to generate the exact flags you need.

</div>

---

## Architecture

```
  Natural-language task
         │
         ▼
  ┌──────────────────────────────────────────────────────────┐
  │                      oxo-call CLI                        │
  │                                                          │
  │  ┌──────────────────────────────────────────────────┐   │
  │  │              Documentation Layer                  │   │
  │  │  --help output · local index cache · remote URL  │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │ combined docs + task          │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │                  Skill System                     │   │
  │  │       built-in → community registry → user       │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │ grounded prompt               │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │                  LLM Backend                      │   │
  │  │   GitHub Copilot · OpenAI · Anthropic · Ollama   │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │ ARGS: ...                     │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │              Command Execution                    │   │
  │  │        run · dry-run · run --ask                 │   │
  │  └───────────────────────┬──────────────────────────┘   │
  │                          │                               │
  │  ┌───────────────────────▼──────────────────────────┐   │
  │  │            History & Output                       │   │
  │  │       JSONL log · exit code · timestamp          │   │
  │  └──────────────────────────────────────────────────┘   │
  └──────────────────────────────────────────────────────────┘
```

---

## Features

- 🧠 **LLM-powered parameter generation** — describe what you want to do, get the right flags
- 📚 **Automatic documentation fetching** — grabs `--help` output and optionally remote docs
- 🗂️ **Local documentation index** — pre-index tools for faster repeated use
- 🔍 **Dry-run mode** — preview commands before executing
- 📜 **Command history** — track every run with exit codes and timestamps
- 🔧 **Flexible LLM backend** — GitHub Copilot (default), OpenAI, Anthropic, Ollama
- 🎯 **Skill system** — built-in expert knowledge for 100+ bioinformatics tools

---

## Quick Start

### 1. Install

```bash
cargo install oxo-call
# or build from source:
cargo install --path .
```

### 2. Obtain a license

A signed license file is required for core commands (free for academic use — see [License](#license)).

```bash
# Place your license.oxo.json at the default path:
# Linux:   ~/.config/oxo-call/license.oxo.json
# macOS:   ~/Library/Application Support/io.traitome.oxo-call/license.oxo.json

# Or point to it at runtime:
export OXO_CALL_LICENSE=/path/to/license.oxo.json
```

### 3. Configure your LLM token

**GitHub Copilot (default):**
```bash
oxo-call config set llm.api_token <your-github-token>
# Or: export OXO_CALL_LLM_API_TOKEN=... / GITHUB_TOKEN=... / GH_TOKEN=...
```

**OpenAI:**
```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token <your-openai-key>
# Or: export OXO_CALL_LLM_PROVIDER=openai OXO_CALL_LLM_API_TOKEN=...
# OPENAI_API_KEY is also accepted as a fallback.
```

**Anthropic:**
```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token <your-anthropic-key>
# Or: export OXO_CALL_LLM_PROVIDER=anthropic OXO_CALL_LLM_API_TOKEN=...
# ANTHROPIC_API_KEY is also accepted as a fallback.
```

**Ollama (local, no token needed):**
```bash
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2
# Or: export OXO_CALL_LLM_PROVIDER=ollama OXO_CALL_LLM_MODEL=llama3.2
```

### 4. Build a documentation index (optional but recommended)

```bash
oxo-call index add samtools
oxo-call index add bwa
oxo-call index add bcftools
```

### 5. Run with natural language

```bash
# Preview the command without executing (dry-run)
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
# → samtools sort -o sorted.bam input.bam

# Execute immediately
oxo-call run bwa "align reads.fastq to reference.fa using 8 threads, output SAM"

# Ask for confirmation before executing
oxo-call run --ask bcftools "call variants from my.bam against ref.fa and output to variants.vcf"
```

---

## Commands

<div align="center">

| Command | Alias | Description |
|:-------:|:-----:|:-----------:|
| `run` | `r` | Generate parameters with LLM and execute the tool |
| `dry-run` | `d` | Generate parameters and print the command — no execution |
| `index` | — | Add, remove, update, or list locally indexed tool docs |
| `docs` | — | Show, fetch, or locate cached tool documentation |
| `config` | — | Read and write LLM/behavior settings |
| `history` | — | Browse past command runs with exit codes and timestamps |
| `skill` | — | List, show, or manage prompting skill files |
| `workflow` | `wf` | Generate Snakemake / Nextflow workflows from natural language |
| `license` | — | Verify your signed license file |

</div>

---

### `run` — Execute a tool with LLM-generated parameters

```
oxo-call run [OPTIONS] <TOOL> <TASK>
oxo-call r   [OPTIONS] <TOOL> <TASK>   # short alias

Options:
  -a, --ask   Prompt for confirmation before executing
```

Executes immediately by default. Use `dry-run` to preview first, or `--ask` to confirm before running.

```bash
oxo-call run samtools "view only primary alignments from file.bam and save to primary.bam"
```

---

### `dry-run` — Preview the command without executing

```
oxo-call dry-run <TOOL> <TASK>
oxo-call d       <TOOL> <TASK>   # short alias
```

```bash
oxo-call dry-run bwa "align paired reads R1.fastq R2.fastq to hg38.fa using 16 threads"
```

---

### `index` — Manage the local documentation index

Pre-indexing speeds up repeated use and works even when the tool is not installed locally.

```
oxo-call index add    <TOOL> [--url <URL>]   # Index a tool (--help + optional remote URL)
oxo-call index remove <TOOL>                 # Remove a tool from the index
oxo-call index update [TOOL] [--url <URL>]   # Refresh one or all indexed tools
oxo-call index list                          # List all indexed tools
```

```bash
# Index from --help output only
oxo-call index add samtools

# Index from --help + a remote man page / docs site
oxo-call index add bwa --url https://bio-bwa.sourceforge.net/bwa.shtml

# Index a tool that is not installed locally (remote URL only)
oxo-call index add gatk --url https://gatk.broadinstitute.org/hc/en-us/articles/...

# Refresh all indexed tools
oxo-call index update
```

---

### `docs` — View or fetch documentation

```
oxo-call docs show  <TOOL>           # Print cached documentation for a tool
oxo-call docs fetch <TOOL> <URL>     # Fetch and cache docs from a remote URL
oxo-call docs path  <TOOL>           # Print the path to the cached docs file
```

```bash
oxo-call docs show samtools
oxo-call docs fetch bwa https://bio-bwa.sourceforge.net/bwa.shtml
```

---

### `config` — Manage configuration

```
oxo-call config set    <KEY> <VALUE>   # Persist a config value
oxo-call config get    <KEY>           # Show the effective value (env overrides applied)
oxo-call config show                   # Show all stored and effective values side-by-side
oxo-call config verify                 # Verify the LLM config with a live API call
oxo-call config path                   # Print the path to config.toml
```

**Config keys:**

| Key | Default | Environment variable | Description |
|-----|---------|----------------------|-------------|
| `llm.provider` | `github-copilot` | `OXO_CALL_LLM_PROVIDER` | LLM provider: `github-copilot`, `openai`, `anthropic`, `ollama` |
| `llm.api_token` | *(unset)* | `OXO_CALL_LLM_API_TOKEN` | API token. Provider-specific env vars are also supported as fallbacks. |
| `llm.api_base` | *(auto)* | `OXO_CALL_LLM_API_BASE` | Override the API base URL |
| `llm.model` | *(auto)* | `OXO_CALL_LLM_MODEL` | Model name (e.g. `gpt-4o`, `claude-3-5-sonnet-20241022`) |
| `llm.max_tokens` | `2048` | `OXO_CALL_LLM_MAX_TOKENS` | Maximum tokens to generate |
| `llm.temperature` | `0.0` | `OXO_CALL_LLM_TEMPERATURE` | Temperature (lower = more deterministic; `0.0` = fully deterministic output) |
| `docs.auto_update` | `true` | `OXO_CALL_DOCS_AUTO_UPDATE` | Auto-refresh docs on first use |

Environment variables override `config.toml` values for every key above.  
`oxo-call config verify` makes a real API call to confirm the current provider, token, base URL, and model are working. On failure it prints the upstream error and configuration suggestions.

---

### `history` — Command execution history

```
oxo-call history list  [-n <N>] [--tool <TOOL>]   # Show recent history
oxo-call history clear [-y]                        # Clear all history
```

```bash
oxo-call history list                   # Last 20 commands
oxo-call history list -n 50             # Last 50 commands
oxo-call history list --tool samtools   # Filter by tool name
```

---

### `skill` — Manage expert knowledge profiles for tools

Skills are curated TOML files that inject **domain-expert knowledge** into the LLM prompt for a specific bioinformatics tool.  They contain key concepts, common pitfalls, and worked command examples.  When oxo-call finds a matching skill it includes this knowledge in the prompt, which dramatically improves command accuracy — especially for complex tools and smaller/weaker LLM models.

```
oxo-call skill list                          # List all available skills (built-in, community, user)
oxo-call skill show  <TOOL>                  # Display the full skill for a tool
oxo-call skill install <TOOL>               # Install a skill from the community registry
oxo-call skill install <TOOL> --url <URL>   # Install a skill from a custom URL
oxo-call skill remove  <TOOL>               # Remove a community or user-installed skill
oxo-call skill create  <TOOL>               # Print a skill TOML template to stdout
oxo-call skill create  <TOOL> -o out.toml   # Write the template to a file
oxo-call skill path                         # Show the user skills directory path
```

Examples:
```bash
# See what skills are available
oxo-call skill list

# Inspect the samtools skill (concepts, pitfalls, worked examples)
oxo-call skill show samtools

# Install a community skill for a tool not yet built-in
oxo-call skill install bismark

# Scaffold a new skill file for your own tool
oxo-call skill create mytool -o ~/.config/oxo-call/skills/mytool.toml
```

**Skill load priority** (highest wins):
1. User-defined: `~/.config/oxo-call/skills/<tool>.toml`
2. Community-installed: `~/.local/share/oxo-call/skills/<tool>.toml`
3. Built-in: compiled into the binary (120 tools as of this release)

**Built-in skill coverage** spans all major omics domains:

| Domain | Tools |
|--------|-------|
| QC & preprocessing | samtools, fastp, fastqc, multiqc, trimmomatic, cutadapt, trim_galore, picard, fastq-screen |
| Short-read alignment | bwa, bwa-mem2, bowtie2, hisat2, star, chromap |
| Long-read alignment | minimap2, pbmm2 |
| RNA-seq | salmon, kallisto, rsem, stringtie, featurecounts, trinity, arriba |
| Variant calling (SNV/indel) | gatk, bcftools, freebayes, deepvariant, strelka2, varscan2, longshot |
| Structural variants | manta, delly, sniffles, pbsv, survivor, truvari |
| CNV | cnvkit |
| Variant annotation | snpeff, vep, vcftools, vcfanno |
| Phasing & benchmarking | whatshap, shapeit4, hap_py |
| Epigenomics | macs2, deeptools, bismark, methyldackel, pairtools, homer, modkit |
| Metagenomics | kraken2, bracken, metaphlan, diamond, prokka, bakta, metabat2, checkm2, gtdbtk, humann3, centrifuge |
| Single-cell | cellranger, starsolo, kb, velocyto, cellsnp-lite |
| Long-read (ONT/PacBio) | dorado, nanoplot, nanostat, chopper, porechop, racon, medaka, pbccs, pbfusion, nanocomp |
| De novo assembly | spades, megahit, flye, hifiasm, canu, miniasm, wtdbg2, verkko |
| Assembly QC & polishing | quast, busco, pilon |
| Genome annotation | prodigal, augustus, agat, repeatmasker, annot8r, bakta, liftoff |
| Sequence search & comparison | blast, hmmer, mmseqs2, diamond, mash, sourmash |
| Utilities | seqtk, seqkit, bedtools, bedops, bamtools, samtools, tabix, mosdepth, crossmap, igvtools, sra-tools |
| MSA & phylogenetics | mafft, muscle, iqtree2, fasttree |
| Population genomics | plink2, admixture, angsd |
| Comparative & functional genomics | orthofinder, eggnog-mapper |

---

### `workflow` — Native workflow engine + compatibility export

The `workflow` command ships a **lightweight native Rust workflow engine** that
executes `.oxo.toml` pipeline files directly — no Snakemake, Nextflow, or Conda
required.  Snakemake and Nextflow are supported as **compatibility export targets**
for environments that need those formats (e.g., HPC clusters with existing Nextflow
infrastructure).

#### Native engine features

- **DAG-based execution** — steps run in dependency order with full parallelism
  across independent tasks via `tokio::task::JoinSet`
- **Wildcard expansion** — `{sample}` automatically expands per sample; `{params.KEY}` for shared parameters
- **Output caching** — steps whose outputs are newer than their inputs are skipped automatically
- **Gather steps** — `gather = true` aggregates across all samples (e.g., MultiQC)
- **Zero external dependencies** — only the bioinformatics tools themselves need to be installed

#### Workflow file format (`.oxo.toml`)

```toml
[workflow]
name        = "rnaseq"
description = "Bulk RNA-seq pipeline"

[wildcards]
sample = ["sample1", "sample2", "sample3"]   # {sample} expands for each

[params]
threads    = "8"
star_index = "/data/star_hg38"
gtf        = "/data/gencode.v44.gtf"

[[step]]
name    = "fastp"
cmd     = "fastp --in1 data/{sample}_R1.fq.gz --in2 data/{sample}_R2.fq.gz ..."
inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
outputs = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz"]

[[step]]
name       = "star"
depends_on = ["fastp"]
cmd        = "STAR --genomeDir {params.star_index} ..."
outputs    = ["aligned/{sample}/Aligned.sortedByCoord.out.bam"]

[[step]]
name       = "multiqc"
gather     = true           # runs once after all {sample} instances of deps
depends_on = ["fastp", "star"]
cmd        = "multiqc qc/ aligned/ -o results/multiqc/"
outputs    = ["results/multiqc/multiqc_report.html"]
```

#### Commands

```
oxo-call workflow list                            # List built-in templates
oxo-call workflow show  <NAME>                    # Print native .oxo.toml template
oxo-call workflow show  <NAME> --engine snakemake # Print the Snakemake version
oxo-call workflow show  <NAME> --engine nextflow  # Print the Nextflow version
oxo-call workflow dry-run <FILE|NAME>             # Preview all steps (no execution)
oxo-call workflow run     <FILE|NAME>             # Execute the workflow
oxo-call workflow export  <FILE|NAME> --to snakemake  # Export as Snakefile
oxo-call workflow export  <FILE|NAME> --to nextflow   # Export as Nextflow DSL2
oxo-call workflow generate "<TASK>"               # LLM-generated native workflow
oxo-call workflow generate "<TASK>" -e snakemake  # LLM-generated Snakemake workflow
oxo-call workflow generate "<TASK>" -o wf.toml    # Save to file
```

**Built-in templates (native + Snakemake + Nextflow DSL2):**

| Template | Assay | Steps |
|----------|-------|-------|
| `rnaseq` | Bulk RNA-seq | fastp → STAR → featureCounts → MultiQC |
| `wgs` | Whole-genome sequencing | fastp → BWA-MEM2 → GATK BQSR → HaplotypeCaller |
| `atacseq` | ATAC-seq / chromatin accessibility | fastp → Bowtie2 → Picard → MACS3 |
| `metagenomics` | Shotgun metagenomics | fastp → host removal → Kraken2 → Bracken |

Examples:
```bash
# Dry-run the RNA-seq pipeline to preview all steps
oxo-call workflow dry-run rnaseq

# Run the metagenomics workflow from a customised TOML file
cp $(oxo-call workflow show metagenomics > metagenomics.toml && echo metagenomics.toml)
# edit wildcards/params in metagenomics.toml, then:
oxo-call workflow run metagenomics.toml

# Export native workflow to Snakemake for HPC submission
oxo-call workflow export wgs.toml --to snakemake -o Snakefile

# Generate a custom ChIP-seq workflow with LLM (native format by default)
oxo-call workflow generate \
  "ChIP-seq for H3K27ac, paired-end, peak calling against input control" \
  -o chipseq_h3k27ac.toml

# Preview the generated workflow before running
oxo-call workflow dry-run chipseq_h3k27ac.toml
```

---

### Skills vs Docs — what's the difference?

oxo-call uses **two complementary knowledge sources** when building the LLM prompt:

| | `docs` | `skill` |
|---|---|---|
| **What it is** | Raw `--help` output + optionally fetched web docs for a tool | Curated TOML file with concepts, pitfalls, and worked examples |
| **Created by** | Automatically fetched at runtime from the tool itself or a URL | Written by domain experts; built-ins are compiled into the binary |
| **Content** | Exhaustive flag reference — everything the tool can do | Selective expert commentary on the most important patterns and gotchas |
| **LLM role** | Provides factual grounding so the LLM knows valid flags | Provides reasoning scaffolding so the LLM applies flags correctly |
| **Managed with** | `oxo-call docs`, `oxo-call index` | `oxo-call skill` |
| **Stored at** | `~/.local/share/oxo-call/docs/` | `~/.local/share/oxo-call/skills/` (community) or `~/.config/oxo-call/skills/` (user) |

In practice, `docs` answer *"what flags exist?"* while `skills` answer *"which flags should I use for this task, and what mistakes should I avoid?"*  Both are injected into the prompt together for best results.

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `OXO_CALL_LLM_PROVIDER` | Override `llm.provider` |
| `OXO_CALL_LLM_API_TOKEN` | Override `llm.api_token` |
| `OXO_CALL_LLM_API_BASE` | Override `llm.api_base` |
| `OXO_CALL_LLM_MODEL` | Override `llm.model` |
| `OXO_CALL_LLM_MAX_TOKENS` | Override `llm.max_tokens` |
| `OXO_CALL_LLM_TEMPERATURE` | Override `llm.temperature` |
| `OXO_CALL_DOCS_AUTO_UPDATE` | Override `docs.auto_update` |
| `OXO_CALL_LICENSE` | Path to `license.oxo.json` |
| `GITHUB_TOKEN` | GitHub Copilot API token |
| `GH_TOKEN` | GitHub token (fallback for Copilot) |
| `OPENAI_API_KEY` | OpenAI API token (fallback) |
| `ANTHROPIC_API_KEY` | Anthropic API token (fallback) |
| `OXO_API_TOKEN` | Generic token fallback |

---

## Configuration File

Stored at a platform-appropriate path — run `oxo-call config path` to find yours.

| Platform | Path |
|----------|------|
| Linux | `~/.config/oxo-call/config.toml` |
| macOS | `~/Library/Application Support/io.traitome.oxo-call/config.toml` |
| Windows | `%APPDATA%\traitome\oxo-call\config\config.toml` |

---

## Data Storage

Documentation cache and history files:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/oxo-call/` |
| macOS | `~/Library/Application Support/io.traitome.oxo-call/` |
| Windows | `%APPDATA%\traitome\oxo-call\data\` |

---

## Building from Source

```bash
git clone https://github.com/Traitome/oxo-call
cd oxo-call
cargo build --release
# Binary: target/release/oxo-call
```

```bash
# Run tests
cargo test

# Lint
cargo clippy -- -D warnings
cargo fmt -- --check
```

---

## License

**Dual License — Academic Free / Commercial Per-Organization**

<div align="center">

| Use case | License | Cost |
|:--------:|:-------:|:----:|
| Academic research, education, personal non-commercial | [LICENSE-ACADEMIC](LICENSE-ACADEMIC) | **Free** — signed license file required |
| Commercial / production (any organization) | [LICENSE-COMMERCIAL](LICENSE-COMMERCIAL) | **USD 100** — per-org, one-time fee |

</div>

### How to obtain a license

All users — academic and commercial — must have a **signed `license.oxo.json`** file to run core commands. Verification is performed **offline** (no network required) using an Ed25519 signature.

#### Academic License (free)

1. Apply at: <https://github.com/Traitome/oxo-call#license>
2. You will receive a `license.oxo.json` file by email.
3. Place it at the default path for your platform (see table below) or set `OXO_CALL_LICENSE`.

#### Commercial License (per-organization, one-time fee)

1. Contact: <w_shixiang@163.com>
2. Pay the **USD 100** authorization fee.
3. You will receive a `license.oxo.json` signed for your organization.
4. One license covers all employees/contractors within your organization.

#### Public academic test license

For quick evaluation and CI-style smoke testing, this repository publishes a public
academic test license at `docs/public-academic-test-license.oxo.json`.

You can copy it into place with:

```bash
# Linux
cp docs/public-academic-test-license.oxo.json ~/.config/oxo-call/license.oxo.json

# macOS
cp docs/public-academic-test-license.oxo.json ~/Library/Application\\ Support/io.traitome.oxo-call/license.oxo.json
```

The published test license content is:

```json
{
  "schema": "oxo-call-license-v1",
  "license_id": "6548e181-e352-402a-ab72-4da51f49e7b5",
  "issued_to_org": "Public Academic Test License (any academic user)",
  "license_type": "academic",
  "scope": "org",
  "perpetual": true,
  "issued_at": "2026-03-12",
  "signature": "duKJcISYPdyZkw1PbyVil5zTjvLhAYsmbzRpH0n6eRYJET90p1b0rYiHO0cJ7IGR6NLEJWqkY1wBXUkfvUvECw=="
}
```

> **Test use only.** This public academic license is published for evaluation and testing.
> Academic users are still encouraged to apply for a formal academic license.
> Commercial users should contact <w_shixiang@163.com>, pay the **USD 100**
> authorization fee, and obtain a formal commercial license.

#### License file locations

| Platform | Default path |
|----------|-------------|
| Linux | `~/.config/oxo-call/license.oxo.json` |
| macOS | `~/Library/Application Support/io.traitome.oxo-call/license.oxo.json` |
| Windows | `%APPDATA%\oxo-call\license.oxo.json` |

```bash
# Option 1 — Copy to the default location (Linux)
cp license.oxo.json ~/.config/oxo-call/license.oxo.json

# Option 2 — CLI flag
oxo-call --license /path/to/license.oxo.json run samtools "..."

# Option 3 — Environment variable
export OXO_CALL_LICENSE=/path/to/license.oxo.json
oxo-call run samtools "..."

# Verify your license
oxo-call license verify
```

> **Skill files** contributed to the community registry are licensed under **CC-BY-4.0** and remain freely usable by everyone.

---

## Developer Notes — Issuing Licenses (Maintainer Only)

The `crates/license-issuer` workspace member provides an offline signing tool.

### Generate a key pair (once per trust root)

```bash
cargo run -p license-issuer --bin license-issuer -- generate-keypair
# Prints: PRIVATE_KEY_SEED=<base64>  PUBLIC_KEY=<base64>
# ▸ Store the private key securely (password manager / offline vault).
# ▸ Update EMBEDDED_PUBLIC_KEY_BASE64 in src/license.rs with the public key.
```

### Issue an academic license

```bash
export OXO_LICENSE_PRIVATE_KEY="<your-base64-private-key-seed>"
cargo run -p license-issuer --bin license-issuer -- issue \
    --org "Recipient University" \
    --email researcher@uni.edu \
    --type academic \
    --output license.oxo.json
```

### Issue a commercial license

```bash
export OXO_LICENSE_PRIVATE_KEY="<your-base64-private-key-seed>"
cargo run -p license-issuer --bin license-issuer -- issue \
    --org "Example Corp" \
    --email admin@example.com \
    --type commercial \
    --output license.oxo.json
```

> **Never commit your private key.** It should only ever exist on an air-gapped machine or in a secure secret store.

For local development and CI, the private key is not needed for `cargo build` / `cargo test`. The repository ships a pre-signed test fixture at `tests/fixtures/test_license.oxo.json` that is verified against the embedded public key in `src/license.rs`. If you rotate the embedded public key, regenerate that fixture with the new private key and commit the updated file.
