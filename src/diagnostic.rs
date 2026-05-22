use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DecisionStep {
    pub step: String,
    pub input: String,
    pub output: String,
    pub duration_ms: u64,
    pub source: DecisionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum DecisionSource {
    Rule,
    Template,
    Llm,
    PostProcess,
    Fallback,
    Cache,
    Skill,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerationTrace {
    pub tool: String,
    pub task: String,
    pub model: String,
    pub prompt_tier: String,
    pub steps: Vec<DecisionStep>,
    pub final_args: String,
    pub total_duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcommand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    pub timestamp: String,
}

impl GenerationTrace {
    pub fn new(tool: &str, task: &str) -> Self {
        Self {
            tool: tool.to_string(),
            task: task.to_string(),
            model: String::new(),
            prompt_tier: String::new(),
            steps: Vec::new(),
            final_args: String::new(),
            total_duration_ms: 0,
            docs_hash: None,
            skill_name: None,
            subcommand: None,
            flag_count: None,
            cache_hit: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    pub fn with_prompt_tier(&mut self, tier: &str) {
        self.prompt_tier = tier.to_string();
    }

    pub fn with_docs_hash(&mut self, hash: Option<String>) {
        self.docs_hash = hash;
    }

    pub fn with_skill_name(&mut self, name: Option<String>) {
        self.skill_name = name;
    }

    pub fn with_cache_hit(&mut self, hit: bool) {
        self.cache_hit = Some(hit);
    }

    pub fn record(
        &mut self,
        step: &str,
        input: &str,
        output: &str,
        source: DecisionSource,
        duration_ms: u64,
    ) {
        self.steps.push(DecisionStep {
            step: step.to_string(),
            input: input.chars().take(200).collect(),
            output: output.chars().take(200).collect(),
            duration_ms,
            source,
            metadata: None,
        });
    }

    pub fn record_with_metadata(
        &mut self,
        step: &str,
        input: &str,
        output: &str,
        source: DecisionSource,
        duration_ms: u64,
        metadata: serde_json::Value,
    ) {
        self.steps.push(DecisionStep {
            step: step.to_string(),
            input: input.chars().take(200).collect(),
            output: output.chars().take(200).collect(),
            duration_ms,
            source,
            metadata: Some(metadata),
        });
    }

    pub fn set_final(&mut self, args: &str, total_ms: u64) {
        self.final_args = args.to_string();
        self.total_duration_ms = total_ms;
        let tokens: Vec<&str> = args.split_whitespace().collect();
        self.flag_count = Some(tokens.iter().filter(|t| t.starts_with('-')).count());
        let sub = tokens
            .first()
            .map(|t| t.to_string())
            .filter(|t| !t.starts_with('-'));
        self.subcommand = sub;
    }

    pub fn emit(&self) {
        if std::env::var("OXO_CALL_TRACE").is_ok() {
            eprintln!("[trace] === GenerationTrace for {} ===", self.tool);
            eprintln!(
                "[trace] Task: {}",
                self.task.chars().take(100).collect::<String>()
            );
            eprintln!("[trace] Model: {} | Tier: {}", self.model, self.prompt_tier);
            for s in &self.steps {
                eprintln!(
                    "[trace]   {} ({:?}): {} -> {} [{}ms]",
                    s.step,
                    s.source,
                    s.input.chars().take(60).collect::<String>(),
                    s.output.chars().take(60).collect::<String>(),
                    s.duration_ms
                );
            }
            eprintln!(
                "[trace] Final: {} [{}ms total]",
                self.final_args, self.total_duration_ms
            );
        }
    }

    pub fn emit_json(&self) -> Option<String> {
        if std::env::var("OXO_CALL_TRACE_JSON").is_ok() {
            Some(serde_json::to_string_pretty(self).unwrap_or_default())
        } else {
            None
        }
    }

    pub fn save_to_file(&self) -> Option<PathBuf> {
        if std::env::var("OXO_CALL_TRACE_DIR").is_ok() {
            let dir = std::env::var("OXO_CALL_TRACE_DIR")
                .unwrap_or_else(|_| "/tmp/oxo-call-traces".to_string());
            let _ = std::fs::create_dir_all(&dir);
            let filename = format!(
                "{}_{}.json",
                self.tool,
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            );
            let path = PathBuf::from(&dir).join(&filename);
            if let Ok(json) = serde_json::to_string_pretty(self) {
                if std::fs::write(&path, json).is_ok() {
                    return Some(path);
                }
            }
        }
        None
    }
}

pub struct StepTimer {
    start: Instant,
    step: String,
    input: String,
    source: DecisionSource,
}

impl StepTimer {
    pub fn start(step: &str, input: &str, source: DecisionSource) -> Self {
        Self {
            start: Instant::now(),
            step: step.to_string(),
            input: input.to_string(),
            source,
        }
    }

    pub fn finish(self, trace: &mut GenerationTrace, output: &str) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        trace.record(&self.step, &self.input, output, self.source, duration_ms);
    }

    pub fn finish_with_metadata(
        self,
        trace: &mut GenerationTrace,
        output: &str,
        metadata: serde_json::Value,
    ) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        trace.record_with_metadata(
            &self.step,
            &self.input,
            output,
            self.source,
            duration_ms,
            metadata,
        );
    }
}
