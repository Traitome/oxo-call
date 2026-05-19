use crate::doc_processor::{FlagEntry, StructuredDoc};
use super::task_values::{TaskValues, extract_task_values, is_no_subcommand_tool};

pub fn apply_corrections_to_args(
    args: &[String],
    tool: &str,
    structured_doc: Option<&StructuredDoc>,
    task: Option<&str>,
) -> Vec<String> {
    if let Some(sdoc) = structured_doc {
        let mut args = args.to_vec();

        if args.is_empty() {
            if let Some(task) = task {
                let task_values = extract_task_values(task);
                args = super::rule_engine::assemble_command_from_rules(tool, task, sdoc, None, &task_values);
            }
            if args.is_empty() {
                return args;
            }
        }

        args = clean_help_text_in_args(&args);

        if !sdoc.flag_catalog.is_empty() {
            args = validate_flags_against_catalog(&args, &sdoc.flag_catalog, &sdoc.quick_flags);
        }

        if !sdoc.flag_catalog.is_empty() && !sdoc.usage_pattern.positional_args.is_empty() {
            let known_flags: std::collections::HashSet<String> = sdoc.flag_catalog.iter()
                .flat_map(|e| {
                    let mut flags = vec![e.flag.clone()];
                    if let Some(ref alt) = e.alt_form { flags.push(alt.clone()); }
                    flags
                })
                .collect();

            let hallucinated_input_flags = ["-f", "-i", "-b", "-r", "-1", "-2"];
            let mut new_args = Vec::new();
            let mut skip_next = false;
            for (idx, arg) in args.iter().enumerate() {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                if hallucinated_input_flags.contains(&arg.as_str()) && !known_flags.contains(arg) {
                    if idx + 1 < args.len() && !args[idx + 1].starts_with('-') {
                        new_args.push(args[idx + 1].clone());
                        skip_next = true;
                    }
                } else {
                    new_args.push(arg.clone());
                }
            }
            args = new_args;
        }

        let args_str = args.join(" ");
        let corrected = crate::validator::correct_format(&args_str, sdoc);
        args = crate::llm::response::parse_shell_args(&corrected);

        let args_str = args.join(" ");
        let corrected = crate::validator::aggressive_correct(&args_str, sdoc, tool, task);
        args = crate::llm::response::parse_shell_args(&corrected);

        let args_str = args.join(" ");
        let corrected = crate::validator::validate_subcommand(&args_str, tool, sdoc);
        args = crate::llm::response::parse_shell_args(&corrected);

        args = apply_tool_specific_corrections(&args, tool, task);

        args = fix_generic_output_bam(&args, tool);

        if let Some(task) = task {
            args = fix_output_extensions(&args, tool, task);
        }

        if let Some(task) = task {
            args = limit_flag_count(&args, sdoc, task);
            args = add_missing_required_flags(&args, sdoc, task);
            args = add_task_implied_flags(&args, sdoc, task);
            args = fill_missing_flag_values(&args, sdoc, task);
            args = replace_generic_values(&args, task);
        }

        args
    } else {
        args.to_vec()
    }
}

pub fn apply_template_corrections(
    args: &[String],
    tool: &str,
    structured_doc: Option<&StructuredDoc>,
    task: Option<&str>,
) -> Vec<String> {
    if let Some(sdoc) = structured_doc {
        let mut args = args.to_vec();

        if args.is_empty() {
            return args;
        }

        let args_str = args.join(" ");
        let corrected = crate::validator::correct_format(&args_str, sdoc);
        args = crate::llm::response::parse_shell_args(&corrected);

        let args_str = args.join(" ");
        let corrected = crate::validator::validate_subcommand(&args_str, tool, sdoc);
        args = crate::llm::response::parse_shell_args(&corrected);

        if let Some(task) = task {
            args = add_missing_required_flags(&args, sdoc, task);
            args = add_task_implied_flags(&args, sdoc, task);
        }

        args
    } else {
        args.to_vec()
    }
}

fn replace_generic_values(args: &[String], task: &str) -> Vec<String> {
    let task_values = extract_task_values(task);
    let generic_patterns: &[&str] = &[
        "output.bam", "output.vcf", "output.fastq", "output.fasta",
        "output.sam", "output.bed", "output.txt", "output_dir/",
        "output_dir", "genome_index", "reference_index", "database",
        "input.bam", "input.vcf", "input.fastq", "input.fasta",
        "input.sam", "input.bed", "input.txt",
        "reads.fq", "reads.fastq", "reads_1.fq", "reads_2.fq",
        "reads_1.fastq", "reads_2.fastq",
        "reference.fa", "reference.fasta", "ref.fa", "ref.fasta",
        "input_file", "output_file", "input_dir", "output_directory",
        "query.fasta", "query.fa", "target.fasta", "target.fa",
        "annotation.gtf", "annotation.gff", "annotation.gff3",
        "metrics.txt", "result.txt", "result.tsv", "result.csv",
        "out.sam", "out.bam", "out.vcf", "out.fastq", "out.fasta",
        "output_file", "input_file", "output_path", "input_path",
    ];

    let placeholder_prefixes: &[&str] = &[
        "/path/to/", "path/to/", "<", "example_", "sample_",
    ];

    let mut result = args.to_vec();
    let mut used_replacements: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..result.len() {
        if result[i].starts_with('-') { continue; }
        let val_lower = result[i].to_ascii_lowercase();

        let is_placeholder = generic_patterns.iter().any(|p| val_lower == *p)
            || placeholder_prefixes.iter().any(|p| val_lower.starts_with(p))
            || (val_lower.starts_with('<') && val_lower.ends_with('>'));

        if !is_placeholder { continue; }

        let replacement = if val_lower.starts_with("output") || val_lower.starts_with("out.") || val_lower.starts_with("out_") {
            task_values.output_files.iter()
                .find(|f| !used_replacements.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        } else if val_lower.contains("index") {
            task_values.reference_files.iter()
                .find(|f| !used_replacements.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    task_values.input_files.iter()
                        .find(|f| {
                            let fl = f.to_ascii_lowercase();
                            (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna"))
                                && !used_replacements.contains(&fl)
                        })
                        .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                })
        } else if val_lower == "database" || val_lower.contains("db") {
            task_values.database_files.iter()
                .find(|f| !used_replacements.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        } else if val_lower.contains("annotation") || val_lower.contains(".gtf") || val_lower.contains(".gff") {
            task_values.annotation_files.iter()
                .find(|f| !used_replacements.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    task_values.input_files.iter()
                        .find(|f| {
                            let fl = f.to_ascii_lowercase();
                            (fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".gff3"))
                                && !used_replacements.contains(&fl)
                        })
                        .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                })
        } else if val_lower.contains("reference") || val_lower.contains("ref.") || val_lower == "ref.fa" || val_lower == "ref.fasta" {
            task_values.reference_files.iter()
                .find(|f| !used_replacements.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    task_values.input_files.iter()
                        .find(|f| {
                            let fl = f.to_ascii_lowercase();
                            (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna"))
                                && !used_replacements.contains(&fl)
                        })
                        .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                })
        } else if val_lower.contains("query") || val_lower.contains("target") {
            task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna"))
                        && !used_replacements.contains(&fl)
                })
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    find_any_unused_file(&task_values.input_files, &used_replacements)
                        .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                })
        } else if val_lower.contains("fastq") || val_lower.contains("reads") {
            task_values.read_files.iter()
                .find(|f| !used_replacements.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    task_values.input_files.iter()
                        .find(|f| {
                            let fl = f.to_ascii_lowercase();
                            (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".gz"))
                                && !used_replacements.contains(&fl)
                        })
                        .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
                })
        } else if val_lower.contains("bam") || val_lower.contains("sam") {
            task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".bam") || fl.ends_with(".sam"))
                        && !used_replacements.contains(&fl)
                })
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        } else if val_lower.contains("vcf") {
            task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".vcf") || fl.ends_with(".bcf"))
                        && !used_replacements.contains(&fl)
                })
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        } else if val_lower.contains("bed") {
            task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    fl.ends_with(".bed") && !used_replacements.contains(&fl)
                })
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        } else if val_lower.starts_with("input") {
            let ext = if val_lower.contains(".bam") { ".bam" }
                else if val_lower.contains(".vcf") { ".vcf" }
                else if val_lower.contains(".fastq") { ".fastq" }
                else if val_lower.contains(".fasta") { ".fasta" }
                else if val_lower.contains(".sam") { ".sam" }
                else if val_lower.contains(".bed") { ".bed" }
                else if val_lower.contains(".txt") { ".txt" }
                else { "" };
            if !ext.is_empty() {
                task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(ext) && !used_replacements.contains(&fl)
                    })
                    .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
            } else {
                find_any_unused_file(&task_values.input_files, &used_replacements)
                    .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
            }
        } else if val_lower.starts_with("/path/to/") || val_lower.starts_with("path/to/") {
            find_any_unused_file(&task_values.input_files, &used_replacements)
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        } else {
            find_any_unused_file(&task_values.input_files, &used_replacements)
                .map(|f| { used_replacements.insert(f.to_ascii_lowercase()); f.clone() })
        };

        if let Some(repl) = replacement {
            result[i] = repl;
        }
    }
    result
}

fn find_any_unused_file<'a>(
    files: &'a [String],
    used: &std::collections::HashSet<String>,
) -> Option<&'a String> {
    files.iter().find(|f| !used.contains(&f.to_ascii_lowercase()))
}

fn clean_help_text_in_args(args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for arg in args {
        if arg.starts_with('-') && arg.contains(',') {
            let parts: Vec<&str> = arg.split(',').collect();
            if parts.len() == 2 {
                let first = parts[0];
                let second = parts[1];
                if second.starts_with('-') && first.starts_with('-') {
                    if second.len() <= first.len() {
                        result.push(second.to_string());
                    } else {
                        result.push(first.to_string());
                    }
                    continue;
                }
                if second.starts_with('-') && !first.contains('=') {
                    result.push(second.to_string());
                    continue;
                }
            }
            if arg.contains("=<") || arg.contains("= <") || arg.contains("[") {
                if let Some(flag_part) = arg.split(',').next() {
                    let cleaned = flag_part.split('=').next().unwrap_or(flag_part);
                    result.push(cleaned.to_string());
                    continue;
                }
            }
        }
        if arg.starts_with('-') && (arg.contains("[") || arg.contains("=<") || arg.contains("= <")) {
            if let Some(flag_part) = arg.split('[').next() {
                let cleaned = flag_part.split('=').next().unwrap_or(flag_part).trim();
                result.push(cleaned.to_string());
                continue;
            }
        }
        result.push(arg.clone());
    }
    result
}

fn fill_missing_flag_values(args: &[String], sdoc: &StructuredDoc, task: &str) -> Vec<String> {
    let task_values = extract_task_values(task);
    let task_lower = task.to_ascii_lowercase();
    let mut result = Vec::new();
    let args_len = args.len();

    let known_value_flags: std::collections::HashSet<String> = sdoc.flag_catalog.iter()
        .filter(|e| e.value_type.is_some() || e.flag.ends_with('='))
        .flat_map(|e| {
            let mut flags = vec![e.flag.split('=').next().unwrap_or(&e.flag).to_string()];
            if let Some(ref alt) = e.alt_form {
                flags.push(alt.split('=').next().unwrap_or(alt).to_string());
            }
            flags
        })
        .collect();

    let mut i = 0;
    while i < args_len {
        result.push(args[i].clone());
        if args[i].starts_with('-') && !args[i].contains('=') {
            let flag_key = if args[i].starts_with("--") {
                args[i].as_str()
            } else if args[i].len() > 2 {
                &args[i][..2]
            } else {
                args[i].as_str()
            };

            let needs_value = known_value_flags.contains(flag_key);
            let has_next_value = i + 1 < args_len && !args[i + 1].starts_with('-');

            if needs_value && !has_next_value {
                let entry = sdoc.flag_catalog.iter().find(|e| {
                    e.flag.split('=').next().unwrap_or(&e.flag) == flag_key
                        || e.alt_form.as_ref().map_or(false, |a| a.split('=').next().unwrap_or(a) == flag_key)
                });

                if let Some(e) = entry {
                    let desc_lower = e.description.to_ascii_lowercase();
                    let flag_lower = e.flag.to_ascii_lowercase();

                    if let Some(ref default) = e.default {
                        result.push(default.clone());
                    } else if desc_lower.contains("sort") && desc_lower.contains("order") {
                        result.push("coordinate".to_string());
                    } else if desc_lower.contains("output") || flag_lower.contains("out") {
                        if let Some(out) = task_values.output_files.first() {
                            result.push(out.clone());
                        }
                    } else if desc_lower.contains("thread") || desc_lower.contains("cpu") {
                        result.push("4".to_string());
                    } else if desc_lower.contains("format") || flag_lower.contains("outfmt") {
                        if !e.enum_values.is_empty() {
                            result.push(e.enum_values[0].clone());
                        }
                    }
                }
            }
        }
        i += 1;
    }
    result
}

fn extract_subcommand_from_task(tool: &str, task: &str) -> Option<String> {
    let tl = task.to_ascii_lowercase();
    let rules: &[(&[&str], &str)] = match tool {
        "gatk" => &[
            (&["haplotypecaller", "haplotype caller", "germline variant"], "HaplotypeCaller"),
            (&["markduplicate", "mark duplicate", "mark pcr", "pcr duplicate"], "MarkDuplicates"),
            (&["mutect2", "somatic mutat"], "Mutect2"),
            (&["filtermutectcalls", "filter mutect"], "FilterMutectCalls"),
            (&["createsequencedictionary", "sequence dictionary"], "CreateSequenceDictionary"),
            (&["addorreplacereadgroup", "add or replace read group", "read group"], "AddOrReplaceReadGroups"),
            (&["baserecalibrator", "base recalibrat", "bqsr step 1", "recalibrat"], "BaseRecalibrator"),
            (&["applybqsr", "apply bqsr", "bqsr step 2", "recalibrated bam"], "ApplyBQSR"),
            (&["selectvariant", "select variant", "select only snp"], "SelectVariants"),
        ],
        "picard" => &[
            (&["sortsam", "sort sam", "coordinate order", "queryname order", "sort bam"], "SortSam"),
            (&["markduplicate", "mark duplicate", "pcr duplicate"], "MarkDuplicates"),
            (&["addorreplacereadgroup", "add or replace read group", "read group"], "AddOrReplaceReadGroups"),
            (&["collectalignmentsummarymetric", "alignment summary", "alignment metric"], "CollectAlignmentSummaryMetrics"),
            (&["collectinsertsizemetric", "insert size", "insert metric"], "CollectInsertSizeMetrics"),
            (&["validatesamfile", "validate sam"], "ValidateSamFile"),
        ],
        "bcftools" => &[
            (&["mpileup", "pileup", "call variant from bam"], "mpileup"),
            (&["view", "filter vcf", "extract sample", "select only snp", "keep only"], "view"),
            (&["merge", "merge multiple vcf", "merge vcf"], "merge"),
            (&["norm", "normalize", "split multi-allelic", "left-align"], "norm"),
            (&["stats", "statistic", "compute variant stat"], "stats"),
            (&["annotate", "annotat vcf", "add id field"], "annotate"),
            (&["isec", "intersection", "shared between", "common variant"], "isec"),
            (&["query", "extract custom field", "extract field", "tsv"], "query"),
        ],
        "blast" => &[
            (&["makeblastdb", "build database", "create database", "blast database from"], "makeblastdb"),
            (&["blastn", "nucleotide blast", "nucleotide sequence", "blastn"], "blastn"),
            (&["blastp", "protein sequence", "protein blast", "search protein"], "blastp"),
            (&["blastx", "nucleotide against protein", "translate nucleotide", "blastx"], "blastx"),
            (&["blastdbcmd", "retrieve sequence from database", "by accession"], "blastdbcmd"),
            (&["tblastn", "protein against nucleotide"], "tblastn"),
        ],
        "bismark" => &[
            (&["genome_preparation", "genome preparation", "prepare genome", "build genome index", "bisulfite genome"], "bismark_genome_preparation"),
            (&["deduplicate", "deduplicate_bismark", "remove duplicate"], "deduplicate_bismark"),
            (&["methylation_extractor", "methylation extract", "extract methylation"], "bismark_methylation_extractor"),
            (&["bismark2report", "report", "summary report"], "bismark2report"),
        ],
        "hmmer" => &[
            (&["hmmsearch", "search profile against sequence", "search hmm against"], "hmmsearch"),
            (&["hmmscan", "scan sequence against profile", "scan against hmm", "search sequence against profile"], "hmmscan"),
            (&["hmmbuild", "build profile", "build hmm", "multiple alignment to profile"], "hmmbuild"),
            (&["hmmpress", "press hmm", "format hmm database"], "hmmpress"),
            (&["phmmer", "search protein sequence against protein"], "phmmer"),
            (&["hmmalign", "align sequence to profile", "multiple alignment with profile"], "hmmalign"),
        ],
        "samtools" => &[
            (&["view", "convert bam to sam", "convert sam to bam", "extract sam", "filter bam"], "view"),
            (&["sort", "sort bam", "sort by coordinate", "sort by name"], "sort"),
            (&["index", "index bam", "bai index", "create index"], "index"),
            (&["flagstat", "flag statistic", "mapping statistic"], "flagstat"),
            (&["fastq", "convert bam to fastq", "bam to fastq", "extract fastq"], "fastq"),
            (&["markdup", "mark duplicate", "mark duplicate in bam"], "markdup"),
            (&["merge", "merge bam", "merge multiple bam"], "merge"),
            (&["depth", "compute depth", "coverage depth", "read depth"], "depth"),
        ],
        "bedtools" => &[
            (&["intersect", "find overlap", "overlap between"], "intersect"),
            (&["genomecov", "genome coverage", "coverage across genome", "bedgraph"], "genomecov"),
            (&["subtract", "remove overlap", "subtract bed"], "subtract"),
            (&["merge", "merge overlapping", "merge bed", "merge interval"], "merge"),
            (&["closest", "nearest feature", "closest feature"], "closest"),
            (&["getfasta", "extract sequence from bed", "sequence from interval", "fasta from bed"], "getfasta"),
            (&["makewindows", "create window", "tile genome", "window"], "makewindows"),
            (&["coverage", "coverage per feature", "compute coverage"], "coverage"),
        ],
        "sourmash" => &[
            (&["sketch", "create sketch", "minhash sketch", "compute signature", "compute minhash", "create signature", "generate signature", "build signature", "minhash from"], "sketch"),
            (&["compare", "compare signature", "compare sketch", "distance matrix", "similarity matrix", "compare minhash"], "compare"),
            (&["gather", "metagenomic gather", "find genome in metagenome", "metagenome"], "gather"),
            (&["taxonomy", "classify taxonom", "taxonomic classificat", "classify genome"], "taxonomy"),
            (&["search", "find similar", "search signature", "nearest neighbor", "find closest"], "search"),
            (&["index", "build index", "sbt index"], "index"),
        ],
        "sra-tools" => &[
            (&["fasterq-dump", "fasterq", "download fastq", "convert sra to fastq", "dump sra"], "fasterq-dump"),
            (&["prefetch", "prefetch sra", "download sra"], "prefetch"),
            (&["vdb-validate", "validate sra", "validate file"], "vdb-validate"),
            (&["sra-stat", "statistic sra", "sra stat"], "sra-stat"),
        ],
        "varscan2" => &[
            (&["mpileup2snp", "snp from pileup", "call snp from mpileup", "somatic snp"], "mpileup2snp"),
            (&["mpileup2indel", "indel from pileup", "call indel from mpileup"], "mpileup2indel"),
            (&["somatic", "somatic variant", "somatic call", "tumor-normal"], "somatic"),
            (&["processsomatic", "process somatic", "filter somatic"], "processSomatic"),
        ],
        "delly" => &[
            (&[" call", "call structural variant", "call sv", "detect structural"], "call"),
            (&[" lr", "long-read sv", "long read structural", "pacbio sv", "ont sv"], "lr"),
            (&["filter", "filter sv", "filter structural variant"], "filter"),
            (&["merge", "merge sv", "merge structural variant", "merge bcf"], "merge"),
            (&["cnv", "copy number variant", "cnv call"], "cnv"),
        ],
        "mmseqs2" => &[
            (&["easy-search", "easy search", "search sequence"], "easy-search"),
            (&["easy-cluster", "easy cluster", "cluster sequence"], "easy-cluster"),
            (&["easy-linclust", "easy linclust", "linear cluster", "linclust"], "easy-linclust"),
            (&["createdb", "create database", "create mmseqs database"], "createdb"),
            (&["search", "mmseqs search", "sensitive search"], "search"),
            (&["result2repseq", "representative sequence", "cluster representative"], "result2repseq"),
        ],
        "bracken" => &[
            (&["bracken-build", "build bracken", "bracken database", "build database"], "bracken-build"),
            (&["combine_bracken_outputs", "combine bracken", "merge bracken", "combine report"], "combine_bracken_outputs"),
        ],
        "diamond" => &[
            (&["makedb", "make database", "build database", "diamond database"], "makedb"),
            (&["blastp", "protein search", "diamond blastp"], "blastp"),
            (&["blastx", "translated search", "diamond blastx"], "blastx"),
            (&["cluster", "cluster protein", "diamond cluster"], "cluster"),
            (&["linclust", "linear cluster", "fast cluster"], "linclust"),
        ],
        "deeptools" => &[
            (&["bamcoverage", "bam coverage", "coverage bigwig", "coverage track"], "bamCoverage"),
            (&["bamcompare", "bam compare", "compare signal", "differential coverage"], "bamCompare"),
            (&["computematrix", "compute matrix", "matrix for heatmap", "signal matrix"], "computeMatrix"),
            (&["plotheatmap", "plot heatmap", "heatmap"], "plotHeatmap"),
            (&["multibamsummary", "multi bam summary", "correlation"], "multiBamSummary"),
            (&["plotfingerprint", "plot fingerprint", "chip-seq quality", "fingerprint"], "plotFingerprint"),
        ],
        "cnvkit" => &[
            (&["batch", "cnvkit batch", "run cnvkit"], "batch"),
            (&["scatter", "cnvkit scatter", "plot cnv", "scatter plot"], "scatter"),
            (&["call", "cnvkit call", "call cnv"], "call"),
            (&["segment", "cnvkit segment", "segment cnv"], "segment"),
            (&["heatmap", "cnvkit heatmap", "heatmap cnv"], "heatmap"),
            (&["genemetrics", "gene metric", "cnv gene"], "genemetrics"),
        ],
        "salmon" => &[
            (&["index", "salmon index", "build index"], "index"),
            (&["quant", "salmon quant", "quantify transcript", "expression quant"], "quant"),
        ],
        "kallisto" => &[
            (&["index", "kallisto index", "build index"], "index"),
            (&["quant", "kallisto quant", "quantify", "expression quant"], "quant"),
            (&["bus", "kallisto bus", "bus format"], "bus"),
        ],
        "rsem" => &[
            (&["rsem-prepare-reference", "prepare reference", "rsem reference"], "rsem-prepare-reference"),
            (&["rsem-calculate-expression", "calculate expression", "rsem quant", "rsem expression"], "rsem-calculate-expression"),
            (&["rsem-generate-data-matrix", "generate data matrix", "rsem matrix"], "rsem-generate-data-matrix"),
        ],
        "mummer" => &[
            (&["nucmer", "nucleotide align", "nucleotide mummer"], "nucmer"),
            (&["dnadiff", "dna diff", "compare genome"], "dnadiff"),
            (&["delta-filter", "filter delta", "filter alignment"], "delta-filter"),
            (&["show-coords", "show coordinate", "show alignment"], "show-coords"),
            (&["mummerplot", "plot mummer", "dot plot"], "mummerplot"),
            (&["show-tiling", "tiling"], "show-tiling"),
        ],
        "igvtools" => &[
            (&["count", "igv count", "coverage count"], "count"),
            (&["index", "igv index"], "index"),
            (&["sort", "igv sort"], "sort"),
            (&["totdf", "to tdf", "tdf"], "toTDF"),
            (&["formatexp", "format exp"], "formatexp"),
        ],
        "homer" => &[
            (&["maketagdirectory", "tag directory", "create tag"], "makeTagDirectory"),
            (&["findpeaks", "find peak", "peak calling"], "findPeaks"),
            (&["annotatepeaks", "annotate peak"], "annotatePeaks.pl"),
            (&["findmotifsgenome", "find motif", "motif finding"], "findMotifsGenome.pl"),
            (&["mergepeaks", "merge peak"], "mergePeaks"),
            (&["pos2bed", "pos to bed"], "pos2bed.pl"),
            (&["makeucscfile", "ucsc file"], "makeUCSCfile"),
            (&["getdifferentialpeaksreplicates", "differential peak"], "getDifferentialPeaksReplicates.pl"),
        ],
        "gtdbtk" => &[
            (&["classify_wf", "classify workflow", "taxonomic classificat"], "classify_wf"),
            (&["identify", "identify marker", "gtdb identify"], "identify"),
            (&["de_novo_wf", "de novo workflow", "de novo tree"], "de_novo_wf"),
            (&["align", "align marker", "gtdb align"], "align"),
            (&["classify", "gtdb classify"], "classify"),
        ],
        "checkm2" => &[
            (&["predict", "checkm predict", "completeness", "contamination"], "predict"),
            (&["database", "checkm database", "download database"], "database"),
            (&["testrun", "test run", "checkm test"], "testrun"),
        ],
        "qualimap" => &[
            (&["bamqc", "bam qc", "quality control bam"], "bamqc"),
            (&["rnaseq", "rna-seq qc", "rnaseq quality"], "rnaseq"),
            (&["multi-bamqc", "multi bam qc"], "multi-bamqc"),
            (&["counts", "qualimap counts"], "counts"),
        ],
        "macs2" => &[
            (&["callpeak", "peak calling", "call peak", "macs2 call"], "callpeak"),
            (&["predictd", "predict fragment", "fragment size"], "predictd"),
        ],
        "mash" => &[
            (&["sketch", "mash sketch", "create sketch"], "sketch"),
            (&["dist", "mash dist", "distance", "compare genome"], "dist"),
            (&["screen", "mash screen", "screen contain"], "screen"),
            (&["triangle", "mash triangle", "all-vs-all", "pairwise distance"], "triangle"),
            (&["paste", "mash paste", "merge sketch"], "paste"),
            (&["info", "mash info", "sketch info"], "info"),
        ],
        "seqkit" => &[
            (&["stats", "seqkit stat", "sequence statistic"], "stats"),
            (&["seq", "seqkit seq", "transform sequence", "reverse complement"], "seq"),
            (&["grep", "seqkit grep", "search sequence"], "grep"),
            (&["sample", "seqkit sample", "random sample"], "sample"),
            (&["fq2fa", "fastq to fasta", "convert fastq"], "fq2fa"),
            (&["split2", "seqkit split", "split sequence"], "split2"),
        ],
        "seqtk" => &[
            (&["sample", "seqtk sample", "random sample"], "sample"),
            (&["seq", "seqtk seq", "convert fastq", "transform sequence"], "seq"),
            (&["subseq", "seqtk subseq", "extract subsequence"], "subseq"),
            (&["trimfq", "seqtk trim", "trim fastq"], "trimfq"),
        ],
        "snpeff" => &[
            (&[" ann", "snpeff annotat", "annotate vcf", "annotate variant"], "ann"),
            (&["build", "snpeff build", "build database"], "build"),
        ],
        "survivor" => &[
            (&["merge", "survivor merge", "merge sv"], "merge"),
            (&["stats", "survivor stat", "sv statistic"], "stats"),
            (&["filter", "survivor filter", "filter sv"], "filter"),
            (&["simsv", "simulate sv", "simulate structural"], "simSV"),
        ],
        "whatshap" => &[
            (&["phase", "whatshap phase", "phasing", "haplotype"], "phase"),
            (&["haplotag", "whatshap haplotag", "assign haplotype", "tag read"], "haplotag"),
            (&["stats", "whatshap stat", "phasing statistic"], "stats"),
        ],
        "modkit" => &[
            (&["pileup", "modkit pileup", "methylation pileup", "call modification"], "pileup"),
            (&["extract", "modkit extract", "extract modification"], "extract"),
            (&["summary", "modkit summary", "modification summary"], "summary"),
            (&["motif-bed", "modkit motif", "motif bed"], "motif-bed"),
            (&["sample-probs", "modkit sample", "sample prob"], "sample-probs"),
        ],
        "pairtools" => &[
            (&["parse", "pairtools parse", "parse sam", "parse alignment"], "parse"),
            (&["sort", "pairtools sort", "sort pair"], "sort"),
            (&["dedup", "pairtools dedup", "deduplicate pair", "remove duplicate"], "dedup"),
            (&["cload", "pairtools cload", "load cooler"], "cload"),
        ],
        "nextflow" => &[
            (&["run", "nextflow run", "execute pipeline"], "run"),
            (&["pull", "nextflow pull", "download pipeline"], "pull"),
            (&["list", "nextflow list", "list pipeline"], "list"),
            (&["clean", "nextflow clean", "clean cache"], "clean"),
        ],
        "strelka2" => &[
            (&["configurestrelkagermlineworkflow", "germline workflow", "germline variant"], "configureStrelkaGermlineWorkflow.py"),
            (&["configurestrelkasomaticworkflow", "somatic workflow", "somatic variant"], "configureStrelkaSomaticWorkflow.py"),
        ],
        "stringtie" => &[
            (&["--merge", "stringtie merge", "merge transcript", "merge gtf"], "--merge"),
            (&["-e", "stringtie -e", "estimate abundance", "ballgown"], "-e"),
        ],
        "bbtools" => &[
            (&["bbduk.sh", "bbduk", "quality filter", "adapter trim", "contaminant"], "bbduk.sh"),
            (&["bbmap.sh", "bbmap", "align read"], "bbmap.sh"),
            (&["bbmerge.sh", "bbmerge", "merge read", "extend read"], "bbmerge.sh"),
            (&["reformat.sh", "reformat", "convert format", "change format"], "reformat.sh"),
            (&["dedupe.sh", "dedupe", "remove duplicate"], "dedupe.sh"),
            (&["bbsplit.sh", "bbsplit", "separate by organism"], "bbsplit.sh"),
        ],
        "agat" => &[
            (&["agat_convert_sp_gff2gtf", "gff to gtf", "convert gff gtf"], "agat_convert_sp_gff2gtf"),
            (&["agat_sp_statistics", "gff statistic", "annotation statistic"], "agat_sp_statistics"),
            (&["agat_sp_filter_gene_by_length", "filter gene by length", "filter by length"], "agat_sp_filter_gene_by_length"),
            (&["agat_convert_sp_gxf2gxf", "fix gff", "gxf to gxf", "standardize gff"], "agat_convert_sp_gxf2gxf"),
            (&["agat_sp_extract_sequences", "extract sequence from gff", "extract from annotation"], "agat_sp_extract_sequences"),
            (&["agat_sp_keep_longest_isoform", "longest isoform", "keep longest"], "agat_sp_keep_longest_isoform"),
            (&["agat_sp_merge_annotations", "merge annotation", "merge gff"], "agat_sp_merge_annotations"),
            (&["agat_sp_manage_ids", "manage id", "fix id"], "agat_sp_manage_IDs"),
            (&["agat_convert_sp_gff2bed", "gff to bed", "convert gff bed"], "agat_convert_sp_gff2bed"),
        ],
        "bamtools" => &[
            (&["stats", "bam statistic"], "stats"),
            (&["count", "count read", "count alignment"], "count"),
            (&["filter", "filter bam", "filter alignment"], "filter"),
            (&["merge", "merge bam"], "merge"),
            (&["split", "split bam", "split by reference", "split by read group"], "split"),
            (&["convert", "convert bam", "bam to json", "bam to bed"], "convert"),
        ],
        "busco" => &[
            (&["--plot", "busco plot", "generate plot"], "--plot"),
            (&["--restart", "busco restart", "restart run"], "--restart"),
            (&["--list-datasets", "list dataset", "available lineage"], "--list-datasets"),
        ],
        "medaka" => &[
            (&["medaka_consensus", "medaka consensus", "consensus call"], "medaka_consensus"),
            (&["medaka_variant", "medaka variant", "variant call"], "medaka_variant"),
            (&["medaka_haploid_variant", "medaka haploid", "haploid variant"], "medaka_haploid_variant"),
        ],
        "pilon" => &[
        ],
        "spades" => &[
            (&["--meta", "metagenomic assembly", "meta spades"], "--meta"),
            (&["--plasmid", "plasmid assembly", "plasmid spades"], "--plasmid"),
            (&["--sc", "single cell assembly", "sc spades"], "--sc"),
            (&["--isolate", "isolate assembly", "isolate spades"], "--isolate"),
            (&["--rnaviral", "rna viral", "viral genome"], "--rnaviral"),
            (&["--corona", "coronavirus", "sars-cov"], "--corona"),
            (&["--bio", "biosynthetic", "biosynthetic spades"], "--bio"),
        ],
        "methyldackel" => &[
            (&["extract", "methyldackel extract", "extract methylation"], "extract"),
            (&["mbias", "methyldackel mbias", "bias plot"], "mbias"),
        ],
        "chromap" => &[
        ],
        "trinity" => &[
            (&["--genome_guided_bam", "genome-guided", "genome guided", "guided assembly"], "--genome_guided_bam"),
        ],
        "trimmomatic" => &[
            (&["pe", "paired-end", "paired end", "trim paired"], "PE"),
            (&["se", "single-end", "single end", "trim single"], "SE"),
        ],
        "muscle" => &[
            (&["-super5", "super5", "large alignment"], "-super5"),
            (&["-align", "muscle align", "multiple alignment"], "-align"),
        ],
        "meme" => &[
            (&["fimo", "scan for motif", "motif occurrence"], "fimo"),
            (&["tomtom", "compare motif", "motif similarity"], "tomtom"),
            (&["ame", "motif enrichment", "enrichment test"], "ame"),
            (&["streme", "discover motif", "find motif", "de novo motif"], "streme"),
        ],
        "truvari" => &[
            (&["bench", "truvari bench", "benchmark variant"], "bench"),
            (&["collapse", "truvari collapse", "collapse variant"], "collapse"),
            (&["refine", "truvari refine", "refine region"], "refine"),
        ],
        "pbmm2" => &[
            (&["align", "pbmm2 align", "pacbio align"], "align"),
            (&["index", "pbmm2 index", "pacbio index"], "index"),
        ],
        "pbsv" => &[
            (&["discover", "pbsv discover", "discover sv", "find sv"], "discover"),
            (&["call", "pbsv call", "call sv"], "call"),
        ],
        "kb" => &[
            (&["ref", "kb ref", "build index", "build reference"], "ref"),
            (&["count", "kb count", "quantify", "count cell"], "count"),
        ],
        "plink2" => &[
        ],
        "shapeit4" => &[
        ],
        "fasttree" => &[
            (&["-nt", "nucleotide tree", "dna tree"], "-nt"),
            (&["-wag", "wag model", "wag"], "-wag"),
            (&["-lg", "lg model", "lg"], "-lg"),
        ],
        "nanoplot" => &[
            (&["--fastq", "nanoplot fastq", "fastq quality"], "--fastq"),
            (&["--summary", "nanoplot summary", "summary quality"], "--summary"),
            (&["--bam", "nanoplot bam", "bam quality"], "--bam"),
        ],
        "nanostat" => &[
            (&["--fastq", "nanostat fastq"], "--fastq"),
            (&["--summary", "nanostat summary"], "--summary"),
            (&["--bam", "nanostat bam"], "--bam"),
        ],
        "centrifuge" => &[
            (&["centrifuge-build", "build centrifuge", "centrifuge database"], "centrifuge-build"),
            (&["centrifuge-kreport", "kreport", "kraken report"], "centrifuge-kreport"),
        ],
        "kraken2" => &[
            (&["kraken2-build", "build kraken", "kraken database"], "kraken2-build"),
        ],
        "orthofinder" => &[
            (&["-f", "orthofinder find", "find ortholog", "from directory"], "-f"),
            (&["-b", "orthofinder from blast", "from blast result"], "-b"),
        ],
        "metabat2" => &[
            (&["jgi_summarize_bam_contig_depths", "jgi summarize", "depth file", "contig depth"], "jgi_summarize_bam_contig_depths"),
        ],
        "repeatmasker" => &[
            (&["-species", "repeatmasker species", "mask repeat"], "-species"),
            (&["-lib", "repeatmasker library", "custom library"], "-lib"),
            (&["-noint", "no int", "without interspersed"], "-noint"),
        ],
        "snakemake" => &[
            (&["--cores", "snakemake run", "execute workflow"], "--cores"),
            (&["--dry-run", "dry run", "snakemake dryrun"], "--dry-run"),
            (&["--executor", "snakemake executor"], "--executor"),
            (&["--configfile", "snakemake config"], "--configfile"),
            (&["--profile", "snakemake profile"], "--profile"),
            (&["--forcerun", "force run"], "--forcerun"),
            (&["--unlock", "unlock directory"], "--unlock"),
            (&["--dag", "dag", "workflow graph"], "--dag"),
            (&["--rerun-incomplete", "rerun incomplete"], "--rerun-incomplete"),
            (&["--use-singularity", "singularity", "container"], "--use-singularity"),
        ],
        "git" => &[
            (&["clone", "clone repo", "download repo"], "clone"),
            (&["checkout", "switch branch", "create branch", "checkout branch"], "checkout"),
            (&["commit", "create commit", "save change"], "commit"),
            (&["push", "upload commit", "push to remote"], "push"),
            (&["pull", "download change", "pull from remote"], "pull"),
            (&["log", "commit log", "commit history", "show log"], "log"),
            (&["branch", "list branch", "create branch", "show branch"], "branch"),
            (&["merge", "merge branch", "merge change"], "merge"),
            (&["fetch", "fetch remote", "download object"], "fetch"),
            (&["status", "working tree", "show status"], "status"),
            (&["diff", "show diff", "compare change"], "diff"),
            (&["add", "stage file", "add file"], "add"),
            (&["stash", "stash change", "temporarily save"], "stash"),
            (&["tag", "create tag", "version tag"], "tag"),
            (&["reset", "undo commit", "reset change"], "reset"),
            (&["rebase", "rebase branch", "rebase commit"], "rebase"),
        ],
        "vcftools" => &[
            (&["--freq", "allele frequency", "frequency"], "--freq"),
            (&["--hardy", "hardy weinberg", "hwe"], "--hardy"),
            (&["--het", "heterozygosity", "inbreeding coefficient"], "--het"),
            (&["--site-pi", "nucleotide diversity", "pi"], "--site-pi"),
            (&["--tajima-d", "tajima d", "tajima"], "--TajimaD"),
            (&["--window-pi", "window pi", "pi in window"], "--window-pi"),
            (&["--remove-indels", "keep only snp", "remove indel"], "--remove-indels"),
            (&["--maf", "minor allele frequency filter", "filter by maf"], "--maf"),
            (&["--max-missing", "missing data filter", "filter by missing"], "--max-missing"),
            (&["--recode", "recode vcf", "output filtered"], "--recode"),
        ],
        "flye" => &[
            (&["--nano-raw", "ont raw", "nanopore raw", "ont read"], "--nano-raw"),
            (&["--nano-corr", "ont corrected", "nanopore corrected"], "--nano-corr"),
            (&["--pacbio-raw", "pacbio raw", "pacbio clr"], "--pacbio-raw"),
            (&["--pacbio-corr", "pacbio corrected", "pacbio hifi corrected"], "--pacbio-corr"),
            (&["--pacbio-hifi", "pacbio hifi", "hifi read", "ccs read"], "--pacbio-hifi"),
        ],
        "hifiasm" => &[
            (&["--h1", "--h2", "trio assembly", "hap1 hap2", "paternal maternal"], "--h1"),
            (&["--n-hap", "polyploid", "haplotype number"], "--n-hap"),
            (&["-l0", "purge duplicate", "l0 purge"], "-l0"),
        ],
        "minimap2" => &[
            (&["map-ont", "ont read", "nanopore map", "ont align"], "-ax map-ont"),
            (&["map-pb", "pacbio map", "pacbio align", "clr map"], "-ax map-pb"),
            (&["map-hifi", "hifi map", "pacbio hifi align", "ccs map"], "-ax map-hifi"),
            (&["splice", "splice aware", "rna map", "long read rna"], "-ax splice"),
        ],
        "star" => &[
            (&["genomegenerate", "generate genome", "genome index", "build index"], "--runMode"),
            (&["alignreads", "align read", "map read", "star align"], "--runMode"),
        ],
        "bowtie2" => &[
            (&["build", "bowtie2-build", "build index", "create index"], "bowtie2-build"),
            (&["inspect", "bowtie2-inspect", "inspect index"], "bowtie2-inspect"),
        ],
        "bwa" => &[
            (&["mem", "bwa mem", "align read", "map read"], "mem"),
            (&["index", "bwa index", "build index"], "index"),
        ],
        "bwa-mem2" => &[
            (&["mem", "bwa-mem2 mem", "align read", "map read"], "mem"),
            (&["index", "bwa-mem2 index", "build index"], "index"),
        ],
        "wget" => &[
            (&["-O", "output file", "save as", "download to file"], "-O"),
            (&["-c", "continue", "resume download"], "-c"),
            (&["-b", "background", "background download"], "-b"),
            (&["-q", "quiet", "silent download"], "-q"),
        ],
        "curl" => &[
            (&["-o", "output file", "save as", "download to file"], "-o"),
            (&["-O", "remote name", "save with original name"], "-O"),
            (&["-L", "follow redirect", "redirect"], "-L"),
            (&["-s", "silent", "quiet"], "-s"),
            (&["-T", "upload file", "put file"], "-T"),
        ],
        "ssh" => &[
            (&["-i", "identity file", "private key", "key file"], "-i"),
            (&["-p", "port", "ssh port"], "-p"),
            (&["-L", "local forward", "port forward"], "-L"),
            (&["-R", "remote forward"], "-R"),
        ],
        "rsync" => &[
            (&["-avz", "archive compress", "sync directory"], "-avz"),
            (&["-a", "archive mode", "preserve"], "-a"),
            (&["--delete", "delete extraneous", "mirror"], "--delete"),
        ],
        "find" => &[
            (&["-name", "find by name", "search file name"], "-name"),
            (&["-type", "find by type", "file type"], "-type"),
            (&["-size", "find by size", "file size"], "-size"),
            (&["-mtime", "find by date", "modification time"], "-mtime"),
            (&["-exec", "execute command", "run command on"], "-exec"),
        ],
        "rm" => &[
            (&["-r", "recursive", "remove directory"], "-r"),
            (&["-rf", "force recursive", "force remove"], "-rf"),
            (&["-f", "force", "force remove file"], "-f"),
            (&["-v", "verbose", "show what removed"], "-v"),
        ],
        "tar" => &[
            (&["-czf", "create gzip", "compress tar"], "-czf"),
            (&["-xzf", "extract gzip", "decompress tar"], "-xzf"),
            (&["-tf", "list content", "show content"], "-tf"),
        ],
        "grep" => &[
            (&["-r", "recursive", "search directory"], "-r"),
            (&["-i", "case insensitive", "ignore case"], "-i"),
            (&["-n", "line number", "show line number"], "-n"),
            (&["-c", "count match", "count occurrence"], "-c"),
            (&["-v", "invert match", "exclude pattern"], "-v"),
            (&["-C", "context line", "surrounding line"], "-C"),
            (&["--include", "file pattern", "search in file type"], "--include"),
        ],
        "sed" => &[
            (&["-i", "in-place", "edit file in place"], "-i"),
            (&["s/", "substitute", "replace", "find replace"], "s/"),
        ],
        "awk" => &[
            (&["-F", "field separator", "delimiter", "csv", "tsv"], "-F"),
            (&["{print", "print column", "print field", "extract column"], "{print"),
        ],
        "pbccs" => &[
            (&["--min-passes", "minimum pass", "ccs pass"], "--min-passes"),
            (&["--hifi-kinetics", "hifi kinetics", "kinetics"], "--hifi-kinetics"),
        ],
        "verkko" => &[
            (&["--hifi", "hifi assembly", "pacbio hifi"], "--hifi"),
            (&["--ont", "ont assembly", "nanopore"], "--ont"),
        ],
        "plink2" => &[
            (&["--pca", "pca", "principal component"], "--pca"),
            (&["--assoc", "association test", "case-control"], "--assoc"),
            (&["--make-bed", "create bed", "binary format"], "--make-bed"),
            (&["--freq", "allele frequency"], "--freq"),
            (&["--hardy", "hardy weinberg"], "--hardy"),
            (&["--mind", "remove sample", "missing person"], "--mind"),
            (&["--geno", "remove variant", "missing genotype"], "--geno"),
            (&["--maf", "maf filter"], "--maf"),
            (&["--hwe", "hwe filter"], "--hwe"),
        ],
        "cutadapt" => &[
            (&["-a", "adapter 3", "3 prime adapter", "remove adapter"], "-a"),
            (&["-g", "adapter 5", "5 prime adapter", "front adapter"], "-g"),
            (&["-A", "adapter r2", "second read adapter"], "-A"),
        ],
        "eggnog-mapper" => &[
            (&["emapper", "annotate protein", "eggnog mapper"], "emapper.py"),
        ],
        "metaphlan" => &[
            (&["merge_metaphlan_tables", "merge metaphlan", "combine metaphlan"], "merge_metaphlan_tables.py"),
        ],
        _ => return None,
    };
    for (keywords, subcmd) in rules {
        for kw in *keywords {
            if tl.contains(kw) {
                return Some(subcmd.to_string());
            }
        }
    }
    None
}

fn fix_subcommand_for_tool(args: &mut Vec<String>, tool: &str, task: &str) {
    let all_known_subcmds: &[(&str, &[&str])] = &[
        ("gatk", &["HaplotypeCaller", "MarkDuplicates", "Mutect2", "FilterMutectCalls", "CreateSequenceDictionary", "AddOrReplaceReadGroups", "BaseRecalibrator", "ApplyBQSR", "SelectVariants"]),
        ("picard", &["SortSam", "MarkDuplicates", "AddOrReplaceReadGroups", "CollectAlignmentSummaryMetrics", "CollectInsertSizeMetrics", "ValidateSamFile"]),
        ("bcftools", &["view", "mpileup", "merge", "norm", "stats", "annotate", "isec", "query", "call", "filter", "sort", "index", "concat", "roh"]),
        ("blast", &["blastn", "blastp", "blastx", "tblastn", "makeblastdb", "blastdbcmd", "blastdb_aliastool"]),
        ("bismark", &["bismark_genome_preparation", "deduplicate_bismark", "bismark_methylation_extractor", "bismark2report"]),
        ("hmmer", &["hmmsearch", "hmmscan", "hmmbuild", "hmmpress", "phmmer", "hmmalign"]),
        ("samtools", &["view", "sort", "index", "flagstat", "fastq", "markdup", "merge", "depth", "stats", "faidx", "dict", "idxstats", "collate", "fixmate", "calmd", "addreplacerg"]),
        ("bedtools", &["intersect", "genomecov", "subtract", "merge", "closest", "getfasta", "makewindows", "coverage", "slop", "shift", "flank", "sort", "bamtofastq", "complement", "window", "cluster", "groupby", "expand", "split", "map", "jaccard", "reldist", "random", "shuffle", "annotate", "multiinter", "unionbedg", "pairtobed", "pairtopair", "bamtofastq", "bedtobam", "bedpetobam", "bamtobed"]),
        ("sourmash", &["sketch", "compare", "gather", "taxonomy", "search", "index", "categorize", "watch", "plot", "sig", "lca"]),
        ("sra-tools", &["fasterq-dump", "prefetch", "vdb-validate", "sra-stat", "fastq-dump", "sam-dump"]),
        ("varscan2", &["mpileup2snp", "mpileup2indel", "somatic", "processSomatic", "copycaller"]),
        ("delly", &["call", "filter", "merge", "lr", "cnv"]),
        ("mmseqs2", &["easy-search", "easy-cluster", "easy-linclust", "createdb", "search", "result2repseq", "convertalis", "linclust", "cluster"]),
        ("bracken", &["bracken-build", "combine_bracken_outputs"]),
        ("diamond", &["makedb", "blastp", "blastx", "cluster", "linclust", "realign", "view"]),
        ("deeptools", &["bamCoverage", "bamCompare", "computeMatrix", "plotHeatmap", "multiBamSummary", "plotFingerprint", "plotCoverage", "plotProfile", "bamPairwiseBias", "estimateReadFiltering", "alignmentSieve", "computeGCBias", "correctGCBias", "plotCorrelation", "plotPCA", "plotEnrichment"]),
        ("cnvkit", &["batch", "scatter", "call", "segment", "heatmap", "genemetrics", "access", "coverage", "reference", "fix", "diagram"]),
        ("salmon", &["index", "quant", "swim", "partial", "validate"]),
        ("kallisto", &["index", "quant", "bus", "h5dump", "merge"]),
        ("rsem", &["rsem-prepare-reference", "rsem-calculate-expression", "rsem-generate-data-matrix"]),
        ("mummer", &["nucmer", "dnadiff", "delta-filter", "show-coords", "mummerplot", "show-tiling", "promer"]),
        ("igvtools", &["count", "index", "sort", "toTDF", "formatexp"]),
        ("homer", &["makeTagDirectory", "findPeaks", "annotatePeaks.pl", "findMotifsGenome.pl", "mergePeaks", "pos2bed.pl", "makeUCSCfile", "getDifferentialPeaksReplicates.pl"]),
        ("gtdbtk", &["classify_wf", "identify", "de_novo_wf", "align", "classify"]),
        ("checkm2", &["predict", "database", "testrun"]),
        ("qualimap", &["bamqc", "rnaseq", "multi-bamqc", "counts"]),
        ("macs2", &["callpeak", "predictd", "bdgcmp", "bdgdiff", "filterdup", "pileup"]),
        ("mash", &["sketch", "dist", "screen", "triangle", "paste", "info"]),
        ("seqkit", &["stats", "seq", "grep", "sample", "fq2fa", "split2", "subseq", "translate", "replace", "rmdup", "sort", "concat", "locate", "bam"]),
        ("seqtk", &["sample", "seq", "subseq", "trimfq", "comp", "mergefa", "mergepe", "dropse"]),
        ("snpeff", &["ann", "build", "download", "databases"]),
        ("survivor", &["merge", "stats", "filter", "simSV", "ls"]),
        ("whatshap", &["phase", "haplotag", "stats", "compare"]),
        ("modkit", &["pileup", "extract", "summary", "motif-bed", "sample-probs"]),
        ("pairtools", &["parse", "sort", "dedup", "cload", "flip", "merge", "select", "restrict", "split", "scale"]),
        ("nextflow", &["run", "pull", "list", "clean", "info", "log"]),
        ("strelka2", &["configureStrelkaGermlineWorkflow.py", "configureStrelkaSomaticWorkflow.py"]),
        ("bbtools", &["bbduk.sh", "bbmap.sh", "bbmerge.sh", "reformat.sh", "dedupe.sh", "bbsplit.sh", "tadpole.sh", "tadshrink.sh"]),
        ("agat", &["agat_convert_sp_gff2gtf", "agat_sp_statistics", "agat_sp_filter_gene_by_length", "agat_convert_sp_gxf2gxf", "agat_sp_extract_sequences", "agat_sp_keep_longest_isoform", "agat_sp_merge_annotations", "agat_sp_manage_IDs", "agat_convert_sp_gff2bed"]),
        ("bamtools", &["stats", "count", "filter", "merge", "split", "convert", "index", "coverage", "header", "random", "resolve", "sort", "subtract", "validate"]),
        ("medaka", &["medaka_consensus", "medaka_variant", "medaka_haploid_variant"]),
        ("truvari", &["bench", "collapse", "refine"]),
        ("pbmm2", &["align", "index"]),
        ("pbsv", &["discover", "call"]),
        ("kb", &["ref", "count"]),
        ("centrifuge", &["centrifuge-build", "centrifuge-kreport"]),
        ("kraken2", &["kraken2-build"]),
        ("metabat2", &["jgi_summarize_bam_contig_depths"]),
        ("quast", &["metaquast.py"]),
        ("git", &["clone", "checkout", "commit", "push", "pull", "log", "branch", "merge", "fetch", "status", "diff", "add", "stash", "tag", "reset", "rebase", "init", "remote"]),
        ("vcftools", &["--freq", "--hardy", "--het", "--site-pi", "--TajimaD", "--window-pi", "--remove-indels", "--maf", "--max-missing", "--recode", "--keep", "--remove", "--thin", "--max-alleles", "--min-alleles", "--minDP"]),
        ("flye", &["--nano-raw", "--nano-corr", "--pacbio-raw", "--pacbio-corr", "--pacbio-hifi"]),
        ("hifiasm", &["--h1", "--h2", "--n-hap", "-l0", "--hifi"]),
        ("minimap2", &["-ax map-ont", "-ax map-pb", "-ax map-hifi", "-ax splice", "-d"]),
        ("star", &["--runMode", "genomeGenerate", "alignReads"]),
        ("bowtie2", &["bowtie2-build", "bowtie2-inspect"]),
        ("bwa", &["mem", "index", "aln", "sampe", "samse", "bwasw"]),
        ("bwa-mem2", &["mem", "index"]),
        ("wget", &["-O", "-c", "-b", "-q", "-r", "-np", "-nd"]),
        ("curl", &["-o", "-O", "-L", "-s", "-T", "-X", "-d", "-H"]),
        ("ssh", &["-i", "-p", "-L", "-R", "-N", "-f"]),
        ("rsync", &["-avz", "-a", "--delete", "-v", "-z", "-r", "-n"]),
        ("find", &["-name", "-type", "-size", "-mtime", "-exec", "-perm", "-user", "-group"]),
        ("rm", &["-r", "-rf", "-f", "-v", "-i", "-d"]),
        ("tar", &["-czf", "-xzf", "-tf", "-cjf", "-xjf", "-czf", "-xf"]),
        ("grep", &["-r", "-i", "-n", "-c", "-v", "-C", "--include", "-l", "-w", "-E"]),
        ("sed", &["-i", "-e", "-n", "s/", "d"]),
        ("awk", &["-F", "-f", "-v"]),
        ("pbccs", &["--min-passes", "--hifi-kinetics", "--min-rq", "--report-file"]),
        ("verkko", &["--hifi", "--ont", "--trio", "-d"]),
        ("plink2", &["--pca", "--assoc", "--make-bed", "--freq", "--hardy", "--mind", "--geno", "--maf", "--hwe", "--bfile", "--vcf", "--out"]),
        ("cutadapt", &["-a", "-g", "-A", "-G", "-e", "-q", "-m", "-M", "-o"]),
        ("eggnog-mapper", &["emapper.py"]),
        ("metaphlan", &["merge_metaphlan_tables.py", "strainphlan"]),
        ("freebayes", &["-f", "-p", "-C", "--min-alternate-count", "--min-alternate-fraction"]),
        ("longshot", &["-F", "-f", "-e", "--min_cov", "--strand_bias_pvalue"]),
        ("sniffles", &["--min_support", "--min_length", "--genotype", "-m"]),
        ("featurecounts", &["-a", "-o", "-T", "-p", "-s", "-t", "-g", "-B", "-C"]),
        ("stringtie", &["--merge", "-e", "-G", "-o", "-A", "-B"]),
        ("trim_galore", &["--paired", "--quality", "--length", "--gzip", "--fastqc", "-o"]),
        ("fastp", &["-i", "-I", "-o", "-O", "-w", "--detect_adapter_for_pe", "--qualified_quality_phred", "--length_required"]),
        ("fastqc", &["-o", "-t", "-n", "--noextract", "--casava", "--nogroup"]),
        ("fastq-screen", &["--conf", "--aligner", "--outdir", "--subset", "--paired"]),
        ("canu", &["-genome", "-p", "-d", "cor", "corMhap", "obt", "utg", "trim"]),
        ("miniasm", &["-f", "-m", "-s", "-c"]),
        ("racon", &["-m", "-x", "-g", "-c", "-q", "-t", "-u"]),
        ("megahit", &["-1", "-2", "-r", "-o", "--min-count", "--k-list", "--presets"]),
        ("prokka", &["--outdir", "--prefix", "--kingdom", "--genus", "--species", "--locustag", "--addgenes", "--usegenus"]),
        ("prodigal", &["-a", "-d", "-f", "-g", "-i", "-m", "-n", "-o", "-p", "-s", "-t"]),
        ("augustus", &["--species", "--gff3", "--protein", "--codingseq", "--outfile", "--AUGUSTUS_CONFIG_PATH"]),
        ("bakta", &["--db", "--output", "--prefix", "--threads", "--genus", "--species", "--strain"]),
        ("arriba", &["-x", "-o", "-g", "-a", "-b"]),
        ("pbfusion", &["-i", "-o", "-g", "-r"]),
        ("nanocomp", &["--outdir", "-o", "--plot", "--raw", "-t"]),
        ("chopper", &["-q", "--min_length", "--max_length", "--headcrop", "--tailcrop", "-i", "-o"]),
        ("liftoff", &["-g", "-o", "-u", "-s", "-a", "-copies", "-flank"]),
        ("cellsnp-lite", &["-s", "-O", "-R", "--minMAF", "--minCOUNT", "-b", "--gzip", "-p"]),
        ("vcfanno", &["-p", "-l", "-b", "-c"]),
        ("shapeit4", &["--input", "--output", "--region", "--thread", "--log"]),
        ("orthofinder", &["-f", "-b", "-t", "-a", "-S", "-M", "-A"]),
    ];

    let known_subcmds_for_tool: Option<&[&str]> = all_known_subcmds.iter()
        .find(|(t, _)| *t == tool)
        .map(|(_, subcmds)| *subcmds);

    if let Some(correct_subcmd) = extract_subcommand_from_task(tool, task) {
        if args.is_empty() {
            args.insert(0, correct_subcmd);
            return;
        }
        let correct_lower = correct_subcmd.to_ascii_lowercase();
        if let Some(known_subcmds) = known_subcmds_for_tool {
            let mut i = 0;
            while i < args.len() {
                let arg_lower = args[i].to_ascii_lowercase();
                if arg_lower == correct_lower {
                    i += 1;
                    continue;
                }
                let is_known = known_subcmds.iter().any(|s| s.eq_ignore_ascii_case(&args[i]));
                if is_known && !args[i].starts_with('-') {
                    args.remove(i);
                } else {
                    i += 1;
                }
            }
        }
        let first_lower = args[0].to_ascii_lowercase();
        if first_lower == correct_lower {
            args[0] = correct_subcmd;
            return;
        }
        if args[0].starts_with('-') {
            args.insert(0, correct_subcmd);
            return;
        }
        let is_known_prefix = first_lower == "rscript"
            || first_lower == "perl"
            || first_lower == "python"
            || first_lower == "python3"
            || first_lower == "bash"
            || first_lower == "java"
            || first_lower == "julia";
        if is_known_prefix {
            args.insert(0, correct_subcmd);
            return;
        }
        let is_wrong_subcmd = !args[0].contains('.') && !args[0].contains('/') && !args[0].contains("://");
        if is_wrong_subcmd {
            args[0] = correct_subcmd;
        } else {
            args.insert(0, correct_subcmd);
        }
    } else {
        if !args.is_empty() {
            let first_lower = args[0].to_ascii_lowercase();
            let companion_binaries: &[(&str, &[&str])] = &[
                ("bismark", &["bismark_genome_preparation", "deduplicate_bismark", "bismark_methylation_extractor", "bismark2report"]),
                ("bowtie2", &["bowtie2-build", "bowtie2-inspect"]),
                ("hisat2", &["hisat2-build", "hisat2-inspect"]),
                ("kraken2", &["kraken2-build"]),
                ("bracken", &["bracken-build", "combine_bracken_outputs"]),
                ("centrifuge", &["centrifuge-build", "centrifuge-kreport"]),
                ("medaka", &["medaka_consensus", "medaka_variant", "medaka_haploid_variant"]),
                ("rsem", &["rsem-prepare-reference", "rsem-calculate-expression", "rsem-generate-data-matrix"]),
                ("strelka2", &["configureStrelkaGermlineWorkflow.py", "configureStrelkaSomaticWorkflow.py"]),
                ("bbtools", &["bbduk.sh", "bbmap.sh", "bbmerge.sh", "reformat.sh", "dedupe.sh", "bbsplit.sh"]),
                ("metabat2", &["jgi_summarize_bam_contig_depths"]),
                ("quast", &["metaquast.py"]),
                ("homer", &["makeTagDirectory", "findPeaks", "annotatePeaks.pl", "findMotifsGenome.pl", "mergePeaks", "pos2bed.pl", "makeUCSCfile", "getDifferentialPeaksReplicates.pl"]),
                ("agat", &["agat_convert_sp_gff2gtf", "agat_sp_statistics", "agat_sp_filter_gene_by_length", "agat_convert_sp_gxf2gxf", "agat_sp_extract_sequences", "agat_sp_keep_longest_isoform", "agat_sp_merge_annotations", "agat_sp_manage_IDs", "agat_convert_sp_gff2bed"]),
                ("gtdbtk", &["classify_wf", "de_novo_wf", "identify", "align", "classify"]),
                ("eggnog-mapper", &["emapper.py"]),
                ("metaphlan", &["merge_metaphlan_tables.py", "strainphlan"]),
                ("sra-tools", &["fasterq-dump", "fastq-dump", "prefetch", "sam-dump", "vdb-validate", "sra-stat"]),
                ("mummer", &["nucmer", "dnadiff", "delta-filter", "show-coords", "mummerplot", "show-tiling", "promer"]),
                ("igvtools", &["count", "index", "sort", "toTDF", "formatexp"]),
                ("star", &["genomeGenerate", "alignReads"]),
                ("bowtie2", &["bowtie2-build", "bowtie2-inspect"]),
                ("bwa", &["mem", "index", "aln", "sampe", "samse"]),
                ("bwa-mem2", &["mem", "index"]),
                ("rsem", &["rsem-prepare-reference", "rsem-calculate-expression", "rsem-generate-data-matrix"]),
                ("homer", &["makeTagDirectory", "findPeaks", "annotatePeaks.pl", "findMotifsGenome.pl", "mergePeaks", "pos2bed.pl", "makeUCSCfile", "getDifferentialPeaksReplicates.pl"]),
                ("agat", &["agat_convert_sp_gff2gtf", "agat_sp_statistics", "agat_sp_filter_gene_by_length", "agat_convert_sp_gxf2gxf", "agat_sp_extract_sequences", "agat_sp_keep_longest_isoform", "agat_sp_merge_annotations", "agat_sp_manage_IDs", "agat_convert_sp_gff2bed"]),
                ("gtdbtk", &["classify_wf", "de_novo_wf", "identify", "align", "classify"]),
            ];
            for (t, companions) in companion_binaries {
                if tool == *t {
                    for comp in *companions {
                        if first_lower == comp.to_ascii_lowercase() {
                            let comp_kw = get_subcmd_keywords(comp);
                            let task_lower = task.to_ascii_lowercase();
                            let matches_task = comp_kw.iter().any(|kw| task_lower.contains(kw));
                            if !matches_task {
                                args.remove(0);
                            }
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn get_subcmd_keywords(subcmd: &str) -> &[&str] {
    match subcmd.to_ascii_lowercase().as_str() {
        "haplotypecaller" => &["haplotypecaller", "germline variant"],
        "markduplicates" => &["markduplicate", "mark duplicate", "pcr duplicate"],
        "mutect2" => &["mutect2", "somatic mutat"],
        "filtermutectcalls" => &["filtermutectcalls", "filter mutect"],
        "createsequencedictionary" => &["sequence dictionary"],
        "addorreplacereadgroups" => &["add or replace read group", "read group"],
        "baserecalibrator" => &["base recalibrat", "bqsr step 1"],
        "applybqsr" => &["apply bqsr", "bqsr step 2"],
        "selectvariants" => &["select variant"],
        "sortsam" => &["sort sam", "sort bam"],
        "collectalignmentsummarymetrics" => &["alignment summary", "alignment metric"],
        "collectinsertsizemetrics" => &["insert size", "insert metric"],
        "validatesamfile" => &["validate sam"],
        "view" => &["view", "filter vcf", "extract sample", "convert bam"],
        "mpileup" => &["mpileup", "pileup"],
        "merge" => &["merge"],
        "norm" => &["normalize", "split multi-allelic"],
        "stats" => &["statistic", "compute stat"],
        "annotate" => &["annotat vcf", "add id field"],
        "isec" => &["intersection", "shared between"],
        "query" => &["extract custom field", "extract field"],
        "call" => &["call variant", "call sv"],
        "filter" => &["filter vcf", "filter bam", "filter sv"],
        "blastn" => &["blastn", "nucleotide blast", "nucleotide sequence"],
        "blastp" => &["blastp", "protein blast", "protein sequence"],
        "blastx" => &["blastx", "translate nucleotide"],
        "makeblastdb" => &["makeblastdb", "build database", "blast database"],
        "blastdbcmd" => &["blastdbcmd", "retrieve sequence"],
        "bismark_genome_preparation" => &["genome preparation", "prepare genome", "build genome index", "bisulfite genome"],
        "deduplicate_bismark" => &["deduplicate", "remove duplicate"],
        "bismark_methylation_extractor" => &["methylation extract", "extract methylation"],
        "bismark2report" => &["bismark2report", "summary report"],
        "hmmsearch" => &["hmmsearch", "search profile against sequence"],
        "hmmscan" => &["hmmscan", "scan sequence against profile"],
        "hmmbuild" => &["hmmbuild", "build profile", "build hmm"],
        "hmmpress" => &["hmmpress", "press hmm"],
        "phmmer" => &["phmmer"],
        "hmmalign" => &["hmmalign", "align sequence to profile"],
        "sort" => &["sort bam", "sort by coordinate"],
        "index" => &["index bam", "create index"],
        "flagstat" => &["flagstat", "flag statistic"],
        "fastq" => &["bam to fastq", "convert bam to fastq"],
        "markdup" => &["markdup", "mark duplicate in bam"],
        "depth" => &["compute depth", "coverage depth"],
        "intersect" => &["intersect", "find overlap"],
        "genomecov" => &["genomecov", "genome coverage"],
        "subtract" => &["subtract", "remove overlap"],
        "getfasta" => &["getfasta", "extract sequence from bed"],
        "makewindows" => &["makewindows", "create window"],
        "coverage" => &["coverage per feature"],
        "sketch" => &["sketch", "create sketch"],
        "compare" => &["compare signature", "compare sketch"],
        "gather" => &["gather", "metagenomic gather"],
        "taxonomy" => &["taxonomy", "classify taxonom"],
        "search" => &["search", "find similar"],
        "fasterq-dump" => &["fasterq", "download fastq", "convert sra"],
        "prefetch" => &["prefetch", "download sra"],
        "vdb-validate" => &["validate sra"],
        "sra-stat" => &["sra-stat", "statistic sra"],
        "mpileup2snp" => &["mpileup2snp", "snp from pileup"],
        "mpileup2indel" => &["mpileup2indel", "indel from pileup"],
        "somatic" => &["somatic variant", "somatic call", "tumor-normal"],
        "processsomatic" => &["processsomatic", "process somatic", "filter somatic"],
        "lr" => &["long-read sv", "long read structural", "pacbio sv", "ont sv"],
        "cnv" => &["copy number variant", "cnv call"],
        "easy-search" => &["easy-search", "easy search"],
        "easy-cluster" => &["easy-cluster", "easy cluster"],
        "easy-linclust" => &["easy-linclust", "easy linclust"],
        "createdb" => &["createdb", "create database"],
        "result2repseq" => &["result2repseq", "representative sequence"],
        "bracken-build" => &["bracken-build", "build bracken"],
        "combine_bracken_outputs" => &["combine_bracken_outputs", "combine bracken"],
        "makedb" => &["makedb", "make database", "build database"],
        "bamcoverage" => &["bamcoverage", "bam coverage"],
        "bamcompare" => &["bamcompare", "bam compare"],
        "computematrix" => &["computematrix", "compute matrix"],
        "plotheatmap" => &["plotheatmap", "plot heatmap"],
        "multibamsummary" => &["multibamsummary", "multi bam summary"],
        "plotfingerprint" => &["plotfingerprint", "plot fingerprint"],
        "batch" => &["batch", "cnvkit batch"],
        "scatter" => &["scatter", "plot cnv"],
        "segment" => &["segment", "segment cnv"],
        "heatmap" => &["heatmap", "heatmap cnv"],
        "genemetrics" => &["genemetrics", "gene metric"],
        "index" => &["index", "build index"],
        "quant" => &["quant", "quantify", "expression quant"],
        "bus" => &["bus", "bus format"],
        "rsem-prepare-reference" => &["rsem-prepare-reference", "prepare reference"],
        "rsem-calculate-expression" => &["rsem-calculate-expression", "calculate expression"],
        "rsem-generate-data-matrix" => &["rsem-generate-data-matrix", "data matrix"],
        "nucmer" => &["nucmer", "nucleotide align"],
        "dnadiff" => &["dnadiff", "dna diff"],
        "delta-filter" => &["delta-filter", "filter delta"],
        "show-coords" => &["show-coords", "show coordinate"],
        "mummerplot" => &["mummerplot", "plot mummer"],
        "show-tiling" => &["show-tiling", "tiling"],
        "count" => &["count", "coverage count"],
        "totdf" => &["totdf", "to tdf"],
        "maketagdirectory" => &["maketagdirectory", "tag directory"],
        "findpeaks" => &["findpeaks", "find peak", "peak calling"],
        "annotatepeaks.pl" => &["annotatepeaks", "annotate peak"],
        "findmotifsgenome.pl" => &["findmotifsgenome", "find motif"],
        "classify_wf" => &["classify_wf", "classify workflow", "taxonomic"],
        "identify" => &["identify marker", "gtdb identify"],
        "de_novo_wf" => &["de_novo_wf", "de novo workflow"],
        "align" => &["align", "pbmm2 align", "pacbio align"],
        "predict" => &["predict", "checkm predict", "completeness"],
        "database" => &["database", "checkm database"],
        "bamqc" => &["bamqc", "bam qc"],
        "rnaseq" => &["rnaseq", "rna-seq qc"],
        "callpeak" => &["callpeak", "peak calling", "call peak"],
        "predictd" => &["predictd", "predict fragment"],
        "dist" => &["dist", "mash dist", "distance"],
        "screen" => &["screen", "mash screen"],
        "triangle" => &["triangle", "mash triangle"],
        "paste" => &["paste", "mash paste"],
        "info" => &["info", "mash info"],
        "stats" => &["stats", "seqkit stat"],
        "seq" => &["seq", "seqkit seq", "seqtk seq"],
        "grep" => &["grep", "seqkit grep"],
        "sample" => &["sample", "seqkit sample", "seqtk sample"],
        "fq2fa" => &["fq2fa", "fastq to fasta"],
        "split2" => &["split2", "seqkit split"],
        "subseq" => &["subseq", "seqtk subseq"],
        "trimfq" => &["trimfq", "seqtk trim"],
        "ann" => &["ann", "snpeff annotat", "annotate vcf"],
        "build" => &["build", "snpeff build"],
        "simsv" => &["simsv", "simulate sv"],
        "phase" => &["phase", "whatshap phase", "phasing"],
        "haplotag" => &["haplotag", "whatshap haplotag"],
        "pileup" => &["pileup", "modkit pileup"],
        "extract" => &["extract", "modkit extract", "methyldackel extract"],
        "summary" => &["summary", "modkit summary"],
        "motif-bed" => &["motif-bed", "modkit motif"],
        "parse" => &["parse", "pairtools parse"],
        "dedup" => &["dedup", "pairtools dedup"],
        "cload" => &["cload", "pairtools cload"],
        "run" => &["run", "nextflow run"],
        "pull" => &["pull", "nextflow pull"],
        "configurestrelkagermlineworkflow.py" => &["germline workflow", "germline variant"],
        "configurestrelkasomaticworkflow.py" => &["somatic workflow", "somatic variant"],
        "bbduk.sh" => &["bbduk", "quality filter", "adapter"],
        "bbmap.sh" => &["bbmap", "align read"],
        "bbmerge.sh" => &["bbmerge", "merge read"],
        "reformat.sh" => &["reformat", "convert format"],
        "dedupe.sh" => &["dedupe", "remove duplicate"],
        "bbsplit.sh" => &["bbsplit", "separate by organism"],
        "jgi_summarize_bam_contig_depths" => &["jgi", "depth file", "contig depth"],
        "metaquast.py" => &["metaquast", "meta quast"],
        "agat_convert_sp_gff2gtf" => &["gff to gtf"],
        "agat_sp_statistics" => &["gff statistic", "annotation statistic"],
        "agat_sp_filter_gene_by_length" => &["filter gene by length"],
        "agat_convert_sp_gxf2gxf" => &["fix gff", "standardize gff"],
        "agat_sp_extract_sequences" => &["extract sequence from gff"],
        "agat_sp_keep_longest_isoform" => &["longest isoform"],
        "agat_sp_merge_annotations" => &["merge annotation", "merge gff"],
        "agat_sp_manage_ids" => &["manage id", "fix id"],
        "agat_convert_sp_gff2bed" => &["gff to bed"],
        "medaka_consensus" => &["consensus"],
        "medaka_variant" => &["variant call", "medaka variant"],
        "medaka_haploid_variant" => &["haploid variant"],
        "centrifuge-build" => &["build centrifuge", "centrifuge database"],
        "centrifuge-kreport" => &["kreport"],
        "kraken2-build" => &["kraken2-build", "build kraken"],
        "discover" => &["discover", "pbsv discover", "discover sv"],
        "ref" => &["ref", "kb ref", "build reference"],
        "count" => &["count", "kb count", "quantify"],
        _ => &[],
    }
}

fn extract_grep_pattern(task: &str) -> Option<String> {
    let task_lower = task.to_ascii_lowercase();
    if let Some(start) = task.find('"') {
        if let Some(end) = task.rfind('"') {
            if start < end {
                let pattern = task[start..=end].to_string();
                if pattern.len() > 2 {
                    return Some(pattern);
                }
            }
        }
    }
    if let Some(start) = task.find('\'') {
        if let Some(end) = task.rfind('\'') {
            if start < end {
                let pattern = task[start..=end].to_string();
                if pattern.len() > 2 {
                    return Some(pattern);
                }
            }
        }
    }
    let pattern_keywords = [
        ("搜索", "search for"), ("查找", "find"), ("匹配", "match"),
        ("包含", "contain"), ("关键字", "keyword"), ("模式", "pattern"),
    ];
    for (cn, _en) in &pattern_keywords {
        if let Some(pos) = task_lower.find(cn) {
            let after = &task[pos + cn.len()..];
            let trimmed = after.trim_start_matches(|c: char| c == ' ' || c == ':' || c == '：');
            if let Some(word) = trimmed.split_whitespace().next() {
                if word.len() > 0 && !word.starts_with('(') && !word.starts_with('（') {
                    return Some(format!("\"{}\"", word));
                }
            }
        }
    }
    None
}

fn strip_spurious_subcommand_for_no_subcommand_tools(args: &mut Vec<String>, tool: &str) {
    if !super::task_values::is_no_subcommand_tool(tool) {
        return;
    }
    let known_subcommands = get_known_subcommands_for_tool(tool);
    let known_prefixes: &[&str] = &[
        "rscript", "perl", "python", "python3", "bash", "java", "julia",
        "bowtie2-build", "bowtie2-inspect", "hisat2-build",
        "bismark_genome_preparation", "deduplicate_bismark",
        "bismark_methylation_extractor", "bismark2report",
        "medaka_consensus", "medaka_variant", "medaka_haploid_variant",
        "kraken2-build", "bracken-build", "centrifuge-build", "centrifuge-kreport",
        "rsem-calculate-expression", "rsem-prepare-reference",
        "emapper.py", "merge_metaphlan_tables.py", "strainphlan",
        "jgi_summarize_bam_contig_depths", "makeblastdb",
        "blastn", "blastp", "blastx", "tblastn", "blastdbcmd",
        "combine_bracken_outputs", "convert_fusions_to_vcf",
        "run_arriba", "run_arriba_on_prealigned_bam", "draw_fusions.r",
        "agat_convert_sp_gff2gtf", "agat_sp_statistics",
        "agat_sp_filter_gene_by_length", "agat_convert_sp_gxf2gxf",
        "agat_sp_extract_sequences", "agat_sp_keep_longest_isoform",
        "agat_sp_merge_annotations", "agat_sp_manage_ids",
        "agat_convert_sp_gff2bed",
        "bedmap", "bedextract", "sort-bed", "starch", "unstarch",
        "hmmscan", "hmmsearch", "hmmbuild", "hmmalign", "hmmpress",
        "phmmer", "jackhmmer", "nhmmer", "nhmmscan",
        "fasterq-dump", "prefetch", "fastq-dump", "vdb-validate", "sra-stat",
        "ccs",
        "bbduk.sh", "bbmap.sh", "bbmerge.sh", "reformat.sh", "dedupe.sh", "bbsplit.sh",
        "homer", "makeTagDirectory", "findPeaks", "annotatePeaks.pl",
        "findMotifsGenome.pl", "mergePeaks", "pos2bed.pl", "makeUCSCfile",
        "convert2bed",
        "sketch", "compare", "gather", "taxonomy", "search", "index",
        "categorize", "watch", "plot", "sig", "lca",
        "mem", "aln", "sampe", "samse", "bwasw",
        "quant", "swim", "partial", "validate",
        "bus", "h5dump",
        "callpeak", "predictd", "bdgcmp", "bdgdiff", "filterdup", "pileup",
        "extract", "summary", "motif-bed", "sample-probs",
        "parse", "dedup", "cload", "flip", "select", "restrict", "scale",
        "run", "pull", "list", "clean", "info", "log",
        "bench", "collapse", "refine",
        "align", "discover",
        "ref", "count",
        "makedb", "cluster", "linclust", "realign", "view",
        "bamCoverage", "bamCompare", "computeMatrix", "plotHeatmap",
        "multiBamSummary", "plotFingerprint", "plotCoverage", "plotProfile",
        "batch", "scatter", "segment", "heatmap", "genemetrics",
        "access", "coverage", "reference", "fix", "diagram",
        "nucmer", "dnadiff", "delta-filter", "show-coords", "mummerplot", "show-tiling", "promer",
        "toTDF", "formatexp",
        "classify_wf", "identify", "de_novo_wf", "classify",
        "predict", "database", "testrun",
        "bamqc", "rnaseq", "multi-bamqc",
        "mpileup2snp", "mpileup2indel", "somatic", "processSomatic",
        "easy-search", "easy-cluster", "easy-linclust", "createdb",
        "result2repseq", "convertalis",
        "configureStrelkaGermlineWorkflow.py", "configureStrelkaSomaticWorkflow.py",
        "tadpole.sh", "tadshrink.sh",
        "stats", "filter", "merge", "convert", "header", "random",
        "resolve", "subtract",
        "medaka_consensus", "medaka_variant", "medaka_haploid_variant",
        "clone", "checkout", "commit", "push", "pull",
        "branch", "fetch", "status", "diff", "add", "stash", "tag", "reset", "rebase", "init", "remote",
        "metaquast.py",
        "emapper.py",
        "merge_metaphlan_tables.py", "strainphlan",
        "fasterq-dump", "fastq-dump", "prefetch", "sam-dump", "vdb-validate", "sra-stat",
        "genomeGenerate", "alignReads",
        "HaplotypeCaller", "MarkDuplicates", "Mutect2", "FilterMutectCalls",
        "CreateSequenceDictionary", "AddOrReplaceReadGroups", "BaseRecalibrator",
        "ApplyBQSR", "SelectVariants",
        "SortSam", "CollectAlignmentSummaryMetrics", "CollectInsertSizeMetrics", "ValidateSamFile",
        "intersect", "genomecov", "subtract", "closest", "getfasta", "makewindows",
        "complement", "window", "groupby", "expand", "map", "jaccard",
        "reldist", "shuffle", "annotate", "multiinter", "unionbedg",
        "pairtobed", "pairtopair", "bedtobam", "bamtobed",
        "ann", "build", "download", "databases",
        "phase", "haplotag",
        "dist", "screen", "triangle", "paste", "info",
        "seq", "grep", "sample", "fq2fa", "split2", "subseq",
        "translate", "replace", "rmdup", "sort", "concat", "locate",
        "trimfq", "comp", "mergefa", "mergepe", "dropse",
    ];
    while !args.is_empty() {
        let first = &args[0];
        let first_lower = first.to_ascii_lowercase();
        if first.starts_with('-') || first.contains('.') || first.contains('/') || first.contains("://") {
            break;
        }
        if first == "." || first == ".." {
            break;
        }
        if known_prefixes.iter().any(|p| *p == first_lower) {
            break;
        }
        if known_subcommands.iter().any(|s| s.eq_ignore_ascii_case(&first_lower)) {
            break;
        }
        args.remove(0);
    }
}

fn get_known_subcommands_for_tool(tool: &str) -> Vec<&'static str> {
    match tool {
        "bismark" => vec!["bismark_genome_preparation", "deduplicate_bismark", "bismark_methylation_extractor", "bismark2report"],
        "bedops" => vec!["--intersect", "--difference", "--merge", "--element-of", "--chop", "bedmap", "bedextract", "sort-bed", "starch", "unstarch"],
        "hmmer" => vec!["hmmscan", "hmmsearch", "hmmbuild", "hmmalign", "hmmpress", "phmmer", "jackhmmer", "nhmmer", "nhmmscan"],
        "sra-tools" => vec!["fasterq-dump", "prefetch", "fastq-dump", "vdb-validate", "sra-stat", "sra-pileup"],
        "delly" => vec!["call", "merge", "filter", "lr", "cnv"],
        "mmseqs2" => vec!["easy-search", "easy-cluster", "easy-linclust", "createdb", "search", "result2repseq"],
        "bracken" => vec!["bracken-build", "combine_bracken_outputs"],
        "arriba" => vec!["draw_fusions.R", "convert_fusions_to_vcf", "run_arriba", "run_arriba_on_prealigned_bam"],
        "meme" => vec!["fimo", "tomtom", "ame", "streme", "meme"],
        "cnvkit" => vec!["batch", "scatter", "call", "segment", "heatmap", "genemetrics"],
        "igvtools" => vec!["count", "index", "sort", "toTDF"],
        "homer" => vec!["makeTagDirectory", "findPeaks", "annotatePeaks.pl", "findMotifsGenome.pl", "mergePeaks"],
        "gtdbtk" => vec!["classify_wf", "identify", "de_novo_wf", "align", "classify"],
        "checkm2" => vec!["predict", "database", "testrun"],
        "qualimap" => vec!["bamqc", "rnaseq", "multi-bamqc"],
        "macs2" => vec!["callpeak", "predictd"],
        "modkit" => vec!["pileup", "extract", "summary", "motif-bed"],
        "pairtools" => vec!["parse", "sort", "dedup", "cload"],
        "nextflow" => vec!["run", "pull", "list", "clean"],
        "strelka2" => vec!["configureStrelkaGermlineWorkflow.py", "configureStrelkaSomaticWorkflow.py"],
        "stringtie" => vec!["--merge", "-e"],
        "bbtools" => vec!["bbduk.sh", "bbmap.sh", "bbmerge.sh", "reformat.sh", "dedupe.sh", "bbsplit.sh"],
        "bamtools" => vec!["stats", "count", "filter", "merge", "split", "convert"],
        "medaka" => vec!["medaka_consensus", "medaka_variant", "medaka_haploid_variant"],
        "pbccs" => vec!["ccs"],
        "pbsv" => vec!["discover", "call"],
        "kb" => vec!["ref", "count"],
        "truvari" => vec!["bench", "collapse", "refine"],
        "survivor" => vec!["merge", "stats", "filter", "simSV"],
        "whatshap" => vec!["phase", "haplotag", "stats"],
        "snpeff" => vec!["ann", "build"],
        "snakemake" => vec!["--cores", "--dry-run"],
        "varscan2" => vec!["mpileup2snp", "mpileup2indel", "somatic", "processSomatic"],
        "mummer" => vec!["nucmer", "dnadiff", "delta-filter", "show-coords", "mummerplot", "show-tiling"],
        "salmon" => vec!["index", "quant"],
        "kallisto" => vec!["index", "quant", "bus"],
        "rsem" => vec!["rsem-prepare-reference", "rsem-calculate-expression", "rsem-generate-data-matrix"],
        "centrifuge" => vec!["centrifuge-build", "centrifuge-kreport"],
        "kraken2" => vec!["kraken2-build"],
        "eggnog-mapper" => vec!["emapper.py"],
        "metaphlan" => vec!["merge_metaphlan_tables.py"],
        "metabat2" => vec!["jgi_summarize_bam_contig_depths"],
        "busco" => vec!["--plot", "--restart", "--list-datasets"],
        "pilon" => vec![],
        "spades" => vec!["--meta", "--plasmid", "--sc", "--isolate", "--rnaviral"],
        "trinity" => vec!["--genome_guided_bam"],
        "trimmomatic" => vec!["PE", "SE"],
        "muscle" => vec!["-super5", "-align"],
        "repeatmasker" => vec!["-species", "-lib", "-noint"],
        "cutadapt" => vec!["-a", "-g", "-A"],
        "orthofinder" => vec!["-f", "-b"],
        "shapeit4" => vec![],
        "fasttree" => vec!["-nt", "-wag", "-lg"],
        "plink2" => vec!["--pca", "--assoc", "--make-bed"],
        "verkko" => vec!["--hifi", "--ont"],
        "pbmm2" => vec!["align", "index"],
        "chromap" => vec![],
        "methyldackel" => vec!["extract", "mbias"],
        _ => vec![],
    }
}

pub fn apply_tool_specific_corrections(args: &[String], tool: &str, task: Option<&str>) -> Vec<String> {
    let tool_lower = tool.to_ascii_lowercase();
    let mut args = args.to_vec();
    let args_lower = args.iter().map(|a| a.to_ascii_lowercase()).collect::<Vec<_>>();
    let args_str_lower = args_lower.join(" ");

    if let Some(task) = task {
        fix_subcommand_for_tool(&mut args, &tool_lower, task);
    }

    strip_spurious_subcommand_for_no_subcommand_tools(&mut args, &tool_lower);

    match tool_lower.as_str() {
        "bowtie2" => {
            let invalid_flags = ["-L", "-b", "--interleaved", "-S"];
            args = remove_specific_flags(&args, &invalid_flags);
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if task_lower.contains("build") || task_lower.contains("index") {
                    let mut new_args = vec!["build".to_string()];
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    }).cloned().unwrap_or_else(|| "reference.fa".to_string());
                    new_args.push(fa);
                    let idx_name = tv.output_files.first().cloned()
                        .unwrap_or_else(|| "reference_index".to_string());
                    new_args.push(idx_name);
                    if task_lower.contains("threads") || task_lower.contains("parallel") {
                        new_args.push("--threads".to_string());
                        new_args.push("8".to_string());
                    }
                    args = new_args;
                } else {
                    if !args.iter().any(|a| a == "-x" || a == "--index") {
                        let idx = tv.reference_files.first().cloned()
                            .or_else(|| tv.input_files.iter().find(|f| {
                                let fl = f.to_ascii_lowercase();
                                fl.contains("index") || fl.contains("genome")
                            }).cloned())
                            .unwrap_or_else(|| "reference_index".to_string());
                        args.insert(0, idx);
                        args.insert(0, "-x".to_string());
                    }
                    if !args.iter().any(|a| a == "-U" || a == "-1" || a == "-2" || a == "--interleaved") {
                        let fq_files: Vec<_> = tv.input_files.iter().filter(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".fq.gz") || fl.ends_with(".fastq.gz")
                        }).collect();
                        if fq_files.len() >= 2 {
                            args.push("-1".to_string());
                            args.push(fq_files[0].clone());
                            args.push("-2".to_string());
                            args.push(fq_files[1].clone());
                        } else if let Some(fq) = fq_files.first() {
                            args.push("-U".to_string());
                            args.push((*fq).clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-p" || a == "--threads") {
                        args.push("-p".to_string());
                        args.push("8".to_string());
                    }
                    if !args_lower.contains(&"--very-sensitive".to_string())
                        && !args_lower.contains(&"--very-sensitive-local".to_string())
                        && !args_lower.contains(&"--fast".to_string())
                        && !args_lower.contains(&"--local".to_string()) {
                        if task_lower.contains("local") || task_lower.contains("soft-clip") {
                            args.push("--local".to_string());
                            args.push("--very-sensitive-local".to_string());
                        } else if task_lower.contains("fast") || task_lower.contains("quick") {
                            args.push("--fast".to_string());
                        } else {
                            args.push("--very-sensitive".to_string());
                        }
                    }
                    if task_lower.contains("no-unal") || task_lower.contains("no unaligned") {
                        args.push("--no-unal".to_string());
                    }
                    if task_lower.contains("rg-id") || task_lower.contains("read group") {
                        args.push("--rg-id".to_string());
                        args.push("sample1".to_string());
                    }
                    if task_lower.contains("un-conc") || task_lower.contains("unaligned concordant") {
                        args.push("--un-conc".to_string());
                        args.push("sample.fq".to_string());
                    }
                    if let Some(out) = tv.output_files.first() {
                        if out.to_ascii_lowercase().ends_with(".sam") {
                            if !args.iter().any(|a| a == "-S") {
                                args.push("-S".to_string());
                                args.push(out.clone());
                            }
                        }
                    }
                }
            }
        }
        "star" => {
            while !args.is_empty() && !args[0].starts_with('-') {
                let first_lower = args[0].to_ascii_lowercase();
                if first_lower == "bam_unsorted" || first_lower == "bam_sortedbycoordinate"
                    || first_lower == "alignreads" || first_lower == "genomegenerate"
                    || first_lower == "bam" || first_lower == "unsorted"
                    || first_lower == "sortedbycoordinate" {
                    args.remove(0);
                } else {
                    break;
                }
            }
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if !args_str_lower.contains("--runmode") && !args_str_lower.contains("--runmode") {
                    if task_lower.contains("genomegenerate") || task_lower.contains("generate genome")
                        || task_lower.contains("genome index") || task_lower.contains("create index")
                        || task_lower.contains("build index") || task_lower.contains("build genome") {
                        args.insert(0, "genomeGenerate".to_string());
                        args.insert(0, "--runMode".to_string());
                    } else {
                        args.insert(0, "alignReads".to_string());
                        args.insert(0, "--runMode".to_string());
                    }
                }
                if !args_str_lower.contains("readfilescommand") {
                    let has_gz = args.iter().any(|a| a.to_ascii_lowercase().ends_with(".gz"));
                    if has_gz {
                        let insert_pos = find_flag_insert_position(&args);
                        args.insert(insert_pos, "zcat".to_string());
                        args.insert(insert_pos, "--readFilesCommand".to_string());
                    }
                }
            }
        }
        "kraken2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if args_str_lower.contains("--confidence 0.0") {
                    let mut new_args = Vec::new();
                    let mut skip = false;
                    for (i, a) in args.iter().enumerate() {
                        if skip { skip = false; continue; }
                        if a.eq_ignore_ascii_case("--confidence") && i + 1 < args.len() && args[i+1] == "0.0" {
                            skip = true;
                            continue;
                        }
                        new_args.push(a.clone());
                    }
                    args = new_args;
                }
                if !args_str_lower.contains("--confidence") && !task_lower.contains("build") {
                    if task_lower.contains("strict") || task_lower.contains("stringen")
                        || task_lower.contains("minimum-hit-groups") {
                        args.push("--confidence".to_string());
                        args.push("0.1".to_string());
                    }
                }
            }
        }
        "bismark" => {
            let invalid_flags = ["--local"];
            args = remove_specific_flags(&args, &invalid_flags);
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if task_lower.contains("prepare") && task_lower.contains("genome") && task_lower.contains("index") {
                    args = vec!["bismark_genome_preparation".to_string()];
                    let genome_dir = extract_task_values(task).genome_dirs.first().cloned()
                        .or_else(|| extract_task_values(task).input_files.first().cloned())
                        .unwrap_or_else(|| "/path/to/genome_directory/".to_string());
                    args.push(genome_dir);
                } else if task_lower.contains("deduplicate") || task_lower.contains("remove duplicate") {
                    let mut new_args = vec!["deduplicate_bismark".to_string()];
                    if task_lower.contains("paired") { new_args.push("--paired".to_string()); }
                    new_args.push("--bam".to_string());
                    let input = extract_task_values(task).input_files.first().cloned()
                        .unwrap_or_else(|| "reads.bam".to_string());
                    new_args.push(input);
                    args = new_args;
                } else if task_lower.contains("methylation") && task_lower.contains("extract") {
                    let mut new_args = vec!["bismark_methylation_extractor".to_string()];
                    if task_lower.contains("paired") { new_args.push("--paired-end".to_string()); }
                    new_args.push("--comprehensive".to_string());
                    new_args.push("--CX_context".to_string());
                    new_args.push("--genome_folder".to_string());
                    let genome_dir = extract_task_values(task).genome_dirs.first().cloned()
                        .unwrap_or_else(|| "/path/to/genome_dir/".to_string());
                    new_args.push(genome_dir);
                    new_args.push("--output_dir".to_string());
                    let output = extract_task_values(task).output_files.first().cloned()
                        .unwrap_or_else(|| "methylation_output/".to_string());
                    new_args.push(output);
                    let input = extract_task_values(task).input_files.first().cloned()
                        .unwrap_or_else(|| "reads.bam".to_string());
                    new_args.push(input);
                    args = new_args;
                } else if task_lower.contains("report") && (task_lower.contains("html") || task_lower.contains("bismark2report")) {
                    let mut new_args = vec!["bismark2report".to_string()];
                    new_args.push("--output_dir".to_string());
                    let output = extract_task_values(task).output_files.first().cloned()
                        .unwrap_or_else(|| "reports/".to_string());
                    new_args.push(output);
                    args = new_args;
                } else if task_lower.contains("rrbs") {
                    if !args_str_lower.contains("--rrbs") {
                        args.push("--rrbs".to_string());
                    }
                } else if task_lower.contains("hisat2") {
                    if !args_str_lower.contains("--hisat2") {
                        args.push("--hisat2".to_string());
                    }
                } else if task_lower.contains("minimap2") || task_lower.contains("nanopore") || task_lower.contains("pacbio") || task_lower.contains("long-read") {
                    if !args_str_lower.contains("--minimap2") {
                        args.push("--minimap2".to_string());
                    }
                } else if task_lower.contains("non-directional") || task_lower.contains("pbat") || task_lower.contains("scbs") {
                    if !args_str_lower.contains("--non_directional") {
                        args.push("--non_directional".to_string());
                    }
                } else if task_lower.contains("slam") || task_lower.contains("slam-seq") {
                    if !args_str_lower.contains("--slam") {
                        args.push("--slam".to_string());
                    }
                }
            }
        }
        "eggnog-mapper" => {
            let invalid_flags = ["--index_chunks"];
            args = remove_specific_flags(&args, &invalid_flags);
        }
        "medaka" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if task_lower.contains("list") && task_lower.contains("model") {
                    args = vec!["tools".to_string(), "list_models".to_string()];
                } else if task_lower.contains("consensus") || (task_lower.contains("polish") && task_lower.contains("assembly")) {
                    let mut new_args = vec!["medaka_consensus".to_string()];
                    new_args.push("-i".to_string());
                    let fq = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".gz") || f.to_ascii_lowercase().ends_with(".fastq") || f.to_ascii_lowercase().ends_with(".fq")).cloned()
                        .unwrap_or_else(|| "input.fastq.gz".to_string());
                    new_args.push(fq);
                    new_args.push("-d".to_string());
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa") || fl.ends_with(".fna")
                    }).cloned().unwrap_or_else(|| "assembly.fasta".to_string());
                    new_args.push(fa);
                    new_args.push("-o".to_string());
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "medaka_output/".to_string());
                    new_args.push(out);
                    new_args.push("-m".to_string());
                    if task_lower.contains("r1041") || task_lower.contains("hac_v4") {
                        new_args.push("r1041_e82_400bps_hac_v4.2.0".to_string());
                    } else {
                        new_args.push("r941_min_hac_g507".to_string());
                    }
                    if task_lower.contains("gpu") { new_args.push("--gpu".to_string()); }
                    if task_lower.contains("low") && task_lower.contains("mem") {
                        new_args.push("--chunk_len".to_string());
                        new_args.push("5000".to_string());
                        new_args.push("--chunk_ovlp".to_string());
                        new_args.push("1000".to_string());
                    }
                    args = new_args;
                } else if task_lower.contains("haploid") && task_lower.contains("variant") {
                    let mut new_args = vec!["medaka_haploid_variant".to_string()];
                    new_args.push("-i".to_string());
                    let fq = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".gz")).cloned()
                        .unwrap_or_else(|| "input.fastq.gz".to_string());
                    new_args.push(fq);
                    new_args.push("-r".to_string());
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).cloned().unwrap_or_else(|| "assembly.fasta".to_string());
                    new_args.push(fa);
                    new_args.push("-o".to_string());
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "medaka_variants/".to_string());
                    new_args.push(out);
                    new_args.push("-m".to_string());
                    new_args.push("r941_min_hac_g507".to_string());
                    args = new_args;
                } else if task_lower.contains("variant") && !task_lower.contains("haploid") {
                    let mut new_args = vec!["medaka_variant".to_string()];
                    new_args.push("-i".to_string());
                    let fq = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".gz")).cloned()
                        .unwrap_or_else(|| "input.fastq.gz".to_string());
                    new_args.push(fq);
                    new_args.push("-r".to_string());
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).cloned().unwrap_or_else(|| "reference.fasta".to_string());
                    new_args.push(fa);
                    new_args.push("-o".to_string());
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "variants/".to_string());
                    new_args.push(out);
                    new_args.push("-m".to_string());
                    new_args.push("r1041_e82_400bps_hac_v4.2.0".to_string());
                    if task_lower.contains("region") || task_lower.contains("target") {
                        let bed = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bed")).cloned()
                            .unwrap_or_else(|| "regions.bed".to_string());
                        new_args.push("--regions".to_string());
                        new_args.push(bed);
                    }
                    args = new_args;
                } else if task_lower.contains("inference") && task_lower.contains("save") && task_lower.contains("feature") {
                    let mut new_args = vec!["medaka".to_string(), "inference".to_string()];
                    new_args.push("--save_features".to_string());
                    new_args.push("--model".to_string());
                    new_args.push("r1041_e82_400bps_hac_v4.2.0".to_string());
                    let bam = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")).cloned()
                        .unwrap_or_else(|| "input.bam".to_string());
                    new_args.push(bam);
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "output.hdf".to_string());
                    new_args.push(out);
                    args = new_args;
                } else if task_lower.contains("inference") && task_lower.contains("chromosome") {
                    let mut new_args = vec!["medaka".to_string(), "inference".to_string()];
                    new_args.push("--regions".to_string());
                    new_args.push("chr1 chr2 chr3".to_string());
                    new_args.push("--model".to_string());
                    new_args.push("r1041_e82_400bps_hac_v4.2.0".to_string());
                    let bam = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")).cloned()
                        .unwrap_or_else(|| "input.bam".to_string());
                    new_args.push(bam);
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "output.hdf".to_string());
                    new_args.push(out);
                    args = new_args;
                } else if task_lower.contains("stitch") || task_lower.contains("sequence") {
                    let mut new_args = vec!["medaka".to_string(), "sequence".to_string()];
                    let hdf = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".hdf") || f.to_ascii_lowercase().ends_with(".hdf5")).cloned()
                        .unwrap_or_else(|| "output.hdf".to_string());
                    new_args.push(hdf);
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).cloned().unwrap_or_else(|| "aligned.fasta".to_string());
                    new_args.push(fa);
                    args = new_args;
                } else if task_lower.contains("vcf") {
                    let mut new_args = vec!["medaka".to_string(), "vcf".to_string()];
                    let hdf = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".hdf") || f.to_ascii_lowercase().ends_with(".hdf5")).cloned()
                        .unwrap_or_else(|| "output.hdf".to_string());
                    new_args.push(hdf);
                    let vcf = tv.output_files.first().cloned().unwrap_or_else(|| "reads.vcf".to_string());
                    new_args.push(vcf);
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).cloned().unwrap_or_else(|| "aligned.fasta".to_string());
                    new_args.push(fa);
                    args = new_args;
                }
            }
        }
        "meme" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if task_lower.contains("fimo") || (task_lower.contains("scan") && task_lower.contains("motif")) || (task_lower.contains("known") && task_lower.contains("tf")) {
                    let mut new_args = vec!["fimo".to_string()];
                    new_args.push("--thresh".to_string());
                    new_args.push("1e-4".to_string());
                    new_args.push("--oc".to_string());
                    let out = tv.output_files.first().cloned()
                        .map(|f| { let p = std::path::Path::new(&f); p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "fimo_output".to_string()) })
                        .unwrap_or_else(|| "fimo_output".to_string());
                    new_args.push(out);
                    let motif_db = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".meme")).cloned()
                        .unwrap_or_else(|| "motif.meme".to_string());
                    new_args.push(motif_db);
                    let fa = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa") || fl.ends_with(".fq")
                    }).cloned().unwrap_or_else(|| "input.fasta".to_string());
                    new_args.push(fa);
                    args = new_args;
                } else if task_lower.contains("tomtom") || (task_lower.contains("compare") && task_lower.contains("motif")) {
                    let mut new_args = vec!["tomtom".to_string()];
                    new_args.push("-oc".to_string());
                    let out = tv.output_files.first().cloned()
                        .map(|f| { let p = std::path::Path::new(&f); p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "tomtom_output".to_string()) })
                        .unwrap_or_else(|| "tomtom_output".to_string());
                    new_args.push(out);
                    let xml = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".xml")).cloned()
                        .unwrap_or_else(|| "meme_output/raw.xml".to_string());
                    new_args.push(xml);
                    let motif_db = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".meme")).cloned()
                        .unwrap_or_else(|| "motif.meme".to_string());
                    new_args.push(motif_db);
                    args = new_args;
                } else if task_lower.contains("ame") || (task_lower.contains("enrichment") && task_lower.contains("motif")) {
                    let mut new_args = vec!["ame".to_string()];
                    new_args.push("--oc".to_string());
                    let out = tv.output_files.first().cloned()
                        .map(|f| { let p = std::path::Path::new(&f); p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "ame_output".to_string()) })
                        .unwrap_or_else(|| "ame_output".to_string());
                    new_args.push(out);
                    new_args.push("--control".to_string());
                    let ctrl = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).nth(1).cloned().unwrap_or_else(|| "control.fasta".to_string());
                    new_args.push(ctrl);
                    let fg = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).cloned().unwrap_or_else(|| "input.fasta".to_string());
                    new_args.push(fg);
                    let motif_db = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".meme")).cloned()
                        .unwrap_or_else(|| "motif.meme".to_string());
                    new_args.push(motif_db);
                    args = new_args;
                } else if task_lower.contains("streme") || (task_lower.contains("short") && task_lower.contains("motif")) {
                    let mut new_args = vec!["streme".to_string()];
                    new_args.push("--oc".to_string());
                    let out = tv.output_files.first().cloned()
                        .map(|f| { let p = std::path::Path::new(&f); p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "streme_output".to_string()) })
                        .unwrap_or_else(|| "streme_output".to_string());
                    new_args.push(out);
                    new_args.push("--dna".to_string());
                    new_args.push("--p".to_string());
                    let fg = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa")
                    }).cloned().unwrap_or_else(|| "input.fasta".to_string());
                    new_args.push(fg);
                    if tv.input_files.len() > 1 {
                        new_args.push("--n".to_string());
                        let bg = tv.input_files.iter().filter(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fasta") || fl.ends_with(".fa")
                        }).nth(1).cloned().unwrap_or_else(|| "control.fasta".to_string());
                        new_args.push(bg);
                    }
                    args = new_args;
                }
            }
        }
        "centrifuge" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if task_lower.contains("centrifuge-build") || task_lower.contains("build") && task_lower.contains("index") {
                    let tv = extract_task_values(task);
                    let mut new_args = vec!["centrifuge-build".to_string()];
                    new_args.push("--taxonomy-tree".to_string());
                    let dmp1 = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".dmp")).cloned()
                        .unwrap_or_else(|| "raw.dmp".to_string());
                    new_args.push(dmp1);
                    new_args.push("--name-table".to_string());
                    let dmp2 = tv.input_files.iter().filter(|f| f.to_ascii_lowercase().ends_with(".dmp")).nth(1).cloned()
                        .unwrap_or_else(|| "sample.dmp".to_string());
                    new_args.push(dmp2);
                    new_args.push("--conversion-table".to_string());
                    let map_file = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".map")).cloned()
                        .unwrap_or_else(|| "results.map".to_string());
                    new_args.push(map_file);
                    let fasta = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fasta") || fl.ends_with(".fa") || fl.ends_with(".fna")
                    }).cloned().unwrap_or_else(|| "report.fasta".to_string());
                    new_args.push(fasta);
                    let db_name = tv.output_files.first().cloned().unwrap_or_else(|| "custom_db".to_string());
                    new_args.push(db_name);
                    args = new_args;
                } else if task_lower.contains("centrifuge-kreport") || task_lower.contains("kraken-style report") || task_lower.contains("pavian") || task_lower.contains("krona") {
                    let tv = extract_task_values(task);
                    let mut new_args = vec!["centrifuge-kreport".to_string()];
                    new_args.push("-x".to_string());
                    let db = tv.database_files.first().cloned()
                        .or_else(|| tv.input_files.iter().find(|f| f.contains("database") || f.contains("db") || f.contains("/databases/")).cloned())
                        .unwrap_or_else(|| "/databases/bv_bacteria".to_string());
                    new_args.push(db);
                    let tsv = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".tsv")).cloned()
                        .unwrap_or_else(|| "variants.tsv".to_string());
                    new_args.push(tsv);
                    args = new_args;
                } else {
                    while !args.is_empty() && !args[0].starts_with('-') && !args[0].contains('/') {
                        let first_lower = args[0].to_ascii_lowercase();
                        if first_lower == "centrifuge" || first_lower == "--min-hitlen" {
                            args.remove(0);
                        } else {
                            break;
                        }
                    }
                    if !args_str_lower.contains("-x") {
                        let tv = extract_task_values(task);
                        let db = tv.database_files.first().cloned()
                            .or_else(|| {
                                if task_lower.contains("bacteria") { Some("/databases/bv_bacteria".to_string()) }
                                else if task_lower.contains("viral") { Some("/databases/viral".to_string()) }
                                else if task_lower.contains("nt") { Some("/databases/nt".to_string()) }
                                else if task_lower.contains("human") || task_lower.contains("hg38") { Some("/databases/hg38".to_string()) }
                                else if task_lower.contains("custom") || task_lower.contains("microbiome") { Some("/databases/custom_microbiome".to_string()) }
                                else { None }
                            })
                            .unwrap_or_else(|| "/databases/bv_bacteria".to_string());
                        args.insert(0, db);
                        args.insert(0, "-x".to_string());
                    }
                    if !args_str_lower.contains("-s") && !args_str_lower.contains("-1") && !args_str_lower.contains("-u") {
                        let tv = extract_task_values(task);
                        let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                        }).collect();
                        if fq_files.len() >= 2 {
                            args.push("-1".to_string());
                            args.push(fq_files[0].clone());
                            args.push("-2".to_string());
                            args.push(fq_files[1].clone());
                        } else if let Some(fq) = fq_files.first() {
                            args.push("-U".to_string());
                            args.push((*fq).clone());
                        }
                    }
                    if !args_str_lower.contains("--min-hitlen") {
                        if task_lower.contains("viral") || task_lower.contains("sensitivity") {
                            args.push("--min-hitlen".to_string());
                            args.push("16".to_string());
                        } else if task_lower.contains("precision") || task_lower.contains("high min") {
                            args.push("--min-hitlen".to_string());
                            args.push("30".to_string());
                        }
                    }
                }
            }
        }
        "snakemake" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if task_lower.contains("dry-run") || task_lower.contains("dry run") {
                    args = vec!["--dry-run".to_string(), "--printshellcmds".to_string()];
                } else if task_lower.contains("dag") || task_lower.contains("dependency graph") {
                    args = vec!["--dag".to_string()];
                } else if task_lower.contains("unlock") {
                    args = vec!["--unlock".to_string()];
                } else if task_lower.contains("singularity") {
                    args = vec!["--use-singularity".to_string(), "--singularity-args".to_string(), "'--bind /scratch'".to_string()];
                } else if task_lower.contains("slurm") || task_lower.contains("cluster") {
                    if task_lower.contains("profile") {
                        args = vec!["--profile".to_string(), "slurm".to_string()];
                    } else {
                        args = vec!["--executor".to_string(), "slurm".to_string(), "--jobs".to_string(), "50".to_string(), "--default-resources".to_string(), "mem_mb=4096 runtime=60".to_string(), "--use-conda".to_string()];
                    }
                } else if task_lower.contains("forcerun") || task_lower.contains("force re-run") {
                    args = vec!["--forcerun".to_string(), "trimming".to_string(), "alignment".to_string()];
                } else if task_lower.contains("configfile") || task_lower.contains("configuration file") {
                    if let Some(config_file) = extract_task_values(task).input_files.first() {
                        args = vec!["--configfile".to_string(), config_file.clone()];
                    } else {
                        args = vec!["--configfile".to_string()];
                    }
                } else if task_lower.contains("rerun-incomplete") || task_lower.contains("incomplete") {
                    args = vec!["--rerun-incomplete".to_string(), "--cores".to_string(), "all".to_string()];
                } else if task_lower.contains("cores") || task_lower.contains("all available") {
                    args = vec!["--cores".to_string(), "all".to_string(), "--use-conda".to_string()];
                }
            }
        }
        "multiqc" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') && !args[0].contains('/') && !args[0].contains('.') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "multiqc" || first_lower == "." || first_lower == "qc" {
                        args.remove(0);
                    } else {
                        break;
                    }
                }
                if !args.is_empty() && !args[0].starts_with('-') && !args[0].contains('/') && !args[0].contains('.') {
                    args.remove(0);
                }
                if args.is_empty() || args[0].starts_with('-') {
                    let dir = if task_lower.contains("current directory") || task_lower.contains("from the current") {
                        ".".to_string()
                    } else if task_lower.contains("specific results") || task_lower.contains("from a specific") {
                        "/path/to/results/".to_string()
                    } else if task_lower.contains("fastqc") && task_lower.contains("trimmomatic") {
                        "fastqc_results/".to_string()
                    } else if !tv.input_files.is_empty() {
                        tv.input_files[0].clone()
                    } else {
                        "results/".to_string()
                    };
                    args.insert(0, dir);
                }
                if task_lower.contains("flat") || task_lower.contains("non-interactive") || task_lower.contains("pdf") {
                    if !args.iter().any(|a| a == "--flat") {
                        args.push("--flat".to_string());
                    }
                }
                if task_lower.contains("json") || task_lower.contains("export data") {
                    if !args.iter().any(|a| a == "--data-format") {
                        args.push("--data-format".to_string());
                        args.push("json".to_string());
                    }
                    if !args.iter().any(|a| a == "--no-report") {
                        args.push("--no-report".to_string());
                    }
                }
                if task_lower.contains("ignore") && !args.iter().any(|a| a == "--ignore") {
                    if let Some(ignore_dir) = tv.input_files.first() {
                        args.push("--ignore".to_string());
                        args.push(ignore_dir.clone());
                    }
                }
                if task_lower.contains("module") || task_lower.contains("only specific module") {
                    if !args.iter().any(|a| a == "-m") {
                        if task_lower.contains("fastqc") {
                            args.push("-m".to_string());
                            args.push("fastqc".to_string());
                        }
                        if task_lower.contains("star") {
                            args.push("-m".to_string());
                            args.push("star".to_string());
                        }
                    }
                }
                if task_lower.contains("exclude") && !args.iter().any(|a| a == "-e") {
                    if task_lower.contains("cutadapt") {
                        args.push("-e".to_string());
                        args.push("cutadapt".to_string());
                    }
                    if task_lower.contains("fastqc") {
                        args.push("-e".to_string());
                        args.push("fastqc".to_string());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    } else {
                        args.push("-o".to_string());
                        args.push("multiqc_report/".to_string());
                    }
                }
                if !args.iter().any(|a| a == "-f") {
                    args.push("-f".to_string());
                }
            }
        }
        "nanocomp" => {
            if !args.is_empty() {
                let first = &args[0];
                if first.to_ascii_lowercase() != "nanocomp" {
                    if !first.starts_with('-') && !first.contains('.') && !first.contains('/') {
                        args.remove(0);
                    }
                    args.insert(0, "NanoComp".to_string());
                }
            }
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !args_str_lower.contains("--fastq") && !args_str_lower.contains("--bam") && !args_str_lower.contains("--summary") {
                    let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".fastq.gz") || fl.ends_with(".fq.gz")
                    }).collect();
                    let bam_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        f.to_ascii_lowercase().ends_with(".bam")
                    }).collect();
                    let txt_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".txt") || fl.ends_with(".tsv")
                    }).collect();
                    if !bam_files.is_empty() {
                        args.push("--bam".to_string());
                        for f in bam_files { args.push(f.clone()); }
                    } else if !fq_files.is_empty() {
                        args.push("--fastq".to_string());
                        for f in fq_files { args.push(f.clone()); }
                    } else if !txt_files.is_empty() {
                        args.push("--summary".to_string());
                        for f in txt_files { args.push(f.clone()); }
                    }
                }
                if !args_str_lower.contains("--names") && !args_str_lower.contains("-n") {
                    let n_files = args.iter().filter(|a| a.contains(".fastq") || a.contains(".fq") || a.contains(".bam") || a.contains(".txt")).count();
                    if n_files > 1 {
                        args.push("--names".to_string());
                        for i in 0..n_files {
                            args.push(format!("Run{}", i + 1));
                        }
                    }
                }
                if !args_str_lower.contains("--outdir") && !args_str_lower.contains("-o") {
                    args.push("--outdir".to_string());
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "nanocomp_out/".to_string());
                    args.push(out);
                }
            }
        }
        "quast" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if task_lower.contains("metaquast") || task_lower.contains("metagenome") {
                    if !args.is_empty() && !args[0].to_ascii_lowercase().starts_with("metaquast") {
                        args.insert(0, "metaquast.py".to_string());
                    }
                }
            }
        }
        "porechop" => {
            while !args.is_empty() && !args[0].starts_with('-') {
                let first_lower = args[0].to_ascii_lowercase();
                if first_lower == "reads" || first_lower == "discard_middle" || first_lower == "fastq"
                    || first_lower == "bam" || first_lower == "trim" || first_lower == "adapter"
                    || first_lower == "demultiplex" || first_lower == "check" || first_lower == "seq"
                    || first_lower == "nanopore" || first_lower == "output" {
                    args.remove(0);
                } else {
                    break;
                }
            }
            if let Some(task) = task {
                let tv = extract_task_values(task);
                if !args_str_lower.contains("-i") {
                    let fq = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }).cloned().unwrap_or_else(|| "input.fastq.gz".to_string());
                    args.push("-i".to_string());
                    args.push(fq);
                }
                let has_b_demux = args.iter().any(|a| a == "-b");
                if !has_b_demux {
                    if !args_str_lower.contains("-o ") && !args_str_lower.contains("-o\t") {
                        let out = tv.output_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                        }).cloned().unwrap_or_else(|| "output.fastq.gz".to_string());
                        args.push("-o".to_string());
                        args.push(out);
                    } else {
                        for i in 0..args.len() {
                            if args[i] == "-o" && i + 1 < args.len() {
                                let out_lower = args[i + 1].to_ascii_lowercase();
                                if out_lower.ends_with(".bam") {
                                    args[i + 1] = args[i + 1].replace(".bam", ".fastq.gz").replace(".BAM", ".fastq.gz");
                                } else if out_lower.ends_with(".sam") {
                                    args[i + 1] = args[i + 1].replace(".sam", ".fastq.gz").replace(".SAM", ".fastq.gz");
                                } else if out_lower == "output.bam" || out_lower == "output_dir/" {
                                    args[i + 1] = "output.fastq.gz".to_string();
                                }
                            }
                        }
                    }
                } else {
                    for i in 0..args.len() {
                        if args[i] == "-b" && i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".fastq") || out_lower.ends_with(".fq") || out_lower.ends_with(".gz") {
                                if !args[i + 1].ends_with('/') {
                                    let path = std::path::Path::new(&args[i + 1]);
                                    if let Some(parent) = path.parent() {
                                        args[i + 1] = format!("{}/", parent.to_string_lossy());
                                    } else {
                                        args[i + 1] = "demultiplexed/".to_string();
                                    }
                                }
                            }
                        }
                    }
                }
                if args_str_lower.contains("discard_middle") && !args.iter().any(|a| a == "--discard_middle") {
                    args.push("--discard_middle".to_string());
                }
            }
        }
        "bedops" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if !args.is_empty() {
                    let first = &args[0];
                    let first_lower = first.to_ascii_lowercase();
                    if first_lower == "bedops" {
                        args.remove(0);
                    } else if first_lower == "bedintersect" {
                        args[0] = "--intersect".to_string();
                    }
                }
                let already_has_op = args.iter().any(|a| a.starts_with("--") && !a.starts_with("--ec") && !a.starts_with("--header") && !a.starts_with("--chrom"));
                let already_has_sub = args.iter().any(|a| {
                    let al = a.to_ascii_lowercase();
                    al == "sort-bed" || al == "starch" || al == "unstarch" || al == "bedmap" || al == "bedextract"
                });
                if !already_has_op && !already_has_sub {
                    if task_lower.contains("intersect") {
                        if !args.is_empty() && !args[0].starts_with('-') { args.remove(0); }
                        args.insert(0, "--intersect".to_string());
                    } else if task_lower.contains("difference") || task_lower.contains("complement") {
                        if !args.is_empty() && !args[0].starts_with('-') { args.remove(0); }
                        args.insert(0, "--difference".to_string());
                    } else if task_lower.contains("merge") {
                        if !args.is_empty() && !args[0].starts_with('-') { args.remove(0); }
                        args.insert(0, "--merge".to_string());
                    } else if task_lower.contains("element-of") || task_lower.contains("subset") {
                        if !args.is_empty() && !args[0].starts_with('-') { args.remove(0); }
                        args.insert(0, "--element-of".to_string());
                        args.insert(1, "1".to_string());
                    } else if task_lower.contains("chop") || task_lower.contains("partition") {
                        if !args.is_empty() && !args[0].starts_with('-') { args.remove(0); }
                        args.insert(0, "--chop".to_string());
                        args.insert(1, "100".to_string());
                    } else if task_lower.contains("map") {
                        if !args.is_empty() && !args[0].starts_with('-') && args[0].to_ascii_lowercase() != "bedmap" { args.remove(0); }
                        args.insert(0, "bedmap".to_string());
                    } else if task_lower.contains("extract") {
                        if !args.is_empty() && !args[0].starts_with('-') && args[0].to_ascii_lowercase() != "bedextract" { args.remove(0); }
                        args.insert(0, "bedextract".to_string());
                    } else if task_lower.contains("sort") {
                        if !args.is_empty() && !args[0].starts_with('-') && args[0].to_ascii_lowercase() != "sort-bed" { args.remove(0); }
                        args.insert(0, "sort-bed".to_string());
                    } else if task_lower.contains("starch") {
                        if !args.is_empty() && !args[0].starts_with('-') && args[0].to_ascii_lowercase() != "starch" { args.remove(0); }
                        args.insert(0, "starch".to_string());
                    }
                }
            }
        }
        "hmmer" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "hmmer" || first_lower == "hmmsearch" || first_lower == "hmmscan" || first_lower == "hmmbuild" || first_lower == "hmmalign" || first_lower == "hmmpress" || first_lower == "phmmer" || first_lower == "jackhmmer" || first_lower == "nhmmer" || first_lower == "nhmmscan" || first_lower == "hmmemit" { args.remove(0); } else { break; }
                }
                let correct_sub = if task_lower.contains("hmmscan") || (task_lower.contains("scan") && (task_lower.contains("profile") || task_lower.contains("pfam"))) || (task_lower.contains("domain") && task_lower.contains("annotat")) { "hmmscan" }
                    else if task_lower.contains("hmmsearch") || (task_lower.contains("search") && (task_lower.contains("sequence") || task_lower.contains("protein") || task_lower.contains("database"))) || task_lower.contains("sensitivity") || task_lower.contains("max") || task_lower.contains("e-value") || task_lower.contains("effective database") { "hmmsearch" }
                    else if task_lower.contains("hmmbuild") || (task_lower.contains("build") && task_lower.contains("profile")) || (task_lower.contains("construct") && task_lower.contains("hmm")) { "hmmbuild" }
                    else if task_lower.contains("hmmalign") || (task_lower.contains("align") && !task_lower.contains("search")) { "hmmalign" }
                    else if task_lower.contains("phmmer") || (task_lower.contains("blast") && task_lower.contains("single")) || (task_lower.contains("protein") && task_lower.contains("query")) { "phmmer" }
                    else if task_lower.contains("jackhmmer") || task_lower.contains("iterative") { "jackhmmer" }
                    else if task_lower.contains("nhmmer") { "nhmmer" }
                    else if task_lower.contains("nhmmscan") { "nhmmscan" }
                    else if task_lower.contains("hmmpress") || (task_lower.contains("press") || task_lower.contains("index")) && task_lower.contains("hmm") { "hmmpress" }
                    else if task_lower.contains("hmmemit") { "hmmemit" }
                    else { "hmmsearch" };
                if args.is_empty() {
                    args.push(correct_sub.to_string());
                } else if !args[0].starts_with('-') {
                    args[0] = correct_sub.to_string();
                } else {
                    args.insert(0, correct_sub.to_string());
                }
                if correct_sub == "hmmpress" {
                    if !args.iter().any(|a| !a.starts_with('-') && a.contains('.')) {
                        if let Some(hmm) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".hmm")) {
                            args.push(hmm.clone());
                        }
                    }
                } else if correct_sub == "hmmbuild" {
                    if !args.iter().any(|a| a == "--cpu") {
                        args.push("--cpu".to_string());
                        args.push("8".to_string());
                    }
                } else if correct_sub == "hmmscan" || correct_sub == "hmmsearch" || correct_sub == "phmmer" || correct_sub == "jackhmmer" {
                    if !args.iter().any(|a| a == "--cpu") {
                        args.push("--cpu".to_string());
                        args.push("8".to_string());
                    }
                    if !args.iter().any(|a| a == "--tblout") {
                        if let Some(out) = tv.output_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".tbl")) {
                            args.push("--tblout".to_string());
                            args.push(out.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-E") {
                        args.push("-E".to_string());
                        args.push("1e-5".to_string());
                    }
                }
                let mut i = 0;
                while i < args.len() {
                    if (args[i] == "--tblout" || args[i] == "--domtblout" || args[i] == "-o" || args[i] == "-A") && i + 1 < args.len() {
                        let val = args[i + 1].to_ascii_lowercase();
                        if val.ends_with(".bam") {
                            let new_ext = if args[i] == "-A" { ".sto" } else { ".tbl" };
                            args[i + 1] = args[i + 1].trim_end_matches(".bam").to_string() + new_ext;
                        }
                    }
                    i += 1;
                }
            }
        }
        "sourmash" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if !args.is_empty() {
                    let first = &args[0];
                    let first_lower = first.to_ascii_lowercase();
                    let correct_sub = if task_lower.contains("sketch") || task_lower.contains("compute") || task_lower.contains("signature") {
                        if task_lower.contains("dna") || task_lower.contains("genome") || task_lower.contains("nucleotide") {
                            Some("sketch dna")
                        } else if task_lower.contains("protein") || task_lower.contains("translate") {
                            Some("sketch protein")
                        } else {
                            Some("sketch dna")
                        }
                    } else if task_lower.contains("compare") {
                        Some("compare")
                    } else if task_lower.contains("gather") {
                        Some("gather")
                    } else if task_lower.contains("search") && !task_lower.contains("gather") {
                        Some("search")
                    } else if task_lower.contains("index") {
                        Some("index")
                    } else if task_lower.contains("tax") || task_lower.contains("taxonomy") || task_lower.contains("classify") || task_lower.contains("annotate") {
                        Some("taxonomy annotate")
                    } else {
                        None
                    };
                    if let Some(sub) = correct_sub {
                        let sub_parts: Vec<&str> = sub.split_whitespace().collect();
                        let sub_main = sub_parts[0];
                        if first_lower != sub_main {
                            if !first.starts_with('-') {
                                args[0] = sub_main.to_string();
                            } else {
                                args.insert(0, sub_main.to_string());
                            }
                        }
                        if sub_parts.len() > 1 {
                            let sub_sub = sub_parts[1];
                            if args.len() > 1 && args[1].to_ascii_lowercase() != sub_sub {
                                args.insert(1, sub_sub.to_string());
                            }
                        }
                    }
                }
                let is_sketch = args.iter().any(|a| a.to_ascii_lowercase() == "sketch");
                if is_sketch {
                    let has_k = args.iter().any(|a| a == "-k");
                    let has_p = args.iter().any(|a| a == "-p");
                    if has_k && !has_p {
                        let mut new_args = Vec::new();
                        let mut skip_next = false;
                        for (i, a) in args.iter().enumerate() {
                            if skip_next { skip_next = false; continue; }
                            if a == "-k" && i + 1 < args.len() {
                                let k_val = &args[i + 1];
                                new_args.push("-p".to_string());
                                new_args.push(format!("k={},scaled=1000", k_val));
                                skip_next = true;
                            } else {
                                new_args.push(a.clone());
                            }
                        }
                        args = new_args;
                    }
                    for i in 0..args.len() {
                        if (args[i] == "-o" || args[i] == "--output") && i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".bam") || out_lower.ends_with(".sam") || out_lower.ends_with(".vcf") {
                                args[i + 1] = args[i + 1].replace(".bam", ".sig").replace(".BAM", ".sig")
                                    .replace(".sam", ".sig").replace(".SAM", ".sig")
                                    .replace(".vcf", ".sig").replace(".VCF", ".sig");
                            } else if out_lower == "output.bam" || out_lower == "output" {
                                args[i + 1] = "output.sig".to_string();
                            }
                        }
                    }
                }
                if args.iter().any(|a| a.to_ascii_lowercase() == "compare") {
                    for i in 0..args.len() {
                        if (args[i] == "-o" || args[i] == "--output") && i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".bam") {
                                args[i + 1] = args[i + 1].replace(".bam", ".csv").replace(".BAM", ".csv");
                            } else if out_lower == "output.bam" {
                                args[i + 1] = "output.csv".to_string();
                            }
                        }
                    }
                    if !args.iter().any(|a| a == "--csv") && !args.iter().any(|a| a == "-k") {
                        let tv = extract_task_values(task);
                        if let Some(out) = tv.output_files.first() {
                            if out.to_ascii_lowercase().ends_with(".csv") {
                                args.push("--csv".to_string());
                                args.push(out.clone());
                            }
                        }
                    }
                }
            }
        }
        "arriba" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if !args.is_empty() {
                    let first = &args[0];
                    let first_lower = first.to_ascii_lowercase();
                    let correct_sub = if task_lower.contains("draw") || task_lower.contains("visualize") {
                        Some("draw_fusions.R")
                    } else if task_lower.contains("convert") && task_lower.contains("vcf") {
                        Some("convert_fusions_to_vcf")
                    } else if task_lower.contains("wrapper") || task_lower.contains("prealigned") {
                        Some("run_arriba_on_prealigned_bam")
                    } else if task_lower.contains("pipeline") || task_lower.contains("full") || task_lower.contains("run_arriba") {
                        Some("run_arriba")
                    } else {
                        None
                    };
                    if let Some(sub) = correct_sub {
                        if first_lower != sub.to_ascii_lowercase() {
                            if !first.starts_with('-') {
                                args[0] = sub.to_string();
                            } else {
                                args.insert(0, sub.to_string());
                            }
                        }
                    }
                }
            }
        }
        "r" => {
            while !args.is_empty() && !args[0].starts_with('-') && !args[0].starts_with('\'') && !args[0].starts_with('"') {
                let first_lower = args[0].to_ascii_lowercase();
                if first_lower != "rscript" && first_lower != "r"
                    && !first_lower.ends_with(".r") && !first_lower.ends_with(".rscript")
                    && !first_lower.ends_with(".rmd") {
                    args.remove(0);
                } else {
                    break;
                }
            }
            if !args.is_empty() {
                let first_lower = args[0].to_ascii_lowercase();
                if first_lower == "r" {
                    args[0] = "Rscript".to_string();
                } else if first_lower.ends_with(".r") || first_lower.ends_with(".rscript") || first_lower.ends_with(".rmd") {
                    args.insert(0, "Rscript".to_string());
                } else if first_lower == "-e" {
                    args.insert(0, "Rscript".to_string());
                }
            } else {
                args.insert(0, "Rscript".to_string());
            }
            if !args.is_empty() && args[0].to_ascii_lowercase() != "rscript" {
                args.insert(0, "Rscript".to_string());
            }
        }
        "awk" | "sed" | "grep" => {
            while !args.is_empty() && !args[0].starts_with('-') && !args[0].starts_with('\'') && !args[0].starts_with('"') && !args[0].starts_with('/') {
                let first_lower = args[0].to_ascii_lowercase();
                if first_lower == "awk" || first_lower == "sed" || first_lower == "grep"
                    || first_lower == "filter" || first_lower == "process" || first_lower == "search"
                    || first_lower == "extract" || first_lower == "transform" || first_lower == "print"
                    || first_lower == "find" || first_lower == "replace" || first_lower == "remove" {
                    args.remove(0);
                } else {
                    break;
                }
            }
            if tool_lower == "grep" {
                let mut i = 0;
                while i < args.len() {
                    if args[i].contains("*") && (args[i].ends_with(".bam") || args[i].ends_with(".sam") || args[i].ends_with(".vcf") || args[i].ends_with(".fa") || args[i].ends_with(".fq")) {
                        args.remove(i);
                    } else {
                        i += 1;
                    }
                }
                if let Some(task) = task {
                    let tv = extract_task_values(task);
                    let has_pattern = args.iter().any(|a| {
                        !a.starts_with('-') && !a.contains('.') && !a.contains('/') && a.len() > 0
                            && a != &tv.input_files.iter().find(|f| *a == **f).cloned().unwrap_or_default()
                    });
                    if !has_pattern {
                        if let Some(pattern) = extract_grep_pattern(task) {
                            let flag_idx = args.iter().position(|a| a.starts_with('-'));
                            if let Some(idx) = flag_idx {
                                let mut next_flag = idx + 1;
                                while next_flag < args.len() && !args[next_flag].starts_with('-') {
                                    next_flag += 1;
                                }
                                args.insert(next_flag, pattern);
                            } else {
                                args.insert(0, pattern);
                            }
                        }
                    }
                }
            }
            if tool_lower == "awk" {
                let mut i = 0;
                while i < args.len() {
                    if args[i] == "/etc/passwd" {
                        args.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            if let Some(task) = task {
                let tv = extract_task_values(task);
                let has_input_file = args.iter().any(|a| {
                    let al = a.to_ascii_lowercase();
                    tv.input_files.iter().any(|f| f.to_ascii_lowercase() == al)
                        || al.ends_with(".txt") || al.ends_with(".csv") || al.ends_with(".log")
                        || al.ends_with(".tsv") || al.ends_with(".bed") || al.ends_with(".fa")
                        || al.ends_with(".fasta") || al.ends_with(".fastq") || al.ends_with(".fq")
                        || al.ends_with(".sam") || al.ends_with(".vcf")
                        || al.ends_with(".py") || al.ends_with(".ini") || al.ends_with(".conf")
                });
                if !has_input_file {
                    if let Some(input) = tv.input_files.first() {
                        args.push(input.clone());
                    }
                }
            }
        }
        "pbccs" => {
            if let Some(_task) = task {
                if !args.is_empty() {
                    let first_lower = args[0].to_ascii_lowercase();
                    if !args[0].starts_with('-') && first_lower != "ccs" {
                        args.remove(0);
                    } else if args[0].starts_with('-') {
                        args.insert(0, "ccs".to_string());
                    }
                }
            }
        }
        "bakta" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                for i in 0..args.len() {
                    if args[i] == "--db" && i + 1 < args.len() {
                        let db_val = &args[i + 1];
                        if db_val.to_ascii_lowercase() == "database" || db_val.to_ascii_lowercase() == "db" {
                            if let Some(db) = tv.database_files.first() {
                                args[i + 1] = db.clone();
                            } else if let Some(dir) = tv.genome_dirs.first() {
                                args[i + 1] = dir.clone();
                            } else {
                                args[i + 1] = "/path/to/bakta_db/".to_string();
                            }
                        }
                    }
                    if args[i] == "--output" && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if out_lower == "output_dir/" || out_lower == "output_dir" {
                            if let Some(out) = tv.output_files.first() {
                                let path = std::path::Path::new(out);
                                if let Some(parent) = path.parent() {
                                    args[i + 1] = format!("{}/", parent.to_string_lossy());
                                }
                            }
                        }
                    }
                    if args[i] == "--prefix" && i + 1 < args.len() {
                        let prefix_val = &args[i + 1];
                        if prefix_val.to_ascii_lowercase() == "output" || prefix_val.to_ascii_lowercase() == "prefix" {
                            if let Some(out) = tv.output_files.first() {
                                let stem = std::path::Path::new(out)
                                    .file_stem()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "genome_annotation".to_string());
                                args[i + 1] = stem;
                            }
                        }
                    }
                }
            }
        }
        "metaphlan" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                let task_lower = task.to_ascii_lowercase();
                if !args.iter().any(|a| a == "--input_type") {
                    let input_type = if task_lower.contains("fastq") {
                        "fastq"
                    } else if task_lower.contains("bam") || task_lower.contains("sam") {
                        "bam"
                    } else if task_lower.contains("mapout") || task_lower.contains("bowtie2out") {
                        "mapout"
                    } else if task_lower.contains("fasta") || task_lower.contains("fa") {
                        "fasta"
                    } else {
                        "fastq"
                    };
                    args.push("--input_type".to_string());
                    args.push(input_type.to_string());
                }
                if !args.iter().any(|a| a == "--db_dir" || a == "--bowtie2db") {
                    if let Some(db) = tv.database_files.first() {
                        args.push("--db_dir".to_string());
                        args.push(db.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o" || a == "--output") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "--nproc" || a == "-nproc") {
                    args.push("--nproc".to_string());
                    args.push("8".to_string());
                }
                for i in 0..args.len() {
                    if args[i] == "--db_dir" && i + 1 < args.len() {
                        if args[i + 1].to_ascii_lowercase() == "database" || args[i + 1].to_ascii_lowercase() == "db" {
                            if let Some(db) = tv.database_files.first() {
                                args[i + 1] = db.clone();
                            } else {
                                args[i + 1] = "/path/to/mpa_db".to_string();
                            }
                        }
                    }
                    if args[i] == "-o" && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if out_lower.ends_with(".bam") {
                            args[i + 1] = args[i + 1].replace(".bam", ".txt").replace(".BAM", ".txt");
                        }
                    }
                }
            }
        }
        "qualimap" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                if !args.iter().any(|a| a == "--java-mem-size") {
                    args.push("--java-mem-size".to_string());
                    args.push("8G".to_string());
                }
                for i in 0..args.len() {
                    if args[i] == "-outdir" && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if !out_lower.ends_with('/') {
                            args[i + 1] = format!("{}/", args[i + 1]);
                        }
                    }
                }
            }
        }
        "muscle" => {
            for i in 0..args.len() {
                if (args[i] == "-output" || args[i] == "-out") && i + 1 < args.len() {
                    let out_lower = args[i + 1].to_ascii_lowercase();
                    if out_lower.ends_with(".bam") {
                        args[i + 1] = args[i + 1].replace(".bam", ".fasta").replace(".BAM", ".fasta");
                    } else if out_lower.ends_with(".vcf") {
                        args[i + 1] = args[i + 1].replace(".vcf", ".fasta").replace(".VCF", ".fasta");
                    } else if out_lower == "output.bam" {
                        args[i + 1] = "output.fasta".to_string();
                    }
                }
            }
        }
        "minimap2" => {
            for i in 0..args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    let out_lower = args[i + 1].to_ascii_lowercase();
                    if out_lower.ends_with(".bam") && !args.iter().any(|a| a.starts_with("-a")) {
                        args[i + 1] = args[i + 1].replace(".bam", ".paf").replace(".BAM", ".paf");
                    }
                }
            }
        }
        "seqkit" => {
            for i in 0..args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    let out_lower = args[i + 1].to_ascii_lowercase();
                    if out_lower.ends_with(".bam") {
                        args[i + 1] = args[i + 1].replace(".bam", ".fasta").replace(".BAM", ".fasta");
                    }
                }
            }
        }
        "rsem" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                for i in 0..args.len() {
                    if args[i] == "--gtf" && i + 1 < args.len() {
                        if args[i + 1].to_ascii_lowercase() == "annotation.gtf" || args[i + 1].to_ascii_lowercase() == "gtf" {
                            if let Some(ann) = tv.annotation_files.first() {
                                args[i + 1] = ann.clone();
                            }
                        }
                    }
                }
            }
        }
        "mmseqs2" => {
            for i in 0..args.len() {
                if (args[i] == "-o" || args[i] == "--output") && i + 1 < args.len() {
                    let out_lower = args[i + 1].to_ascii_lowercase();
                    if out_lower.ends_with(".bam") {
                        args[i + 1] = args[i + 1].replace(".bam", ".m8").replace(".BAM", ".m8");
                    }
                }
            }
        }
        "pbsv" => {
            for i in 0..args.len() {
                if args[i] == "--output" && i + 1 < args.len() {
                    let out_lower = args[i + 1].to_ascii_lowercase();
                    if out_lower.ends_with(".bam") {
                        args[i + 1] = args[i + 1].replace(".bam", ".svsig.gz").replace(".BAM", ".svsig.gz");
                    } else if out_lower.ends_with(".gz") && !out_lower.contains("svsig") {
                        args[i + 1] = args[i + 1].replace(".gz", ".svsig.gz");
                    }
                }
            }
        }
        "perl" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                let help_text_flags = ["-[mM][-]module", "-c", "-p", "-x[directory]", "-l[octal]", "-V[:variable]", "-F/pattern/"];
                let is_help_text = args.iter().any(|a| help_text_flags.iter().any(|hf| a.contains(hf)));
                if is_help_text {
                    args = Vec::new();
                }
                if task_lower.contains("version") || task_lower.contains("-v") || task_lower.contains("-V") {
                    args = vec!["-V".to_string()];
                } else if task_lower.contains("one-liner") || task_lower.contains("-ne") || task_lower.contains("-pe") || task_lower.contains("-e") {
                    let mut new_args = Vec::new();
                    if task_lower.contains("-ne") {
                        new_args.push("-ne".to_string());
                    } else if task_lower.contains("-pe") || task_lower.contains("-lane") {
                        new_args.push("-pe".to_string());
                    } else {
                        new_args.push("-e".to_string());
                    }
                    if let Some(input) = tv.input_files.first() {
                        new_args.push(input.clone());
                    }
                    args = new_args;
                } else if task_lower.contains("in-place") || task_lower.contains("-i") {
                    let mut new_args = vec!["-i.bak".to_string(), "-pe".to_string()];
                    if let Some(input) = tv.input_files.first() {
                        new_args.push(input.clone());
                    }
                    args = new_args;
                } else if task_lower.contains("cpan") || task_lower.contains("install") || task_lower.contains("module") {
                    args = vec!["-MCPAN".to_string(), "-e".to_string(), "'CPAN::Shell->install(\"Module\")'".to_string()];
                } else if let Some(input) = tv.input_files.first() {
                    if input.to_ascii_lowercase().ends_with(".pl") {
                        args = vec![input.clone()];
                    }
                }
            }
        }
        "python" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                let help_text_flags = ["-i", "-m", "-B", "-E", "-s", "-V", "-W"];
                let help_count = args.iter().filter(|a| help_text_flags.iter().any(|hf| a.as_str() == *hf)).count();
                if help_count >= 3 {
                    args = Vec::new();
                }
                if task_lower.contains("version") {
                    args = vec!["--version".to_string()];
                } else if task_lower.contains("module") || task_lower.contains("-m") {
                    let mut new_args = vec!["-m".to_string()];
                    if task_lower.contains("http.server") || task_lower.contains("http") {
                        new_args.push("http.server".to_string());
                        new_args.push("8080".to_string());
                    } else if task_lower.contains("pytest") || task_lower.contains("test") {
                        new_args.push("pytest".to_string());
                        if let Some(input) = tv.input_files.first() {
                            new_args.push(input.clone());
                        }
                        new_args.push("-v".to_string());
                    } else if task_lower.contains("venv") || task_lower.contains("virtual") {
                        new_args.push("venv".to_string());
                        if let Some(out) = tv.output_files.first() {
                            new_args.push(out.clone());
                        } else {
                            new_args.push(".venv".to_string());
                        }
                    } else if task_lower.contains("cprofile") || task_lower.contains("profile") {
                        new_args.push("cProfile".to_string());
                        new_args.push("-s".to_string());
                        new_args.push("cumtime".to_string());
                        if let Some(input) = tv.input_files.first() {
                            new_args.push(input.clone());
                        }
                    } else if task_lower.contains("json") || task_lower.contains("process") {
                        new_args.push("json".to_string());
                    } else {
                        new_args.push("module".to_string());
                    }
                    args = new_args;
                } else if task_lower.contains("-c") || task_lower.contains("one-liner") || task_lower.contains("expression") {
                    args = vec!["-c".to_string(), "\"expression\"".to_string()];
                } else if let Some(input) = tv.input_files.first() {
                    if input.to_ascii_lowercase().ends_with(".py") {
                        args = vec![input.clone()];
                    }
                }
            }
        }
        "bash" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if task_lower.contains("version") {
                    args = vec!["--version".to_string()];
                } else if task_lower.contains("strict") || task_lower.contains("pipefail") || task_lower.contains("-c") {
                    let mut new_args = vec!["-euo".to_string(), "pipefail".to_string(), "-c".to_string()];
                    if let Some(input) = tv.input_files.first() {
                        new_args.push(format!("'{}'", input));
                    }
                    args = new_args;
                } else if task_lower.contains("debug") || task_lower.contains("trace") || task_lower.contains("-x") {
                    let mut new_args = vec!["-x".to_string()];
                    if let Some(input) = tv.input_files.first() {
                        new_args.push(input.clone());
                    }
                    args = new_args;
                } else if let Some(input) = tv.input_files.first() {
                    if input.to_ascii_lowercase().ends_with(".sh") {
                        args = vec![input.clone()];
                    }
                }
            }
        }
        "java" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if task_lower.contains("version") {
                    args = vec!["-version".to_string()];
                } else if task_lower.contains("jar") || task_lower.contains("-jar") {
                    let jar_file = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".jar")).cloned();
                    let mut new_args = vec!["-Xmx8g".to_string(), "-jar".to_string()];
                    if let Some(jar) = jar_file {
                        new_args.push(jar);
                    } else if let Some(input) = tv.input_files.first() {
                        new_args.push(input.clone());
                    }
                    args = new_args;
                } else if task_lower.contains("gatk") || task_lower.contains("haplotypecaller") {
                    let jar_file = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".jar")).cloned();
                    let mut new_args = vec!["-Xmx8g".to_string(), "-jar".to_string()];
                    if let Some(jar) = jar_file {
                        new_args.push(jar);
                    } else {
                        new_args.push("gatk.jar".to_string());
                    }
                    new_args.push("HaplotypeCaller".to_string());
                    args = new_args;
                } else if task_lower.contains("trimmomatic") {
                    let jar_file = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".jar")).cloned();
                    let mut new_args = vec!["-Xmx4g".to_string(), "-jar".to_string()];
                    if let Some(jar) = jar_file {
                        new_args.push(jar);
                    } else {
                        new_args.push("trimmomatic.jar".to_string());
                    }
                    args = new_args;
                }
            }
        }
        "julia" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if task_lower.contains("version") {
                    args = vec!["-e".to_string(), "'println(VERSION)'".to_string()];
                } else if task_lower.contains("project") || task_lower.contains("environment") {
                    let mut new_args = vec!["--project=.".to_string()];
                    if let Some(input) = tv.input_files.first() {
                        new_args.push(input.clone());
                    }
                    args = new_args;
                } else if task_lower.contains("threads") || task_lower.contains("multi-thread") {
                    let mut new_args = vec!["--threads".to_string(), "auto".to_string()];
                    if let Some(input) = tv.input_files.first() {
                        new_args.push(input.clone());
                    }
                    args = new_args;
                } else if task_lower.contains("-e") || task_lower.contains("expression") {
                    args = vec!["-e".to_string(), "'expression'".to_string()];
                } else if let Some(input) = tv.input_files.first() {
                    if input.to_ascii_lowercase().ends_with(".jl") {
                        args = vec![input.clone()];
                    }
                }
            }
        }
        "ssh" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                let help_text_flags = ["-g", "-A", "-k", "-M", "-N", "-f", "-D", "-L", "-R"];
                let help_count = args.iter().filter(|a| help_text_flags.iter().any(|hf| a.as_str() == *hf)).count();
                if help_count >= 3 && !args.iter().any(|a| a.contains("@")) {
                    args = Vec::new();
                }
                if args.is_empty() {
                    if let Some(host) = tv.input_files.first() {
                        args.push(host.clone());
                    }
                }
            }
        }
        "hifiasm" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "hifiasm" || first_lower == "assemble" || first_lower == "run" {
                        args.remove(0);
                    } else {
                        break;
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                for i in 0..args.len() {
                    if args[i] == "-o" && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if out_lower.ends_with(".bam") {
                            args[i + 1] = args[i + 1].replace(".bam", ".fasta").replace(".BAM", ".fasta");
                        }
                    }
                }
                if !args.iter().any(|a| a == "-t" || a == "--threads") {
                    args.push("-t".to_string());
                    args.push("16".to_string());
                }
                let has_input = args.iter().any(|a| {
                    let al = a.to_ascii_lowercase();
                    al.ends_with(".fastq") || al.ends_with(".fq") || al.ends_with(".gz")
                        || al.ends_with(".fasta") || al.ends_with(".fa")
                });
                if !has_input {
                    if let Some(fq) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".fastq.gz") || fl.ends_with(".fq.gz")
                    }) {
                        args.push(fq.clone());
                    } else if let Some(input) = tv.input_files.first() {
                        args.push(input.clone());
                    }
                }
                if !args.iter().any(|a| a.starts_with("--hifi") || a.starts_with("--nano") || a.starts_with("-l0")) {
                    if task_lower.contains("hifi") || task_lower.contains("hi-fi") || task_lower.contains("ccs") || task_lower.contains("pacbio") {
                        args.push("--hifi".to_string());
                    } else if task_lower.contains("nano") || task_lower.contains("ont") {
                        args.push("--nano".to_string());
                    } else {
                        args.push("-l0".to_string());
                    }
                }
            }
        }
        "verkko" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    args.remove(0);
                }
                let has_d = args.iter().any(|a| a == "-d" || a == "--directory");
                if !has_d {
                    args.insert(0, "-d".to_string());
                    if let Some(out) = tv.output_files.first() {
                        args.insert(1, out.clone());
                    } else {
                        args.insert(1, "verkko_output/".to_string());
                    }
                }
                let has_hifi = args.iter().any(|a| a == "--hifi");
                let has_nano = args.iter().any(|a| a == "--nano");
                if !has_hifi && !has_nano {
                    let task_lower = task.to_ascii_lowercase();
                    if task_lower.contains("hifi") || task_lower.contains("hi-fi") || task_lower.contains("pacbio") || task_lower.contains("ccs") {
                        args.push("--hifi".to_string());
                    } else {
                        args.push("--nano".to_string());
                    }
                }
                let has_input = args.iter().any(|a| {
                    let al = a.to_ascii_lowercase();
                    al.ends_with(".fastq") || al.ends_with(".fq") || al.ends_with(".gz")
                });
                if !has_input {
                    if let Some(fq) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".fastq.gz") || fl.ends_with(".fq.gz")
                    }) {
                        args.push(fq.clone());
                    }
                }
            }
        }
        "busco" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !args.iter().any(|a| a == "-i" || a == "--in") {
                    if let Some(input) = tv.input_files.first() {
                        args.push("-i".to_string());
                        args.push(input.clone());
                    }
                }
                if !args.iter().any(|a| a == "-l" || a == "--lineage_dataset") || args.iter().any(|a| a == "-l" && args.iter().position(|x| x == a).map(|p| args.get(p+1)).flatten().map(|v| v == "bacteria").unwrap_or(false)) {
                    let lineage = if task_lower.contains("bacteria") { "bacteria_odb10" }
                        else if task_lower.contains("eukaryota") || task_lower.contains("eukaryote") { "eukaryota_odb10" }
                        else if task_lower.contains("fungi") || task_lower.contains("fungal") { "fungi_odb10" }
                        else if task_lower.contains("metazoa") || task_lower.contains("animal") { "metazoa_odb10" }
                        else if task_lower.contains("plant") { "embryophyta_odb10" }
                        else if task_lower.contains("virus") || task_lower.contains("viral") { "viruses_odb10" }
                        else { "bacteria_odb10" };
                    let l_idx = args.iter().position(|a| a == "-l" || a == "--lineage_dataset");
                    if let Some(idx) = l_idx {
                        if idx + 1 < args.len() { args[idx + 1] = lineage.to_string(); }
                    } else {
                        args.push("-l".to_string());
                        args.push(lineage.to_string());
                    }
                }
                if !args.iter().any(|a| a == "-m" || a == "--mode") {
                    let mode = if task_lower.contains("protein") || task_lower.contains("proteome") { "protein" }
                        else if task_lower.contains("genome") || task_lower.contains("assembly") { "genome" }
                        else if task_lower.contains("transcriptome") || task_lower.contains("transcript") { "transcriptome" }
                        else { "genome" };
                    args.push("-m".to_string());
                    args.push(mode.to_string());
                }
                if !args.iter().any(|a| a == "-o" || a == "--out") {
                    if let Some(out) = tv.output_files.first() {
                        let stem = std::path::Path::new(out).file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "busco_output".to_string());
                        args.push("-o".to_string());
                        args.push(stem);
                    }
                }
                if !args.iter().any(|a| a == "-f" || a == "--force") {
                    args.push("-f".to_string());
                }
                if task_lower.contains("auto-lineage") && !args.iter().any(|a| a == "--auto-lineage") {
                    args.push("--auto-lineage".to_string());
                }
                if task_lower.contains("auto-lineage-euk") && !args.iter().any(|a| a == "--auto-lineage-euk") {
                    args.push("--auto-lineage-euk".to_string());
                }
                if task_lower.contains("auto-lineage-prok") && !args.iter().any(|a| a == "--auto-lineage-prok") {
                    args.push("--auto-lineage-prok".to_string());
                }
            }
        }
        "kraken2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !task_lower.contains("build") {
                    if !args.iter().any(|a| a == "--db") {
                        let db = tv.database_files.first().cloned()
                            .unwrap_or_else(|| "/path/to/kraken2_db".to_string());
                        args.push("--db".to_string());
                        args.push(db);
                    }
                    for i in 0..args.len() {
                        if args[i] == "--db" && i + 1 < args.len() {
                            let db_val = &args[i + 1];
                            if db_val.to_ascii_lowercase() == "database" || db_val.to_ascii_lowercase() == "db" {
                                if let Some(db) = tv.database_files.first() {
                                    args[i + 1] = db.clone();
                                } else {
                                    args[i + 1] = "/path/to/kraken2_db".to_string();
                                }
                            }
                        }
                        if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".bam") {
                                args[i + 1] = args[i + 1].replace(".bam", ".txt").replace(".BAM", ".txt");
                            }
                        }
                        if args[i] == "--report" && i + 1 < args.len() {
                            let rep_lower = args[i + 1].to_ascii_lowercase();
                            if rep_lower.ends_with(".fastq") || rep_lower.ends_with(".fq") || rep_lower.ends_with(".bam") {
                                args[i + 1] = args[i + 1].replace(".fastq", ".txt").replace(".fq", ".txt").replace(".bam", ".txt");
                            }
                        }
                    }
                    if !args.iter().any(|a| a == "--output") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("--output".to_string());
                            args.push(out.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "--report") {
                        args.push("--report".to_string());
                        args.push("report.txt".to_string());
                    }
                    if !args.iter().any(|a| a == "--paired" || a == "-1" || a == "-U") {
                        let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                        }).collect();
                        if fq_files.len() >= 2 {
                            args.push("--paired".to_string());
                            args.push("-1".to_string());
                            args.push(fq_files[0].clone());
                            args.push("-2".to_string());
                            args.push(fq_files[1].clone());
                        } else if let Some(fq) = fq_files.first() {
                            args.push((*fq).clone());
                        }
                    }
                }
            }
        }
        "vcftools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                for i in 0..args.len() {
                    if args[i] == "--out" && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if out_lower.ends_with(".bam") || out_lower.ends_with(".vcf") || out_lower.ends_with(".txt") {
                            let stem = std::path::Path::new(&args[i + 1]).file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "output".to_string());
                            args[i + 1] = stem;
                        }
                    }
                }
                if !args.iter().any(|a| a == "--vcf" || a == "--gzvcf" || a == "--bcf") {
                    if let Some(vcf) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".vcf") || fl.ends_with(".vcf.gz") || fl.ends_with(".bcf")
                    }) {
                        let flag = if vcf.to_ascii_lowercase().ends_with(".gz") { "--gzvcf" } else if vcf.to_ascii_lowercase().ends_with(".bcf") { "--bcf" } else { "--vcf" };
                        args.push(flag.to_string());
                        args.push(vcf.clone());
                    }
                }
                if !args.iter().any(|a| a == "--out") {
                    if let Some(out) = tv.output_files.first() {
                        let stem = std::path::Path::new(out).file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "output".to_string());
                        args.push("--out".to_string());
                        args.push(stem);
                    }
                }
            }
        }
        "cellsnp-lite" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                let mut i = 0;
                while i < args.len() {
                    if args[i] == "--inclFLAG" || args[i] == "--incl_flag" {
                        args.remove(i);
                        if i < args.len() && !args[i].starts_with('-') { args.remove(i); }
                    } else { i += 1; }
                }
                for i in 0..args.len() {
                    if args[i] == "-o" {
                        args[i] = "-O".to_string();
                    }
                    if args[i] == "-O" && i + 1 < args.len() {
                        if !args[i + 1].ends_with('/') {
                            args[i + 1] = format!("{}/", args[i + 1]);
                        }
                    }
                    if args[i] == "-R" && i + 1 < args.len() {
                        let r_val = &args[i + 1];
                        if r_val.to_ascii_lowercase().ends_with(".bam") || r_val.to_ascii_lowercase().ends_with(".fa") || r_val.to_ascii_lowercase().ends_with(".fasta") {
                            if let Some(vcf) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".vcf") || f.to_ascii_lowercase().ends_with(".vcf.gz")) {
                                args[i + 1] = vcf.clone();
                            }
                        }
                    }
                    if args[i] == "-s" && i + 1 < args.len() {
                        let s_val = &args[i + 1];
                        if s_val.to_ascii_lowercase().ends_with(".vcf") || s_val.to_ascii_lowercase().ends_with(".fa") || s_val.to_ascii_lowercase().ends_with(".fasta") {
                            if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                                args[i + 1] = bam.clone();
                            }
                        }
                    }
                }
                if !args.iter().any(|a| a == "-O") {
                    if let Some(out) = tv.output_files.first() {
                        let out_dir = if out.ends_with('/') { out.clone() } else { format!("{}/", out) };
                        args.push("-O".to_string());
                        args.push(out_dir);
                    }
                }
                if !args.iter().any(|a| a == "-R" || a == "-T") {
                    if let Some(vcf) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".vcf") || f.to_ascii_lowercase().ends_with(".vcf.gz")) {
                        args.push("-R".to_string());
                        args.push(vcf.clone());
                    }
                }
                if !args.iter().any(|a| a == "-b") {
                    if let Some(barcode) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".tsv") || fl.ends_with(".txt") || fl.ends_with(".csv")
                    }) {
                        args.push("-b".to_string());
                        args.push(barcode.clone());
                    }
                }
                if !args.iter().any(|a| a == "--minMAF") {
                    args.push("--minMAF".to_string());
                    args.push("0.1".to_string());
                }
                if !args.iter().any(|a| a == "--minCOUNT") {
                    args.push("--minCOUNT".to_string());
                    args.push("20".to_string());
                }
                if !args.iter().any(|a| a == "-p" || a == "--nproc") {
                    args.push("-p".to_string());
                    args.push("4".to_string());
                }
            }
        }
        "gtdbtk" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "gtdbtk" { args.remove(0); } else { break; }
                }
                let subcmd = if task_lower.contains("identify") || (task_lower.contains("marker") && task_lower.contains("identif")) { "identify" }
                    else if task_lower.contains("align") || (task_lower.contains("alignment") && !task_lower.contains("classify")) { "align" }
                    else if task_lower.contains("classify") && !task_lower.contains("classify_wf") && !task_lower.contains("workflow") { "classify" }
                    else if task_lower.contains("de_novo") || task_lower.contains("denovo") || task_lower.contains("de novo") { "de_novo_wf" }
                    else if task_lower.contains("infer") { "infer" }
                    else if task_lower.contains("ani_screen") { "ani_screen" }
                    else { "classify_wf" };
                if args.is_empty() {
                    args.push(subcmd.to_string());
                } else if !args[0].starts_with('-') && args[0].to_ascii_lowercase() != subcmd {
                    args[0] = subcmd.to_string();
                } else if args[0].starts_with('-') {
                    args.insert(0, subcmd.to_string());
                }
                if subcmd == "identify" || subcmd == "classify_wf" || subcmd == "de_novo_wf" {
                    if !args.iter().any(|a| a == "--genome_dir") {
                        let dir = tv.genome_dirs.first().cloned()
                            .or_else(|| tv.input_files.iter().find(|f| f.contains("/") || f.contains("genome") || f.contains("bin")).cloned())
                            .unwrap_or_else(|| "bins/".to_string());
                        args.push("--genome_dir".to_string());
                        args.push(dir);
                    }
                }
                if subcmd == "classify" {
                    if !args.iter().any(|a| a == "--genome_dir") {
                        let dir = tv.genome_dirs.first().cloned()
                            .or_else(|| tv.input_files.iter().find(|f| f.contains("/") || f.contains("genome") || f.contains("bin")).cloned())
                            .unwrap_or_else(|| "bins/".to_string());
                        args.push("--genome_dir".to_string());
                        args.push(dir);
                    }
                    if !args.iter().any(|a| a == "--align_dir") {
                        args.push("--align_dir".to_string());
                        args.push("gtdbtk_align/".to_string());
                    }
                }
                if subcmd == "align" {
                    if !args.iter().any(|a| a == "--identify_dir") {
                        args.push("--identify_dir".to_string());
                        args.push("gtdbtk_identify/".to_string());
                    }
                }
                if subcmd == "classify_wf" {
                    if task_lower.contains("batchfile") || task_lower.contains("batch") {
                        if !args.iter().any(|a| a == "--batchfile") {
                            if let Some(tsv) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".tsv") || f.to_ascii_lowercase().ends_with(".csv")) {
                                args.push("--batchfile".to_string());
                                args.push(tsv.clone());
                            }
                        }
                    }
                    if task_lower.contains("genes") {
                        if !args.iter().any(|a| a == "--genes") {
                            args.push("--genes".to_string());
                        }
                    }
                    if task_lower.contains("skip_ani_screen") {
                        if !args.iter().any(|a| a == "--skip_ani_screen") {
                            args.push("--skip_ani_screen".to_string());
                        }
                    }
                    if task_lower.contains("scratch") {
                        if !args.iter().any(|a| a == "--scratch_dir") {
                            args.push("--scratch_dir".to_string());
                            args.push("/scratch/gtdbtk".to_string());
                        }
                    }
                    if task_lower.contains("pplacer_cpus") {
                        if !args.iter().any(|a| a == "--pplacer_cpus") {
                            args.push("--pplacer_cpus".to_string());
                            args.push("4".to_string());
                        }
                    }
                    if task_lower.contains("min_perc_aa") {
                        if !args.iter().any(|a| a == "--min_perc_aa") {
                            args.push("--min_perc_aa".to_string());
                            args.push("50".to_string());
                        }
                    }
                }
                if subcmd == "de_novo_wf" {
                    if task_lower.contains("bacteria") {
                        if !args.iter().any(|a| a == "--bacteria") {
                            args.push("--bacteria".to_string());
                        }
                    }
                }
                if !args.iter().any(|a| a == "--out_dir") {
                    let out = tv.output_files.first().cloned().unwrap_or_else(|| "gtdbtk_output/".to_string());
                    args.push("--out_dir".to_string());
                    args.push(out);
                }
                if !args.iter().any(|a| a == "--cpus") {
                    args.push("--cpus".to_string());
                    args.push("8".to_string());
                }
                if !args.iter().any(|a| a == "--extension") {
                    if task_lower.contains("faa") {
                        args.push("--extension".to_string());
                        args.push("faa".to_string());
                    } else if task_lower.contains("fasta") {
                        args.push("--extension".to_string());
                        args.push("fasta".to_string());
                    } else if task_lower.contains("fna") {
                        args.push("--extension".to_string());
                        args.push("fna".to_string());
                    } else {
                        args.push("--extension".to_string());
                        args.push("fa".to_string());
                    }
                }
            }
        }
        "truvari" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower != "bench" && first_lower != "collapse" && first_lower != "normalize" && first_lower != "anno" {
                        if task_lower.contains("bench") || task_lower.contains("compare") {
                            args[0] = "bench".to_string();
                        } else if task_lower.contains("collapse") {
                            args[0] = "collapse".to_string();
                        } else if task_lower.contains("normalize") {
                            args[0] = "normalize".to_string();
                        } else if task_lower.contains("anno") {
                            args[0] = "anno".to_string();
                        } else {
                            args[0] = "bench".to_string();
                        }
                    }
                }
                if args.iter().any(|a| a == "bench") {
                    if !args.iter().any(|a| a == "-b" || a == "--baseline") {
                        if let Some(vcf) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".vcf") || f.to_ascii_lowercase().ends_with(".vcf.gz")) {
                            args.push("-b".to_string());
                            args.push(vcf.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-c" || a == "--call") {
                        let vcfs: Vec<&String> = tv.input_files.iter().filter(|f| f.to_ascii_lowercase().ends_with(".vcf") || f.to_ascii_lowercase().ends_with(".vcf.gz")).collect();
                        if vcfs.len() >= 2 {
                            args.push("-c".to_string());
                            args.push(vcfs[1].clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-o" || a == "--output-dir") {
                        let out = tv.output_files.first().cloned().unwrap_or_else(|| "bench_output/".to_string());
                        args.push("-o".to_string());
                        args.push(out);
                    }
                }
            }
        }
        "sra-tools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !args.is_empty() {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "sra-tools" || first_lower == "sra_toolkit" || first_lower == "sra-tools" {
                        args.remove(0);
                    }
                }
                let subcmd = if task_lower.contains("vdb-validate") || task_lower.contains("validate") || task_lower.contains("integrity") { "vdb-validate" }
                    else if task_lower.contains("sra-stat") || task_lower.contains("statistics") || (task_lower.contains("stat") && task_lower.contains("sra")) { "sra-stat" }
                    else if task_lower.contains("prefetch") || (task_lower.contains("download") && !task_lower.contains("fastq")) || task_lower.contains("ena") || task_lower.contains("ebi") { "prefetch" }
                    else if task_lower.contains("fastq-dump") || task_lower.contains("split") { "fastq-dump" }
                    else if task_lower.contains("fasterq-dump") || task_lower.contains("fastq") || task_lower.contains("convert") || task_lower.contains("compress") || task_lower.contains("gzip") { "fasterq-dump" }
                    else if task_lower.contains("sam-dump") || task_lower.contains("bam") { "sam-dump" }
                    else { "fasterq-dump" };
                if args.is_empty() || !args[0].starts_with("prefetch") && !args[0].starts_with("fasterq") && !args[0].starts_with("fastq") && !args[0].starts_with("vdb") && !args[0].starts_with("sra-stat") && !args[0].starts_with("sam-dump") {
                    args = vec![subcmd.to_string()];
                }
                if subcmd == "prefetch" {
                    if !args.iter().any(|a| a.starts_with("SRR") || a.starts_with("ERR") || a.starts_with("DRR")) {
                        if let Some(srr) = tv.input_files.first() {
                            args.push(srr.clone());
                        } else {
                            args.push("SRR123456".to_string());
                        }
                    }
                    if !args.iter().any(|a| a == "-O") {
                        args.push("-O".to_string());
                        if let Some(out) = tv.output_files.first() {
                            args.push(out.clone());
                        } else {
                            args.push("sra_downloads/".to_string());
                        }
                    }
                    if task_lower.contains("option-file") || task_lower.contains("batch") {
                        if !args.iter().any(|a| a == "--option-file") {
                            if let Some(txt) = tv.input_files.iter().find(|f| {
                                let fl = f.to_ascii_lowercase();
                                fl.ends_with(".txt") || fl.ends_with(".tsv")
                            }) {
                                args.push("--option-file".to_string());
                                args.push(txt.clone());
                            }
                        }
                    }
                } else if subcmd == "fasterq-dump" {
                    if !args.iter().any(|a| a.starts_with("SRR") || a.starts_with("ERR") || a.starts_with("DRR")) {
                        if let Some(srr) = tv.input_files.first() {
                            args.push(srr.clone());
                        } else {
                            args.push("SRR123456".to_string());
                        }
                    }
                    if !args.iter().any(|a| a == "-O") {
                        args.push("-O".to_string());
                        if let Some(out) = tv.output_files.first() {
                            args.push(out.clone());
                        } else {
                            args.push("output/".to_string());
                        }
                    }
                    if !args.iter().any(|a| a == "-e") {
                        args.push("-e".to_string());
                        args.push("8".to_string());
                    }
                    if task_lower.contains("skip-technical") {
                        if !args.iter().any(|a| a == "--skip-technical") {
                            args.push("--skip-technical".to_string());
                        }
                    }
                    if task_lower.contains("stdout") {
                        if !args.iter().any(|a| a == "--stdout") {
                            args.push("--stdout".to_string());
                        }
                    }
                    if task_lower.contains("check-space") {
                        if !args.iter().any(|a| a == "--check-space") {
                            args.push("--check-space".to_string());
                        }
                    }
                } else if subcmd == "vdb-validate" {
                    if !args.iter().any(|a| a.ends_with(".sra") || a.ends_with(".vdb")) {
                        if let Some(sra) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".sra")) {
                            args.push(sra.clone());
                        }
                    }
                } else if subcmd == "sra-stat" {
                    if !args.iter().any(|a| a.starts_with("SRR") || a.starts_with("ERR")) {
                        args.push("SRR123456".to_string());
                    }
                    if task_lower.contains("quick") {
                        args.push("--quick".to_string());
                    }
                    if task_lower.contains("xml") {
                        args.push("--xml".to_string());
                    }
                }
            }
        }
        "kallisto" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !args.is_empty() {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "kallisto" { args.remove(0); }
                }
                if task_lower.contains("index") || task_lower.contains("build") {
                    if args.is_empty() || args[0].to_ascii_lowercase() != "index" {
                        args = vec!["index".to_string()];
                        if let Some(fa) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                        }) {
                            args.push("-i".to_string());
                            if let Some(out) = tv.output_files.first() {
                                args.push(out.clone());
                            } else {
                                args.push("index.idx".to_string());
                            }
                            args.push(fa.clone());
                        }
                    }
                } else {
                    if args.is_empty() || args[0].to_ascii_lowercase() != "quant" {
                        args.insert(0, "quant".to_string());
                    }
                    if !args.iter().any(|a| a == "-i" || a == "--index") {
                        if let Some(idx) = tv.reference_files.first().cloned().or_else(|| tv.input_files.iter().find(|f| f.contains("index") || f.contains("idx")).cloned()) {
                            args.push("-i".to_string());
                            args.push(idx);
                        }
                    }
                    if !args.iter().any(|a| a == "-o" || a == "--output-dir") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-o".to_string());
                            args.push(out.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-b" || a == "--bootstrap-samples") {
                        args.push("-b".to_string());
                        args.push("100".to_string());
                    }
                    if task_lower.contains("single") || task_lower.contains("single-end") {
                        if !args.iter().any(|a| a == "--single") {
                            args.push("--single".to_string());
                            args.push("-l".to_string());
                            args.push("200".to_string());
                            args.push("-s".to_string());
                            args.push("20".to_string());
                        }
                    }
                }
            }
        }
        "arriba" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                if !args.is_empty() {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "arriba" { args.remove(0); }
                }
                let correct_sub = if task_lower.contains("draw") || task_lower.contains("visualize") {
                    Some("draw_fusions.R")
                } else if task_lower.contains("convert") && task_lower.contains("vcf") {
                    Some("convert_fusions_to_vcf")
                } else if task_lower.contains("wrapper") || task_lower.contains("prealigned") {
                    Some("run_arriba_on_prealigned_bam")
                } else if task_lower.contains("pipeline") || task_lower.contains("full") || task_lower.contains("run_arriba") {
                    Some("run_arriba")
                } else {
                    None
                };
                if let Some(sub) = correct_sub {
                    if args.is_empty() || args[0].to_ascii_lowercase() != sub.to_ascii_lowercase() {
                        args.insert(0, sub.to_string());
                    }
                }
                if args.iter().any(|a| a.to_ascii_lowercase() == "run_arriba") {
                    if !args.iter().any(|a| a == "-x" || a == "--star-index") {
                        if let Some(idx) = tv.genome_dirs.first().cloned().or_else(|| tv.input_files.iter().find(|f| f.contains("star") || f.contains("index")).cloned()) {
                            args.push("-x".to_string());
                            args.push(idx);
                        }
                    }
                }
            }
        }
        "wget" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                let has_url = args.iter().any(|a| a.starts_with("http://") || a.starts_with("https://") || a.starts_with("ftp://"));
                if !has_url {
                    let url = tv.input_files.iter().find(|f| f.starts_with("http://") || f.starts_with("https://") || f.starts_with("ftp://")).cloned();
                    if let Some(u) = url {
                        args.push(u);
                    }
                }
                let has_output = args.iter().any(|a| a == "-O" || a == "--output-document");
                if !has_output {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-O".to_string());
                        args.push(out.clone());
                    }
                }
                for i in 0..args.len() {
                    if args[i].starts_with("--body-data=STRING") {
                        args[i] = args[i].replace("--body-data=STRING", "");
                        if args[i].is_empty() {
                            args.remove(i);
                            break;
                        }
                    }
                }
                args.retain(|a| !a.starts_with("--body-data=STRING"));
            }
        }
        "curl" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                let has_url = args.iter().any(|a| a.starts_with("http://") || a.starts_with("https://") || a.starts_with("ftp://"));
                if !has_url {
                    let url = tv.input_files.iter().find(|f| f.starts_with("http://") || f.starts_with("https://") || f.starts_with("ftp://")).cloned();
                    if let Some(u) = url {
                        args.push(u);
                    }
                }
            }
        }
        "porechop" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "reads" || first_lower == "read" || first_lower == "fastq"
                        || first_lower == "discard_middle" || first_lower == "discard"
                        || first_lower == "trim" || first_lower == "adapter"
                        || first_lower == "porechop" || first_lower == "chop" {
                        args.remove(0);
                    } else {
                        break;
                    }
                }
                if !args.iter().any(|a| a == "-i" || a == "--input") {
                    if let Some(fq) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }) {
                        args.push("-i".to_string());
                        args.push(fq.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o" || a == "--output") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                for i in 0..args.len() {
                    if (args[i] == "-o" || args[i] == "--output") && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if out_lower.ends_with(".bam") {
                            args[i + 1] = args[i + 1].replace(".bam", ".fastq.gz").replace(".BAM", ".fastq.gz");
                        }
                    }
                }
                if !args.iter().any(|a| a == "--threads") {
                    args.push("--threads".to_string());
                    args.push("4".to_string());
                }
            }
        }
        "bracken" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "threshold" || first_lower == "bracken"
                        || first_lower == "classify" || first_lower == "abundance" {
                        args.remove(0);
                    } else {
                        break;
                    }
                }
                let task_lower = task.to_ascii_lowercase();
                let is_build = task_lower.contains("build") || task_lower.contains("database") || task_lower.contains("create db");
                if is_build {
                    if args.is_empty() || args[0].to_ascii_lowercase() != "bracken-build" {
                        args.insert(0, "bracken-build".to_string());
                    }
                    if !args.iter().any(|a| a == "-d") {
                        let db = tv.database_files.first().cloned()
                            .unwrap_or_else(|| "/path/to/kraken2_db".to_string());
                        args.push("-d".to_string());
                        args.push(db);
                    }
                    if !args.iter().any(|a| a == "-k") {
                        args.push("-k".to_string());
                        args.push("35".to_string());
                    }
                    if !args.iter().any(|a| a == "-l") {
                        args.push("-l".to_string());
                        args.push("150".to_string());
                    }
                    if !args.iter().any(|a| a == "-y") {
                        args.push("-y".to_string());
                        args.push("kraken2".to_string());
                    }
                } else {
                    if !args.iter().any(|a| a == "-d") {
                        let db = tv.database_files.first().cloned()
                            .unwrap_or_else(|| "/path/to/kraken2_db".to_string());
                        args.push("-d".to_string());
                        args.push(db);
                    }
                    if !args.iter().any(|a| a == "-i") {
                        if let Some(inp) = tv.input_files.first() {
                            args.push("-i".to_string());
                            args.push(inp.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-o") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-o".to_string());
                            args.push(out.clone());
                        }
                    }
                    for i in 0..args.len() {
                        if args[i] == "-o" && i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".bam") {
                                args[i + 1] = args[i + 1].replace(".bam", ".bracken");
                            }
                        }
                    }
                    if !args.iter().any(|a| a == "-l") {
                        args.push("-l".to_string());
                        args.push("S".to_string());
                    }
                    if !args.iter().any(|a| a == "-r") {
                        args.push("-r".to_string());
                        args.push("150".to_string());
                    }
                }
            }
        }
        "wtdbg2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                let valid_flags: &[&str] = &["-x", "-g", "-i", "-fo", "-t", "-L", "-f", "--edge-min", "--rescue-low-cov"];
                let mut clean_args = Vec::new();
                let mut skip_next = false;
                for (idx, arg) in args.iter().enumerate() {
                    if skip_next { skip_next = false; continue; }
                    if arg.starts_with('-') {
                        if valid_flags.iter().any(|f| *f == arg.as_str()) {
                            clean_args.push(arg.clone());
                            if idx + 1 < args.len() && !args[idx + 1].starts_with('-') {
                                clean_args.push(args[idx + 1].clone());
                                skip_next = true;
                            }
                        }
                    } else if arg.contains('.') || arg.contains('/') {
                        clean_args.push(arg.clone());
                    }
                }
                args = clean_args;
                if !args.iter().any(|a| a == "-x") {
                    let preset = if task_lower.contains("hifi") || task_lower.contains("ccs") || task_lower.contains("pacbio hifi") {
                        "ccs"
                    } else if task_lower.contains("rs") || task_lower.contains("clr") || task_lower.contains("pacbio clr") {
                        "rs"
                    } else {
                        "ont"
                    };
                    args.insert(0, "-x".to_string());
                    args.insert(1, preset.to_string());
                }
                if !args.iter().any(|a| a == "-g") {
                    let g_pos = args.iter().position(|a| a == "-x").map(|p| p + 2).unwrap_or(2);
                    args.insert(g_pos, "-g".to_string());
                    args.insert(g_pos + 1, "5m".to_string());
                }
                if !args.iter().any(|a| a == "-i") {
                    if let Some(fq) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }) {
                        args.push("-i".to_string());
                        args.push(fq.clone());
                    }
                }
                if !args.iter().any(|a| a == "-fo") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-fo".to_string());
                        args.push(out.clone());
                    } else {
                        args.push("-fo".to_string());
                        args.push("assembly".to_string());
                    }
                }
                if !args.iter().any(|a| a == "-t") {
                    args.push("-t".to_string());
                    args.push("16".to_string());
                }
            }
        }
        "pairtools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "pairtools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("parse") { "parse" }
                        else if task_lower.contains("sort") { "sort" }
                        else if task_lower.contains("dedup") { "dedup" }
                        else if task_lower.contains("cload") || task_lower.contains("load") { "cload" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("flip") { "flip" }
                        else if task_lower.contains("restrict") { "restrict" }
                        else { "parse" };
                    if args.is_empty() || args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "repeatmasker" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "repeatmasker" || first_lower == "mask" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-species") {
                    let species = if task_lower.contains("human") { "human" }
                        else if task_lower.contains("mouse") { "mouse" }
                        else if task_lower.contains("arabidopsis") { "arabidopsis" }
                        else if task_lower.contains("fly") || task_lower.contains("drosophila") { "drosophila" }
                        else if task_lower.contains("yeast") { "yeast" }
                        else if task_lower.contains("ecoli") || task_lower.contains("e.coli") { "ecoli" }
                        else if task_lower.contains("zebrafish") { "zebrafish" }
                        else { "human" };
                    args.push("-species".to_string());
                    args.push(species.to_string());
                }
                if !args.iter().any(|a| a == "-dir") {
                    if let Some(out) = tv.output_files.first() {
                        let dir = if out.ends_with('/') { out.clone() } else { format!("{}/", out) };
                        args.push("-dir".to_string());
                        args.push(dir);
                    }
                }
                if !args.iter().any(|a| a == "-pa") {
                    args.push("-pa".to_string());
                    args.push("8".to_string());
                }
                if !args.iter().any(|a| a == "-xsmall") {
                    args.push("-xsmall".to_string());
                }
            }
        }
        "spades" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    args.remove(0);
                }
                let has_mode = args.iter().any(|a| a.starts_with("--meta") || a.starts_with("--plasmid")
                    || a.starts_with("--sc") || a.starts_with("--isolate") || a.starts_with("--rnaviral")
                    || a.starts_with("--corona") || a.starts_with("--bio") || a.starts_with("--rna"));
                if !has_mode {
                    if task_lower.contains("meta") { args.push("--meta".to_string()); }
                    else if task_lower.contains("plasmid") { args.push("--plasmid".to_string()); }
                    else if task_lower.contains("single cell") || task_lower.contains("sc") { args.push("--sc".to_string()); }
                    else if task_lower.contains("isolate") { args.push("--isolate".to_string()); }
                    else if task_lower.contains("rna") || task_lower.contains("viral") { args.push("--rnaviral".to_string()); }
                    else if task_lower.contains("corona") { args.push("--corona".to_string()); }
                    else if task_lower.contains("bio") { args.push("--bio".to_string()); }
                }
                if !args.iter().any(|a| a == "-1") {
                    let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }).collect();
                    if fq_files.len() >= 2 {
                        args.push("-1".to_string());
                        args.push(fq_files[0].clone());
                        args.push("-2".to_string());
                        args.push(fq_files[1].clone());
                    } else if let Some(fq) = fq_files.first() {
                        args.push("-1".to_string());
                        args.push((*fq).clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    args.push("-o".to_string());
                    args.push("spades_output/".to_string());
                }
                if !args.iter().any(|a| a == "--memory") {
                    args.push("--memory".to_string());
                    args.push("32".to_string());
                }
            }
        }
        "macs2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "macs2" { args.remove(0); } else { break; }
                }
                if args.is_empty() || (!args[0].starts_with('-') && args[0] != "callpeak" && args[0] != "predictd" && args[0] != "bdgcmp" && args[0] != "bdgopt") {
                    let subcmd = if task_lower.contains("predict") || task_lower.contains("model") { "predictd" }
                        else { "callpeak" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args[0] = subcmd.to_string();
                    }
                }
                if args.iter().any(|a| a == "callpeak") {
                    if !args.iter().any(|a| a == "-t") {
                        let bam_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                            f.to_ascii_lowercase().ends_with(".bam")
                        }).collect();
                        if let Some(bam) = bam_files.first() {
                            args.push("-t".to_string());
                            args.push((*bam).clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-c") {
                        let bam_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                            f.to_ascii_lowercase().ends_with(".bam")
                        }).collect();
                        if bam_files.len() >= 2 {
                            args.push("-c".to_string());
                            args.push(bam_files[1].clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-f") {
                        if task_lower.contains("bampe") || task_lower.contains("paired-end") {
                            args.push("-f".to_string());
                            args.push("BAMPE".to_string());
                        } else {
                            args.push("-f".to_string());
                            args.push("BAM".to_string());
                        }
                    }
                    if !args.iter().any(|a| a == "-g") {
                        args.push("-g".to_string());
                        args.push("hs".to_string());
                    }
                    if !args.iter().any(|a| a == "-n") {
                        args.push("-n".to_string());
                        args.push("sample".to_string());
                    }
                    if !args.iter().any(|a| a == "--outdir") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("--outdir".to_string());
                            args.push(out.clone());
                        }
                    }
                }
            }
        }
        "delly" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "delly" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("filter") { "filter" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("cnv") { "cnv" }
                        else if task_lower.contains("lr") { "lr" }
                        else { "call" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "cnvkit" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "cnvkit" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("batch") { "batch" }
                        else if task_lower.contains("scatter") { "scatter" }
                        else if task_lower.contains("call") { "call" }
                        else if task_lower.contains("segment") { "segment" }
                        else if task_lower.contains("heatmap") { "heatmap" }
                        else if task_lower.contains("genemetrics") { "genemetrics" }
                        else { "batch" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "liftoff" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "liftoff" { args.remove(0); } else { break; }
                }
                for i in 0..args.len() {
                    if args[i] == "-o" && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        if out_lower.ends_with(".bam") {
                            args[i + 1] = args[i + 1].replace(".bam", ".gff3");
                        }
                    }
                }
                if !args.iter().any(|a| a == "-u") {
                    args.push("-u".to_string());
                    args.push("unmapped.txt".to_string());
                }
            }
        }
        "featurecounts" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "featurecounts" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-a") {
                    let gtf = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".saf")
                    });
                    if let Some(g) = gtf {
                        args.push("-a".to_string());
                        args.push(g.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-s") {
                    args.push("-s".to_string());
                    args.push("2".to_string());
                }
                if !args.iter().any(|a| a == "-p") && (task_lower.contains("paired") || task_lower.contains("pair")) {
                    args.push("-p".to_string());
                }
            }
        }
        "freebayes" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "freebayes" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-f") {
                    if let Some(fa) = tv.reference_files.first().cloned().or_else(|| tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    }).cloned()) {
                        args.push("-f".to_string());
                        args.push(fa);
                    }
                }
                if !args.iter().any(|a| a == "-b") {
                    if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                        args.push("-b".to_string());
                        args.push(bam.clone());
                    }
                }
            }
        }
        "whatshap" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "whatshap" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("haplotag") { "haplotag" }
                        else if task_lower.contains("stat") { "stats" }
                        else { "phase" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
                if args.iter().any(|a| a == "phase") {
                    if !args.iter().any(|a| a == "--reference") {
                        if let Some(fa) = tv.reference_files.first().cloned().or_else(|| tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fa") || fl.ends_with(".fasta")
                        }).cloned()) {
                            args.push("--reference".to_string());
                            args.push(fa);
                        }
                    }
                    if !args.iter().any(|a| a == "--output") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("--output".to_string());
                            args.push(out.clone());
                        }
                    }
                }
            }
        }
        "orthofinder" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "orthofinder" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-f") {
                    if let Some(dir) = tv.genome_dirs.first() {
                        args.push("-f".to_string());
                        args.push(dir.clone());
                    } else if let Some(input) = tv.input_files.first() {
                        args.push("-f".to_string());
                        args.push(input.clone());
                    }
                }
                if !args.iter().any(|a| a == "-t") {
                    args.push("-t".to_string());
                    args.push("16".to_string());
                }
            }
        }
        "nextflow" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "nextflow" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("run") { "run" }
                        else if task_lower.contains("pull") { "pull" }
                        else if task_lower.contains("list") { "list" }
                        else if task_lower.contains("clean") { "clean" }
                        else if task_lower.contains("config") { "config" }
                        else if task_lower.contains("info") || task_lower.contains("version") { "-version" }
                        else { "run" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "metabat2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "metabat2" || first_lower == "metabat" { args.remove(0); } else { break; }
                }
                if task_lower.contains("jgi") || task_lower.contains("depth") || task_lower.contains("summarize") {
                    if args.is_empty() || args[0] != "jgi_summarize_bam_contig_depths" {
                        args = vec!["jgi_summarize_bam_contig_depths".to_string()];
                        args.push("--outputDepth".to_string());
                        if let Some(out) = tv.output_files.first() {
                            args.push(out.clone());
                        }
                        let bams: Vec<&String> = tv.input_files.iter().filter(|f| f.to_ascii_lowercase().ends_with(".bam")).collect();
                        for bam in bams {
                            args.push(bam.clone());
                        }
                    }
                } else {
                    if !args.iter().any(|a| a == "-i") {
                        if let Some(fa) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                        }) {
                            args.push("-i".to_string());
                            args.push(fa.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-a") {
                        if let Some(depth) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".txt")) {
                            args.push("-a".to_string());
                            args.push(depth.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-o") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-o".to_string());
                            args.push(out.clone());
                        }
                    }
                }
            }
        }
        "checkm2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "checkm2" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("database") || task_lower.contains("download") { "database" }
                        else if task_lower.contains("testrun") || task_lower.contains("test") { "testrun" }
                        else { "predict" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
                if args.iter().any(|a| a == "predict") {
                    if !args.iter().any(|a| a == "--input") {
                        args.push("--input".to_string());
                        args.push("bins_directory/".to_string());
                    }
                    if !args.iter().any(|a| a == "--output-directory") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("--output-directory".to_string());
                            args.push(out.clone());
                        }
                    }
                }
            }
        }
        "nanoplot" | "nanostat" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "nanoplot" || first_lower == "nanostat" { args.remove(0); } else { break; }
                }
                if tool_lower == "nanoplot" {
                    if !args.iter().any(|a| a == "--fastq" || a == "--bam" || a == "--summary") {
                        if let Some(fq) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                        }) {
                            args.push("--fastq".to_string());
                            args.push(fq.clone());
                        } else if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                            args.push("--bam".to_string());
                            args.push(bam.clone());
                        }
                    }
                    for i in 0..args.len() {
                        if args[i] == "--bam" && i + 1 < args.len() {
                            if args[i + 1].to_ascii_lowercase().ends_with(".fastq") || args[i + 1].to_ascii_lowercase().ends_with(".fq") {
                                args[i] = "--fastq".to_string();
                            }
                        }
                        if args[i] == "--fastq" && i + 1 < args.len() {
                            if args[i + 1].to_ascii_lowercase().ends_with(".bam") {
                                args[i] = "--bam".to_string();
                            }
                        }
                    }
                    if !args.iter().any(|a| a == "-o") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-o".to_string());
                            args.push(out.clone());
                        }
                    }
                } else if tool_lower == "nanostat" {
                    let mut i = 0;
                    while i < args.len() {
                        if args[i] == "-o" {
                            args.remove(i);
                            if i < args.len() { args.remove(i); }
                        } else { i += 1; }
                    }
                    if !args.iter().any(|a| a == "--fastq" || a == "--bam" || a == "--summary") {
                        if let Some(fq) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".fastq.gz") || fl.ends_with(".fq.gz") || (fl.ends_with(".gz") && !fl.ends_with(".bam"))
                        }) {
                            args.push("--fastq".to_string());
                            args.push(fq.clone());
                        } else if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                            args.push("--bam".to_string());
                            args.push(bam.clone());
                        } else if let Some(txt) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".txt") || fl.ends_with(".log")
                        }) {
                            args.push("--summary".to_string());
                            args.push(txt.clone());
                        }
                    }
                    for j in 0..args.len() {
                        if args[j] == "--fastq" && j + 1 < args.len() {
                            let val = args[j + 1].to_ascii_lowercase();
                            if val.ends_with(".bam") {
                                args[j] = "--bam".to_string();
                            } else if val.ends_with(".txt") || val.ends_with(".log") {
                                args[j] = "--summary".to_string();
                            }
                        }
                        if args[j] == "--bam" && j + 1 < args.len() {
                            let val = args[j + 1].to_ascii_lowercase();
                            if val.ends_with(".fastq") || val.ends_with(".fq") || val.ends_with(".gz") {
                                args[j] = "--fastq".to_string();
                            } else if val.ends_with(".txt") || val.ends_with(".log") {
                                args[j] = "--summary".to_string();
                            }
                        }
                    }
                    if !args.iter().any(|a| a == "--name") {
                        args.push("--name".to_string());
                        args.push("sample_name".to_string());
                    }
                }
            }
        }
        "mmseqs2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "mmseqs" || first_lower == "mmseqs2" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("createdb") { "createdb" }
                        else if task_lower.contains("search") { "search" }
                        else if task_lower.contains("easy-search") || task_lower.contains("easy") { "easy-search" }
                        else if task_lower.contains("cluster") { "cluster" }
                        else if task_lower.contains("linclust") { "linclust" }
                        else if task_lower.contains("taxonomy") { "taxonomy" }
                        else { "easy-search" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "sourmash" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "sourmash" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("sketch") { "sketch" }
                        else if task_lower.contains("compare") { "compare" }
                        else if task_lower.contains("gather") { "gather" }
                        else if task_lower.contains("search") { "search" }
                        else if task_lower.contains("index") { "index" }
                        else if task_lower.contains("taxonomy") { "taxonomy" }
                        else { "sketch" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
                if args.iter().any(|a| a == "sketch") {
                    if !args.iter().any(|a| a == "dna" || a == "protein" || a == "translate") {
                        args.push("dna".to_string());
                    }
                    if !args.iter().any(|a| a == "-p") {
                        args.push("-p".to_string());
                        args.push("k=31,scaled=1000".to_string());
                    }
                    if !args.iter().any(|a| a == "-o") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-o".to_string());
                            args.push(out.clone());
                        }
                    }
                    for i in 0..args.len() {
                        if args[i] == "-o" && i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".bam") {
                                args[i + 1] = args[i + 1].replace(".bam", ".sig");
                            }
                        }
                    }
                }
            }
        }
        "salmon" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "salmon" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("index") || task_lower.contains("build") { "index" }
                        else { "quant" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "bismark" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bismark" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("genome_preparation") || task_lower.contains("prepare") || task_lower.contains("index") || task_lower.contains("build") {
                        "bismark_genome_preparation"
                    } else if task_lower.contains("deduplicate") || task_lower.contains("dedup") {
                        "deduplicate_bismark"
                    } else if task_lower.contains("methylation_extractor") || task_lower.contains("extractor") || task_lower.contains("extract") {
                        "bismark_methylation_extractor"
                    } else if task_lower.contains("report") || task_lower.contains("summary") {
                        "bismark2report"
                    } else {
                        ""
                    };
                    if !subcmd.is_empty() && (args.is_empty() || args[0] != subcmd) {
                        if args.is_empty() {
                            args.push(subcmd.to_string());
                        } else {
                            args[0] = subcmd.to_string();
                        }
                    }
                }
            }
        }
        "hisat2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "hisat2" { args.remove(0); } else { break; }
                }
                if task_lower.contains("build") || task_lower.contains("index") {
                    if args.is_empty() || args[0].to_ascii_lowercase() != "hisat2-build" {
                        args = vec!["hisat2-build".to_string()];
                        if let Some(fa) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                        }) {
                            args.push(fa.clone());
                            args.push("reference_index".to_string());
                        }
                    }
                } else {
                    if !args.iter().any(|a| a == "-x") {
                        if let Some(idx) = tv.reference_files.first().cloned().or_else(|| tv.input_files.iter().find(|f| f.contains("index") || f.contains("genome")).cloned()) {
                            args.push("-x".to_string());
                            args.push(idx);
                        }
                    }
                    if !args.iter().any(|a| a == "-1" || a == "-U" || a == "-r") {
                        let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                        }).collect();
                        if fq_files.len() >= 2 {
                            args.push("-1".to_string());
                            args.push(fq_files[0].clone());
                            args.push("-2".to_string());
                            args.push(fq_files[1].clone());
                        } else if let Some(fq) = fq_files.first() {
                            args.push("-U".to_string());
                            args.push((*fq).clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-S") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-S".to_string());
                            args.push(out.clone());
                        }
                    }
                }
            }
        }
        "rm" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "rm" || first_lower == "remove" || first_lower == "delete" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-r" || a == "-rf" || a == "-v" || a == "-d" || a == "-i" || a == "-f") {
                    if task_lower.contains("recursive") || task_lower.contains("directory") || task_lower.contains("dir") {
                        args.push("-rf".to_string());
                    } else if task_lower.contains("verbose") {
                        args.push("-v".to_string());
                    } else if task_lower.contains("interactive") {
                        args.push("-i".to_string());
                    } else if task_lower.contains("force") {
                        args.push("-f".to_string());
                    } else if task_lower.contains("empty") || task_lower.contains("directory") {
                        args.push("-d".to_string());
                    }
                }
                let has_target = args.iter().any(|a| {
                    !a.starts_with('-') && (a.contains(".") || a.contains("/") || a.contains("*") || a.contains("dir"))
                });
                if !has_target {
                    if let Some(target) = tv.input_files.first() {
                        args.push(target.clone());
                    }
                }
            }
        }
        "tar" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "tar" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a.starts_with('-') && (a.contains('c') || a.contains('x') || a.contains('t'))) {
                    if task_lower.contains("extract") || task_lower.contains("decompress") || task_lower.contains("untar") {
                        args.push("-xzf".to_string());
                    } else {
                        args.push("-czf".to_string());
                    }
                }
                for i in 0..args.len() {
                    if args[i] == "-czf" || args[i] == "-xzf" || args[i] == "-cf" || args[i] == "-xf" {
                        if i + 1 < args.len() {
                            let out_lower = args[i + 1].to_ascii_lowercase();
                            if out_lower.ends_with(".bam") {
                                args[i + 1] = args[i + 1].replace(".bam", ".tar.gz");
                            }
                        }
                    }
                }
            }
        }
        "tabix" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "tabix" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a.ends_with(".gz") && !a.starts_with('-')) {
                    if let Some(vcf) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".gz")) {
                        args.push(vcf.clone());
                    }
                }
            }
        }
        "kb" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "kb" || first_lower == "kallisto" || first_lower == "bustools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("ref") || task_lower.contains("reference") || task_lower.contains("index") { "ref" }
                        else if task_lower.contains("count") || task_lower.contains("quant") { "count" }
                        else { "ref" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "snpeff" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "snpeff" || first_lower == "snpeff" { args.remove(0); } else { break; }
                }
            }
        }
        "strelka2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "strelka" || first_lower == "strelka2" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("somatic") { "configureSomaticWorkflow.py" }
                        else if task_lower.contains("germline") { "configureGermlineWorkflow.py" }
                        else { "configureSomaticWorkflow.py" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "varscan2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "varscan" || first_lower == "varscan2" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("somatic") { "somatic" }
                        else if task_lower.contains("copynumber") || task_lower.contains("cnv") { "copynumber" }
                        else if task_lower.contains("mpileup") || task_lower.contains("pileup") { "mpileup2cns" }
                        else { "mpileup2snv" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "plink2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "plink" || first_lower == "plink2" { args.remove(0); } else { break; }
                }
            }
        }
        "mummer" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "mummer" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("nucmer") { "nucmer" }
                        else if task_lower.contains("promer") { "promer" }
                        else if task_lower.contains("dnadiff") { "dnadiff" }
                        else if task_lower.contains("show") || task_lower.contains("coords") { "show-coords" }
                        else if task_lower.contains("delta") || task_lower.contains("filter") { "delta-filter" }
                        else { "nucmer" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "homer" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "homer" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("findmotif") || task_lower.contains("motif") { "findMotifsGenome.pl" }
                        else if task_lower.contains("annotate") { "annotatePeaks.pl" }
                        else if task_lower.contains("make") || task_lower.contains("tag") { "makeTagDirectory" }
                        else { "findMotifsGenome.pl" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "deeptools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "deeptools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("bamcoverage") || task_lower.contains("coverage") { "bamCoverage" }
                        else if task_lower.contains("computematrix") || task_lower.contains("matrix") { "computeMatrix" }
                        else if task_lower.contains("plot") && task_lower.contains("heatmap") { "plotHeatmap" }
                        else if task_lower.contains("plot") && task_lower.contains("profile") { "plotProfile" }
                        else if task_lower.contains("plottingerprint") || task_lower.contains("fingerprint") { "plotFingerprint" }
                        else if task_lower.contains("multibamsummary") || task_lower.contains("correlation") { "multiBamSummary" }
                        else if task_lower.contains("plotcorrelation") { "plotCorrelation" }
                        else if task_lower.contains("bamcoverage") { "bamCoverage" }
                        else if task_lower.contains("estimateinsertsize") || task_lower.contains("insert") { "estimateInsertSize" }
                        else { "bamCoverage" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "igvtools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "igvtools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("count") || task_lower.contains("tdf") { "count" }
                        else if task_lower.contains("index") { "index" }
                        else if task_lower.contains("sort") { "sort" }
                        else { "count" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "trimmomatic" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "trimmomatic" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "PE" || a == "SE") {
                    if task_lower.contains("paired") || task_lower.contains("pair") {
                        args.insert(0, "PE".to_string());
                    } else {
                        args.insert(0, "SE".to_string());
                    }
                }
            }
        }
        "star" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "star" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--runMode") {
                    args.push("--runMode".to_string());
                    args.push("alignReads".to_string());
                }
                if !args.iter().any(|a| a == "--genomeDir") {
                    if let Some(dir) = tv.genome_dirs.first() {
                        args.push("--genomeDir".to_string());
                        args.push(dir.clone());
                    }
                }
                if !args.iter().any(|a| a == "--readFilesIn") {
                    let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }).collect();
                    if !fq_files.is_empty() {
                        args.push("--readFilesIn".to_string());
                        args.push(fq_files[0].clone());
                        if fq_files.len() >= 2 {
                            args.push(fq_files[1].clone());
                        }
                    }
                }
                if !args.iter().any(|a| a == "--outSAMtype") {
                    args.push("--outSAMtype".to_string());
                    args.push("BAM".to_string());
                    args.push("SortedByCoordinate".to_string());
                }
                if !args.iter().any(|a| a == "--outFileNamePrefix") {
                    args.push("--outFileNamePrefix".to_string());
                    args.push("sample/".to_string());
                }
            }
        }
        "trinity" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "trinity" || first_lower == "trinityrnaseq" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--seqType") {
                    args.push("--seqType".to_string());
                    args.push("fq".to_string());
                }
                if !args.iter().any(|a| a == "--left" || a == "--single") {
                    let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }).collect();
                    if fq_files.len() >= 2 {
                        args.push("--left".to_string());
                        args.push(fq_files[0].clone());
                        args.push("--right".to_string());
                        args.push(fq_files[1].clone());
                    } else if let Some(fq) = fq_files.first() {
                        args.push("--single".to_string());
                        args.push((*fq).clone());
                    }
                }
                if !args.iter().any(|a| a == "--output") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("--output".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "stringtie" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "stringtie" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-G") {
                    if let Some(gtf) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".gtf") || fl.ends_with(".gff")
                    }) {
                        args.push("-G".to_string());
                        args.push(gtf.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-e") && task_lower.contains("estimate") {
                    args.push("-e".to_string());
                }
            }
        }
        "methyldackel" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "methyldackel" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("extract") { "extract" }
                        else if task_lower.contains("mbias") { "mbias" }
                        else { "extract" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "modkit" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "modkit" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("pileup") { "pileup" }
                        else if task_lower.contains("summary") { "summary" }
                        else if task_lower.contains("call") { "call-mods" }
                        else { "pileup" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "chromap" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "chromap" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("index") { "index" }
                        else { "" };
                    if !subcmd.is_empty() && args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "bamtools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bamtools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("sort") { "sort" }
                        else if task_lower.contains("index") { "index" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("split") { "split" }
                        else if task_lower.contains("stats") || task_lower.contains("flagstat") { "stats" }
                        else if task_lower.contains("filter") { "filter" }
                        else if task_lower.contains("convert") { "convert" }
                        else { "sort" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "seqkit" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "seqkit" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("seq") { "seq" }
                        else if task_lower.contains("grep") { "grep" }
                        else if task_lower.contains("stats") { "stats" }
                        else if task_lower.contains("fx2tab") { "fx2tab" }
                        else if task_lower.contains("sort") { "sort" }
                        else if task_lower.contains("rmdup") { "rmdup" }
                        else if task_lower.contains("sample") { "sample" }
                        else if task_lower.contains("split") { "split" }
                        else { "seq" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "seqtk" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "seqtk" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("seq") { "seq" }
                        else if task_lower.contains("subseq") { "subseq" }
                        else if task_lower.contains("sample") { "sample" }
                        else if task_lower.contains("trimfq") { "trimfq" }
                        else { "seq" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "mosdepth" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "mosdepth" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--by") {
                    args.push("--by".to_string());
                    args.push("10000".to_string());
                }
            }
        }
        "pilon" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "pilon" || first_lower == "java" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--genome") {
                    if let Some(fa) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta")
                    }) {
                        args.push("--genome".to_string());
                        args.push(fa.clone());
                    }
                }
                if !args.iter().any(|a| a == "--frags") {
                    if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                        args.push("--frags".to_string());
                        args.push(bam.clone());
                    }
                }
                if !args.iter().any(|a| a == "--output") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("--output".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "--fix") {
                    args.push("--fix".to_string());
                    args.push("all".to_string());
                }
            }
        }
        "megahit" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "megahit" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-r") {
                    let fq_files: Vec<&String> = tv.input_files.iter().filter(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }).collect();
                    if !fq_files.is_empty() {
                        args.push("-r".to_string());
                        args.push(fq_files.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(","));
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    args.push("-o".to_string());
                    args.push("megahit_output/".to_string());
                }
                if !args.iter().any(|a| a == "-t") {
                    args.push("-t".to_string());
                    args.push("16".to_string());
                }
            }
        }
        "angsd" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "angsd" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-bam") {
                    if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                        args.push("-bam".to_string());
                        args.push(bam.clone());
                    }
                }
            }
        }
        "shapeit4" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "shapeit4" || first_lower == "shapeit" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--input") {
                    if let Some(vcf) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".vcf") || fl.ends_with(".bcf") || fl.ends_with(".vcf.gz")
                    }) {
                        args.push("--input".to_string());
                        args.push(vcf.clone());
                    }
                }
                if !args.iter().any(|a| a == "--output") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("--output".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "--region") {
                    args.push("--region".to_string());
                    args.push("chr1".to_string());
                }
            }
        }
        "pbmm2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "pbmm2" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("index") { "index" }
                        else { "align" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "pbccs" => {
            if let Some(task) = task {
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "pbccs" { args.remove(0); } else { break; }
                }
                if args.is_empty() || (!args[0].starts_with('-') && args[0].to_ascii_lowercase() != "ccs") {
                    args.insert(0, "ccs".to_string());
                }
            }
        }
        "pbsv" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "pbsv" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("discover") { "discover" }
                        else if task_lower.contains("call") { "call" }
                        else { "discover" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "longshot" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "longshot" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--bam") {
                    if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam")) {
                        args.push("--bam".to_string());
                        args.push(bam.clone());
                    }
                }
                if !args.iter().any(|a| a == "--ref") {
                    if let Some(fa) = tv.reference_files.first().cloned().or_else(|| tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta")
                    }).cloned()) {
                        args.push("--ref".to_string());
                        args.push(fa);
                    }
                }
                if !args.iter().any(|a| a == "--out") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("--out".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "sniffles" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "sniffles" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-i" || a == "--input") {
                    if let Some(bam) = tv.input_files.iter().find(|f| f.to_ascii_lowercase().ends_with(".bam") || f.to_ascii_lowercase().ends_with(".sam")) {
                        args.push("-i".to_string());
                        args.push(bam.clone());
                    }
                }
                if !args.iter().any(|a| a == "-v" || a == "--vcf") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-v".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "survivor" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "survivor" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("sim") || task_lower.contains("simulate") { "sim" }
                        else if task_lower.contains("filter") { "filter" }
                        else { "merge" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "prodigal" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "prodigal" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-i") {
                    if let Some(fa) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    }) {
                        args.push("-i".to_string());
                        args.push(fa.clone());
                    }
                }
                if !args.iter().any(|a| a == "-a") && task_lower.contains("protein") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-a".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-d") && task_lower.contains("nucleotide") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-d".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") && !task_lower.contains("protein") && !task_lower.contains("nucleotide") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-p") {
                    if task_lower.contains("meta") {
                        args.push("-p".to_string());
                        args.push("meta".to_string());
                    }
                }
            }
        }
        "prokka" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "prokka" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--outdir") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("--outdir".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "--prefix") {
                    args.push("--prefix".to_string());
                    args.push("sample".to_string());
                }
                if !args.iter().any(|a| a == "--kingdom") {
                    if task_lower.contains("bacteria") {
                        args.push("--kingdom".to_string());
                        args.push("Bacteria".to_string());
                    } else if task_lower.contains("archaea") {
                        args.push("--kingdom".to_string());
                        args.push("Archaea".to_string());
                    }
                }
            }
        }
        "diamond" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "diamond" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("makedb") || task_lower.contains("build") || task_lower.contains("index") { "makedb" }
                        else if task_lower.contains("blastp") { "blastp" }
                        else if task_lower.contains("blastx") { "blastx" }
                        else if task_lower.contains("blastn") { "blastn" }
                        else { "blastp" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "fastp" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "fastp" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-i") {
                    if let Some(fq) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }) {
                        args.push("-i".to_string());
                        args.push(fq.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "fastq-screen" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "fastq-screen" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--conf") {
                    args.push("--conf".to_string());
                    args.push("fastq_screen.conf".to_string());
                }
            }
        }
        "fastqc" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "fastqc" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-t") {
                    args.push("-t".to_string());
                    args.push("8".to_string());
                }
            }
        }
        "flye" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "flye" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--nano-raw" || a == "--nano-corr" || a == "--nano-hq" || a == "--pacbio-raw" || a == "--pacbio-corr" || a == "--pacbio-hifi") {
                    if task_lower.contains("hifi") || task_lower.contains("ccs") || task_lower.contains("pacbio-hifi") {
                        args.push("--pacbio-hifi".to_string());
                    } else if task_lower.contains("pacbio") {
                        args.push("--pacbio-raw".to_string());
                    } else if task_lower.contains("hq") || task_lower.contains("high-quality") {
                        args.push("--nano-hq".to_string());
                    } else if task_lower.contains("corrected") || task_lower.contains("corr") {
                        args.push("--nano-corr".to_string());
                    } else {
                        args.push("--nano-raw".to_string());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-g") {
                    args.push("-g".to_string());
                    args.push("5m".to_string());
                }
                if !args.iter().any(|a| a == "--threads") {
                    args.push("--threads".to_string());
                    args.push("16".to_string());
                }
            }
        }
        "miniasm" => {
            if let Some(task) = task {
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "miniasm" { args.remove(0); } else { break; }
                }
            }
        }
        "racon" => {
            if let Some(task) = task {
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "racon" { args.remove(0); } else { break; }
                }
            }
        }
        "canu" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "canu" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-p") {
                    args.push("-p".to_string());
                    args.push("assembly".to_string());
                }
                if !args.iter().any(|a| a == "-d") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-d".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "genomeSize=" || a.starts_with("genomeSize=")) {
                    args.push("genomeSize=5m".to_string());
                }
                if !args.iter().any(|a| a == "-pacbio-raw" || a == "-pacbio-corr" || a == "-pacbio-hifi" || a == "-nanopore-raw" || a == "-nanopore-corr") {
                    if task_lower.contains("hifi") || task_lower.contains("ccs") {
                        args.push("-pacbio-hifi".to_string());
                    } else if task_lower.contains("pacbio") {
                        args.push("-pacbio-raw".to_string());
                    } else {
                        args.push("-nanopore-raw".to_string());
                    }
                    if let Some(fq) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fastq") || fl.ends_with(".fq") || fl.ends_with(".gz")
                    }) {
                        args.push(fq.clone());
                    }
                }
            }
        }
        "gatk" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "gatk" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("haplotypecaller") { "HaplotypeCaller" }
                        else if task_lower.contains("markduplicates") { "MarkDuplicates" }
                        else if task_lower.contains("base") && task_lower.contains("recalibrator") { "BaseRecalibrator" }
                        else if task_lower.contains("applybqsr") { "ApplyBQSR" }
                        else if task_lower.contains("splitncigarreads") { "SplitNCigarReads" }
                        else if task_lower.contains("genomicsdbimport") { "GenomicsDBImport" }
                        else if task_lower.contains("genotypegvcfs") { "GenotypeGVCFs" }
                        else if task_lower.contains("selectvariants") { "SelectVariants" }
                        else if task_lower.contains("variantfiltration") { "VariantFiltration" }
                        else if task_lower.contains("combinegvcfs") { "CombineGVCFs" }
                        else { "HaplotypeCaller" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "bcftools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bcftools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("view") { "view" }
                        else if task_lower.contains("filter") { "filter" }
                        else if task_lower.contains("sort") { "sort" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("concat") { "concat" }
                        else if task_lower.contains("index") { "index" }
                        else if task_lower.contains("stats") { "stats" }
                        else if task_lower.contains("call") { "call" }
                        else if task_lower.contains("mpileup") { "mpileup" }
                        else if task_lower.contains("norm") { "norm" }
                        else if task_lower.contains("annotate") { "annotate" }
                        else if task_lower.contains("query") { "query" }
                        else if task_lower.contains("isec") { "isec" }
                        else { "view" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "samtools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "samtools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("view") { "view" }
                        else if task_lower.contains("sort") { "sort" }
                        else if task_lower.contains("index") { "index" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("flagstat") { "flagstat" }
                        else if task_lower.contains("idxstats") { "idxstats" }
                        else if task_lower.contains("depth") { "depth" }
                        else if task_lower.contains("faidx") { "faidx" }
                        else if task_lower.contains("mpileup") { "mpileup" }
                        else if task_lower.contains("stats") { "stats" }
                        else if task_lower.contains("fastq") { "fastq" }
                        else if task_lower.contains("collate") { "collate" }
                        else if task_lower.contains("calmd") { "calmd" }
                        else { "view" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "bedtools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bedtools" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("intersect") { "intersect" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("sort") { "sort" }
                        else if task_lower.contains("coverage") { "coverage" }
                        else if task_lower.contains("getfasta") { "getfasta" }
                        else if task_lower.contains("slop") { "slop" }
                        else if task_lower.contains("closest") { "closest" }
                        else if task_lower.contains("subtract") { "subtract" }
                        else if task_lower.contains("complement") { "complement" }
                        else if task_lower.contains("window") { "window" }
                        else if task_lower.contains("flank") { "flank" }
                        else if task_lower.contains("makewindows") { "makewindows" }
                        else if task_lower.contains("multicov") { "multicov" }
                        else if task_lower.contains("genomecov") { "genomecov" }
                        else { "intersect" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "vep" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "vep" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-i") {
                    if let Some(vcf) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".vcf") || fl.ends_with(".vcf.gz")
                    }) {
                        args.push("-i".to_string());
                        args.push(vcf.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "vcfanno" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "vcfanno" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a.ends_with(".toml") || a.ends_with(".conf")) {
                    if let Some(conf) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".toml") || fl.ends_with(".conf") || fl.ends_with(".txt")
                    }) {
                        args.push(conf.clone());
                    }
                }
            }
        }
        "cutadapt" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "cutadapt" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "trim_galore" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "trim_galore" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "chopper" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "chopper" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-i") {
                    if let Some(fq) = tv.input_files.first() {
                        args.push("-i".to_string());
                        args.push(fq.clone());
                    }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "admixture" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "admixture" { args.remove(0); } else { break; }
                }
            }
        }
        "blast" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "blast" || first_lower == "blastn" || first_lower == "blastp" || first_lower == "blastx" || first_lower == "tblastn" || first_lower == "makeblastdb" || first_lower == "blastdbcmd" { args.remove(0); } else { break; }
                }
                let subcmd = if task_lower.contains("makeblastdb") || task_lower.contains("makedb") || (task_lower.contains("build") && task_lower.contains("database")) || (task_lower.contains("construct") && task_lower.contains("database")) { "makeblastdb" }
                    else if task_lower.contains("blastdbcmd") || task_lower.contains("retrieve") || task_lower.contains("fetch sequence") || task_lower.contains("accession") { "blastdbcmd" }
                    else if task_lower.contains("blastn") || (task_lower.contains("nucleotide") && task_lower.contains("search")) || (task_lower.contains("similar") && task_lower.contains("nucleotide")) { "blastn" }
                    else if task_lower.contains("blastp") || (task_lower.contains("protein") && task_lower.contains("search")) || (task_lower.contains("nr") && task_lower.contains("protein")) { "blastp" }
                    else if task_lower.contains("blastx") || (task_lower.contains("translate") && task_lower.contains("protein")) || (task_lower.contains("nucleotide") && task_lower.contains("protein")) { "blastx" }
                    else if task_lower.contains("tblastn") { "tblastn" }
                    else if task_lower.contains("remote") || task_lower.contains("ncbi") { "blastn" }
                    else if task_lower.contains("short sequence") || task_lower.contains("blastn-short") { "blastn" }
                    else if task_lower.contains("distant homolog") || task_lower.contains("traditional blastn") { "blastn" }
                    else if task_lower.contains("subject") || task_lower.contains("without database") { "blastn" }
                    else if task_lower.contains("taxid") || task_lower.contains("taxonomy filter") { "blastn" }
                    else { "blastn" };
                if args.is_empty() {
                    args.push(subcmd.to_string());
                } else if !args[0].starts_with('-') && args[0].to_ascii_lowercase() != subcmd {
                    args[0] = subcmd.to_string();
                } else if args[0].starts_with('-') {
                    args.insert(0, subcmd.to_string());
                }
                if subcmd == "makeblastdb" {
                    if !args.iter().any(|a| a == "-in") {
                        if let Some(fa) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna") || fl.ends_with(".faa")
                        }) {
                            args.push("-in".to_string());
                            args.push(fa.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-dbtype") {
                        args.push("-dbtype".to_string());
                        args.push("nucl".to_string());
                    }
                    if !args.iter().any(|a| a == "-out") {
                        args.push("-out".to_string());
                        args.push("genome_db".to_string());
                    }
                    if !args.iter().any(|a| a == "-parse_seqids") {
                        args.push("-parse_seqids".to_string());
                    }
                } else if subcmd == "blastdbcmd" {
                    if !args.iter().any(|a| a == "-db") {
                        args.push("-db".to_string());
                        args.push("genome_db".to_string());
                    }
                    if !args.iter().any(|a| a == "-entry") {
                        args.push("-entry".to_string());
                        args.push("NM_001234".to_string());
                    }
                    if !args.iter().any(|a| a == "-out") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-out".to_string());
                            args.push(out.clone());
                        }
                    }
                } else {
                    if !args.iter().any(|a| a == "-query") {
                        if let Some(fa) = tv.input_files.iter().find(|f| {
                            let fl = f.to_ascii_lowercase();
                            fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna") || fl.ends_with(".faa")
                        }) {
                            args.push("-query".to_string());
                            args.push(fa.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-db") && !args.iter().any(|a| a == "-subject") {
                        args.push("-db".to_string());
                        args.push("genome_db".to_string());
                    }
                    if !args.iter().any(|a| a == "-out") {
                        if let Some(out) = tv.output_files.first() {
                            args.push("-out".to_string());
                            args.push(out.clone());
                        }
                    }
                    if !args.iter().any(|a| a == "-outfmt") {
                        args.push("-outfmt".to_string());
                        args.push("6".to_string());
                    }
                    if !args.iter().any(|a| a == "-evalue") {
                        args.push("-evalue".to_string());
                        args.push("1e-5".to_string());
                    }
                }
                let mut i = 0;
                while i < args.len() {
                    if (args[i] == "-out" || args[i] == "--out") && i + 1 < args.len() {
                        let val = args[i + 1].to_ascii_lowercase();
                        if val.ends_with(".bam") {
                            args[i + 1] = args[i + 1].trim_end_matches(".bam").to_string() + ".txt";
                        }
                    }
                    i += 1;
                }
            }
        }
        "bbtools" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bbtools" || first_lower == "bbmap" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("bbmap") { "bbmap.sh" }
                        else if task_lower.contains("bbduk") || task_lower.contains("trim") || task_lower.contains("filter") { "bbduk.sh" }
                        else if task_lower.contains("reformat") { "reformat.sh" }
                        else if task_lower.contains("bbsplit") { "bbsplit.sh" }
                        else if task_lower.contains("clumpify") { "clumpify.sh" }
                        else { "bbduk.sh" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "bwa" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bwa" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("index") || task_lower.contains("build") { "index" }
                        else if task_lower.contains("mem") { "mem" }
                        else { "mem" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "bwa-mem2" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "bwa-mem2" || first_lower == "bwa" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("index") || task_lower.contains("build") { "index" }
                        else if task_lower.contains("mem") { "mem" }
                        else { "mem" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "picard" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "picard" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("markduplicates") { "MarkDuplicates" }
                        else if task_lower.contains("sortsam") || task_lower.contains("sort") { "SortSam" }
                        else if task_lower.contains("addorreplacereadgroups") { "AddOrReplaceReadGroups" }
                        else if task_lower.contains("collectalignmentsummarymetrics") { "CollectAlignmentSummaryMetrics" }
                        else if task_lower.contains("collectinsertsizemetrics") { "CollectInsertSizeMetrics" }
                        else if task_lower.contains("createsequencedictionary") { "CreateSequenceDictionary" }
                        else { "MarkDuplicates" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "pbfusion" => {
            if let Some(task) = task {
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "pbfusion" { args.remove(0); } else { break; }
                }
            }
        }
        "agat" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "agat" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("sp") && task_lower.contains("fix") { "sp_fix_features_ids_duplicated.pl" }
                        else if task_lower.contains("sp") && task_lower.contains("merge") { "sp_merge_annotations.pl" }
                        else if task_lower.contains("sp") && task_lower.contains("convert") { "sp_gff2gtf.pl" }
                        else if task_lower.contains("sp") && task_lower.contains("statistics") { "sp_statistics.pl" }
                        else { "sp_statistics.pl" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    }
                }
            }
        }
        "augustus" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "augustus" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| !a.starts_with('-') && (a.ends_with(".fa") || a.ends_with(".fasta") || a.ends_with(".fna"))) {
                    if let Some(fa) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    }) {
                        args.push(fa.clone());
                    }
                }
            }
        }
        "mash" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "mash" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("sketch") { "sketch" }
                        else if task_lower.contains("dist") { "dist" }
                        else if task_lower.contains("screen") { "screen" }
                        else if task_lower.contains("paste") { "paste" }
                        else if task_lower.contains("info") { "info" }
                        else { "sketch" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "fastani" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "fastani" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "--query" || a == "-q") {
                    if let Some(fa) = tv.input_files.iter().find(|f| {
                        let fl = f.to_ascii_lowercase();
                        fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    }) {
                        args.push("--query".to_string());
                        args.push(fa.clone());
                    }
                }
                if !args.iter().any(|a| a == "--ref" || a == "-r") {
                    if let Some(fa) = tv.reference_files.first().cloned() {
                        args.push("--ref".to_string());
                        args.push(fa);
                    }
                }
                if !args.iter().any(|a| a == "--output" || a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("--output".to_string());
                        args.push(out.clone());
                    }
                }
            }
        }
        "quast" => {
            if let Some(task) = task {
                let tv = extract_task_values(task);
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "quast" || first_lower == "quast.py" { args.remove(0); } else { break; }
                }
                if !args.iter().any(|a| a == "-o") {
                    if let Some(out) = tv.output_files.first() {
                        args.push("-o".to_string());
                        args.push(out.clone());
                    }
                }
                if !args.iter().any(|a| a == "-r") {
                    if let Some(fa) = tv.reference_files.first().cloned() {
                        args.push("-r".to_string());
                        args.push(fa);
                    }
                }
            }
        }
        "git" => {
            if let Some(task) = task {
                let task_lower = task.to_ascii_lowercase();
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "git" { args.remove(0); } else { break; }
                }
                if args.is_empty() || !args[0].starts_with('-') {
                    let subcmd = if task_lower.contains("clone") { "clone" }
                        else if task_lower.contains("pull") { "pull" }
                        else if task_lower.contains("push") { "push" }
                        else if task_lower.contains("commit") { "commit" }
                        else if task_lower.contains("checkout") { "checkout" }
                        else if task_lower.contains("branch") { "branch" }
                        else if task_lower.contains("log") { "log" }
                        else if task_lower.contains("status") { "status" }
                        else if task_lower.contains("add") { "add" }
                        else if task_lower.contains("diff") { "diff" }
                        else if task_lower.contains("merge") { "merge" }
                        else if task_lower.contains("fetch") { "fetch" }
                        else if task_lower.contains("stash") { "stash" }
                        else { "clone" };
                    if args.is_empty() {
                        args.push(subcmd.to_string());
                    } else if args[0] != subcmd {
                        args.insert(0, subcmd.to_string());
                    }
                }
            }
        }
        "find" => {
            if let Some(task) = task {
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "find" { args.remove(0); } else { break; }
                }
            }
        }
        "rsync" => {
            if let Some(task) = task {
                while !args.is_empty() && !args[0].starts_with('-') {
                    let first_lower = args[0].to_ascii_lowercase();
                    if first_lower == "rsync" { args.remove(0); } else { break; }
                }
            }
        }
        _ => {}
    }

    args
}

fn remove_specific_flags(args: &[String], flags: &[&str]) -> Vec<String> {
    let flags_set: std::collections::HashSet<String> = flags.iter().map(|f| f.to_ascii_lowercase()).collect();
    let mut result = Vec::new();
    let mut skip_next = false;
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if flags_set.contains(&arg.to_ascii_lowercase()) {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                skip_next = true;
            }
            continue;
        }
        result.push(arg.clone());
    }
    result
}

pub fn fix_output_extensions(args: &[String], tool: &str, task: &str) -> Vec<String> {
    let tool_lower = tool.to_ascii_lowercase();
    let mut args = args.to_vec();

    let extension_rules: &[(&str, &[&str], &str)] = &[
        ("porechop", &["-o"], ".fastq.gz"),
        ("muscle", &["-output", "-out"], ".fasta"),
        ("nanocomp", &["--outdir", "-o"], "/"),
        ("sourmash", &["-o"], ".sig"),
        ("metaphlan", &["-o", "--output"], ".txt"),
        ("bakta", &["--output"], "/"),
        ("multiqc", &["-o"], "/"),
        ("qualimap", &["-outdir"], "/"),
        ("seqkit", &["-o"], ".fasta"),
        ("minimap2", &["-o"], ".sam"),
        ("hifiasm", &["-o"], ".fasta"),
        ("kraken2", &["--output"], ".txt"),
        ("vcftools", &["--out"], ""),
        ("mmseqs2", &["-o"], ".m8"),
        ("iqtree2", &["-pre", "-s"], ".treefile"),
        ("nanoplot", &["-o", "--outdir"], "/"),
        ("nanostat", &["-o"], ".txt"),
        ("fastqc", &["-o"], "/"),
        ("trimmomatic", &["-o"], ".fastq.gz"),
        ("wget", &["-O"], ""),
        ("curl", &["-o"], ""),
        ("chopper", &["-o"], ".fastq.gz"),
        ("fastp", &["-o", "-O"], ".fastq.gz"),
        ("cutadapt", &["-o"], ".fastq.gz"),
        ("trim_galore", &["-o", "--output_dir"], "/"),
        ("flye", &["--out-dir", "-o"], "/"),
        ("canu", &["-d"], "/"),
        ("spades", &["-o"], "/"),
        ("megahit", &["-o"], "/"),
        ("bracken", &["-o"], ".txt"),
        ("centrifuge", &["-S"], ".tsv"),
        ("sniffles", &["--output", "-o"], ".vcf"),
        ("freebayes", &["-v"], ".vcf"),
        ("longshot", &["-o"], ".vcf"),
        ("busco", &["-o"], "/"),
        ("prokka", &["--outdir"], "/"),
        ("prodigal", &["-a"], ".faa"),
        ("stringtie", &["-o"], ".gtf"),
        ("featurecounts", &["-o"], ".txt"),
        ("salmon", &["-o"], "/"),
        ("kallisto", &["-o"], "/"),
        ("liftoff", &["-o"], ".gff3"),
        ("repeatmasker", &["-dir"], "/"),
        ("macs2", &["-n"], ""),
        ("mosdepth", &["-o"], ""),
        ("bedtools", &["-fo"], ".bed"),
        ("igvtools", &[], ""),
        ("gtdbtk", &["--out_dir"], "/"),
        ("orthofinder", &["-o"], "/"),
        ("fastani", &["--output"], ".txt"),
        ("plink2", &["--out"], ""),
        ("shapeit4", &["--output"], ".vcf"),
        ("admixture", &[], ""),
        ("angsd", &["-out"], ""),
        ("delly", &["-o"], ".bcf"),
        ("mmseqs2", &["-o"], ".m8"),
        ("racon", &["-o"], ".fasta"),
        ("miniasm", &["-o"], ".fasta"),
        ("pilon", &["--output"], ".fasta"),
        ("quast", &["-o"], "/"),
        ("eggnog-mapper", &["-o"], ".annotations"),
        ("checkm2", &["--output-directory"], "/"),
        ("nextflow", &[], ""),
        ("snakemake", &[], ""),
        ("meme", &["--oc", "-oc"], "/"),
        ("methyldackel", &["-o"], ".csv"),
        ("chromap", &["-o"], ".bed"),
        ("metabat2", &["-o"], "/"),
        ("arriba", &["-o"], ".tsv"),
        ("pbfusion", &["--output-dir"], "/"),
        ("strelka2", &["--runDir"], "/"),
        ("medaka", &["-o"], "/"),
        ("nanocomp", &["--outdir"], "/"),
        ("sra-tools", &["-O"], "/"),
        ("java", &["-O"], ""),
        ("r", &[], ""),
        ("python", &[], ""),
        ("perl", &[], ""),
        ("bash", &[], ""),
        ("julia", &[], ""),
        ("git", &[], ""),
        ("ssh", &[], ""),
        ("wget", &["-O"], ""),
        ("curl", &["-o"], ""),
        ("rsync", &[], ""),
        ("find", &[], ""),
        ("rm", &[], ""),
        ("tar", &[], ""),
        ("grep", &[], ""),
        ("sed", &[], ""),
        ("awk", &[], ""),
    ];

    let rule = extension_rules.iter().find(|(t, _, _)| *t == tool_lower);
    if let Some((_, output_flags, correct_ext)) = rule {
        if correct_ext.is_empty() {
            return args;
        }

        let tv = extract_task_values(task);
        let ref_output = tv.output_files.first().cloned();
        let ref_ext = ref_output.as_ref().map(|f| {
            let fl = f.to_ascii_lowercase();
            if fl.ends_with('/') { "/" }
            else if fl.ends_with(".fastq.gz") || fl.ends_with(".fq.gz") { ".fastq.gz" }
            else if fl.ends_with(".fasta.gz") || fl.ends_with(".fa.gz") { ".fasta.gz" }
            else if fl.ends_with(".fastq") || fl.ends_with(".fq") { ".fastq" }
            else if fl.ends_with(".fasta") || fl.ends_with(".fa") || fl.ends_with(".fna") { ".fasta" }
            else if fl.ends_with(".bam") { ".bam" }
            else if fl.ends_with(".sam") { ".sam" }
            else if fl.ends_with(".vcf") || fl.ends_with(".vcf.gz") { ".vcf" }
            else if fl.ends_with(".sig") { ".sig" }
            else if fl.ends_with(".csv") { ".csv" }
            else if fl.ends_with(".tsv") { ".tsv" }
            else if fl.ends_with(".txt") { ".txt" }
            else if fl.ends_with(".html") { ".html" }
            else { "" }
        }).unwrap_or("");

        let correct_ext_str = *correct_ext;
        let is_dir_rule = correct_ext_str == "/";

        for flag in *output_flags {
            let flag_str = flag.to_string();
            for i in 0..args.len() {
                if args[i] == flag_str && i + 1 < args.len() {
                    let out_lower = args[i + 1].to_ascii_lowercase();
                    let is_wrong = if is_dir_rule {
                        !out_lower.ends_with('/') && !out_lower.contains("output") && !out_lower.contains("result")
                    } else {
                        !out_lower.ends_with(correct_ext_str) && !out_lower.ends_with(ref_ext)
                    };
                    if is_wrong {
                        if let Some(ref_out) = ref_output.as_ref() {
                            args[i + 1] = ref_out.clone();
                        } else if is_dir_rule {
                            if !args[i + 1].ends_with('/') {
                                args[i + 1] = format!("{}/", args[i + 1]);
                            }
                        } else {
                            let stem = std::path::Path::new(&args[i + 1])
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "output".to_string());
                            args[i + 1] = format!("{}{}", stem, correct_ext_str);
                        }
                    }
                }
            }
        }
    } else {
        let tv = extract_task_values(task);
        if let Some(ref_output) = tv.output_files.first() {
            let ref_ext = get_file_extension(&ref_output.to_ascii_lowercase());
            if !ref_ext.is_empty() {
                let output_flags = ["-o", "--output", "-out", "--out", "--output-file", "-O", "--outfile"];
                for i in 0..args.len() {
                    if output_flags.iter().any(|f| *f == args[i]) && i + 1 < args.len() {
                        let out_lower = args[i + 1].to_ascii_lowercase();
                        let out_ext = get_file_extension(&out_lower);
                        if !out_ext.is_empty() && out_ext != ref_ext && !out_lower.ends_with('/') {
                            args[i + 1] = ref_output.clone();
                        }
                    }
                }
            }
        }
    }

    args
}

fn get_file_extension(filename: &str) -> String {
    if filename.ends_with(".fastq.gz") || filename.ends_with(".fq.gz") { return ".fastq.gz".to_string(); }
    if filename.ends_with(".fasta.gz") || filename.ends_with(".fa.gz") { return ".fasta.gz".to_string(); }
    if filename.ends_with(".vcf.gz") { return ".vcf.gz".to_string(); }
    if filename.ends_with(".bed.gz") { return ".bed.gz".to_string(); }
    if filename.ends_with(".gtf.gz") { return ".gtf.gz".to_string(); }
    if filename.ends_with(".gff.gz") { return ".gff.gz".to_string(); }
    if filename.ends_with('/') { return "/".to_string(); }
    if let Some(dot_pos) = filename.rfind('.') {
        return filename[dot_pos..].to_string();
    }
    String::new()
}

pub fn fix_generic_output_bam(args: &[String], tool: &str) -> Vec<String> {
    let tool_lower = tool.to_ascii_lowercase();
    let bam_output_tools: &[&str] = &[
        "samtools", "bamtools", "picard", "gatk", "sambamba", "bamutil",
        "bwa", "bwa-mem2", "bowtie2", "hisat2", "star", "minimap2",
        "bcftools", "freebayes", "varscan2", "longshot",
    ];
    if bam_output_tools.iter().any(|t| *t == tool_lower) {
        return args.to_vec();
    }
    let mut args = args.to_vec();
    let output_flags = ["-o", "--output", "-out", "--out", "--output-file", "-O"];
    for i in 0..args.len() {
        if output_flags.iter().any(|f| *f == args[i]) && i + 1 < args.len() {
            let val = args[i + 1].to_ascii_lowercase();
            if val.ends_with(".bam") {
                let stem = args[i + 1].trim_end_matches(".bam")
                    .trim_end_matches(".BAM")
                    .to_string();
                let new_ext = if tool_lower == "blast" || tool_lower == "blastn" || tool_lower == "blastp" || tool_lower == "blastx" {
                    ".txt"
                } else if tool_lower == "hmmer" || tool_lower == "hmmsearch" || tool_lower == "hmmscan" {
                    ".tbl"
                } else if tool_lower == "nanostat" || tool_lower == "nanoplot" {
                    ".txt"
                } else if tool_lower == "fasttree" || tool_lower == "iqtree2" || tool_lower == "raxml" {
                    ".treefile"
                } else if tool_lower == "mafft" || tool_lower == "muscle" || tool_lower == "clustalo" {
                    ".fasta"
                } else if tool_lower == "prodigal" {
                    ".faa"
                } else if tool_lower == "stringtie" {
                    ".gtf"
                } else if tool_lower == "sniffles" || tool_lower == "pbsv" || tool_lower == "delly" || tool_lower == "svim" {
                    ".vcf"
                } else if tool_lower == "kraken2" || tool_lower == "bracken" || tool_lower == "centrifuge" {
                    ".txt"
                } else if tool_lower == "sourmash" {
                    ".sig"
                } else if tool_lower == "racon" || tool_lower == "miniasm" || tool_lower == "flye" {
                    ".fasta"
                } else if tool_lower == "pilon" {
                    ".fasta"
                } else if tool_lower == "macs2" {
                    ""
                } else {
                    ".txt"
                };
                if new_ext.is_empty() {
                    args[i + 1] = stem;
                } else {
                    args[i + 1] = format!("{}{}", stem, new_ext);
                }
            }
        }
    }
    args
}

fn find_flag_insert_position(args: &[String]) -> usize {
    let mut pos = 0;
    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with('-') {
            pos = i + 1;
            if pos < args.len() && !args[pos].starts_with('-') {
                pos += 1;
            }
        } else {
            break;
        }
    }
    if pos == 0 && !args.is_empty() { args.len() } else { pos }
}

pub fn add_missing_required_flags(args: &[String], sdoc: &StructuredDoc, task: &str) -> Vec<String> {
    let args_str = args.join(" ");
    let args_lower = args_str.to_ascii_lowercase();
    let task_values = extract_task_values(task);
    let task_lower = task.to_ascii_lowercase();

    let mut additions: Vec<String> = Vec::new();
    let mut used_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    for arg in args {
        let al = arg.to_ascii_lowercase();
        if al.contains('.') && (al.contains('/') || al.ends_with(".bam") || al.ends_with(".sam")
            || al.ends_with(".fq") || al.ends_with(".fastq") || al.ends_with(".fa") || al.ends_with(".fasta")
            || al.ends_with(".vcf") || al.ends_with(".bed") || al.ends_with(".gtf") || al.ends_with(".gff")) {
            used_files.insert(al);
        }
    }

    let has_input_file_in_args = args.iter().any(|a| {
        let al = a.to_ascii_lowercase();
        al.ends_with(".bam") || al.ends_with(".sam") || al.ends_with(".fq") || al.ends_with(".fastq")
            || al.ends_with(".fa") || al.ends_with(".fasta") || al.ends_with(".fna")
            || al.ends_with(".vcf") || al.ends_with(".bed") || al.ends_with(".gtf") || al.ends_with(".gff")
            || al.ends_with(".gz") || al.ends_with(".fastq.gz") || al.ends_with(".fq.gz")
    });
    let has_output_file_in_args = args.iter().any(|a| {
        let al = a.to_ascii_lowercase();
        (al.contains("out") || al.contains("result") || al.contains("output"))
            && (al.contains('.') || al.contains('/'))
    });
    let task_has_input = !task_values.input_files.is_empty() || !task_values.read_files.is_empty();
    let task_has_output = !task_values.output_files.is_empty();
    let task_has_reference = !task_values.reference_files.is_empty();
    let task_has_genome_dir = !task_values.genome_dirs.is_empty();
    let task_mentions_threads = task_lower.contains("thread") || task_lower.contains("cpu")
        || task_lower.contains("core") || task_lower.contains("parallel")
        || task_lower.contains("process") || task_lower.contains("-p ") || task_lower.contains("-t ")
        || task_lower.contains("-@ ");

    let has_any_input_flag = args_lower.contains("input") || args_lower.contains("read")
        || args_lower.contains("fastq") || args_lower.contains("bam")
        || args_lower.contains("query") || args_lower.contains("-i ") || args_lower.contains("-f ")
        || args_lower.contains("-1 ") || args_lower.contains("-r ");
    let has_any_output_flag = args_lower.contains("output") || args_lower.contains("out")
        || args_lower.contains("-o ") || args_lower.contains("--out") || args_lower.contains("--o ");
    let has_any_thread_flag = args_lower.contains("thread") || args_lower.contains("cpu")
        || args_lower.contains("nproc") || args_lower.contains("-p ") || args_lower.contains("-t ")
        || args_lower.contains("-@ ") || args_lower.contains("--threads");
    let has_any_reference_flag = args_lower.contains("reference") || args_lower.contains("ref")
        || args_lower.contains("genome") || args_lower.contains("index")
        || args_lower.contains("-x ") || args_lower.contains("-r ");
    let has_any_db_flag = args_lower.contains("database") || args_lower.contains("db")
        || args_lower.contains("-d ");

    for entry in &sdoc.flag_catalog {
        if !entry.required {
            let desc_lower = entry.description.to_ascii_lowercase();
            let flag_lower = entry.flag.to_ascii_lowercase();

            let is_critical = (desc_lower.contains("runmode") || flag_lower.contains("runmode"))
                || (desc_lower.contains("genomedir") || flag_lower.contains("genomedir"))
                || (desc_lower.contains("genomefastafiles") || flag_lower.contains("genomefastafiles"))
                || (desc_lower.contains("readfilesin") || flag_lower.contains("readfilesin"))
                || (desc_lower.contains("readfilescommand") || flag_lower.contains("readfilescommand"))
                || (desc_lower.contains("outsamtype") || flag_lower.contains("outsamtype"))
                || (desc_lower.contains("outfilenamprefix") || flag_lower.contains("outfilenamprefix"));

            let is_semantic_critical = (task_has_input && !has_any_input_flag && !has_input_file_in_args
                && (desc_lower.contains("input") || desc_lower.contains("read file") || desc_lower.contains("fastq")
                    || desc_lower.contains("query") || desc_lower.contains("bam file")
                    || (flag_lower.contains("input") && !desc_lower.contains("format"))))
            || (task_has_output && !has_any_output_flag && !has_output_file_in_args
                && (desc_lower.contains("output file") || desc_lower.contains("output dir")
                    || desc_lower.contains("output prefix") || desc_lower.contains("output name")
                    || (flag_lower.contains("out") && !desc_lower.contains("format") && !desc_lower.contains("stdout"))))
            || (task_mentions_threads && !has_any_thread_flag
                && (desc_lower.contains("thread") || desc_lower.contains("number of cpu") || desc_lower.contains("nproc")
                    || desc_lower.contains("parallel") || desc_lower.contains("number of process")))
            || (task_has_reference && !has_any_reference_flag
                && (desc_lower.contains("reference") || desc_lower.contains("reference genome")
                    || desc_lower.contains("reference sequence") || desc_lower.contains("genome fasta")))
            || (task_has_genome_dir && !has_any_reference_flag
                && (desc_lower.contains("genome dir") || desc_lower.contains("genome directory")
                    || desc_lower.contains("genome index") || desc_lower.contains("index dir")))
            || (task_lower.contains("database") && !has_any_db_flag
                && (desc_lower.contains("database") || desc_lower.contains("db path") || desc_lower.contains("db file")));

            if !is_critical && !is_semantic_critical { continue; }
        }

        let flag_present = args_lower.contains(&entry.flag.to_ascii_lowercase());
        let alt_present = entry.alt_form.as_ref()
            .map(|a| args_lower.contains(&a.to_ascii_lowercase()))
            .unwrap_or(false);
        if flag_present || alt_present { continue; }

        let desc_lower = entry.description.to_ascii_lowercase();
        let flag_lower = entry.flag.to_ascii_lowercase();

        let value = resolve_flag_value(entry, &desc_lower, &flag_lower, &task_lower, &task_values, &mut used_files, sdoc);

        if entry.flag.contains('=') {
            if let Some(val) = value {
                additions.push(format!("{}{}", entry.flag, val));
            }
        } else {
            additions.push(entry.flag.clone());
            if let Some(val) = value {
                additions.push(val);
            }
        }
    }

    if !additions.is_empty() {
        let result = args.to_vec();
        let sub_end = if sdoc.has_subcommands && !result.is_empty() {
            if sdoc.subcommands.contains(&result[0]) { 1 } else { 0 }
        } else {
            0
        };
        let mut final_args = result[..sub_end].to_vec();
        final_args.extend(additions);
        final_args.extend(result[sub_end..].to_vec());
        final_args
    } else {
        args.to_vec()
    }
}

pub fn add_task_implied_flags(args: &[String], sdoc: &StructuredDoc, task: &str) -> Vec<String> {
    let task_values = extract_task_values(task);
    let args_str = args.join(" ");
    let args_lower = args_str.to_ascii_lowercase();
    let task_lower = task.to_ascii_lowercase();

    let mut additions: Vec<String> = Vec::new();
    let mut used_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    for arg in args {
        let al = arg.to_ascii_lowercase();
        if al.ends_with(".bam") || al.ends_with(".sam") || al.ends_with(".fq") || al.ends_with(".fastq")
            || al.ends_with(".fa") || al.ends_with(".fasta") || al.ends_with(".vcf") || al.ends_with(".bed")
            || al.ends_with(".gtf") || al.ends_with(".gff") || al.ends_with(".gz") || al.ends_with(".txt") {
            used_files.insert(arg.to_ascii_lowercase());
        }
    }

    if !args_lower.contains("runmode") && !args_lower.contains("--runmode") {
        let is_star_like = sdoc.flag_catalog.iter().any(|e| e.flag.to_ascii_lowercase().contains("runmode"))
            || sdoc.flag_catalog.iter().any(|e| e.flag.to_ascii_lowercase().contains("genomedir"))
            || sdoc.flag_catalog.iter().any(|e| e.flag.to_ascii_lowercase().contains("genomefastafiles"));
        if is_star_like {
            let run_mode = if task_lower.contains("genomegenerate") || task_lower.contains("generate genome") || task_lower.contains("genome index") {
                "genomeGenerate"
            } else {
                "alignReads"
            };
            additions.push("--runMode".to_string());
            additions.push(run_mode.to_string());
            additions.push("--genomeDir".to_string());
            let genome_dir = task_values.genome_dirs.first().cloned()
                .unwrap_or_else(|| "/path/to/star_index".to_string());
            additions.push(genome_dir);

            if run_mode == "genomeGenerate" {
                additions.push("--genomeFastaFiles".to_string());
                for f in &task_values.input_files {
                    let fl = f.to_ascii_lowercase();
                    if fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna") {
                        additions.push(f.clone());
                    }
                }
            } else {
                additions.push("--readFilesIn".to_string());
                let read_files: Vec<&String> = task_values.read_files.iter()
                    .filter(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .collect();
                if !read_files.is_empty() {
                    for f in &read_files {
                        additions.push((*f).clone());
                        used_files.insert(f.to_ascii_lowercase());
                    }
                } else {
                    for f in &task_values.input_files {
                        let fl = f.to_ascii_lowercase();
                        if (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".gz"))
                            && !used_files.contains(&fl) {
                            additions.push(f.clone());
                            used_files.insert(fl);
                        }
                    }
                }
                if task_values.input_files.iter().any(|f| f.to_ascii_lowercase().ends_with(".gz")) {
                    additions.push("--readFilesCommand".to_string());
                    additions.push("zcat".to_string());
                }
            }
        }
    }

    if !additions.is_empty() {
        let result = args.to_vec();
        let sub_end = if sdoc.has_subcommands && !result.is_empty() {
            if sdoc.subcommands.contains(&result[0]) { 1 } else { 0 }
        } else {
            0
        };
        let mut final_args = result[..sub_end].to_vec();
        final_args.extend(additions);
        final_args.extend(result[sub_end..].to_vec());
        return final_args;
    }

    args.to_vec()
}

fn resolve_flag_value(
    entry: &FlagEntry,
    desc_lower: &str,
    flag_lower: &str,
    task_lower: &str,
    task_values: &TaskValues,
    used_files: &mut std::collections::HashSet<String>,
    _sdoc: &StructuredDoc,
) -> Option<String> {
    if desc_lower.contains("output") || flag_lower.contains("out") {
        if desc_lower.contains("stdout") || desc_lower.contains("format") {
            return None;
        }
        if desc_lower.contains("dir") || desc_lower.contains("directory") || flag_lower.contains("outdir") || flag_lower.contains("output-dir") || flag_lower.contains("output_dir") {
            return task_values.output_files.first()
                .map(|f| {
                    let path = std::path::Path::new(f);
                    path.parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| ".".to_string())
                })
                .or_else(|| task_values.input_files.first()
                    .map(|f| {
                        let path = std::path::Path::new(f);
                        path.parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| ".".to_string())
                    }))
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("prefix") || flag_lower.contains("prefix") || flag_lower.contains("outfilenamprefix") {
            return task_values.output_files.first()
                .map(|f| {
                    let path = std::path::Path::new(f);
                    path.parent()
                        .map(|p| format!("{}/", p.to_string_lossy()))
                        .unwrap_or_else(|| "".to_string())
                })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("name") || flag_lower == "-n" {
            return task_values.output_files.first()
                .map(|f| {
                    std::path::Path::new(f)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| f.clone())
                })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("file") || desc_lower.contains("path") {
            return task_values.output_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone());
        }
        return task_values.output_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("input") || flag_lower.contains("in") || flag_lower.contains("bam") {
        if desc_lower.contains("bam") || desc_lower.contains("sam") {
            return task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".bam") || fl.ends_with(".sam") || fl.ends_with(".cram"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("fastq") || desc_lower.contains("fq") || desc_lower.contains("read") {
            return task_values.read_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".gz"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone()));
        }
        if desc_lower.contains("vcf") {
            return task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".vcf") || fl.ends_with(".bcf") || fl.ends_with(".vcf.gz"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("bed") {
            return task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".bed") || fl.ends_with(".bed.gz"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("gtf") || desc_lower.contains("gff") || desc_lower.contains("annotation") {
            return task_values.annotation_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".gff3"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone()));
        }
        return task_values.input_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("thread") || desc_lower.contains("cpu") || flag_lower == "-@" || flag_lower.contains("thread") {
        return task_values.numbers.iter()
            .find(|n| {
                let v: f64 = n.parse().unwrap_or(0.0);
                v >= 1.0 && v <= 128.0
            })
            .cloned()
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("reference") || flag_lower.contains("ref") || flag_lower.contains("genome") {
        if desc_lower.contains("dir") || desc_lower.contains("directory") || desc_lower.contains("path") || flag_lower.contains("genomedir") || flag_lower.contains("genome-dir") {
            return task_values.genome_dirs.first().cloned()
                .or_else(|| entry.default.clone());
        }
        return task_values.reference_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                        || fl.ends_with(".fa.gz") || fl.ends_with(".fasta.gz") || fl.ends_with(".fna.gz")
                        || fl.contains("genome") || fl.contains("reference") || fl.contains("ref."))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone()));
    }
    if desc_lower.contains("annotation") || desc_lower.contains("gtf") || desc_lower.contains("gff") {
        return task_values.annotation_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".gff3")
                        || fl.ends_with(".gtf.gz") || fl.ends_with(".gff.gz"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone()));
    }
    if desc_lower.contains("database") || desc_lower.contains("db") {
        return task_values.database_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("species") || flag_lower.contains("species") {
        return resolve_species(task_lower, &entry.default);
    }
    if desc_lower.contains("region") || flag_lower.contains("region") || flag_lower.contains("chrom") {
        return resolve_region(task_lower, &entry.default);
    }
    if desc_lower.contains("runmode") || flag_lower.contains("runmode") || flag_lower.contains("run-mode") {
        return if task_lower.contains("genomegenerate") || task_lower.contains("generate genome") || task_lower.contains("genome index") || task_lower.contains("create index") {
            Some("genomeGenerate".to_string())
        } else {
            Some("alignReads".to_string())
        };
    }
    if desc_lower.contains("genomedir") || flag_lower.contains("genomedir") || flag_lower.contains("genome-dir") {
        return task_values.genome_dirs.first().cloned()
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("genomefastafiles") || flag_lower.contains("genomefastafiles") || flag_lower.contains("genome-fasta") {
        return task_values.reference_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone()));
    }
    if desc_lower.contains("readfilesin") || flag_lower.contains("readfilesin") || flag_lower.contains("read-files") {
        return task_values.read_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".gz"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone()));
    }
    if desc_lower.contains("readfilescommand") || flag_lower.contains("readfilescommand") {
        return if task_values.input_files.iter().any(|f| f.to_ascii_lowercase().ends_with(".gz")) {
            Some("zcat".to_string())
        } else {
            None
        };
    }
    if desc_lower.contains("outsamtype") || flag_lower.contains("outsamtype") {
        return Some("BAM SortedByCoordinate".to_string());
    }
    if desc_lower.contains("outfilenamprefix") || flag_lower.contains("outfilenamprefix") {
        return task_values.output_files.first().cloned()
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("k") || flag_lower.contains("k=") || flag_lower == "-k" {
        return task_values.numbers.iter().find(|n| {
            let v: f64 = n.parse().unwrap_or(0.0);
            v >= 1.0 && v <= 100.0
        }).cloned().or_else(|| entry.default.clone());
    }
    if desc_lower.contains("index") || flag_lower == "-x" || flag_lower.contains("index-prefix")
        || flag_lower.contains("index_path") || flag_lower.contains("index-dir") {
        if desc_lower.contains("dir") || desc_lower.contains("path") || flag_lower.contains("dir") {
            return task_values.genome_dirs.first().cloned()
                .or_else(|| entry.default.clone());
        }
        return task_values.reference_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                        || fl.contains("index") || fl.contains("genome"))
                        && !used_files.contains(&fl)
                })
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| entry.default.clone()));
    }
    if desc_lower.contains("outfmt") || desc_lower.contains("output format") || flag_lower.contains("outfmt")
        || flag_lower == "-O" || flag_lower == "--format" {
        if !entry.enum_values.is_empty() {
            return Some(entry.enum_values[0].clone());
        }
        return entry.default.clone();
    }
    if desc_lower.contains("fasta") || desc_lower.contains("fna") {
        return task_values.input_files.iter()
            .find(|f| {
                let fl = f.to_ascii_lowercase();
                (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    || fl.ends_with(".fa.gz") || fl.ends_with(".fasta.gz"))
                    && !used_files.contains(&fl)
            })
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| entry.default.clone());
    }
    if desc_lower.contains("seed") || flag_lower.contains("seed") {
        return task_values.numbers.iter().find(|n| {
            let v: f64 = n.parse().unwrap_or(0.0);
            v >= 1.0 && v <= 999999.0
        }).cloned().or_else(|| entry.default.clone());
    }
    entry.default.clone()
}

fn resolve_species(task_lower: &str, default: &Option<String>) -> Option<String> {
    let species_map: &[(&str, &str)] = &[
        ("human", "human"), ("homo sapiens", "human"), ("hg38", "human"), ("hg19", "human"),
        ("mouse", "mouse"), ("mus musculus", "mouse"), ("mm10", "mouse"), ("mm9", "mouse"),
        ("arabidopsis", "arabidopsis"), ("thaliana", "arabidopsis"),
        ("fly", "fly"), ("drosophila", "fly"), ("dm6", "fly"), ("dm3", "fly"),
        ("yeast", "yeast"), ("saccharomyces", "yeast"),
        ("ecoli", "ecoli"), ("e. coli", "ecoli"), ("escherichia", "ecoli"),
        ("zebrafish", "zebrafish"), ("danio", "zebrafish"),
        ("rat", "rat"), ("rattus", "rat"),
        ("chicken", "chicken"), ("gallus", "chicken"),
        ("worm", "worm"), ("celegans", "worm"), ("c. elegans", "worm"),
        ("rice", "rice"), ("oryza", "rice"),
        ("maize", "maize"), ("zea", "maize"),
        ("soybean", "soybean"), ("glycine", "soybean"),
        ("tomato", "tomato"), ("solanum", "tomato"),
        ("pig", "pig"), ("sus scrofa", "pig"),
        ("cow", "cow"), ("bos taurus", "cow"), ("bovine", "cow"),
        ("dog", "dog"), ("canis", "dog"),
        ("cat", "cat"), ("felis", "cat"),
        ("frog", "frog"), ("xenopus", "frog"),
    ];
    for (pattern, value) in species_map {
        if task_lower.contains(pattern) {
            return Some(value.to_string());
        }
    }
    default.clone()
}

fn resolve_region(task_lower: &str, default: &Option<String>) -> Option<String> {
    for i in 1..=22 {
        if task_lower.contains(&format!("chr{}", i)) {
            return Some(format!("chr{}", i));
        }
    }
    if task_lower.contains("chrx") { return Some("chrX".to_string()); }
    if task_lower.contains("chry") { return Some("chrY".to_string()); }
    if task_lower.contains("chrm") || task_lower.contains("chrmt") { return Some("chrM".to_string()); }
    default.clone()
}

pub fn validate_flags_against_catalog(
    args: &[String],
    catalog: &[FlagEntry],
    quick_flags: &[String],
) -> Vec<String> {
    let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in catalog {
        for part in entry.flag.split([',', ' ', '\t']) {
            let part = part.trim();
            if part.starts_with('-') {
                known.insert(part.trim_end_matches('=').to_string());
            }
        }
        if let Some(ref alt) = entry.alt_form {
            for part in alt.split([',', ' ', '\t']) {
                let part = part.trim();
                if part.starts_with('-') {
                    known.insert(part.trim_end_matches('=').to_string());
                }
            }
        }
    }
    for qf in quick_flags {
        for part in qf.split([',', ' ', '\t']) {
            let part = part.trim();
            if part.starts_with('-') {
                known.insert(part.trim_end_matches('=').to_string());
            }
        }
    }

    if catalog.is_empty() {
        return args.to_vec();
    }

    for &universal in &[
        "-h", "--help", "-v", "--version",
        "-o", "--output", "-t", "--threads", "-@",
        "--outdir", "--prefix", "--out",
    ] {
        known.insert(universal.to_string());
    }

    let mut result: Vec<String> = Vec::new();
    let mut skip_next = false;
    let args_len = args.len();

    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if !arg.starts_with('-') {
            result.push(arg.clone());
            continue;
        }

        let flag_key = if arg.starts_with("--") {
            arg.split('=').next().unwrap_or(arg)
        } else if arg.len() > 2 && !arg.contains('=') {
            &arg[..2]
        } else {
            arg.as_str()
        };

        if known.contains(flag_key) {
            result.push(arg.clone());

            if !arg.contains('=') {
                let entry = catalog.iter().find(|e| {
                    e.flag.split([',', ' ', '\t'])
                        .any(|p| p.trim().trim_end_matches('=') == flag_key)
                        || e.alt_form.as_ref().map_or(false, |a| {
                            a.split([',', ' ', '\t'])
                                .any(|p| p.trim().trim_end_matches('=') == flag_key)
                        })
                });

                let takes_value = entry.map_or(true, |e| {
                    !e.flag.ends_with('=') && e.value_type.is_some()
                });

                let next_is_value = i + 1 < args_len
                    && !args[i + 1].starts_with('-');

                if takes_value && next_is_value {
                    result.push(args[i + 1].clone());
                    skip_next = true;
                }
            }
        } else {
            let mut corrected_flag: Option<String> = None;
            for entry in catalog {
                let entry_flags: Vec<&str> = entry.flag.split([',', ' ', '\t'])
                    .map(|p| p.trim().trim_end_matches('='))
                    .filter(|p| p.starts_with('-'))
                    .collect();
                for ef in &entry_flags {
                    if ef.starts_with(flag_key) && ef != &flag_key {
                        corrected_flag = Some(ef.to_string());
                        break;
                    }
                }
                if corrected_flag.is_some() { break; }
            }

            if let Some(ref cf) = corrected_flag {
                tracing::debug!("Correcting flag: {arg} -> {cf}");
                result.push(cf.clone());

                if !arg.contains('=') {
                    let next_is_value = i + 1 < args_len
                        && !args[i + 1].starts_with('-');
                    if next_is_value {
                        result.push(args[i + 1].clone());
                        skip_next = true;
                    }
                }
            } else {
                tracing::debug!("Removing unknown flag: {arg}");
                if !arg.contains('=') {
                    let next_is_value = i + 1 < args_len
                        && !args[i + 1].starts_with('-');
                    if next_is_value {
                        skip_next = true;
                    }
                }
            }
        }
    }

    result
}

pub fn limit_flag_count(
    args: &[String],
    sdoc: &StructuredDoc,
    task: &str,
) -> Vec<String> {
    let required_flags: std::collections::HashSet<String> = sdoc.flag_catalog.iter()
        .filter(|e| e.required)
        .flat_map(|e| {
            let mut flags = vec![e.flag.to_ascii_lowercase()];
            if let Some(ref alt) = e.alt_form {
                flags.push(alt.to_ascii_lowercase());
            }
            flags
        })
        .collect();

    let flag_count = args.iter().filter(|a| a.starts_with('-')).count();
    let required_count = sdoc.flag_catalog.iter().filter(|e| e.required).count();

    let max_optional = if required_count >= 8 { 2 } else if required_count >= 5 { 3 } else if required_count >= 3 { 4 } else { 4 };
    let max_total_flags = required_count + max_optional;

    if flag_count <= max_total_flags {
        return args.to_vec();
    }

    let task_lower = task.to_ascii_lowercase();
    let task_keywords: Vec<&str> = task_lower.split_whitespace()
        .filter(|w| w.len() >= 3 && !w.contains('.'))
        .collect();

    let score_flag_relevance = |flag: &str| -> i32 {
        let flag_lower = flag.to_ascii_lowercase();
        let mut score = 0;

        if required_flags.contains(&flag_lower) {
            score += 1000;
        }

        let entry = sdoc.flag_catalog.iter().find(|e| {
            e.flag.to_ascii_lowercase() == flag_lower
                || e.alt_form.as_ref().map_or(false, |a| a.to_ascii_lowercase() == flag_lower)
        });

        if let Some(e) = entry {
            if e.required { score += 500; }

            let desc_lower = e.description.to_ascii_lowercase();
            for kw in &task_keywords {
                if desc_lower.contains(kw) { score += 10; }
                if flag_lower.contains(kw) { score += 8; }
            }

            if desc_lower.contains("output") || flag_lower.contains("out") { score += 20; }
            if desc_lower.contains("input") || flag_lower.contains("in") { score += 15; }
            if desc_lower.contains("thread") || desc_lower.contains("cpu") { score += 10; }
            if desc_lower.contains("reference") || flag_lower.contains("ref") { score += 15; }
            if desc_lower.contains("genome") { score += 10; }
            if desc_lower.contains("annotation") || desc_lower.contains("gtf") { score += 10; }
            if desc_lower.contains("database") || flag_lower.contains("db") { score += 10; }

            if desc_lower.contains("verbose") || desc_lower.contains("debug") { score -= 30; }
            if desc_lower.contains("quiet") || desc_lower.contains("silent") { score -= 30; }
            if desc_lower.contains("help") || desc_lower.contains("version") { score -= 50; }
            if desc_lower.contains("log") && !task_lower.contains("log") { score -= 10; }
            if desc_lower.contains("color") || desc_lower.contains("colour") { score -= 20; }
            if desc_lower.contains("test") && !task_lower.contains("test") { score -= 20; }
        } else {
            score -= 50;
        }

        score
    };

    let mut flag_indices: Vec<(usize, i32)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with('-') {
            let score = score_flag_relevance(&args[i]);
            flag_indices.push((i, score));
            if !args[i].contains('=') && i + 1 < args.len() && !args[i + 1].starts_with('-') {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    flag_indices.sort_by(|a, b| b.1.cmp(&a.1));

    let mut keep_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut kept_count = 0;
    for (idx, score) in &flag_indices {
        if *score >= 500 || kept_count < max_total_flags {
            keep_indices.insert(*idx);
            let arg = &args[*idx];
            if !arg.contains('=') && *idx + 1 < args.len() && !args[*idx + 1].starts_with('-') {
                keep_indices.insert(*idx + 1);
            }
            kept_count += 1;
        }
    }

    let mut result = Vec::new();
    let sub_end = if sdoc.has_subcommands && !args.is_empty() {
        if sdoc.subcommands.contains(&args[0]) { 1 } else { 0 }
    } else {
        0
    };

    for (i, arg) in args.iter().enumerate() {
        if i < sub_end {
            result.push(arg.clone());
        } else if arg.starts_with('-') {
            if keep_indices.contains(&i) {
                result.push(arg.clone());
            }
        } else if keep_indices.contains(&i) {
            result.push(arg.clone());
        } else {
            let prev_is_kept_flag = i > 0 && args[i - 1].starts_with('-') && keep_indices.contains(&(i - 1));
            if prev_is_kept_flag {
                result.push(arg.clone());
            } else {
                result.push(arg.clone());
            }
        }
    }

    result
}
