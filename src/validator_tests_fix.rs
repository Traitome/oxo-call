
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_subcommand_requirement() {
        let sdoc = StructuredDoc {
            usage: "Usage: samtools <command> [options]".to_string(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: vec![],
            flag_catalog: vec![],
            extracted_examples: vec![],
            quality_score: 0.0,
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            subcommands: vec![],
            subcommand_descriptions: Vec::new(),
            format_hint: None,
            companion_binaries: vec![],
        };
        assert!(detect_subcommand_requirement(&sdoc));

        let sdoc2 = StructuredDoc {
            usage: "Usage: fastp [options] -i input -o output".to_string(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: vec![],
            flag_catalog: vec![],
            extracted_examples: vec![],
            quality_score: 0.0,
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            subcommands: vec![],
            subcommand_descriptions: Vec::new(),
            format_hint: None,
            companion_binaries: vec![],
        };
        assert!(!detect_subcommand_requirement(&sdoc2));
    }

    #[test]
    fn test_is_likely_hallucinated_subcommand() {
        assert!(is_likely_hallucinated_subcommand("run"));
        assert!(is_likely_hallucinated_subcommand("process"));
        assert!(is_likely_hallucinated_subcommand("pileup"));
        assert!(!is_likely_hallucinated_subcommand("--help"));
    }

    #[test]
    fn test_pattern_based_correction_removes_hallucinated_subcommand() {
        let sdoc = StructuredDoc {
            usage: "Usage: fastp [options]".to_string(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: vec![],
            flag_catalog: vec![
                FlagEntry {
                    flag: "-i".to_string(),
                    description: "input file".to_string(),
                    value_type: None,
                    required: false,
                    default: None,
                    alt_form: None,
                    enum_values: Vec::new(),
                },
            ],
            extracted_examples: vec![],
            quality_score: 0.0,
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            subcommands: vec![],
            subcommand_descriptions: Vec::new(),
            format_hint: None,
            companion_binaries: vec![],
        };

        let input = "run -i input.fastq";
        let result = pattern_based_correction(input, &sdoc, "fastp");
        assert!(!result.contains("run"));
        assert!(result.contains("-i"));
    }

    #[test]
    fn test_correct_admixture() {
        // Model generates: admixture -i data.bed -K 5 --cv=10
        // Should be: admixture data.bed 5 --cv=10
        let input = "-i data.bed -K 5 --cv=10";
        let result = correct_admixture(input);
        assert!(!result.contains("-i "));
        assert!(!result.contains("-K "));
        assert!(result.contains("data.bed"));
        assert!(result.contains("5"));
        assert!(result.contains("--cv=10"));
    }
}
