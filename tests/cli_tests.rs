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
    // Index a tool
    let _ = oxo_call().args(["index", "add", "cat"]).output();

    // List should contain the tool
    let output = oxo_call()
        .args(["index", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Either we see "cat" or the list header
    assert!(stdout.contains("cat") || stdout.contains("Tool"));
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
        .args(["--license", "/tmp/nonexistent-oxo-license-12345.json", "license", "verify"])
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
        .env("OXO_CALL_LICENSE", "/tmp/nonexistent-license-enforcement-test.json")
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
        .env("OXO_CALL_LICENSE", "/tmp/nonexistent-license-enforcement-test.json")
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
        .env("OXO_CALL_LICENSE", "/tmp/nonexistent-license-enforcement-test.json")
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
        .env("OXO_CALL_LICENSE", "/tmp/nonexistent-license-enforcement-test.json")
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
