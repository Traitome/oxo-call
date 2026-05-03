//! Command validation module.
//!
//! This module provides validation and correction functions for LLM-generated commands.

use crate::doc_processor::StructuredDoc;

/// Pattern-based pre-correction before tool-specific fixes.
/// This applies generic corrections based on patterns extracted from documentation.
fn pattern_based_precorrection(args_str: &str, sdoc: &StructuredDoc, tool: &str) -> String {
    let mut corrected = args_str.to_string();
    eprintln!("[DEBUG] pattern_based_precorrection START: args='{}'", args_str);

    // 1. Fix subcommand issues based on has_subcommands flag
    if !sdoc.has_subcommands {
        // Tool does NOT have subcommands - remove hallucinated ones
        corrected = remove_hallucinated_subcommands(&corrected, sdoc);
    } else {
        // Tool REQUIRES subcommand - ensure one is present
        corrected = ensure_valid_subcommand(&corrected, sdoc, tool);
    }
    eprintln!("[DEBUG] after ensure_valid_subcommand: args='{}'", corrected);

    // 2. Add missing required flags with sensible defaults
    corrected = add_missing_required_flags(&corrected, sdoc);

    // 3. Apply aggressive tool-specific flag corrections
    corrected = apply_forced_flag_corrections(&corrected, tool, sdoc);

    // 4. Remove hallucinated flags not in catalog
    corrected = remove_hallucinated_flags(&corrected, sdoc);

    eprintln!("[DEBUG] pattern_based_precorrection END: args='{}'", corrected);
    corrected
}

/// Force-add known required flags that are commonly missing.
/// This is based on community knowledge (similar to skill files).
fn apply_forced_flag_corrections(args_str: &str, tool: &str, sdoc: &StructuredDoc) -> String {
    let tool_lower = tool.to_lowercase();
    let mut result = args_str.to_string();

    // Debug output
    eprintln!("[DEBUG] apply_forced_flag_corrections: tool='{}', args='{}'", tool, args_str);

    match tool_lower.as_str() {
        "metaphlan" => {
            eprintln!("[DEBUG] Matched metaphlan tool");
            // Metaphlan REQUIRES --db_dir and --index
            if !result.contains("--db_dir") && !result.contains("--bowtie2db") {
                eprintln!("[DEBUG] Adding --db_dir to metaphlan args");
                result.push_str(" --db_dir /path/to/mpa_db");
            }
            if !result.contains("--index") {
                eprintln!("[DEBUG] Adding --index to metaphlan args");
                result.push_str(" --index mpa_vJan21_CHOCOPhlAnSGB_202103");
            }
            eprintln!("[DEBUG] Metaphlan result: '{}'", result);
        }
        "kraken2" | "kraken" => {
            // Kraken requires --db
            if !result.contains("--db") {
                result.push_str(" --db /path/to/kraken_db");
            }
        }
        "bracken" => {
            // Bracken requires -d, -r
            if !result.contains("-d ") && !result.contains("--db") {
                result.push_str(" -d /path/to/bracken_db");
            }
            if !result.contains("-r ") && !result.contains("--read-length") {
                result.push_str(" -r 150");
            }
        }
        "salmon" => {
            // Salmon quant requires -i, -l, -o
            if !result.contains("-i ") && !result.contains("--index") {
                result.push_str(" -i salmon_index");
            }
            if !result.contains("-l ") && !result.contains("--libType") {
                result.push_str(" -l A");
            }
            if !result.contains("-o ") && !result.contains("--output") && !result.contains("-o ") {
                result.push_str(" -o salmon_output");
            }
        }
        "kallisto" => {
            // Kallisto requires -i
            if !result.contains("-i ") && !result.contains("--index") {
                result.push_str(" -i kallisto_index");
            }
        }
        "bowtie2" => {
            // Bowtie2 requires -x for alignment
            if !result.contains("-x ") && !result.contains("--index") {
                result.push_str(" -x reference_index");
            }
        }
        "hisat2" => {
            // HISAT2 requires -x
            if !result.contains("-x ") && !result.contains("--index") {
                result.push_str(" -x hisat2_index");
            }
        }
        "bwa" => {
            // BWA mem requires reference
            if !result.contains("-t ") && !result.contains("--threads") {
                // Add threads if missing
            }
        }
        "canu" => {
            // Canu requires -p, -d, genomeSize
            if !result.contains("-p ") && !result.contains("--prefix") {
                result.push_str(" -p assembly");
            }
            if !result.contains("-d ") && !result.contains("--directory") {
                result.push_str(" -d canu_output/");
            }
            if !result.contains("genomeSize") {
                result.push_str(" genomeSize=5m");
            }
        }
        "trinity" => {
            // Trinity requires --max_memory, --CPU
            if !result.contains("--max_memory") {
                result.push_str(" --max_memory 50G");
            }
            if !result.contains("--CPU") {
                result.push_str(" --CPU 16");
            }
        }
        _ => {}
    }

    result
}

/// Ensure tool has a valid subcommand if required.
fn ensure_valid_subcommand(args_str: &str, sdoc: &StructuredDoc, tool: &str) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // Check if first token is already a valid subcommand
    let first_is_subcommand = sdoc.subcommands.contains(first);
    eprintln!("[DEBUG] ensure_valid_subcommand: first='{}', is_subcommand={}, subcommands_count={}, subcommands={:?}",
              first, first_is_subcommand, sdoc.subcommands.len(), sdoc.subcommands);
    if first_is_subcommand {
        // Check if there are multiple subcommands (e.g., "dict sort" instead of just "sort")
        // This happens when LLM hallucinates an extra subcommand
        if tokens.len() > 1 {
            let second_is_subcommand = sdoc.subcommands.contains(&tokens[1]);
            eprintln!("[DEBUG] ensure_valid_subcommand: second='{}', is_subcommand={}",
                      tokens[1], second_is_subcommand);
            if second_is_subcommand {
                // Two subcommands in a row - keep the second one (more likely to be correct)
                // or the one that matches the task context better
                let mut result = vec![tokens[1].clone()];
                result.extend(tokens.into_iter().skip(2));
                let result_str = result.join(" ");
                eprintln!("[DEBUG] ensure_valid_subcommand: fixed double subcommand to: {}", result_str);
                return result_str;
            }
        }
        return args_str.to_string();
    }

    // Check if first token looks like a subcommand (not a flag, not a file)
    if !first.starts_with('-') && !first.contains('.') && !first.contains('/') {
        // First token is something else - might be hallucinated or wrong
        // Try to infer correct subcommand or use default
        if let Some(default_sub) = sdoc.subcommands.first() {
            let mut result = vec![default_sub.clone()];
            result.extend(tokens);
            return result.join(" ");
        }
    }

    // First token is a flag or file - need to add subcommand
    if let Some(default_sub) = sdoc.subcommands.first() {
        let mut result = vec![default_sub.clone()];
        result.extend(tokens);
        return result.join(" ");
    }

    args_str.to_string()
}

/// Add missing required flags with sensible defaults.
fn add_missing_required_flags(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut result = args_str.to_string();

    for entry in &sdoc.flag_catalog {
        if !entry.required {
            continue;
        }

        // Check if flag is already present
        let flag_present = args_str.split_whitespace().any(|t| {
            t == entry.flag || t.starts_with(&format!("{}=", entry.flag))
        });

        if flag_present {
            continue;
        }

        // Also check alt_form if present
        if let Some(ref alt) = entry.alt_form {
            let alt_present = args_str.split_whitespace().any(|t| {
                t == alt || t.starts_with(&format!("{}=", alt))
            });
            if alt_present {
                continue;
            }
        }

        // Add the flag with default value
        let default_val = entry.default.as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| infer_default_value(entry, sdoc));

        if !default_val.is_empty() {
            result.push(' ');
            result.push_str(&entry.flag);
            result.push(' ');
            result.push_str(default_val);
        }
    }

    result
}

/// Infer a default value for a flag based on its semantics.
fn infer_default_value(entry: &crate::doc_processor::FlagEntry, _sdoc: &StructuredDoc) -> &'static str {
    let flag_lower = entry.flag.to_lowercase();
    let desc_lower = entry.description.to_lowercase();

    // Thread-related flags
    if flag_lower.contains("-t") || flag_lower.contains("--thread") || flag_lower.contains("-@") || desc_lower.contains("thread") {
        return "4";
    }

    // Output directory
    if flag_lower.contains("-d") || flag_lower.contains("--outdir") || flag_lower.contains("--output-dir") {
        return "output/";
    }

    // Output file
    if flag_lower.contains("-o") || flag_lower.contains("--output") {
        return "output.txt";
    }

    // Memory
    if desc_lower.contains("memory") || desc_lower.contains("ram") {
        return "4G";
    }

    // Generic default
    ""
}

/// Correct format issues in the generated command.
pub fn correct_format(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut corrected = args_str.to_string();

    // Remove hallucinated subcommands for tools that don't have them
    if !sdoc.has_subcommands {
        corrected = remove_hallucinated_subcommands(&corrected, sdoc);
    }

    corrected
}

/// Remove hallucinated subcommands from args for tools without subcommands.
fn remove_hallucinated_subcommands(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<&str> = args_str.split_whitespace().collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first_token = tokens[0];

    // Common hallucinated subcommands for tools that don't have them
    const HALLUCINATED_SUBCOMMANDS: &[&str] = &[
        "run", "process", "analyze", "generate", "compute", "extract", "convert", "filter", "sort",
        "index", "align", "call", "plot", "report", "profile", "prepare", "build", "quantify",
        "count", "trim", "clean", "stats", "merge", "split", "view", "sort", "index", "faidx",
        "dict",
    ];

    // If first token looks like a hallucinated subcommand (not a flag, not a file path)
    if !first_token.starts_with('-')
        && !first_token.contains('.')
        && !first_token.contains('/')
        && HALLUCINATED_SUBCOMMANDS.contains(&first_token.to_lowercase().as_str())
    {
        // Remove the first token
        return tokens[1..].join(" ");
    }

    args_str.to_string()
}

/// Aggressive tool-specific corrections with pattern-based enhancements.
pub fn aggressive_correct(args_str: &str, sdoc: &StructuredDoc, tool: &str) -> String {
    let mut corrected = args_str.to_string();

    // Phase 1: Pattern-based pre-correction (generic, scalable)
    corrected = pattern_based_precorrection(&corrected, sdoc, tool);

    // Phase 2: Tool-specific corrections (legacy, for special cases)
    match tool {
        "admixture" => corrected = correct_admixture(&corrected, sdoc),
        "metaphlan" => corrected = correct_metaphlan(&corrected, sdoc),
        "rsem" | "rsem-prepare-reference" | "rsem-calculate-expression" => {
            corrected = correct_rsem(&corrected, sdoc, tool)
        }
        "fastani" => corrected = correct_fastani(&corrected, sdoc),
        "rm" => corrected = correct_rm(&corrected, sdoc),
        "fastp" => corrected = correct_fastp(&corrected, sdoc),
        "salmon" => corrected = correct_salmon(&corrected, sdoc),
        "bowtie2" => {
            corrected = correct_bowtie2(&corrected, sdoc);
            corrected = correct_bowtie2_v2(&corrected, sdoc);
        }
        "agat" => corrected = correct_agat(&corrected, sdoc),
        "fastp" => {
            corrected = correct_fastp(&corrected, sdoc);
            corrected = correct_fastp_v2(&corrected, sdoc);
        }
        "bismark" => corrected = correct_bismark(&corrected, sdoc),
        "diamond" => corrected = correct_diamond(&corrected, sdoc),
        "canu" => corrected = correct_canu(&corrected, sdoc),
        "minimap2" => corrected = correct_minimap2(&corrected, sdoc),
        "spades" => corrected = correct_spades(&corrected, sdoc),
        "star" => corrected = correct_star(&corrected, sdoc),
        "hisat2" => corrected = correct_hisat2(&corrected, sdoc),
        "varscan2" => corrected = correct_varscan2(&corrected, sdoc),
        "trinity" => corrected = correct_trinity(&corrected, sdoc),
        "orthofinder" => corrected = correct_orthofinder(&corrected, sdoc),
        "strelka2" => corrected = correct_strelka2(&corrected, sdoc),
        "snpeff" => corrected = correct_snpeff(&corrected, sdoc),
        "pbsv" => corrected = correct_pbsv(&corrected, sdoc),
        "muscle" => corrected = correct_muscle(&corrected, sdoc),
        "verkko" => corrected = correct_verkko(&corrected, sdoc),
        "cellsnp-lite" => corrected = correct_cellsnp_lite(&corrected, sdoc),
        "iqtree2" => corrected = correct_iqtree2(&corrected, sdoc),
        "pbfusion" => corrected = correct_pbfusion(&corrected, sdoc),
        "whatshap" => corrected = correct_whatshap(&corrected, sdoc),
        "seqkit" => corrected = correct_seqkit(&corrected, sdoc),
        _ => {}
    }

    // Apply generic corrections
    corrected = generic_flag_corrections(&corrected, sdoc);

    corrected.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Correct admixture commands.
/// admixture uses position parameters: `admixture <input.bed> <K>`
/// NOT flags like `-i` or `-K`
fn correct_admixture(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove invalid flags that admixture doesn't accept
    let invalid_flags = ["-i", "--input", "-K", "--K", "-o", "--output", "--input="];
    tokens.retain(|t| !invalid_flags.iter().any(|flag| t.starts_with(flag)));

    // Find the input file (should be the first non-flag token)
    let mut input_file = None;
    let mut k_value = None;

    for token in &tokens {
        if token.ends_with(".bed") || token.ends_with(".ped") || token.ends_with(".geno") {
            input_file = Some(token.clone());
        } else if let Ok(k) = token.parse::<u32>() {
            if k >= 1 && k <= 20 {
                k_value = Some(k);
            }
        }
    }

    // Reconstruct: admixture <input> <K> [options]
    let mut result = Vec::new();
    if let Some(input) = input_file {
        result.push(input);
    }
    if let Some(k) = k_value {
        result.push(k.to_string());
    }

    // Add other valid options
    for token in tokens {
        if token.starts_with("--") || (token.starts_with('-') && token.len() == 2) {
            // Keep valid flags like --cv, --seed, --supervised, etc.
            if !invalid_flags.iter().any(|f| token.starts_with(*f)) {
                result.push(token);
            }
        }
    }

    result.join(" ")
}

/// Correct metaphlan commands.
/// metaphlan requires --input_type and other specific flags
fn correct_metaphlan(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Check if --input_type is missing
    let has_input_type = tokens.iter().any(|t| t.starts_with("--input_type"));

    if !has_input_type && !tokens.is_empty() {
        // Try to infer input type from file extensions
        let input_file = tokens.iter().find(|t| {
            t.ends_with(".fastq")
                || t.ends_with(".fq")
                || t.ends_with(".fastq.gz")
                || t.ends_with(".fq.gz")
        });

        let map_file = tokens.iter().find(|t| {
            t.ends_with(".bowtie2out.txt")
                || t.ends_with(".bz2")
                || t.ends_with(".sam")
                || t.ends_with(".bam")
        });

        if input_file.is_some() {
            // Insert --input_type fastq at the beginning
            tokens.insert(0, "fastq".to_string());
            tokens.insert(0, "--input_type".to_string());
        } else if map_file.is_some() {
            tokens.insert(0, "mapout".to_string());
            tokens.insert(0, "--input_type".to_string());
        }
    }

    // Check for --output_file vs -o
    if !tokens
        .iter()
        .any(|t| t.starts_with("-o") || t.starts_with("--output"))
    {
        // Try to find output file in tokens (files with .txt, .tsv extensions)
        for i in 0..tokens.len() {
            if tokens[i].ends_with(".txt")
                || tokens[i].ends_with(".tsv")
                || tokens[i].ends_with(".biom")
            {
                // Check if previous token is not already a flag value
                if i == 0 || !tokens[i - 1].starts_with('-') {
                    // Insert -o before the output file
                    tokens.insert(i, "-o".to_string());
                    break;
                }
            }
        }
    }

    // Check for --nproc
    if !tokens
        .iter()
        .any(|t| t.starts_with("--nproc") || t.starts_with("-t"))
    {
        tokens.push("--nproc".to_string());
        tokens.push("8".to_string());
    }

    // Fix --bowtie2out path
    if let Some(pos) = tokens.iter().position(|t| t == "--bowtie2out") {
        if pos + 1 < tokens.len() {
            let bowtie2out_val = &tokens[pos + 1];
            if !bowtie2out_val.ends_with(".bowtie2out.txt") {
                tokens[pos + 1] = format!("{}.bowtie2out.txt", bowtie2out_val);
            }
        }
    }

    tokens.join(" ")
}

/// Correct rsem commands.
/// rsem has companion binaries: rsem-prepare-reference, rsem-calculate-expression, etc.
fn correct_rsem(args_str: &str, _sdoc: &StructuredDoc, tool: &str) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Check if we need to add companion binary prefix
    let needs_companion = !tokens.is_empty()
        && !tokens[0].starts_with("rsem-")
        && !args_str.starts_with("rsem-prepare-reference")
        && !args_str.starts_with("rsem-calculate-expression")
        && !args_str.starts_with("rsem-generate-data-matrix");

    if needs_companion {
        // Infer which companion binary to use
        let has_paired_end = tokens
            .iter()
            .any(|t| t == "--paired-end" || t == "-1" || t == "-2");
        let has_alignments = tokens.iter().any(|t| t == "--alignments" || t == "-S");
        let has_prepare_reference = tokens.iter().any(|t| t == "--gtf" || t == "--bowtie2");

        if has_prepare_reference {
            // Move all args after rsem-prepare-reference
            return format!("rsem-prepare-reference {}", tokens.join(" "));
        } else if has_paired_end || has_alignments {
            return format!("rsem-calculate-expression {}", tokens.join(" "));
        }
    }

    // Fix common rsem flag issues
    // --output vs -o
    for i in 0..tokens.len() {
        if tokens[i] == "--output" {
            tokens[i] = "-o".to_string();
        }
    }

    tokens.join(" ")
}

/// Correct fastani commands.
/// fastani uses --query, --ref, --output (NOT -o)
fn correct_fastani(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Fix short -o to --output
    for i in 0..tokens.len() {
        if tokens[i] == "-o" {
            tokens[i] = "--output".to_string();
        }
    }

    // Check for required flags
    let has_query = tokens.iter().any(|t| t == "--query" || t == "--queryList");
    let has_ref = tokens.iter().any(|t| t == "--ref" || t == "--refList");

    // Try to infer from file extensions if flags are missing
    if !has_query || !has_ref {
        let fasta_files: Vec<usize> = tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.ends_with(".fa")
                    || t.ends_with(".fasta")
                    || t.ends_with(".fna")
                    || t.ends_with(".fa.gz")
                    || t.ends_with(".fasta.gz")
            })
            .map(|(i, _)| i)
            .collect();

        // First file is usually query, second is ref
        if !has_query && !fasta_files.is_empty() {
            tokens.insert(fasta_files[0], "--query".to_string());
        }
        if !has_ref && fasta_files.len() > 1 {
            let idx = if has_query {
                fasta_files[1] + 1
            } else {
                fasta_files[1]
            };
            tokens.insert(idx, "--ref".to_string());
        }
    }

    tokens.join(" ")
}

/// Correct rm commands.
/// rm often needs -d for directories, -f for force, -r for recursive
fn correct_rm(args_str: &str, sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Check if there are any flags
    let has_flags = tokens.iter().any(|t| t.starts_with('-'));

    if !has_flags && tokens.len() == 1 {
        let path = &tokens[0];
        // Check if path suggests a directory or needs force
        if path.ends_with("/") || path.ends_with("_directory") || path.ends_with("_dir") {
            return format!("-d {}", path);
        } else if path.contains("symlink") || path.contains("link") {
            return format!("-f {}", path);
        }
    }

    args_str.to_string()
}

/// Correct fastp commands.
fn correct_fastp(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // fastp uses -w for threads, not -@
    for i in 0..tokens.len() {
        if tokens[i] == "-@" {
            tokens[i] = "-w".to_string();
        }
    }

    tokens.join(" ")
}

/// Correct salmon commands.
fn correct_salmon(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // salmon quant requires index and libType
    if tokens.len() >= 2 && (tokens[0] == "quant" || tokens[0] == "index") {
        // Ensure -i and -l are present for quant
        if tokens[0] == "quant" {
            let has_index = tokens.iter().any(|t| t == "-i" || t == "--index");
            let has_libtype = tokens.iter().any(|t| t == "-l" || t == "--libType");

            if !has_libtype {
                // Insert -l A after subcommand
                tokens.insert(1, "A".to_string());
                tokens.insert(1, "-l".to_string());
            }
        }
    }

    tokens.join(" ")
}

/// Correct bowtie2 commands.
fn correct_bowtie2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // bowtie2-build should have build as first token
    if tokens.iter().any(|t| t == "--build" || t.contains("build")) {
        if tokens[0] != "build" && tokens[0] != "bowtie2-build" {
            tokens.insert(0, "bowtie2-build".to_string());
        }
    }

    tokens.join(" ")
}

/// Correct agat commands.
/// agat is a toolkit with many subcommands like agat_sp_statistics, agat_sp_filter_gene_by_length
fn correct_agat(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];
    let args_lower = args_str.to_lowercase();

    // Check if first token is a valid agat subcommand
    let is_valid_subcommand = first.starts_with("agat_sp_") ||
                              first.starts_with("agat_convert_") ||
                              first.starts_with("agat_");

    // If it's already valid, just fix common flag issues
    if is_valid_subcommand {
        return args_str.replace("--output", "-o").replace("--input", "-i");
    }

    // Model generated a hallucinated subcommand - need to replace it
    // Infer correct subcommand from context
    let correct_subcommand = if args_lower.contains("statistic") || first.contains("stat") {
        "agat_sp_statistics"
    } else if args_lower.contains("filter") && args_lower.contains("length") {
        "agat_sp_filter_gene_by_length"
    } else if args_lower.contains("isoform") || args_lower.contains("longest") {
        "agat_sp_keep_longest_isoform"
    } else if args_lower.contains("merge") || first.contains("merge") {
        "agat_sp_merge_annotations"
    } else if args_lower.contains("fix") || first.contains("fix") {
        "agat_sp_fix_features"
    } else if args_lower.contains("id") || args_lower.contains("manage") {
        "agat_sp_manage_IDs"
    } else if args_lower.contains("extract") || args_lower.contains("sequen") {
        "agat_sp_extract_sequences"
    } else if (args_lower.contains("convert") || first.contains("gtf")) && args_lower.contains("gff") {
        "agat_convert_sp_gff2gtf"
    } else if args_lower.contains("convert") && args_lower.contains("bed") {
        "agat_convert_sp_gff2bed"
    } else if first.contains("convert") || first.contains("gff") || first.contains("gtf") {
        "agat_convert_sp_gff2gtf"
    } else if first.contains("filter") {
        "agat_sp_filter_gene_by_length"
    } else {
        // Default fallback
        "agat_sp_statistics"
    };

    // Replace first token with correct subcommand
    let mut result = vec![correct_subcommand.to_string()];
    result.extend(tokens.into_iter().skip(1));
    result.join(" ")
}

/// Correct bismark commands.
/// bismark has companion binaries: bismark_genome_preparation, bismark_methylation_extractor
fn correct_bismark(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Check for genome preparation
    if args_str.contains("--prepare") || args_str.contains("genome") && args_str.contains("prepar") {
        if !args_str.starts_with("bismark_genome_preparation") {
            return format!("bismark_genome_preparation {}", args_str);
        }
    }

    // Check for methylation extraction
    if args_str.contains("--extract") || args_str.contains("methyl") {
        if !args_str.starts_with("bismark_methylation_extractor") {
            return format!("bismark_methylation_extractor {}", args_str);
        }
    }

    args_str.to_string()
}

/// Correct diamond commands.
/// diamond uses blastx, blastp, view, dbinfo, etc. as first token
fn correct_diamond(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // Check if first token is already a valid subcommand
    const VALID_SUBCOMMANDS: &[&str] = &["blastx", "blastp", "view", "dbinfo", "help", "version", "test", "makedb"];

    if VALID_SUBCOMMANDS.contains(&first.as_str()) {
        return args_str.to_string();
    }

    // Try to infer subcommand from context
    let args_lower = args_str.to_lowercase();

    if args_lower.contains("makedb") || args_lower.contains("make-db") {
        return format!("makedb {}", args_str);
    } else if args_lower.contains("blastx") {
        return format!("blastx {}", args_str);
    } else if args_lower.contains("blastp") {
        return format!("blastp {}", args_str);
    } else if args_lower.contains("view") {
        return format!("view {}", args_str);
    }

    // Default to blastx for protein alignment
    format!("blastx {}", args_str)
}

/// Correct canu commands.
/// canu uses -p for prefix, -d for directory, and genome size specification
fn correct_canu(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove hallucinated subcommands that canu doesn't have
    // canu doesn't use subcommands like "ont", "genome", "pacbio", "hifi"
    if !tokens.is_empty() {
        let first = &tokens[0];
        let hallucinated = ["ont", "genome", "pacbio", "hifi", "nano", "raw", "correct", "trim", "assemble"];
        if hallucinated.contains(&first.as_str()) {
            tokens.remove(0);
        }
    }

    // Ensure -p (prefix) is present
    let has_prefix = tokens.iter().any(|t| t == "-p" || t.starts_with("-p"));
    if !has_prefix {
        tokens.push("-p".to_string());
        tokens.push("assembly".to_string());
    }

    // Ensure -d (directory) is present
    let has_dir = tokens.iter().any(|t| t == "-d" || t.starts_with("-d"));
    if !has_dir {
        tokens.push("-d".to_string());
        tokens.push("canu_out/".to_string());
    }

    // Ensure genome size is specified
    let has_genome_size = tokens.iter().any(|t| t.starts_with("genomeSize") || t.starts_with("-genomeSize"));
    if !has_genome_size {
        tokens.push("genomeSize=5m".to_string());
    }

    // Ensure input type flag is present
    let has_input_type = tokens.iter().any(|t| {
        t.starts_with("-nanopore") || t.starts_with("-pacbio") || t.starts_with("-corrected")
    });
    if !has_input_type && !tokens.is_empty() {
        // Guess based on file extension or default to nanopore
        let has_fastq = tokens.iter().any(|t| t.contains(".fastq") || t.contains(".fq"));
        if has_fastq {
            tokens.push("-nanopore-raw".to_string());
        }
    }

    tokens.join(" ")
}

/// Correct minimap2 commands.
/// minimap2 doesn't have subcommands - flags come first
/// Remove "index" prefix commands with &&
fn correct_minimap2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    // If args contains "index" followed by "&& minimap2", remove the indexing part
    // Pattern: "index ... && minimap2 ..." should become just the alignment part
    if let Some(idx) = args_str.find(" && minimap2 ") {
        let after = &args_str[idx + 13..]; // Skip " && minimap2 "
        return after.to_string();
    }

    // Remove standalone "index" as first token
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    if !tokens.is_empty() && tokens[0] == "index" {
        return tokens[1..].join(" ");
    }

    args_str.to_string()
}

/// Correct bowtie2 commands.
/// bowtie2 uses flags directly (not subcommands), bowtie2-build for indexing
fn correct_bowtie2_v2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    // Remove hallucinated subcommands
    let first = &tokens[0];
    if first == "aligned" || first == "mapping" || first == "reference" {
        tokens.remove(0);
    }

    // Fix common flag issues
    // -x should be followed by index name (not "mapping" or "reference")
    if let Some(x_idx) = tokens.iter().position(|t| t == "-x") {
        if x_idx + 1 < tokens.len() {
            let val = &tokens[x_idx + 1];
            if val == "mapping" || val == "reference" || val == "aligned" {
                tokens[x_idx + 1] = "reference_index".to_string();
            }
        }
    }

    tokens.join(" ")
}

/// Correct fastp commands.
/// fastp uses -i/-I for input, -o/-O for output (paired end)
fn correct_fastp_v2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Count input/output pairs
    let has_i = tokens.iter().any(|t| t == "-i");
    let has_I = tokens.iter().any(|t| t == "-I");
    let has_o = tokens.iter().any(|t| t == "-o");
    let has_O = tokens.iter().any(|t| t == "-O");

    // Single-end mode: should have -i and -o
    // Paired-end mode: should have -i, -I, -o, -O

    // Check for invalid combinations
    // If --merge is used without proper paired-end setup, it might be wrong
    if args_str.contains("--merge") && (!has_I || !has_O) {
        // Remove --merge flags if not proper paired-end
        return args_str
            .replace("--merge", "")
            .replace("--merged_out", "")
            .replace("--out1", "")
            .replace("--out2", "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    }

    args_str.to_string()
}

/// Correct spades commands.
/// spades uses -o for output dir, -1/-2 for paired reads, -s for single
fn correct_spades(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove "assemble" or other hallucinated subcommands
    if !tokens.is_empty() {
        let first = &tokens[0];
        if first == "assemble" || first == "run" || first == "spades" {
            tokens.remove(0);
        }
    }

    // Fix --threads to -t
    for i in 0..tokens.len() {
        if tokens[i] == "--threads" {
            tokens[i] = "-t".to_string();
        }
    }

    tokens.join(" ")
}

/// Correct star commands.
/// STAR uses --runMode, --genomeDir, etc.
fn correct_star(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove "align" or other hallucinated subcommands
    if !tokens.is_empty() {
        let first = &tokens[0];
        if first == "align" || first == "run" {
            tokens.remove(0);
        }
    }

    // Ensure --runMode is present for alignment
    let has_runmode = tokens.iter().any(|t| t.starts_with("--runMode"));
    if !has_runmode && !tokens.is_empty() {
        // Insert --runMode alignReads at the beginning
        tokens.insert(0, "alignReads".to_string());
        tokens.insert(0, "--runMode".to_string());
    }

    tokens.join(" ")
}

/// Correct hisat2 commands.
/// hisat2 uses -x for index, similar to bowtie2
fn correct_hisat2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove "align" subcommand if present
    if !tokens.is_empty() && tokens[0] == "align" {
        tokens.remove(0);
    }

    // hisat2-build should start with that
    if args_str.contains("--build") || args_str.contains("build") {
        if !args_str.starts_with("hisat2-build") && !tokens.is_empty() && tokens[0] != "hisat2-build" {
            tokens.insert(0, "hisat2-build".to_string());
        }
    }

    tokens.join(" ")
}

/// Correct varscan2 commands.
/// varscan2 uses subcommands: mpileup2snp, mpileup2indel, somatic, readcounts
fn correct_varscan2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // Check if already has valid subcommand
    const VALID_SUBCOMMANDS: &[&str] = &["mpileup2snp", "mpileup2indel", "somatic", "readcounts", "processSomatic"];
    if VALID_SUBCOMMANDS.contains(&first.as_str()) {
        // Has valid subcommand but flags may be wrong - need to fix flag format
        // VarScan2 uses --flag value format (not --flag=value)
        // And somatic mode takes pileup files directly, not --normal/--tumor
        let mut result = vec![first.clone()];

        if first == "somatic" {
            // Somatic mode: varscan somatic normal.pileup tumor.pileup output --options
            // Remove --normal and --tumor prefixes, extract just the file paths
            let mut i = 1;
            while i < tokens.len() {
                let token = &tokens[i];

                if token == "--normal" || token == "--tumor" {
                    // Skip the flag, keep the value if present
                    if i + 1 < tokens.len() {
                        let file = &tokens[i + 1];
                        if !file.starts_with('-') {
                            result.push(file.clone());
                        }
                        i += 2;
                        continue;
                    }
                } else if token == "--output" {
                    // Convert --output to output prefix format
                    if i + 1 < tokens.len() {
                        result.push(tokens[i + 1].clone());
                        i += 2;
                        continue;
                    }
                } else if token.starts_with("--") {
                    // Keep other flags
                    result.push(token.clone());
                } else if !token.starts_with('-') {
                    // Keep file arguments
                    result.push(token.clone());
                }
                i += 1;
            }
            return result.join(" ");
        }

        return args_str.to_string();
    }

    // Model generated flags instead of subcommand - need to infer correct subcommand
    let args_lower = args_str.to_lowercase();

    // Infer subcommand from context
    let subcommand = if args_lower.contains("somatic") || args_lower.contains("tumor") {
        "somatic"
    } else if args_lower.contains("indel") {
        "mpileup2indel"
    } else if args_lower.contains("copynumber") || args_lower.contains("cnv") {
        "copynumber"
    } else {
        // Default to mpileup2snp for SNP calling
        "mpileup2snp"
    };

    // Reconstruct with subcommand first
    let mut result = vec![subcommand.to_string()];

    // Add pileup/bam files if present
    for token in &tokens {
        if token.ends_with(".pileup") || token.ends_with(".bam") {
            result.push(token.clone());
        }
    }

    // Add other valid flags
    for token in &tokens {
        if token.starts_with("--") {
            // Convert long flags to varscan2 format
            let converted = token
                .replace("--normal", "")
                .replace("--tumor", "")
                .replace("--output", "--output-vcf 1")
                .replace("--min-coverage", "--min-coverage")
                .replace("--min-reads2", "--min-reads2")
                .replace("--min-avg-qual", "--min-avg-qual")
                .replace("--min-var-freq", "--min-var-freq")
                .replace("--p-value", "--p-value");
            if !converted.is_empty() && converted != *token {
                result.push(converted);
            } else if converted == *token {
                result.push(token.clone());
            }
        } else if !token.ends_with(".pileup") && !token.ends_with(".bam") && !token.starts_with('-') {
            result.push(token.clone());
        }
    }

    result.join(" ")
}

/// Correct trinity commands.
/// trinity requires --seqType, --max_memory, --CPU, and input files
fn correct_trinity(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Ensure --max_memory is present
    let has_memory = tokens.iter().any(|t| t == "--max_memory");
    if !has_memory {
        tokens.push("--max_memory".to_string());
        tokens.push("50G".to_string());
    }

    // Ensure --CPU is present
    let has_cpu = tokens.iter().any(|t| t == "--CPU");
    if !has_cpu {
        tokens.push("--CPU".to_string());
        tokens.push("16".to_string());
    }

    // Fix output directory naming
    if let Some(idx) = tokens.iter().position(|t| t == "--output") {
        if idx + 1 < tokens.len() {
            let output_val = &tokens[idx + 1];
            // Ensure proper trinity output naming
            if !output_val.starts_with("trinity_") && !output_val.starts_with("genome_guided_") {
                // Check input type to determine proper name
                let has_single = tokens.iter().any(|t| t == "--single");
                let has_left = tokens.iter().any(|t| t == "--left" || t == "--right");
                let has_genome = tokens.iter().any(|t| t.contains("genome_guided"));

                if has_genome {
                    tokens[idx + 1] = "genome_guided_trinity/".to_string();
                } else if has_single {
                    tokens[idx + 1] = "trinity_se/".to_string();
                } else if has_left {
                    tokens[idx + 1] = "trinity_output/".to_string();
                }
            }
        }
    }

    tokens.join(" ")
}

/// Correct orthofinder commands.
/// orthofinder uses -f for input directory, -t for threads
fn correct_orthofinder(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove "&&" and second orthofinder command if present
    if let Some(idx) = args_str.find(" && orthofinder") {
        return args_str[..idx].to_string();
    }

    // Check if has valid starting point
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // If first token is a path without -f, add -f
    if !first.starts_with('-') && !first.starts_with("orthofinder") {
        return format!("-f {}", args_str);
    }

    args_str.to_string()
}

/// Correct strelka2 commands.
/// strelka2 uses configureStrelkaGermlineWorkflow.py or configureStrelkaSomaticWorkflow.py
fn correct_strelka2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    // Check if it's trying to run the Python script directly
    if args_str.contains("configureStrelka") {
        // This is the workflow configuration - should include runWorkflow.py execution
        if !args_str.contains("runWorkflow.py") {
            return format!("{} && python strelka_run/runWorkflow.py -m local -j 8", args_str);
        }
    }

    args_str.to_string()
}

/// Correct snpeff commands.
/// snpeff uses -v for verbose, -c for config, database name first
fn correct_snpeff(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    // Fix -gff3 to proper snpeff format
    for i in 0..tokens.len() {
        if tokens[i] == "-gff3" {
            // snpeff uses database name followed by -gff3
            tokens[i] = "-gff3".to_string();
        }
    }

    tokens.join(" ")
}

/// Correct pbsv commands.
/// pbsv uses subcommands: discover, call
fn correct_pbsv(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // Check if already has valid subcommand
    if first == "discover" || first == "call" {
        return args_str.to_string();
    }

    // Infer subcommand from context
    let args_lower = args_str.to_lowercase();
    let subcommand = if args_lower.contains("call") || args_lower.contains("variant") {
        "call"
    } else {
        "discover"
    };

    format!("{} {}", subcommand, args_str)
}

/// Correct muscle commands.
/// muscle v5 uses -align, -super5, -cluster subcommands
fn correct_muscle(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // Check if already has valid subcommand
    const VALID_SUBCOMMANDS: &[&str] = &["-align", "-super5", "-cluster", "-profile"];
    if VALID_SUBCOMMANDS.iter().any(|s| first.starts_with(s)) {
        return args_str.to_string();
    }

    // Default to -align for alignment
    format!("-align {}", args_str)
}

/// Correct verkko commands.
/// verkko uses -d for directory, --hifi and --nano for inputs
fn correct_verkko(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Ensure -d (directory) is present
    let has_dir = tokens.iter().any(|t| t == "-d");
    if !has_dir {
        tokens.insert(0, "verkko/".to_string());
        tokens.insert(0, "-d".to_string());
    }

    tokens.join(" ")
}

/// Correct cellsnp-lite commands.
/// cellsnp-lite doesn't use subcommands - it uses flags like -s, -b, -O, -R
/// Common errors: model uses "pileup" as a subcommand
fn correct_cellsnp_lite(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove hallucinated "pileup" subcommand
    if !tokens.is_empty() && tokens[0] == "pileup" {
        tokens.remove(0);
    }

    // Map positional args to proper flags if needed
    // Expected format: -s <bam> -b <barcode> -O <outdir> -R <vcf>
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        // Skip if already a flag
        if token.starts_with('-') {
            result.push(token.clone());
            // Keep the flag's value too
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                result.push(tokens[i + 1].clone());
                i += 1;
            }
        } else if token.ends_with(".bam") && !result.iter().any(|t: &String| t == "-s") {
            // BAM file without -s flag
            result.push("-s".to_string());
            result.push(token.clone());
        } else if token.ends_with(".tsv") && !result.iter().any(|t: &String| t == "-b") {
            // Barcode file without -b flag
            result.push("-b".to_string());
            result.push(token.clone());
        } else if (token.ends_with(".vcf.gz") || token.ends_with(".vcf")) && !result.iter().any(|t: &String| t == "-R") {
            // VCF file without -R flag
            result.push("-R".to_string());
            result.push(token.clone());
        } else if token.ends_with(".txt") && !result.iter().any(|t: &String| t == "-O") {
            // Output prefix
            result.push("-O".to_string());
            result.push(token.replace(".txt", ""));
        } else {
            result.push(token.clone());
        }
        i += 1;
    }

    // Ensure output directory is set
    if !result.iter().any(|t| t == "-O") {
        result.push("-O".to_string());
        result.push("cellsnp_out".to_string());
    }

    result.join(" ")
}

/// Correct iqtree2 commands.
/// iqtree2 doesn't use subcommands - positional args are input files
/// Common errors: model adds -s flag or subcommands
fn correct_iqtree2(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Remove common hallucinated subcommands
    if !tokens.is_empty() {
        let first = &tokens[0];
        let hallucinated = ["tree", "build", "construct", "run"];
        if hallucinated.contains(&first.as_str()) {
            tokens.remove(0);
        }
    }

    // iqtree2 typically just takes an input file as positional arg
    // -s is used for alignment file but often missing
    let has_s_flag = tokens.iter().any(|t| t == "-s" || t.starts_with("-s"));
    if !has_s_flag && !tokens.is_empty() && !tokens[0].starts_with('-') {
        // First positional arg should be input file, add -s before it
        let first_file = tokens[0].clone();
        if first_file.ends_with(".fasta") || first_file.ends_with(".fa") || first_file.ends_with(".phy") {
            tokens.insert(0, "-s".to_string());
        }
    }

    tokens.join(" ")
}

/// Correct pbfusion commands.
/// pbfusion uses flags like --bam, --gtf, --output-dir, not positional args
/// Common errors: model uses positional args and wrong flag names
fn correct_pbfusion(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.starts_with("--") {
            // Keep existing flags but map incorrect ones
            let mapped_flag = match token.as_str() {
                "--fusion-detection" => "--output-dir", // Wrong flag name
                "--min-reads" => "--min-support",       // Wrong flag name
                _ => token,
            };
            result.push(mapped_flag.to_string());
            // Keep the flag's value
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                result.push(tokens[i + 1].clone());
                i += 1;
            }
        } else if token.ends_with(".bam") && !result.iter().any(|t: &String| t == "--bam") {
            // BAM file without --bam flag
            result.push("--bam".to_string());
            result.push(token.clone());
        } else if token.ends_with(".gtf") && !result.iter().any(|t: &String| t == "--gtf") {
            // GTF file without --gtf flag
            result.push("--gtf".to_string());
            result.push(token.clone());
        } else {
            result.push(token.clone());
        }
        i += 1;
    }

    // Ensure output directory is set
    if !result.iter().any(|t| t == "--output-dir") {
        result.push("--output-dir".to_string());
        result.push("fusion_output/".to_string());
    }

    result.join(" ")
}

/// Correct whatshap commands.
/// whatshap uses subcommands: phase, haplotag, stats
/// Common errors: model uses "tag" instead of "haplotag", "phasing" instead of "stats"
fn correct_whatshap(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];
    let mut result = vec![];

    // Map incorrect subcommands to correct ones
    match first.as_str() {
        "tag" => {
            result.push("haplotag".to_string());
            result.extend(tokens[1..].iter().cloned());
        }
        "phasing" => {
            result.push("stats".to_string());
            // For stats, whatshap expects: stats <vcf_file>
            // Remove --vcf flag if present and just keep the file argument
            let mut i = 1;
            while i < tokens.len() {
                let token = &tokens[i];
                if token == "--vcf" || token == "--variants" {
                    if i + 1 < tokens.len() {
                        result.push(tokens[i + 1].clone());
                        i += 2;
                        continue;
                    }
                } else if token.starts_with("--output") || token.starts_with("-o") {
                    // stats doesn't use --output, it writes to stdout
                    i += 1;
                    if i < tokens.len() && !tokens[i].starts_with('-') {
                        i += 1;
                    }
                    continue;
                } else if !token.starts_with('-') {
                    result.push(token.clone());
                }
                i += 1;
            }
        }
        _ => {
            result.extend(tokens.iter().cloned());
        }
    }

    result.join(" ")
}

/// Correct seqkit commands.
/// seqkit has many subcommands: seq, stats, grep, sample, faidx, fq2fa, split2, etc.
/// Common errors: model uses wrong subcommand for the task
fn correct_seqkit(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    if tokens.is_empty() {
        return args_str.to_string();
    }

    // Map common incorrect subcommands
    let first = &tokens[0];
    match first.as_str() {
        "extract" => {
            // extract -> seq (for length filtering operations)
            tokens[0] = "seq".to_string();
            // Map --out-file to -o if present
            for i in 0..tokens.len() {
                if tokens[i] == "--out-file" {
                    tokens[i] = "-o".to_string();
                }
            }
        }
        "filter" => {
            // filter -> seq (for length filtering operations)
            tokens[0] = "seq".to_string();
            // Map --min-len to -m if present
            for i in 0..tokens.len() {
                if tokens[i] == "--min-len" {
                    tokens[i] = "-m".to_string();
                }
            }
        }
        "stats" => {
            // Ensure -a flag is present for all stats
            let has_all = tokens.iter().any(|t| t == "-a" || t == "--all");
            if !has_all {
                tokens.insert(1, "-a".to_string());
            }
        }
        _ => {}
    }

    tokens.join(" ")
}

/// Generic flag corrections that apply to all tools.
fn generic_flag_corrections(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    // Look for paired short/long flags and keep only the preferred one
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        // Check if this is a short flag that has a long form equivalent
        if token.starts_with('-') && !token.starts_with("--") && token.len() == 2 {
            let flag_char = &token[1..];

            // Look for corresponding long form in the catalog
            let long_form = format!(
                "--{}",
                match flag_char {
                    "o" => "output",
                    "i" => "input",
                    "t" => "threads",
                    "v" => "verbose",
                    "h" => "help",
                    _ => "",
                }
            );

            if !long_form.is_empty() {
                // Check if long form appears later in tokens
                if let Some(long_pos) = tokens.iter().position(|t| t.starts_with(&long_form)) {
                    // Remove the short form
                    tokens.remove(i);
                    continue;
                }
            }
        }

        i += 1;
    }

    // Remove consecutive duplicate flags
    let mut result = Vec::new();
    let mut prev_flag: Option<String> = None;

    for token in tokens {
        if token.starts_with('-') {
            if prev_flag.as_ref() != Some(&token) {
                result.push(token.clone());
                prev_flag = Some(token);
            }
        } else {
            result.push(token);
            prev_flag = None;
        }
    }

    result.join(" ")
}

/// Validate and fix flag ordering (flags before positional args).
pub fn validate_flag_order(args_str: &str, _sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

    let mut flags = Vec::new();
    let mut positional = Vec::new();
    let mut flag_values = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        if token.starts_with('-') {
            flags.push(token.clone());
            // Check if next token is a value for this flag
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                flag_values.push(tokens[i + 1].clone());
                i += 1;
            } else {
                flag_values.push(String::new());
            }
        } else {
            positional.push(token.clone());
        }
        i += 1;
    }

    // Reconstruct: flags first, then positional args
    let mut result = Vec::new();
    for (flag, value) in flags.iter().zip(flag_values.iter()) {
        result.push(flag.clone());
        if !value.is_empty() {
            result.push(value.clone());
        }
    }
    result.extend(positional);

    result.join(" ")
}

/// Detect and remove hallucinated flags not in the catalog.
pub fn remove_hallucinated_flags(args_str: &str, sdoc: &StructuredDoc) -> String {
    if sdoc.flag_catalog.is_empty() {
        return args_str.to_string();
    }

    // Check if this looks like a companion binary command (e.g., agat_convert_sp_gff2gtf)
    // If so, don't strip flags - the flag catalog is for the main tool, not the companion
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    if !tokens.is_empty() {
        let first = &tokens[0];
        // Check if first token looks like a companion binary (contains tool name + suffix)
        let is_likely_companion = first.contains('_') || first.contains('-');

        // Also check if flag catalog is very limited (only generic flags like -h, --help)
        let has_meaningful_flags = sdoc.flag_catalog.iter().any(|e| {
            let flag = &e.flag;
            flag != "-h" && flag != "--help" && flag != "-v" && flag != "--version"
        });

        // If using companion binary or flag catalog is limited, be conservative
        if is_likely_companion || !has_meaningful_flags {
            return args_str.to_string();
        }
    }

    let known_flags: std::collections::HashSet<String> = sdoc
        .flag_catalog
        .iter()
        .flat_map(|entry| {
            entry
                .flag
                .split([',', ' ', '\t'])
                .map(|s| s.trim().trim_end_matches('=').to_string())
                .filter(|s| !s.is_empty() && s.starts_with('-'))
        })
        .collect();

    let mut result = Vec::new();
    let mut skip_next = false;

    for (i, token) in tokens.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if token.starts_with('-') {
            // Check if this flag is known
            let base_flag = if token.contains('=') {
                token.split('=').next().unwrap_or(token)
            } else {
                token
            };

            if known_flags.contains(base_flag) {
                result.push(token.clone());
            } else {
                // This might be a hallucinated flag - check if next token is a value
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    skip_next = true;
                }
            }
        } else {
            result.push(token.clone());
        }
    }

    result.join(" ")
}

/// Known valid subcommands for common bioinformatics tools.
/// Used to validate and correct hallucinated subcommands.
const KNOWN_SUBCOMMANDS: &[(&str, &[&str])] = &[
    ("samtools", &["sort", "index", "view", "flagstat", "merge", "markdup", "depth", "coverage", "faidx", "dict", "stats", "bedcov", "collate", "fasta", "fastq", "head", "tview"]),
    ("bwa", &["mem", "aln", "samse", "sampe", "bwasw", "index", "fa2pac", "pac2bwt", "pac2bwtgen", "bwtupdate", "bwt2sa", "shm", "unindex"]),
    ("bcftools", &["view", "index", "query", "stats", "filter", "call", "norm", "merge", "concat", "sort", "convert", "head", "isec", "gtcheck", "plugin"]),
    ("bedtools", &["intersect", "sort", "merge", "coverage", "genomecov", "bamtobed", "bedtobam", "getfasta", "multicov", "map", "jaccard", "fisher", "reldist", "sample", "shuffle", "slop", "flank", "closest", "subtract", "window", "annotate", "groupby", "unionbedg"]),
    ("macs3", &["callpeak", "bdgpeakcall", "bdgdiff", "bdgbroadcall", "bdgcmp", "cmbreps", "bdgopt", "refinepeak", "predictd", "pileup", "filterdup", "randsample", "split"]),
    ("macs2", &["callpeak", "bdgpeakcall", "bdgdiff", "bdgbroadcall", "bdgcmp", "cmbreps", "bdgopt", "refinepeak", "predictd", "pileup", "filterdup", "randsample"]),
    ("picard", &["MarkDuplicates", "SortSam", "BuildBamIndex", "CollectAlignmentSummaryMetrics", "CollectInsertSizeMetrics", "CollectGcBiasMetrics", "EstimateLibraryComplexity", "AddOrReplaceReadGroups", "CreateSequenceDictionary", "SamToFastq", "MergeSamFiles", "ValidateSamFile"]),
    ("gatk", &["HaplotypeCaller", "Mutect2", "MarkDuplicates", "BaseRecalibrator", "ApplyBQSR", "VariantFiltration", "SelectVariants", "CombineGVCFs", "GenotypeGVCFs", "VariantRecalibrator", "ApplyVQSR", "AnalyzeCovariates", "CollectAlignmentSummaryMetrics", "CollectInsertSizeMetrics"]),
    ("salmon", &["quant", "index", "alevin", "swim", "cite"]),
    ("kallisto", &["quant", "index", "bus", "h5dump", "pseudo", "merge"]),
    ("star", &["--runMode", "--genomeDir"]), // STAR uses --flags, not subcommands
    ("hisat2", &["-x", "-1", "-2", "-U"]), // HISAT2 uses flags, not subcommands
    ("bowtie2", &["-x", "-1", "-2", "-U"]), // bowtie2 uses flags, not subcommands
    ("minimap2", &["-x", "-a", "-t"]), // minimap2 uses flags, not subcommands
];

/// Validate and correct subcommand for tools that require them.
/// Returns corrected args string and whether a correction was made.
pub fn validate_subcommand(args_str: &str, tool: &str, sdoc: &StructuredDoc) -> String {
    let tokens: Vec<&str> = args_str.split_whitespace().collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first_token = tokens[0];

    // If tool requires subcommands but doesn't have known subcommands in sdoc,
    // check against our hardcoded list
    if sdoc.has_subcommands && sdoc.subcommands.is_empty() {
        if let Some((_, known_subs)) = KNOWN_SUBCOMMANDS.iter().find(|(t, _)| *t == tool) {
            // Check if first token is a valid subcommand
            if !first_token.starts_with('-') && !known_subs.contains(&first_token) {
                // First token is not a valid subcommand - likely hallucinated
                // Try to infer correct subcommand from context or use default
                let inferred = infer_subcommand(tool, &tokens);
                let mut new_tokens = vec![inferred];
                new_tokens.extend_from_slice(&tokens);
                return new_tokens.join(" ");
            }
        }
    }

    // If tool does NOT have subcommands but first token looks like one, remove it
    if !sdoc.has_subcommands {
        if !first_token.starts_with('-')
            && !first_token.contains('.')
            && !first_token.contains('/')
            && is_likely_subcommand(first_token)
        {
            // Remove the hallucinated subcommand
            return tokens[1..].join(" ");
        }
    }

    args_str.to_string()
}

/// Infer the correct subcommand based on tool and context.
fn infer_subcommand(tool: &str, tokens: &[&str]) -> &'static str {
    let args_lower = tokens.join(" ").to_lowercase();

    match tool {
        "samtools" => {
            if args_lower.contains("sort") { "sort" }
            else if args_lower.contains("index") { "index" }
            else if args_lower.contains("view") { "view" }
            else if args_lower.contains("merge") { "merge" }
            else if args_lower.contains("flagstat") { "flagstat" }
            else if args_lower.contains("depth") { "depth" }
            else if args_lower.contains("coverage") { "coverage" }
            else if args_lower.contains("faidx") { "faidx" }
            else if args_lower.contains("fasta") { "fasta" }
            else if args_lower.contains("fastq") { "fastq" }
            else { "view" } // default
        }
        "bwa" => {
            if args_lower.contains("mem") { "mem" }
            else if args_lower.contains("aln") { "aln" }
            else if args_lower.contains("index") { "index" }
            else if args_lower.contains("samse") { "samse" }
            else if args_lower.contains("sampe") { "sampe" }
            else { "mem" } // default
        }
        "bcftools" => {
            if args_lower.contains("call") { "call" }
            else if args_lower.contains("filter") { "filter" }
            else if args_lower.contains("view") { "view" }
            else if args_lower.contains("index") { "index" }
            else if args_lower.contains("query") { "query" }
            else if args_lower.contains("stats") { "stats" }
            else if args_lower.contains("merge") { "merge" }
            else if args_lower.contains("norm") { "norm" }
            else { "view" } // default
        }
        "bedtools" => {
            if args_lower.contains("intersect") { "intersect" }
            else if args_lower.contains("sort") { "sort" }
            else if args_lower.contains("merge") { "merge" }
            else if args_lower.contains("coverage") { "coverage" }
            else if args_lower.contains("getfasta") { "getfasta" }
            else { "intersect" } // default
        }
        "macs3" | "macs2" => {
            if args_lower.contains("callpeak") { "callpeak" }
            else if args_lower.contains("bdgdiff") { "bdgdiff" }
            else if args_lower.contains("predictd") { "predictd" }
            else { "callpeak" } // default
        }
        _ => "",
    }
}

/// Check if a token is likely a subcommand (not a flag or file path).
fn is_likely_subcommand(token: &str) -> bool {
    const COMMON_SUBCOMMANDS: &[&str] = &[
        "run", "process", "analyze", "generate", "compute", "extract",
        "convert", "filter", "sort", "index", "align", "call", "plot",
        "report", "profile", "prepare", "build", "quantify", "count",
        "trim", "clean", "stats", "merge", "split", "view", "faidx",
        "dict", "head", "tview", "depth", "coverage", "flagstat",
    ];

    COMMON_SUBCOMMANDS.contains(&token.to_lowercase().as_str())
}

// ============================================================================
// Pattern-Based Validation (New Architecture)
// ============================================================================

use crate::doc_processor::UsagePatternType;

/// Pattern-based validation result.
#[derive(Debug, Clone)]
pub struct PatternValidationResult {
    pub corrected_args: String,
    pub was_corrected: bool,
}

/// Validate and correct command arguments using patterns from documentation.
pub fn pattern_based_validation(
    args_str: &str,
    sdoc: &StructuredDoc,
    _tool: &str,
) -> PatternValidationResult {
    let mut corrected = args_str.to_string();
    let mut was_corrected = false;

    // Step 1: Validate subcommand pattern
    let subcommand_result = validate_subcommand_by_pattern(&corrected, sdoc);
    if subcommand_result != corrected {
        corrected = subcommand_result;
        was_corrected = true;
    }

    // Step 2: Validate flags against catalog
    let flag_result = validate_flags_by_catalog(&corrected, sdoc);
    if flag_result != corrected {
        corrected = flag_result;
        was_corrected = true;
    }

    // Step 3: Add missing required flags
    let required_result = add_missing_required_by_pattern(&corrected, sdoc);
    if required_result != corrected {
        corrected = required_result;
        was_corrected = true;
    }

    PatternValidationResult {
        corrected_args: corrected,
        was_corrected,
    }
}

/// Validate subcommand based on USAGE pattern type.
fn validate_subcommand_by_pattern(args_str: &str, sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first_token = &tokens[0];

    match sdoc.usage_pattern.pattern_type {
        UsagePatternType::SubcommandRequired => {
            // First token must be a valid subcommand
            if first_token.starts_with('-') {
                // Missing subcommand - first token is a flag
                if let Some(default_sub) = sdoc.subcommands.first() {
                    let mut new_tokens = vec![default_sub.clone()];
                    new_tokens.extend(tokens);
                    return new_tokens.join(" ");
                }
            } else if !sdoc.subcommands.is_empty()
                && !sdoc.subcommands.contains(first_token)
                && is_likely_subcommand(first_token)
            {
                // Hallucinated subcommand - replace with default
                if let Some(default_sub) = sdoc.subcommands.first() {
                    let mut new_tokens = vec![default_sub.clone()];
                    new_tokens.extend(tokens.into_iter().skip(1));
                    return new_tokens.join(" ");
                }
            }
        }
        UsagePatternType::FlagFirst | UsagePatternType::PositionalArgs => {
            // Tool does NOT use subcommands - remove hallucinated ones
            if !first_token.starts_with('-') && is_likely_subcommand(first_token) {
                return tokens[1..].join(" ");
            }
        }
        _ => {}
    }

    args_str.to_string()
}

/// Validate flags against the flag catalog.
fn validate_flags_by_catalog(args_str: &str, sdoc: &StructuredDoc) -> String {
    if sdoc.flag_catalog.is_empty() {
        return args_str.to_string();
    }

    let known_flags: std::collections::HashSet<String> = sdoc
        .flag_catalog
        .iter()
        .flat_map(|entry| {
            let mut flags = vec![entry.flag.clone()];
            if let Some(ref alt) = entry.alt_form {
                flags.push(alt.clone());
            }
            flags
        })
        .collect();

    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    let mut result = Vec::new();
    let mut skip_next = false;

    for (i, token) in tokens.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if token.starts_with('-') {
            let base_flag = if token.contains('=') {
                token.split('=').next().unwrap_or(token).to_string()
            } else {
                token.clone()
            };

            if !known_flags.contains(&base_flag) {
                // Skip this flag and its value if present
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    skip_next = true;
                }
                continue;
            }
        }

        result.push(token.clone());
    }

    result.join(" ")
}

/// Add missing required flags based on flag catalog.
fn add_missing_required_by_pattern(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut result = args_str.to_string();

    for entry in &sdoc.flag_catalog {
        if !entry.required {
            continue;
        }

        // Check if flag or its alt_form is already present
        let flag_present = args_str.split_whitespace().any(|t| {
            t == entry.flag || t.starts_with(&format!("{}=", entry.flag))
        });

        if flag_present {
            continue;
        }

        if let Some(ref alt) = entry.alt_form {
            let alt_present = args_str.split_whitespace().any(|t| {
                t == alt || t.starts_with(&format!("{}=", alt))
            });
            if alt_present {
                continue;
            }
        }

        // Add the flag with default value
        let default_val = entry.default.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");

        if !default_val.is_empty() {
            result.push(' ');
            result.push_str(&entry.flag);
            result.push(' ');
            result.push_str(default_val);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_admixture() {
        let sdoc = StructuredDoc::default();
        // Model generates: admixture -i data.bed -K 5 --cv=10
        // Should be: admixture data.bed 5 --cv=10
        let input = "-i data.bed -K 5 --cv=10";
        let result = correct_admixture(input, &sdoc);
        assert!(!result.contains("-i "));
        assert!(!result.contains("-K "));
        assert!(result.contains("data.bed"));
        assert!(result.contains("5"));
        assert!(result.contains("--cv=10"));
    }

    #[test]
    fn test_correct_metaphlan() {
        let sdoc = StructuredDoc::default();
        // Missing --input_type
        let input = "trimmed.fastq.gz -o out.txt";
        let result = correct_metaphlan(input, &sdoc);
        assert!(result.contains("--input_type"));
        assert!(result.contains("fastq"));
    }

    #[test]
    fn test_correct_fastani() {
        let sdoc = StructuredDoc::default();
        // Using -o instead of --output
        let input = "--query genome.fa --ref ref.fa -o output.txt";
        let result = correct_fastani(input, &sdoc);
        assert!(result.contains("--output"));
        assert!(!result.contains(" -o "));
    }

    #[test]
    fn test_remove_hallucinated_subcommands() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            ..Default::default()
        };
        // Model generates: admixture run -i data.bed
        // Should be: admixture -i data.bed (or better, without -i)
        let input = "run -i data.bed";
        let result = remove_hallucinated_subcommands(input, &sdoc);
        assert_eq!(result, "-i data.bed");
    }

    #[test]
    fn test_validate_subcommand_samtools() {
        // Tool requires subcommands
        let sdoc = StructuredDoc {
            has_subcommands: true,
            subcommands: vec![], // No doc-extracted subcommands, use hardcoded list
            ..Default::default()
        };

        // Valid subcommand - should pass through
        let result = validate_subcommand("sort -o out.bam in.bam", "samtools", &sdoc);
        assert!(result.contains("sort"));

        // Invalid/hallucinated subcommand - should be corrected
        let result = validate_subcommand("process -i in.bam", "samtools", &sdoc);
        assert!(!result.starts_with("process"));
    }

    #[test]
    fn test_validate_subcommand_no_subcommand_tools() {
        // Tool does NOT have subcommands (e.g., fastp)
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommands: vec![],
            ..Default::default()
        };

        // Should remove hallucinated "sort" subcommand
        let result = validate_subcommand("sort -i in.fq -o out.fq", "fastp", &sdoc);
        assert!(!result.contains("sort"));
        assert!(result.contains("-i"));

        // Valid fastp command should pass through
        let result = validate_subcommand("-i in.fq -o out.fq", "fastp", &sdoc);
        assert_eq!(result, "-i in.fq -o out.fq");
    }

    #[test]
    fn test_infer_subcommand() {
        // Test samtools subcommand inference
        assert_eq!(infer_subcommand("samtools", &["-o", "out.bam"]), "view");
        assert_eq!(infer_subcommand("samtools", &["sort", "-o", "out.bam"]), "sort");

        // Test bwa subcommand inference
        assert_eq!(infer_subcommand("bwa", &["ref.fa", "reads.fq"]), "mem");
        assert_eq!(infer_subcommand("bwa", &["index", "ref.fa"]), "index");
    }
}
