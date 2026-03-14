/// Remote server management for oxo-call.
///
/// The **server** module provides SSH-based remote server management,
/// allowing users to register, connect to, and execute commands on
/// remote servers — including both standalone workstations and HPC
/// cluster login nodes.
///
/// # Server types
///
/// - **Workstation** — commands are executed directly on the host.
/// - **HPC cluster** — the host is a login/management node.  Commands
///   should be submitted through the cluster scheduler (Slurm, PBS, etc.)
///   rather than executed directly, and the module issues warnings when
///   direct execution is requested on a login node.
///
/// # SSH configuration
///
/// Servers are stored in the oxo-call config file under `[[server.hosts]]`.
/// The module can also read `~/.ssh/config` to discover pre-configured hosts.
use crate::config;
use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The type of remote server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    /// Standalone workstation — commands run directly.
    Workstation,
    /// HPC cluster login node — commands should be submitted via a scheduler.
    Hpc,
}

impl std::fmt::Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerType::Workstation => write!(f, "workstation"),
            ServerType::Hpc => write!(f, "hpc"),
        }
    }
}

impl std::str::FromStr for ServerType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "workstation" | "ws" => Ok(ServerType::Workstation),
            "hpc" | "cluster" => Ok(ServerType::Hpc),
            _ => Err(format!(
                "Unknown server type: '{s}'. Use 'workstation' or 'hpc'."
            )),
        }
    }
}

/// A registered remote server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHost {
    /// User-defined name or SSH config alias.
    pub name: String,
    /// SSH hostname or IP address.
    pub host: String,
    /// SSH username.
    pub user: Option<String>,
    /// SSH port (defaults to 22).
    pub port: Option<u16>,
    /// Path to the SSH identity (private key) file.
    pub identity_file: Option<String>,
    /// Server type: workstation or hpc.
    pub server_type: ServerType,
    /// Scheduler type for HPC nodes (e.g. "slurm", "pbs", "sge", "lsf").
    pub scheduler: Option<String>,
    /// Default working directory on the remote host.
    pub work_dir: Option<String>,
}

impl ServerHost {
    /// Build an SSH destination string like `user@host` or `host`.
    pub fn ssh_dest(&self) -> String {
        match &self.user {
            Some(u) => format!("{u}@{}", self.host),
            None => self.host.clone(),
        }
    }

    /// Build an ssh command argument list for this host.
    pub fn ssh_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(port) = self.port {
            args.push("-p".to_string());
            args.push(port.to_string());
        }
        if let Some(ref id) = self.identity_file {
            args.push("-i".to_string());
            args.push(id.clone());
        }
        args.push(self.ssh_dest());
        args
    }
}

/// Server configuration section stored in config.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default)]
    pub hosts: Vec<ServerHost>,
}

// ─── SSH config parser ────────────────────────────────────────────────────────

/// A simplified entry from `~/.ssh/config`.
#[derive(Debug, Clone)]
pub struct SshConfigEntry {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
}

/// Parse `~/.ssh/config` and return all `Host` entries (excluding wildcards).
pub fn parse_ssh_config() -> Vec<SshConfigEntry> {
    let ssh_config_path = dirs_ssh_config();
    if !ssh_config_path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&ssh_config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    parse_ssh_config_content(&content)
}

fn dirs_ssh_config() -> PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    {
        directories::BaseDirs::new()
            .map(|d| d.home_dir().join(".ssh").join("config"))
            .unwrap_or_else(|| PathBuf::from("~/.ssh/config"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from("~/.ssh/config")
    }
}

fn is_concrete_alias(alias: &str) -> bool {
    !alias.contains('*') && !alias.contains('?')
}

fn parse_ssh_config_content(content: &str) -> Vec<SshConfigEntry> {
    let mut entries = Vec::new();
    let mut current: Option<SshConfigEntry> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split on first whitespace or '='
        let (key, value) = match line.split_once(|c: char| c.is_whitespace() || c == '=') {
            Some((k, v)) => (k.trim().to_lowercase(), v.trim().to_string()),
            None => continue,
        };

        match key.as_str() {
            "host" => {
                // Save the previous entry
                if let Some(entry) = current.take()
                    && is_concrete_alias(&entry.alias)
                {
                    entries.push(entry);
                }
                current = Some(SshConfigEntry {
                    alias: value.clone(),
                    hostname: None,
                    user: None,
                    port: None,
                    identity_file: None,
                });
            }
            "hostname" => {
                if let Some(ref mut entry) = current {
                    entry.hostname = Some(value);
                }
            }
            "user" => {
                if let Some(ref mut entry) = current {
                    entry.user = Some(value);
                }
            }
            "port" => {
                if let Some(ref mut entry) = current {
                    entry.port = value.parse().ok();
                }
            }
            "identityfile" => {
                if let Some(ref mut entry) = current {
                    entry.identity_file = Some(value);
                }
            }
            _ => {}
        }
    }

    // Don't forget the last entry
    if let Some(entry) = current
        && is_concrete_alias(&entry.alias)
    {
        entries.push(entry);
    }

    entries
}

// ─── Server manager ──────────────────────────────────────────────────────────

pub struct ServerManager {
    config: config::Config,
}

impl ServerManager {
    pub fn new(config: config::Config) -> Self {
        Self { config }
    }

    /// Return all registered server hosts.
    pub fn list(&self) -> &[ServerHost] {
        &self.config.server.hosts
    }

    /// Find a server by name.
    pub fn find(&self, name: &str) -> Option<&ServerHost> {
        self.config.server.hosts.iter().find(|h| h.name == name)
    }

    /// Add (register) a new server host.
    pub fn add(&mut self, host: ServerHost) -> Result<()> {
        if self.config.server.hosts.iter().any(|h| h.name == host.name) {
            return Err(OxoError::ConfigError(format!(
                "Server '{}' is already registered. Use 'server remove' first.",
                host.name
            )));
        }
        self.config.server.hosts.push(host);
        self.config.save()?;
        Ok(())
    }

    /// Remove a server by name.
    pub fn remove(&mut self, name: &str) -> Result<()> {
        let before = self.config.server.hosts.len();
        self.config.server.hosts.retain(|h| h.name != name);
        if self.config.server.hosts.len() == before {
            return Err(OxoError::ConfigError(format!(
                "No server found with name '{name}'"
            )));
        }
        self.config.save()?;
        Ok(())
    }

    /// Check SSH connectivity to a server.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn check_connection(&self, server: &ServerHost) -> Result<bool> {
        let mut cmd = std::process::Command::new("ssh");
        for arg in &server.ssh_args() {
            cmd.arg(arg);
        }
        // Quick connectivity test with timeout
        cmd.args([
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            "echo",
            "oxo-call-connected",
        ]);

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(output.status.success() && stdout.contains("oxo-call-connected"))
            }
            Err(_) => Ok(false),
        }
    }

    /// Detect the scheduler on an HPC server by checking for common commands.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn detect_scheduler(&self, server: &ServerHost) -> Option<String> {
        let schedulers = [
            ("slurm", "sinfo --version"),
            ("pbs", "qstat --version"),
            ("sge", "qhost -help"),
            ("lsf", "bsub -V"),
            ("htcondor", "condor_version"),
        ];

        for (name, check_cmd) in &schedulers {
            let mut cmd = std::process::Command::new("ssh");
            for arg in &server.ssh_args() {
                cmd.arg(arg);
            }
            cmd.args(["-o", "ConnectTimeout=5", "-o", "BatchMode=yes", check_cmd]);

            if let Ok(output) = cmd.output()
                && output.status.success()
            {
                return Some(name.to_string());
            }
        }
        None
    }

    /// Check if a command looks like a compute-intensive command that should
    /// NOT be run on an HPC login node.
    pub fn is_compute_command(cmd: &str) -> bool {
        let compute_patterns = [
            "samtools",
            "bwa",
            "bowtie2",
            "hisat2",
            "star",
            "salmon",
            "kallisto",
            "fastp",
            "fastqc",
            "gatk",
            "bcftools",
            "deepvariant",
            "cellranger",
            "minimap2",
            "kraken2",
            "diamond",
            "blast",
            "spades",
            "megahit",
            "flye",
            "canu",
            "python",
            "Rscript",
            "julia",
            "matlab",
            "make",
            "cmake",
            "gcc",
            "g++",
        ];
        let cmd_lower = cmd.to_lowercase();
        compute_patterns
            .iter()
            .any(|p| cmd_lower.starts_with(p) || cmd_lower.contains(&format!("/{p}")))
    }

    /// Return commands that are safe to run on login nodes.
    #[allow(dead_code)]
    pub fn is_login_safe_command(cmd: &str) -> bool {
        let safe_patterns = [
            "ls",
            "pwd",
            "cd",
            "cat",
            "head",
            "tail",
            "wc",
            "du",
            "df",
            "echo",
            "which",
            "whoami",
            "hostname",
            "module",
            "conda",
            "pip",
            "sinfo",
            "squeue",
            "sacct",
            "sbatch",
            "scancel",
            "qstat",
            "qsub",
            "qdel",
            "pbsnodes",
            "bjobs",
            "bsub",
            "bkill",
            "bqueues",
            "condor_q",
            "condor_submit",
            "qhost",
            "qconf",
            "scontrol",
            "sacctmgr",
            "sshare",
        ];
        let first_word = cmd.split_whitespace().next().unwrap_or("");
        let cmd_name = first_word.rsplit('/').next().unwrap_or(first_word);
        safe_patterns.contains(&cmd_name)
    }
}

// ─── Selection helper ─────────────────────────────────────────────────────────

/// Parse a user selection string into a sorted, deduplicated list of 0-based
/// indices.  Input may be:
/// - `"all"` or `"a"` — selects every item
/// - Comma-separated 1-based numbers and/or inclusive ranges, e.g. `"1,3,5-7"`
///
/// Numbers outside `[1, len]` are silently ignored.
pub fn parse_selection(input: &str, len: usize) -> Vec<usize> {
    let input = input.trim().to_lowercase();
    if input == "all" || input == "a" {
        return (0..len).collect();
    }
    let mut indices = std::collections::BTreeSet::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            if let (Ok(s), Ok(e)) = (start.trim().parse::<usize>(), end.trim().parse::<usize>()) {
                for i in s..=e {
                    if i >= 1 && i <= len {
                        indices.insert(i - 1);
                    }
                }
            }
        } else if let Ok(n) = part.parse::<usize>()
            && n >= 1
            && n <= len
        {
            indices.insert(n - 1);
        }
    }
    indices.into_iter().collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_type_display() {
        assert_eq!(ServerType::Workstation.to_string(), "workstation");
        assert_eq!(ServerType::Hpc.to_string(), "hpc");
    }

    #[test]
    fn test_server_type_from_str() {
        assert_eq!(
            "workstation".parse::<ServerType>().unwrap(),
            ServerType::Workstation
        );
        assert_eq!("hpc".parse::<ServerType>().unwrap(), ServerType::Hpc);
        assert_eq!("cluster".parse::<ServerType>().unwrap(), ServerType::Hpc);
        assert_eq!("ws".parse::<ServerType>().unwrap(), ServerType::Workstation);
        assert!("unknown".parse::<ServerType>().is_err());
    }

    #[test]
    fn test_ssh_dest_with_user() {
        let host = ServerHost {
            name: "test".to_string(),
            host: "example.com".to_string(),
            user: Some("alice".to_string()),
            port: None,
            identity_file: None,
            server_type: ServerType::Workstation,
            scheduler: None,
            work_dir: None,
        };
        assert_eq!(host.ssh_dest(), "alice@example.com");
    }

    #[test]
    fn test_ssh_dest_without_user() {
        let host = ServerHost {
            name: "test".to_string(),
            host: "example.com".to_string(),
            user: None,
            port: None,
            identity_file: None,
            server_type: ServerType::Workstation,
            scheduler: None,
            work_dir: None,
        };
        assert_eq!(host.ssh_dest(), "example.com");
    }

    #[test]
    fn test_ssh_args_with_port_and_key() {
        let host = ServerHost {
            name: "test".to_string(),
            host: "10.0.0.1".to_string(),
            user: Some("bob".to_string()),
            port: Some(2222),
            identity_file: Some("/home/bob/.ssh/id_ed25519".to_string()),
            server_type: ServerType::Hpc,
            scheduler: Some("slurm".to_string()),
            work_dir: None,
        };
        let args = host.ssh_args();
        assert_eq!(
            args,
            vec![
                "-p",
                "2222",
                "-i",
                "/home/bob/.ssh/id_ed25519",
                "bob@10.0.0.1"
            ]
        );
    }

    #[test]
    fn test_parse_ssh_config_content() {
        let content = r#"
Host myserver
    HostName 192.168.1.100
    User alice
    Port 2222
    IdentityFile ~/.ssh/id_rsa

Host hpc-cluster
    HostName login.hpc.example.edu
    User bob

# Wildcard hosts are excluded
Host *
    ServerAliveInterval 60
"#;
        let entries = parse_ssh_config_content(content);
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].alias, "myserver");
        assert_eq!(entries[0].hostname.as_deref(), Some("192.168.1.100"));
        assert_eq!(entries[0].user.as_deref(), Some("alice"));
        assert_eq!(entries[0].port, Some(2222));
        assert_eq!(entries[0].identity_file.as_deref(), Some("~/.ssh/id_rsa"));

        assert_eq!(entries[1].alias, "hpc-cluster");
        assert_eq!(
            entries[1].hostname.as_deref(),
            Some("login.hpc.example.edu")
        );
        assert_eq!(entries[1].user.as_deref(), Some("bob"));
        assert_eq!(entries[1].port, None);
    }

    #[test]
    fn test_is_compute_command() {
        assert!(ServerManager::is_compute_command("samtools sort input.bam"));
        assert!(ServerManager::is_compute_command("bwa mem ref.fa reads.fq"));
        assert!(ServerManager::is_compute_command("python script.py"));
        assert!(!ServerManager::is_compute_command("ls -la"));
        assert!(!ServerManager::is_compute_command("squeue -u user"));
        assert!(!ServerManager::is_compute_command("module load samtools"));
    }

    #[test]
    fn test_is_login_safe_command() {
        assert!(ServerManager::is_login_safe_command("ls -la"));
        assert!(ServerManager::is_login_safe_command("squeue -u user"));
        assert!(ServerManager::is_login_safe_command("sbatch job.sh"));
        assert!(ServerManager::is_login_safe_command("module load samtools"));
        assert!(ServerManager::is_login_safe_command("conda activate env"));
        assert!(!ServerManager::is_login_safe_command(
            "samtools sort input.bam"
        ));
        assert!(!ServerManager::is_login_safe_command(
            "bwa mem ref.fa reads.fq"
        ));
    }

    #[test]
    fn test_empty_ssh_config() {
        let entries = parse_ssh_config_content("");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_ssh_config_comments_only() {
        let content = "# This is a comment\n# Another comment\n";
        let entries = parse_ssh_config_content(content);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_server_config_default() {
        let cfg = ServerConfig::default();
        assert!(cfg.hosts.is_empty());
    }

    // ─── parse_selection tests ─────────────────────────────────────────────

    #[test]
    fn test_parse_selection_all() {
        assert_eq!(parse_selection("all", 5), vec![0, 1, 2, 3, 4]);
        assert_eq!(parse_selection("a", 3), vec![0, 1, 2]);
        assert_eq!(parse_selection("ALL", 2), vec![0, 1]);
    }

    #[test]
    fn test_parse_selection_single() {
        assert_eq!(parse_selection("1", 5), vec![0]);
        assert_eq!(parse_selection("3", 5), vec![2]);
        assert_eq!(parse_selection("5", 5), vec![4]);
    }

    #[test]
    fn test_parse_selection_comma_list() {
        assert_eq!(parse_selection("1,3,5", 5), vec![0, 2, 4]);
        assert_eq!(parse_selection("2, 4", 5), vec![1, 3]);
    }

    #[test]
    fn test_parse_selection_range() {
        assert_eq!(parse_selection("1-3", 5), vec![0, 1, 2]);
        assert_eq!(parse_selection("2-4", 5), vec![1, 2, 3]);
        assert_eq!(parse_selection("1-5", 5), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_parse_selection_mixed() {
        assert_eq!(parse_selection("1,3-5", 5), vec![0, 2, 3, 4]);
        assert_eq!(parse_selection("1-2,5", 5), vec![0, 1, 4]);
    }

    #[test]
    fn test_parse_selection_deduplication() {
        assert_eq!(parse_selection("1,1,2", 5), vec![0, 1]);
        assert_eq!(parse_selection("1-3,2-4", 5), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_parse_selection_out_of_bounds() {
        assert_eq!(parse_selection("0", 5), Vec::<usize>::new());
        assert_eq!(parse_selection("6", 5), Vec::<usize>::new());
        assert_eq!(parse_selection("1-10", 3), vec![0, 1, 2]);
    }

    #[test]
    fn test_parse_selection_empty() {
        assert_eq!(parse_selection("", 5), Vec::<usize>::new());
        assert_eq!(parse_selection("   ", 5), Vec::<usize>::new());
    }
}
