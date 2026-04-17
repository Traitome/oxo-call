use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "oxo-call",
    version,
    about = "Model-intelligent orchestration for CLI bioinformatics",
    long_about = r#"oxo-call uses LLM intelligence + expert Skills to help you call bioinformatics
tools without memorizing every flag and parameter.

Quick start:
  1. Obtain a license file (academic: free; commercial: w_shixiang@163.com)
     and place it at the platform config path
     (macOS: ~/Library/Application Support/io.traitome.oxo-call/license.oxo.json;
      legacy Unix path ~/.config/oxo-call/license.oxo.json is also accepted)

  2. Set up your API token:
       oxo-call config set llm.api_token <your-github-token>

  3. Run a tool with a natural-language task (documentation is auto-indexed):
       oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"
       oxo-call dry-run bwa "align reads.fastq to reference.fa with 8 threads"

  4. Optionally pre-build the documentation index or add extra sources:
       oxo-call docs add samtools
       oxo-call docs add samtools --url https://www.htslib.org/doc/samtools.html
       oxo-call docs add myapp --file /path/to/manual.md
       oxo-call docs add myapp --dir /path/to/docs/

Skills — expert knowledge for reliable LLM output even with small models:
  oxo-call skill list               # see all built-in skills (samtools, bwa, gatk, ...)
  oxo-call skill show samtools      # inspect the samtools skill
  oxo-call skill install <tool>     # install a community skill from the registry
  oxo-call skill create <tool>      # generate a skill template for a new tool

Supported LLM providers: github-copilot (default), openai, anthropic, ollama
Task descriptions may be written in any language (English, Chinese, etc.).
License: Dual (Academic free / Commercial per-org) — run 'oxo-call license' for details."#
)]
pub struct Cli {
    /// Path to the license file (overrides OXO_CALL_LICENSE env var and default path)
    #[arg(long, global = true, value_name = "PATH")]
    pub license: Option<PathBuf>,

    /// Enable verbose output (show docs source, skill info, LLM details)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a bioinformatics tool with LLM-generated parameters
    #[command(
        visible_alias = "r",
        long_about = "\
Execute a bioinformatics tool with LLM-generated parameters.\n\n\
The tool must be installed and available in your PATH. Documentation is \
automatically fetched and cached on first use.\n\n\
EXAMPLES:\n  \
  oxo-call run samtools 'sort input.bam by coordinate and output to sorted.bam'\n  \
  oxo-call run bwa 'align reads.fq to ref.fa with 8 threads'\n  \
  oxo-call run --ask samtools 'filter only mapped reads from input.bam'\n  \
  oxo-call run --model gpt-4 samtools 'index sorted.bam'\n  \
  oxo-call run --json samtools 'flagstat input.bam'\n  \
  oxo-call run --verify samtools 'sort input.bam by coordinate'\n  \
  oxo-call run --scenario doc samtools 'sort bam'"
    )]
    Run {
        /// The tool to run (must be in PATH)
        tool: String,
        /// Natural-language description of the task (any language supported)
        task: String,
        /// Ask for confirmation before executing the generated command
        #[arg(short, long)]
        ask: bool,
        /// Override the LLM model for this invocation (e.g. gpt-4, claude-3-5-sonnet-20241022)
        #[arg(short, long, value_name = "MODEL")]
        model: Option<String>,
        /// Skip cached documentation and fetch fresh --help output
        #[arg(long)]
        no_cache: bool,
        /// Output result as JSON (useful for scripting and CI integration)
        #[arg(long)]
        json: bool,
        /// After execution, ask the LLM to verify results: checks output files,
        /// stderr patterns, and exit code, then reports issues and suggestions
        #[arg(long)]
        verify: bool,
        /// Set a named variable for `{KEY}` substitution in the task description (repeatable)
        ///
        /// Example: --var SAMPLE=sample1 --var THREADS=8
        #[arg(short = 'V', long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        /// Read input items from a file (one per line); runs the command for each item
        ///
        /// Use `{item}` in the task to refer to the current item.
        /// Blank lines and lines starting with `#` are ignored.
        #[arg(short = 'i', long = "input-list", value_name = "FILE")]
        input_list: Option<String>,
        /// Comma-separated input items; runs the command for each item
        ///
        /// Example: --input-items sample1.bam,sample2.bam,sample3.bam
        #[arg(long = "input-items", value_name = "ITEMS")]
        input_items: Option<String>,
        /// Number of parallel jobs when using --input-list or --input-items (default: 1)
        #[arg(short = 'j', long = "jobs", default_value = "1", value_name = "N")]
        jobs: usize,
        /// Stop processing after the first failed item (exit immediately, do not run remaining items)
        #[arg(short = 'x', long = "stop-on-error")]
        stop_on_error: bool,
        /// When the command fails, automatically use the LLM to analyze stderr
        /// and retry with a corrected command (up to 2 retries)
        #[arg(long)]
        auto_retry: bool,
        /// Force a specific workflow scenario (auto-detected by default)
        ///
        /// Scenarios:
        /// - basic: Tool + Task only (fastest)
        /// - prompt: Basic + custom prompt
        /// - doc: Basic + documentation + mini-skill generation
        /// - skill: Basic + skill file
        /// - full: Doc + skill combined (most accurate)
        #[arg(long, value_name = "SCENARIO")]
        scenario: Option<String>,
    },
    #[command(
        name = "dry-run",
        visible_alias = "d",
        long_about = "\
Preview the command that would be generated without executing it.\n\n\
This is useful for reviewing and testing LLM-generated commands before running \
them, or for building shell scripts from natural-language descriptions.\n\n\
EXAMPLES:\n  \
  oxo-call dry-run samtools 'sort input.bam by coordinate'\n  \
  oxo-call dry-run bwa 'align reads.fq to reference.fa with 8 threads'\n  \
  oxo-call dry-run --model gpt-4 gatk 'call variants on sample.bam'\n  \
  oxo-call dry-run --json samtools 'flagstat input.bam'\n  \
  oxo-call dry-run --scenario doc samtools 'sort bam'"
    )]
    DryRun {
        /// The tool to preview
        tool: String,
        /// Natural-language description of the task (any language supported)
        task: String,
        /// Override the LLM model for this invocation (e.g. gpt-4, claude-3-5-sonnet-20241022)
        #[arg(short, long, value_name = "MODEL")]
        model: Option<String>,
        /// Skip cached documentation and fetch fresh --help output
        #[arg(long)]
        no_cache: bool,
        /// Output result as JSON (useful for scripting and CI integration)
        #[arg(long)]
        json: bool,
        /// [Ablation] Do not load the skill file for this tool
        #[arg(long, hide = true)]
        no_skill: bool,
        /// [Ablation] Do not load tool documentation (--help output)
        #[arg(long, hide = true)]
        no_doc: bool,
        /// [Ablation] Do not use the oxo-call system prompt
        #[arg(long, hide = true)]
        no_prompt: bool,
        /// Set a named variable for `{KEY}` substitution in the task description (repeatable)
        ///
        /// Example: --var SAMPLE=sample1 --var THREADS=8
        #[arg(short = 'V', long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        /// Read input items from a file (one per line); previews the command for each item
        ///
        /// Use `{item}` in the task to refer to the current item.
        #[arg(short = 'i', long = "input-list", value_name = "FILE")]
        input_list: Option<String>,
        /// Comma-separated input items; previews the command for each item
        #[arg(long = "input-items", value_name = "ITEMS")]
        input_items: Option<String>,
        /// Force a specific workflow scenario (auto-detected by default)
        #[arg(long, value_name = "SCENARIO")]
        scenario: Option<String>,
    },

    /// Manage tool documentation (add, remove, update, list, show)
    #[command(visible_alias = "doc")]
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
    },

    /// [Deprecated] Manage the local documentation index — use 'docs' instead
    #[command(visible_alias = "i", hide = true)]
    Index {
        #[command(subcommand)]
        command: IndexCommands,
    },

    /// Manage oxo-call configuration
    #[command(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Show command execution history
    #[command(visible_alias = "hist")]
    History {
        #[command(subcommand)]
        command: HistoryCommands,
    },

    /// Manage bioinformatics tool skills (expert knowledge for LLM prompts)
    #[command(visible_alias = "sk")]
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },

    /// Show license information or verify the current license file
    #[command(visible_alias = "lic")]
    License {
        #[command(subcommand)]
        command: Option<LicenseCommands>,
    },

    /// Generate bioinformatics workflow files (Snakemake / Nextflow)
    #[command(visible_alias = "wf")]
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },

    /// Manage remote servers for SSH-based execution on workstations and HPC clusters
    #[command(
        visible_alias = "srv",
        long_about = "\
Manage remote servers for SSH-based command execution.\n\n\
Register workstations or HPC cluster login nodes, then run commands remotely. \
For HPC clusters (Slurm, PBS, SGE, LSF), commands are submitted through the \
scheduler rather than executed directly on the login node.\n\n\
EXAMPLES:\n  \
  oxo-call server add myserver --host login.hpc.edu --user alice --type hpc\n  \
  oxo-call server add workstation1 --host 10.0.0.5 --type workstation\n  \
  oxo-call server list\n  \
  oxo-call server status myserver\n  \
  oxo-call server ssh-config                   # import from ~/.ssh/config\n  \
  oxo-call server run myserver samtools 'sort input.bam by coordinate'\n  \
  oxo-call server dry-run myserver bwa 'align reads to reference'"
    )]
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },

    /// Manage named jobs — command shortcuts with full lifecycle management
    #[command(
        visible_alias = "j",
        alias = "cmd",
        long_about = "\
Manage your personal library of named jobs (command shortcuts).\n\n\
Store, organize, and execute frequently-used shell commands with full lifecycle\n\
support: descriptions, tags, cron scheduling, execution history, status\n\
tracking, and optional remote (SSH) execution.\n\n\
Use 'job generate' to let the LLM create a job from a plain-English description.\n\
Use 'job list --builtin' to browse the built-in easy-to-remember job templates.\n\n\
EXAMPLES:\n  \
  oxo-call job add gpu-check 'nvidia-smi' --description 'Check GPU status'\n  \
  oxo-call job add squeue-me 'squeue -u $USER' --tag hpc --tag slurm\n  \
  oxo-call job list\n  \
  oxo-call job list --tag hpc\n  \
  oxo-call job list --builtin\n  \
  oxo-call job show squeue-me\n  \
  oxo-call job run squeue-me\n  \
  oxo-call job run gpu-check --server mycluster\n  \
  oxo-call job status squeue-me\n  \
  oxo-call job history squeue-me\n  \
  oxo-call job schedule squeue-me '*/5 * * * *'\n  \
  oxo-call job generate 'check disk usage and alert if over 90%'\n  \
  oxo-call job edit squeue-me --command 'squeue -u $USER -o \"%.18i %.9P %.8j %.8u %.2t %.10M\"'\n  \
  oxo-call job remove gpu-check"
    )]
    Job {
        #[command(subcommand)]
        command: JobCommands,
    },

    /// Interactive chat with AI about bioinformatics tools
    #[command(
        visible_alias = "c",
        long_about = "\
Chat with an AI assistant about bioinformatics tools and concepts.\n\n\
Two modes are available:\n\n\
1. Single-shot Q&A (non-interactive):\n   \
  oxo-call chat samtools 'How do I sort a BAM file?'\n   \
  oxo-call chat bwa 'What is the difference between mem and aln?'\n\n\
2. Interactive multi-turn chat:\n   \
  oxo-call chat -i\n   \
  oxo-call chat -i --tool samtools  # pre-set tool context\n\n\
Scenarios control what context is injected:\n  \
  --scenario bare   : plain chat (no prompt/docs/skill)\n  \
  --scenario prompt : oxo-call system prompt only\n  \
  --scenario skill  : load skill file only\n  \
  --scenario doc    : load tool documentation only\n  \
  --scenario full   : load everything (default)\n\n\
EXAMPLES:\n  \
  oxo-call chat samtools 'Explain the difference between SAM and BAM'\n  \
  oxo-call chat --scenario skill bwa 'What are common pitfalls?'\n  \
  oxo-call chat -i\n  \
  oxo-call chat -i --tool gatk\n  \
  oxo-call chat --model gpt-4 samtools 'How to extract unmapped reads?'"
    )]
    Chat {
        /// Tool name to discuss (optional in interactive mode)
        #[arg(required = false)]
        tool: Option<String>,
        /// Question to ask (optional in interactive mode)
        #[arg(required = false)]
        question: Option<String>,
        /// Start interactive chat session
        #[arg(short, long)]
        interactive: bool,
        /// Override the LLM model for this invocation
        #[arg(short, long, value_name = "MODEL")]
        model: Option<String>,
        /// Skip cached documentation and fetch fresh --help output
        #[arg(long)]
        no_cache: bool,
        /// Scenario mode: bare, prompt, skill, doc, full (default: full)
        #[arg(long, value_enum, default_value = "full")]
        scenario: ChatScenario,
        /// Output result as JSON (non-interactive mode only)
        #[arg(long)]
        json: bool,
    },

    /// Generate shell completion scripts
    #[command(long_about = "\
Generate shell completion scripts for oxo-call.\n\n\
SETUP (Zsh):\n  \
  mkdir -p ~/.zfunc\n  \
  oxo-call completion zsh > ~/.zfunc/_oxo-call\n  \
  # Add to ~/.zshrc (before compinit):\n  \
  #   fpath=(~/.zfunc $fpath)\n  \
  #   autoload -Uz compinit && compinit\n\n\
SETUP (Bash):\n  \
  oxo-call completion bash > ~/.local/share/bash-completion/completions/oxo-call\n\n\
SETUP (Fish):\n  \
  oxo-call completion fish > ~/.config/fish/completions/oxo-call.fish\n\n\
EXAMPLES:\n  \
  oxo-call completion bash > ~/.local/share/bash-completion/completions/oxo-call\n  \
  oxo-call completion zsh > ~/.zfunc/_oxo-call\n  \
  oxo-call completion fish > ~/.config/fish/completions/oxo-call.fish\n  \
  oxo-call completion powershell > oxo-call.ps1")]
    Completion {
        /// Shell to generate completions for
        shell: ShellType,
    },
}

#[derive(Subcommand, Debug)]
pub enum DocsCommands {
    /// Add (or re-index) a tool's documentation from any combination of sources
    Add {
        /// Tool name (must be in PATH unless --url/--file/--dir is provided)
        tool: String,
        /// Remote documentation URL to include (http:// or https://)
        #[arg(long)]
        url: Option<String>,
        /// Local documentation file to include (.md, .txt, .rst, .html)
        #[arg(long, value_name = "PATH")]
        file: Option<PathBuf>,
        /// Local directory containing documentation files to include
        #[arg(long, value_name = "DIR")]
        dir: Option<PathBuf>,
    },
    /// Remove a tool's cached documentation
    Remove {
        /// Tool name to remove
        tool: String,
    },
    /// Update (re-index) documentation for a tool or all indexed tools
    Update {
        /// Tool name to update, or omit to update all indexed tools
        tool: Option<String>,
        /// Optional remote documentation URL
        #[arg(long)]
        url: Option<String>,
    },
    /// List all indexed tools
    List,
    /// Show the cached documentation for a tool
    Show {
        /// Tool name
        tool: String,
    },
    /// Fetch and cache documentation for a tool from a URL (alias for 'add --url')
    Fetch {
        /// Tool name
        tool: String,
        /// Remote documentation URL
        url: String,
    },
    /// Show the path where documentation is cached
    Path {
        /// Tool name
        tool: String,
    },
}

/// Deprecated index subcommands — these now mirror the 'docs' equivalents.
#[derive(Subcommand, Debug)]
pub enum IndexCommands {
    /// Add a tool to the documentation index (use 'docs add' instead)
    Add {
        /// Tool name (must be in PATH, or --url must be provided)
        tool: String,
        /// Optional remote documentation URL to include
        #[arg(long)]
        url: Option<String>,
        /// Local documentation file to include (.md, .txt, .rst, .html)
        #[arg(long, value_name = "PATH")]
        file: Option<PathBuf>,
        /// Local directory containing documentation files
        #[arg(long, value_name = "DIR")]
        dir: Option<PathBuf>,
    },
    /// Remove a tool from the documentation index (use 'docs remove' instead)
    Remove {
        /// Tool name to remove
        tool: String,
    },
    /// Update (re-index) documentation for a tool (use 'docs update' instead)
    Update {
        /// Tool name to update, or omit to update all
        tool: Option<String>,
        /// Optional remote documentation URL
        #[arg(long)]
        url: Option<String>,
    },
    /// List all indexed tools (use 'docs list' instead)
    List,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Set a configuration key to a value
    Set {
        /// Configuration key (e.g. llm.provider, llm.api_token, llm.model)
        key: String,
        /// Value to set
        value: String,
    },
    /// Get the current value of a configuration key
    Get {
        /// Configuration key
        key: String,
    },
    /// Show all current configuration
    Show,
    /// Verify the effective LLM configuration with a real API call
    Verify {
        /// Show the raw API response body when an error occurs
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show the path to the configuration file
    Path,
    /// Authenticate with a provider using an interactive login flow.
    ///
    /// For `github-copilot` this uses the OAuth 2.0 device-authorization flow
    /// with the Copilot CLI's GitHub App, which produces tokens compatible with
    /// the Copilot API. On success the token and provider are saved to config.
    ///
    /// Examples:
    ///   oxo-call config login
    ///   oxo-call config login --provider github-copilot
    Login {
        /// Provider to authenticate with (default: github-copilot)
        #[arg(long, default_value = "github-copilot")]
        provider: String,
    },
    /// Manage the configured model list and switch the active model.
    ///
    /// Models are stored in config and can be switched instantly without re-running login.
    /// The active model (used for all LLM calls) is displayed with a ★ marker.
    ///
    /// Examples:
    ///   oxo-call config model list
    ///   oxo-call config model add gpt-4.1
    ///   oxo-call config model use gpt-5-mini
    ///   oxo-call config model remove gpt-4.1
    #[command(visible_alias = "models")]
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },
}

/// Subcommands for `config model`
#[derive(Subcommand, Debug)]
pub enum ModelCommands {
    /// List all configured models (active model is marked with ★)
    #[command(visible_alias = "ls")]
    List,
    /// Add a model ID to the configured model list
    Add {
        /// Model ID to add (e.g. gpt-5-mini, gpt-4.1, o3-mini)
        model: String,
    },
    /// Remove a model ID from the configured model list
    #[command(visible_alias = "rm")]
    Remove {
        /// Model ID to remove
        model: String,
    },
    /// Switch the active model (sets llm.model in config)
    ///
    /// The model does not need to be in the configured list — any valid model ID is accepted.
    ///
    /// Examples:
    ///   oxo-call config model use gpt-5-mini
    ///   oxo-call config model use gpt-4.1
    #[command(visible_alias = "switch")]
    Use {
        /// Model ID to activate
        model: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum HistoryCommands {
    /// List recent command history
    List {
        /// Number of entries to show (default: 20)
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
        /// Filter by tool name
        #[arg(long)]
        tool: Option<String>,
    },
    /// Clear all history
    Clear {
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    /// List all available skills (built-in, community, MCP, and user-defined)
    List,
    /// Show the content of a skill
    Show {
        /// Tool name
        tool: String,
    },
    /// Install a skill from the community registry or a custom URL
    Install {
        /// Tool name
        tool: String,
        /// Custom URL to a skill file (optional; defaults to community registry).
        /// Accepts `.md` (YAML front-matter + Markdown, preferred) and legacy `.toml`.
        #[arg(long)]
        url: Option<String>,
    },
    /// Remove a community or user-installed skill
    Remove {
        /// Tool name
        tool: String,
    },
    /// Generate a skill template for a new tool
    Create {
        /// Tool name
        tool: String,
        /// Write template to this file (defaults to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        /// Use LLM to generate a pre-filled skill template (requires LLM config)
        #[arg(long)]
        llm: bool,
    },
    /// Verify a skill file for format correctness and quality (structural + LLM review)
    Verify {
        /// Tool name
        tool: String,
        /// Skip the LLM review and only run structural validation
        #[arg(long)]
        no_llm: bool,
    },
    /// Polish a user or community skill using LLM to improve quality and fix format issues
    Polish {
        /// Tool name
        tool: String,
        /// Write the polished skill to this file instead of updating in-place
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    /// Show the path to the user skills directory
    Path,
    /// Manage MCP skill provider servers
    #[command(visible_alias = "mcp")]
    McpServer {
        #[command(subcommand)]
        command: SkillMcpCommands,
    },
}

/// Subcommands for `skill mcp`
#[derive(Subcommand, Debug)]
pub enum SkillMcpCommands {
    /// Register an MCP skill provider server
    Add {
        /// MCP server base URL (e.g. http://localhost:3000)
        url: String,
        /// Human-readable label for this server (defaults to the URL)
        #[arg(long)]
        name: Option<String>,
        /// Bearer token for authenticated servers
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Unregister an MCP skill provider server by URL or name
    Remove {
        /// URL or name of the server to remove
        url_or_name: String,
    },
    /// List all registered MCP skill provider servers
    List,
    /// Test connectivity to all registered MCP servers
    Ping,
}

/// Subcommands for `oxo-call job`
#[derive(Subcommand, Debug)]
pub enum JobCommands {
    /// Add a new named job to your library
    #[command(visible_alias = "a")]
    Add {
        /// Short name used to invoke this job (must be unique)
        name: String,
        /// The shell command to save
        command: String,
        /// Brief description of what this job does
        #[arg(short, long)]
        description: Option<String>,
        /// Tags for organizing jobs (repeatable: --tag hpc --tag slurm)
        #[arg(short, long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Cron expression for scheduled execution (e.g. "0 * * * *")
        #[arg(long)]
        schedule: Option<String>,
    },

    /// Remove a job from your library
    #[command(visible_alias = "rm")]
    Remove {
        /// Name of the job to remove
        name: String,
    },

    /// List saved jobs
    #[command(visible_alias = "ls")]
    List {
        /// Only show jobs with this tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Show built-in job templates instead of user-defined jobs
        #[arg(long)]
        builtin: bool,
    },

    /// Show full details of a saved job
    Show {
        /// Name of the job to show
        name: String,
    },

    /// Run a saved job locally or on a remote server
    #[command(visible_alias = "r")]
    Run {
        /// Name of the job to run
        name: String,
        /// Run on this registered remote server (via SSH)
        #[arg(short, long)]
        server: Option<String>,
        /// Print the command without executing it
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// Set a named variable for `{KEY}` substitution in the command (repeatable)
        ///
        /// Example: --var THREADS=8 --var REF=hg38.fa
        #[arg(short = 'V', long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        /// Read input items from a file (one per line); expands `{item}` in the command
        ///
        /// Blank lines and lines starting with `#` are ignored.
        #[arg(short = 'i', long = "input-list", value_name = "FILE")]
        input_list: Option<String>,
        /// Comma-separated input items; expands `{item}` in the command
        ///
        /// Example: --input-items sample1.bam,sample2.bam,sample3.bam
        #[arg(long = "input-items", value_name = "ITEMS")]
        input_items: Option<String>,
        /// Number of parallel jobs (default: 1 = sequential)
        ///
        /// Example: -j 4  runs up to 4 jobs concurrently
        #[arg(short = 'j', long = "jobs", default_value = "1", value_name = "N")]
        jobs: usize,
        /// Preserve the order of output when running in parallel
        #[arg(short = 'k', long = "keep-order")]
        keep_order: bool,
        /// Stop after the first failed item instead of continuing the batch
        #[arg(short = 'x', long = "stop-on-error")]
        stop_on_error: bool,
    },

    /// Edit an existing job entry
    #[command(visible_alias = "e")]
    Edit {
        /// Name of the job to edit
        name: String,
        /// Replace the command string
        #[arg(short, long)]
        command: Option<String>,
        /// Replace the description
        #[arg(short, long)]
        description: Option<String>,
        /// Replace all tags (repeatable: --tag hpc --tag slurm)
        #[arg(short, long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Set a new cron schedule (e.g. "0 * * * *")
        #[arg(long)]
        schedule: Option<String>,
        /// Remove the cron schedule
        #[arg(long)]
        clear_schedule: bool,
    },

    /// Rename a job
    Rename {
        /// Current name
        from: String,
        /// New name
        to: String,
    },

    /// Show recent execution status for a job (or all jobs)
    Status {
        /// Name of the job (omit to show status for all jobs)
        name: Option<String>,
    },

    /// Show execution history for a job
    History {
        /// Name of the job (omit to show history for all jobs)
        name: Option<String>,
        /// Number of most-recent runs to show (default: 10)
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },

    /// Set (or clear) a cron schedule on a job
    Schedule {
        /// Name of the job
        name: String,
        /// Cron expression, e.g. "0 * * * *" (5-field standard cron)
        /// Omit to clear the current schedule.
        cron: Option<String>,
    },

    /// Generate a job from a plain-English description using the LLM
    Generate {
        /// Natural-language description of what the job should do
        description: String,
        /// Save the generated job with this name (defaults to a slugified form of the description)
        #[arg(short, long)]
        name: Option<String>,
        /// Tags to assign to the generated job (repeatable)
        #[arg(short, long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Skip saving and only print the generated command
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Copy a built-in job template into your personal library
    Import {
        /// Name of the built-in template to import (omit with --all to import every template)
        name: Option<String>,
        /// Override the name used in your library (defaults to the template name)
        #[arg(long)]
        as_name: Option<String>,
        /// Import every built-in template at once (skips templates that already exist)
        #[arg(long, conflicts_with_all = ["name", "as_name"])]
        all: bool,
    },
}

/// Chat scenario modes for controlling context injection
#[derive(Clone, Debug, ValueEnum)]
pub enum ChatScenario {
    /// Bare: no system prompt, no docs, no skill (plain chat)
    Bare,
    /// Prompt: use oxo-call system prompt only
    Prompt,
    /// Skill: load skill file only
    Skill,
    /// Doc: load tool documentation only
    Doc,
    /// Full: load everything (default)
    Full,
}

/// Supported shell types for completion generation
#[derive(Clone, Debug, ValueEnum)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    Powershell,
    /// Elvish shell
    Elvish,
}

#[derive(Subcommand, Debug)]
pub enum LicenseCommands {
    /// Verify the license file and display its details
    Verify,
}

#[derive(Subcommand, Debug)]
pub enum WorkflowCommands {
    /// Run a workflow file with the oxo-call native engine
    #[command(name = "run", visible_alias = "r")]
    RunWorkflow {
        /// Path to an .oxo.toml workflow file (or a built-in template name)
        file: String,
        /// After all steps complete, ask the LLM to verify outputs: checks
        /// expected output files and provides feedback and suggestions
        #[arg(long)]
        verify: bool,
    },

    /// Preview a workflow without executing any steps (dry-run)
    #[command(name = "dry-run", visible_alias = "d")]
    DryRunWorkflow {
        /// Path to an .oxo.toml workflow file (or a built-in template name)
        file: String,
    },

    /// Export a native .oxo.toml workflow to Snakemake or Nextflow format
    Export {
        /// Path to an .oxo.toml workflow file (or a built-in template name)
        file: String,
        /// Target format: snakemake or nextflow
        #[arg(long, default_value = "snakemake", value_parser = ["snakemake", "nextflow"])]
        to: String,
        /// Write output to this file (defaults to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Generate a workflow with LLM from a natural-language description
    Generate {
        /// Plain-English description of the bioinformatics workflow
        task: String,
        /// Output format: native (default), snakemake, or nextflow
        #[arg(short, long, default_value = "native", value_parser = ["native", "snakemake", "nextflow"])]
        engine: String,
        /// Write the generated workflow to this file (defaults to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Infer and generate a workflow from a task description and input data directory
    ///
    /// Scans the data directory to discover sample names and file patterns, then
    /// uses the LLM to generate a workflow with real paths and sample names already filled in.
    Infer {
        /// Plain-English description of the analysis task
        task: String,
        /// Path to the directory containing input data files (FASTQ, BAM, etc.)
        #[arg(short, long, value_name = "DIR")]
        data: std::path::PathBuf,
        /// Output format: native (default), snakemake, or nextflow
        #[arg(short, long, default_value = "native", value_parser = ["native", "snakemake", "nextflow"])]
        engine: String,
        /// Write the generated workflow to this file (defaults to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        /// After generating, immediately run the workflow (only with --output)
        #[arg(long)]
        run: bool,
    },

    /// List built-in workflow templates
    List,

    /// Show a built-in workflow template
    Show {
        /// Template name (see 'workflow list')
        name: String,
        /// Format to show: native (default), snakemake, or nextflow
        #[arg(short, long, default_value = "native", value_parser = ["native", "snakemake", "nextflow"])]
        engine: String,
    },

    /// Verify a .oxo.toml workflow file for correctness (parse, dependency, cycle checks)
    #[command(visible_alias = "check")]
    Verify {
        /// Path to an .oxo.toml workflow file (or a built-in template name)
        file: String,
    },

    /// Auto-format a .oxo.toml workflow file to canonical style
    #[command(name = "fmt", visible_alias = "format")]
    Format {
        /// Path to an .oxo.toml workflow file (or a built-in template name)
        file: String,
        /// Write formatted output to stdout instead of overwriting the file
        #[arg(long)]
        stdout: bool,
    },

    /// Visualize the workflow DAG as a phase diagram
    #[command(visible_alias = "dag")]
    Vis {
        /// Path to an .oxo.toml workflow file (or a built-in template name)
        file: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    /// Register a remote server (workstation or HPC cluster login node)
    Add {
        /// A short name for this server (used in subsequent commands)
        name: String,
        /// SSH hostname or IP address
        #[arg(long)]
        host: String,
        /// SSH username (defaults to current user)
        #[arg(long)]
        user: Option<String>,
        /// SSH port (defaults to 22)
        #[arg(long)]
        port: Option<u16>,
        /// Path to SSH identity (private key) file
        #[arg(long, value_name = "PATH")]
        identity_file: Option<String>,
        /// Server type: 'workstation' for direct execution, 'hpc' for cluster login nodes
        #[arg(long = "type", default_value = "workstation", value_parser = ["workstation", "hpc", "ws", "cluster"])]
        server_type: String,
        /// Job scheduler (slurm, pbs, sge, lsf, htcondor) — auto-detected for HPC nodes
        #[arg(long)]
        scheduler: Option<String>,
        /// Default working directory on the remote host
        #[arg(long, value_name = "DIR")]
        work_dir: Option<String>,
    },

    /// Remove a registered server
    Remove {
        /// Server name to remove
        name: String,
    },

    /// List all registered servers
    List,

    /// Check SSH connectivity to a registered server
    Status {
        /// Server name to check
        name: String,
    },

    /// Import hosts from ~/.ssh/config as registered servers
    SshConfig {
        /// Import all hosts without interactive selection
        #[arg(long, short = 'y')]
        yes: bool,
        /// Default server type for all imported hosts
        #[arg(
            long = "type",
            value_name = "TYPE",
            default_value = "workstation",
            value_parser = ["workstation", "hpc", "ws", "cluster"]
        )]
        server_type: String,
    },

    /// Set a server as the active (default) server
    ///
    /// Once set, `server run` and `server dry-run` will use this server when
    /// no `--server` flag is provided.
    #[command(name = "use")]
    Use {
        /// Server name to make active
        name: String,
    },

    /// Clear the active (default) server
    Unuse,

    /// Run a tool on a remote server with LLM-generated parameters
    #[command(visible_alias = "r")]
    Run {
        /// The tool to run
        tool: String,
        /// Natural-language description of the task
        task: String,
        /// Server to run on (uses the active server if not specified)
        #[arg(long, short = 's', value_name = "SERVER")]
        server: Option<String>,
        /// Override the LLM model for this invocation
        #[arg(short, long, value_name = "MODEL")]
        model: Option<String>,
        /// Skip cached documentation and fetch fresh --help output
        #[arg(long)]
        no_cache: bool,
        /// Output result as JSON (useful for scripting and CI integration)
        #[arg(long)]
        json: bool,
        /// After execution, ask the LLM to verify results: checks output files,
        /// stderr patterns, and exit code, then reports issues and suggestions
        #[arg(long)]
        verify: bool,
    },

    /// Preview a command for a remote server (no execution)
    #[command(name = "dry-run", visible_alias = "d")]
    DryRun {
        /// The tool to preview
        tool: String,
        /// Natural-language description of the task
        task: String,
        /// Server to target (uses the active server if not specified)
        #[arg(long, short = 's', value_name = "SERVER")]
        server: Option<String>,
        /// Override the LLM model for this invocation
        #[arg(short, long, value_name = "MODEL")]
        model: Option<String>,
        /// Skip cached documentation and fetch fresh --help output
        #[arg(long)]
        no_cache: bool,
        /// Output result as JSON (useful for scripting and CI integration)
        #[arg(long)]
        json: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_defaults_to_non_interactive_execution() {
        let cli = Cli::parse_from(["oxo-call", "run", "date", "current time"]);

        match cli.command {
            Commands::Run { ask, .. } => assert!(!ask),
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn test_run_supports_explicit_ask_flag() {
        let cli = Cli::parse_from(["oxo-call", "run", "--ask", "date", "current time"]);

        match cli.command {
            Commands::Run { ask, .. } => assert!(ask),
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn test_run_supports_model_override() {
        let cli = Cli::parse_from([
            "oxo-call", "run", "--model", "gpt-4", "samtools", "sort bam",
        ]);

        match cli.command {
            Commands::Run { model, tool, .. } => {
                assert_eq!(model.as_deref(), Some("gpt-4"));
                assert_eq!(tool, "samtools");
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn test_run_supports_no_cache_flag() {
        let cli = Cli::parse_from(["oxo-call", "run", "--no-cache", "samtools", "sort bam"]);

        match cli.command {
            Commands::Run { no_cache, .. } => assert!(no_cache),
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn test_run_supports_json_flag() {
        let cli = Cli::parse_from(["oxo-call", "run", "--json", "samtools", "sort bam"]);

        match cli.command {
            Commands::Run { json, .. } => assert!(json),
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn test_dry_run_supports_model_override() {
        let cli = Cli::parse_from([
            "oxo-call",
            "dry-run",
            "--model",
            "claude-3-5-sonnet-20241022",
            "bwa",
            "align reads",
        ]);

        match cli.command {
            Commands::DryRun { model, tool, .. } => {
                assert_eq!(model.as_deref(), Some("claude-3-5-sonnet-20241022"));
                assert_eq!(tool, "bwa");
            }
            _ => panic!("expected dry-run command"),
        }
    }

    #[test]
    fn test_dry_run_supports_json_flag() {
        let cli = Cli::parse_from(["oxo-call", "dry-run", "--json", "samtools", "sort bam"]);

        match cli.command {
            Commands::DryRun { json, .. } => assert!(json),
            _ => panic!("expected dry-run command"),
        }
    }

    #[test]
    fn test_verbose_flag_is_global() {
        let cli = Cli::parse_from(["oxo-call", "--verbose", "run", "date", "current time"]);
        assert!(cli.verbose);

        let cli = Cli::parse_from(["oxo-call", "run", "--verbose", "date", "current time"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_completion_command_accepts_shell_types() {
        let cli = Cli::parse_from(["oxo-call", "completion", "bash"]);
        match cli.command {
            Commands::Completion { shell } => {
                assert!(matches!(shell, ShellType::Bash));
            }
            _ => panic!("expected completion command"),
        }
    }

    #[test]
    fn test_server_add_parses_correctly() {
        let cli = Cli::parse_from([
            "oxo-call",
            "server",
            "add",
            "mycluster",
            "--host",
            "login.hpc.edu",
            "--user",
            "alice",
            "--type",
            "hpc",
            "--scheduler",
            "slurm",
        ]);

        match cli.command {
            Commands::Server {
                command:
                    ServerCommands::Add {
                        name,
                        host,
                        user,
                        server_type,
                        scheduler,
                        ..
                    },
            } => {
                assert_eq!(name, "mycluster");
                assert_eq!(host, "login.hpc.edu");
                assert_eq!(user.as_deref(), Some("alice"));
                assert_eq!(server_type, "hpc");
                assert_eq!(scheduler.as_deref(), Some("slurm"));
            }
            _ => panic!("expected server add command"),
        }
    }

    #[test]
    fn test_server_add_default_type() {
        let cli = Cli::parse_from(["oxo-call", "server", "add", "mybox", "--host", "10.0.0.5"]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Add { server_type, .. },
            } => {
                assert_eq!(server_type, "workstation");
            }
            _ => panic!("expected server add command"),
        }
    }

    #[test]
    fn test_server_dry_run_parses_with_server_flag() {
        let cli = Cli::parse_from([
            "oxo-call",
            "server",
            "dry-run",
            "samtools",
            "sort input.bam",
            "--server",
            "mycluster",
        ]);

        match cli.command {
            Commands::Server {
                command:
                    ServerCommands::DryRun {
                        server, tool, task, ..
                    },
            } => {
                assert_eq!(server.as_deref(), Some("mycluster"));
                assert_eq!(tool, "samtools");
                assert_eq!(task, "sort input.bam");
            }
            _ => panic!("expected server dry-run command"),
        }
    }

    #[test]
    fn test_server_dry_run_parses_without_server() {
        let cli = Cli::parse_from([
            "oxo-call",
            "server",
            "dry-run",
            "samtools",
            "sort input.bam",
        ]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::DryRun { server, .. },
            } => {
                assert!(server.is_none(), "server should be None when not specified");
            }
            _ => panic!("expected server dry-run command"),
        }
    }

    #[test]
    fn test_server_run_parses_with_server_flag() {
        let cli = Cli::parse_from([
            "oxo-call",
            "server",
            "run",
            "ls",
            "list home directory",
            "--server",
            "lab-wsx",
        ]);

        match cli.command {
            Commands::Server {
                command:
                    ServerCommands::Run {
                        server, tool, task, ..
                    },
            } => {
                assert_eq!(server.as_deref(), Some("lab-wsx"));
                assert_eq!(tool, "ls");
                assert_eq!(task, "list home directory");
            }
            _ => panic!("expected server run command"),
        }
    }

    #[test]
    fn test_server_run_parses_without_server() {
        let cli = Cli::parse_from(["oxo-call", "server", "run", "ls", "list home directory"]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Run { server, .. },
            } => {
                assert!(server.is_none(), "server should be None when not specified");
            }
            _ => panic!("expected server run command"),
        }
    }

    #[test]
    fn test_server_use_parses() {
        let cli = Cli::parse_from(["oxo-call", "server", "use", "lab-wsx"]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Use { name },
            } => {
                assert_eq!(name, "lab-wsx");
            }
            _ => panic!("expected server use command"),
        }
    }

    #[test]
    fn test_server_unuse_parses() {
        let cli = Cli::parse_from(["oxo-call", "server", "unuse"]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Unuse,
            } => {}
            _ => panic!("expected server unuse command"),
        }
    }

    #[test]
    fn test_server_run_supports_no_cache_flag() {
        let cli = Cli::parse_from([
            "oxo-call",
            "server",
            "run",
            "--no-cache",
            "ls",
            "list files",
        ]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Run { no_cache, .. },
            } => assert!(no_cache),
            _ => panic!("expected server run command"),
        }
    }

    #[test]
    fn test_server_run_supports_json_flag() {
        let cli = Cli::parse_from(["oxo-call", "server", "run", "--json", "ls", "list files"]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Run { json, .. },
            } => assert!(json),
            _ => panic!("expected server run command"),
        }
    }

    #[test]
    fn test_server_run_supports_verify_flag() {
        let cli = Cli::parse_from(["oxo-call", "server", "run", "--verify", "ls", "list files"]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::Run { verify, .. },
            } => assert!(verify),
            _ => panic!("expected server run command"),
        }
    }

    #[test]
    fn test_server_dry_run_supports_no_cache_flag() {
        let cli = Cli::parse_from([
            "oxo-call",
            "server",
            "dry-run",
            "--no-cache",
            "samtools",
            "sort bam",
        ]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::DryRun { no_cache, .. },
            } => assert!(no_cache),
            _ => panic!("expected server dry-run command"),
        }
    }

    #[test]
    fn test_server_dry_run_supports_json_flag() {
        let cli = Cli::parse_from([
            "oxo-call", "server", "dry-run", "--json", "samtools", "sort bam",
        ]);

        match cli.command {
            Commands::Server {
                command: ServerCommands::DryRun { json, .. },
            } => assert!(json),
            _ => panic!("expected server dry-run command"),
        }
    }
}
