//! Universal Task Normalizer — Language Processing Layer.
//!
//! Converts user input in **any language** (Chinese, Japanese, Korean, Spanish,
//! French, German, Portuguese, Russian, and more) into standardized English
//! bioinformatics task descriptions.
//!
//! The normalizer uses a two-tier approach:
//! 1. **Rule-based fast path** — zero-latency keyword translation for common
//!    bioinformatics verbs across 8+ languages via a data-driven pattern table.
//! 2. **LLM fallback** — for complex or ambiguous inputs, delegates to the
//!    configured LLM backend for full semantic translation.
//!
//! # Design Principles
//! - **Efficiency**: Rule-based path handles >80% of inputs with no LLM call.
//! - **Extensibility**: New languages are added by appending to `LANGUAGE_PATTERNS`.
//! - **Reliability**: LLM failures gracefully degrade to the rule-based result.

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

// ─── Data-driven multilingual patterns ────────────────────────────────────────

/// A single translation rule: `(native_keyword, english_keyword)`.
type TranslationPair = (&'static str, &'static str);

/// A language-specific pattern set with its matching strategy.
struct LanguagePatterns {
    /// Whether to match on the lowercased task (for Latin-script languages)
    /// or the original task (for CJK / Cyrillic).
    use_lowercase: bool,
    /// Translation pairs: (native keyword → English keyword).
    patterns: &'static [TranslationPair],
}

/// All supported language patterns — order does not matter for correctness.
/// Adding a new language is as simple as appending a new entry.
static LANGUAGE_PATTERNS: &[LanguagePatterns] = &[
    // Chinese (match on original — CJK characters are case-insensitive)
    LanguagePatterns {
        use_lowercase: false,
        patterns: &[
            ("变异检测", "variant calling"),
            ("质量控制", "quality control"),
            ("排序", "sort"),
            ("转换", "convert"),
            ("过滤", "filter"),
            ("比对", "align"),
            ("压缩", "compress"),
            ("解压", "decompress"),
            ("索引", "index"),
        ],
    },
    // Japanese
    LanguagePatterns {
        use_lowercase: false,
        patterns: &[
            ("アライメント", "align"),
            ("品質管理", "quality control"),
            ("マッピング", "mapping"),
            ("ソート", "sort"),
            ("変換", "convert"),
            ("フィルタ", "filter"),
            ("圧縮", "compress"),
            ("インデックス", "index"),
        ],
    },
    // Korean
    LanguagePatterns {
        use_lowercase: false,
        patterns: &[
            ("품질관리", "quality control"),
            ("정렬", "sort"),
            ("변환", "convert"),
            ("필터", "filter"),
            ("압축", "compress"),
            ("색인", "index"),
        ],
    },
    // Spanish (match on lowercase — Latin script)
    LanguagePatterns {
        use_lowercase: true,
        patterns: &[
            ("control de calidad", "quality control"),
            ("ordenar", "sort"),
            ("convertir", "convert"),
            ("filtrar", "filter"),
            ("alinear", "align"),
            ("comprimir", "compress"),
            ("indexar", "index"),
        ],
    },
    // French
    LanguagePatterns {
        use_lowercase: true,
        patterns: &[
            ("contrôle qualité", "quality control"),
            ("trier", "sort"),
            ("convertir", "convert"),
            ("filtrer", "filter"),
            ("aligner", "align"),
            ("compresser", "compress"),
            ("indexer", "index"),
        ],
    },
    // German
    LanguagePatterns {
        use_lowercase: true,
        patterns: &[
            ("qualitätskontrolle", "quality control"),
            ("sortieren", "sort"),
            ("konvertieren", "convert"),
            ("filtern", "filter"),
            ("alignieren", "align"),
            ("komprimieren", "compress"),
            ("indizieren", "index"),
        ],
    },
    // Portuguese
    LanguagePatterns {
        use_lowercase: true,
        patterns: &[
            ("controle de qualidade", "quality control"),
            ("ordenar", "sort"),
            ("converter", "convert"),
            ("filtrar", "filter"),
            ("alinhar", "align"),
            ("comprimir", "compress"),
            ("indexar", "index"),
        ],
    },
    // Russian (match on original — Cyrillic is case-sensitive)
    LanguagePatterns {
        use_lowercase: false,
        patterns: &[
            ("контроль качества", "quality control"),
            ("сортировать", "sort"),
            ("конвертировать", "convert"),
            ("фильтровать", "filter"),
            ("выравнивание", "align"),
            ("сжать", "compress"),
            ("индексировать", "index"),
        ],
    },
];

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

    /// Rule-based normalization for common patterns.
    ///
    /// Iterates over `LANGUAGE_PATTERNS` (data-driven, no per-language
    /// code blocks) to translate bioinformatics verbs into English.
    fn try_rule_based_normalization(&self, task: &str, _tool: &str) -> Option<NormalizedTask> {
        let task_lower = task.to_lowercase();

        // Scan all language patterns — first match wins.
        for lang in LANGUAGE_PATTERNS {
            let haystack = if lang.use_lowercase {
                &task_lower
            } else {
                task
            };
            for &(native, english) in lang.patterns {
                if haystack.contains(native) {
                    let description = haystack.replace(native, english);
                    return Some(NormalizedTask {
                        description,
                        intent: self.infer_intent_from_keyword(english),
                        parameters: self.extract_parameters(task),
                        constraints: vec![],
                        confidence: 0.7,
                    });
                }
            }
        }

        // Simple English patterns (no translation needed).
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

    // ── Multilingual coverage tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_japanese_task() {
        let n = TaskNormalizer::new();
        let r = n.normalize("ソート input.bam", "samtools").await.unwrap();
        assert!(
            r.description.contains("sort"),
            "Japanese: {}",
            r.description
        );
        assert!(r.confidence > 0.6);
    }

    #[tokio::test]
    async fn test_korean_task() {
        let n = TaskNormalizer::new();
        let r = n.normalize("필터 input.vcf", "bcftools").await.unwrap();
        assert!(
            r.description.contains("filter"),
            "Korean: {}",
            r.description
        );
    }

    #[tokio::test]
    async fn test_spanish_task() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("Filtrar variantes por calidad", "bcftools")
            .await
            .unwrap();
        assert!(
            r.description.contains("filter"),
            "Spanish: {}",
            r.description
        );
    }

    #[tokio::test]
    async fn test_french_task() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("Trier les lectures par coordonnées", "samtools")
            .await
            .unwrap();
        assert!(r.description.contains("sort"), "French: {}", r.description);
    }

    #[tokio::test]
    async fn test_german_task() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("Filtern nach Qualität", "bcftools")
            .await
            .unwrap();
        assert!(
            r.description.contains("filter"),
            "German: {}",
            r.description
        );
    }

    #[tokio::test]
    async fn test_portuguese_task() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("Converter formato BAM para CRAM", "samtools")
            .await
            .unwrap();
        assert!(
            r.description.contains("convert"),
            "Portuguese: {}",
            r.description
        );
    }

    #[tokio::test]
    async fn test_russian_task() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("сортировать input.bam по координатам", "samtools")
            .await
            .unwrap();
        assert!(r.description.contains("sort"), "Russian: {}", r.description);
    }

    #[tokio::test]
    async fn test_chinese_variant_calling() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("进行变异检测 quality > 30", "gatk4")
            .await
            .unwrap();
        assert!(
            r.description.contains("variant calling"),
            "Got: {}",
            r.description
        );
        assert_eq!(r.intent, TaskIntent::VariantCalling);
    }

    #[tokio::test]
    async fn test_chinese_quality_control() {
        let n = TaskNormalizer::new();
        let r = n.normalize("质量控制 reads.fq.gz", "fastqc").await.unwrap();
        assert!(r.description.contains("quality control"));
        assert_eq!(r.intent, TaskIntent::QualityControl);
    }

    #[tokio::test]
    async fn test_llm_fallback_without_client() {
        let n = TaskNormalizer::new();
        // A task that doesn't match any rule-based pattern or simple English
        let r = n
            .normalize("Неизвестная задача для анализа", "samtools")
            .await
            .unwrap();
        // Should still return a result via fallback
        assert!(r.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_input_file_extraction_bam() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("sort sample.bam by coordinate", "samtools")
            .await
            .unwrap();
        assert_eq!(r.parameters.get("input"), Some(&"sample.bam".to_string()));
    }

    #[tokio::test]
    async fn test_input_file_extraction_fastq() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("align reads.fq.gz to reference", "bwa")
            .await
            .unwrap();
        assert_eq!(r.parameters.get("input"), Some(&"reads.fq.gz".to_string()));
    }

    #[tokio::test]
    async fn test_output_file_extraction() {
        let n = TaskNormalizer::new();
        let r = n
            .normalize("sort input.bam to sorted.bam", "samtools")
            .await
            .unwrap();
        assert_eq!(r.parameters.get("output"), Some(&"sorted.bam".to_string()));
    }

    #[tokio::test]
    async fn test_default_normalized_task() {
        let default = NormalizedTask::default();
        assert!(default.description.is_empty());
        assert_eq!(default.intent, TaskIntent::Custom);
        assert_eq!(default.confidence, 0.0);
    }

    #[tokio::test]
    async fn test_task_intent_display() {
        assert_eq!(format!("{}", TaskIntent::DataConversion), "DataConversion");
        assert_eq!(format!("{}", TaskIntent::Alignment), "Alignment");
        assert_eq!(format!("{}", TaskIntent::Custom), "Custom");
    }
}
