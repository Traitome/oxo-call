use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "oxo-call",
    version,
    about = "Model-intelligent orchestration for CLI bioinformatics",
    long_about = r#"oxo-call uses LLM intelligence to help you call bioinformatics tools
without memorizing every flag and parameter.

Quick start:
  1. Set up your API token:
       oxo-call config set llm.api_token <your-github-token>

  2. Build a documentation index for a tool:
       oxo-call index add bwa
       oxo-call index add samtools

  3. Run a tool with a natural-language task:
       oxo-call run samtools "sort input.bam by coordinate and output to sorted.bam"
       oxo-call dry-run bwa "align reads.fastq to reference.fa with 8 threads"

Supported LLM providers: github-copilot (default), openai, anthropic, ollama"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a bioinformatics tool with LLM-generated parameters
    #[command(visible_alias = "r")]
    Run {
        /// The tool to run (must be in PATH)
        tool: String,
        /// Natural-language description of the task
        task: String,
        /// Execute without confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Preview the command that would be executed (no actual execution)
    #[command(name = "dry-run", visible_alias = "d")]
    DryRun {
        /// The tool to preview
        tool: String,
        /// Natural-language description of the task
        task: String,
    },

    /// Manage the local documentation index
    #[command(visible_alias = "i")]
    Index {
        #[command(subcommand)]
        command: IndexCommands,
    },

    /// View or fetch documentation for a tool
    #[command(visible_alias = "doc")]
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
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
}

#[derive(Subcommand, Debug)]
pub enum IndexCommands {
    /// Add a tool to the documentation index (fetches --help and optionally a remote URL)
    Add {
        /// Tool name (must be in PATH, or --url must be provided)
        tool: String,
        /// Optional remote documentation URL to include
        #[arg(long)]
        url: Option<String>,
    },
    /// Remove a tool from the documentation index
    Remove {
        /// Tool name to remove
        tool: String,
    },
    /// Update (re-index) documentation for a tool
    Update {
        /// Tool name to update, or omit to update all
        tool: Option<String>,
        /// Optional remote documentation URL
        #[arg(long)]
        url: Option<String>,
    },
    /// List all indexed tools
    List,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommands {
    /// Show the cached documentation for a tool
    Show {
        /// Tool name
        tool: String,
    },
    /// Fetch and cache documentation for a tool from a URL
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
    /// Show the path to the configuration file
    Path,
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
