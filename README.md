# oxo-call

**Model-intelligent orchestration for CLI bioinformatics**

`oxo-call` is a Rust CLI tool that uses LLM intelligence to help you call bioinformatics (and other command-line) tools without memorizing every flag and parameter. Simply describe your task in plain language, and `oxo-call` will automatically fetch the tool's documentation and generate the right command for you.

## Features

- 🧠 **LLM-powered parameter generation** — describe what you want to do, get the right flags
- 📚 **Automatic documentation fetching** — grabs `--help` output and optionally remote docs
- 🗂️ **Local documentation index** — pre-index tools for faster repeated use
- 🔍 **Dry-run mode** — preview commands before executing
- 📜 **Command history** — track every run with exit codes and timestamps
- 🔧 **Flexible LLM backend** — GitHub Copilot (default), OpenAI, Anthropic, Ollama

## Quick Start

### 1. Install

```bash
cargo install --path .
```

### 2. Configure your LLM token

**GitHub Copilot (default):**
```bash
oxo-call config set llm.api_token <your-github-token>
# Or set OXO_CALL_LLM_API_TOKEN or GITHUB_TOKEN / GH_TOKEN
```

**OpenAI:**
```bash
oxo-call config set llm.provider openai
oxo-call config set llm.api_token <your-openai-key>
# Or set OXO_CALL_LLM_PROVIDER=openai and OXO_CALL_LLM_API_TOKEN
# OPENAI_API_KEY is also supported as a backward-compatible fallback
```

**Anthropic:**
```bash
oxo-call config set llm.provider anthropic
oxo-call config set llm.api_token <your-anthropic-key>
# Or set OXO_CALL_LLM_PROVIDER=anthropic and OXO_CALL_LLM_API_TOKEN
# ANTHROPIC_API_KEY is also supported as a backward-compatible fallback
```

**Ollama (local, no token needed):**
```bash
oxo-call config set llm.provider ollama
oxo-call config set llm.model llama3.2
# Or set OXO_CALL_LLM_PROVIDER=ollama and OXO_CALL_LLM_MODEL=llama3.2
```

### 3. Build a documentation index (optional but recommended)

```bash
oxo-call index add samtools
oxo-call index add bwa
oxo-call index add bcftools
```

### 4. Run with natural language!

```bash
# Preview the command (dry-run)
oxo-call dry-run samtools "sort input.bam by coordinate and output to sorted.bam"
# → samtools sort -o sorted.bam input.bam

# Execute the command
oxo-call run bwa "align reads.fastq to reference.fa using 8 threads, output SAM"

# Ask before executing
oxo-call run --ask bcftools "call variants from my.bam against ref.fa and output to variants.vcf"
```

## Subcommands

### `run` — Execute a tool with LLM-generated parameters

```
oxo-call run [OPTIONS] <TOOL> <TASK>
oxo-call r   [OPTIONS] <TOOL> <TASK>   # short alias

Options:
  -a, --ask   Ask for confirmation before executing
```

Example:
```bash
oxo-call run samtools "view only primary alignments from file.bam and save to primary.bam"
```

`run` now executes immediately by default. Use `dry-run` to preview without executing, or `run --ask` if you want a confirmation prompt before the generated command runs.

---

### `dry-run` — Preview the command without executing

```
oxo-call dry-run <TOOL> <TASK>
oxo-call d      <TOOL> <TASK>   # short alias
```

Example:
```bash
oxo-call dry-run bwa "align paired reads R1.fastq R2.fastq to hg38.fa using 16 threads"
```

---

### `index` — Manage the local documentation index

Pre-indexing tools speeds up repeated use and works even when the tool is not installed.

```
oxo-call index add    <TOOL> [--url <URL>]   # Index a tool (fetches --help + optional URL)
oxo-call index remove <TOOL>                 # Remove a tool from the index
oxo-call index update [TOOL] [--url <URL>]   # Update one or all indexed tools
oxo-call index list                          # Show all indexed tools
```

Examples:
```bash
# Index from --help output
oxo-call index add samtools

# Index from --help + a remote URL (e.g. man page or docs site)
oxo-call index add bwa --url https://bio-bwa.sourceforge.net/bwa.shtml

# Just from a remote URL (for tools not installed locally)
oxo-call index add gatk --url https://gatk.broadinstitute.org/hc/en-us/articles/...

# Update all indexed tools
oxo-call index update
```

---

### `docs` — View or fetch documentation

```
oxo-call docs show  <TOOL>         # Show cached documentation for a tool
oxo-call docs fetch <TOOL> <URL>   # Fetch and cache docs from a URL
oxo-call docs path  <TOOL>         # Show the path to the cached docs file
```

Examples:
```bash
oxo-call docs show samtools
oxo-call docs fetch bwa https://bio-bwa.sourceforge.net/bwa.shtml
```

---

### `config` — Manage configuration

```
oxo-call config set  <KEY> <VALUE>   # Set a config value
oxo-call config get  <KEY>           # Get the effective value (after env overrides)
oxo-call config show                 # Show stored values and effective values
oxo-call config verify               # Verify the effective LLM config with a real API call
oxo-call config path                 # Show config file path
```

**Config keys:**

| Key | Default | Environment variable | Description |
|-----|---------|----------------------|-------------|
| `llm.provider` | `github-copilot` | `OXO_CALL_LLM_PROVIDER` | LLM provider: `github-copilot`, `openai`, `anthropic`, `ollama` |
| `llm.api_token` | *(unset)* | `OXO_CALL_LLM_API_TOKEN` | API token. Backward-compatible provider-specific token env vars are also supported. |
| `llm.api_base` | *(auto)* | `OXO_CALL_LLM_API_BASE` | Override API base URL |
| `llm.model` | *(auto)* | `OXO_CALL_LLM_MODEL` | Model name (e.g. `gpt-4o`, `claude-3-5-sonnet-20241022`) |
| `llm.max_tokens` | `2048` | `OXO_CALL_LLM_MAX_TOKENS` | Maximum tokens to generate |
| `llm.temperature` | `0.0` | `OXO_CALL_LLM_TEMPERATURE` | Temperature (lower = more deterministic; `0.0` = fully deterministic output) |
| `docs.auto_update` | `true` | `OXO_CALL_DOCS_AUTO_UPDATE` | Auto-refresh docs on first use |

Environment variables override values from `config.toml` for these keys. `oxo-call config get <KEY>` reports the effective value after applying environment-variable overrides, and `oxo-call config show` separates stored values from effective ones.

Use `oxo-call config verify` when you want to check whether the current effective provider, token, API base, and model can actually complete a chat request. On failure it prints the upstream error plus targeted configuration suggestions.

---

### `history` — Command execution history

```
oxo-call history list [-n <N>] [--tool <TOOL>]   # Show recent history
oxo-call history clear [-y]                       # Clear all history
```

Examples:
```bash
oxo-call history list           # Show last 20 commands
oxo-call history list -n 50    # Show last 50 commands
oxo-call history list --tool samtools  # Filter by tool
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

| Variable | Used for |
|----------|----------|
| `OXO_CALL_LLM_PROVIDER` | Override `llm.provider` |
| `OXO_CALL_LLM_API_TOKEN` | Override `llm.api_token` |
| `OXO_CALL_LLM_API_BASE` | Override `llm.api_base` |
| `OXO_CALL_LLM_MODEL` | Override `llm.model` |
| `OXO_CALL_LLM_MAX_TOKENS` | Override `llm.max_tokens` |
| `OXO_CALL_LLM_TEMPERATURE` | Override `llm.temperature` |
| `OXO_CALL_DOCS_AUTO_UPDATE` | Override `docs.auto_update` |
| `GITHUB_TOKEN` | GitHub Copilot API token |
| `GH_TOKEN` | GitHub token (fallback) |
| `OPENAI_API_KEY` | OpenAI API token |
| `ANTHROPIC_API_KEY` | Anthropic API token |
| `OXO_API_TOKEN` | Generic fallback token for providers without a dedicated token env var |

## Configuration File

The config file is stored at a platform-appropriate location:
- **Linux**: `~/.config/oxo-call/config.toml`
- **macOS**: `~/Library/Application Support/io.traitome.oxo-call/config.toml`
- **Windows**: `%APPDATA%\traitome\oxo-call\config\config.toml`

Find it with: `oxo-call config path`

## Data Storage

Documentation cache and history are stored at:
- **Linux**: `~/.local/share/oxo-call/`
- **macOS**: `~/Library/Application Support/io.traitome.oxo-call/`
- **Windows**: `%APPDATA%\traitome\oxo-call\data\`

## Building from Source

```bash
git clone https://github.com/Traitome/oxo-call
cd oxo-call
cargo build --release
# Binary is at target/release/oxo-call
```

## Running Tests

```bash
cargo test
```

## License

**Dual License — Academic Free / Commercial Per-Organization**

| Use case | License | Cost |
|----------|---------|------|
| Academic research, education, personal non-commercial | [LICENSE-ACADEMIC](LICENSE-ACADEMIC) | **Free** — license file required |
| Commercial / production (any organization) | [LICENSE-COMMERCIAL](LICENSE-COMMERCIAL) | Paid — per-org, one-time fee |

### Licensing / How to obtain a license

All users — academic and commercial — must have a **signed license file** to run
core commands.  The license is verified **offline** (no network required) using
an Ed25519 signature.

#### Academic License (free)

1. Apply via: <https://github.com/Traitome/oxo-call#license>
2. You will receive a `license.oxo.json` file by email.
3. Place the file at:
   - Linux: `~/.config/oxo-call/license.oxo.json`
   - macOS: `~/Library/Application Support/io.traitome.oxo-call/license.oxo.json`
   - Windows: `%APPDATA%\oxo-call\license.oxo.json`
   - Legacy Unix fallback also accepted: `~/.config/oxo-call/license.oxo.json`

#### Commercial License (per-organization, one-time fee)

1. Contact: <w_shixiang@163.com>
2. You will receive a `license.oxo.json` signed for your organization.
3. Place it in the same location as the academic license (see above).
4. One license covers all employees/contractors within your organization.

#### Using the license file

```bash
# Option 1 — Place in the default location
# Linux:
cp license.oxo.json ~/.config/oxo-call/license.oxo.json

# macOS:
cp license.oxo.json ~/Library/Application\\ Support/io.traitome.oxo-call/license.oxo.json

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

### Generate a key pair (once per trust root / deployment)

```bash
cargo run -p license-issuer --bin license-issuer -- generate-keypair
# Prints: PRIVATE_KEY_SEED=<base64>  PUBLIC_KEY=<base64>
# Store the private key securely (password manager / offline vault).
# Update EMBEDDED_PUBLIC_KEY_BASE64 in src/license.rs with the public key.
# You do NOT need to regenerate keys for every code change or release.
```

### Issue an academic license

```bash
export OXO_LICENSE_PRIVATE_KEY="<your-base64-private-key-seed>"
cargo run -p license-issuer --bin license-issuer -- issue \
    --org "Recipient University" \
    --email researcher@uni.edu \
    --type academic \
    --output license.oxo.json
# Send license.oxo.json to the recipient.
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

> **Never commit your private key.**  The private key should only ever exist on
> an air-gapped machine or in a secure secret store.

For local development and GitHub Actions, the private key is not needed for
normal `cargo build` / `cargo test` runs. The repository keeps a pre-signed
test fixture at `tests/fixtures/test_license.oxo.json`, and the runtime verifies
that fixture using the embedded public key in `src/license.rs`. If you rotate
the embedded public key, regenerate that fixture license with the new private
key and commit the updated fixture; CI still does not need the private key.
