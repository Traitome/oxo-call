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
  oxo-call run --optimize-task samtools 'sort bam'"
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
        /// Before generating the command, use the LLM to optimize and expand
        /// the task description for better accuracy
        #[arg(long)]
        optimize_task: bool,
    },

    /// Preview the command that would be executed (no actual execution)
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
  oxo-call dry-run --optimize-task samtools 'sort bam'"
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
        /// Before generating the command, use the LLM to optimize and expand
        /// the task description for better accuracy
        #[arg(long)]
        optimize_task: bool,
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

    /// Generate shell completion scripts
    #[command(long_about = "\
Generate shell completion scripts for oxo-call.\n\n\
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
    Verify,
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

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    /// List all available skills (built-in, community, and user-defined)
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
    },
    /// Show the path to the user skills directory
    Path,
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
}
