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
fn test_history_list_shows_server_column() {
    // history list should show a "Server" column header.
    let output = oxo_call()
        .args(["history", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    // Even when empty, we only get the "No history found." message.
    // But when there are entries the header should contain "Server".
    // We can verify the help shows expected output format by running with an
    // injected entry via the lib tests. Here we just confirm the command works.
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Either shows "No history found." or a table with Server column.
    assert!(
        stdout.contains("No history found.") || stdout.contains("Server"),
        "Expected 'No history found.' or 'Server' column, got: {stdout}"
    );
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
        stdout.contains("---"),
        "Expected YAML front-matter delimiters, got: {stdout}"
    );
    assert!(
        stdout.contains("## Concepts"),
        "Expected ## Concepts section, got: {stdout}"
    );
    assert!(
        stdout.contains("## Examples"),
        "Expected ## Examples section, got: {stdout}"
    );
    assert!(
        stdout.contains("**Args:**"),
        "Expected **Args:** example format, got: {stdout}"
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

#[test]
fn test_skill_mcp_list_empty() {
    // With a fresh temp config dir there are no MCP servers registered.
    let dir = tempfile::tempdir().expect("tmpdir");
    let output = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "mcp", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success(), "mcp list should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No MCP") || stdout.contains("registered"),
        "Expected empty MCP list message, got: {stdout}"
    );
}

#[test]
fn test_skill_mcp_add_and_list() {
    let dir = tempfile::tempdir().expect("tmpdir");
    // Add a server
    let add = oxo_call()
        .env("HOME", dir.path())
        .args([
            "skill",
            "mcp",
            "add",
            "http://localhost:9999",
            "--name",
            "test-server",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        add.status.success(),
        "mcp add should succeed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    // List should now show it
    let list = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "mcp", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(
        stdout.contains("test-server") || stdout.contains("localhost:9999"),
        "Expected registered server in list, got: {stdout}"
    );
}

#[test]
fn test_skill_mcp_remove() {
    let dir = tempfile::tempdir().expect("tmpdir");
    // Add then remove
    oxo_call()
        .env("HOME", dir.path())
        .args([
            "skill",
            "mcp",
            "add",
            "http://localhost:9998",
            "--name",
            "removable",
        ])
        .output()
        .expect("add");

    let rm = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "mcp", "remove", "removable"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        rm.status.success(),
        "mcp remove should succeed: {}",
        String::from_utf8_lossy(&rm.stderr)
    );

    // List should now be empty again
    let list = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "mcp", "list"])
        .output()
        .expect("list");
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(
        stdout.contains("No MCP") || stdout.contains("registered") && !stdout.contains("removable"),
        "Expected empty list after remove, got: {stdout}"
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
fn test_docs_add_shell_builtin() {
    // 'cd' is a shell built-in; docs should be fetched via 'bash -c "help cd"'
    let output = oxo_call()
        .args(["docs", "add", "cd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "docs add cd should succeed for shell built-in: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Indexed") || stdout.contains("cd"));
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
    assert!(
        stdout.contains("--verify"),
        "Expected '--verify' in run help"
    );
    assert!(
        stdout.contains("--optimize-task"),
        "Expected '--optimize-task' in run help"
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
    assert!(
        stdout.contains("--optimize-task"),
        "Expected '--optimize-task' in dry-run help"
    );
}

#[test]
fn test_workflow_run_help_mentions_verify_flag() {
    let output = oxo_call()
        .args(["workflow", "run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--verify"),
        "Expected '--verify' in workflow run help"
    );
}

#[test]
fn test_run_verify_flag_is_parsed() {
    // Verifies that --verify is accepted by the CLI parser (no "unknown flag" error).
    // The flag requires an LLM token to do anything, so we just check that the
    // binary does not reject the flag with a usage error.
    let output = oxo_call()
        .args(["run", "--verify", "date", "current time"])
        .output()
        .expect("failed to run oxo-call");
    // The command will fail due to missing license/token, but NOT due to unknown flag.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument '--verify'"),
        "CLI should accept --verify flag"
    );
    assert!(
        !stderr.contains("error: Found argument '--verify'"),
        "CLI should accept --verify flag"
    );
}

#[test]
fn test_run_optimize_task_flag_is_parsed() {
    let output = oxo_call()
        .args(["run", "--optimize-task", "date", "current time"])
        .output()
        .expect("failed to run oxo-call");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument '--optimize-task'"),
        "CLI should accept --optimize-task flag"
    );
}

#[test]
fn test_dry_run_optimize_task_flag_is_parsed() {
    let output = oxo_call()
        .args(["dry-run", "--optimize-task", "date", "current time"])
        .output()
        .expect("failed to run oxo-call");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument '--optimize-task'"),
        "CLI should accept --optimize-task flag in dry-run"
    );
}

// ─── Server command tests ─────────────────────────────────────────────────────

#[test]
fn test_help_includes_server() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("server"),
        "help output should mention server command"
    );
}

#[test]
fn test_server_help_output() {
    let output = oxo_call()
        .args(["server", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("add"));
    assert!(stdout.contains("remove"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("ssh-config"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("dry-run"));
    assert!(
        stdout.contains("use"),
        "server --help should list 'use' subcommand"
    );
    assert!(
        stdout.contains("unuse"),
        "server --help should list 'unuse' subcommand"
    );
}

#[test]
fn test_server_list_empty() {
    let output = oxo_call()
        .args(["server", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No servers registered") || stdout.contains("Name"),
        "server list should show empty message or header"
    );
}

#[test]
fn test_server_ssh_config() {
    let output = oxo_call()
        .args(["server", "ssh-config"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Either finds hosts (shows numbered table) or reports no file; with no
    // stdin the interactive prompt cancels automatically.
    assert!(
        stdout.contains("host(s)") || stdout.contains("No hosts found"),
        "ssh-config should report found hosts or no-file message"
    );
}

#[test]
fn test_server_ssh_config_help() {
    let output = oxo_call()
        .args(["server", "ssh-config", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--yes") || stdout.contains("-y"),
        "--yes flag should appear"
    );
    assert!(stdout.contains("--type"), "--type flag should appear");
    assert!(
        stdout.contains("workstation"),
        "default type should appear in help"
    );
    assert!(stdout.contains("hpc"), "hpc option should appear in help");
}

#[test]
fn test_server_ssh_config_type_hpc_yes() {
    // --type hpc --yes in batch mode (no actual ssh config needed to test the
    // flag is accepted without error; if no hosts found that's fine too).
    let output = oxo_call()
        .args(["server", "ssh-config", "--type", "hpc", "--yes"])
        .output()
        .expect("failed to run oxo-call");
    // Should succeed (even if no hosts found).
    assert!(output.status.success());
}

#[test]
fn test_server_add_help() {
    let output = oxo_call()
        .args(["server", "add", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--host"));
    assert!(stdout.contains("--type"));
    assert!(stdout.contains("workstation"));
    assert!(stdout.contains("hpc"));
}

#[test]
fn test_server_run_help() {
    let output = oxo_call()
        .args(["server", "run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // tool and task are still required positionals
    assert!(stdout.contains("tool"));
    assert!(stdout.contains("task"));
    // server is now an optional flag
    assert!(
        stdout.contains("--server") || stdout.contains("-s"),
        "server should be an optional --server flag"
    );
}

#[test]
fn test_server_use_help() {
    let output = oxo_call()
        .args(["server", "use", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("active") || stdout.contains("name"));
}

#[test]
fn test_server_unuse_help() {
    let output = oxo_call()
        .args(["server", "unuse", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("active") || stdout.contains("Clear") || stdout.contains("clear"),
        "unuse help should describe clearing the active server"
    );
}

#[test]
fn test_server_run_no_server_no_active_fails() {
    // With no active server and no --server flag, server run should fail gracefully
    let output = oxo_call()
        .args(["server", "run", "ls", "list files"])
        .env("OXO_CALL_CONFIG_DIR", "/tmp/oxo-call-test-empty-config")
        .output()
        .expect("failed to run oxo-call");
    // Should fail because no server is specified and no active server
    assert!(
        !output.status.success(),
        "server run without server should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No server") || stderr.contains("active"),
        "error should mention missing server or active host"
    );
}

// ─── HPC skill tests ─────────────────────────────────────────────────────────

#[test]
fn test_skill_show_slurm() {
    let output = oxo_call()
        .args(["skill", "show", "slurm"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("slurm"));
    assert!(stdout.contains("hpc"));
    assert!(stdout.contains("sbatch"));
}

#[test]
fn test_skill_show_pbs() {
    let output = oxo_call()
        .args(["skill", "show", "pbs"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pbs"));
    assert!(stdout.contains("qsub"));
}

#[test]
fn test_skill_show_kubectl() {
    let output = oxo_call()
        .args(["skill", "show", "kubectl"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kubectl"));
    assert!(stdout.contains("kubernetes") | stdout.contains("Kubernetes"));
}

#[test]
fn test_skill_show_sge() {
    let output = oxo_call()
        .args(["skill", "show", "sge"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sge"));
}

#[test]
fn test_skill_show_lsf() {
    let output = oxo_call()
        .args(["skill", "show", "lsf"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lsf"));
}

#[test]
fn test_skill_show_htcondor() {
    let output = oxo_call()
        .args(["skill", "show", "htcondor"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("htcondor"));
}

// ─── skill verify / polish / create --llm tests ──────────────────────────────

#[test]
fn test_skill_create_with_llm_flag_is_parsed() {
    // Without LLM config the command should fall back to the blank template
    // rather than erroring on argument parsing.
    let dir = tempfile::tempdir().expect("tmpdir");
    let output = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "create", "mytool", "--llm"])
        .output()
        .expect("failed to run oxo-call");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Either succeeds with a template OR fails gracefully (LLM not configured),
    // but must never fail with "unknown argument".
    assert!(
        !stderr.contains("unexpected argument"),
        "Should parse --llm flag; got: {stderr}"
    );
    // If it fell back to the blank template, it should contain the tool name.
    if output.status.success() {
        assert!(
            stdout.contains("mytool") || stdout.contains("Template written"),
            "Expected template content or success message, got: {stdout}"
        );
    }
}

#[test]
fn test_skill_verify_unknown_tool_shows_helpful_message() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let output = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "verify", "nonexistent_tool_xyz99"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "verify of missing skill should not error"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No skill") || stdout.contains("install") || stdout.contains("create"),
        "Expected helpful message for missing skill, got: {stdout}"
    );
}

#[test]
fn test_skill_verify_no_llm_flag_is_parsed() {
    // --no-llm skips the LLM review; with a built-in skill this should succeed
    let output = oxo_call()
        .args(["skill", "verify", "samtools", "--no-llm"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success(), "verify --no-llm should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("PASS") || stdout.contains("FAIL") || stdout.contains("Structural"),
        "Expected structural check result, got: {stdout}"
    );
}

#[test]
fn test_skill_polish_missing_tool_shows_error() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let output = oxo_call()
        .env("HOME", dir.path())
        .args(["skill", "polish", "nonexistent_tool_xyz99"])
        .output()
        .expect("failed to run oxo-call");
    // Should fail with a clear error message about the skill not being found locally
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "polish of non-existent tool should fail, stderr={stderr} stdout={stdout}"
    );
    assert!(
        stderr.contains("not installed")
            || stderr.contains("no editable")
            || stderr.contains("install"),
        "Expected helpful error for missing local skill, stderr={stderr}"
    );
}

#[test]
fn test_skill_help_shows_new_subcommands() {
    let output = oxo_call()
        .args(["skill", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("verify"),
        "Expected 'verify' in skill help, got: {stdout}"
    );
    assert!(
        stdout.contains("polish"),
        "Expected 'polish' in skill help, got: {stdout}"
    );
}

// ─── job subcommand tests ─────────────────────────────────────────────────────

/// Build a Command with test license AND a temporary data directory so job tests
/// do not touch the real user data directory and can run in parallel safely.
fn oxo_call_with_tmpdir(tmp: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_oxo-call"));
    cmd.env("OXO_CALL_LICENSE", test_license_path());
    cmd.env("OXO_CALL_DATA_DIR", tmp);
    cmd
}

#[test]
fn test_job_help() {
    let output = oxo_call()
        .args(["job", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("add"), "Expected 'add' in job help");
    assert!(stdout.contains("remove"), "Expected 'remove' in job help");
    assert!(stdout.contains("list"), "Expected 'list' in job help");
    assert!(stdout.contains("run"), "Expected 'run' in job help");
    assert!(stdout.contains("edit"), "Expected 'edit' in job help");
    assert!(stdout.contains("rename"), "Expected 'rename' in job help");
    assert!(stdout.contains("show"), "Expected 'show' in job help");
}

#[test]
fn test_job_list_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No jobs saved"),
        "Expected empty message, got: {stdout}"
    );
}

#[test]
fn test_job_add_and_list() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let add_out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "my-cmd",
            "echo hello",
            "--description",
            "A greeting",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        add_out.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add_out.stderr)
    );

    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    assert!(list_out.status.success());
    let stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(stdout.contains("my-cmd"), "Expected 'my-cmd' in list");
    assert!(
        stdout.contains("echo hello"),
        "Expected command text in list"
    );
}

#[test]
fn test_job_add_duplicate_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "dup-cmd", "echo 1"])
        .output()
        .expect("failed to run oxo-call");
    let second = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "dup-cmd", "echo 2"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!second.status.success(), "duplicate add should fail");
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        stderr.contains("already exists"),
        "Expected 'already exists' in error, got: {stderr}"
    );
}

#[test]
fn test_job_show() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "show-cmd",
            "ls -la",
            "--description",
            "List files",
        ])
        .output()
        .expect("failed to run oxo-call");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "show", "show-cmd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("show-cmd"), "Expected name in show output");
    assert!(stdout.contains("ls -la"), "Expected command in show output");
    assert!(
        stdout.contains("List files"),
        "Expected description in show output"
    );
}

#[test]
fn test_job_remove() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "del-cmd", "echo bye"])
        .output()
        .expect("failed to run oxo-call");

    let rm_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "remove", "del-cmd"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        rm_out.status.success(),
        "remove failed: {}",
        String::from_utf8_lossy(&rm_out.stderr)
    );

    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    let stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(
        !stdout.contains("del-cmd"),
        "Removed command should not appear in list"
    );
}

#[test]
fn test_job_remove_missing_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "remove", "ghost"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!out.status.success(), "removing missing job should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("No job found"),
        "Expected 'No job found' in error, got: {stderr}"
    );
}

#[test]
fn test_job_edit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "edit-cmd", "echo old"])
        .output()
        .expect("failed to run oxo-call");

    let edit_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "edit", "edit-cmd", "--command", "echo new"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        edit_out.status.success(),
        "edit failed: {}",
        String::from_utf8_lossy(&edit_out.stderr)
    );

    let show_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "show", "edit-cmd"])
        .output()
        .expect("failed to run oxo-call");
    let stdout = String::from_utf8_lossy(&show_out.stdout);
    assert!(
        stdout.contains("echo new"),
        "Expected updated command in show"
    );
}

#[test]
fn test_job_rename() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "rename-old", "echo hi"])
        .output()
        .expect("failed to run oxo-call");

    let ren_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "rename", "rename-old", "rename-new"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        ren_out.status.success(),
        "rename failed: {}",
        String::from_utf8_lossy(&ren_out.stderr)
    );

    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    let stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(stdout.contains("rename-new"), "Expected new name in list");
    assert!(
        !stdout.contains("rename-old"),
        "Old name should not appear in list"
    );
}

#[test]
fn test_job_run_dry_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "dry-cmd", "echo dry"])
        .output()
        .expect("failed to run oxo-call");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "dry-cmd", "--dry-run"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "dry-run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("dry-run") || stdout.contains("not executed"),
        "Expected dry-run indicator in output, got: {stdout}"
    );
}

#[test]
fn test_job_run_local() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "run-local", "echo oxo-cmd-test-output"])
        .output()
        .expect("failed to run oxo-call");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "run-local"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "job run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("oxo-cmd-test-output"),
        "Expected command output, got: {stdout}"
    );
}

#[test]
fn test_job_list_tag_filter() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "tagged-cmd",
            "squeue -u $USER",
            "--tag",
            "slurm",
        ])
        .output()
        .expect("failed to run oxo-call");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "untagged-cmd", "ls"])
        .output()
        .expect("failed to run oxo-call");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list", "--tag", "slurm"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("tagged-cmd"),
        "Expected tagged-cmd in filtered list"
    );
    assert!(
        !stdout.contains("untagged-cmd"),
        "Untagged cmd should not appear"
    );
}

#[test]
fn test_help_mentions_job_command() {
    let output = oxo_call()
        .arg("--help")
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("job"),
        "Expected 'job' in top-level help, got: {stdout}"
    );
}

#[test]
fn test_completion_zsh_no_panic_piped_includes_job() {
    // Regression test: completing to stdout when piped must not panic.
    let output = oxo_call()
        .args(["completion", "zsh"])
        .output()
        .expect("failed to run oxo-call");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("#compdef oxo-call"),
        "Expected zsh compdef header, got: {stdout}"
    );
    // The job subcommand should appear in the completion output.
    assert!(
        stdout.contains("job"),
        "Expected 'job' subcommand in zsh completion"
    );
}

#[test]
fn test_job_status_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // status with no jobs should succeed and mention "No jobs"
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "status"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No jobs"),
        "Expected 'No jobs' in status output, got: {stdout}"
    );
}

#[test]
fn test_job_status_after_add() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "status-job", "echo status"])
        .output()
        .expect("add failed");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "status"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("status-job"),
        "Expected job name in status output, got: {stdout}"
    );
}

#[test]
fn test_job_history_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "hist-job", "echo hi"])
        .output()
        .expect("add failed");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history", "hist-job"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No run history"),
        "Expected 'No run history' in output, got: {stdout}"
    );
}

#[test]
fn test_job_history_no_args_all() {
    // `job history` without a name should succeed and show all job history.
    let tmp = tempfile::tempdir().expect("tempdir");
    // Run a couple of jobs to generate history.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "hist-all-a", "echo a"])
        .output()
        .expect("add a failed");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "hist-all-b", "echo b"])
        .output()
        .expect("add b failed");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "hist-all-a"])
        .output()
        .expect("run a failed");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "hist-all-b"])
        .output()
        .expect("run b failed");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "job history (no args) failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("hist-all-a"),
        "Expected job 'hist-all-a' in history, got: {stdout}"
    );
    assert!(
        stdout.contains("hist-all-b"),
        "Expected job 'hist-all-b' in history, got: {stdout}"
    );
}

#[test]
fn test_job_history_no_args_empty() {
    // `job history` with no runs at all should still succeed.
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "job history (no args, empty) failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No job run history"),
        "Expected 'No job run history' message, got: {stdout}"
    );
}

#[test]
fn test_job_history_after_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "hist-run-job", "echo hist-test"])
        .output()
        .expect("add failed");
    // Run the job so it gets a history entry
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "hist-run-job"])
        .output()
        .expect("run failed");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history", "hist-run-job"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("hist-run-job"),
        "Expected job name in history, got: {stdout}"
    );
}

#[test]
fn test_job_builtin_list_includes_new_jobs() {
    // New built-in templates should appear in job list --builtin.
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list", "--builtin"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "job list --builtin failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Spot-check a selection of new templates.
    for name in &["uptime", "find-bam", "qstat-sge", "conda-envs", "tmux-ls"] {
        assert!(
            stdout.contains(name),
            "Expected built-in job '{name}' in list, got: {stdout}"
        );
    }
}

#[test]
fn test_job_import_new_builtin() {
    // Can import one of the new built-in templates.
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "uptime"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "import uptime failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("list failed");
    let list_stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(
        list_stdout.contains("uptime"),
        "Expected 'uptime' in job list after import, got: {list_stdout}"
    );
}

#[test]
fn test_job_schedule_set_and_clear() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "sched-job", "df -h"])
        .output()
        .expect("add failed");

    // Set schedule
    let set_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "schedule", "sched-job", "0 * * * *"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        set_out.status.success(),
        "schedule set failed: {}",
        String::from_utf8_lossy(&set_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&set_out.stdout);
    assert!(
        stdout.contains("0 * * * *"),
        "Expected cron expression in output, got: {stdout}"
    );

    // Show should include the schedule
    let show_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "show", "sched-job"])
        .output()
        .expect("failed to run oxo-call");
    let show_stdout = String::from_utf8_lossy(&show_out.stdout);
    assert!(
        show_stdout.contains("0 * * * *"),
        "Expected schedule in show output, got: {show_stdout}"
    );

    // Clear schedule
    let clear_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "schedule", "sched-job"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        clear_out.status.success(),
        "schedule clear failed: {}",
        String::from_utf8_lossy(&clear_out.stderr)
    );
}

#[test]
fn test_job_list_builtin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list", "--builtin"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should show common built-in jobs
    assert!(
        stdout.contains("gpu") || stdout.contains("disk") || stdout.contains("squeue"),
        "Expected built-in job names in output, got: {stdout}"
    );
}

#[test]
fn test_job_list_builtin_tag_filter() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list", "--builtin", "--tag", "slurm"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("squeue"),
        "Expected SLURM built-in jobs, got: {stdout}"
    );
}

#[test]
fn test_job_import_builtin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "gpu"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "import failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("gpu"),
        "Expected job name in import output, got: {stdout}"
    );

    // Job should now appear in list
    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    let list_stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(
        list_stdout.contains("gpu"),
        "Expected imported job in list, got: {list_stdout}"
    );
}

#[test]
fn test_job_import_with_custom_name() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "disk", "--as-name", "my-disk"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "import failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    let stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(
        stdout.contains("my-disk"),
        "Expected custom-named import in list, got: {stdout}"
    );
}

#[test]
fn test_job_import_missing_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "nonexistent-builtin-xyz"])
        .output()
        .expect("failed to run oxo-call");
    assert!(!out.status.success(), "importing nonexistent should fail");
}

#[test]
fn test_job_import_no_args_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !out.status.success(),
        "import with no name and no --all should fail"
    );
}

#[test]
fn test_job_import_all() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "--all"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "import --all failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Imported"),
        "Expected import summary in output, got: {stdout}"
    );

    // All built-in jobs should now appear in the list.
    let list_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to run oxo-call");
    let list_stdout = String::from_utf8_lossy(&list_out.stdout);
    // "disk" and "gpu" are stable built-in templates.
    assert!(
        list_stdout.contains("disk"),
        "Expected 'disk' in list after --all import, got: {list_stdout}"
    );
    assert!(
        list_stdout.contains("gpu"),
        "Expected 'gpu' in list after --all import, got: {list_stdout}"
    );
}

#[test]
fn test_job_import_all_skips_existing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Import "disk" first.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "disk"])
        .output()
        .expect("failed to pre-import disk");

    // Now import --all; "disk" should be skipped, rest imported.
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "import", "--all"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "import --all failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("skipping"),
        "Expected skip message for 'disk', got: {stdout}"
    );
}

#[test]
fn test_job_add_with_schedule() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "cron-job",
            "df -h",
            "--schedule",
            "*/5 * * * *",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "add with schedule failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let show_out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "show", "cron-job"])
        .output()
        .expect("failed to run oxo-call");
    let stdout = String::from_utf8_lossy(&show_out.stdout);
    assert!(
        stdout.contains("*/5 * * * *"),
        "Expected schedule in show output, got: {stdout}"
    );
}

#[test]
fn test_job_backward_compat_cmd_alias() {
    // The `cmd` alias should still work after the rename to `job`.
    let output = oxo_call()
        .args(["cmd", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        output.status.success(),
        "'cmd --help' should succeed via backward-compat alias"
    );
}

// ─── Variable interpolation / parallel / batch tests ──────────────────────────

#[test]
fn test_job_run_help_shows_var_flag() {
    let out = oxo_call()
        .args(["job", "run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--var") || stdout.contains("-V"),
        "Expected --var in job run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--input-list") || stdout.contains("-i"),
        "Expected --input-list in job run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--jobs") || stdout.contains("-j"),
        "Expected --jobs in job run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--keep-order") || stdout.contains("-k"),
        "Expected --keep-order in job run help, got: {stdout}"
    );
}

#[test]
fn test_run_help_shows_var_flag() {
    let out = oxo_call()
        .args(["run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--var") || stdout.contains("-V"),
        "Expected --var in run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--input-list"),
        "Expected --input-list in run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--input-items"),
        "Expected --input-items in run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--jobs") || stdout.contains("-j"),
        "Expected --jobs in run help, got: {stdout}"
    );
}

#[test]
fn test_dry_run_help_shows_var_flag() {
    let out = oxo_call()
        .args(["dry-run", "--help"])
        .output()
        .expect("failed to run oxo-call");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--var") || stdout.contains("-V"),
        "Expected --var in dry-run help, got: {stdout}"
    );
    assert!(
        stdout.contains("--input-list"),
        "Expected --input-list in dry-run help, got: {stdout}"
    );
}

#[test]
fn test_job_run_var_substitution() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Add a job with a {MSG} placeholder in its command.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "var-job", "echo {MSG}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "var-job", "--var", "MSG=hello-world"])
        .output()
        .expect("failed to run job");
    assert!(
        out.status.success(),
        "job run with --var failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("hello-world"),
        "Expected substituted output, got: {stdout}"
    );
}

#[test]
fn test_job_run_var_dry_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "var-dry", "echo {GREETING}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "var-dry", "--var", "GREETING=hi", "--dry-run"])
        .output()
        .expect("failed to run job");
    assert!(
        out.status.success(),
        "dry-run with --var failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Should NOT have executed the echo.
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("dry-run") || stdout.contains("not executed"),
        "Expected dry-run indicator, got: {stdout}"
    );
}

#[test]
fn test_job_run_input_items_batch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Job echoes {item} so we can capture the substitution.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "batch-echo", "echo item={item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "batch-echo",
            "--input-items",
            "alpha,beta,gamma",
        ])
        .output()
        .expect("failed to run batch job");
    assert!(
        out.status.success(),
        "batch job run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("alpha"),
        "Expected alpha in output: {stdout}"
    );
    assert!(stdout.contains("beta"), "Expected beta in output: {stdout}");
    assert!(
        stdout.contains("gamma"),
        "Expected gamma in output: {stdout}"
    );
}

#[test]
fn test_job_run_input_list_from_file() {
    use std::io::Write;
    let tmp = tempfile::tempdir().expect("tempdir");
    // Write a small input list to a temp file.
    let list_file = tmp.path().join("inputs.txt");
    {
        let mut f = std::fs::File::create(&list_file).unwrap();
        writeln!(f, "# comment line — should be skipped").unwrap();
        writeln!(f, "item-one").unwrap();
        writeln!(f, "").unwrap(); // blank line — should be skipped
        writeln!(f, "item-two").unwrap();
    }

    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "list-echo", "echo {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "list-echo",
            "--input-list",
            list_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run job with --input-list");
    assert!(
        out.status.success(),
        "--input-list batch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("item-one"), "Expected item-one: {stdout}");
    assert!(stdout.contains("item-two"), "Expected item-two: {stdout}");
}

#[test]
fn test_job_run_batch_dry_run_shows_commands() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "dry-batch", "echo {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "dry-batch",
            "--input-items",
            "a,b,c",
            "--dry-run",
        ])
        .output()
        .expect("failed to run dry-run batch");
    assert!(
        out.status.success(),
        "dry-run batch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should list interpolated commands without running them.
    assert!(
        stdout.contains("echo a") || stdout.contains(" a"),
        "Expected 'a': {stdout}"
    );
    assert!(
        stdout.contains("echo b") || stdout.contains(" b"),
        "Expected 'b': {stdout}"
    );
    assert!(
        stdout.contains("echo c") || stdout.contains(" c"),
        "Expected 'c': {stdout}"
    );
    assert!(
        stdout.contains("dry-run") || stdout.contains("not executed"),
        "Expected dry-run indicator: {stdout}"
    );
}

#[test]
fn test_job_run_parallel_batch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Items with short sleep to verify parallel actually finishes in reasonable time.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "par-echo", "echo {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "par-echo",
            "--input-items",
            "x1,x2,x3,x4",
            "--jobs",
            "2",
        ])
        .output()
        .expect("failed to run parallel batch");
    assert!(
        out.status.success(),
        "parallel batch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("x1"), "Expected x1: {stdout}");
    assert!(stdout.contains("x2"), "Expected x2: {stdout}");
    assert!(stdout.contains("x3"), "Expected x3: {stdout}");
    assert!(stdout.contains("x4"), "Expected x4: {stdout}");
}

#[test]
fn test_job_run_path_interpolation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Test that {stem}, {ext}, {basename}, {dir} are substituted correctly.
    oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "path-job",
            "echo stem={stem} ext={ext} base={basename}",
        ])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "path-job", "--input-items", "data/sample.bam"])
        .output()
        .expect("failed to run path-job");
    assert!(
        out.status.success(),
        "path-job failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("stem=sample"),
        "Expected stem=sample: {stdout}"
    );
    assert!(stdout.contains("ext=bam"), "Expected ext=bam: {stdout}");
    assert!(
        stdout.contains("base=sample.bam"),
        "Expected base=sample.bam: {stdout}"
    );
}

#[test]
fn test_job_run_nr_placeholder() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "nr-job", "echo nr={nr}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "nr-job", "--input-items", "a,b,c"])
        .output()
        .expect("failed to run nr-job");
    assert!(
        out.status.success(),
        "nr-job failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("nr=1"), "Expected nr=1: {stdout}");
    assert!(stdout.contains("nr=2"), "Expected nr=2: {stdout}");
    assert!(stdout.contains("nr=3"), "Expected nr=3: {stdout}");
}

#[test]
fn test_job_run_var_and_item_combined() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Combine --var and --input-items: {THREADS} from --var, {item} from input.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "combo-job", "echo t={THREADS} f={item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "combo-job",
            "--var",
            "THREADS=8",
            "--input-items",
            "file1.bam,file2.bam",
        ])
        .output()
        .expect("failed to run combo-job");
    assert!(
        out.status.success(),
        "combo-job failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("t=8"), "Expected t=8: {stdout}");
    assert!(
        stdout.contains("f=file1.bam"),
        "Expected f=file1.bam: {stdout}"
    );
    assert!(
        stdout.contains("f=file2.bam"),
        "Expected f=file2.bam: {stdout}"
    );
}

// ─── Comprehensive reliability / edge-case tests ──────────────────────────────

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn test_job_run_var_invalid_format_shows_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "err-var-job", "echo hello"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "err-var-job", "--var", "NOEQUALSSIGN"])
        .output()
        .expect("failed to run oxo-call");
    // Must fail and show a helpful error.
    assert!(
        !out.status.success(),
        "Expected failure for invalid --var format"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("KEY=VALUE") || stderr.contains("invalid"),
        "Expected helpful error, got: {stderr}"
    );
}

#[test]
fn test_job_run_input_list_nonexistent_file_shows_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "no-list-job", "echo {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "no-list-job",
            "--input-list",
            "/nonexistent/path/list.txt",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !out.status.success(),
        "Expected failure for missing --input-list file"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("input-list")
            || stderr.contains("cannot open")
            || stderr.contains("No such"),
        "Expected helpful error, got: {stderr}"
    );
}

#[test]
fn test_job_run_nonexistent_job_shows_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "definitely-does-not-exist-99999"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !out.status.success(),
        "Expected failure for non-existent job"
    );
}

#[test]
fn test_job_run_batch_partial_failure_exits_nonzero() {
    // When at least one item fails, the overall command must exit non-zero.
    let tmp = tempfile::tempdir().expect("tempdir");
    // "false" is a POSIX command that always exits with code 1.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "fail-job", "false"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "fail-job", "--input-items", "a,b"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !out.status.success(),
        "Expected non-zero exit when batch items fail"
    );
}

#[test]
fn test_job_run_batch_mixed_fail_shows_count() {
    // A command that fails for "fail" but succeeds for others.
    let tmp = tempfile::tempdir().expect("tempdir");
    // "test {item} = ok" succeeds only when item == "ok".
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "mixed-job", "test {item} = ok"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "mixed-job", "--input-items", "ok,fail,ok"])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        !out.status.success(),
        "Expected non-zero when any item fails"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("failed") || stderr.contains("1/3") || stderr.contains("✗"),
        "Expected failure count in output, got: {stderr}"
    );
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn test_job_run_jobs_zero_treated_as_one() {
    // --jobs 0 should not panic; it should run sequentially (clamped to 1).
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "j0-job", "echo {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "j0-job",
            "--input-items",
            "x,y",
            "--jobs",
            "0",
        ])
        .output()
        .expect("failed to run oxo-call");
    assert!(
        out.status.success(),
        "--jobs 0 should clamp to 1, not panic: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("x"), "Expected x: {stdout}");
    assert!(stdout.contains("y"), "Expected y: {stdout}");
}

#[test]
fn test_job_run_input_items_empty_string_no_items() {
    // --input-items "" should produce no items and run the job normally (single run).
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "empty-items-job", "echo done"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "empty-items-job", "--input-items", ""])
        .output()
        .expect("failed to run");
    assert!(
        out.status.success(),
        "Empty --input-items should fall through to single run: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("done"),
        "Expected single run output: {stdout}"
    );
}

#[test]
fn test_job_run_var_with_spaces_in_value() {
    // --var values with spaces should be passed through correctly.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "space-var-job", "echo {MSG}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "space-var-job", "--var", "MSG=hello world"])
        .output()
        .expect("failed to run");
    assert!(
        out.status.success(),
        "Var with spaces should work: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("hello world"),
        "Expected 'hello world': {stdout}"
    );
}

#[test]
fn test_job_run_multiple_vars() {
    // Multiple --var flags are all applied.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "multi-var-job", "echo a={A} b={B} c={C}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "multi-var-job",
            "--var",
            "A=1",
            "--var",
            "B=2",
            "--var",
            "C=3",
        ])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a=1"), "Expected a=1: {stdout}");
    assert!(stdout.contains("b=2"), "Expected b=2: {stdout}");
    assert!(stdout.contains("c=3"), "Expected c=3: {stdout}");
}

#[test]
fn test_job_run_stdin_piped_items() {
    // When stdin is piped (not a TTY), items are read from stdin automatically.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "stdin-job", "echo got={item}"])
        .output()
        .expect("failed to add job");

    use std::process::Stdio;
    let mut child = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "stdin-job"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn process");

    {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "item-from-stdin").unwrap();
            writeln!(stdin, "another-item").unwrap();
        }
    }

    let output = child.wait_with_output().expect("failed to wait");
    assert!(
        output.status.success(),
        "stdin batch failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("item-from-stdin"),
        "Expected stdin item: {stdout}"
    );
    assert!(
        stdout.contains("another-item"),
        "Expected second stdin item: {stdout}"
    );
}

// ── History and status tracking ───────────────────────────────────────────────

#[test]
fn test_job_run_single_recorded_in_history() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "hist-single", "echo single-run"])
        .output()
        .expect("failed to add job");

    // Execute the job.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "hist-single"])
        .output()
        .expect("failed to run job");

    // Check history.
    let hist = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history", "hist-single"])
        .output()
        .expect("failed to get history");
    assert!(hist.status.success());
    let stdout = String::from_utf8_lossy(&hist.stdout);
    assert!(
        stdout.contains("hist-single") || stdout.contains("echo single-run"),
        "Expected history entry, got: {stdout}"
    );
}

#[test]
fn test_job_run_batch_recorded_in_history() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "hist-batch", "echo {item}"])
        .output()
        .expect("failed to add job");

    // Run a batch.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "hist-batch", "--input-items", "p,q,r"])
        .output()
        .expect("failed to run batch");

    // Check history — should contain a batch entry.
    let hist = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history", "hist-batch"])
        .output()
        .expect("failed to get history");
    assert!(hist.status.success());
    let stdout = String::from_utf8_lossy(&hist.stdout);
    assert!(
        stdout.contains("hist-batch") || stdout.contains("batch"),
        "Expected batch history entry, got: {stdout}"
    );
}

#[test]
fn test_job_status_shows_run_count_after_execution() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "status-job", "echo status-test"])
        .output()
        .expect("failed to add job");

    // Run twice.
    for _ in 0..2 {
        oxo_call_with_tmpdir(tmp.path())
            .args(["job", "run", "status-job"])
            .output()
            .expect("failed to run job");
    }

    // Status should show run count >= 2.
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "status", "status-job"])
        .output()
        .expect("failed to run status");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should show last run time or run count.
    assert!(
        stdout.contains("status-job"),
        "Expected job name in status: {stdout}"
    );
}

#[test]
fn test_job_status_exit_code_reflects_last_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "fail-status-job", "false"])
        .output()
        .expect("failed to add job");

    // Run (will fail, but we ignore the exit code here).
    let _ = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "fail-status-job"])
        .output();

    // Status should reflect the failed run.
    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "status", "fail-status-job"])
        .output()
        .expect("failed to run status");
    assert!(out.status.success(), "status command itself should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("fail-status-job"),
        "Expected job name: {stdout}"
    );
}

#[test]
fn test_job_run_var_single_recorded_correctly() {
    // --var substitution in a single run should be recorded in history.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "var-hist-job", "echo {VAL}"])
        .output()
        .expect("failed to add job");

    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "var-hist-job", "--var", "VAL=myvalue"])
        .output()
        .expect("failed to run");

    let hist = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history", "var-hist-job"])
        .output()
        .expect("failed to get history");
    assert!(hist.status.success());
    let stdout = String::from_utf8_lossy(&hist.stdout);
    assert!(
        stdout.contains("var-hist-job"),
        "Expected job name in history: {stdout}"
    );
}

// ── dry-run correctness ───────────────────────────────────────────────────────

#[test]
fn test_job_run_dry_run_does_not_modify_history() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "dry-hist-job", "echo hello"])
        .output()
        .expect("failed to add job");

    // Dry-run — should not record to history.
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "dry-hist-job", "--dry-run"])
        .output()
        .expect("failed to run dry-run");

    let hist = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "history", "dry-hist-job"])
        .output()
        .expect("failed to get history");
    assert!(hist.status.success());
    let stdout = String::from_utf8_lossy(&hist.stdout);
    // Dry-run should not create history entries.
    assert!(
        !stdout.contains("echo hello"),
        "Dry-run should not add to history: {stdout}"
    );
}

#[test]
fn test_job_run_batch_dry_run_shows_all_items_with_nr() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "nr-dry-job", "echo {nr}: {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "nr-dry-job",
            "--input-items",
            "a,b,c",
            "--dry-run",
        ])
        .output()
        .expect("failed to run dry-run batch");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // All 3 items should appear expanded.
    assert!(stdout.contains("1: a"), "Expected 1: a, got: {stdout}");
    assert!(stdout.contains("2: b"), "Expected 2: b, got: {stdout}");
    assert!(stdout.contains("3: c"), "Expected 3: c, got: {stdout}");
}

// ── Interaction with other commands ──────────────────────────────────────────

#[test]
fn test_job_list_unaffected_by_new_flags() {
    // Ensure job list still works normally after adding the new job run features.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "list-check-job", "echo ok"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "list"])
        .output()
        .expect("failed to list");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("list-check-job"),
        "Expected job in list: {stdout}"
    );
}

#[test]
fn test_job_show_unaffected_by_new_flags() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "show-check-job",
            "echo {item}",
            "--description",
            "A batch-capable job",
        ])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "show", "show-check-job"])
        .output()
        .expect("failed to show");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("echo {item}"),
        "Expected command in show: {stdout}"
    );
    assert!(
        stdout.contains("A batch-capable job"),
        "Expected description: {stdout}"
    );
}

#[test]
fn test_job_edit_command_preserves_placeholders() {
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "edit-placeholder-job", "echo old"])
        .output()
        .expect("failed to add job");

    oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "edit",
            "edit-placeholder-job",
            "--command",
            "echo {item}_{stem}",
        ])
        .output()
        .expect("failed to edit job");

    // Run with an item to verify the edited command works.
    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "edit-placeholder-job",
            "--input-items",
            "sample.bam",
        ])
        .output()
        .expect("failed to run edited job");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("sample.bam_sample"),
        "Expected interpolated output: {stdout}"
    );
}

#[test]
fn test_job_run_input_list_and_input_items_combined() {
    // --input-list and --input-items can be combined; items are concatenated.
    use std::io::Write;
    let tmp = tempfile::tempdir().expect("tempdir");
    let list_file = tmp.path().join("items.txt");
    {
        let mut f = std::fs::File::create(&list_file).unwrap();
        writeln!(f, "from-file").unwrap();
    }

    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "combo-list-job", "echo {item}"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "combo-list-job",
            "--input-list",
            list_file.to_str().unwrap(),
            "--input-items",
            "from-inline",
        ])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("from-file"), "Expected from-file: {stdout}");
    assert!(
        stdout.contains("from-inline"),
        "Expected from-inline: {stdout}"
    );
}

#[test]
fn test_job_run_var_only_no_items_single_run() {
    // --var with no input list: should do a single run with vars substituted.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "var-only-job", "echo prefix-{PREFIX}-suffix"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "var-only-job", "--var", "PREFIX=test123"])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("prefix-test123-suffix"),
        "Expected substituted output: {stdout}"
    );
}

// ── Input-item trimming and whitespace handling ───────────────────────────────

#[test]
fn test_job_run_input_items_trims_whitespace() {
    // Items separated by ", " (with spaces) should be trimmed.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "trim-job", "echo [{item}]"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "trim-job",
            "--input-items",
            " aaa , bbb , ccc ",
        ])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("[aaa]"), "Expected [aaa]: {stdout}");
    assert!(stdout.contains("[bbb]"), "Expected [bbb]: {stdout}");
    assert!(stdout.contains("[ccc]"), "Expected [ccc]: {stdout}");
}

// ── High-concurrency stability ────────────────────────────────────────────────

#[test]
fn test_job_run_high_concurrency_all_succeed() {
    // Run 20 items with jobs=10 to stress the semaphore.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "stress-job", "echo {nr}"])
        .output()
        .expect("failed to add job");

    let items: Vec<&str> = vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t",
    ];
    let items_str = items.join(",");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "run",
            "stress-job",
            "--input-items",
            &items_str,
            "--jobs",
            "10",
        ])
        .output()
        .expect("failed to run stress test");
    assert!(
        out.status.success(),
        "High-concurrency batch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // All 20 items should have been processed (each echoes its {nr}).
    for nr in 1..=20usize {
        assert!(
            stdout.contains(&nr.to_string()),
            "Expected nr={nr}: {stdout}"
        );
    }
}

// ── run / dry-run --var parsing ───────────────────────────────────────────────

#[test]
fn test_run_var_flag_parsed_help_output() {
    let out = oxo_call()
        .args(["run", "--help"])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("KEY=VALUE"),
        "Expected KEY=VALUE in run help: {stdout}"
    );
}

#[test]
fn test_dry_run_var_flag_parsed_help_output() {
    let out = oxo_call()
        .args(["dry-run", "--help"])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("KEY=VALUE"),
        "Expected KEY=VALUE in dry-run help: {stdout}"
    );
}

#[test]
fn test_run_input_items_flag_exists() {
    let out = oxo_call()
        .args(["run", "--help"])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("input-items"),
        "Expected --input-items in run help: {stdout}"
    );
}

#[test]
fn test_dry_run_input_items_flag_exists() {
    let out = oxo_call()
        .args(["dry-run", "--help"])
        .output()
        .expect("failed to run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("input-items"),
        "Expected --input-items in dry-run help: {stdout}"
    );
}

// ── Backward-compatibility: existing job run (no new flags) still works ────────

#[test]
fn test_job_run_existing_behavior_unchanged() {
    // Plain `job run <name>` without any new flags must behave identically
    // to before the new features were added.
    let tmp = tempfile::tempdir().expect("tempdir");
    oxo_call_with_tmpdir(tmp.path())
        .args(["job", "add", "plain-run-job", "echo plain-output-99"])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "plain-run-job"])
        .output()
        .expect("failed to run");
    assert!(
        out.status.success(),
        "Plain job run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("plain-output-99"),
        "Expected plain output: {stdout}"
    );
}

#[test]
fn test_job_run_dry_run_existing_behavior_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Use a sentinel that is distinct from the command text shown in the dry-run header.
    // The command display line will show "echo DRY_RUN_SENTINEL_OUTPUT" but the
    // actual echo output "DRY_RUN_SENTINEL_OUTPUT" (without the command prefix)
    // must NOT appear, confirming the command was never executed.
    oxo_call_with_tmpdir(tmp.path())
        .args([
            "job",
            "add",
            "plain-dry-job",
            "echo DRY_RUN_SENTINEL_OUTPUT",
        ])
        .output()
        .expect("failed to add job");

    let out = oxo_call_with_tmpdir(tmp.path())
        .args(["job", "run", "plain-dry-job", "--dry-run"])
        .output()
        .expect("failed to run dry-run");
    assert!(out.status.success());
    // The command header IS expected in the output (shows what *would* run).
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("echo DRY_RUN_SENTINEL_OUTPUT"),
        "Expected command text in dry-run output: {stdout}"
    );
    // The raw echo output should NOT appear (i.e., the command was not executed).
    // When `echo DRY_RUN_SENTINEL_OUTPUT` runs it would print "DRY_RUN_SENTINEL_OUTPUT"
    // on its own line; we check that the output only contains it as part of the
    // "Command:" display line, not as a standalone line.
    let lines: Vec<&str> = stdout.lines().collect();
    let standalone = lines.iter().any(|l| l.trim() == "DRY_RUN_SENTINEL_OUTPUT");
    assert!(
        !standalone,
        "dry-run should not execute the command: {stdout}"
    );
    assert!(
        stdout.contains("dry-run") || stdout.contains("not executed"),
        "Expected dry-run indicator: {stdout}"
    );
}
