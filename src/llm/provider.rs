use crate::config::Config;
use crate::copilot_auth;
use crate::doc_processor::StructuredDoc;
use crate::error::{OxoError, Result};
use crate::skill::Skill;
use crate::validator::{aggressive_correct, correct_format, validate_subcommand};
use colored::Colorize;
use sha2::Digest;

use super::postprocess::{
    add_missing_required_flags, add_task_implied_flags, apply_corrections_to_args,
    apply_template_corrections, apply_tool_specific_corrections, fill_missing_flag_values,
    filter_irrelevant_flags_for_small_model, fix_output_extensions, limit_flag_count,
    replace_generic_values, validate_flags_against_catalog,
};
use super::prompt::{
    build_prompt, build_retry_prompt, build_skill_generate_prompt, build_skill_polish_prompt,
    build_skill_verify_prompt, build_verification_prompt, skill_reviewer_system_prompt,
    system_prompt, system_prompt_unified, verification_system_prompt,
};
use super::response::{
    is_valid_suggestion, parse_response, parse_skill_verify_response, parse_verification_response,
    sanitize_args, strip_markdown_fences,
};
use super::rule_engine::assemble_command_from_rules;
use super::streaming::apply_provider_auth_headers;
use super::task_values::{
    detect_subcommand_for_tool, extract_task_values, is_no_subcommand_tool,
    rule_based_subcommand_match,
};
use super::template_engine::{
    fill_template, find_best_template, generate_from_template, generate_from_template_with_score,
    merge_llm_into_template, merge_template_with_llm, remove_duplicate_flags_vec,
};
use super::types::{
    ChatMessage, ChatRequest, ChatRequestStreaming, ChatResponse, LlmCommandSuggestion,
    LlmRunVerification, LlmSkillVerification, LlmVerificationResult, PromptTier,
};
use crate::streaming_display;

fn find_best_example(
    tool: &str,
    task: &str,
    examples: &[String],
    task_values: &super::task_values::TaskValues,
) -> Option<Vec<String>> {
    if examples.is_empty() {
        return None;
    }

    let task_lower = task.to_ascii_lowercase();
    let task_words: std::collections::HashSet<&str> = task_lower
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .collect();

    let mut best_example: Option<(&String, i32)> = None;

    for example in examples {
        let ex_lower = example.to_ascii_lowercase();

        if ex_lower.contains('>')
            || ex_lower.contains('|')
            || ex_lower.contains("&&")
            || ex_lower.contains(" in ")
            || ex_lower.contains(" out ")
            || ex_lower.contains("(fast)")
            || ex_lower.contains("(slow)")
            || ex_lower.contains("[options]")
            || ex_lower.contains("<input>")
            || ex_lower.contains("<output>")
            || ex_lower.contains("...")
            || ex_lower.contains(" may also be given ")
            || ex_lower.contains(" to be expanded to ")
            || ex_lower.contains(" does not have to be specified ")
            || ex_lower.contains("e.g.") && ex_lower.len() < 60
        {
            continue;
        }

        let mut score: i32 = 0;

        for word in &task_words {
            if ex_lower.contains(word) {
                score += 10;
            }
        }

        let ex_args: Vec<&str> = ex_lower.split_whitespace().collect();
        let has_flags = ex_args.iter().any(|a| a.starts_with('-'));
        if has_flags {
            score += 5;
        }

        if let Some(subcmd) = ex_args.first() {
            if !subcmd.starts_with('-') {
                let sub_lower = subcmd.to_string();
                if task_words.contains(&sub_lower.as_str()) {
                    score += 20;
                }
            }
        }

        for input_file in &task_values.input_files {
            let file_name = std::path::Path::new(input_file)
                .file_name()
                .map(|f| f.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            if !file_name.is_empty() && ex_lower.contains(&file_name) {
                score += 15;
            }
        }

        for ext in &[
            ".bam", ".fq", ".fastq", ".fa", ".fasta", ".vcf", ".bed", ".gtf",
        ] {
            if task_lower.contains(ext) && ex_lower.contains(ext) {
                score += 5;
            }
        }

        if score > 0 {
            match best_example {
                Some((_, best_score)) if score <= best_score => {}
                _ => best_example = Some((example, score)),
            }
        }
    }

    best_example.map(|(example, score)| {
        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
            eprintln!(
                "[verbose] find_best_example: tool={}, score={}, example='{}'",
                tool,
                score,
                example.chars().take(80).collect::<String>()
            );
        }

        let mut args = crate::llm::response::parse_shell_args(example);

        if !args.is_empty() && !args[0].starts_with('-') {
            let first = args[0].to_ascii_lowercase();
            if first == tool.to_lowercase() || first == format!("{}-call", tool.to_lowercase()) {
                args.remove(0);
            }
        }

        let mut used_files: std::collections::HashSet<String> = std::collections::HashSet::new();
        for arg in &args {
            let al = arg.to_ascii_lowercase();
            if al.contains('.')
                && (al.contains('/')
                    || al.ends_with(".bam")
                    || al.ends_with(".sam")
                    || al.ends_with(".fq")
                    || al.ends_with(".fastq")
                    || al.ends_with(".fa")
                    || al.ends_with(".fasta")
                    || al.ends_with(".vcf")
                    || al.ends_with(".bed")
                    || al.ends_with(".gtf"))
            {
                used_files.insert(al);
            }
        }

        let mut new_args = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg.starts_with('-')
                && !arg.contains('=')
                && i + 1 < args.len()
                && !args[i + 1].starts_with('-')
            {
                let value = &args[i + 1];
                let vl = value.to_ascii_lowercase();

                if (vl.ends_with(".bam")
                    || vl.ends_with(".sam")
                    || vl.ends_with(".fq")
                    || vl.ends_with(".fastq")
                    || vl.ends_with(".fa")
                    || vl.ends_with(".fasta")
                    || vl.ends_with(".vcf")
                    || vl.ends_with(".bed")
                    || vl.ends_with(".gtf")
                    || vl.ends_with(".gz"))
                    && !task_values
                        .input_files
                        .iter()
                        .any(|f| f.to_ascii_lowercase() == vl)
                    && !task_values
                        .output_files
                        .iter()
                        .any(|f| f.to_ascii_lowercase() == vl)
                {
                    let replacement = task_values
                        .input_files
                        .iter()
                        .find(|f| {
                            let fl = f.to_ascii_lowercase();
                            let same_ext = vl.rsplit('.').next() == fl.rsplit('.').next();
                            !used_files.contains(&fl) && same_ext
                        })
                        .or_else(|| {
                            task_values
                                .input_files
                                .iter()
                                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                        });
                    if let Some(rep) = replacement {
                        used_files.insert(rep.to_ascii_lowercase());
                        new_args.push(arg.clone());
                        new_args.push(rep.clone());
                    } else {
                        new_args.push(arg.clone());
                        new_args.push(value.clone());
                    }
                } else {
                    new_args.push(arg.clone());
                    new_args.push(value.clone());
                }
                i += 2;
            } else {
                new_args.push(arg.clone());
                i += 1;
            }
        }

        new_args
    })
}

pub struct LlmClient {
    pub(crate) config: Config,
    client: reqwest::Client,
    stream_enabled: bool,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_max_idle_per_host(16)
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to build configured HTTP client: {e}; using defaults");
                reqwest::Client::new()
            });
        let stream_enabled = config.llm.stream;
        LlmClient {
            config,
            client,
            stream_enabled,
        }
    }

    pub fn set_no_stream(&mut self, no_stream: bool) {
        if no_stream {
            self.stream_enabled = false;
        }
    }

    pub async fn suggest_command_two_step(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
    ) -> Result<LlmCommandSuggestion> {
        let model = self.config.effective_model();
        let profile = crate::config::get_model_profile(&model);
        let temperature = Some(profile.optimal_temperature);

        let sdoc = match structured_doc {
            Some(s) => s,
            None => {
                return self
                    .suggest_command(tool, documentation, task, skill, no_prompt, structured_doc)
                    .await;
            }
        };

        let task_values = extract_task_values(task);

        let tool_specific_subcmd = detect_subcommand_for_tool(tool, task, &[]);

        let template_subcmd: Option<String> = find_best_template(tool, task).and_then(|tmpl| {
            let tmpl_args = crate::llm::response::parse_shell_args(tmpl);
            if !tmpl_args.is_empty() {
                let first = &tmpl_args[0];
                if !first.starts_with('-') && !first.contains('.') && !first.contains('/') {
                    if sdoc.has_subcommands
                        && sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == first.to_ascii_lowercase())
                    {
                        return Some(first.clone());
                    } else if !sdoc.has_subcommands {
                        return Some(first.clone());
                    }
                }
            }
            None
        });

        let rule_subcmd = if tool_specific_subcmd.is_some()
            && !tool_specific_subcmd
                .as_deref()
                .unwrap_or("")
                .starts_with("_NO_SUB_")
        {
            tool_specific_subcmd.clone()
        } else if is_no_subcommand_tool(tool) {
            None
        } else if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
            rule_based_subcommand_match(task, &sdoc.subcommands, &sdoc.subcommand_descriptions)
        } else {
            None
        };

        let llm_subcmd = if tool_specific_subcmd.is_some()
            && !tool_specific_subcmd
                .as_deref()
                .unwrap_or("")
                .starts_with("_NO_SUB_")
        {
            None
        } else if is_no_subcommand_tool(tool) {
            None
        } else if sdoc.has_subcommands
            && !sdoc.subcommands.is_empty()
            && template_subcmd.is_none()
            && rule_subcmd.is_none()
        {
            self.llm_select_subcommand(tool, task, sdoc, temperature)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        let selected_subcommand = if tool_specific_subcmd.is_some()
            && !tool_specific_subcmd
                .as_deref()
                .unwrap_or("")
                .starts_with("_NO_SUB_")
        {
            tool_specific_subcmd.clone()
        } else if is_no_subcommand_tool(tool) {
            if tool_specific_subcmd.is_some()
                && !tool_specific_subcmd
                    .as_deref()
                    .unwrap_or("")
                    .starts_with("_NO_SUB_")
            {
                tool_specific_subcmd.clone()
            } else if let Some(ref tsub) = template_subcmd {
                if sdoc
                    .subcommands
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(tsub))
                    || sdoc
                        .companion_binaries
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(tsub))
                {
                    Some(tsub.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
            if let Some(ref tsub) = template_subcmd {
                if sdoc
                    .subcommands
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(tsub))
                {
                    Some(tsub.clone())
                } else {
                    rule_subcmd
                        .or(llm_subcmd)
                        .or_else(|| sdoc.subcommands.first().cloned())
                }
            } else if let Some(rule) = rule_subcmd {
                Some(rule)
            } else {
                llm_subcmd.or_else(|| sdoc.subcommands.first().cloned())
            }
        } else if !sdoc.companion_binaries.is_empty() {
            let task_lower = task.to_ascii_lowercase();
            if tool_specific_subcmd.is_some()
                && !tool_specific_subcmd
                    .as_deref()
                    .unwrap_or("")
                    .starts_with("_NO_SUB_")
            {
                let sub = tool_specific_subcmd.clone().unwrap();
                if sdoc
                    .companion_binaries
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&sub))
                {
                    Some(sub)
                } else {
                    sdoc.companion_binaries
                        .iter()
                        .find(|s| {
                            let s_lower = s.to_ascii_lowercase();
                            let parts: Vec<&str> = s_lower
                                .split(|c: char| c == '-' || c == '_')
                                .filter(|p| p.len() >= 3)
                                .collect();
                            parts.iter().any(|p| task_lower.contains(p))
                        })
                        .cloned()
                        .or_else(|| sdoc.companion_binaries.first().cloned())
                }
            } else {
                sdoc.companion_binaries
                    .iter()
                    .find(|s| {
                        let s_lower = s.to_ascii_lowercase();
                        let parts: Vec<&str> = s_lower
                            .split(|c: char| c == '-' || c == '_')
                            .filter(|p| p.len() >= 3)
                            .collect();
                        parts.iter().any(|p| task_lower.contains(p))
                    })
                    .cloned()
                    .or_else(|| sdoc.companion_binaries.first().cloned())
            }
        } else {
            None
        };

        let template_args =
            find_best_template(tool, task).map(|tmpl| fill_template(tmpl, task, &task_values));

        if let Some(ref tmpl_args) = template_args {
            if !tmpl_args.is_empty() {
                let tmpl_has_sub = sdoc.has_subcommands
                    && !tmpl_args[0].starts_with('-')
                    && sdoc
                        .subcommands
                        .iter()
                        .any(|s| s.to_ascii_lowercase() == tmpl_args[0].to_ascii_lowercase());
                let tmpl_has_flags = tmpl_args.iter().any(|a| a.starts_with('-'));
                let tmpl_has_real_files = tmpl_args.iter().any(|a| {
                    a.contains('.')
                        && !a.starts_with('-')
                        && a != "input.bam"
                        && a != "output.bam"
                        && a != "reads_1.fq"
                        && a != "reads_2.fq"
                        && a != "reference.fa"
                        && a != "input2.bed"
                        && a != "annotation.gtf"
                        && a != "database"
                        && a != "metrics.txt"
                        && a != "tool.jar"
                        && a != "*.bam"
                        && a != "SRR123456"
                        && !a.starts_with("/path/to/")
                });
                let tmpl_has_task_files =
                    !task_values.input_files.is_empty() || !task_values.output_files.is_empty();

                let no_sub_tool = is_no_subcommand_tool(tool);
                let tmpl_has_companion = !sdoc.has_subcommands
                    && !tmpl_args[0].starts_with('-')
                    && sdoc
                        .companion_binaries
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(&tmpl_args[0]));

                let tmpl_is_complete = (tmpl_has_sub
                    && tmpl_has_flags
                    && (tmpl_has_real_files || tmpl_has_task_files))
                    || (tmpl_has_sub && tmpl_has_flags && !sdoc.has_subcommands)
                    || (tmpl_has_flags && !sdoc.has_subcommands && tmpl_args.len() >= 3)
                    || (tmpl_has_sub && tmpl_has_flags && tmpl_args.len() >= 4)
                    || (no_sub_tool
                        && tmpl_has_flags
                        && (tmpl_has_real_files || tmpl_has_task_files)
                        && tmpl_args.len() >= 2)
                    || (tmpl_has_companion
                        && tmpl_has_flags
                        && (tmpl_has_real_files || tmpl_has_task_files))
                    || (tmpl_has_sub && tmpl_has_task_files)
                    || (tmpl_has_flags && tmpl_has_task_files && tmpl_args.len() >= 3)
                    || (no_sub_tool && tmpl_has_task_files && tmpl_args.len() >= 2)
                    || (tmpl_has_companion && tmpl_has_task_files);

                if tmpl_is_complete {
                    let subcmd_matches = if no_sub_tool && selected_subcommand.is_none() {
                        true
                    } else if no_sub_tool && selected_subcommand.is_some() {
                        tmpl_args[0].eq_ignore_ascii_case(selected_subcommand.as_ref().unwrap())
                            || tmpl_args[0].starts_with('-')
                    } else {
                        selected_subcommand
                            .as_ref()
                            .map(|s| tmpl_args[0].eq_ignore_ascii_case(s))
                            .unwrap_or(true)
                    };

                    if subcmd_matches {
                        let mut final_args = tmpl_args.clone();
                        if let Some(ref sub) = selected_subcommand {
                            if !final_args.is_empty() && !final_args[0].eq_ignore_ascii_case(sub) {
                                final_args[0] = sub.clone();
                            }
                        }
                        final_args = add_missing_required_flags(&final_args, sdoc, task);
                        final_args = add_task_implied_flags(&final_args, sdoc, task);
                        final_args = remove_duplicate_flags_vec(&final_args);
                        final_args = apply_corrections_to_args(
                            &final_args,
                            tool,
                            structured_doc,
                            Some(task),
                        );

                        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                            eprintln!(
                                "{} [TwoStep] template-direct: sub={:?} final='{}'",
                                "[verbose]".dimmed(),
                                selected_subcommand,
                                final_args.join(" ").chars().take(80).collect::<String>()
                            );
                        }

                        return Ok(LlmCommandSuggestion {
                            args: final_args,
                            explanation: String::new(),
                            inference_ms: 0.0,
                        });
                    }
                }
            }
        }

        let is_small_model = crate::config::infer_model_parameter_count(&model)
            .map(|p| p <= 8.0)
            .unwrap_or(false);

        let rule_assembled = assemble_command_from_rules(
            tool,
            task,
            sdoc,
            selected_subcommand.as_deref(),
            &task_values,
        );

        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
            eprintln!(
                "{} [TwoStep] rule_assembled='{}' has_subcommands={}",
                "[verbose]".dimmed(),
                rule_assembled
                    .join(" ")
                    .chars()
                    .take(120)
                    .collect::<String>(),
                sdoc.has_subcommands
            );
        }

        if !rule_assembled.is_empty() {
            let has_subcmd = sdoc.has_subcommands
                && !rule_assembled.is_empty()
                && !rule_assembled[0].starts_with('-')
                && sdoc
                    .subcommands
                    .iter()
                    .any(|s| s.to_ascii_lowercase() == rule_assembled[0].to_ascii_lowercase());
            let has_flags = rule_assembled.iter().any(|a| a.starts_with('-'));
            let has_files = rule_assembled
                .iter()
                .any(|a| a.contains('.') && !a.starts_with('-'));

            let rule_is_good = if is_small_model {
                (has_subcmd || !sdoc.has_subcommands) && (has_flags || has_files)
            } else {
                (has_subcmd || !sdoc.has_subcommands) && (has_flags || has_files)
            };

            let has_required_flags = {
                let required_flags: Vec<&str> = sdoc
                    .flag_catalog
                    .iter()
                    .filter(|e| e.required)
                    .map(|e| e.flag.as_str())
                    .collect();
                if required_flags.is_empty() {
                    true
                } else {
                    let args_lower = rule_assembled
                        .iter()
                        .map(|a| a.to_ascii_lowercase())
                        .collect::<Vec<_>>();
                    required_flags.iter().all(|rf| {
                        let rf_lower = rf.to_ascii_lowercase();
                        args_lower
                            .iter()
                            .any(|a| a == &rf_lower || a.starts_with(&format!("{}=", rf_lower)))
                    })
                }
            };

            let has_positional_args = {
                let flag_args: std::collections::HashSet<String> = sdoc
                    .flag_catalog
                    .iter()
                    .flat_map(|e| {
                        let mut flags = vec![e.flag.clone()];
                        if let Some(ref alt) = e.alt_form {
                            flags.push(alt.clone());
                        }
                        flags
                    })
                    .map(|f| f.to_ascii_lowercase())
                    .collect();
                rule_assembled
                    .iter()
                    .filter(|a| !a.starts_with('-') && !flag_args.contains(&a.to_ascii_lowercase()))
                    .count()
                    > if sdoc.has_subcommands { 1 } else { 0 }
            };

            let needs_llm_for_pattern = tool == "grep"
                || tool == "awk"
                || tool == "sed"
                || tool == "perl"
                || tool == "python"
                || tool == "r"
                || tool == "bash"
                || tool == "julia"
                || tool == "java"
                || tool == "find"
                || tool == "ssh";

            let rule_is_excellent = is_small_model
                && rule_is_good
                && (has_required_flags || sdoc.flag_catalog.iter().all(|e| !e.required))
                && !needs_llm_for_pattern;

            if rule_is_excellent {
                let mut final_args = rule_assembled;
                final_args = add_missing_required_flags(&final_args, sdoc, task);
                final_args = add_task_implied_flags(&final_args, sdoc, task);
                final_args = remove_duplicate_flags_vec(&final_args);
                final_args =
                    apply_corrections_to_args(&final_args, tool, structured_doc, Some(task));
                final_args = fill_missing_flag_values(&final_args, sdoc, task);
                final_args = replace_generic_values(&final_args, task);

                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                    eprintln!(
                        "{} [TwoStep] rule-direct (excellent): sub={:?} final='{}'",
                        "[verbose]".dimmed(),
                        selected_subcommand,
                        final_args.join(" ").chars().take(80).collect::<String>()
                    );
                }

                return Ok(LlmCommandSuggestion {
                    args: final_args,
                    explanation: String::new(),
                    inference_ms: 0.0,
                });
            }
        }

        let rule_hint = if !rule_assembled.is_empty() {
            format!(
                "\n\nSuggested base command (refine this):\n{}\n",
                rule_assembled.join(" ")
            )
        } else {
            String::new()
        };

        let step2_prompt = rule_hint.clone();

        let step2_system = if selected_subcommand.is_some() {
            "Generate CLI arguments. Respond with exactly:\nARGS: <complete arguments>\nEXPLANATION: <brief>\n\nRules:\n1. Start with the subcommand.\n2. Use ONLY flags from the list above.\n3. Include ALL required flags. Skip optional flags not mentioned in task.\n4. Extract EXACT file paths and values from the task.\n5. Include positional arguments (file paths, patterns, targets) after flags.\n6. Maximum 8 flags. Fewer is better.\n7. Do NOT include the tool name."
        } else if sdoc.has_subcommands {
            "Generate CLI arguments. Respond with exactly:\nARGS: <subcommand then flags and values>\nEXPLANATION: <brief>\n\nRules:\n1. First token MUST be a valid subcommand.\n2. Use ONLY flags from the list above.\n3. Include ALL required flags. Skip optional flags not mentioned in task.\n4. Extract EXACT file paths and values from the task.\n5. Include positional arguments (file paths, patterns, targets) after flags.\n6. Maximum 8 flags. Fewer is better."
        } else {
            "Generate CLI arguments. Respond with exactly:\nARGS: <flags and values>\nEXPLANATION: <brief>\n\nRules:\n1. Do NOT start with a subcommand - this tool has none.\n2. Use ONLY flags from the list above.\n3. Include ALL required flags. Skip optional flags not mentioned in task.\n4. Extract EXACT file paths and values from the task.\n5. Positional arguments (files, patterns, numbers) MUST be included after flags.\n6. Maximum 8 flags. Fewer is better."
        };

        let api_start = std::time::Instant::now();
        match self
            .request_with_system(step2_system, &step2_prompt, Some(768), temperature)
            .await
        {
            Ok(raw) => {
                let inference_ms = api_start.elapsed().as_secs_f64() * 1000.0;
                let parsed = match parse_response(&raw) {
                    Ok(s) => s,
                    Err(_) => {
                        let args_str = raw.trim().to_string();
                        LlmCommandSuggestion {
                            args: args_str.split_whitespace().map(String::from).collect(),
                            explanation: String::new(),
                            inference_ms: 0.0,
                        }
                    }
                };

                if parsed.args.is_empty() {
                    if !rule_assembled.is_empty() {
                        let corrected_args = apply_corrections_to_args(
                            &rule_assembled,
                            tool,
                            structured_doc,
                            Some(task),
                        );
                        return Ok(LlmCommandSuggestion {
                            args: corrected_args,
                            explanation: parsed.explanation,
                            inference_ms,
                        });
                    }
                    if let Some(ref tmpl_args) = template_args {
                        if !tmpl_args.is_empty() {
                            let corrected_args = apply_corrections_to_args(
                                tmpl_args,
                                tool,
                                structured_doc,
                                Some(task),
                            );
                            return Ok(LlmCommandSuggestion {
                                args: corrected_args,
                                explanation: parsed.explanation,
                                inference_ms,
                            });
                        }
                    }
                    if let Some(ref sub) = selected_subcommand {
                        let fallback_args = vec![sub.clone()];
                        let corrected_args = apply_corrections_to_args(
                            &fallback_args,
                            tool,
                            structured_doc,
                            Some(task),
                        );
                        return Ok(LlmCommandSuggestion {
                            args: corrected_args,
                            explanation: parsed.explanation,
                            inference_ms,
                        });
                    }
                    return self
                        .suggest_command(
                            tool,
                            documentation,
                            task,
                            skill,
                            no_prompt,
                            structured_doc,
                        )
                        .await;
                }

                let mut final_args = if let Some(ref sub) = selected_subcommand {
                    let args_joined = parsed.args.join(" ");
                    if args_joined
                        .to_ascii_lowercase()
                        .starts_with(&sub.to_ascii_lowercase())
                    {
                        parsed.args
                    } else {
                        let mut v = vec![sub.clone()];
                        v.extend(parsed.args);
                        v
                    }
                } else {
                    parsed.args
                };

                if is_no_subcommand_tool(tool) && !final_args.is_empty() {
                    let first = &final_args[0];
                    let first_lower = first.to_ascii_lowercase();
                    let is_known_companion = sdoc
                        .companion_binaries
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(&first_lower));
                    let is_known_prefix = first_lower == "rscript"
                        || first_lower == "perl"
                        || first_lower == "python"
                        || first_lower == "python3"
                        || first_lower == "bash"
                        || first_lower == "java"
                        || first_lower == "julia"
                        || first_lower == "mem"
                        || first_lower == "index"
                        || first_lower == "aln"
                        || first_lower == "sampe"
                        || first_lower == "samse"
                        || first_lower == "bwasw"
                        || first_lower == "bowtie2-build"
                        || first_lower == "bowtie2-inspect"
                        || first_lower == "hisat2-build"
                        || first_lower == "bismark_genome_preparation"
                        || first_lower == "deduplicate_bismark"
                        || first_lower == "bismark_methylation_extractor"
                        || first_lower == "bismark2report"
                        || first_lower == "medaka_consensus"
                        || first_lower == "medaka_variant"
                        || first_lower == "medaka_haploid_variant"
                        || first_lower == "medaka_inference"
                        || first_lower == "medaka_sequence"
                        || first_lower == "kraken2-build"
                        || first_lower == "bracken-build"
                        || first_lower == "centrifuge-build"
                        || first_lower == "centrifuge-kreport"
                        || first_lower == "rsem-calculate-expression"
                        || first_lower == "rsem-prepare-reference"
                        || first_lower == "emapper.py"
                        || first_lower == "merge_metaphlan_tables.py"
                        || first_lower == "strainphlan"
                        || first_lower == "metaquast.py"
                        || first_lower == "jgi_summarize_bam_contig_depths"
                        || first_lower == "normalize_by_kmer_coverage"
                        || first_lower == "run_bowtie2_for_trinity.pl"
                        || first_lower == "makeblastdb"
                        || first_lower == "blastn"
                        || first_lower == "blastp"
                        || first_lower == "blastx"
                        || first_lower == "tblastn"
                        || first_lower == "blastdbcmd"
                        || first_lower == "bakta_db"
                        || first_lower == "bakta_proteins"
                        || first_lower == "convert2bed"
                        || first_lower == "combine_bracken_outputs"
                        || first_lower == "findmotifsgenome.pl"
                        || first_lower == "annotatepeaks.pl"
                        || first_lower == "pos2bed.pl"
                        || first_lower == "makeucscfile"
                        || first_lower == "draw_fusions.r"
                        || first_lower == "convert_fusions_to_vcf"
                        || first_lower == "run_arriba"
                        || first_lower == "run_arriba_on_prealigned_bam"
                        || first_lower == "agat_convert_sp_gff2gtf"
                        || first_lower == "agat_sp_statistics"
                        || first_lower == "agat_sp_filter_gene_by_length"
                        || first_lower == "agat_convert_sp_gxf2gxf"
                        || first_lower == "agat_sp_extract_sequences"
                        || first_lower == "agat_sp_keep_longest_isoform"
                        || first_lower == "agat_sp_merge_annotations"
                        || first_lower == "agat_sp_manage_ids"
                        || first_lower == "agat_convert_sp_gff2bed";
                    if !first.starts_with('-')
                        && !first.contains('.')
                        && !first.contains('/')
                        && !is_known_companion
                        && !is_known_prefix
                    {
                        final_args.remove(0);
                    }
                }

                if let Some(ref tmpl_args) = template_args {
                    if !tmpl_args.is_empty() {
                        if is_small_model {
                            final_args = merge_llm_into_template(tmpl_args, &final_args, sdoc);
                        } else {
                            final_args = merge_template_with_llm(tmpl_args, &final_args, sdoc);
                        }
                        final_args = remove_duplicate_flags_vec(&final_args);
                    }
                }

                final_args =
                    apply_corrections_to_args(&final_args, tool, structured_doc, Some(task));

                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                    eprintln!(
                        "{} [TwoStep] sub={:?} final='{}'",
                        "[verbose]".dimmed(),
                        selected_subcommand,
                        final_args.join(" ").chars().take(80).collect::<String>()
                    );
                }

                Ok(LlmCommandSuggestion {
                    args: final_args,
                    explanation: parsed.explanation,
                    inference_ms,
                })
            }
            Err(e) => {
                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                    eprintln!(
                        "{} [TwoStep] LLM error: {}, fallback",
                        "[verbose]".dimmed(),
                        e
                    );
                }
                if !rule_assembled.is_empty() {
                    let corrected_args = apply_corrections_to_args(
                        &rule_assembled,
                        tool,
                        structured_doc,
                        Some(task),
                    );
                    return Ok(LlmCommandSuggestion {
                        args: corrected_args,
                        explanation: String::new(),
                        inference_ms: 0.0,
                    });
                }
                if let Some(ref tmpl_args) = template_args {
                    if !tmpl_args.is_empty() {
                        let corrected_args =
                            apply_corrections_to_args(tmpl_args, tool, structured_doc, Some(task));
                        return Ok(LlmCommandSuggestion {
                            args: corrected_args,
                            explanation: String::new(),
                            inference_ms: 0.0,
                        });
                    }
                }
                self.suggest_command(tool, documentation, task, skill, no_prompt, structured_doc)
                    .await
            }
        }
    }

    async fn llm_select_subcommand(
        &self,
        tool: &str,
        task: &str,
        sdoc: &StructuredDoc,
        temperature: Option<f32>,
    ) -> Result<Option<String>> {
        let step1_prompt = format!(
            "Tool: {}\nTask: {}\n\nAvailable subcommands:\n{}\n\nRespond with ONLY the subcommand name.",
            tool,
            task,
            sdoc.subcommands
                .iter()
                .take(20)
                .enumerate()
                .map(|(i, s)| {
                    let desc = sdoc
                        .subcommand_descriptions
                        .iter()
                        .find(|(sub, _)| sub == s)
                        .and_then(|(_, d)| if d.is_empty() { None } else { Some(d.as_str()) })
                        .unwrap_or("");
                    if desc.is_empty() {
                        format!("{}. {}", i + 1, s)
                    } else {
                        format!(
                            "{}. {} - {}",
                            i + 1,
                            s,
                            desc.chars().take(60).collect::<String>()
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
        match self.request_with_system("Select the correct subcommand. Respond with ONLY the subcommand name from the list. No explanation.", &step1_prompt, Some(32), temperature).await {
            Ok(raw) => {
                let cleaned = raw.trim()
                    .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ')
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !cleaned.is_empty() {
                    Ok(sdoc.subcommands.iter()
                        .find(|s| s.eq_ignore_ascii_case(&cleaned))
                        .cloned()
                        .or_else(|| {
                            let cleaned_lower = cleaned.to_ascii_lowercase();
                            sdoc.subcommands.iter()
                                .filter_map(|s| {
                                    let s_lower = s.to_ascii_lowercase();
                                    if cleaned_lower.contains(&s_lower) || s_lower.contains(&cleaned_lower) {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .next()
                        }))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn suggest_command(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
    ) -> Result<LlmCommandSuggestion> {
        const MAX_RETRIES: usize = 2;
        let overall_start = std::time::Instant::now();
        let mut trace = crate::diagnostic::GenerationTrace::new(tool, task);

        let context_window = self.config.effective_context_window();
        let tier = self.config.effective_prompt_tier();
        let model = self.config.effective_model();
        let profile = crate::config::get_model_profile(&model);
        let temperature = Some(profile.optimal_temperature);

        let tool_specific_subcmd = detect_subcommand_for_tool(tool, task, &[]);

        let rule_subcommand: Option<String> = if let Some(sdoc) = structured_doc {
            if is_no_subcommand_tool(tool) {
                None
            } else if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
                let result = if tool_specific_subcmd.is_some()
                    && !tool_specific_subcmd
                        .as_deref()
                        .unwrap_or("")
                        .starts_with("_NO_SUB_")
                {
                    tool_specific_subcmd.clone()
                } else {
                    rule_based_subcommand_match(
                        task,
                        &sdoc.subcommands,
                        &sdoc.subcommand_descriptions,
                    )
                };
                if let Some(ref sub) = result {
                    trace.record(
                        "rule_subcommand",
                        task,
                        sub,
                        crate::diagnostic::DecisionSource::Rule,
                        0,
                    );
                }
                result
            } else {
                None
            }
        } else {
            None
        };

        let template_subcommand: Option<String> = if let Some(sdoc) = structured_doc {
            if let Some(template) = find_best_template(tool, task) {
                let template_args = crate::llm::response::parse_shell_args(template);
                if !template_args.is_empty() {
                    let first = &template_args[0];
                    if !first.starts_with('-') && !first.contains('.') && !first.contains('/') {
                        if sdoc.has_subcommands
                            && sdoc
                                .subcommands
                                .iter()
                                .any(|s| s.to_ascii_lowercase() == first.to_ascii_lowercase())
                        {
                            Some(first.clone())
                        } else if !sdoc.has_subcommands {
                            Some(first.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let template_base_args: Option<Vec<String>> = if let Some(sdoc) = structured_doc {
            if let Some(template) = find_best_template(tool, task) {
                let task_values = extract_task_values(task);
                let args = fill_template(template, task, &task_values);
                if !args.is_empty() {
                    if !sdoc.has_subcommands && !args[0].starts_with('-') {
                        let first_lower = args[0].to_ascii_lowercase();
                        let looks_like_flag = first_lower.starts_with('-');
                        let looks_like_file =
                            first_lower.contains('.') || first_lower.contains('/');
                        let known_subcommand = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == first_lower);
                        if !looks_like_flag && !looks_like_file && !known_subcommand {
                            let mut fixed = args;
                            fixed.remove(0);
                            Some(fixed)
                        } else {
                            Some(args)
                        }
                    } else {
                        Some(args)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
            eprintln!(
                "[verbose] Template subcommand hint for {}: {:?}",
                tool, template_subcommand
            );
        }

        if skill.is_none() {
            if let Some(sdoc) = structured_doc {
                let is_small_model = crate::config::infer_model_parameter_count(&model)
                    .map(|p| p <= 8.0)
                    .unwrap_or(false);

                let tmpl_result = generate_from_template_with_score(tool, task, sdoc);

                if is_small_model {
                    let task_values = extract_task_values(task);
                    let effective_subcmd = if tool_specific_subcmd.is_some()
                        && !tool_specific_subcmd
                            .as_deref()
                            .unwrap_or("")
                            .starts_with("_NO_SUB_")
                    {
                        tool_specific_subcmd.clone()
                    } else if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
                        rule_based_subcommand_match(
                            task,
                            &sdoc.subcommands,
                            &sdoc.subcommand_descriptions,
                        )
                    } else {
                        None
                    };
                    let rule_assembled = assemble_command_from_rules(
                        tool,
                        task,
                        sdoc,
                        effective_subcmd.as_deref(),
                        &task_values,
                    );

                    let needs_llm_for_pattern = tool == "grep"
                        || tool == "awk"
                        || tool == "sed"
                        || tool == "perl"
                        || tool == "python"
                        || tool == "r"
                        || tool == "bash"
                        || tool == "julia"
                        || tool == "java"
                        || tool == "find"
                        || tool == "ssh";

                    let tmpl_match_score = tmpl_result
                        .as_ref()
                        .map(|(_, score, _)| *score)
                        .unwrap_or(0);
                    let tmpl_is_high_quality =
                        tmpl_result.as_ref().map(|(_, _, hq)| *hq).unwrap_or(false);

                    if tmpl_is_high_quality || tmpl_match_score >= 8 {
                        if let Some((tmpl_args, _match_score, _hq)) = tmpl_result.as_ref() {
                            if !tmpl_args.is_empty() {
                                let tmpl_has_flags = tmpl_args.iter().any(|a| a.starts_with('-'));
                                if tmpl_has_flags || tmpl_args.len() >= 2 {
                                    let mut final_args = tmpl_args.clone();
                                    final_args = remove_duplicate_flags_vec(&final_args);
                                    final_args = apply_template_corrections(
                                        &final_args,
                                        tool,
                                        structured_doc,
                                        Some(task),
                                    );
                                    final_args = apply_tool_specific_corrections(
                                        &final_args,
                                        tool,
                                        Some(task),
                                    );
                                    final_args = fix_output_extensions(&final_args, tool, task);
                                    trace.record(
                                        "template_direct",
                                        task,
                                        &final_args.join(" "),
                                        crate::diagnostic::DecisionSource::Template,
                                        0,
                                    );
                                    trace.set_final(
                                        &final_args.join(" "),
                                        overall_start.elapsed().as_millis() as u64,
                                    );
                                    trace.emit();
                                    if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                        eprintln!(
                                            "[verbose] Template-direct path (small model, high quality, score={}): using template for {}",
                                            tmpl_match_score, tool
                                        );
                                    }
                                    return Ok(LlmCommandSuggestion {
                                        args: final_args,
                                        explanation: String::new(),
                                        inference_ms: 0.0,
                                    });
                                }
                            }
                        }
                    }

                    if !rule_assembled.is_empty()
                        && !needs_llm_for_pattern
                        && rule_assembled.len() >= 2
                    {
                        let rule_has_subcmd = sdoc.has_subcommands
                            && !rule_assembled[0].starts_with('-')
                            && sdoc.subcommands.iter().any(|s| {
                                s.to_ascii_lowercase() == rule_assembled[0].to_ascii_lowercase()
                            });
                        let rule_has_flags = rule_assembled.iter().any(|a| a.starts_with('-'));
                        let rule_has_real_files = rule_assembled.iter().any(|a| {
                            a.contains('.')
                                && !a.starts_with('-')
                                && a != "input.bam"
                                && a != "output.bam"
                                && a != "reads_1.fq"
                                && a != "reads_2.fq"
                                && a != "reference.fa"
                                && a != "input2.bed"
                                && a != "annotation.gtf"
                                && a != "database"
                                && a != "metrics.txt"
                                && a != "tool.jar"
                                && a != "*.bam"
                                && a != "SRR123456"
                                && !a.starts_with("/path/to/")
                        });

                        if (rule_has_subcmd || !sdoc.has_subcommands)
                            && (rule_has_flags || rule_has_real_files)
                        {
                            let mut final_args = rule_assembled;
                            final_args = add_missing_required_flags(&final_args, sdoc, task);
                            final_args = add_task_implied_flags(&final_args, sdoc, task);
                            final_args = limit_flag_count(&final_args, sdoc, task);
                            final_args = remove_duplicate_flags_vec(&final_args);
                            final_args = apply_corrections_to_args(
                                &final_args,
                                tool,
                                structured_doc,
                                Some(task),
                            );
                            final_args = fill_missing_flag_values(&final_args, sdoc, task);
                            final_args = replace_generic_values(&final_args, task);
                            trace.record(
                                "rule_engine_direct",
                                task,
                                &final_args.join(" "),
                                crate::diagnostic::DecisionSource::Rule,
                                0,
                            );
                            trace.set_final(
                                &final_args.join(" "),
                                overall_start.elapsed().as_millis() as u64,
                            );
                            trace.emit();
                            if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                eprintln!(
                                    "[verbose] Rule-engine-direct path (small model, low tmpl score={}): using rules for {}",
                                    tmpl_match_score, tool
                                );
                            }
                            return Ok(LlmCommandSuggestion {
                                args: final_args,
                                explanation: String::new(),
                                inference_ms: 0.0,
                            });
                        }
                    }

                    if let Some((tmpl_args, _match_score, _hq)) = tmpl_result.as_ref() {
                        if !tmpl_args.is_empty() {
                            let tmpl_has_flags = tmpl_args.iter().any(|a| a.starts_with('-'));
                            if tmpl_has_flags || tmpl_args.len() >= 2 {
                                let mut final_args = tmpl_args.clone();
                                final_args = remove_duplicate_flags_vec(&final_args);
                                final_args = apply_template_corrections(
                                    &final_args,
                                    tool,
                                    structured_doc,
                                    Some(task),
                                );
                                final_args =
                                    apply_tool_specific_corrections(&final_args, tool, Some(task));
                                final_args = fix_output_extensions(&final_args, tool, task);
                                trace.record(
                                    "template_direct",
                                    task,
                                    &final_args.join(" "),
                                    crate::diagnostic::DecisionSource::Template,
                                    0,
                                );
                                trace.set_final(
                                    &final_args.join(" "),
                                    overall_start.elapsed().as_millis() as u64,
                                );
                                trace.emit();
                                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                    eprintln!(
                                        "[verbose] Template-direct path (small model, fallback, score={}): using template for {}",
                                        tmpl_match_score, tool
                                    );
                                }
                                return Ok(LlmCommandSuggestion {
                                    args: final_args,
                                    explanation: String::new(),
                                    inference_ms: 0.0,
                                });
                            }
                        }
                    }
                } else {
                    if let Some((tmpl_args, _match_score, _hq)) = tmpl_result {
                        if !tmpl_args.is_empty() {
                            let tmpl_has_flags = tmpl_args.iter().any(|a| a.starts_with('-'));
                            if tmpl_has_flags || tmpl_args.len() >= 2 {
                                let mut final_args = tmpl_args.clone();
                                final_args = remove_duplicate_flags_vec(&final_args);
                                final_args = apply_template_corrections(
                                    &final_args,
                                    tool,
                                    structured_doc,
                                    Some(task),
                                );
                                final_args =
                                    apply_tool_specific_corrections(&final_args, tool, Some(task));
                                final_args = fix_output_extensions(&final_args, tool, task);
                                trace.record(
                                    "template_direct",
                                    task,
                                    &final_args.join(" "),
                                    crate::diagnostic::DecisionSource::Template,
                                    0,
                                );
                                trace.set_final(
                                    &final_args.join(" "),
                                    overall_start.elapsed().as_millis() as u64,
                                );
                                trace.emit();
                                return Ok(LlmCommandSuggestion {
                                    args: final_args,
                                    explanation: String::new(),
                                    inference_ms: 0.0,
                                });
                            }
                        }
                    }
                }

                if !sdoc.extracted_examples.is_empty() {
                    let task_values = extract_task_values(task);
                    if let Some(example_args) =
                        find_best_example(tool, task, &sdoc.extracted_examples, &task_values)
                    {
                        let mut final_args = example_args;
                        final_args = limit_flag_count(&final_args, sdoc, task);
                        final_args = add_missing_required_flags(&final_args, sdoc, task);
                        final_args = add_task_implied_flags(&final_args, sdoc, task);
                        final_args = remove_duplicate_flags_vec(&final_args);
                        final_args = apply_tool_specific_corrections(&final_args, tool, Some(task));
                        final_args = fix_output_extensions(&final_args, tool, task);
                        trace.record(
                            "example_direct",
                            task,
                            &final_args.join(" "),
                            crate::diagnostic::DecisionSource::Template,
                            0,
                        );
                        trace.set_final(
                            &final_args.join(" "),
                            overall_start.elapsed().as_millis() as u64,
                        );
                        trace.emit();
                        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                            eprintln!(
                                "[verbose] Example-first path: using extracted example for {}",
                                tool
                            );
                        }
                        return Ok(LlmCommandSuggestion {
                            args: final_args,
                            explanation: String::new(),
                            inference_ms: 0.0,
                        });
                    }
                }

                if !is_small_model {
                    let task_values = extract_task_values(task);
                    let rule_subcmd = if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
                        rule_based_subcommand_match(
                            task,
                            &sdoc.subcommands,
                            &sdoc.subcommand_descriptions,
                        )
                    } else {
                        None
                    };
                    let assembled = assemble_command_from_rules(
                        tool,
                        task,
                        sdoc,
                        rule_subcmd.as_deref(),
                        &task_values,
                    );
                    if !assembled.is_empty() {
                        let has_subcmd = sdoc.has_subcommands
                            && !assembled.is_empty()
                            && !assembled[0].starts_with('-')
                            && sdoc.subcommands.iter().any(|s| {
                                s.to_ascii_lowercase() == assembled[0].to_ascii_lowercase()
                            });
                        let has_flags = assembled.iter().any(|a| a.starts_with('-'));
                        let has_files = assembled
                            .iter()
                            .any(|a| a.contains('.') && !a.starts_with('-'));

                        let should_use_rule =
                            (has_subcmd || !sdoc.has_subcommands) && (has_flags || has_files);

                        if should_use_rule {
                            let mut final_args = assembled;
                            final_args = add_missing_required_flags(&final_args, sdoc, task);
                            final_args = add_task_implied_flags(&final_args, sdoc, task);
                            final_args = limit_flag_count(&final_args, sdoc, task);
                            final_args = remove_duplicate_flags_vec(&final_args);
                            final_args = apply_corrections_to_args(
                                &final_args,
                                tool,
                                structured_doc,
                                Some(task),
                            );
                            final_args = fill_missing_flag_values(&final_args, sdoc, task);
                            final_args = replace_generic_values(&final_args, task);
                            trace.record(
                                "rule_engine_direct",
                                task,
                                &final_args.join(" "),
                                crate::diagnostic::DecisionSource::Rule,
                                0,
                            );
                            trace.set_final(
                                &final_args.join(" "),
                                overall_start.elapsed().as_millis() as u64,
                            );
                            trace.emit();
                            return Ok(LlmCommandSuggestion {
                                args: final_args,
                                explanation: String::new(),
                                inference_ms: 0.0,
                            });
                        }
                    }
                }
            }
        }

        let docs_hash = if documentation.is_empty() {
            None
        } else {
            Some(hex::encode(sha2::Sha256::digest(documentation.as_bytes())))
        };
        let skill_name = skill.map(|s| s.meta.name.clone());

        if let Ok(Some(cached)) = crate::cache::LlmCache::lookup(
            tool,
            task,
            docs_hash.as_deref(),
            skill_name.as_deref(),
            &model,
        ) {
            let args_vec: Vec<String> = cached.args.split_whitespace().map(String::from).collect();
            let mut corrected_args =
                apply_corrections_to_args(&args_vec, tool, structured_doc, Some(task));
            corrected_args = apply_tool_specific_corrections(&corrected_args, tool, Some(task));
            corrected_args = fix_output_extensions(&corrected_args, tool, task);
            return Ok(LlmCommandSuggestion {
                args: corrected_args,
                explanation: cached.explanation,
                inference_ms: 0.0,
            });
        }

        let mut last_raw = String::new();
        let mut total_inference_ms: f64 = 0.0;
        let mut had_empty_output = false;

        for attempt in 0..=MAX_RETRIES {
            let effective_docs = if had_empty_output && attempt > 0 {
                ""
            } else {
                documentation
            };

            let user_prompt = if attempt == 0 {
                build_prompt(
                    tool,
                    effective_docs,
                    task,
                    skill,
                    no_prompt,
                    context_window,
                    tier,
                    structured_doc,
                )
            } else if had_empty_output {
                build_prompt(
                    tool,
                    effective_docs,
                    task,
                    skill,
                    no_prompt,
                    context_window,
                    tier,
                    structured_doc,
                )
            } else {
                build_retry_prompt(
                    tool,
                    effective_docs,
                    task,
                    skill,
                    &last_raw,
                    no_prompt,
                    context_window,
                    tier,
                )
            };

            let api_start = std::time::Instant::now();
            if std::env::var("OXO_CALL_DEBUG_PROMPT").is_ok() {
                eprintln!(
                    "[DEBUG PROMPT] === System Prompt ===\n{}",
                    match tier {
                        PromptTier::Slim => system_prompt_unified(),
                        PromptTier::Full => system_prompt(),
                    }
                );
                eprintln!("[DEBUG PROMPT] === User Prompt ===\n{}", user_prompt);
            }
            let raw = {
                let mut result = self
                    .call_api(&user_prompt, no_prompt, tier, temperature)
                    .await;
                let is_retryable = |e: &OxoError| {
                    let msg = e.to_string();
                    msg.contains("429")
                        || msg.contains("Stream read error")
                        || msg.contains("error sending request")
                        || msg.contains("connection reset")
                        || msg.contains("timed out")
                        || msg.contains("502")
                        || msg.contains("503")
                };
                for retry in 0..3 {
                    match &result {
                        Err(e) if is_retryable(e) => {
                            let delay = std::time::Duration::from_secs(2u64.pow(retry as u32 + 1));
                            eprintln!(
                                "{} API error (retry {}/3 after {:?}): {}",
                                "[warn]".yellow(),
                                retry + 1,
                                delay,
                                e
                            );
                            tokio::time::sleep(delay).await;
                            result = self
                                .call_api(&user_prompt, no_prompt, tier, temperature)
                                .await;
                        }
                        _ => break,
                    }
                }
                result?
            };
            total_inference_ms += api_start.elapsed().as_secs_f64() * 1000.0;

            if raw.trim().is_empty() {
                had_empty_output = true;
            }

            let mut suggestion = parse_response(&raw)?;
            suggestion.inference_ms = total_inference_ms;

            suggestion.args = sanitize_args(tool, suggestion.args);

            if let Some(sdoc) = structured_doc {
                if !sdoc.flag_catalog.is_empty() {
                    let before_validation = suggestion.args.clone();
                    suggestion.args = validate_flags_against_catalog(
                        &suggestion.args,
                        &sdoc.flag_catalog,
                        &sdoc.quick_flags,
                    );

                    let removed_flags: Vec<String> = before_validation
                        .iter()
                        .filter(|a| a.starts_with('-') && !suggestion.args.contains(a))
                        .cloned()
                        .collect();

                    if removed_flags.len() >= 2 && attempt < MAX_RETRIES {
                        let valid_flags_list: Vec<String> = sdoc
                            .flag_catalog
                            .iter()
                            .take(20)
                            .map(|e| e.flag.clone())
                            .collect();
                        let correction_prompt = format!(
                            "Your previous command had INVALID flags: {}\n\
                             These flags do NOT exist for this tool.\n\
                             Valid flags: {}\n\
                             Task: {}\n\n\
                             Fix the command. Replace invalid flags with valid ones or remove them.\n\
                             Output JSON: {{\"subcommand\":\"\",\"flags\":{{}},\"positional_args\":[],\"explanation\":\"\"}}",
                            removed_flags.join(", "),
                            valid_flags_list.join(", "),
                            task,
                        );
                        match self
                            .request_with_system(
                                system_prompt_unified(),
                                &correction_prompt,
                                Some(512),
                                temperature,
                            )
                            .await
                        {
                            Ok(correction_raw) => {
                                let corrected = parse_response(&correction_raw);
                                if let Ok(mut corr_suggestion) = corrected {
                                    corr_suggestion.args =
                                        sanitize_args(tool, corr_suggestion.args);
                                    corr_suggestion.args = validate_flags_against_catalog(
                                        &corr_suggestion.args,
                                        &sdoc.flag_catalog,
                                        &sdoc.quick_flags,
                                    );
                                    if !corr_suggestion.args.is_empty() {
                                        suggestion = corr_suggestion;
                                        suggestion.inference_ms = total_inference_ms;
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }

                let is_small_for_filter = crate::config::infer_model_parameter_count(&model)
                    .map(|p| p <= 8.0)
                    .unwrap_or(false);
                if is_small_for_filter {
                    suggestion.args =
                        filter_irrelevant_flags_for_small_model(&suggestion.args, sdoc, task);
                }

                let args_str = suggestion.args.join(" ");
                let corrected = correct_format(&args_str, sdoc);
                if corrected != args_str {
                    trace.record(
                        "correct_format",
                        &args_str,
                        &corrected,
                        crate::diagnostic::DecisionSource::PostProcess,
                        0,
                    );
                }
                suggestion.args = crate::llm::response::parse_shell_args(&corrected);

                // ── 100% subcommand enforcement ────────────────────────
                // When the tool requires a subcommand, force the best match
                // from the pre-processor, overriding any LLM error.
                if sdoc.has_subcommands
                    && !sdoc.subcommands.is_empty()
                    && !suggestion.args.is_empty()
                {
                    let best_sub = crate::llm::prompt::find_best_subcommand_for_task(task, sdoc);
                    let first = &suggestion.args[0];
                    let first_is_flag = first.starts_with('-');
                    let first_in_subs = sdoc
                        .subcommands
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(first));

                    if let Some(ref forced) = best_sub {
                        if first_is_flag {
                            // LLM skipped subcommand — insert it
                            suggestion.args.insert(0, forced.clone());
                        } else if !first_in_subs {
                            // LLM hallucinated subcommand — replace it
                            suggestion.args[0] = forced.clone();
                        } else if !forced.eq_ignore_ascii_case(first) {
                            // LLM picked wrong valid subcommand — force best
                            suggestion.args[0] = forced.clone();
                        }
                    } else if first_is_flag {
                        // No match found but tool needs subcommand — use first available
                        if let Some(fallback) = sdoc.subcommands.first() {
                            suggestion.args.insert(0, fallback.clone());
                        }
                    }
                }

                if is_no_subcommand_tool(tool) && !suggestion.args.is_empty() {
                    let first = &suggestion.args[0];
                    let first_lower = first.to_ascii_lowercase();
                    let is_known_companion = sdoc
                        .companion_binaries
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(&first_lower));
                    let is_known_prefix = first_lower == "rscript"
                        || first_lower == "perl"
                        || first_lower == "python"
                        || first_lower == "python3"
                        || first_lower == "bash"
                        || first_lower == "java"
                        || first_lower == "julia"
                        || first_lower == "mem"
                        || first_lower == "index"
                        || first_lower == "aln"
                        || first_lower == "sampe"
                        || first_lower == "samse"
                        || first_lower == "bwasw"
                        || first_lower == "bowtie2-build"
                        || first_lower == "bowtie2-inspect"
                        || first_lower == "hisat2-build"
                        || first_lower == "bismark_genome_preparation"
                        || first_lower == "deduplicate_bismark"
                        || first_lower == "bismark_methylation_extractor"
                        || first_lower == "bismark2report"
                        || first_lower == "medaka_consensus"
                        || first_lower == "medaka_variant"
                        || first_lower == "medaka_haploid_variant"
                        || first_lower == "medaka_inference"
                        || first_lower == "medaka_sequence"
                        || first_lower == "kraken2-build"
                        || first_lower == "bracken-build"
                        || first_lower == "centrifuge-build"
                        || first_lower == "centrifuge-kreport"
                        || first_lower == "rsem-calculate-expression"
                        || first_lower == "rsem-prepare-reference"
                        || first_lower == "emapper.py"
                        || first_lower == "merge_metaphlan_tables.py"
                        || first_lower == "strainphlan"
                        || first_lower == "metaquast.py"
                        || first_lower == "jgi_summarize_bam_contig_depths"
                        || first_lower == "normalize_by_kmer_coverage"
                        || first_lower == "run_bowtie2_for_trinity.pl"
                        || first_lower == "makeblastdb"
                        || first_lower == "blastn"
                        || first_lower == "blastp"
                        || first_lower == "blastx"
                        || first_lower == "tblastn"
                        || first_lower == "blastdbcmd"
                        || first_lower == "bakta_db"
                        || first_lower == "bakta_proteins"
                        || first_lower == "convert2bed"
                        || first_lower == "combine_bracken_outputs"
                        || first_lower == "findmotifsgenome.pl"
                        || first_lower == "annotatepeaks.pl"
                        || first_lower == "pos2bed.pl"
                        || first_lower == "makeucscfile"
                        || first_lower == "draw_fusions.r"
                        || first_lower == "convert_fusions_to_vcf"
                        || first_lower == "run_arriba"
                        || first_lower == "run_arriba_on_prealigned_bam"
                        || first_lower == "agat_convert_sp_gff2gtf"
                        || first_lower == "agat_sp_statistics"
                        || first_lower == "agat_sp_filter_gene_by_length"
                        || first_lower == "agat_convert_sp_gxf2gxf"
                        || first_lower == "agat_sp_extract_sequences"
                        || first_lower == "agat_sp_keep_longest_isoform"
                        || first_lower == "agat_sp_merge_annotations"
                        || first_lower == "agat_sp_manage_ids"
                        || first_lower == "agat_convert_sp_gff2bed";
                    if !first.starts_with('-')
                        && !first.contains('.')
                        && !first.contains('/')
                        && !is_known_companion
                        && !is_known_prefix
                    {
                        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                            eprintln!(
                                "[verbose] is_no_subcommand_tool: stripping hallucinated subcommand '{}' for {}",
                                first, tool
                            );
                        }
                        suggestion.args.remove(0);
                    }
                }

                // ── Aggressive subcommand enforcement ──────────────────────
                // Fixes ~70% of subcommand errors by forcing the correct
                // subcommand when the LLM picks wrong or skips it entirely.
                if sdoc.has_subcommands
                    && !sdoc.subcommands.is_empty()
                    && !suggestion.args.is_empty()
                {
                    let best_sub = crate::llm::prompt::find_best_subcommand_for_task(task, sdoc);
                    let first = &suggestion.args[0];
                    let first_is_flag = first.starts_with('-');
                    let first_is_sub = !first_is_flag
                        && sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.eq_ignore_ascii_case(first));

                    if let Some(ref forced) = best_sub {
                        // Case 1: LLM started with a flag — missing subcommand entirely
                        if first_is_flag {
                            suggestion.args.insert(0, forced.clone());
                        }
                        // Case 2: LLM picked a different valid subcommand
                        else if first_is_sub && !forced.eq_ignore_ascii_case(first) {
                            suggestion.args[0] = forced.clone();
                        }
                        // Case 3: LLM's first token is not a valid subcommand
                        else if !first_is_sub {
                            suggestion.args[0] = forced.clone();
                        }
                    } else if first_is_flag {
                        // No best match found but tool requires subcommand —
                        // use the first available subcommand as fallback
                        if let Some(first_sub) = sdoc.subcommands.first() {
                            suggestion.args.insert(0, first_sub.clone());
                        }
                    }
                }

                if let Some(ref tool_sub) = tool_specific_subcmd {
                    if !tool_sub.starts_with("_NO_SUB_") && !suggestion.args.is_empty() {
                        let sub_parts: Vec<&str> = tool_sub.split_whitespace().collect();
                        if !sub_parts.is_empty() {
                            let sub_main = sub_parts[0];
                            let first = &suggestion.args[0];
                            let first_lower = first.to_ascii_lowercase();
                            let sub_main_lower = sub_main.to_ascii_lowercase();

                            let is_valid_sub = sdoc
                                .subcommands
                                .iter()
                                .any(|s| s.to_ascii_lowercase() == first_lower);
                            let sub_is_valid = sdoc
                                .subcommands
                                .iter()
                                .any(|s| s.to_ascii_lowercase() == sub_main_lower);

                            if !first.starts_with('-')
                                && (!is_valid_sub
                                    || (sub_is_valid && first_lower != sub_main_lower))
                            {
                                if sub_is_valid {
                                    if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                        eprintln!(
                                            "[verbose] Tool-specific subcommand override: {} -> {} for {}",
                                            first, sub_main, tool
                                        );
                                    }
                                    suggestion.args[0] = sub_main.to_string();
                                }
                            } else if first.starts_with('-') && sub_is_valid {
                                suggestion.args.insert(0, sub_main.to_string());
                                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                    eprintln!(
                                        "[verbose] Tool-specific subcommand insert: {} for {}",
                                        sub_main, tool
                                    );
                                }
                            }

                            if sub_parts.len() > 1 {
                                let flag_part = &sub_parts[1];
                                let flag_lower = flag_part.to_ascii_lowercase();
                                if flag_lower.starts_with("-") {
                                    let already_has = suggestion.args.iter().any(|a| {
                                        a.split('=').next().unwrap_or(a).to_ascii_lowercase()
                                            == flag_lower.split('=').next().unwrap_or(&flag_lower)
                                    });
                                    if !already_has {
                                        if sub_parts.len() > 2 {
                                            suggestion.args.insert(1, flag_part.to_string());
                                            suggestion.args.insert(2, sub_parts[2].to_string());
                                        } else {
                                            suggestion.args.insert(1, flag_part.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    } else if tool_sub.starts_with("_NO_SUB_") && !suggestion.args.is_empty() {
                        let first = &suggestion.args[0];
                        if !first.starts_with('-') && !first.contains('.') && !first.contains('/') {
                            if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                eprintln!(
                                    "[verbose] _NO_SUB_ stripping subcommand '{}' for {}",
                                    first, tool
                                );
                            }
                            suggestion.args.remove(0);
                        }
                    }
                }

                if template_base_args.is_some() {
                    if let Some(ref tmpl_args) = template_base_args {
                        let tmpl_starts_with_flag = tmpl_args
                            .first()
                            .map(|f| f.starts_with('-'))
                            .unwrap_or(true);
                        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                            eprintln!(
                                "[verbose] Template strip check: tmpl_starts_with_flag={}, first_arg={:?}, tmpl_first={:?}",
                                tmpl_starts_with_flag,
                                suggestion.args.first(),
                                tmpl_args.first()
                            );
                        }
                        if tmpl_starts_with_flag
                            && !suggestion.args.is_empty()
                            && !suggestion.args[0].starts_with('-')
                        {
                            if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                eprintln!(
                                    "[verbose] Stripping hallucinated subcommand: {}",
                                    suggestion.args[0]
                                );
                            }
                            suggestion.args.remove(0);
                        }
                    }
                }

                if let Some(ref tmpl_sub) = template_subcommand {
                    if !suggestion.args.is_empty() && sdoc.has_subcommands {
                        let first = &suggestion.args[0];
                        let first_lower = first.to_ascii_lowercase();
                        let tmpl_lower = tmpl_sub.to_ascii_lowercase();
                        let is_valid_sub = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == first_lower);
                        let tmpl_valid = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == tmpl_lower);

                        if !is_valid_sub && tmpl_valid {
                            suggestion.args[0] = tmpl_sub.clone();
                        } else if is_valid_sub && tmpl_valid && first_lower != tmpl_lower {
                            let tmpl_desc_match = sdoc
                                .subcommand_descriptions
                                .iter()
                                .find(|(s, _)| s.eq_ignore_ascii_case(&tmpl_lower))
                                .and_then(|(_, d)| {
                                    let d_lower = d.to_ascii_lowercase();
                                    let task_lower = task.to_ascii_lowercase();
                                    let match_count = task_lower
                                        .split_whitespace()
                                        .filter(|w| w.len() >= 3 && d_lower.contains(w))
                                        .count();
                                    if match_count >= 1 {
                                        Some(match_count)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(1);

                            let first_desc_match = sdoc
                                .subcommand_descriptions
                                .iter()
                                .find(|(s, _)| s.eq_ignore_ascii_case(&first_lower))
                                .and_then(|(_, d)| {
                                    let d_lower = d.to_ascii_lowercase();
                                    let task_lower = task.to_ascii_lowercase();
                                    let match_count = task_lower
                                        .split_whitespace()
                                        .filter(|w| w.len() >= 3 && d_lower.contains(w))
                                        .count();
                                    if match_count >= 1 {
                                        Some(match_count)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0);

                            if tmpl_desc_match >= first_desc_match {
                                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                    eprintln!(
                                        "[verbose] Template subcommand override: {} -> {} (tmpl_desc={}, first_desc={})",
                                        first, tmpl_sub, tmpl_desc_match, first_desc_match
                                    );
                                }
                                suggestion.args[0] = tmpl_sub.clone();
                            }
                        }
                    } else if suggestion.args.is_empty() && sdoc.has_subcommands {
                        let tmpl_lower = tmpl_sub.to_ascii_lowercase();
                        let tmpl_valid = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == tmpl_lower);
                        if tmpl_valid {
                            suggestion.args.insert(0, tmpl_sub.clone());
                        }
                    }
                }

                if let Some(ref rule_sub) = rule_subcommand {
                    if !suggestion.args.is_empty() && sdoc.has_subcommands {
                        let first = &suggestion.args[0];
                        let first_lower = first.to_ascii_lowercase();
                        let rule_lower = rule_sub.to_ascii_lowercase();
                        let is_valid_sub = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == first_lower);
                        let rule_valid = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == rule_lower);

                        if !is_valid_sub && rule_valid {
                            if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                eprintln!(
                                    "[verbose] Rule subcommand override (invalid->valid): {} -> {}",
                                    first, rule_sub
                                );
                            }
                            suggestion.args[0] = rule_sub.clone();
                        } else if is_valid_sub && rule_valid && first_lower != rule_lower {
                            let rule_desc_match = sdoc
                                .subcommand_descriptions
                                .iter()
                                .find(|(s, _)| s.eq_ignore_ascii_case(&rule_lower))
                                .and_then(|(_, d)| {
                                    let d_lower = d.to_ascii_lowercase();
                                    let task_lower = task.to_ascii_lowercase();
                                    let match_count = task_lower
                                        .split_whitespace()
                                        .filter(|w| w.len() >= 3 && d_lower.contains(w))
                                        .count();
                                    if match_count >= 2 {
                                        Some(match_count)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0);

                            let first_desc_match = sdoc
                                .subcommand_descriptions
                                .iter()
                                .find(|(s, _)| s.eq_ignore_ascii_case(&first_lower))
                                .and_then(|(_, d)| {
                                    let d_lower = d.to_ascii_lowercase();
                                    let task_lower = task.to_ascii_lowercase();
                                    let match_count = task_lower
                                        .split_whitespace()
                                        .filter(|w| w.len() >= 3 && d_lower.contains(w))
                                        .count();
                                    if match_count >= 2 {
                                        Some(match_count)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0);

                            if rule_desc_match > first_desc_match {
                                if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                                    eprintln!(
                                        "[verbose] Rule subcommand override (better match): {} -> {} (rule_desc={}, first_desc={})",
                                        first, rule_sub, rule_desc_match, first_desc_match
                                    );
                                }
                                suggestion.args[0] = rule_sub.clone();
                            }
                        }
                    } else if suggestion.args.is_empty() && sdoc.has_subcommands {
                        let rule_lower = rule_sub.to_ascii_lowercase();
                        let rule_valid = sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.to_ascii_lowercase() == rule_lower);
                        if rule_valid {
                            suggestion.args.insert(0, rule_sub.clone());
                        }
                    }
                }

                if let Some(ref tmpl_args) = template_base_args {
                    let tmpl_has_subcommand = sdoc.has_subcommands
                        && !tmpl_args.is_empty()
                        && sdoc
                            .subcommands
                            .iter()
                            .any(|s| s.eq_ignore_ascii_case(&tmpl_args[0]));
                    let tmpl_has_flags = tmpl_args.iter().any(|a| a.starts_with('-'));
                    let tmpl_has_real_files = tmpl_args.iter().any(|a| {
                        a.contains('.')
                            && !a.starts_with('-')
                            && a != "input.bam"
                            && a != "output.bam"
                            && a != "reads_1.fq"
                            && a != "reads_2.fq"
                            && a != "reference.fa"
                            && a != "input2.bed"
                            && a != "annotation.gtf"
                            && a != "database"
                            && a != "metrics.txt"
                            && a != "tool.jar"
                            && a != "*.bam"
                            && a != "SRR123456"
                            && !a.starts_with("/path/to/")
                    });
                    let tmpl_is_complete = (tmpl_has_subcommand || is_no_subcommand_tool(tool))
                        && tmpl_has_flags
                        && tmpl_has_real_files;

                    if tmpl_is_complete {
                        if std::env::var("OXO_CALL_VERBOSE").is_ok() {
                            eprintln!(
                                "[verbose] Template-only path: template is complete, skipping LLM merge"
                            );
                        }
                        suggestion.args = tmpl_args.clone();
                    } else {
                        suggestion.args =
                            merge_template_with_llm(tmpl_args, &suggestion.args, sdoc);
                    }
                    suggestion.args = remove_duplicate_flags_vec(&suggestion.args);
                } else {
                    let args_str = suggestion.args.join(" ");
                    let corrected = aggressive_correct(&args_str, sdoc, tool, Some(task));
                    suggestion.args = crate::llm::response::parse_shell_args(&corrected);

                    let args_str = suggestion.args.join(" ");
                    let corrected = validate_subcommand(&args_str, tool, sdoc);
                    suggestion.args = crate::llm::response::parse_shell_args(&corrected);

                    let is_small = crate::config::infer_model_parameter_count(&model)
                        .map(|p| p <= 8.0)
                        .unwrap_or(false);

                    if is_small {
                        let tv = extract_task_values(task);
                        let rsc = if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
                            rule_based_subcommand_match(
                                task,
                                &sdoc.subcommands,
                                &sdoc.subcommand_descriptions,
                            )
                        } else {
                            None
                        };
                        let rule_args =
                            assemble_command_from_rules(tool, task, sdoc, rsc.as_deref(), &tv);
                        if !rule_args.is_empty() && rule_args.len() >= 2 {
                            suggestion.args =
                                merge_llm_into_template(&rule_args, &suggestion.args, sdoc);
                            suggestion.args = remove_duplicate_flags_vec(&suggestion.args);
                        }
                    }

                    suggestion.args = add_missing_required_flags(&suggestion.args, sdoc, task);
                    suggestion.args = add_task_implied_flags(&suggestion.args, sdoc, task);
                    suggestion.args = limit_flag_count(&suggestion.args, sdoc, task);
                    suggestion.args = fill_missing_flag_values(&suggestion.args, sdoc, task);
                    suggestion.args = replace_generic_values(&suggestion.args, task);
                }
            }

            suggestion.args = apply_tool_specific_corrections(&suggestion.args, tool, Some(task));

            if is_valid_suggestion(&suggestion) {
                let args_str = suggestion.args.join(" ");
                let _ = crate::cache::LlmCache::store(
                    tool,
                    task,
                    docs_hash.as_deref(),
                    skill_name.as_deref(),
                    &model,
                    &args_str,
                    &suggestion.explanation,
                );
                trace.set_final(&args_str, overall_start.elapsed().as_millis() as u64);
                trace.emit();
                return Ok(suggestion);
            }

            last_raw = raw;
            if attempt == MAX_RETRIES {
                if suggestion.args.is_empty() {
                    if let Some(sdoc) = structured_doc {
                        if let Some(template_args) = generate_from_template(tool, task, sdoc) {
                            if !template_args.is_empty() {
                                trace.record(
                                    "fallback",
                                    "empty",
                                    &template_args.join(" "),
                                    crate::diagnostic::DecisionSource::Fallback,
                                    0,
                                );
                                trace.set_final(
                                    &template_args.join(" "),
                                    overall_start.elapsed().as_millis() as u64,
                                );
                                trace.emit();
                                return Ok(LlmCommandSuggestion {
                                    args: template_args,
                                    explanation: String::new(),
                                    inference_ms: total_inference_ms,
                                });
                            }
                        }
                        let task_values = extract_task_values(task);
                        let assembled =
                            assemble_command_from_rules(tool, task, sdoc, None, &task_values);
                        if !assembled.is_empty() {
                            trace.record(
                                "fallback",
                                "empty",
                                &assembled.join(" "),
                                crate::diagnostic::DecisionSource::Fallback,
                                0,
                            );
                            trace.set_final(
                                &assembled.join(" "),
                                overall_start.elapsed().as_millis() as u64,
                            );
                            trace.emit();
                            return Ok(LlmCommandSuggestion {
                                args: assembled,
                                explanation: suggestion.explanation,
                                inference_ms: total_inference_ms,
                            });
                        }
                    }
                }
                return Ok(suggestion);
            }
        }

        unreachable!()
    }

    pub async fn verify_configuration(&self) -> Result<LlmVerificationResult> {
        let provider = self.config.effective_provider();
        let api_base = self.config.effective_api_base();
        let model = self.config.effective_model();
        let raw = self
            .request_text("Reply with exactly OK.", Some(16), Some(0.0))
            .await?;
        let response_preview = raw.lines().next().unwrap_or("").trim().to_string();

        Ok(LlmVerificationResult {
            provider,
            api_base,
            model,
            response_preview,
        })
    }

    pub async fn verify_run_result(
        &self,
        tool: &str,
        task: &str,
        command: &str,
        exit_code: i32,
        stderr: &str,
        output_files: &[(String, Option<u64>)],
    ) -> Result<LlmRunVerification> {
        let user_prompt =
            build_verification_prompt(tool, task, command, exit_code, stderr, output_files);

        let raw = self
            .request_with_system(
                verification_system_prompt(),
                &user_prompt,
                Some(512),
                Some(0.2),
            )
            .await?;

        Ok(parse_verification_response(&raw))
    }

    async fn call_api(
        &self,
        user_prompt: &str,
        no_prompt: bool,
        tier: PromptTier,
        temperature: Option<f32>,
    ) -> Result<String> {
        let sys_prompt = if no_prompt {
            ""
        } else {
            match tier {
                PromptTier::Slim => system_prompt_unified(),
                PromptTier::Full => system_prompt(),
            }
        };

        if tier == PromptTier::Slim && user_prompt.contains("\n\n---FEW-SHOT---\n\n") {
            let raw = self
                .request_few_shot(sys_prompt, user_prompt, temperature)
                .await?;
            return Ok(raw);
        }

        self.request_with_system(sys_prompt, user_prompt, None, temperature)
            .await
    }

    async fn request_few_shot(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        let provider = self.config.effective_provider();
        let token = if self.config.provider_requires_token() {
            self.config
                .effective_api_token()
                .ok_or_else(|| OxoError::LlmError("No API token configured".to_string()))?
        } else {
            String::new()
        };
        let api_base = self.config.effective_api_base();
        let model = self.config.effective_model();
        let url = format!("{api_base}/chat/completions");

        let mut messages = Vec::new();

        if !sys_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: sys_prompt.to_string(),
            });
        }

        let parts: Vec<&str> = user_prompt
            .split("\n\n---FEW-SHOT---\n\n")
            .filter(|p| !p.is_empty())
            .collect();

        if parts.len() >= 2 {
            let mut is_assistant = false;
            for part in &parts {
                if is_assistant {
                    messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: part.to_string(),
                    });
                } else {
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: part.to_string(),
                    });
                }
                is_assistant = !is_assistant;
            }
        } else {
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            });
        }

        let max_tokens = self.config.effective_max_tokens()?;
        let temp = temperature.unwrap_or_else(|| {
            let profile = crate::config::get_model_profile(&model);
            profile.optimal_temperature
        });

        let auth_token = if provider == "github-copilot" {
            let manager = copilot_auth::get_token_manager();
            manager.get_session_token(&token).await?
        } else {
            token.clone()
        };

        if self.stream_enabled {
            let request = ChatRequestStreaming {
                model: model.clone(),
                messages,
                max_tokens,
                temperature: temp,
                stream: true,
            };

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json");

            req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

            let resp = req_builder.json(&request).send().await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(OxoError::LlmError(format!(
                    "LLM API error: {status} — {body}"
                )));
            }

            let content = streaming_display::read_sse_with_display(
                resp,
                streaming_display::StreamingDisplayConfig {
                    message: "Generating command".to_string(),
                    max_preview_lines: 2,
                    show_preview: true,
                },
            )
            .await
            .map_err(OxoError::LlmError)?;
            return Ok(content);
        }

        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens,
            temperature: temp,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

        let resp = req_builder.json(&request).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(OxoError::LlmError(format!(
                "LLM API error: {status} — {body}"
            )));
        }

        let chat_resp: ChatResponse = resp.json().await?;
        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content.trim().to_string())
    }

    async fn request_text(
        &self,
        user_prompt: &str,
        max_tokens_override: Option<u32>,
        temperature_override: Option<f32>,
    ) -> Result<String> {
        self.request_with_system(
            system_prompt(),
            user_prompt,
            max_tokens_override,
            temperature_override,
        )
        .await
    }

    async fn request_with_system(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
        max_tokens_override: Option<u32>,
        temperature_override: Option<f32>,
    ) -> Result<String> {
        let provider = self.config.effective_provider();
        let token_opt = self.config.effective_api_token();
        let token = if self.config.provider_requires_token() {
            token_opt.ok_or_else(|| {
                let token_hint = match provider.as_str() {
                    "github-copilot" => "  For GitHub Copilot, run: oxo-call config login",
                    "openai" => "  For OpenAI, create an API key at:\n    https://platform.openai.com/api-keys",
                    "anthropic" => "  For Anthropic, create an API key at:\n    https://console.anthropic.com/settings/keys",
                    _ => "  Check your provider's documentation for token setup.",
                };
                OxoError::LlmError(
                    format!(
                        "No API token configured for provider '{provider}'.\n\n\
                        Option 1 — Interactive login (recommended for github-copilot):\n  \
                          oxo-call config login\n\n\
                        Option 2 — Set via config:\n  \
                          oxo-call config set llm.api_token <your-token>\n\n\
                        Option 3 — Set via environment variable:\n  \
                          export OXO_CALL_LLM_API_TOKEN=<your-token>\n\n\
                        How to get a token:\n{token_hint}\n\n\
                        Test your setup: oxo-call config verify"
                    ),
                )
            })?
        } else {
            String::new()
        };

        let api_base = self.config.effective_api_base();

        if !api_base.starts_with("https://")
            && !api_base.starts_with("http://localhost")
            && !api_base.starts_with("http://127.0.0.1")
            && !api_base.starts_with("http://[::1]")
        {
            return Err(OxoError::LlmError(format!(
                "API base URL must use HTTPS for remote endpoints: {api_base}"
            )));
        }

        let model = self.config.effective_model();
        let url = format!("{api_base}/chat/completions");

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: sys_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ];

        let max_tokens = max_tokens_override.unwrap_or(self.config.effective_max_tokens()?);
        let temperature = temperature_override.unwrap_or_else(|| {
            let profile = crate::config::get_model_profile(&model);
            profile.optimal_temperature
        });

        let auth_token = if provider == "github-copilot" {
            let manager = copilot_auth::get_token_manager();
            manager.get_session_token(&token).await?
        } else {
            token.clone()
        };

        if self.stream_enabled {
            let request = ChatRequestStreaming {
                model: model.clone(),
                messages,
                max_tokens,
                temperature,
                stream: true,
            };

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json");

            req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

            let response = req_builder
                .json(&request)
                .send()
                .await
                .map_err(|e| OxoError::LlmError(format!("HTTP request failed: {e}")))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(OxoError::LlmError(format!("API returned {status}: {body}")));
            }

            let content = streaming_display::read_sse_with_display(
                response,
                streaming_display::StreamingDisplayConfig {
                    message: "Processing".to_string(),
                    max_preview_lines: 2,
                    show_preview: true,
                },
            )
            .await
            .map_err(OxoError::LlmError)?;
            return Ok(content);
        }

        let request = ChatRequest {
            model: model.clone(),
            messages,
            max_tokens,
            temperature,
        };

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        req_builder = apply_provider_auth_headers(req_builder, &provider, &auth_token);

        let response = req_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| OxoError::LlmError(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OxoError::LlmError(format!("API returned {status}: {body}")));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| OxoError::LlmError(format!("Failed to parse API response: {e}")))?;

        Ok(chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    pub async fn verify_skill(
        &self,
        tool: &str,
        skill_content: &str,
    ) -> Result<LlmSkillVerification> {
        let user_prompt = build_skill_verify_prompt(tool, skill_content);
        let raw = self
            .request_with_system(
                skill_reviewer_system_prompt(),
                &user_prompt,
                Some(1024),
                Some(0.2),
            )
            .await?;
        Ok(parse_skill_verify_response(&raw))
    }

    pub async fn polish_skill(&self, tool: &str, skill_content: &str) -> Result<String> {
        let user_prompt = build_skill_polish_prompt(tool, skill_content);
        let raw = self
            .request_with_system(
                skill_reviewer_system_prompt(),
                &user_prompt,
                Some(4096),
                Some(0.3),
            )
            .await?;
        Ok(strip_markdown_fences(&raw))
    }

    pub async fn generate_skill_template(&self, tool: &str) -> Result<String> {
        let user_prompt = build_skill_generate_prompt(tool);
        let raw = self
            .request_with_system(
                skill_reviewer_system_prompt(),
                &user_prompt,
                Some(4096),
                Some(0.4),
            )
            .await?;
        Ok(strip_markdown_fences(&raw))
    }

    pub async fn generate_shell_command(&self, description: &str) -> Result<(String, String)> {
        let system = "You are a shell command expert for Linux/macOS. \
            Given a plain-English description (in any language), produce a single \
            production-ready shell command or short pipeline. Use standard coreutils, \
            common bioinformatics tools, and POSIX-compatible syntax. \
            Reply with exactly two lines and nothing else:\n\
            COMMAND: <the shell command>\n\
            EXPLANATION: <one-sentence explanation in the same language as the input>";

        let raw = self
            .request_with_system(system, description, Some(256), Some(0.1))
            .await?;

        let mut command = String::new();
        let mut explanation = String::new();
        for line in raw.lines() {
            if let Some(rest) = line.strip_prefix("COMMAND:") {
                command = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("EXPLANATION:") {
                explanation = rest.trim().to_string();
            }
        }
        if command.is_empty() {
            command = raw.trim().to_string();
        }
        Ok((command, explanation))
    }
}

impl super::types::LlmProvider for LlmClient {
    async fn chat_completion(
        &self,
        system: &str,
        user_prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String> {
        self.request_with_system(system, user_prompt, Some(max_tokens), Some(temperature))
            .await
    }

    fn name(&self) -> &str {
        match self.config.effective_provider().as_str() {
            "openai" => "openai",
            "anthropic" => "anthropic",
            "github-copilot" => "github-copilot",
            "ollama" => "ollama",
            "deepseek" => "deepseek",
            "moonshot" | "kimi" => "moonshot",
            "zhipu" | "glm" => "zhipu",
            "minimax" => "minimax",
            _ => "custom",
        }
    }
}
