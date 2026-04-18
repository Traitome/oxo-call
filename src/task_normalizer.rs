//! Task Normalizer
//!
//! Converts user input (Chinese, ambiguous, colloquial) into standardized English task descriptions.

#![allow(dead_code)]

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::Config;
use crate::llm::LlmClient;

/// Normalized task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTask {
    /// Standardized English description
    pub description: String,
    /// Task intent classification
    pub intent: TaskIntent,
    /// Extracted parameters
    pub parameters: HashMap<String, String>,
    /// Constraints and requirements
    pub constraints: Vec<String>,
    /// Confidence score
    pub confidence: f32,
}

impl Default for NormalizedTask {
    fn default() -> Self {
        Self {
            description: String::new(),
            intent: TaskIntent::default(),
            parameters: HashMap::new(),
            constraints: Vec::new(),
            confidence: 0.0,
        }
    }
}

/// Task intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskIntent {
    DataConversion,
    QualityControl,
    Alignment,
    VariantCalling,
    Filtering,
    Aggregation,
    Indexing,
    Statistics,
    Visualization,
    #[default]
    Custom,
}

impl std::fmt::Display for TaskIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskIntent::DataConversion => write!(f, "DataConversion"),
            TaskIntent::QualityControl => write!(f, "QualityControl"),
            TaskIntent::Alignment => write!(f, "Alignment"),
            TaskIntent::VariantCalling => write!(f, "VariantCalling"),
            TaskIntent::Filtering => write!(f, "Filtering"),
            TaskIntent::Aggregation => write!(f, "Aggregation"),
            TaskIntent::Indexing => write!(f, "Indexing"),
            TaskIntent::Statistics => write!(f, "Statistics"),
            TaskIntent::Visualization => write!(f, "Visualization"),
            TaskIntent::Custom => write!(f, "Custom"),
        }
    }
}

/// LLM response for task normalization
#[derive(Debug, Clone, Deserialize)]
struct NormalizationResponse {
    description: String,
    intent: String,
    parameters: HashMap<String, String>,
    constraints: Vec<String>,
}

/// Task Normalizer using LLM
pub struct TaskNormalizer {
    llm_client: Option<LlmClient>,
}

impl Default for TaskNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskNormalizer {
    pub fn new() -> Self {
        Self { llm_client: None }
    }

    /// Create a normalizer backed by an LLM client for complex cases.
    pub fn new_with_llm(config: Config) -> Self {
        Self {
            llm_client: Some(LlmClient::new(config)),
        }
    }

    /// Normalize user task using LLM
    ///
    /// # Arguments
    /// * `task` - User's original task description
    /// * `tool` - Tool name
    ///
    /// # Returns
    /// Normalized task with structured information
    pub async fn normalize(&self, task: &str, tool: &str) -> Result<NormalizedTask> {
        // Step 1: Try rule-based normalization first (fast path)
        if let Some(normalized) = self.try_rule_based_normalization(task, tool) {
            return Ok(normalized);
        }

        // Step 2: Use LLM for complex cases
        self.llm_normalize(task, tool).await
    }

    /// Rule-based normalization for common patterns
    fn try_rule_based_normalization(&self, task: &str, _tool: &str) -> Option<NormalizedTask> {
        let task_lower = task.to_lowercase();

        // Pattern 1: Chinese common patterns
        let chinese_patterns = [
            ("排序", "sort"),
            ("转换", "convert"),
            ("过滤", "filter"),
            ("比对", "align"),
            ("变异检测", "variant calling"),
            ("质量控制", "quality control"),
            ("压缩", "compress"),
            ("解压", "decompress"),
            ("索引", "index"),
        ];

        for (chinese, english) in chinese_patterns {
            if task.contains(chinese) {
                let description = task.replace(chinese, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 2: Japanese common patterns
        let japanese_patterns = [
            ("ソート", "sort"),
            ("変換", "convert"),
            ("フィルタ", "filter"),
            ("アライメント", "align"),
            ("品質管理", "quality control"),
            ("圧縮", "compress"),
            ("インデックス", "index"),
            ("マッピング", "mapping"),
        ];

        for (japanese, english) in japanese_patterns {
            if task.contains(japanese) {
                let description = task.replace(japanese, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 3: Korean common patterns
        let korean_patterns = [
            ("정렬", "sort"),
            ("변환", "convert"),
            ("필터", "filter"),
            ("정렬", "align"),
            ("품질관리", "quality control"),
            ("압축", "compress"),
            ("색인", "index"),
        ];

        for (korean, english) in korean_patterns {
            if task.contains(korean) {
                let description = task.replace(korean, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 4: Spanish common patterns
        let spanish_patterns = [
            ("ordenar", "sort"),
            ("convertir", "convert"),
            ("filtrar", "filter"),
            ("alinear", "align"),
            ("control de calidad", "quality control"),
            ("comprimir", "compress"),
            ("indexar", "index"),
        ];

        for (spanish, english) in &spanish_patterns {
            if task_lower.contains(spanish) {
                let description = task_lower.replace(spanish, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 5: French common patterns
        let french_patterns = [
            ("trier", "sort"),
            ("convertir", "convert"),
            ("filtrer", "filter"),
            ("aligner", "align"),
            ("contrôle qualité", "quality control"),
            ("compresser", "compress"),
            ("indexer", "index"),
        ];

        for (french, english) in &french_patterns {
            if task_lower.contains(french) {
                let description = task_lower.replace(french, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 6: German common patterns
        let german_patterns = [
            ("sortieren", "sort"),
            ("konvertieren", "convert"),
            ("filtern", "filter"),
            ("alignieren", "align"),
            ("qualitätskontrolle", "quality control"),
            ("komprimieren", "compress"),
            ("indizieren", "index"),
        ];

        for (german, english) in &german_patterns {
            if task_lower.contains(german) {
                let description = task_lower.replace(german, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 7: Portuguese common patterns
        let portuguese_patterns = [
            ("ordenar", "sort"),
            ("converter", "convert"),
            ("filtrar", "filter"),
            ("alinhar", "align"),
            ("controle de qualidade", "quality control"),
            ("comprimir", "compress"),
            ("indexar", "index"),
        ];

        for (portuguese, english) in &portuguese_patterns {
            if task_lower.contains(portuguese) {
                let description = task_lower.replace(portuguese, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 8: Russian common patterns
        let russian_patterns = [
            ("сортировать", "sort"),
            ("конвертировать", "convert"),
            ("фильтровать", "filter"),
            ("выравнивание", "align"),
            ("контроль качества", "quality control"),
            ("сжать", "compress"),
            ("индексировать", "index"),
        ];

        for (russian, english) in russian_patterns {
            if task.contains(russian) {
                let description = task.replace(russian, english);
                return Some(NormalizedTask {
                    description,
                    intent: self.infer_intent_from_keyword(english),
                    parameters: self.extract_parameters(task),
                    constraints: vec![],
                    confidence: 0.7,
                });
            }
        }

        // Pattern 9: Simple English patterns
        if self.is_simple_english(&task_lower) {
            return Some(NormalizedTask {
                description: task.to_string(),
                intent: self.infer_intent(&task_lower),
                parameters: self.extract_parameters(task),
                constraints: vec![],
                confidence: 0.9,
            });
        }

        None
    }

    /// Check if task is simple English
    fn is_simple_english(&self, task: &str) -> bool {
        // No Chinese characters
        !task.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
            // Reasonable length
            && task.len() < 100
            // Contains common bioinformatics verbs
            && ["sort", "filter", "align", "convert", "index", "call", "merge", "split"]
                .iter()
                .any(|v| task.contains(v))
    }

    /// Infer intent from task description
    fn infer_intent(&self, task: &str) -> TaskIntent {
        // Check for filtering first (higher priority than quality control)
        if task.contains("filter") || task.contains("select") {
            TaskIntent::Filtering
        } else if task.contains("sort") || task.contains("order") {
            TaskIntent::DataConversion
        } else if task.contains("quality") || task.contains("qc") {
            TaskIntent::QualityControl
        } else if task.contains("align") || task.contains("map") {
            TaskIntent::Alignment
        } else if task.contains("variant") || task.contains("snp") || task.contains("call") {
            TaskIntent::VariantCalling
        } else if task.contains("merge") || task.contains("combine") || task.contains("aggregate") {
            TaskIntent::Aggregation
        } else if task.contains("index") {
            TaskIntent::Indexing
        } else if task.contains("stat") || task.contains("count") {
            TaskIntent::Statistics
        } else if task.contains("plot") || task.contains("visual") {
            TaskIntent::Visualization
        } else {
            TaskIntent::Custom
        }
    }

    /// Infer intent from keyword
    fn infer_intent_from_keyword(&self, keyword: &str) -> TaskIntent {
        match keyword {
            "sort" | "convert" => TaskIntent::DataConversion,
            "filter" => TaskIntent::Filtering,
            "align" => TaskIntent::Alignment,
            "variant calling" => TaskIntent::VariantCalling,
            "quality control" => TaskIntent::QualityControl,
            "compress" | "decompress" => TaskIntent::DataConversion,
            "index" => TaskIntent::Indexing,
            _ => TaskIntent::Custom,
        }
    }

    /// Extract parameters from task description
    fn extract_parameters(&self, task: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();

        // Extract input file
        let input_patterns = [
            (r"input[:\s]+([^\s]+)", "input"),
            (r"([^\s]+\.bam)", "input"),
            (r"([^\s]+\.fq\.gz)", "input"),
            (r"([^\s]+\.fastq)", "input"),
            (r"([^\s]+\.vcf)", "input"),
        ];

        for (pattern, key) in input_patterns {
            if let Ok(re) = regex::Regex::new(pattern)
                && let Some(caps) = re.captures(task)
                && let Some(value) = caps.get(1)
            {
                params.insert(key.to_string(), value.as_str().to_string());
                break;
            }
        }

        // Extract output file
        let output_patterns = [
            (r"output[:\s]+([^\s]+)", "output"),
            (r"to\s+([^\s]+\.bam)", "output"),
            (r"to\s+([^\s]+\.vcf)", "output"),
        ];

        for (pattern, key) in output_patterns {
            if let Ok(re) = regex::Regex::new(pattern)
                && let Some(caps) = re.captures(task)
                && let Some(value) = caps.get(1)
            {
                params.insert(key.to_string(), value.as_str().to_string());
                break;
            }
        }

        // Extract thread count
        if let Ok(re) = regex::Regex::new(r"(\d+)\s+threads?")
            && let Some(caps) = re.captures(task)
            && let Some(value) = caps.get(1)
        {
            params.insert("threads".to_string(), value.as_str().to_string());
        }

        params
    }

    /// LLM-based normalization
    async fn llm_normalize(&self, task: &str, tool: &str) -> Result<NormalizedTask> {
        let prompt = self.build_normalization_prompt(task, tool);

        // If we have an LLM client, try to use it
        if let Some(ref llm) = self.llm_client {
            let system = "You are a bioinformatics task standardizer. Output only valid JSON.";
            match llm
                .chat_completion(system, &prompt, Some(512), Some(0.1))
                .await
            {
                Ok(raw) => {
                    // Try to parse the JSON response
                    let trimmed = raw.trim();
                    // Strip markdown fences if present
                    let json_str = if trimmed.starts_with("```") {
                        trimmed
                            .trim_start_matches("```json")
                            .trim_start_matches("```")
                            .trim_end_matches("```")
                            .trim()
                    } else {
                        trimmed
                    };

                    if let Ok(resp) = serde_json::from_str::<NormalizationResponse>(json_str) {
                        let intent = match resp.intent.as_str() {
                            "DataConversion" => TaskIntent::DataConversion,
                            "QualityControl" => TaskIntent::QualityControl,
                            "Alignment" => TaskIntent::Alignment,
                            "VariantCalling" => TaskIntent::VariantCalling,
                            "Filtering" => TaskIntent::Filtering,
                            "Aggregation" => TaskIntent::Aggregation,
                            "Indexing" => TaskIntent::Indexing,
                            "Statistics" => TaskIntent::Statistics,
                            "Visualization" => TaskIntent::Visualization,
                            _ => TaskIntent::Custom,
                        };
                        return Ok(NormalizedTask {
                            description: resp.description,
                            intent,
                            parameters: resp.parameters,
                            constraints: resp.constraints,
                            confidence: 0.85,
                        });
                    }
                    // JSON parse failed — fall through to rule-based fallback
                }
                Err(_) => {
                    // LLM call failed — fall through to rule-based fallback
                }
            }
        }

        // Fallback: return rule-based result
        Ok(NormalizedTask {
            description: task.to_string(),
            intent: self.infer_intent(&task.to_lowercase()),
            parameters: self.extract_parameters(task),
            constraints: vec![],
            confidence: 0.5,
        })
    }

    /// Build LLM prompt for task normalization
    fn build_normalization_prompt(&self, task: &str, tool: &str) -> String {
        format!(
            r#"You are a bioinformatics task standardizer. Convert the user's task into a clear, structured English description.

Tool: {tool}
User task: {task}

Output JSON format:
{{
  "description": "Clear English description of the task",
  "intent": "One of: DataConversion, QualityControl, Alignment, VariantCalling, Filtering, Aggregation, Indexing, Statistics, Visualization, Custom",
  "parameters": {{
    "input": "extracted input file",
    "output": "extracted output file",
    "threads": "extracted thread count"
  }},
  "constraints": [
    "quality > 30",
    "depth > 10"
  ]
}}

Examples:
1. User input: "把 input.bam 按坐标排序"
   Output: {{"description": "Sort BAM file by coordinate order", "intent": "DataConversion", "parameters": {{"input": "input.bam"}}, "constraints": []}}

2. User input: "call variants with quality > 30"
   Output: {{"description": "Call variants with quality filter", "intent": "VariantCalling", "parameters": {{}}, "constraints": ["quality > 30"]}}

Now process the user task above. Output only the JSON, no explanation."#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_english_task() {
        let normalizer = TaskNormalizer::new();

        let result = normalizer
            .normalize("sort input.bam by coordinate", "samtools")
            .await
            .unwrap();

        assert_eq!(result.intent, TaskIntent::DataConversion);
        assert!(result.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_chinese_task() {
        let normalizer = TaskNormalizer::new();

        let result = normalizer
            .normalize("把 input.bam 按坐标排序", "samtools")
            .await
            .unwrap();

        assert!(result.description.contains("sort"));
        assert!(result.confidence > 0.6);
    }

    #[tokio::test]
    async fn test_parameter_extraction() {
        let normalizer = TaskNormalizer::new();

        let result = normalizer
            .normalize("align reads.fq to ref.fa with 8 threads", "bwa")
            .await
            .unwrap();

        assert_eq!(result.parameters.get("threads"), Some(&"8".to_string()));
    }

    #[tokio::test]
    async fn test_intent_inference() {
        let normalizer = TaskNormalizer::new();

        let test_cases = [
            ("sort input.bam", TaskIntent::DataConversion),
            ("quality control for fastq", TaskIntent::QualityControl),
            ("align reads to reference", TaskIntent::Alignment),
            ("call variants from bam", TaskIntent::VariantCalling),
            ("filter vcf by quality", TaskIntent::Filtering),
            ("merge multiple vcf files", TaskIntent::Aggregation),
        ];

        for (task, expected_intent) in test_cases {
            let result = normalizer.normalize(task, "tool").await.unwrap();
            assert_eq!(result.intent, expected_intent, "Failed for task: {}", task);
        }
    }
}
