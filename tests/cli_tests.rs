/// Integration tests for oxo-call CLI
use std::path::PathBuf;
use std::process::Command;

/// Path to the pre-generated test license fixture (signed with the demo key).
fn test_license_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test_license.oxo.json")
}

/// Build a Command that automatically injects the test license via env var,
/// so all core commands can run without manual license setup.
fn oxo_call() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_oxo-call"));
    cmd.env("OXO_CALL_LICENSE", test_license_path());
    cmd
}

/// Build a Command WITHOUT any license (for testing license-enforcement paths).
fn oxo_call_no_license() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_oxo-call"));
    cmd.env_remove("OXO_CALL_LICENSE");
    cmd
}

#[test]
fn test_help_output() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("oxo-call"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("dry-run"));
    assert!(stdout.contains("index"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("docs"));
    assert!(stdout.contains("history"));
}

#[test]
fn test_run_help_mentions_ask_flag() {
    let output = oxo_call()
        .args(["run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--ask"));
    assert!(stdout.contains("Ask for confirmation"));
}

#[test]
fn test_version_output() {
    let output = oxo_call()
        .arg("--version")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("oxo-call"));
}

#[test]
fn test_config_show() {
    let output = oxo_call()
        .args(["config", "show"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("github-copilot"));
    assert!(stdout.contains("max_tokens"));
    assert!(stdout.contains("temperature"));
    assert!(stdout.contains("Stored values"));
    assert!(stdout.contains("Effective values"));
}

#[test]
fn test_config_path() {
    let output = oxo_call()
        .args(["config", "path"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("oxo-call"));
    assert!(stdout.contains("config.toml"));
}

#[test]
fn test_config_set_and_get() {
    // Set a value
    let set_output = oxo_call()
        .args(["config", "set", "llm.max_tokens", "1024"])
        .output()
        .expect("failed to run oxo-call");
    assert!(set_output.status.success());

    // Get the value back
    let get_output = oxo_call()
        .args(["config", "get", "llm.max_tokens"])
        .output()
        .expect("failed to run oxo-call");
    assert!(get_output.status.success());
    let stdout = String::from_utf8_lossy(&get_output.stdout);
    assert!(stdout.trim() == "1024");

    // Restore default
    let _ = oxo_call()
        .args(["config", "set", "llm.max_tokens", "2048"])
        .output();
}

#[test]
fn test_config_invalid_key() {
    let output = oxo_call()
        .args(["config", "set", "invalid.key", "value"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown config key") || stderr.contains("error"));
}

#[test]
fn test_config_get_uses_env_overrides() {
    let provider = oxo_call()
        .env("OXO_CALL_LLM_PROVIDER", "ollama")
        .args(["config", "get", "llm.provider"])
        .output()
        .expect("failed to run oxo-call");
    assert!(provider.status.success());
    assert_eq!(String::from_utf8_lossy(&provider.stdout).trim(), "ollama");

    let api_base = oxo_call()
        .env("OXO_CALL_LLM_API_BASE", "http://localhost:1234/v1")
        .args(["config", "get", "llm.api_base"])
        .output()
        .expect("failed to run oxo-call");
    assert!(api_base.status.success());
    assert_eq!(
        String::from_utf8_lossy(&api_base.stdout).trim(),
        "http://localhost:1234/v1"
    );

    let model = oxo_call()
        .env("OXO_CALL_LLM_MODEL", "custom-model")
        .args(["config", "get", "llm.model"])
        .output()
        .expect("failed to run oxo-call");
    assert!(model.status.success());
    assert_eq!(
        String::from_utf8_lossy(&model.stdout).trim(),
        "custom-model"
    );

    let max_tokens = oxo_call()
        .env("OXO_CALL_LLM_MAX_TOKENS", "4096")
        .args(["config", "get", "llm.max_tokens"])
        .output()
        .expect("failed to run oxo-call");
    assert!(max_tokens.status.success());
    assert_eq!(String::from_utf8_lossy(&max_tokens.stdout).trim(), "4096");

    let temperature = oxo_call()
        .env("OXO_CALL_LLM_TEMPERATURE", "0.7")
        .args(["config", "get", "llm.temperature"])
        .output()
        .expect("failed to run oxo-call");
    assert!(temperature.status.success());
    assert_eq!(String::from_utf8_lossy(&temperature.stdout).trim(), "0.7");

    let auto_update = oxo_call()
        .env("OXO_CALL_DOCS_AUTO_UPDATE", "false")
        .args(["config", "get", "docs.auto_update"])
        .output()
        .expect("failed to run oxo-call");
    assert!(auto_update.status.success());
    assert_eq!(String::from_utf8_lossy(&auto_update.stdout).trim(), "false");
}

#[test]
fn test_config_get_api_token_supports_key_specific_and_legacy_env_vars() {
    let key_specific = oxo_call()
        .env("OXO_CALL_LLM_API_TOKEN", "token-from-key-env")
        .args(["config", "get", "llm.api_token"])
        .output()
        .expect("failed to run oxo-call");
    assert!(key_specific.status.success());
    assert_eq!(
        String::from_utf8_lossy(&key_specific.stdout).trim(),
        "token-from-key-env"
    );

    let legacy = oxo_call()
        .env("OXO_CALL_LLM_PROVIDER", "openai")
        .env_remove("OXO_CALL_LLM_API_TOKEN")
        .env("OPENAI_API_KEY", "token-from-openai-env")
        .args(["config", "get", "llm.api_token"])
        .output()
        .expect("failed to run oxo-call");
    assert!(legacy.status.success());
    assert_eq!(
        String::from_utf8_lossy(&legacy.stdout).trim(),
        "token-from-openai-env"
    );
}

#[test]
fn test_config_get_invalid_env_value_fails() {
    let output = oxo_call()
        .env("OXO_CALL_LLM_MAX_TOKENS", "not-a-number")
        .args(["config", "get", "llm.max_tokens"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("OXO_CALL_LLM_MAX_TOKENS") || stderr.contains("Invalid value"));
}

#[test]
fn test_config_show_displays_effective_sources() {
    let output = oxo_call()
        .env("OXO_CALL_LLM_PROVIDER", "ollama")
        .env("OXO_CALL_LLM_API_TOKEN", "token-from-env")
        .args(["config", "show"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("stored config.toml / built-in defaults")
            || stdout.contains("Stored values")
    );
    assert!(stdout.contains("env:OXO_CALL_LLM_PROVIDER"));
    assert!(stdout.contains("env:OXO_CALL_LLM_API_TOKEN"));
}

#[test]
fn test_config_verify_insecure_remote_api_base_fails_with_guidance() {
    let output = oxo_call()
        .env("OXO_CALL_LLM_PROVIDER", "openai")
        .env("OXO_CALL_LLM_API_TOKEN", "dummy-token")
        .env("OXO_CALL_LLM_API_BASE", "http://example.com/v1")
        .args(["config", "verify"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("configuration check failed"));
    assert!(stderr.contains("HTTPS") || stderr.contains("https://"));
    assert!(stderr.contains("llm.api_base") || stderr.contains("Use an `https://` API base"));
}

#[test]
fn test_index_list_empty_or_filled() {
    let output = oxo_call()
        .args(["index", "list"])
        .output()
        .expect("failed to run oxo-call");
    // Should succeed whether or not there are entries
    assert!(output.status.success());
}

#[test]
fn test_index_add_real_tool() {
    // 'ls' is always available
    let output = oxo_call()
        .args(["index", "add", "ls"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Indexed") || stdout.contains("ls"));
}

#[test]
fn test_index_add_and_list() {
    // Index a tool via the legacy 'index add' command
    let add_output = oxo_call()
        .args(["index", "add", "cat"])
        .output()
        .expect("failed to run oxo-call");
    assert!(add_output.status.success(), "index add cat should succeed");

    // List should contain the indexed tool
    let output = oxo_call()
        .args(["index", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cat"),
        "Expected 'cat' in index list output, got: {stdout}"
    );
}

#[test]
fn test_docs_show_for_indexed_tool() {
    // Make sure 'ls' is indexed
    let _ = oxo_call().args(["index", "add", "ls"]).output();

    let output = oxo_call()
        .args(["docs", "show", "ls"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain ls help text
    assert!(stdout.len() > 100);
}

#[test]
fn test_docs_path() {
    let output = oxo_call()
        .args(["docs", "path", "ls"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ls.md"));
}

#[test]
fn test_history_list_empty() {
    let output = oxo_call()
        .args(["history", "list"])
        .output()
        .expect("failed to run oxo-call");
    // Should succeed
    assert!(output.status.success());
}

#[test]
fn test_index_remove_nonexistent() {
    let output = oxo_call()
        .args(["index", "remove", "nonexistent_tool_xyz"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("not in the index"));
}

#[test]
fn test_index_add_nonexistent_tool() {
    let output = oxo_call()
        .args(["index", "add", "nonexistent_tool_xyz_123"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
}

#[test]
fn test_dry_run_requires_llm_token() {
    // dry-run for a non-indexed tool should fail gracefully
    let output = oxo_call()
        .args(["dry-run", "ls", "show all files"])
        .env_remove("GITHUB_TOKEN")
        .env_remove("GH_TOKEN")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OXO_API_TOKEN")
        .output()
        .expect("failed to run oxo-call");
    // Should fail due to missing API token or network error - either is acceptable
    // (exit code != 0 or it proceeds to make an HTTP call that fails)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Either an error about token or about network
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("token")
            || combined.contains("API")
            || combined.contains("error")
            || combined.contains("Fetching"),
        "Expected some output from dry-run, got: {combined}"
    );
}

#[test]
fn test_index_add_path_traversal_fails() {
    let output = oxo_call()
        .args(["index", "add", "../etc/passwd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("path"),
        "Expected error for path traversal tool name, got: {stderr}"
    );
}

#[test]
fn test_index_add_slash_in_name_fails() {
    let output = oxo_call()
        .args(["index", "add", "some/tool"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
}

#[test]
fn test_docs_fetch_non_http_url_fails() {
    let output = oxo_call()
        .args(["docs", "fetch", "sometool", "file:///etc/passwd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("https"),
        "Expected error for non-http URL, got: {stderr}"
    );
}

// ─── Skill command tests ───────────────────────────────────────────────────────

#[test]
fn test_skill_list() {
    let output = oxo_call()
        .args(["skill", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list the built-in skills
    assert!(
        stdout.contains("samtools"),
        "Expected samtools in skill list, got: {stdout}"
    );
    assert!(
        stdout.contains("bwa"),
        "Expected bwa in skill list, got: {stdout}"
    );
    assert!(
        stdout.contains("built-in"),
        "Expected 'built-in' label, got: {stdout}"
    );
}

#[test]
fn test_skill_show_builtin() {
    let output = oxo_call()
        .args(["skill", "show", "samtools"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("samtools"),
        "Expected skill content, got: {stdout}"
    );
    assert!(
        stdout.contains("Expert"),
        "Expected expert knowledge section, got: {stdout}"
    );
    assert!(
        stdout.contains("Example"),
        "Expected worked examples, got: {stdout}"
    );
}

#[test]
fn test_skill_show_unknown_tool() {
    let output = oxo_call()
        .args(["skill", "show", "nonexistent_tool_xyz"])
        .output()
        .expect("failed to run oxo-call");
    // Should succeed (just shows "no skill found" message)
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No skill") || stdout.contains("install"),
        "Expected helpful message, got: {stdout}"
    );
}

#[test]
fn test_skill_create_template() {
    let output = oxo_call()
        .args(["skill", "create", "mytool"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("mytool"),
        "Expected tool name in template, got: {stdout}"
    );
    assert!(
        stdout.contains("[meta]"),
        "Expected TOML structure, got: {stdout}"
    );
    assert!(
        stdout.contains("[[examples]]"),
        "Expected examples section, got: {stdout}"
    );
}

#[test]
fn test_skill_path() {
    let output = oxo_call()
        .args(["skill", "path"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("skills"),
        "Expected skills path, got: {stdout}"
    );
}

// ─── License command tests ────────────────────────────────────────────────────

#[test]
fn test_license_command() {
    let output = oxo_call()
        .args(["license"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("oxo-call License Information"),
        "Expected license info header, got: {stdout}"
    );
    assert!(
        stdout.contains("academic"),
        "Expected academic use mention, got: {stdout}"
    );
    assert!(
        stdout.contains("commercial"),
        "Expected commercial info, got: {stdout}"
    );
    assert!(
        stdout.contains("license.oxo.json") || stdout.contains("OXO_CALL_LICENSE"),
        "Expected license file instructions, got: {stdout}"
    );
}

#[test]
fn test_license_verify_no_file() {
    // Verify command with a non-existent license file should exit non-zero
    let output = oxo_call()
        .args([
            "--license",
            "/tmp/nonexistent-oxo-license-12345.json",
            "license",
            "verify",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("license error") || stderr.contains("No license"),
        "Expected license error, got: {stderr}"
    );
}

#[test]
fn test_help_includes_skill_and_license() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("skill") || stdout.contains("Skill"),
        "Expected skill subcommand in help, got: {stdout}"
    );
    assert!(
        stdout.contains("license") || stdout.contains("License"),
        "Expected license subcommand in help, got: {stdout}"
    );
}

// ─── License enforcement tests ────────────────────────────────────────────────

#[test]
fn test_core_command_blocked_without_license() {
    // Run a core command (config show) without any license file — should fail.
    let output = oxo_call_no_license()
        .env(
            "OXO_CALL_LICENSE",
            "/tmp/nonexistent-license-enforcement-test.json",
        )
        .args(["config", "show"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !output.status.success(),
        "Expected failure without license, but command succeeded"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("license") || stderr.contains("No license"),
        "Expected license error message, got: {stderr}"
    );
}

#[test]
fn test_help_allowed_without_license() {
    // --help must work even without a license
    let output = oxo_call_no_license()
        .env(
            "OXO_CALL_LICENSE",
            "/tmp/nonexistent-license-enforcement-test.json",
        )
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "--help should work without a license"
    );
}

#[test]
fn test_version_allowed_without_license() {
    // --version must work even without a license
    let output = oxo_call_no_license()
        .env(
            "OXO_CALL_LICENSE",
            "/tmp/nonexistent-license-enforcement-test.json",
        )
        .arg("--version")
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "--version should work without a license"
    );
}

#[test]
fn test_license_command_allowed_without_license() {
    // `oxo-call license` must work even without a license
    let output = oxo_call_no_license()
        .env(
            "OXO_CALL_LICENSE",
            "/tmp/nonexistent-license-enforcement-test.json",
        )
        .arg("license")
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "'license' command should work without a license file"
    );
}

#[test]
fn test_license_verify_with_valid_fixture() {
    // Verify command with the test fixture should succeed
    let output = oxo_call()
        .args(["license", "verify"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "license verify with valid fixture should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("valid") || stdout.contains("✓"),
        "Expected valid license output, got: {stdout}"
    );
    assert!(
        stdout.contains("academic"),
        "Expected license type in output, got: {stdout}"
    );
}

// ─── Docs subcommand management tests (add/remove/update/list) ────────────────

#[test]
fn test_docs_add_real_tool() {
    // 'ls' is always available
    let output = oxo_call()
        .args(["docs", "add", "ls"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Indexed") || stdout.contains("ls"));
}

#[test]
fn test_docs_list_empty_or_filled() {
    let output = oxo_call()
        .args(["docs", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
}

#[test]
fn test_docs_add_and_list() {
    // Index a tool via 'docs add'
    let add_output = oxo_call()
        .args(["docs", "add", "date"])
        .output()
        .expect("failed to run oxo-call");
    assert!(add_output.status.success(), "docs add date should succeed");

    // List must show the tool that was just indexed
    let output = oxo_call()
        .args(["docs", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("date"),
        "Expected 'date' in docs list output, got: {stdout}"
    );
}

#[test]
fn test_docs_remove_nonexistent() {
    let output = oxo_call()
        .args(["docs", "remove", "nonexistent_tool_xyz_docs"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("not in the index"),
        "Expected error for missing tool, got: {stderr}"
    );
}

#[test]
fn test_docs_add_from_file() {
    use std::io::Write;
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("mytool.md");
    let mut f = std::fs::File::create(&file_path).expect("create file");
    writeln!(
        f,
        "# mytool\n\nUsage: mytool [options]\n\nOptions:\n  --help   Show this help"
    )
    .expect("write");
    drop(f);

    let output = oxo_call()
        .args([
            "docs",
            "add",
            "mytool",
            "--file",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "docs add --file should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Indexed") || stdout.contains("mytool"),
        "Expected success output, got: {stdout}"
    );
}

#[test]
fn test_docs_add_from_dir() {
    use std::io::Write;
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("usage.md");
    let mut f = std::fs::File::create(&file_path).expect("create file");
    writeln!(f, "# dirtool\n\nUsage: dirtool [options]\n\nOptions:\n  --help   Show help\n  --version  Show version").expect("write");
    drop(f);

    let output = oxo_call()
        .args([
            "docs",
            "add",
            "dirtool",
            "--dir",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "docs add --dir should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Indexed") || stdout.contains("dirtool"),
        "Expected success output, got: {stdout}"
    );
}

#[test]
fn test_docs_add_unsupported_file_type_fails() {
    use std::io::Write;
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("manual.pdf");
    let mut f = std::fs::File::create(&file_path).expect("create file");
    writeln!(f, "fake pdf content").expect("write");
    drop(f);

    let output = oxo_call()
        .args([
            "docs",
            "add",
            "sometool",
            "--file",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !output.status.success(),
        "docs add --file with unsupported type should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("Unsupported"),
        "Expected unsupported file type error, got: {stderr}"
    );
}

#[test]
fn test_docs_add_path_traversal_fails() {
    let output = oxo_call()
        .args(["docs", "add", "../etc/passwd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("path"),
        "Expected error for path traversal tool name, got: {stderr}"
    );
}

#[test]
fn test_docs_fetch_non_http_url_via_add_fails() {
    let output = oxo_call()
        .args(["docs", "add", "sometool", "--url", "file:///etc/passwd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("https"),
        "Expected error for non-http URL, got: {stderr}"
    );
}

// ─── Workflow command tests ───────────────────────────────────────────────────

#[test]
fn test_workflow_help_output() {
    let output = oxo_call()
        .args(["workflow", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("generate") || stdout.contains("Generate"),
        "Expected generate subcommand in workflow help, got: {stdout}"
    );
    assert!(
        stdout.contains("list") || stdout.contains("List"),
        "Expected list subcommand in workflow help, got: {stdout}"
    );
    assert!(
        stdout.contains("show") || stdout.contains("Show"),
        "Expected show subcommand in workflow help, got: {stdout}"
    );
}

#[test]
fn test_workflow_list_shows_builtin_templates() {
    let output = oxo_call()
        .args(["workflow", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rnaseq"),
        "Expected rnaseq template, got: {stdout}"
    );
    assert!(
        stdout.contains("wgs"),
        "Expected wgs template, got: {stdout}"
    );
    assert!(
        stdout.contains("atacseq"),
        "Expected atacseq template, got: {stdout}"
    );
    assert!(
        stdout.contains("metagenomics"),
        "Expected metagenomics template, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_rnaseq_native() {
    let output = oxo_call()
        .args(["workflow", "show", "rnaseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Default format is now native .oxo.toml
    assert!(
        stdout.contains("[workflow]") || stdout.contains("[[step]]"),
        "Expected native TOML syntax in rnaseq template, got: {stdout}"
    );
    assert!(
        stdout.contains("star") || stdout.contains("STAR"),
        "Expected STAR alignment step, got: {stdout}"
    );
    assert!(
        stdout.contains("fastp"),
        "Expected fastp QC step, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_rnaseq_snakemake() {
    let output = oxo_call()
        .args(["workflow", "show", "rnaseq", "--engine", "snakemake"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rule all") || stdout.contains("configfile"),
        "Expected Snakemake syntax in rnaseq template, got: {stdout}"
    );
    assert!(
        stdout.contains("STAR") || stdout.contains("star"),
        "Expected STAR alignment step, got: {stdout}"
    );
    assert!(
        stdout.contains("fastp"),
        "Expected fastp QC step, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_wgs_nextflow() {
    let output = oxo_call()
        .args(["workflow", "show", "wgs", "--engine", "nextflow"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nextflow.enable.dsl"),
        "Expected Nextflow DSL2 syntax, got: {stdout}"
    );
    assert!(
        stdout.contains("bwa-mem2") || stdout.contains("BWA_MEM2"),
        "Expected BWA-MEM2 alignment step, got: {stdout}"
    );
    assert!(
        stdout.contains("HaplotypeCaller") || stdout.contains("HAPLOTYPE_CALLER"),
        "Expected GATK HaplotypeCaller step, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_unknown_template() {
    let output = oxo_call()
        .args(["workflow", "show", "nonexistent_workflow_xyz"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !output.status.success(),
        "Expected non-zero exit for unknown template"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown") || stderr.contains("error"),
        "Expected error message for unknown template, got: {stderr}"
    );
}

#[test]
fn test_workflow_show_atacseq_snakemake() {
    let output = oxo_call()
        .args(["workflow", "show", "atacseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("bowtie2") || stdout.contains("BOWTIE2"),
        "Expected Bowtie2 alignment in ATAC-seq workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("macs3") || stdout.contains("MACS3"),
        "Expected MACS3 peak calling in ATAC-seq workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_metagenomics_snakemake() {
    let output = oxo_call()
        .args(["workflow", "show", "metagenomics"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("kraken2") || stdout.contains("KRAKEN2"),
        "Expected Kraken2 classification in metagenomics workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("bracken") || stdout.contains("BRACKEN"),
        "Expected Bracken abundance estimation in metagenomics workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_generate_requires_llm_token() {
    // Without a configured LLM token, generate should fail gracefully
    let output = oxo_call()
        .args(["workflow", "generate", "RNA-seq pipeline for human samples"])
        .env_remove("OXO_CALL_LLM_API_TOKEN")
        .output()
        .expect("failed to run oxo-call");
    // Should fail because no API token is configured (in CI there is no real token)
    assert!(
        !output.status.success() || {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("WORKFLOW:") || stdout.contains("workflow")
        },
        "Expected either a failure or a valid workflow output"
    );
}

#[test]
fn test_help_includes_workflow() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("workflow") || stdout.contains("Workflow"),
        "Expected workflow subcommand in help, got: {stdout}"
    );
}

#[test]
fn test_workflow_dry_run_builtin_template() {
    let output = oxo_call()
        .args(["workflow", "dry-run", "rnaseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fastp") || stdout.contains("▷"),
        "Expected dry-run preview output, got: {stdout}"
    );
    assert!(
        stdout.contains("dry-run"),
        "Expected dry-run label in output, got: {stdout}"
    );
}

#[test]
fn test_workflow_dry_run_from_file() {
    let tmp = tempfile::NamedTempFile::with_suffix(".toml").expect("tempfile");
    let toml_content = r#"
[workflow]
name = "test"
description = "test workflow"

[wildcards]
sample = ["s1"]

[params]
threads = "4"

[[step]]
name = "echo_step"
cmd = "echo hello {sample}"
outputs = ["out_{sample}.txt"]
"#;
    std::fs::write(tmp.path(), toml_content).expect("write");
    let path = tmp.path().to_str().unwrap().to_string();

    let output = oxo_call()
        .args(["workflow", "dry-run", &path])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("echo_step") || stdout.contains("echo"),
        "Expected step name in dry-run output, got: {stdout}"
    );
    assert!(
        stdout.contains("s1"),
        "Expected wildcard expansion in dry-run output, got: {stdout}"
    );
}

#[test]
fn test_workflow_export_to_snakemake() {
    let output = oxo_call()
        .args(["workflow", "export", "rnaseq", "--to", "snakemake"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rule") || stdout.contains("configfile"),
        "Expected Snakemake syntax in export output, got: {stdout}"
    );
}

#[test]
fn test_workflow_export_to_nextflow() {
    let output = oxo_call()
        .args(["workflow", "export", "metagenomics", "--to", "nextflow"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nextflow.enable.dsl") || stdout.contains("process"),
        "Expected Nextflow syntax in export output, got: {stdout}"
    );
}

#[test]
fn test_workflow_run_from_file() {
    use std::io::Write;
    let dir = tempfile::tempdir().expect("tempdir");
    let wf_path = dir.path().join("test.toml");
    let out_path = dir.path().join("out.txt");

    std::fs::write(
        &wf_path,
        format!(
            r#"
[workflow]
name = "test"
description = "trivial test workflow"

[[step]]
name = "echo_hello"
cmd = "echo hello > {out}"
outputs = ["{out}"]
"#,
            out = out_path.display()
        ),
    )
    .expect("write workflow");

    let output = oxo_call()
        .args(["workflow", "run", wf_path.to_str().unwrap()])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "workflow run should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_path.exists(), "Expected output file to be created");
}

// ─── New omics template tests ─────────────────────────────────────────────────

#[test]
fn test_workflow_list_shows_new_templates() {
    let output = oxo_call()
        .args(["workflow", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // All original templates must still be present.
    assert!(stdout.contains("rnaseq"), "rnaseq missing from list");
    assert!(stdout.contains("wgs"), "wgs missing from list");
    assert!(stdout.contains("atacseq"), "atacseq missing from list");
    assert!(
        stdout.contains("metagenomics"),
        "metagenomics missing from list"
    );
    // New templates.
    assert!(stdout.contains("chipseq"), "chipseq missing from list");
    assert!(stdout.contains("methylseq"), "methylseq missing from list");
    assert!(stdout.contains("scrnaseq"), "scrnaseq missing from list");
    assert!(
        stdout.contains("amplicon16s"),
        "amplicon16s missing from list"
    );
    assert!(stdout.contains("longreads"), "longreads missing from list");
}

#[test]
fn test_workflow_show_chipseq_native() {
    let output = oxo_call()
        .args(["workflow", "show", "chipseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("macs3") || stdout.contains("MACS3"),
        "Expected MACS3 peak calling in ChIP-seq workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("bamCoverage") || stdout.contains("bigwig"),
        "Expected bigWig generation in ChIP-seq workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_methylseq_native() {
    let output = oxo_call()
        .args(["workflow", "show", "methylseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("bismark"),
        "Expected Bismark alignment in methylseq workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("methylation_extract") || stdout.contains("bismark_methylation_extractor"),
        "Expected methylation extraction in methylseq workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_scrnaseq_native() {
    let output = oxo_call()
        .args(["workflow", "show", "scrnaseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("STARsolo") || stdout.contains("starsolo") || stdout.contains("STAR"),
        "Expected STARsolo in scrnaseq workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("CB_UMI_Simple") || stdout.contains("soloType"),
        "Expected 10x Chromium STARsolo params in scrnaseq workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_amplicon16s_native() {
    let output = oxo_call()
        .args(["workflow", "show", "amplicon16s"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cutadapt"),
        "Expected cutadapt primer trimming in amplicon16s workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("dada2") || stdout.contains("DADA2"),
        "Expected DADA2 in amplicon16s workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_show_longreads_native() {
    let output = oxo_call()
        .args(["workflow", "show", "longreads"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("flye") || stdout.contains("Flye"),
        "Expected Flye assembly in longreads workflow, got: {stdout}"
    );
    assert!(
        stdout.contains("medaka"),
        "Expected Medaka polishing in longreads workflow, got: {stdout}"
    );
}

#[test]
fn test_workflow_export_chipseq_snakemake() {
    let output = oxo_call()
        .args(["workflow", "export", "chipseq", "--to", "snakemake"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rule all") || stdout.contains("configfile"),
        "Expected Snakemake structure in chipseq export, got: {stdout}"
    );
}

#[test]
fn test_workflow_export_scrnaseq_nextflow() {
    let output = oxo_call()
        .args(["workflow", "export", "scrnaseq", "--to", "nextflow"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nextflow.enable.dsl") || stdout.contains("process"),
        "Expected Nextflow DSL2 structure in scrnaseq export, got: {stdout}"
    );
}

// ─── workflow infer tests ─────────────────────────────────────────────────────

#[test]
fn test_workflow_infer_help() {
    let output = oxo_call()
        .args(["workflow", "infer", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("data") || stdout.contains("task"),
        "Expected --data and --task in infer help, got: {stdout}"
    );
}

#[test]
fn test_workflow_infer_missing_data_dir_fails() {
    let output = oxo_call()
        .args([
            "workflow",
            "infer",
            "RNA-seq analysis",
            "--data",
            "/nonexistent/path/xyz",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !output.status.success(),
        "infer with nonexistent data dir should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("not exist"),
        "Expected error message for missing data dir, got: {stderr}"
    );
}

#[test]
fn test_workflow_infer_scans_data_dir() {
    // Create a temp directory with some fake FASTQ files.
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let data_dir = tmp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Create fake paired-end FASTQ files.
    for sample in &["ctrl_rep1", "treat_rep1", "treat_rep2"] {
        std::fs::write(data_dir.join(format!("{sample}_R1.fastq.gz")), b"fake").unwrap();
        std::fs::write(data_dir.join(format!("{sample}_R2.fastq.gz")), b"fake").unwrap();
    }

    // Run infer — it will scan the directory and try to call LLM.
    // Without a real LLM token it should fail after printing data context.
    let output = oxo_call()
        .args([
            "workflow",
            "infer",
            "ChIP-seq analysis for H3K27ac mark",
            "--data",
            data_dir.to_str().unwrap(),
        ])
        .env_remove("OXO_CALL_LLM_API_TOKEN")
        .output()
        .expect("failed to run oxo-call");

    // Whether it succeeds or fails depends on the LLM token.
    // Either way, we expect to see the data scan output before the LLM call.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // Should print at least the data directory scan summary.
    assert!(
        combined.contains("Scanning") || combined.contains("sample") || combined.contains("error"),
        "Expected scan output or error, got stdout={stdout} stderr={stderr}"
    );
}

// ─── workflow verify tests ─────────────────────────────────────────────────────

#[test]
fn test_workflow_verify_valid_builtin_template() {
    // A built-in template should always be valid.
    let output = oxo_call()
        .args(["workflow", "verify", "rnaseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "verify should succeed for rnaseq built-in template"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("valid") || stdout.contains("No issues"),
        "Expected 'valid' or 'No issues' in verify output, got: {stdout}"
    );
}

#[test]
fn test_workflow_verify_valid_file() {
    let tmp = tempfile::NamedTempFile::with_suffix(".toml").expect("tempfile");
    let toml_content = r#"
[workflow]
name = "test"
description = "test workflow"

[wildcards]
sample = ["s1", "s2"]

[params]
threads = "4"

[[step]]
name = "qc"
cmd = "fastp --in1 data/{sample}_R1.fq.gz --json qc/{sample}.json"
inputs = ["data/{sample}_R1.fq.gz"]
outputs = ["qc/{sample}.json"]

[[step]]
name = "align"
depends_on = ["qc"]
cmd = "bwa mem -t {params.threads} ref.fa qc/{sample}.json > {sample}.sam"
inputs = ["qc/{sample}.json"]
outputs = ["{sample}.sam"]
"#;
    std::fs::write(tmp.path(), toml_content).expect("write");
    let output = oxo_call()
        .args(["workflow", "verify", tmp.path().to_str().unwrap()])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "verify should succeed for valid workflow file"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("valid") || stdout.contains("No issues"),
        "Expected valid output, got: {stdout}"
    );
}

#[test]
fn test_workflow_verify_unknown_dep_fails() {
    let tmp = tempfile::NamedTempFile::with_suffix(".toml").expect("tempfile");
    let toml_content = r#"
[workflow]
name = "broken"

[[step]]
name = "step_b"
depends_on = ["nonexistent_step"]
cmd = "echo b"
"#;
    std::fs::write(tmp.path(), toml_content).expect("write");
    let output = oxo_call()
        .args(["workflow", "verify", tmp.path().to_str().unwrap()])
        .output()
        .expect("failed to run oxo-call");
    // Should exit with error code since there's an unknown dep.
    assert!(
        !output.status.success(),
        "verify should fail for workflow with unknown dependency"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nonexistent_step") || stdout.contains("unknown"),
        "Expected error about unknown step, got: {stdout}"
    );
}

#[test]
fn test_workflow_verify_all_builtin_templates() {
    // All built-in templates must pass verification.
    for name in &[
        "rnaseq",
        "wgs",
        "atacseq",
        "chipseq",
        "metagenomics",
        "amplicon16s",
        "scrnaseq",
        "longreads",
        "methylseq",
    ] {
        let output = oxo_call()
            .args(["workflow", "verify", name])
            .output()
            .expect("failed to run oxo-call");
        assert!(
            output.status.success(),
            "verify failed for built-in template '{name}': {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn test_workflow_verify_alias_check() {
    // 'check' is a visible alias for 'verify'.
    let output = oxo_call()
        .args(["workflow", "check", "rnaseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "'workflow check' alias should work"
    );
}

// ─── workflow fmt tests ────────────────────────────────────────────────────────

#[test]
fn test_workflow_fmt_stdout_builtin() {
    // fmt --stdout on a built-in template should print canonical TOML.
    let output = oxo_call()
        .args(["workflow", "fmt", "rnaseq", "--stdout"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[workflow]"),
        "Expected [workflow] section in formatted output, got: {stdout}"
    );
    assert!(
        stdout.contains("[[step]]"),
        "Expected [[step]] in formatted output, got: {stdout}"
    );
    assert!(
        stdout.contains("name       ="),
        "Expected canonical name alignment in formatted output, got: {stdout}"
    );
}

#[test]
fn test_workflow_fmt_inplace() {
    // fmt without --stdout should reformat the file in-place.
    let tmp = tempfile::NamedTempFile::with_suffix(".toml").expect("tempfile");
    // Write deliberately un-aligned TOML.
    let input = r#"
[workflow]
name="my-pipeline"
description="test"

[wildcards]
sample=["s1","s2"]

[params]
threads="8"

[[step]]
name="qc"
cmd="echo {sample}"
outputs=["out/{sample}.txt"]
"#;
    std::fs::write(tmp.path(), input).expect("write");

    let output = oxo_call()
        .args(["workflow", "fmt", tmp.path().to_str().unwrap()])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "fmt should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Formatted"),
        "Expected 'Formatted' confirmation, got: {stdout}"
    );

    // The file should now have canonical formatting.
    let formatted = std::fs::read_to_string(tmp.path()).expect("read formatted");
    assert!(
        formatted.contains("[workflow]"),
        "Formatted file should contain [workflow], got: {formatted}"
    );
    assert!(
        formatted.contains("name       ="),
        "Formatted file should use aligned name field, got: {formatted}"
    );
}

#[test]
fn test_workflow_format_alias() {
    // 'format' is a visible alias for 'fmt'.
    let output = oxo_call()
        .args(["workflow", "format", "rnaseq", "--stdout"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "'workflow format' alias should work"
    );
}

// ─── workflow vis tests ────────────────────────────────────────────────────────

#[test]
fn test_workflow_vis_builtin_template() {
    let output = oxo_call()
        .args(["workflow", "vis", "rnaseq"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Phase"),
        "Expected 'Phase' in vis output, got: {stdout}"
    );
    assert!(
        stdout.contains("fastp"),
        "Expected 'fastp' step in vis output, got: {stdout}"
    );
    assert!(
        stdout.contains("multiqc") || stdout.contains("gather"),
        "Expected multiqc gather step in vis output, got: {stdout}"
    );
    assert!(
        stdout.contains("Depends on") || stdout.contains("Step details"),
        "Expected step details table in vis output, got: {stdout}"
    );
}

#[test]
fn test_workflow_vis_dag_alias() {
    // 'dag' is a visible alias for 'vis'.
    let output = oxo_call()
        .args(["workflow", "dag", "wgs"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "'workflow dag' alias should work: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Phase"),
        "Expected phase diagram in dag output, got: {stdout}"
    );
}

#[test]
fn test_workflow_vis_from_file() {
    let tmp = tempfile::NamedTempFile::with_suffix(".toml").expect("tempfile");
    let toml_content = r#"
[workflow]
name = "simple"
description = "simple test pipeline"

[wildcards]
sample = ["s1", "s2", "s3"]

[[step]]
name = "qc"
cmd = "fastp -i {sample}.fq"

[[step]]
name = "multiqc"
gather = true
depends_on = ["qc"]
cmd = "multiqc qc/"
outputs = ["multiqc_report.html"]

[[step]]
name = "align"
depends_on = ["qc"]
cmd = "bwa mem ref.fa {sample}.fq > {sample}.bam"
"#;
    std::fs::write(tmp.path(), toml_content).expect("write");
    let output = oxo_call()
        .args(["workflow", "vis", tmp.path().to_str().unwrap()])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Phase"),
        "Expected phase output, got: {stdout}"
    );
    // multiqc and align should be in the same phase (both depend on qc).
    assert!(
        stdout.contains("multiqc") || stdout.contains("align"),
        "Expected step names in vis output, got: {stdout}"
    );
    assert!(
        stdout.contains("simple"),
        "Expected workflow name in vis output, got: {stdout}"
    );
}

#[test]
fn test_workflow_vis_shows_wildcards() {
    let output = oxo_call()
        .args(["workflow", "vis", "metagenomics"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sample") || stdout.contains("Wildcard"),
        "Expected wildcard info in vis output, got: {stdout}"
    );
}

#[test]
fn test_workflow_vis_unknown_template_fails() {
    let output = oxo_call()
        .args(["workflow", "vis", "nonexistent_workflow"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !output.status.success(),
        "vis should fail for unknown template"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a file") || stderr.contains("error"),
        "Expected error message, got: {stderr}"
    );
}

#[test]
fn test_workflow_help_shows_new_commands() {
    let output = oxo_call()
        .args(["workflow", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("verify") || stdout.contains("Verify"),
        "Expected 'verify' in workflow help, got: {stdout}"
    );
    assert!(
        stdout.contains("fmt") || stdout.contains("format"),
        "Expected 'fmt' in workflow help, got: {stdout}"
    );
    assert!(
        stdout.contains("vis") || stdout.contains("dag"),
        "Expected 'vis' in workflow help, got: {stdout}"
    );
}

// ─── Shell completion tests ──────────────────────────────────────────────────

#[test]
fn test_completion_bash() {
    let output = oxo_call()
        .args(["completion", "bash"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("_oxo-call"),
        "Expected bash completion function name in output"
    );
    assert!(
        stdout.contains("COMPREPLY"),
        "Expected COMPREPLY in bash completion output"
    );
}

#[test]
fn test_completion_zsh() {
    let output = oxo_call()
        .args(["completion", "zsh"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("#compdef oxo-call"),
        "Expected zsh compdef header"
    );
}

#[test]
fn test_completion_fish() {
    let output = oxo_call()
        .args(["completion", "fish"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("oxo_call") || stdout.contains("oxo-call"),
        "Expected oxo-call references in fish completion"
    );
}

#[test]
fn test_completion_works_without_license() {
    let output = oxo_call_no_license()
        .args(["completion", "bash"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "completion should work without a license"
    );
}

// ─── New flag tests ──────────────────────────────────────────────────────────

#[test]
fn test_help_mentions_completion_command() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("completion"),
        "Expected 'completion' in top-level help"
    );
}

#[test]
fn test_help_mentions_verbose_flag() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--verbose"),
        "Expected '--verbose' in top-level help"
    );
}

#[test]
fn test_run_help_mentions_new_flags() {
    let output = oxo_call()
        .args(["run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--model"), "Expected '--model' in run help");
    assert!(
        stdout.contains("--no-cache"),
        "Expected '--no-cache' in run help"
    );
    assert!(stdout.contains("--json"), "Expected '--json' in run help");
    assert!(
        stdout.contains("EXAMPLES"),
        "Expected 'EXAMPLES' in run help"
    );
}

#[test]
fn test_dry_run_help_mentions_new_flags() {
    let output = oxo_call()
        .args(["dry-run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--model"),
        "Expected '--model' in dry-run help"
    );
    assert!(
        stdout.contains("--no-cache"),
        "Expected '--no-cache' in dry-run help"
    );
    assert!(
        stdout.contains("--json"),
        "Expected '--json' in dry-run help"
    );
    assert!(
        stdout.contains("EXAMPLES"),
        "Expected 'EXAMPLES' in dry-run help"
    );
}
