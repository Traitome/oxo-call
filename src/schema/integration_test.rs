//! Integration test for Schema effectiveness

#[cfg(test)]
mod tests {
    use crate::schema::{CliSchema, build_schema_prompt_section, parse_help};

    #[test]
    fn test_samtools_sort_schema_extraction() {
        let samtools_sort_help = r#"Usage: samtools sort [options...] [in.bam]
Options:
  -l INT     Set compression level, from 0 (uncompressed) to 9 (best)
  -u         Output uncompressed data (equivalent to -l 0)
  -m INT     Set maximum memory per thread; suffix K/M/G recognized [768M]
  -n         Sort by read name (natural): cannot be used with samtools index
  -o FILE    Write final output to FILE rather than standard output
  -T PREFIX  Write temporary files to PREFIX.nnnn.bam
  -@, --threads INT
               Number of additional threads to use [0]
  --output-fmt FORMAT[,OPT[=VAL]]...
               Specify output format (SAM, BAM, CRAM)
  --reference FILE
               Reference sequence FASTA FILE [null]
"#;

        let schema = parse_help("samtools", samtools_sort_help);

        // Should extract key flags
        assert!(
            schema.flags.len() >= 5,
            "Expected at least 5 flags, got {}",
            schema.flags.len()
        );

        // Check critical flags are present
        let has_threads = schema
            .flags
            .iter()
            .any(|f| f.matches_name("-@") || f.matches_name("--threads"));
        let has_output = schema.flags.iter().any(|f| f.matches_name("-o"));
        let has_memory = schema.flags.iter().any(|f| f.matches_name("-m"));

        assert!(has_threads, "Missing -@/--threads flag");
        assert!(has_output, "Missing -o output flag");
        assert!(has_memory, "Missing -m memory flag");

        println!("Extracted {} flags from samtools sort", schema.flags.len());
        for flag in &schema.flags {
            println!("  {} ({:?})", flag.name, flag.param_type);
        }
    }

    #[test]
    fn test_schema_detects_hallucinated_flag() {
        let help = r#"Usage: tool [options]
  -i FILE    Input file
  -o FILE    Output file
  -v         Verbose
"#;
        let schema = parse_help("tool", help);

        // Hallucinated flag should be detected
        let hallucinated_args: Vec<(String, Option<String>)> =
            vec![("--invalid-flag".to_string(), Some("file.txt".to_string()))];
        let result = schema.validate_command(&hallucinated_args, None);

        assert!(
            !result.is_valid,
            "Hallucinated flag should be detected as invalid"
        );
        assert!(!result.errors.is_empty(), "Should have validation error");

        // Error should be InvalidFlag type
        let has_invalid_flag_error = result
            .errors
            .iter()
            .any(|e| matches!(e, crate::schema::ValidationError::InvalidFlag { .. }));
        assert!(has_invalid_flag_error, "Should have InvalidFlag error");
    }

    #[test]
    fn test_schema_whitelist_prompt() {
        let help = r#"Usage: tool [options]
  -i FILE    Input file (required)
  -o FILE    Output file (required)
  -v         Verbose mode
"#;
        let schema = parse_help("tool", help);

        let section = build_schema_prompt_section(&schema, "process input file");

        // Section should mention valid flags
        assert!(
            section.contains("-i") || section.contains("-o"),
            "Should mention valid flags"
        );
        assert!(
            section.contains("Valid Flags"),
            "Should have valid flags header"
        );

        // Should have whitelist warning
        assert!(
            section.contains("use ONLY these"),
            "Should have whitelist warning"
        );
    }

    #[test]
    fn test_fastp_python_argparse_style() {
        // Python argparse style (common in bioinformatics)
        let help = r#"usage: fastp [options]

optional arguments:
  -h, --help            show this help message and exit
  -i INPUT, --input INPUT
                        input fastq file
  -o OUTPUT, --output OUTPUT
                        output fastq file
  -w INT, --threads INT
                        worker threads number [default: 2]
  -q INT, --qualified_quality_phred INT
                        quality threshold
"#;

        let schema = parse_help("fastp", help);

        // Should detect this as Python argparse style
        assert_eq!(schema.schema_source, "python-argparse");

        // Should extract flags with type inference
        let input_flag = schema.flags.iter().find(|f| f.matches_name("-i"));
        assert!(input_flag.is_some(), "Should find -i flag");
        let input = input_flag.unwrap();
        assert_eq!(
            input.param_type,
            crate::schema::ParamType::File,
            "-i should be File type"
        );

        let threads_flag = schema
            .flags
            .iter()
            .find(|f| f.matches_name("-w") || f.matches_name("--threads"));
        assert!(threads_flag.is_some(), "Should find threads flag");
        let threads = threads_flag.unwrap();
        assert_eq!(
            threads.param_type,
            crate::schema::ParamType::Int,
            "threads should be Int type"
        );
    }
}
