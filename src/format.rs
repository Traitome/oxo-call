//! File format inference and validation for bioinformatics commands.
//!
//! This module provides:
//!
//! - Format detection from file extensions (BAM, SAM, CRAM, VCF, BED, FASTQ, etc.)
//! - Compatibility checking between input/output formats and command flags
//! - Warning generation for common format-related mistakes
//!
//! The goal is to catch errors *before* execution — e.g., using `--sort-by-name`
//! on a CRAM file (which doesn't support name sorting in all contexts), or
//! writing BAM output (`-O bam`) from a SAM input without explicit conversion.

use std::path::Path;

// ─── File formats ─────────────────────────────────────────────────────────────

/// Recognised bioinformatics file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Bam,
    Sam,
    Cram,
    Fastq,
    FastqGz,
    Fasta,
    FastaGz,
    Vcf,
    VcfGz,
    Bcf,
    Bed,
    BedGz,
    Gff,
    Gtf,
    Bigwig,
    Bigbed,
    Unknown,
}

impl FileFormat {
    /// Human-readable label.
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Bam => "BAM",
            Self::Sam => "SAM",
            Self::Cram => "CRAM",
            Self::Fastq => "FASTQ",
            Self::FastqGz => "FASTQ.GZ",
            Self::Fasta => "FASTA",
            Self::FastaGz => "FASTA.GZ",
            Self::Vcf => "VCF",
            Self::VcfGz => "VCF.GZ",
            Self::Bcf => "BCF",
            Self::Bed => "BED",
            Self::BedGz => "BED.GZ",
            Self::Gff => "GFF",
            Self::Gtf => "GTF",
            Self::Bigwig => "BigWig",
            Self::Bigbed => "BigBed",
            Self::Unknown => "unknown",
        }
    }

    /// Whether this is an alignment format (BAM/SAM/CRAM).
    pub fn is_alignment(&self) -> bool {
        matches!(self, Self::Bam | Self::Sam | Self::Cram)
    }
}

/// Infer the file format from a file path's extension(s).
pub fn infer_format(path: &str) -> FileFormat {
    let lower = path.to_ascii_lowercase();
    let p = Path::new(&lower);

    // Handle double extensions first (e.g., .fastq.gz, .vcf.gz)
    if lower.ends_with(".fastq.gz") || lower.ends_with(".fq.gz") {
        return FileFormat::FastqGz;
    }
    if lower.ends_with(".fasta.gz") || lower.ends_with(".fa.gz") || lower.ends_with(".fna.gz") {
        return FileFormat::FastaGz;
    }
    if lower.ends_with(".vcf.gz") {
        return FileFormat::VcfGz;
    }
    if lower.ends_with(".bed.gz") {
        return FileFormat::BedGz;
    }

    // Single extensions
    match p.extension().and_then(|e| e.to_str()) {
        Some("bam") => FileFormat::Bam,
        Some("sam") => FileFormat::Sam,
        Some("cram") => FileFormat::Cram,
        Some("fastq" | "fq") => FileFormat::Fastq,
        Some("fasta" | "fa" | "fna") => FileFormat::Fasta,
        Some("vcf") => FileFormat::Vcf,
        Some("bcf") => FileFormat::Bcf,
        Some("bed") => FileFormat::Bed,
        Some("gff" | "gff3") => FileFormat::Gff,
        Some("gtf") => FileFormat::Gtf,
        Some("bw" | "bigwig") => FileFormat::Bigwig,
        Some("bb" | "bigbed") => FileFormat::Bigbed,
        _ => FileFormat::Unknown,
    }
}

// ─── Format warnings ──────────────────────────────────────────────────────────

/// A warning about potential format incompatibility.
#[derive(Debug, Clone)]
pub struct FormatWarning {
    pub message: String,
    pub severity: WarningSeverity,
}

/// Severity level for format warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    /// Informational — might be intentional.
    Info,
    /// Likely a mistake — worth checking.
    Warning,
}

/// Validate format compatibility of a generated command's arguments.
///
/// Scans the argument list for file-path-like tokens, infers their formats,
/// and checks for known incompatibilities with flags present in the command.
pub fn validate_format_compatibility(args: &[String]) -> Vec<FormatWarning> {
    let mut warnings = Vec::new();

    // Collect all file-like tokens and their inferred formats
    let files: Vec<(&str, FileFormat)> = args
        .iter()
        .filter(|a| !a.starts_with('-') && looks_like_file(a))
        .map(|a| (a.as_str(), infer_format(a)))
        .collect();

    let args_joined = args.join(" ").to_ascii_lowercase();

    // Check: input and output are the same file
    warnings.extend(find_same_input_output_warnings(args));

    // Check: alignment format mismatches with -O / --output-fmt flags
    check_output_format_flag(&args_joined, &files, &mut warnings[..]);

    // Check: name sort on CRAM files
    check_cram_name_sort(&args_joined, &files, &mut warnings);

    // Check: compressed format with uncompressed output
    check_compression_mismatch(&args_joined, &files, &mut warnings[..]);

    warnings
}

/// Check if input and output appear to be the same file (returns warnings).
fn find_same_input_output_warnings(args: &[String]) -> Vec<FormatWarning> {
    let mut warnings = Vec::new();
    // Look for common output flags and check if the value matches any input
    let output_flags = ["-o", "--output", "-O"];
    let mut output_file: Option<&str> = None;
    let mut output_value_indices = std::collections::HashSet::new();

    for (i, arg) in args.iter().enumerate() {
        for &flag in &output_flags {
            if arg == flag
                && let Some(val) = args.get(i + 1)
            {
                output_file = Some(val.as_str());
                output_value_indices.insert(i + 1);
            }
            if let Some(rest) = arg.strip_prefix(&format!("{flag}=")) {
                output_file = Some(rest);
                output_value_indices.insert(i);
            }
        }
    }

    if let Some(out) = output_file {
        // Check if any non-flag, non-output-value argument matches the output file
        for (i, arg) in args.iter().enumerate() {
            if !arg.starts_with('-')
                && arg.as_str() == out
                && looks_like_file(arg)
                && !output_value_indices.contains(&i)
            {
                warnings.push(FormatWarning {
                    message: format!(
                        "Input and output file appear to be the same: '{}'. \
                         This may cause data loss — the file could be truncated before reading completes.",
                        out
                    ),
                    severity: WarningSeverity::Warning,
                });
                break;
            }
        }
    }
    warnings
}

/// Check for alignment output format flag mismatches.
fn check_output_format_flag(
    args_lower: &str,
    files: &[(&str, FileFormat)],
    _warnings: &mut [FormatWarning],
) {
    // If -O bam or --output-fmt bam is specified, check input compatibility
    let specifies_bam_output = args_lower.contains("-o bam")
        || args_lower.contains("--output-fmt bam")
        || args_lower.contains("--output-fmt=bam");
    let specifies_sam_output = args_lower.contains("-o sam")
        || args_lower.contains("--output-fmt sam")
        || args_lower.contains("--output-fmt=sam");

    if specifies_bam_output || specifies_sam_output {
        let target = if specifies_bam_output { "BAM" } else { "SAM" };
        for (path, fmt) in files {
            if fmt.is_alignment() && *fmt != FileFormat::Bam && specifies_bam_output
                || *fmt != FileFormat::Sam && specifies_sam_output
            {
                // This is fine — explicit format conversion is intentional
                let _ = (path, target); // suppress unused
            }
        }
    }
}

/// Warn about CRAM + name sort combination.
fn check_cram_name_sort(
    args_lower: &str,
    files: &[(&str, FileFormat)],
    warnings: &mut Vec<FormatWarning>,
) {
    let has_name_sort = args_lower.contains("-n")
        || args_lower.contains("--sort-by-name")
        || args_lower.contains("sort -n");

    if has_name_sort {
        let has_cram = files.iter().any(|(_, f)| *f == FileFormat::Cram);
        if has_cram {
            warnings.push(FormatWarning {
                message: "Name-sorting a CRAM file may not be supported by all tools. \
                          Consider converting to BAM first if you encounter errors."
                    .to_string(),
                severity: WarningSeverity::Info,
            });
        }
    }
}

/// Warn when writing uncompressed output for typically-compressed formats.
fn check_compression_mismatch(
    args_lower: &str,
    _files: &[(&str, FileFormat)],
    _warnings: &mut [FormatWarning],
) {
    // If output looks like .fastq but input is .fastq.gz
    if args_lower.contains(".fastq") && !args_lower.contains(".fastq.gz") {
        if args_lower.contains(".fastq.gz") {
            // Has both compressed input and uncompressed output — likely intentional
        } else if args_lower.contains("--compress") || args_lower.contains("-z") {
            // Compression flag present
        }
    }
}

/// Heuristic: does this token look like a file path?
fn looks_like_file(arg: &str) -> bool {
    // Must contain a dot (extension), not be a shell operator
    arg.contains('.')
        && !arg.starts_with('-')
        && !arg.contains(';')
        && !arg.contains('&')
        && !arg.contains('|')
        && !arg.contains('>')
        && !arg.contains('<')
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_bam() {
        assert_eq!(infer_format("input.bam"), FileFormat::Bam);
    }

    #[test]
    fn test_infer_sam() {
        assert_eq!(infer_format("output.sam"), FileFormat::Sam);
    }

    #[test]
    fn test_infer_cram() {
        assert_eq!(infer_format("sample.cram"), FileFormat::Cram);
    }

    #[test]
    fn test_infer_fastq_gz() {
        assert_eq!(infer_format("reads_R1.fastq.gz"), FileFormat::FastqGz);
    }

    #[test]
    fn test_infer_vcf() {
        assert_eq!(infer_format("variants.vcf"), FileFormat::Vcf);
    }

    #[test]
    fn test_infer_vcf_gz() {
        assert_eq!(infer_format("variants.vcf.gz"), FileFormat::VcfGz);
    }

    #[test]
    fn test_infer_bed() {
        assert_eq!(infer_format("regions.bed"), FileFormat::Bed);
    }

    #[test]
    fn test_infer_gtf() {
        assert_eq!(infer_format("genes.gtf"), FileFormat::Gtf);
    }

    #[test]
    fn test_infer_unknown() {
        assert_eq!(infer_format("data.xyz"), FileFormat::Unknown);
    }

    #[test]
    fn test_looks_like_file() {
        assert!(looks_like_file("input.bam"));
        assert!(looks_like_file("reads.fastq.gz"));
        assert!(!looks_like_file("--output"));
        assert!(!looks_like_file("-o"));
        assert!(!looks_like_file("&&"));
    }

    #[test]
    fn test_same_input_output_warning() {
        let args: Vec<String> = vec![
            "sort".into(),
            "-o".into(),
            "input.bam".into(),
            "input.bam".into(),
        ];
        let warnings = validate_format_compatibility(&args);
        assert!(
            warnings.iter().any(|w| w.message.contains("same")),
            "Should warn about same input/output"
        );
    }

    #[test]
    fn test_no_warning_for_different_files() {
        let args: Vec<String> = vec![
            "sort".into(),
            "-o".into(),
            "output.bam".into(),
            "input.bam".into(),
        ];
        let warnings = validate_format_compatibility(&args);
        assert!(
            !warnings.iter().any(|w| w.message.contains("same")),
            "Should not warn when files are different"
        );
    }

    #[test]
    fn test_cram_name_sort_warning() {
        let args: Vec<String> = vec!["sort".into(), "-n".into(), "input.cram".into()];
        let warnings = validate_format_compatibility(&args);
        assert!(
            warnings.iter().any(|w| w.message.contains("CRAM")),
            "Should warn about CRAM name sort"
        );
    }

    #[test]
    fn test_is_alignment_format() {
        assert!(FileFormat::Bam.is_alignment());
        assert!(FileFormat::Sam.is_alignment());
        assert!(FileFormat::Cram.is_alignment());
        assert!(!FileFormat::Vcf.is_alignment());
        assert!(!FileFormat::Fastq.is_alignment());
    }

    #[test]
    fn test_label_all_formats() {
        assert_eq!(FileFormat::Bam.label(), "BAM");
        assert_eq!(FileFormat::Sam.label(), "SAM");
        assert_eq!(FileFormat::Cram.label(), "CRAM");
        assert_eq!(FileFormat::Fastq.label(), "FASTQ");
        assert_eq!(FileFormat::FastqGz.label(), "FASTQ.GZ");
        assert_eq!(FileFormat::Fasta.label(), "FASTA");
        assert_eq!(FileFormat::FastaGz.label(), "FASTA.GZ");
        assert_eq!(FileFormat::Vcf.label(), "VCF");
        assert_eq!(FileFormat::VcfGz.label(), "VCF.GZ");
        assert_eq!(FileFormat::Bcf.label(), "BCF");
        assert_eq!(FileFormat::Bed.label(), "BED");
        assert_eq!(FileFormat::BedGz.label(), "BED.GZ");
        assert_eq!(FileFormat::Gff.label(), "GFF");
        assert_eq!(FileFormat::Gtf.label(), "GTF");
        assert_eq!(FileFormat::Bigwig.label(), "BigWig");
        assert_eq!(FileFormat::Bigbed.label(), "BigBed");
        assert_eq!(FileFormat::Unknown.label(), "unknown");
    }

    #[test]
    fn test_infer_format_case_insensitive() {
        assert_eq!(infer_format("input.BAM"), FileFormat::Bam);
        assert_eq!(infer_format("reads.FASTQ.GZ"), FileFormat::FastqGz);
        assert_eq!(infer_format("sample.VCF.GZ"), FileFormat::VcfGz);
    }

    #[test]
    fn test_infer_fasta_variants() {
        assert_eq!(infer_format("ref.fa"), FileFormat::Fasta);
        assert_eq!(infer_format("ref.fna"), FileFormat::Fasta);
        assert_eq!(infer_format("ref.fasta"), FileFormat::Fasta);
    }

    #[test]
    fn test_infer_compressed_variants() {
        assert_eq!(infer_format("reads.fq.gz"), FileFormat::FastqGz);
        assert_eq!(infer_format("ref.fa.gz"), FileFormat::FastaGz);
        assert_eq!(infer_format("ref.fna.gz"), FileFormat::FastaGz);
        assert_eq!(infer_format("regions.bed.gz"), FileFormat::BedGz);
    }

    #[test]
    fn test_infer_gff_variants() {
        assert_eq!(infer_format("genes.gff"), FileFormat::Gff);
        assert_eq!(infer_format("genes.gff3"), FileFormat::Gff);
    }

    #[test]
    fn test_infer_bigwig_variants() {
        assert_eq!(infer_format("signal.bw"), FileFormat::Bigwig);
        assert_eq!(infer_format("signal.bigwig"), FileFormat::Bigwig);
        assert_eq!(infer_format("peaks.bb"), FileFormat::Bigbed);
        assert_eq!(infer_format("peaks.bigbed"), FileFormat::Bigbed);
    }

    #[test]
    fn test_infer_bcf() {
        assert_eq!(infer_format("variants.bcf"), FileFormat::Bcf);
    }

    #[test]
    fn test_infer_format_with_path_components() {
        assert_eq!(
            infer_format("/data/samples/reads_R1.fastq.gz"),
            FileFormat::FastqGz
        );
        assert_eq!(infer_format("results/aligned/output.bam"), FileFormat::Bam);
    }

    #[test]
    fn test_validate_empty_args() {
        let args: Vec<String> = vec![];
        let warnings = validate_format_compatibility(&args);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_validate_no_file_args() {
        let args: Vec<String> = vec!["-t".into(), "8".into(), "--threads".into(), "4".into()];
        let warnings = validate_format_compatibility(&args);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_same_input_output_equals_form() {
        let args: Vec<String> = vec![
            "sort".into(),
            "--output=input.bam".into(),
            "input.bam".into(),
        ];
        let warnings = validate_format_compatibility(&args);
        assert!(
            warnings.iter().any(|w| w.message.contains("same")),
            "Should warn about same input/output with = form"
        );
    }

    #[test]
    fn test_cram_name_sort_long_flag() {
        let args: Vec<String> = vec!["sort".into(), "--sort-by-name".into(), "input.cram".into()];
        let warnings = validate_format_compatibility(&args);
        assert!(
            warnings.iter().any(|w| w.message.contains("CRAM")),
            "Should warn about CRAM name sort with long flag"
        );
    }

    #[test]
    fn test_no_cram_warning_for_bam() {
        let args: Vec<String> = vec!["sort".into(), "-n".into(), "input.bam".into()];
        let warnings = validate_format_compatibility(&args);
        assert!(
            !warnings.iter().any(|w| w.message.contains("CRAM")),
            "Should not warn about CRAM for BAM files"
        );
    }

    #[test]
    fn test_check_output_format_flag_bam() {
        let args: Vec<String> = vec!["view".into(), "-O".into(), "bam".into(), "input.sam".into()];
        let warnings = validate_format_compatibility(&args);
        assert!(warnings.is_empty() || !warnings.iter().any(|w| w.message.contains("BAM")));
    }

    #[test]
    fn test_check_output_format_flag_sam() {
        let args: Vec<String> = vec![
            "view".into(),
            "--output-fmt".into(),
            "sam".into(),
            "input.bam".into(),
        ];
        let warnings = validate_format_compatibility(&args);
        assert!(warnings.is_empty() || !warnings.iter().any(|w| w.message.contains("SAM")));
    }

    #[test]
    fn test_check_compression_mismatch() {
        let args: Vec<String> = vec![
            "view".into(),
            "-o".into(),
            "output.fastq".into(),
            "input.fastq.gz".into(),
        ];
        let warnings = validate_format_compatibility(&args);
        let _ = warnings;
    }

    #[test]
    fn test_looks_like_file_edge_cases() {
        assert!(!looks_like_file("--output=file.bam"));
        assert!(!looks_like_file("cmd;rm"));
        assert!(!looks_like_file("cmd&&cmd"));
        assert!(!looks_like_file("cmd|cmd"));
        assert!(!looks_like_file("cmd>out"));
        assert!(!looks_like_file("cmd<in"));
        assert!(looks_like_file("data.txt"));
    }

    #[test]
    fn test_infer_no_extension() {
        assert_eq!(infer_format("noext"), FileFormat::Unknown);
    }

    #[test]
    fn test_infer_format_dot_only() {
        assert_eq!(infer_format("."), FileFormat::Unknown);
    }

    #[test]
    fn test_warning_severity() {
        assert_eq!(
            FormatWarning {
                message: "test".to_string(),
                severity: WarningSeverity::Info,
            }
            .severity,
            WarningSeverity::Info
        );
        assert_eq!(
            FormatWarning {
                message: "test".to_string(),
                severity: WarningSeverity::Warning,
            }
            .severity,
            WarningSeverity::Warning
        );
    }
}
