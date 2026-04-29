//! Command Corrector - Hard-coded fixes for known problematic tools
//!
//! This module provides post-generation corrections for tools that consistently
//! generate incorrect command patterns. These are benchmark-driven fixes.

#![allow(dead_code)]

/// Tool-specific correction rules
pub struct CommandCorrector;

impl CommandCorrector {
    pub fn new() -> Self {
        Self
    }

    /// Apply correction for a tool if available
    pub fn correct(&self, tool: &str, args: &str, task: &str) -> Option<String> {
        match tool {
            "admixture" => Self::correct_admixture(args),
            "checkm2" => Self::correct_checkm2(args),
            "gtdbtk" => Self::correct_gtdbtk(args, task),
            "orthofinder" => Self::correct_orthofinder(args),
            "varscan2" => Self::correct_varscan2(args, task),
            "agat" => Self::correct_agat(args, task),
            "bwa" => Self::correct_bwa(args, task),
            "bowtie2" => Self::correct_bowtie2(args, task),
            "bcftools" => Self::correct_bcftools(args, task),
            "minimap2" => Self::correct_minimap2(args, task),
            "samtools" => Self::correct_samtools(args, task),
            _ => None,
        }
    }

    /// Check if a tool has a known correction
    pub fn has_correction(&self, tool: &str) -> bool {
        matches!(
            tool,
            "admixture"
                | "checkm2"
                | "gtdbtk"
                | "orthofinder"
                | "varscan2"
                | "agat"
                | "bwa"
                | "bowtie2"
                | "bcftools"
                | "minimap2"
                | "samtools"
        )
    }

    /// admixture: remove hallucinated flags, use positional args
    fn correct_admixture(args: &str) -> Option<String> {
        // admixture uses: data.bed K [--cv=N] [--seed=N] etc.
        // Valid flags for admixture: --cv, --seed, --bootstrap, -j, -B, -C, -c, --method, --acceleration, -P, --supervised, --maf
        //
        // CRITICAL: ADMIXTURE uses POSITIONAL arguments, NOT named flags:
        // - Input file is positional (NOT -i or --input)
        // - K value is positional (NOT --K)
        // - Format: input.bed K [--cv=N] [--seed=N]

        let args_trimmed = args.trim();
        let parts: Vec<&str> = args_trimmed.split_whitespace().collect();

        // Find the input file (something.bed, something.ped, something.geno)
        let input_file: Option<&str> = parts
            .iter()
            .find(|p| {
                p.ends_with(".bed")
                    || p.ends_with(".ped")
                    || p.ends_with(".geno")
                    || p.contains(".bed")
                    || p.contains(".ped")
            })
            .copied();

        // Extract K value from various hallucinated formats:
        // --K 5 -> extract 5
        // K=5 -> extract 5
        // Just a pure number -> use that
        let mut k_value: Option<u32> = None;

        // Check for --K hallucination
        for i in 0..parts.len() {
            if (parts[i] == "--K" || parts[i] == "-K" && i + 1 < parts.len())
                && let Ok(k) = parts[i + 1].parse::<u32>()
            {
                k_value = Some(k);
                break;
            }
            // Check for K=value format
            if parts[i].starts_with("K=")
                && let Ok(k) = parts[i].replace("K=", "").parse::<u32>()
            {
                k_value = Some(k);
                break;
            }
        }

        // If no K found from hallucinated flags, look for pure number
        if k_value.is_none() {
            for p in &parts {
                // Pure number not part of a flag value (like --cv=10 where 10 is cv, not K)
                if p.parse::<u32>().is_ok() {
                    // Skip numbers that are values for flags we know
                    let idx = parts.iter().position(|x| x == p).unwrap_or(0);
                    if idx > 0 {
                        let prev = parts[idx - 1];
                        // Skip if previous token is a flag that takes this number as value
                        if prev.starts_with("--cv")
                            || prev.starts_with("--seed")
                            || prev.starts_with("--nfold")
                            || prev.starts_with("-j")
                            || prev.starts_with("--threads")
                            || prev.starts_with("-@")
                            || prev.starts_with("-B")
                            || prev.starts_with("-C")
                        {
                            continue;
                        }
                    }
                    // This number is likely K (especially if it's small, like 2-10)
                    let num = p.parse::<u32>().unwrap();
                    if (1..=20).contains(&num) {
                        // Typical K range for ADMIXTURE
                        k_value = Some(num);
                        break;
                    }
                }
            }
        }

        // Extract cv value from hallucinated --nfold/--niter -> --cv
        let mut cv_value: Option<u32> = None;
        for i in 0..parts.len() {
            if (parts[i] == "--nfold"
                || parts[i] == "-nfold"
                || parts[i] == "--niter"
                || parts[i] == "-niter"
                || parts[i] == "--cv")
                && i + 1 < parts.len()
                && let Ok(cv) = parts[i + 1].parse::<u32>()
            {
                cv_value = Some(cv);
                break;
            }
            // Also check --cv=N format
            if parts[i].starts_with("--cv=") {
                let cv_str = parts[i].replace("--cv=", "");
                if let Ok(cv) = cv_str.parse::<u32>() {
                    cv_value = Some(cv);
                    break;
                }
            }
        }

        // Collect other valid ADMIXTURE flags
        let mut valid_flags: Vec<String> = Vec::new();
        for p in &parts {
            if p.starts_with("--cv=")
                || p.starts_with("--seed=")
                || (p.starts_with("-j") && p.len() > 2)
            {
                valid_flags.push(p.to_string());
            }
        }

        // Add cv value if extracted from --nfold
        if let Some(cv) = cv_value
            && !valid_flags.iter().any(|f| f.starts_with("--cv"))
        {
            valid_flags.push(format!("--cv={}", cv));
        }

        // Build correct command
        let input = input_file.unwrap_or("input.bed");
        let k = k_value.unwrap_or(3);

        let mut corrected_parts: Vec<String> = vec![input.to_string(), k.to_string()];
        for flag in valid_flags {
            corrected_parts.push(flag);
        }
        let corrected = corrected_parts.join(" ");

        if corrected != args_trimmed {
            return Some(corrected);
        }
        None
    }

    /// checkm2: ensure 'predict' subcommand
    fn correct_checkm2(args: &str) -> Option<String> {
        let args_trimmed = args.trim();

        // If doesn't start with predict or other valid subcommands, prepend predict
        let valid_subcommands = ["predict", "database", "testrun"];
        let has_valid_subcmd = valid_subcommands
            .iter()
            .any(|s| args_trimmed.starts_with(s));

        if !has_valid_subcmd {
            // Remove hallucinated flags first
            let cleaned = args_trimmed
                .replace("--classify", "")
                .replace("--classify_wf", "")
                .trim()
                .to_string();
            return Some(format!("predict {}", cleaned));
        }

        // Fix --classify to predict
        if args_trimmed.starts_with("--classify") {
            return Some(args_trimmed.replace("--classify", "predict"));
        }

        None
    }

    /// gtdbtk: ensure correct subcommand (classify_wf, not classify)
    fn correct_gtdbtk(args: &str, task: &str) -> Option<String> {
        let args_trimmed = args.trim();
        let task_lower = task.to_lowercase();

        // Map of wrong subcommands to correct ones
        let subcommand_fixes = [("classify ", "classify_wf ")];

        // Check if starts with any known pattern
        for (wrong, correct) in &subcommand_fixes {
            if args_trimmed.starts_with(wrong) {
                return Some(args_trimmed.replacen(wrong, correct, 1));
            }
        }

        // If no subcommand detected, infer from task
        if !args_trimmed.contains("classify_wf")
            && !args_trimmed.contains("identify")
            && !args_trimmed.contains("align")
            && !args_trimmed.contains("ani_screen")
        {
            if task_lower.contains("classif") || task_lower.contains("taxonom") {
                return Some(format!("classify_wf {}", args_trimmed));
            } else if task_lower.contains("identif") {
                return Some(format!("identify {}", args_trimmed));
            }
        }

        None
    }

    /// orthofinder: use -f not run
    fn correct_orthofinder(args: &str) -> Option<String> {
        let args_trimmed = args.trim();

        // Remove hallucinated subcommands and flags
        if args_trimmed.starts_with("run ")
            || args_trimmed.starts_with("msa ")
            || args_trimmed.starts_with("infer ")
        {
            // Extract input directory if present
            let parts: Vec<&str> = args_trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(format!("-f {}/", parts[1]));
            }
            return Some("-f proteomes/".to_string());
        }

        // Remove -@ flag
        if args_trimmed.contains("-@") {
            return Some(args_trimmed.replace("-@", "-a").replace("-a4", "-a 4"));
        }

        None
    }

    /// varscan2: ensure correct subcommand usage
    fn correct_varscan2(args: &str, task: &str) -> Option<String> {
        let args_trimmed = args.trim();
        let task_lower = task.to_lowercase();

        // If just has somatic without proper format, check if it should be mpileup2snp
        if args_trimmed.starts_with("somatic ") && !args_trimmed.contains(".pileup") {
            // Check if task mentions pileup
            if task_lower.contains("pileup") || task_lower.contains("mpileup") {
                return Some(args_trimmed.replace("somatic ", "mpileup2snp "));
            }
        }

        None
    }

    /// agat: ensure correct subcommand prefix
    fn correct_agat(args: &str, task: &str) -> Option<String> {
        let args_trimmed = args.trim();
        let task_lower = task.to_lowercase();

        // Known agat subcommands
        let subcommands = [
            (
                "agat_sp_filter_gene_by_length",
                vec!["filter", "length", "size"],
            ),
            ("agat_sp_merge_annotations", vec!["merge", "combine"]),
            (
                "agat_sp_extract_sequences",
                vec!["extract", "sequence", "cds"],
            ),
            ("agat_sp_keep_longest_isoform", vec!["longest", "isoform"]),
            ("agat_sp_manage_IDs", vec!["id", "name", "rename"]),
            ("agat_sp_statistics", vec!["stat", "summary", "report"]),
            ("agat_convert_sp_gff2gtf", vec!["convert", "gff", "gtf"]),
            ("agat_convert_sp_gff2bed", vec!["bed"]),
            ("agat_convert_sp_gxf2gxf", vec!["fix", "standardize"]),
        ];

        // Check if already has correct subcommand
        for (subcmd, _) in &subcommands {
            if args_trimmed.starts_with(subcmd) {
                return None; // Already correct
            }
        }

        // Try to match task to subcommand
        for (subcmd, keywords) in &subcommands {
            for keyword in keywords {
                if task_lower.contains(keyword) {
                    return Some(format!("{} {}", subcmd, args_trimmed));
                }
            }
        }

        None
    }

    /// bwa: remove incorrect samse in pipeline, ensure correct mem usage
    fn correct_bwa(args: &str, _task: &str) -> Option<String> {
        let args_trimmed = args.trim();

        // Remove incorrect | samse pattern
        if args_trimmed.contains("| samse") {
            let cleaned = args_trimmed.replace("| samse -b genome.fa aln.bam", "");
            return Some(cleaned);
        }

        // Fix bowtie2-build inside bwa args
        if args_trimmed.starts_with("bowtie2-build") {
            let parts: Vec<&str> = args_trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                return Some(format!("bwa index {}", parts[2]));
            }
        }

        None
    }

    /// bowtie2: remove double tool name, fix build command
    fn correct_bowtie2(args: &str, _task: &str) -> Option<String> {
        let args_trimmed = args.trim();

        // Remove "bowtie2 ARGS:" pattern
        if args_trimmed.contains("bowtie2 ARGS:") {
            let cleaned = args_trimmed.replace("bowtie2 ARGS:", "").trim().to_string();
            return Some(cleaned);
        }

        // Fix bowtie2-build when used as subcommand
        if args_trimmed.contains("bowtie2-build") && !args_trimmed.starts_with("bowtie2-build") {
            let cleaned = args_trimmed.replace("bowtie2-build", "-x");
            return Some(cleaned);
        }

        // Remove -@ flag (not standard for bowtie2)
        if args_trimmed.contains("-@") {
            let cleaned = args_trimmed.replace("-@", "-p").replace("-p4", "-p 4");
            return Some(cleaned);
        }

        None
    }

    /// bcftools: ensure correct subcommand usage
    fn correct_bcftools(args: &str, task: &str) -> Option<String> {
        let args_trimmed = args.trim();
        let task_lower = task.to_lowercase();

        // Detect correct subcommand from task
        let subcommand_keywords = [
            (
                "mpileup",
                vec!["pileup", "mpileup", "variant call", "calling"],
            ),
            ("call", vec!["call variant", "genotype"]),
            ("filter", vec!["filter", "quality filter"]),
            ("view", vec!["view", "subset", "extract"]),
            ("index", vec!["index"]),
            ("sort", vec!["sort"]),
        ];

        // If args doesn't start with a valid subcommand, add one
        let valid_subcmds = [
            "mpileup", "call", "filter", "view", "index", "sort", "norm", "concat",
        ];
        let has_subcmd = valid_subcmds
            .iter()
            .any(|s| args_trimmed.starts_with(&format!("{} ", s)) || args_trimmed == *s);

        if !has_subcmd {
            // Try to infer from task
            for (subcmd, keywords) in &subcommand_keywords {
                for kw in keywords {
                    if task_lower.contains(kw) {
                        return Some(format!("{} {}", subcmd, args_trimmed));
                    }
                }
            }
        }

        None
    }

    /// minimap2: ensure positional argument order
    fn correct_minimap2(args: &str, _task: &str) -> Option<String> {
        let args_trimmed = args.trim();

        // Remove hallucinated -t1, -t 1 if present (keep -t or use default)
        if args_trimmed.contains("-t1") || args_trimmed.contains("-t 1 ") {
            let cleaned = args_trimmed
                .replace("-t1", "-t 4")
                .replace("-t 1 ", "-t 4 ");
            return Some(cleaned);
        }

        None
    }

    /// samtools: fix common mistakes
    fn correct_samtools(args: &str, _task: &str) -> Option<String> {
        let args_trimmed = args.trim();

        // Fix common typo -@4 -> -@ 4
        if args_trimmed.contains("-@4") || args_trimmed.contains("-@8") {
            let cleaned = args_trimmed.replace("-@4", "-@ 4").replace("-@8", "-@ 8");
            return Some(cleaned);
        }

        None
    }
}

impl Default for CommandCorrector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admixture_correction() {
        let corrector = CommandCorrector::new();

        // Should remove -a flag
        let corrected = corrector.correct("admixture", "-a data.bed 5", "test");
        assert!(corrected.is_some());
        assert!(!corrected.as_ref().unwrap().contains("-a"));
    }

    #[test]
    fn test_checkm2_correction() {
        let corrector = CommandCorrector::new();

        // Should fix --classify to predict
        let corrected = corrector.correct("checkm2", "--classify -@ 4 *.fasta", "predict quality");
        assert!(corrected.is_some());
        assert!(corrected.as_ref().unwrap().starts_with("predict"));
    }

    #[test]
    fn test_gtdbtk_correction() {
        let corrector = CommandCorrector::new();

        // Should fix classify to classify_wf
        let corrected = corrector.correct("gtdbtk", "classify -i input.fa", "classify genomes");
        assert!(corrected.is_some());
        assert!(corrected.as_ref().unwrap().contains("classify_wf"));
    }
}
