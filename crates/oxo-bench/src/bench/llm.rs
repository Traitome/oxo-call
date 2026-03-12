//! LLM model evaluation harness for oxo-call benchmarking.
//!
//! Measures the accuracy and consistency of LLM-generated bioinformatics
//! commands across different model sizes/costs (gpt-4o, gpt-4o-mini, etc.).
//!
//! # Evaluation methodology
//!
//! For each (tool, task) pair drawn from a fixed evaluation suite:
//! 1. Generate a command `n_repeats` times with the target model.
//! 2. Parse each response and check:
//!    - Format validity (does it contain `ARGS:` and `EXPLANATION:`?).
//!    - Semantic correctness via a reference checklist (key flags expected).
//!    - Self-consistency (do repeated calls produce the same flags?).
//! 3. Aggregate metrics: accuracy@1, consistency, avg_latency_ms, avg_tokens.


/// Configuration for a model evaluation run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelBenchConfig {
    /// LLM model identifier (e.g. "gpt-4o-mini", "gpt-4o", "claude-3-haiku-20240307").
    pub model: String,
    /// Number of times to repeat each task for consistency measurement.
    pub n_repeats: usize,
    /// Temperature to use for generation (0.0 = deterministic).
    pub temperature: f32,
    /// Maximum tokens to generate per response.
    pub max_tokens: u32,
}

impl Default for ModelBenchConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o-mini".to_string(),
            n_repeats: 3,
            temperature: 0.0,
            max_tokens: 512,
        }
    }
}

/// A single evaluation task: tool + natural language description + expected key flags.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvalTask {
    /// Tool binary name.
    pub tool: String,
    /// Natural language task description (the user input).
    pub task: String,
    /// Key flags or substrings that MUST appear in a correct ARGS line.
    pub required_patterns: Vec<String>,
    /// Category for grouping results (e.g. "alignment", "qc", "variant-calling").
    pub category: String,
}

/// Result for a single model × task evaluation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelBenchResult {
    pub model: String,
    pub tool: String,
    pub task_summary: String,
    pub category: String,
    /// Number of responses that matched ALL required patterns.
    pub correct_count: usize,
    /// Total number of attempts.
    pub total_count: usize,
    /// Fraction of responses with valid ARGS:/EXPLANATION: format.
    pub format_validity_rate: f64,
    /// Fraction of non-empty responses that are identical to the first response.
    pub self_consistency_rate: f64,
    /// Average latency per call (milliseconds). None if not measured.
    pub avg_latency_ms: Option<f64>,
}

impl ModelBenchResult {
    /// Accuracy: fraction of correct responses out of total.
    pub fn accuracy(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.correct_count as f64 / self.total_count as f64
        }
    }
}

/// The canonical evaluation task suite covering common bioinformatics operations.
pub fn canonical_eval_tasks() -> Vec<EvalTask> {
    vec![
        // ── Alignment ─────────────────────────────────────────────────────────
        EvalTask {
            tool: "bwa".to_string(),
            task: "align reads.fastq to ref.fa with 8 threads".to_string(),
            required_patterns: vec!["mem".to_string(), "-t 8".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "bwa-mem2".to_string(),
            task: "align paired reads R1.fastq R2.fastq to reference.fa with read group ID sample1".to_string(),
            required_patterns: vec!["mem".to_string(), "-R".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "bowtie2".to_string(),
            task: "align R1.fastq.gz and R2.fastq.gz to index bt2_index with 4 threads".to_string(),
            required_patterns: vec!["-x".to_string(), "-1".to_string(), "-2".to_string()],
            category: "alignment".to_string(),
        },
        // ── QC ───────────────────────────────────────────────────────────────
        EvalTask {
            tool: "fastp".to_string(),
            task: "quality trim paired reads R1.fastq.gz R2.fastq.gz with adapter auto-detection and 8 threads".to_string(),
            required_patterns: vec!["--in1".to_string(), "--in2".to_string(), "--detect_adapter_for_pe".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "sort aligned.bam by coordinate and output to sorted.bam using 4 threads".to_string(),
            required_patterns: vec!["sort".to_string(), "-o".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "index sorted.bam".to_string(),
            required_patterns: vec!["index".to_string()],
            category: "qc".to_string(),
        },
        // ── Variant calling ────────────────────────────────────────────────
        EvalTask {
            tool: "gatk".to_string(),
            task: "call variants in gVCF mode on sample.bam against reference.fa".to_string(),
            required_patterns: vec!["HaplotypeCaller".to_string(), "-ERC GVCF".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "bcftools".to_string(),
            task: "call SNVs and indels from input.bcf with ploidy 2".to_string(),
            required_patterns: vec!["call".to_string(), "-m".to_string()],
            category: "variant-calling".to_string(),
        },
        // ── Quantification ─────────────────────────────────────────────────
        EvalTask {
            tool: "featureCounts".to_string(),
            task: "count reads in aligned.bam against annotation.gtf for paired-end data with 8 threads".to_string(),
            required_patterns: vec!["-a".to_string(), "-p".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "salmon".to_string(),
            task: "quantify expression from R1.fastq.gz and R2.fastq.gz against index salmon_index".to_string(),
            required_patterns: vec!["quant".to_string(), "-1".to_string(), "-2".to_string()],
            category: "quantification".to_string(),
        },
        // ── Metagenomics ────────────────────────────────────────────────────
        EvalTask {
            tool: "kraken2".to_string(),
            task: "classify paired reads R1.fastq.gz R2.fastq.gz against database /db/kraken2 and write report to report.txt".to_string(),
            required_patterns: vec!["--db".to_string(), "--paired".to_string(), "--report".to_string()],
            category: "metagenomics".to_string(),
        },
        EvalTask {
            tool: "bracken".to_string(),
            task: "estimate abundance from kraken2 report.txt with database /db/kraken2 and read length 150".to_string(),
            required_patterns: vec!["-d".to_string(), "-i".to_string(), "-r 150".to_string()],
            category: "metagenomics".to_string(),
        },
    ]
}

/// Parse a raw LLM response string for `ARGS:` and `EXPLANATION:` lines.
///
/// Returns `(args, explanation)` where `args` is `None` if the format is invalid.
pub fn parse_llm_response(raw: &str) -> (Option<String>, Option<String>) {
    let args = raw
        .lines()
        .find(|l| l.trim_start().starts_with("ARGS:"))
        .map(|l| l.trim_start_matches("ARGS:").trim().to_string());

    let explanation = raw
        .lines()
        .find(|l| l.trim_start().starts_with("EXPLANATION:"))
        .map(|l| l.trim_start_matches("EXPLANATION:").trim().to_string());

    (args, explanation)
}

/// Check whether a parsed ARGS line satisfies all required patterns for a task.
pub fn check_correctness(args: &str, required_patterns: &[String]) -> bool {
    required_patterns
        .iter()
        .all(|pat| args.contains(pat.as_str()))
}

/// Compute self-consistency: fraction of responses matching the first non-empty response.
pub fn compute_consistency(responses: &[Option<String>]) -> f64 {
    let non_empty: Vec<&str> = responses
        .iter()
        .filter_map(|r| r.as_deref())
        .filter(|s| !s.is_empty())
        .collect();

    if non_empty.len() <= 1 {
        return 1.0;
    }

    let reference = non_empty[0];
    let matching = non_empty.iter().filter(|&&r| r == reference).count();
    matching as f64 / non_empty.len() as f64
}

/// Run the LLM model benchmark for an offline-simulated scenario
/// (without actual LLM API calls — for unit testing the harness logic).
///
/// In production use, the `oxo-bench eval-models` CLI command wires up real LLM calls.
pub fn run_model_bench(
    config: &ModelBenchConfig,
    tasks: &[EvalTask],
    // Injected response generator — allows testing without real API calls.
    response_fn: &dyn Fn(&str, &str) -> String,
) -> Vec<ModelBenchResult> {
    tasks
        .iter()
        .map(|task| {
            let mut correct_count = 0;
            let mut valid_format = 0;
            let mut args_responses: Vec<Option<String>> = Vec::new();

            for _ in 0..config.n_repeats {
                let raw = response_fn(&task.tool, &task.task);
                let (args, _explanation) = parse_llm_response(&raw);

                if args.is_some() {
                    valid_format += 1;
                }

                if let Some(ref a) = args
                    && check_correctness(a, &task.required_patterns) {
                    correct_count += 1;
                }
                args_responses.push(args);
            }

            let consistency = compute_consistency(&args_responses);
            let format_rate = valid_format as f64 / config.n_repeats as f64;

            ModelBenchResult {
                model: config.model.clone(),
                tool: task.tool.clone(),
                task_summary: task.task.chars().take(60).collect(),
                category: task.category.clone(),
                correct_count,
                total_count: config.n_repeats,
                format_validity_rate: format_rate,
                self_consistency_rate: consistency,
                avg_latency_ms: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_response(tool: &str, _task: &str) -> String {
        // Simulate a well-formed response for well-known tools.
        match tool {
            "samtools" => "ARGS: sort -o sorted.bam -@ 4 aligned.bam\nEXPLANATION: Sort by coordinate.".to_string(),
            "fastp" => "ARGS: --in1 R1.fastq.gz --in2 R2.fastq.gz --detect_adapter_for_pe\nEXPLANATION: Trim paired reads.".to_string(),
            _ => "ARGS: --help\nEXPLANATION: Show help.".to_string(),
        }
    }

    #[test]
    fn test_parse_valid_response() {
        let raw = "ARGS: sort -o out.bam in.bam\nEXPLANATION: Sort BAM file.";
        let (args, expl) = parse_llm_response(raw);
        assert_eq!(args.as_deref(), Some("sort -o out.bam in.bam"));
        assert_eq!(expl.as_deref(), Some("Sort BAM file."));
    }

    #[test]
    fn test_parse_invalid_response() {
        let raw = "some random text without required format";
        let (args, _) = parse_llm_response(raw);
        assert!(args.is_none());
    }

    #[test]
    fn test_check_correctness() {
        let patterns = vec!["sort".to_string(), "-o".to_string()];
        assert!(check_correctness("sort -o out.bam input.bam", &patterns));
        assert!(!check_correctness("view -b input.bam", &patterns));
    }

    #[test]
    fn test_consistency_perfect() {
        let responses = vec![
            Some("sort -o out.bam".to_string()),
            Some("sort -o out.bam".to_string()),
            Some("sort -o out.bam".to_string()),
        ];
        assert_eq!(compute_consistency(&responses), 1.0);
    }

    #[test]
    fn test_consistency_partial() {
        let responses = vec![
            Some("sort -o out.bam".to_string()),
            Some("sort -o out.bam".to_string()),
            Some("sort -n out.bam".to_string()),
        ];
        let c = compute_consistency(&responses);
        assert!((c - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_run_model_bench_basic() {
        let config = ModelBenchConfig {
            model: "mock".to_string(),
            n_repeats: 3,
            temperature: 0.0,
            max_tokens: 256,
        };
        let tasks = canonical_eval_tasks();
        let results = run_model_bench(&config, &tasks[..2], &mock_response);
        assert_eq!(results.len(), 2);
        for r in &results {
            assert_eq!(r.total_count, 3);
        }
    }

    #[test]
    fn test_canonical_eval_tasks_not_empty() {
        let tasks = canonical_eval_tasks();
        assert!(!tasks.is_empty());
    }
}
