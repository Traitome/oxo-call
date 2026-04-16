//! Unit and integration tests for the LLM module.

use super::prompt::*;
use super::provider::LlmClient;
use super::response::*;
use super::types::*;

#[test]
fn test_parse_verification_response_success() {
    let raw = "STATUS: success\nSUMMARY: Command completed successfully.\nISSUES:\n- none\nSUGGESTIONS:\n- none";
    let v = parse_verification_response(raw);
    assert!(v.success);
    assert_eq!(v.summary, "Command completed successfully.");
}

#[test]
fn test_parse_verification_response_failure() {
    let raw = "STATUS: failure\nSUMMARY: Command failed.\nISSUES:\n- Error\nSUGGESTIONS:\n- Retry";
    let v = parse_verification_response(raw);
    assert!(!v.success);
    assert_eq!(v.issues.len(), 1);
}

#[test]
fn test_parse_shell_args_simple() {
    let args = parse_shell_args("-o out.bam input.bam");
    assert_eq!(args, vec!["-o", "out.bam", "input.bam"]);
}

#[test]
fn test_is_valid_suggestion() {
    let s = LlmCommandSuggestion {
        args: vec!["-o".to_string()],
        explanation: "Test".to_string(),
        raw_response: String::new(),
        inference_ms: 0.0,
    };
    assert!(is_valid_suggestion(&s));
}

#[test]
fn test_build_prompt_basic() {
    let prompt = build_prompt(
        "samtools",
        "docs",
        "sort bam",
        None,
        false,
        0,
        PromptTier::Full,
    );
    assert!(prompt.contains("samtools"));
    assert!(prompt.contains("sort bam"));
}

#[test]
fn test_system_prompt_not_empty() {
    let p = system_prompt();
    assert!(!p.is_empty());
    assert!(p.contains("ARGS"));
}

#[test]
fn test_estimate_tokens() {
    assert_eq!(estimate_tokens(""), 0);
    assert_eq!(estimate_tokens("abcd"), 1);
    assert_eq!(estimate_tokens("abcde"), 2);
}

#[test]
fn test_prompt_tier() {
    assert_eq!(prompt_tier(0, "model"), PromptTier::Full);
    assert_eq!(prompt_tier(4096, "model"), PromptTier::Medium);
    assert_eq!(prompt_tier(2048, "model"), PromptTier::Compact);
}

#[test]
fn test_strip_markdown_fences() {
    let raw = "```markdown\ncontent\n```";
    let stripped = strip_markdown_fences(raw);
    assert!(!stripped.starts_with("```"));
}

#[test]
fn test_parse_response_basic() {
    let raw = "ARGS: sort -o out.bam\nEXPLANATION: Sort the BAM.";
    let suggestion = parse_response(raw).unwrap();
    assert_eq!(suggestion.args, vec!["sort", "-o", "out.bam"]);
}

#[test]
fn test_sanitize_args_strips_tool_name() {
    let args = vec!["samtools".to_string(), "sort".to_string()];
    let result = sanitize_args("samtools", args);
    assert_eq!(result, vec!["sort"]);
}

#[test]
fn test_llm_client_new() {
    use crate::config::Config;
    let cfg = Config::default();
    let _client = LlmClient::new(cfg);
}

#[test]
fn test_chat_message_clone() {
    let msg = ChatMessage {
        role: "user".to_string(),
        content: "hello".to_string(),
    };
    let cloned = msg.clone();
    assert_eq!(cloned.role, "user");
}

#[test]
fn test_chat_request_serializes() {
    let req = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![],
        max_tokens: 2048,
        temperature: 0.0,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("gpt-4o"));
}

#[test]
fn test_chat_response_deserializes() {
    let json = r#"{"choices": [{"message": {"role": "assistant", "content": "test"}}]}"#;
    let resp: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.choices.len(), 1);
}

#[test]
fn test_parse_skill_verify_response() {
    let raw = "VERDICT: pass\nSUMMARY: Good.\nISSUES:\n- none\nSUGGESTIONS:\n- none";
    let v = parse_skill_verify_response(raw);
    assert!(v.passed);
}

#[test]
fn test_build_skill_verify_prompt() {
    let prompt = build_skill_verify_prompt("samtools", "content");
    assert!(prompt.contains("samtools"));
}

#[test]
fn test_verification_system_prompt() {
    let prompt = verification_system_prompt();
    assert!(!prompt.is_empty());
}

#[test]
fn test_skill_reviewer_system_prompt() {
    let prompt = skill_reviewer_system_prompt();
    assert!(!prompt.is_empty());
}

#[test]
fn test_build_retry_prompt() {
    let prompt = build_retry_prompt(
        "samtools",
        "docs",
        "task",
        None,
        "prev",
        false,
        0,
        PromptTier::Full,
    );
    assert!(prompt.contains("samtools"));
}

#[test]
fn test_strip_prefix_case_insensitive() {
    assert_eq!(
        strip_prefix_case_insensitive("ARGS: test", "ARGS:"),
        Some(" test")
    );
    assert_eq!(
        strip_prefix_case_insensitive("args: test", "ARGS:"),
        Some(" test")
    );
}

#[test]
fn test_parse_json_response() {
    let json = r#"{"args": "-o out.bam", "explanation": "Test"}"#;
    let result = try_parse_json_response(json).unwrap();
    assert!(!result.args.is_empty());
}
