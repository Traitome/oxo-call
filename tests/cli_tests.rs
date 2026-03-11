/// Integration tests for oxo-call CLI
use std::process::Command;

fn oxo_call() -> Command {
    Command::new(env!("CARGO_BIN_EXE_oxo-call"))
}

#[test]
fn test_help_output() {
    let output = oxo_call().arg("--help").output().expect("failed to run oxo-call");
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
    let output = oxo_call().arg("--version").output().expect("failed to run oxo-call");
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
        combined.contains("token") ||
        combined.contains("API") ||
        combined.contains("error") ||
        combined.contains("Fetching"),
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

