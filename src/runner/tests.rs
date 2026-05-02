//! Unit and integration tests for the runner module.

use super::utils::*;
use super::*;
use std::collections::HashMap;

#[test]
fn test_detect_output_files_short_flag() {
    let args: Vec<String> = vec![
        "-o".to_string(),
        "out.bam".to_string(),
        "input.bam".to_string(),
    ];
    let files = detect_output_files(&args);
    assert!(files.contains(&"out.bam".to_string()));
}

#[test]
fn test_detect_output_files_long_flag() {
    let args: Vec<String> = vec!["--output".to_string(), "result.vcf".to_string()];
    let files = detect_output_files(&args);
    assert!(files.contains(&"result.vcf".to_string()));
}

#[test]
fn test_detect_output_files_equals_form() {
    let args: Vec<String> = vec!["--output=sorted.bam".to_string()];
    let files = detect_output_files(&args);
    assert!(files.contains(&"sorted.bam".to_string()));
}

#[test]
fn test_detect_output_files_positional() {
    let args: Vec<String> = vec![
        "-t".to_string(),
        "8".to_string(),
        "input.fastq.gz".to_string(),
        "output.fastq.gz".to_string(),
    ];
    let files = detect_output_files(&args);
    assert!(files.contains(&"input.fastq.gz".to_string()));
    assert!(files.contains(&"output.fastq.gz".to_string()));
}

#[test]
fn test_detect_output_files_deduplicates() {
    let args: Vec<String> = vec![
        "-o".to_string(),
        "out.bam".to_string(),
        "out.bam".to_string(),
    ];
    let files = detect_output_files(&args);
    assert_eq!(files.iter().filter(|f| *f == "out.bam").count(), 1);
}

#[test]
fn test_detect_output_files_skips_flags() {
    let args: Vec<String> = vec![
        "--threads".to_string(),
        "8".to_string(),
        "--sort".to_string(),
    ];
    let files = detect_output_files(&args);
    assert!(!files.contains(&"--threads".to_string()));
    assert!(!files.contains(&"--sort".to_string()));
}

// ─── build_command_string ─────────────────────────────────────────────────

#[test]
fn test_build_command_string_no_args() {
    assert_eq!(build_command_string("echo", &[]), "echo");
}

#[test]
fn test_build_command_string_simple_args() {
    let args: Vec<String> = vec!["-o".to_string(), "out.bam".to_string()];
    let cmd = build_command_string("samtools", &args);
    assert_eq!(cmd, "samtools -o out.bam");
}

#[test]
fn test_build_command_string_quotes_args_with_spaces() {
    let args: Vec<String> = vec!["--output".to_string(), "my output file.bam".to_string()];
    let cmd = build_command_string("samtools", &args);
    assert!(
        cmd.contains("'my output file.bam'"),
        "args with spaces should be quoted"
    );
}

#[test]
fn test_build_command_string_quotes_args_with_special_chars() {
    let args: Vec<String> = vec!["--filter".to_string(), "flag & 0x4".to_string()];
    let cmd = build_command_string("samtools", &args);
    assert!(cmd.contains("'flag"), "args with & should be quoted");
}

#[test]
fn test_build_command_string_does_not_quote_shell_and_and() {
    let args: Vec<String> = vec![
        "sort".to_string(),
        "-o".to_string(),
        "sorted.bam".to_string(),
        "input.bam".to_string(),
        "&&".to_string(),
        "samtools".to_string(),
        "index".to_string(),
        "sorted.bam".to_string(),
    ];
    let cmd = build_command_string("samtools", &args);
    assert!(cmd.contains(" && "), "cmd should contain unquoted &&");
    assert!(!cmd.contains("'&&'"), "&& must not be single-quoted");
    assert!(
        cmd.contains("samtools index"),
        "second subcommand must be present"
    );
}

#[test]
fn test_build_command_string_does_not_quote_pipe() {
    let args: Vec<String> = vec![
        "view".to_string(),
        "input.bam".to_string(),
        "|".to_string(),
        "grep".to_string(),
        "^SQ".to_string(),
    ];
    let cmd = build_command_string("samtools", &args);
    assert!(cmd.contains(" | "), "cmd should contain unquoted |");
    assert!(!cmd.contains("'|'"), "| must not be single-quoted");
}

// ─── is_shell_operator ────────────────────────────────────────────────────

#[test]
fn test_is_shell_operator_known_operators() {
    assert!(is_shell_operator("&&"));
    assert!(is_shell_operator("||"));
    assert!(is_shell_operator(";"));
    assert!(is_shell_operator("|"));
    assert!(is_shell_operator(">"));
    assert!(is_shell_operator(">>"));
    assert!(is_shell_operator("<"));
    assert!(is_shell_operator("2>"));
    assert!(is_shell_operator("2>>"));
}

#[test]
fn test_is_shell_operator_rejects_non_operators() {
    assert!(!is_shell_operator("-o"));
    assert!(!is_shell_operator("out.bam"));
    assert!(!is_shell_operator("&"));
    assert!(!is_shell_operator("flag & 0x4"));
    assert!(!is_shell_operator("samtools"));
    assert!(!is_shell_operator(""));
}

// ─── args_require_shell ───────────────────────────────────────────────────

#[test]
fn test_args_require_shell_with_double_ampersand() {
    let args: Vec<String> = vec![
        "sort".to_string(),
        "-o".to_string(),
        "sorted.bam".to_string(),
        "&&".to_string(),
        "samtools".to_string(),
        "index".to_string(),
        "sorted.bam".to_string(),
    ];
    assert!(args_require_shell(&args));
}

#[test]
fn test_args_require_shell_with_pipe() {
    let args: Vec<String> = vec!["view".to_string(), "|".to_string(), "grep".to_string()];
    assert!(args_require_shell(&args));
}

#[test]
fn test_args_require_shell_without_operators() {
    let args: Vec<String> = vec![
        "sort".to_string(),
        "-o".to_string(),
        "out.bam".to_string(),
        "input.bam".to_string(),
    ];
    assert!(!args_require_shell(&args));
}

#[test]
fn test_args_require_shell_empty() {
    assert!(!args_require_shell(&[]));
}

// ─── needs_quoting ────────────────────────────────────────────────────────

#[test]
fn test_needs_quoting_simple_arg_false() {
    assert!(!needs_quoting("-o"));
    assert!(!needs_quoting("out.bam"));
    assert!(!needs_quoting("--threads=8"));
}

#[test]
fn test_needs_quoting_space_true() {
    assert!(needs_quoting("my file.bam"));
}

#[test]
fn test_needs_quoting_special_chars_true() {
    assert!(needs_quoting("a;b"));
    assert!(needs_quoting("a&b"));
    assert!(needs_quoting("a|b"));
    assert!(needs_quoting("$HOME"));
    assert!(needs_quoting("`cmd`"));
    assert!(needs_quoting("(subshell)"));
    assert!(needs_quoting("a<b"));
    assert!(needs_quoting("a>b"));
    assert!(needs_quoting("a!b"));
    assert!(needs_quoting("a\\b"));
    assert!(needs_quoting("a\"b"));
    assert!(needs_quoting("a'b"));
}

#[test]
fn test_needs_quoting_tab_true() {
    assert!(needs_quoting("a\tb"));
}

// ─── sha256_hex ───────────────────────────────────────────────────────────

#[test]
fn test_sha256_hex_empty_string() {
    let hash = sha256_hex("");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_sha256_hex_hello_world() {
    let hash = sha256_hex("hello world");
    assert_eq!(hash.len(), 64, "SHA256 hex should be 64 characters");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_sha256_hex_deterministic() {
    let hash1 = sha256_hex("test input");
    let hash2 = sha256_hex("test input");
    assert_eq!(hash1, hash2, "SHA256 should be deterministic");
}

#[test]
fn test_sha256_hex_different_inputs_produce_different_hashes() {
    let hash1 = sha256_hex("input one");
    let hash2 = sha256_hex("input two");
    assert_ne!(hash1, hash2);
}

// ─── Runner::new and builder methods ─────────────────────────────────────

#[test]
fn test_runner_new() {
    use crate::config::Config;
    let cfg = Config::default();
    let mut runner = Runner::new(cfg);
    runner.with_verbose(true);
    runner.with_no_cache(true);
    runner.with_verify(true);
    runner.with_auto_retry(true);
}

// ─── detect_tool_version ─────────────────────────────────────────────────

#[test]
fn test_detect_tool_version_existing_tool() {
    let result = detect_tool_version("ls");
    let _ = result;
}

#[test]
fn test_detect_tool_version_nonexistent_tool_returns_none() {
    let result = detect_tool_version("__nonexistent_binary_oxo_call_test__");
    assert!(result.is_none(), "nonexistent tool should return None");
}

#[test]
fn test_detect_tool_version_echo_command() {
    let _result = detect_tool_version("echo");
}

// ─── make_spinner ─────────────────────────────────────────────────────────

#[test]
fn test_make_spinner_creates_without_panic() {
    let pb = make_spinner("Test message");
    pb.finish_and_clear();
}

#[test]
fn test_make_spinner_with_empty_message() {
    let pb = make_spinner("");
    pb.finish_and_clear();
}

// ─── detect_output_files extra edge cases ────────────────────────────────

#[test]
fn test_detect_output_files_bam_flag() {
    let args: Vec<String> = vec!["--bam".to_string(), "output.bam".to_string()];
    let files = detect_output_files(&args);
    assert!(
        files.contains(&"output.bam".to_string()),
        "--bam flag should capture next arg"
    );
}

#[test]
fn test_detect_output_files_short_b_flag() {
    let args: Vec<String> = vec!["-b".to_string(), "output.bam".to_string()];
    let files = detect_output_files(&args);
    assert!(
        files.contains(&"output.bam".to_string()),
        "-b flag should capture next arg"
    );
}

#[test]
fn test_detect_output_files_equals_form_bam() {
    let args: Vec<String> = vec!["-b=output.bam".to_string()];
    let files = detect_output_files(&args);
    assert!(files.contains(&"output.bam".to_string()));
}

#[test]
fn test_detect_output_files_equals_form_empty_value_ignored() {
    let args: Vec<String> = vec!["--output=".to_string()];
    let files = detect_output_files(&args);
    assert!(
        !files.contains(&String::new()),
        "empty value after = should not be collected"
    );
}

#[test]
fn test_detect_output_files_positional_with_semicolon_excluded() {
    let args: Vec<String> = vec!["input;rm -rf /".to_string()];
    let files = detect_output_files(&args);
    assert!(files.is_empty(), "args with ; should be excluded");
}

#[test]
fn test_detect_output_files_positional_with_pipe_excluded() {
    let args: Vec<String> = vec!["input|cat".to_string()];
    let files = detect_output_files(&args);
    assert!(files.is_empty(), "args with | should be excluded");
}

#[test]
fn test_detect_output_files_positional_with_ampersand_excluded() {
    let args: Vec<String> = vec!["input&output".to_string()];
    let files = detect_output_files(&args);
    assert!(files.is_empty(), "args with & should be excluded");
}

#[test]
fn test_detect_output_files_truncates_at_20() {
    let mut args: Vec<String> = Vec::new();
    for i in 0..25 {
        args.push(format!("positional_{i}.bam"));
    }
    let files = detect_output_files(&args);
    assert!(
        files.len() <= 20,
        "detect_output_files should cap at 20 entries"
    );
}

#[test]
fn test_detect_output_files_no_dot_excluded() {
    let args: Vec<String> = vec!["nodot".to_string(), "anotherword".to_string()];
    let files = detect_output_files(&args);
    assert!(
        !files.contains(&"nodot".to_string()),
        "arg without dot should not be collected"
    );
}

// ─── build_command_string: single-quote escaping ─────────────────────────

#[test]
fn test_build_command_string_escapes_single_quotes_in_args() {
    let args: Vec<String> = vec!["it's".to_string()];
    let cmd = build_command_string("echo", &args);
    assert!(
        cmd.contains("'\\'"),
        "single quote should be escaped as '\\'"
    );
}

// ─── companion binary detection ───────────────────────────────────────────

#[test]
fn test_is_companion_binary_bowtie2_build() {
    assert!(is_companion_binary("bowtie2", "bowtie2-build"));
}

#[test]
fn test_is_companion_binary_hisat2_build() {
    assert!(is_companion_binary("hisat2", "hisat2-build"));
}

#[test]
fn test_is_companion_binary_bismark_underscore_prefix() {
    assert!(is_companion_binary("bismark", "bismark_genome_preparation"));
    assert!(is_companion_binary(
        "bismark",
        "bismark_methylation_extractor"
    ));
}

#[test]
fn test_is_companion_binary_reverse_suffix() {
    assert!(is_companion_binary("bismark", "deduplicate_bismark"));
}

#[test]
fn test_is_companion_binary_reverse_suffix_requires_prefix() {
    assert!(!is_companion_binary("bismark", "_bismark"));
}

#[test]
fn test_is_companion_binary_flag_is_not_companion() {
    assert!(!is_companion_binary("bowtie2", "-x"));
    assert!(!is_companion_binary("bowtie2", "--no-unal"));
}

#[test]
fn test_is_companion_binary_filename_is_not_companion() {
    assert!(!is_companion_binary("bowtie2", "bowtie2-input.fq"));
    assert!(!is_companion_binary("samtools", "sorted.bam"));
}

#[test]
fn test_is_companion_binary_script_extension() {
    assert!(is_companion_binary("manta", "configureManta.py"));
    assert!(is_companion_binary("strelka2", "configureStrelka2.py"));
    assert!(!is_companion_binary("homer", "annotatePeaks.pl"));
}

#[test]
fn test_is_companion_binary_script_prefix() {
    assert!(!is_companion_binary("bbtools", "bbduk.sh"));
}

#[test]
fn test_is_companion_binary_no_prefix_match() {
    assert!(!is_companion_binary("samtools", "sort"));
    assert!(!is_companion_binary("samtools", "index"));
}

#[test]
fn test_effective_command_companion_redirects_tool() {
    let args: Vec<String> = vec![
        "bowtie2-build".to_string(),
        "reference.fa".to_string(),
        "ref_idx".to_string(),
    ];
    let (eff_tool, eff_args) = effective_command("bowtie2", &args);
    assert_eq!(eff_tool, "bowtie2-build");
    assert_eq!(eff_args, &["reference.fa", "ref_idx"]);
}

#[test]
fn test_effective_command_normal_args_unchanged() {
    let args: Vec<String> = vec![
        "-x".to_string(),
        "ref_idx".to_string(),
        "-1".to_string(),
        "R1.fq.gz".to_string(),
    ];
    let (eff_tool, eff_args) = effective_command("bowtie2", &args);
    assert_eq!(eff_tool, "bowtie2");
    assert_eq!(eff_args, args.as_slice());
}

#[test]
fn test_effective_command_samtools_subcommand_unchanged() {
    let args: Vec<String> = vec![
        "sort".to_string(),
        "-@".to_string(),
        "4".to_string(),
        "-o".to_string(),
        "sorted.bam".to_string(),
    ];
    let (eff_tool, eff_args) = effective_command("samtools", &args);
    assert_eq!(eff_tool, "samtools");
    assert_eq!(eff_args, args.as_slice());
}

#[test]
fn test_build_command_string_companion_binary() {
    let args: Vec<String> = vec![
        "bowtie2-build".to_string(),
        "reference.fa".to_string(),
        "ref_idx".to_string(),
    ];
    let cmd = build_command_string("bowtie2", &args);
    assert_eq!(cmd, "bowtie2-build reference.fa ref_idx");
    assert!(
        !cmd.starts_with("bowtie2 bowtie2-build"),
        "must not double the tool name"
    );
}

#[test]
fn test_effective_command_script_companion() {
    let args: Vec<String> = vec![
        "configureManta.py".to_string(),
        "--bam".to_string(),
        "input.bam".to_string(),
        "--referenceFasta".to_string(),
        "ref.fa".to_string(),
    ];
    let (eff_tool, eff_args) = effective_command("manta", &args);
    assert_eq!(eff_tool, "configureManta.py");
    assert_eq!(
        eff_args,
        &["--bam", "input.bam", "--referenceFasta", "ref.fa"]
    );
}

#[test]
fn test_effective_command_standalone_script() {
    let args: Vec<String> = vec![
        "bbduk.sh".to_string(),
        "in=reads.fq".to_string(),
        "out=clean.fq".to_string(),
        "ref=adapters.fa".to_string(),
    ];
    let (eff_tool, eff_args) = effective_command("bbtools", &args);
    assert_eq!(eff_tool, "bbduk.sh");
    assert_eq!(
        eff_args,
        &["in=reads.fq", "out=clean.fq", "ref=adapters.fa"]
    );
}

#[test]
fn test_effective_command_rseqc_script() {
    let args: Vec<String> = vec![
        "infer_experiment.py".to_string(),
        "-i".to_string(),
        "aligned.bam".to_string(),
        "-r".to_string(),
        "ref.bed".to_string(),
    ];
    let (eff_tool, eff_args) = effective_command("rseqc", &args);
    assert_eq!(eff_tool, "infer_experiment.py");
    assert_eq!(eff_args, &["-i", "aligned.bam", "-r", "ref.bed"]);
}

#[test]
fn test_is_script_executable() {
    assert!(is_script_executable("bbduk.sh"));
    assert!(is_script_executable("infer_experiment.py"));
    assert!(is_script_executable("annotatePeaks.pl"));
    assert!(is_script_executable("draw_fusions.R"));
    assert!(is_script_executable("configureStrelkaGermlineWorkflow.py"));
    assert!(!is_script_executable("reads.fastq.gz"));
    assert!(!is_script_executable("-i"));
    assert!(!is_script_executable("/usr/bin/script.py"));
    assert!(!is_script_executable("sort"));
    assert!(!is_script_executable("input.bam"));
}

#[test]
fn test_effective_command_data_file_not_script() {
    let args: Vec<String> = vec!["input.bam".to_string(), "-o".to_string()];
    let (eff_tool, _) = effective_command("samtools", &args);
    assert_eq!(eff_tool, "samtools");
}

// ─── Risk assessment tests ────────────────────────────────────────────

#[test]
fn test_risk_safe_command() {
    let args: Vec<String> = vec![
        "sort".into(),
        "-@".into(),
        "4".into(),
        "-o".into(),
        "out.bam".into(),
        "in.bam".into(),
    ];
    assert_eq!(assess_command_risk(&args), RiskLevel::Safe);
}

#[test]
fn test_risk_dangerous_rm() {
    let args: Vec<String> = vec!["rm".into(), "-rf".into(), "/tmp/foo".into()];
    assert_eq!(assess_command_risk(&args), RiskLevel::Dangerous);
}

#[test]
fn test_risk_dangerous_sudo() {
    let args: Vec<String> = vec!["sudo".into(), "apt".into(), "install".into()];
    assert_eq!(assess_command_risk(&args), RiskLevel::Dangerous);
}

#[test]
fn test_risk_dangerous_in_pipeline() {
    let args: Vec<String> = vec![
        "sort".into(),
        "-o".into(),
        "out.bam".into(),
        "&&".into(),
        "rm".into(),
        "in.bam".into(),
    ];
    assert_eq!(assess_command_risk(&args), RiskLevel::Dangerous);
}

#[test]
fn test_risk_warning_force_flag() {
    let args: Vec<String> = vec![
        "sort".into(),
        "--force".into(),
        "-o".into(),
        "out.bam".into(),
    ];
    assert_eq!(assess_command_risk(&args), RiskLevel::Warning);
}

#[test]
fn test_risk_warning_redirect() {
    let args: Vec<String> = vec!["view".into(), "in.bam".into(), ">".into(), "out.sam".into()];
    assert_eq!(assess_command_risk(&args), RiskLevel::Warning);
}

#[test]
fn test_risk_warning_same_input_output() {
    let args: Vec<String> = vec![
        "sort".into(),
        "-o".into(),
        "data.bam".into(),
        "data.bam".into(),
    ];
    assert_eq!(assess_command_risk(&args), RiskLevel::Warning);
}

#[test]
fn test_risk_empty_args() {
    let args: Vec<String> = vec![];
    assert_eq!(assess_command_risk(&args), RiskLevel::Safe);
}

#[test]
fn test_risk_warning_message() {
    assert!(risk_warning_message(RiskLevel::Dangerous).is_some());
    assert!(risk_warning_message(RiskLevel::Warning).is_some());
    assert!(risk_warning_message(RiskLevel::Safe).is_none());
}

// ─── Input file validation tests ──────────────────────────────────────

#[test]
fn test_validate_input_files_nonexistent() {
    let args: Vec<String> = vec![
        "sort".into(),
        "-o".into(),
        "out.bam".into(),
        "nonexistent_file.bam".into(),
    ];
    let missing = validate_input_files(&args);
    assert!(missing.contains(&"nonexistent_file.bam".to_string()));
}

#[test]
fn test_validate_input_files_skips_output() {
    let args: Vec<String> = vec!["sort".into(), "-o".into(), "nonexistent_output.bam".into()];
    let missing = validate_input_files(&args);
    assert!(!missing.contains(&"nonexistent_output.bam".to_string()));
}

#[test]
fn test_has_bio_extension() {
    assert!(has_bio_extension("reads.fastq.gz"));
    assert!(has_bio_extension("output.bam"));
    assert!(has_bio_extension("variants.vcf"));
    assert!(has_bio_extension("regions.bed"));
    assert!(!has_bio_extension("script.py"));
    assert!(!has_bio_extension("config.toml"));
}

// ─── Version parsing tests ─────────────────────────────────────────────────────

#[test]
fn test_parse_version_simple() {
    assert_eq!(parse_version("1.17.0"), Some((1, 17, 0)));
    assert_eq!(parse_version("1.17"), Some((1, 17, 0)));
    assert_eq!(parse_version("2.0"), Some((2, 0, 0)));
}

#[test]
fn test_parse_version_with_prefix() {
    assert_eq!(parse_version("samtools 1.17"), Some((1, 17, 0)));
    assert_eq!(parse_version("bwa-mem2 version 2.2.1"), Some((2, 2, 1)));
    assert_eq!(parse_version("tool v1.0.0"), Some((1, 0, 0)));
}

#[test]
fn test_parse_version_invalid() {
    assert_eq!(parse_version("no version here"), None);
    assert_eq!(parse_version(""), None);
}

#[test]
fn test_check_version_compatibility_ok() {
    assert!(check_version_compatibility("1.17.0", Some("1.10"), Some("2.0")).is_ok());
    assert!(check_version_compatibility("1.17", Some("1.17"), None).is_ok());
    assert!(check_version_compatibility("2.0.0", None, Some("2.5")).is_ok());
}

#[test]
fn test_check_version_compatibility_below_min() {
    let result = check_version_compatibility("1.5.0", Some("1.10"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("below minimum"));
}

#[test]
fn test_check_version_compatibility_above_max() {
    let result = check_version_compatibility("3.0.0", None, Some("2.5"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceeds maximum"));
}

#[test]
fn test_check_version_compatibility_no_constraints() {
    assert!(check_version_compatibility("1.0.0", None, None).is_ok());
}

// ─── Runner builder comprehensive tests ──────────────────────────────────

#[test]
fn test_runner_builder_verbose() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_verbose(true);
    assert!(runner.verbose);
}

#[test]
fn test_runner_builder_no_cache() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_no_cache(true);
    assert!(runner.no_cache);
}

#[test]
fn test_runner_builder_verify() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_verify(true);
    assert!(runner.verify);
}

#[test]
fn test_runner_builder_auto_retry() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_auto_retry(true);
    assert!(runner.auto_retry);
}

#[test]
fn test_runner_builder_vars() {
    use crate::config::Config;
    let mut vars = HashMap::new();
    vars.insert("sample".to_string(), "NA12878".to_string());
    vars.insert("ref".to_string(), "hg38.fa".to_string());
    let mut runner = Runner::new(Config::default());
    runner.with_vars(vars);
    assert_eq!(runner.vars.len(), 2);
    assert_eq!(runner.vars.get("sample"), Some(&"NA12878".to_string()));
}

#[test]
fn test_runner_builder_input_items() {
    use crate::config::Config;
    let items = vec!["sample1.bam".to_string(), "sample2.bam".to_string()];
    let mut runner = Runner::new(Config::default());
    runner.with_input_items(items);
    assert_eq!(runner.input_items.len(), 2);
}

#[test]
fn test_runner_builder_jobs() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_jobs(4);
    assert_eq!(runner.jobs, 4);
}

#[test]
fn test_runner_builder_jobs_minimum_one() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_jobs(0);
    assert_eq!(runner.jobs, 1, "jobs should be clamped to minimum of 1");
}

#[test]
fn test_runner_builder_stop_on_error() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_stop_on_error(true);
    assert!(runner.stop_on_error);
}

#[test]
fn test_runner_builder_chaining() {
    use crate::config::Config;
    let mut runner = Runner::new(Config::default());
    runner.with_verbose(true);
    runner.with_no_cache(true);
    runner.with_verify(true);
    runner.with_auto_retry(true);
    runner.with_jobs(8);
    runner.with_stop_on_error(true);

    assert!(runner.verbose);
    assert!(runner.no_cache);
    assert!(runner.verify);
    assert!(runner.auto_retry);
    assert_eq!(runner.jobs, 8);
    assert!(runner.stop_on_error);
}

#[test]
fn test_runner_defaults() {
    use crate::config::Config;
    let runner = Runner::new(Config::default());
    assert!(!runner.verbose);
    assert!(!runner.no_cache);
    assert!(!runner.verify);
    assert!(!runner.auto_retry);
    assert_eq!(runner.jobs, 1);
    assert!(!runner.stop_on_error);
    assert!(runner.vars.is_empty());
    assert!(runner.input_items.is_empty());
}
