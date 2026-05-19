use crate::doc_processor::{FlagEntry, StructuredDoc};
use super::task_values::{TaskValues, rule_based_subcommand_match, get_known_subcommands_for_tool};

pub fn assemble_command_from_rules(
    tool: &str,
    task: &str,
    sdoc: &StructuredDoc,
    selected_subcommand: Option<&str>,
    task_values: &TaskValues,
) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let task_lower = task.to_ascii_lowercase();
    let task_words: std::collections::HashSet<&str> = task_lower
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .collect();

    let effective_subcommand = selected_subcommand.map(|s| s.to_string())
        .or_else(|| {
            if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
                rule_based_subcommand_match(task, &sdoc.subcommands, &sdoc.subcommand_descriptions)
            } else {
                None
            }
        })
        .or_else(|| {
            let known_subs = get_known_subcommands_for_tool(tool);
            if known_subs.is_empty() {
                return None;
            }
            let task_lower = task.to_ascii_lowercase();
            for sub in &known_subs {
                let sub_lower = sub.to_ascii_lowercase();
                if sub.starts_with("--") {
                    let sub_name = sub.trim_start_matches("--");
                    if task_lower.contains(&sub_name.to_ascii_lowercase()) {
                        return Some(sub.clone());
                    }
                } else if task_lower.split_whitespace().any(|w| w == &sub_lower) {
                    return Some(sub.clone());
                }
            }
            for sub in &known_subs {
                let sub_lower = sub.to_ascii_lowercase();
                let sub_parts: Vec<&str> = sub_lower.split(|c: char| c == '_' || c == '-')
                    .filter(|p| p.len() >= 3)
                    .collect();
                for part in &sub_parts {
                    if task_lower.contains(part) {
                        return Some(sub.clone());
                    }
                }
            }
            None
        });

    if let Some(ref sub) = effective_subcommand {
        parts.push(sub.clone());
    }

    let mut used_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    let score_flag = |entry: &FlagEntry| -> i32 {
        let desc_lower = entry.description.to_ascii_lowercase();
        let flag_lower = entry.flag.to_ascii_lowercase();
        let mut score = 0;

        if entry.required {
            score += 100;
        }

        for word in &task_words {
            if desc_lower.contains(word) { score += 5; }
            if flag_lower.contains(word) { score += 3; }
        }

        if desc_lower.contains("output") && (task_lower.contains("output") || task_lower.contains("save") || task_lower.contains("write") || task_lower.contains("to ") || task_lower.contains("export") || task_lower.contains("generate") || task_lower.contains("produce") || task_lower.contains("create") || task_lower.contains("result") || task_lower.contains("store")) {
            score += 20;
        }
        if (desc_lower.contains("thread") || desc_lower.contains("cpu") || desc_lower.contains("proc")) && (task_lower.contains("thread") || task_lower.contains("cpu") || task_lower.contains("core") || task_lower.contains("parallel") || task_lower.contains("process")) {
            score += 20;
        }
        if desc_lower.contains("input") && (task_lower.contains("input") || task_lower.contains("read") || task_lower.contains("file") || task_lower.contains("bam") || task_lower.contains("fastq") || task_lower.contains("vcf") || task_lower.contains("from")) {
            score += 12;
        }
        if desc_lower.contains("reference") && (task_lower.contains("reference") || task_lower.contains("genome") || task_lower.contains("ref") || task_lower.contains("fasta") || task_lower.contains("index")) {
            score += 15;
        }
        if (desc_lower.contains("bam") || desc_lower.contains("sam")) && (task_lower.contains("bam") || task_lower.contains("sam")) {
            score += 10;
        }
        if (desc_lower.contains("fastq") || desc_lower.contains("fq")) && (task_lower.contains("fastq") || task_lower.contains("fq")) {
            score += 10;
        }
        if (desc_lower.contains("vcf") || desc_lower.contains("variant")) && (task_lower.contains("vcf") || task_lower.contains("variant") || task_lower.contains("snp")) {
            score += 10;
        }
        if (desc_lower.contains("gtf") || desc_lower.contains("gff") || desc_lower.contains("annotation")) && (task_lower.contains("gtf") || task_lower.contains("gff") || task_lower.contains("annotation")) {
            score += 10;
        }
        if desc_lower.contains("quality") && (task_lower.contains("quality") || task_lower.contains("qual")) {
            score += 8;
        }
        if desc_lower.contains("region") && (task_lower.contains("region") || task_lower.contains("chrom") || task_lower.contains("window")) {
            score += 10;
        }
        if desc_lower.contains("species") && task_lower.contains("species") {
            score += 10;
        }
        if desc_lower.contains("seed") && task_lower.contains("seed") {
            score += 15;
        }
        if (desc_lower.contains("bootstrap") || desc_lower.contains("replicat")) && (task_lower.contains("bootstrap") || task_lower.contains("replicat")) {
            score += 15;
        }
        if desc_lower.contains("convergence") && task_lower.contains("convergence") {
            score += 15;
        }
        if desc_lower.contains("supervised") && task_lower.contains("supervised") {
            score += 15;
        }
        if desc_lower.contains("projection") && (task_lower.contains("projection") || task_lower.contains("project")) {
            score += 15;
        }
        if desc_lower.contains("acceleration") && task_lower.contains("acceleration") {
            score += 15;
        }
        if desc_lower.contains("method") && task_lower.contains("method") {
            score += 10;
        }
        if desc_lower.contains("cross-validation") && (task_lower.contains("cross-validation") || task_lower.contains("cv")) {
            score += 15;
        }
        if desc_lower.contains("format") && task_lower.contains("format") {
            score += 8;
        }
        if desc_lower.contains("strand") && task_lower.contains("strand") {
            score += 10;
        }
        if desc_lower.contains("paired") && (task_lower.contains("paired") || task_lower.contains("pair")) {
            score += 10;
        }
        if desc_lower.contains("single") && (task_lower.contains("single-end") || task_lower.contains("single")) {
            score += 10;
        }
        if desc_lower.contains("min") && task_lower.contains("min") {
            score += 5;
        }
        if desc_lower.contains("max") && task_lower.contains("max") {
            score += 5;
        }
        if desc_lower.contains("cutoff") && (task_lower.contains("cutoff") || task_lower.contains("threshold")) {
            score += 10;
        }
        if desc_lower.contains("pvalue") || desc_lower.contains("p-value") || desc_lower.contains("pval") {
            if task_lower.contains("pvalue") || task_lower.contains("p-value") || task_lower.contains("pval") || task_lower.contains("significance") {
                score += 15;
            }
        }

        if let Some(ref sub) = effective_subcommand {
            let sub_lower = sub.to_lowercase();
            if desc_lower.contains(&sub_lower) { score += 5; }
        }

        if desc_lower.contains("help") || flag_lower.contains("version") {
            score -= 50;
        }
        if desc_lower.contains("verbose") || desc_lower.contains("debug") || desc_lower.contains("quiet") {
            score -= 20;
        }
        if desc_lower.contains("test") && !task_lower.contains("test") {
            score -= 10;
        }
        if desc_lower.contains("example") && !task_lower.contains("example") {
            score -= 10;
        }
        if desc_lower.contains("log") && !task_lower.contains("log") {
            score -= 5;
        }
        if desc_lower.contains("config") && !task_lower.contains("config") && !task_lower.contains("setting") {
            score -= 3;
        }
        if desc_lower.contains("tmp") || desc_lower.contains("temp") {
            score -= 5;
        }
        if (desc_lower.contains("color") || desc_lower.contains("colour")) && !task_lower.contains("color") {
            score -= 10;
        }
        if desc_lower.contains("silent") && !task_lower.contains("silent") {
            score -= 10;
        }

        if desc_lower.contains("output") && !desc_lower.contains("stdout") && !desc_lower.contains("format") {
            if task_lower.contains("output") || task_lower.contains("save") || task_lower.contains("write")
                || task_lower.contains("to ") || task_lower.contains("export") || task_lower.contains("generate")
                || task_lower.contains("produce") || task_lower.contains("create") || task_lower.contains("result")
                || task_lower.contains("convert") || task_lower.contains("call") || task_lower.contains("assemble")
                || task_lower.contains("align") || task_lower.contains("map") || task_lower.contains("index")
                || task_lower.contains("sort") || task_lower.contains("filter") || task_lower.contains("quantify")
                || task_lower.contains("annotate") || task_lower.contains("predict") {
                score += 8;
            }
        }

        if (desc_lower.contains("thread") || desc_lower.contains("cpu") || desc_lower.contains("proc") || desc_lower.contains("parallel"))
            && !task_lower.contains("single") {
            score += 3;
        }

        if (desc_lower.contains("database") || flag_lower.contains("db")) && (task_lower.contains("database") || task_lower.contains("db") || task_lower.contains("classify") || task_lower.contains("search") || task_lower.contains("blast")) {
            score += 5;
        }

        if (desc_lower.contains("index") || flag_lower == "-x") && (task_lower.contains("index") || task_lower.contains("align") || task_lower.contains("map") || task_lower.contains("reference")) {
            score += 12;
        }

        if (desc_lower.contains("annotation") || desc_lower.contains("gtf") || desc_lower.contains("gff")) && (task_lower.contains("annotation") || task_lower.contains("gtf") || task_lower.contains("gff") || task_lower.contains("transcript")) {
            score += 10;
        }

        if desc_lower.contains("preset") || desc_lower.contains("preset option") {
            score += 8;
        }

        if (desc_lower.contains("evalue") || desc_lower.contains("e-value") || desc_lower.contains("expect")) && (task_lower.contains("evalue") || task_lower.contains("e-value") || task_lower.contains("significance")) {
            score += 12;
        }

        if (desc_lower.contains("identity") || desc_lower.contains("similarity")) && (task_lower.contains("identity") || task_lower.contains("similarity") || task_lower.contains("percent")) {
            score += 10;
        }

        if desc_lower.contains("coverage") && (task_lower.contains("coverage") || task_lower.contains("depth") || task_lower.contains("cov")) {
            score += 10;
        }

        if (desc_lower.contains("length") || desc_lower.contains("len")) && (task_lower.contains("length") || task_lower.contains("len") || task_lower.contains("size")) {
            score += 5;
        }

        if desc_lower.contains("memory") || desc_lower.contains("mem") {
            score -= 3;
        }

        if desc_lower.contains("intermediate") || desc_lower.contains("temp") || desc_lower.contains("tmp") {
            score -= 8;
        }

        score
    };

    let mut all_flags: Vec<&FlagEntry> = sdoc.flag_catalog.iter().collect();
    all_flags.sort_by(|a, b| score_flag(b).cmp(&score_flag(a)));

    let mut included_flags: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in &all_flags {
        let flag_key = entry.flag.clone();
        if included_flags.contains(&flag_key) { continue; }
        if let Some(ref alt) = entry.alt_form {
            if included_flags.contains(alt) { continue; }
        }

        let score = score_flag(entry);
        let should_include = entry.required || score >= 1;

        if !should_include { continue; }

        let desc_lower = entry.description.to_ascii_lowercase();
        let flag_lower = entry.flag.to_ascii_lowercase();

        let value = if desc_lower.contains("output") || flag_lower.contains("out") {
            if desc_lower.contains("stdout") || desc_lower.contains("format") {
                None
            } else if desc_lower.contains("prefix") {
                task_values.output_files.first()
                    .map(|f| {
                        let path = std::path::Path::new(f);
                        path.with_extension("").to_string_lossy().to_string()
                    })
                    .or_else(|| entry.default.clone())
            } else if desc_lower.contains("dir") || desc_lower.contains("directory") || desc_lower.contains("path") {
                task_values.output_files.first()
                    .map(|f| {
                        let path = std::path::Path::new(f);
                        path.parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| ".".to_string())
                    })
                    .or_else(|| entry.default.clone())
            } else {
                task_values.output_files.iter()
                    .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone())
            }
        } else if desc_lower.contains("input") || flag_lower.contains("in") {
            if desc_lower.contains("bam") || desc_lower.contains("sam") {
                task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".bam") || fl.ends_with(".sam"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone())
            } else if desc_lower.contains("fastq") || desc_lower.contains("fq") || desc_lower.contains("read") {
                task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".gz"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone())
            } else if desc_lower.contains("vcf") {
                task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".vcf") || fl.ends_with(".bcf"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone())
            } else {
                task_values.input_files.iter()
                    .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| entry.default.clone())
            }
        } else if desc_lower.contains("thread") || desc_lower.contains("cpu") || flag_lower == "-@" || flag_lower.contains("thread") {
            task_values.numbers.iter()
                .find(|n| {
                    let v: f64 = n.parse().unwrap_or(0.0);
                    v >= 1.0 && v <= 128.0
                })
                .cloned()
                .or_else(|| entry.default.clone())
        } else if desc_lower.contains("reference") || flag_lower.contains("ref") || flag_lower.contains("genome") {
            if desc_lower.contains("dir") || desc_lower.contains("directory") || desc_lower.contains("path") {
                task_values.genome_dirs.first().cloned()
                    .or_else(|| entry.default.clone())
            } else {
                task_values.reference_files.iter()
                    .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .map(|f| {
                        used_files.insert(f.to_ascii_lowercase());
                        f.clone()
                    })
                    .or_else(|| task_values.input_files.iter()
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
                        .or_else(|| entry.default.clone()))
            }
        } else if desc_lower.contains("species") {
            if task_lower.contains("human") {
                Some("human".to_string())
            } else if task_lower.contains("mouse") {
                Some("mouse".to_string())
            } else if task_lower.contains("arabidopsis") {
                Some("arabidopsis".to_string())
            } else if task_lower.contains("fly") {
                Some("fly".to_string())
            } else if task_lower.contains("yeast") {
                Some("yeast".to_string())
            } else if task_lower.contains("ecoli") || task_lower.contains("e. coli") {
                Some("ecoli".to_string())
            } else if task_lower.contains("zebrafish") {
                Some("zebrafish".to_string())
            } else {
                entry.default.clone()
            }
        } else if desc_lower.contains("seed") {
            task_values.numbers.iter().find(|n| {
                let v: f64 = n.parse().unwrap_or(0.0);
                v >= 1.0 && v <= 999999.0
            }).cloned().or_else(|| entry.default.clone())
        } else if desc_lower.contains("k") || flag_lower.contains("k=") || flag_lower == "-k" {
            task_values.numbers.iter().find(|n| {
                let v: f64 = n.parse().unwrap_or(0.0);
                v >= 1.0 && v <= 100.0
            }).cloned().or_else(|| entry.default.clone())
        } else if desc_lower.contains("region") || flag_lower.contains("region") || flag_lower.contains("chrom") {
            if task_lower.contains("chr1") {
                Some("chr1".to_string())
            } else if task_lower.contains("chr2") {
                Some("chr2".to_string())
            } else if task_lower.contains("chr22") {
                Some("chr22".to_string())
            } else {
                entry.default.clone()
            }
        } else if desc_lower.contains("runmode") || flag_lower.contains("runmode") {
            if task_lower.contains("genomegenerate") || task_lower.contains("generate genome") || task_lower.contains("genome index") {
                Some("genomeGenerate".to_string())
            } else {
                Some("alignReads".to_string())
            }
        } else if desc_lower.contains("readfilescommand") || flag_lower.contains("readfilescommand") {
            if task_values.input_files.iter().any(|f| f.to_ascii_lowercase().ends_with(".gz")) {
                Some("zcat".to_string())
            } else {
                None
            }
        } else if desc_lower.contains("outsamtype") || flag_lower.contains("outsamtype") {
            Some("BAM SortedByCoordinate".to_string())
        } else if desc_lower.contains("outfilenamprefix") || flag_lower.contains("outfilenamprefix") {
            task_values.output_files.first()
                .map(|f| {
                    let path = std::path::Path::new(f);
                    path.parent()
                        .map(|p| format!("{}/", p.to_string_lossy()))
                        .unwrap_or_else(|| "".to_string())
                })
                .or_else(|| entry.default.clone())
        } else if desc_lower.contains("index") || flag_lower == "-x" || flag_lower.contains("index-prefix") {
            if desc_lower.contains("dir") || desc_lower.contains("path") {
                task_values.genome_dirs.first().cloned()
                    .or_else(|| entry.default.clone())
            } else {
                task_values.reference_files.iter()
                    .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                    .or_else(|| task_values.input_files.iter()
                        .find(|f| {
                            let fl = f.to_ascii_lowercase();
                            (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                                || fl.contains("index") || fl.contains("genome"))
                                && !used_files.contains(&fl)
                        })
                        .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                        .or_else(|| entry.default.clone()))
            }
        } else if desc_lower.contains("database") || flag_lower.contains("db") {
            task_values.database_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone())
        } else if desc_lower.contains("annotation") || desc_lower.contains("gtf") || desc_lower.contains("gff") {
            task_values.annotation_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".gff3"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                    .or_else(|| entry.default.clone()))
        } else if desc_lower.contains("bed") {
            task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".bed") || fl.ends_with(".bed.gz")) && !used_files.contains(&fl)
                })
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone())
        } else if desc_lower.contains("fasta") || desc_lower.contains("fna") {
            task_values.input_files.iter()
                .find(|f| {
                    let fl = f.to_ascii_lowercase();
                    (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                        || fl.ends_with(".fa.gz") || fl.ends_with(".fasta.gz"))
                        && !used_files.contains(&fl)
                })
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone())
        } else if desc_lower.contains("outfmt") || desc_lower.contains("output format") || flag_lower.contains("outfmt") {
            if !entry.enum_values.is_empty() {
                Some(entry.enum_values[0].clone())
            } else {
                entry.default.clone()
            }
        } else {
            entry.default.clone()
        };

        if entry.flag.contains('=') {
            if let Some(val) = value {
                parts.push(format!("{}{}", entry.flag, val));
            } else {
                parts.push(entry.flag.clone());
            }
        } else {
            parts.push(entry.flag.clone());
            if let Some(val) = value {
                parts.push(val);
            }
        }

        included_flags.insert(flag_key);
        if let Some(ref alt) = entry.alt_form {
            included_flags.insert(alt.clone());
        }
    }

    let positional_remaining: Vec<&String> = task_values.input_files.iter()
        .filter(|f| !used_files.contains(&f.to_ascii_lowercase()))
        .collect();

    if !sdoc.usage_pattern.positional_args.is_empty() || !positional_remaining.is_empty() {
        for f in &positional_remaining {
            parts.push((*f).clone());
        }
    }

    if !task_values.numbers.is_empty() && !sdoc.has_subcommands {
        let args_str = parts.join(" ");
        let has_number_arg = task_values.numbers.iter().any(|n| args_str.contains(n.as_str()));
        if !has_number_arg {
            for n in &task_values.numbers {
                let v: f64 = n.parse().unwrap_or(0.0);
                if v < 1.0 || v > 128.0 {
                    parts.push(n.clone());
                }
            }
        }
    }

    parts
}
