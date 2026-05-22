use crate::doc_processor::{FlagEntry, StructuredDoc};
use super::task_values::{TaskValues, rule_based_subcommand_match, get_known_subcommands_for_tool};

const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "from", "into",
    "when", "where", "which", "what", "how", "can", "will", "shall",
    "may", "must", "should", "could", "would", "also", "than",
    "then", "been", "being", "have", "has", "had", "does", "did",
    "not", "but", "are", "was", "were", "its", "our", "your",
    "all", "any", "each", "every", "both", "few", "more", "most",
    "other", "some", "such", "only", "own", "same", "very", "just",
    "use", "used", "using", "set", "specify", "specifies", "specified",
    "option", "optional", "default", "given", "provide", "provided",
    "whether", "either", "neither", "number", "name", "value",
    "file", "path", "list", "type", "mode", "format",
];

fn compute_word_overlap_score(task_words: &std::collections::HashSet<&str>, desc_lower: &str) -> i32 {
    let desc_word_set: std::collections::HashSet<&str> = desc_lower
        .split(|c: char| c.is_whitespace() || c == '_' || c == '-' || c == '/' || c == ',')
        .filter(|w| w.len() > 2 && !STOP_WORDS.contains(w))
        .collect();

    let mut score = 0i32;
    for word in task_words {
        if STOP_WORDS.contains(word) { continue; }
        if desc_word_set.contains(word) {
            score += if word.len() >= 7 { 10 } else if word.len() >= 5 { 7 } else if word.len() >= 4 { 5 } else { 3 };
        }
    }
    score
}

fn compute_semantic_bonus(desc_lower: &str, flag_lower: &str, task_lower: &str, task_values: &TaskValues) -> i32 {
    let mut bonus = 0i32;

    let task_has_output = !task_values.output_files.is_empty()
        || task_lower.contains("output") || task_lower.contains("save") || task_lower.contains("write")
        || task_lower.contains(" to ") || task_lower.contains("export") || task_lower.contains("generate")
        || task_lower.contains("produce") || task_lower.contains("create") || task_lower.contains("result")
        || task_lower.contains("convert") || task_lower.contains("call") || task_lower.contains("assemble")
        || task_lower.contains("align") || task_lower.contains("map") || task_lower.contains("index")
        || task_lower.contains("sort") || task_lower.contains("filter") || task_lower.contains("quantify")
        || task_lower.contains("annotate") || task_lower.contains("predict") || task_lower.contains("store");

    if desc_lower.contains("output") && !desc_lower.contains("stdout") && !desc_lower.contains("format") && task_has_output {
        bonus += 20;
    }

    if (desc_lower.contains("thread") || desc_lower.contains("cpu") || desc_lower.contains("proc") || desc_lower.contains("parallel"))
        && !task_lower.contains("single") {
        let task_mentions_threads = task_lower.contains("thread") || task_lower.contains("cpu")
            || task_lower.contains("core") || task_lower.contains("parallel") || task_lower.contains("process");
        if task_mentions_threads {
            bonus += 20;
        } else {
            bonus += 3;
        }
    }

    if desc_lower.contains("input") && (task_lower.contains("input") || task_lower.contains("read")
        || task_lower.contains("file") || task_lower.contains("bam") || task_lower.contains("fastq")
        || task_lower.contains("vcf") || task_lower.contains("from")) {
        bonus += 12;
    }

    if (desc_lower.contains("reference") || desc_lower.contains("ref genome") || desc_lower.contains("genome fasta"))
        && (task_lower.contains("reference") || task_lower.contains("genome") || task_lower.contains("ref")
            || task_lower.contains("fasta") || task_lower.contains("index")) {
        bonus += 15;
    }

    if (desc_lower.contains("index") || flag_lower == "-x" || flag_lower.contains("index-prefix"))
        && (task_lower.contains("index") || task_lower.contains("align") || task_lower.contains("map") || task_lower.contains("reference")) {
        bonus += 12;
    }

    if (desc_lower.contains("database") || flag_lower.contains("db"))
        && (task_lower.contains("database") || task_lower.contains("db") || task_lower.contains("classify")
            || task_lower.contains("search") || task_lower.contains("blast")) {
        bonus += 8;
    }

    if (desc_lower.contains("annotation") || desc_lower.contains("gtf") || desc_lower.contains("gff"))
        && (task_lower.contains("annotation") || task_lower.contains("gtf") || task_lower.contains("gff")
            || task_lower.contains("transcript")) {
        bonus += 10;
    }

    let file_type_pairs: &[(&[&str], &[&str])] = &[
        (&["bam", "sam"], &["bam", "sam"]),
        (&["fastq", "fq", "read"], &["fastq", "fq"]),
        (&["vcf", "variant", "snp"], &["vcf", "variant", "snp"]),
        (&["bed", "interval"], &["bed"]),
        (&["fasta", "fna", "sequence"], &["fa", "fasta", "fna"]),
    ];
    for (desc_keywords, task_keywords) in file_type_pairs {
        let desc_matches = desc_keywords.iter().any(|k| desc_lower.contains(k));
        let task_matches = task_keywords.iter().any(|k| task_lower.contains(k));
        if desc_matches && task_matches {
            bonus += 10;
        }
    }

    let semantic_pairs: &[(&[&str], &[&str], i32)] = &[
        (&["quality", "qual"], &["quality", "qual", "trim", "filter"], 8),
        (&["region", "chrom", "window"], &["region", "chrom", "window", "interval", "locus"], 10),
        (&["species"], &["species", "organism", "taxon"], 10),
        (&["seed"], &["seed", "random"], 15),
        (&["bootstrap", "replicat"], &["bootstrap", "replicat", "resample"], 15),
        (&["evalue", "e-value", "expect"], &["evalue", "e-value", "significance"], 12),
        (&["identity", "similarity"], &["identity", "similarity", "percent", "match"], 10),
        (&["coverage", "depth"], &["coverage", "depth", "cov"], 10),
        (&["preset"], &["preset", "sensitivity", "sensitive", "fast"], 8),
        (&["method"], &["method", "algorithm", "approach", "mode", "strategy"], 8),
        (&["strand"], &["strand", "forward", "reverse", "sense"], 10),
        (&["paired", "pair"], &["paired", "pair", "mate", "dual"], 10),
        (&["cutoff", "threshold"], &["cutoff", "threshold", "minimum", "min", "maximum", "max"], 10),
        (&["pvalue", "p-value", "pval"], &["pvalue", "p-value", "pval", "significance"], 15),
        (&["format"], &["format", "output format", "type"], 8),
        (&["length", "len"], &["length", "len", "size", "bp", "base"], 5),
        (&["adapter", "barcode", "index"], &["adapter", "barcode", "index", "ligation"], 12),
        (&["contig", "scaffold"], &["contig", "scaffold", "assembly", "assemble"], 10),
        (&["variant", "snp", "genotype"], &["variant", "snp", "genotype", "call", "mutation"], 10),
        (&["expression", "abundance", "count"], &["expression", "abundance", "count", "quantify", "tpm", "fpkm"], 10),
        (&["annotation", "feature", "gene"], &["annotation", "feature", "gene", "transcript", "exon", "cds"], 10),
        (&["taxonomic", "classify", "taxon"], &["taxonomic", "classify", "taxon", "taxonomy", "identify"], 10),
        (&["read", "read1", "read2"], &["read", "read1", "read2", "mate", "pair-end", "single-end"], 8),
        (&["bam", "sam", "alignment"], &["bam", "sam", "alignment", "mapped", "align"], 8),
        (&["vcf", "variant", "genotype"], &["vcf", "variant", "genotype", "snp", "indel"], 8),
        (&["fastq", "fq", "sequence"], &["fastq", "fq", "sequence", "read", "raw"], 8),
        (&["bed", "interval", "peak"], &["bed", "interval", "peak", "region", "chip"], 8),
        (&["fasta", "reference", "genome"], &["fasta", "reference", "genome", "assembly", "fna"], 8),
        (&["gtf", "gff", "annotation"], &["gtf", "gff", "annotation", "transcript", "gene model"], 8),
        (&["mem", "memory", "buffer"], &["mem", "memory", "buffer", "ram"], 3),
        (&["compress", "decompress", "gzip", "zip"], &["compress", "decompress", "gzip", "zip", "gz", "archive"], 8),
        (&["score", "bit", "bitscore"], &["score", "bit", "bitscore", "ranking"], 8),
        (&["gap", "mismatch", "penalty"], &["gap", "mismatch", "penalty", "open", "extend"], 8),
        (&["motif", "pattern", "domain"], &["motif", "pattern", "domain", "consensus", "pfam"], 10),
        (&["tree", "phylogeny", "newick"], &["tree", "phylogeny", "newick", "phylogen", "branch"], 10),
        (&["busco", "completeness", "lineage"], &["busco", "completeness", "lineage", "quality", "assessment"], 10),
        (&["trim", "clip", "cut"], &["trim", "clip", "cut", "remove", "discard"], 10),
        (&["merge", "combine", "join", "concat"], &["merge", "combine", "join", "concat", "union"], 8),
        (&["split", "partition", "chunk"], &["split", "partition", "chunk", "divide"], 8),
        (&["sort", "order", "arrange"], &["sort", "order", "arrange", "rank"], 8),
        (&["dedup", "duplicate", "markdup"], &["dedup", "duplicate", "markdup", "remove duplicate"], 10),
        (&["realignment", "recalibration", "base recal"], &["realignment", "recalibration", "base recal", "bqsr"], 10),
        (&["contamination", "cross"], &["contamination", "cross", "pollut"], 10),
    ];
    for (desc_keywords, task_keywords, score) in semantic_pairs {
        let desc_matches = desc_keywords.iter().any(|k| desc_lower.contains(k));
        let task_matches = task_keywords.iter().any(|k| task_lower.contains(k));
        if desc_matches && task_matches {
            bonus += score;
        }
    }

    bonus
}

fn compute_negative_score(desc_lower: &str, flag_lower: &str, task_lower: &str) -> i32 {
    let mut neg = 0i32;

    if desc_lower.contains("help") || flag_lower.contains("version") { neg -= 50; }
    if desc_lower.contains("verbose") || desc_lower.contains("quiet") { neg -= 20; }
    if desc_lower.contains("debug") && !task_lower.contains("debug") { neg -= 20; }
    if desc_lower.contains("test") && !task_lower.contains("test") { neg -= 10; }
    if desc_lower.contains("example") && !task_lower.contains("example") { neg -= 10; }
    if desc_lower.contains("log") && !task_lower.contains("log") { neg -= 5; }
    if desc_lower.contains("config") && !task_lower.contains("config") && !task_lower.contains("setting") { neg -= 3; }
    if desc_lower.contains("tmp") || desc_lower.contains("temp") { neg -= 5; }
    if (desc_lower.contains("color") || desc_lower.contains("colour")) && !task_lower.contains("color") { neg -= 10; }
    if desc_lower.contains("silent") && !task_lower.contains("silent") { neg -= 10; }
    if desc_lower.contains("memory") || desc_lower.contains("mem") { neg -= 3; }
    if desc_lower.contains("intermediate") { neg -= 8; }
    if desc_lower.contains("progress") && !task_lower.contains("progress") { neg -= 5; }
    if desc_lower.contains("statistics") && !task_lower.contains("stat") && !task_lower.contains("statistics") { neg -= 3; }
    if (desc_lower.contains("report") || desc_lower.contains("summary")) && !task_lower.contains("report") && !task_lower.contains("summary") { neg -= 5; }
    if desc_lower.contains("dry-run") || desc_lower.contains("dry_run") || desc_lower.contains("pretend") { neg -= 15; }
    if (desc_lower.contains("force") || desc_lower.contains("overwrite")) && !task_lower.contains("force") && !task_lower.contains("overwrite") { neg -= 3; }
    if desc_lower.contains("trace") && !task_lower.contains("trace") { neg -= 10; }
    if (desc_lower.contains("benchmark") || desc_lower.contains("timing")) && !task_lower.contains("benchmark") { neg -= 8; }
    if desc_lower.contains("citation") || desc_lower.contains("cite") { neg -= 10; }
    if desc_lower.contains("warranty") || desc_lower.contains("license") { neg -= 15; }

    neg
}

fn find_matching_file<'a>(
    files: &'a [String],
    used_files: &std::collections::HashSet<String>,
    extensions: &[&str],
) -> Option<&'a String> {
    files.iter().find(|f| {
        let fl = f.to_ascii_lowercase();
        extensions.iter().any(|ext| fl.ends_with(ext)) && !used_files.contains(&fl)
    })
}

fn find_any_unused_file<'a>(
    files: &'a [String],
    used_files: &std::collections::HashSet<String>,
) -> Option<&'a String> {
    files.iter().find(|f| !used_files.contains(&f.to_ascii_lowercase()))
}

fn infer_output_from_input_for_rule(
    desc_lower: &str,
    flag_lower: &str,
    input_files: &[String],
    used_files: &std::collections::HashSet<String>,
) -> Option<String> {
    let first_unused = input_files.iter()
        .find(|f| !used_files.contains(&f.to_ascii_lowercase()))?;

    let path = std::path::Path::new(first_unused);
    let stem = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let stem = stem.trim_end_matches(".fastq.gz").trim_end_matches(".fq.gz")
        .trim_end_matches(".fasta.gz").trim_end_matches(".fa.gz")
        .trim_end_matches(".vcf.gz").trim_end_matches(".bed.gz")
        .trim_end_matches(".gtf.gz").trim_end_matches(".gff.gz")
        .trim_end_matches(".sam.gz").trim_end_matches(".bam.gz")
        .trim_end_matches(".fastq").trim_end_matches(".fq")
        .trim_end_matches(".fa").trim_end_matches(".fasta")
        .trim_end_matches(".bam").trim_end_matches(".sam")
        .trim_end_matches(".vcf").trim_end_matches(".gz")
        .trim_end_matches(".bed").trim_end_matches(".txt")
        .trim_end_matches(".gff").trim_end_matches(".gtf")
        .trim_end_matches(".cram").trim_end_matches(".bai")
        .trim_end_matches(".csi").trim_end_matches(".tbi");
    let parent = path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let stem_with_dir = if parent != "." {
        format!("{}/{}", parent, stem)
    } else {
        stem.to_string()
    };

    if desc_lower.contains("dir") || desc_lower.contains("directory") || flag_lower.contains("outdir") || flag_lower.contains("out-dir") {
        return Some(format!("{}/", stem_with_dir));
    }
    if desc_lower.contains("prefix") || flag_lower.contains("prefix") {
        return Some(stem_with_dir);
    }
    if desc_lower.contains(".bam") || flag_lower.contains("bam") {
        return Some(format!("{}.bam", stem_with_dir));
    }
    if desc_lower.contains(".vcf") || flag_lower.contains("vcf") || flag_lower.contains("variant") {
        return Some(format!("{}.vcf", stem_with_dir));
    }
    if desc_lower.contains(".sam") || flag_lower.contains("sam") {
        return Some(format!("{}.sam", stem_with_dir));
    }
    if desc_lower.contains(".fasta") || desc_lower.contains(".fa") || flag_lower.contains("fasta") {
        return Some(format!("{}.fasta", stem_with_dir));
    }
    if desc_lower.contains(".fastq") || desc_lower.contains(".fq") || flag_lower.contains("fastq") {
        return Some(format!("{}.fastq", stem_with_dir));
    }
    if desc_lower.contains(".txt") || flag_lower.contains("txt") {
        return Some(format!("{}.txt", stem_with_dir));
    }
    if desc_lower.contains(".html") || flag_lower.contains("html") {
        return Some(format!("{}.html", stem_with_dir));
    }
    if desc_lower.contains(".json") || flag_lower.contains("json") {
        return Some(format!("{}.json", stem_with_dir));
    }
    if desc_lower.contains(".tsv") || flag_lower.contains("tsv") {
        return Some(format!("{}.tsv", stem_with_dir));
    }
    if desc_lower.contains(".csv") || flag_lower.contains("csv") {
        return Some(format!("{}.csv", stem_with_dir));
    }
    if desc_lower.contains(".gff") || flag_lower.contains("gff") {
        return Some(format!("{}.gff", stem_with_dir));
    }
    if desc_lower.contains(".gtf") || flag_lower.contains("gtf") {
        return Some(format!("{}.gtf", stem_with_dir));
    }
    if desc_lower.contains(".bed") || flag_lower.contains("bed") {
        return Some(format!("{}.bed", stem_with_dir));
    }
    if desc_lower.contains("file") || desc_lower.contains("name") || desc_lower.contains("path") {
        return Some(format!("{}.out", stem_with_dir));
    }
    Some(format!("{}/", stem_with_dir))
}

fn resolve_flag_value(
    entry: &FlagEntry,
    desc_lower: &str,
    flag_lower: &str,
    task_lower: &str,
    task_values: &TaskValues,
    used_files: &mut std::collections::HashSet<String>,
) -> Option<String> {
    if desc_lower.contains("output") || flag_lower.contains("out") {
        if desc_lower.contains("stdout") || desc_lower.contains("format") {
            return None;
        }
        if desc_lower.contains("prefix") || flag_lower.contains("prefix") || flag_lower.contains("-n ") || flag_lower == "-n" {
            return task_values.output_files.first()
                .map(|f| {
                    let path = std::path::Path::new(f);
                    let stem = path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "output".to_string());
                    let parent = path.parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| ".".to_string());
                    if parent == "." {
                        stem
                    } else {
                        format!("{}/{}", parent, stem)
                    }
                })
                .or_else(|| infer_output_from_input_for_rule(desc_lower, flag_lower, &task_values.input_files, used_files))
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("dir") || desc_lower.contains("directory") || desc_lower.contains("path") {
            return task_values.output_files.first()
                .map(|f| {
                    let path = std::path::Path::new(f);
                    path.parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| ".".to_string())
                })
                .or_else(|| infer_output_from_input_for_rule(desc_lower, flag_lower, &task_values.input_files, used_files))
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("file") || desc_lower.contains("name") {
            return task_values.output_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| {
                    used_files.insert(f.to_ascii_lowercase());
                    f.clone()
                })
                .or_else(|| infer_output_from_input_for_rule(desc_lower, flag_lower, &task_values.input_files, used_files))
                .or_else(|| entry.default.clone());
        }
        return task_values.output_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| {
                used_files.insert(f.to_ascii_lowercase());
                f.clone()
            })
            .or_else(|| infer_output_from_input_for_rule(desc_lower, flag_lower, &task_values.input_files, used_files))
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("input") || flag_lower.contains("in") {
        if desc_lower.contains("bam") || desc_lower.contains("sam") {
            return find_matching_file(&task_values.input_files, used_files, &[".bam", ".sam"])
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("fastq") || desc_lower.contains("fq") || desc_lower.contains("read") {
            if !task_values.read_files.is_empty() {
                return task_values.read_files.iter()
                    .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                    .or_else(|| entry.default.clone());
            }
            return find_matching_file(&task_values.input_files, used_files, &[".fq", ".fastq", ".fq.gz", ".fastq.gz"])
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("vcf") {
            return find_matching_file(&task_values.input_files, used_files, &[".vcf", ".bcf", ".vcf.gz"])
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("fasta") || desc_lower.contains("fna") || desc_lower.contains("genome") {
            return task_values.reference_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    find_matching_file(&task_values.input_files, used_files, &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"])
                        .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("bed") {
            return find_matching_file(&task_values.input_files, used_files, &[".bed", ".bed.gz"])
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone());
        }
        if desc_lower.contains("gtf") || desc_lower.contains("gff") {
            return task_values.annotation_files.iter()
                .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| {
                    find_matching_file(&task_values.input_files, used_files, &[".gtf", ".gff", ".gff3"])
                        .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                })
                .or_else(|| entry.default.clone());
        }
        return find_any_unused_file(&task_values.input_files, used_files)
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("query") || flag_lower.contains("query") {
        return find_matching_file(&task_values.input_files, used_files, &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"])
            .or_else(|| find_any_unused_file(&task_values.input_files, used_files))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("target") || flag_lower.contains("target") {
        return find_matching_file(&task_values.input_files, used_files, &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"])
            .or_else(|| find_any_unused_file(&task_values.input_files, used_files))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("subject") || flag_lower.contains("subject") {
        return find_matching_file(&task_values.input_files, used_files, &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"])
            .or_else(|| find_any_unused_file(&task_values.input_files, used_files))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("thread") || desc_lower.contains("cpu") || flag_lower == "-@" || flag_lower.contains("thread") {
        return task_values.numbers.iter()
            .find(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 1.0 && v <= 128.0 })
            .cloned()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("reference") || flag_lower.contains("ref") || flag_lower.contains("genome") {
        if desc_lower.contains("dir") || desc_lower.contains("directory") || desc_lower.contains("path") {
            return task_values.genome_dirs.first().cloned()
                .or_else(|| entry.default.clone());
        }
        return task_values.reference_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| {
                find_matching_file(&task_values.input_files, used_files, &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"])
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("index") || flag_lower == "-x" || flag_lower.contains("index-prefix") {
        if desc_lower.contains("dir") || desc_lower.contains("path") {
            return task_values.genome_dirs.first().cloned()
                .or_else(|| entry.default.clone());
        }
        return task_values.reference_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| {
                task_values.input_files.iter()
                    .find(|f| {
                        let fl = f.to_ascii_lowercase();
                        (fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                            || fl.contains("index") || fl.contains("genome"))
                            && !used_files.contains(&fl)
                    })
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("database") || flag_lower.contains("db") {
        return task_values.database_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("annotation") || desc_lower.contains("gtf") || desc_lower.contains("gff") {
        return task_values.annotation_files.iter()
            .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| {
                find_matching_file(&task_values.input_files, used_files, &[".gtf", ".gff", ".gff3"])
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("bed") {
        return find_matching_file(&task_values.input_files, used_files, &[".bed", ".bed.gz"])
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("fasta") || desc_lower.contains("fna") {
        return find_matching_file(&task_values.input_files, used_files, &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"])
            .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("species") {
        let species_map: &[(&str, &str)] = &[
            ("human", "human"), ("mouse", "mouse"), ("arabidopsis", "arabidopsis"),
            ("fly", "fly"), ("yeast", "yeast"), ("ecoli", "ecoli"), ("e. coli", "ecoli"),
            ("zebrafish", "zebrafish"), ("rat", "rat"), ("chicken", "chicken"),
            ("worm", "worm"), ("celegans", "celegans"), ("drosophila", "drosophila"),
        ];
        for (pattern, value) in species_map {
            if task_lower.contains(pattern) {
                return Some(value.to_string());
            }
        }
        return entry.default.clone();
    }

    if desc_lower.contains("seed") {
        return task_values.numbers.iter()
            .find(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 1.0 && v <= 999999.0 })
            .cloned()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("k") || flag_lower.contains("k=") || flag_lower == "-k" {
        return task_values.numbers.iter()
            .find(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 1.0 && v <= 100.0 })
            .cloned()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("region") || flag_lower.contains("region") || flag_lower.contains("chrom") {
        let chr_patterns = ["chr1", "chr2", "chr3", "chr4", "chr5", "chr6", "chr7", "chr8",
            "chr9", "chr10", "chr11", "chr12", "chr13", "chr14", "chr15", "chr16", "chr17",
            "chr18", "chr19", "chr20", "chr21", "chr22", "chrx", "chry", "chrm"];
        for pat in &chr_patterns {
            if task_lower.contains(pat) {
                return Some(pat.to_string());
            }
        }
        return entry.default.clone();
    }

    if desc_lower.contains("runmode") || flag_lower.contains("runmode") {
        if task_lower.contains("genomegenerate") || task_lower.contains("generate genome") || task_lower.contains("genome index") {
            return Some("genomeGenerate".to_string());
        }
        return Some("alignReads".to_string());
    }

    if desc_lower.contains("readfilescommand") || flag_lower.contains("readfilescommand") {
        if task_values.input_files.iter().any(|f| f.to_ascii_lowercase().ends_with(".gz")) {
            return Some("zcat".to_string());
        }
        return None;
    }

    if desc_lower.contains("outsamtype") || flag_lower.contains("outsamtype") {
        return Some("BAM SortedByCoordinate".to_string());
    }

    if desc_lower.contains("outfilenamprefix") || flag_lower.contains("outfilenamprefix") {
        return task_values.output_files.first()
            .map(|f| {
                let path = std::path::Path::new(f);
                path.parent()
                    .map(|p| format!("{}/", p.to_string_lossy()))
                    .unwrap_or_else(|| "".to_string())
            })
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("outfmt") || desc_lower.contains("output format") || flag_lower.contains("outfmt") {
        if !entry.enum_values.is_empty() {
            if let Some(best) = entry.enum_values.iter().find(|v| {
                let v_lower = v.to_ascii_lowercase();
                task_lower.contains(&v_lower) || task_lower.contains(&v_lower.replace("_", " ").replace("-", " "))
            }) {
                return Some(best.clone());
            }
            return Some(entry.enum_values[0].clone());
        }
        if task_lower.contains("bam") { return Some("bam".to_string()); }
        if task_lower.contains("sam") { return Some("sam".to_string()); }
        if task_lower.contains("vcf") { return Some("vcf".to_string()); }
        if task_lower.contains("json") { return Some("json".to_string()); }
        if task_lower.contains("tsv") { return Some("tsv".to_string()); }
        if task_lower.contains("csv") { return Some("csv".to_string()); }
        return entry.default.clone();
    }

    if desc_lower.contains("readgroup") || flag_lower.contains("read-group") || flag_lower.contains("rg") {
        if task_lower.contains("read group") || task_lower.contains("readgroup") || task_lower.contains("rg_") {
            let rg_match = task_lower.split("read group")
                .nth(1)
                .or_else(|| task_lower.split("readgroup").nth(1))
                .or_else(|| task_lower.split("rg").nth(1));
            if let Some(rest) = rg_match {
                let id = rest.trim_start()
                    .split(|c: char| c.is_whitespace() || c == '=' || c == ':')
                    .filter(|s| !s.is_empty())
                    .next();
                if let Some(id) = id {
                    return Some(id.to_string());
                }
            }
        }
        return entry.default.clone();
    }

    if desc_lower.contains("library") && (desc_lower.contains("name") || desc_lower.contains("id")) {
        for word in task_lower.split_whitespace() {
            if word.starts_with("lib") || word.contains("library") {
                return Some(word.to_string());
            }
        }
        return entry.default.clone();
    }

    if desc_lower.contains("sample") && (desc_lower.contains("name") || desc_lower.contains("id")) {
        for word in task_lower.split_whitespace() {
            if word.starts_with("sample") || word.starts_with("s_") {
                return Some(word.to_string());
            }
        }
        return entry.default.clone();
    }

    if desc_lower.contains("platform") || flag_lower.contains("platform") {
        if task_lower.contains("illumina") { return Some("ILLUMINA".to_string()); }
        if task_lower.contains("pacbio") { return Some("PACBIO".to_string()); }
        if task_lower.contains("ont") || task_lower.contains("oxford") { return Some("ONT".to_string()); }
        return entry.default.clone();
    }

    if (desc_lower.contains("preset") || flag_lower.contains("preset"))
        && !desc_lower.contains("format") {
        if task_lower.contains("very-sensitive") { return Some("very-sensitive".to_string()); }
        if task_lower.contains("sensitive") { return Some("sensitive".to_string()); }
        if task_lower.contains("very-fast") { return Some("very-fast".to_string()); }
        if task_lower.contains("fast") && !task_lower.contains("fastq") { return Some("fast".to_string()); }
        if task_lower.contains("pacbio") || task_lower.contains("hifi") { return Some("map-pb".to_string()); }
        if task_lower.contains("ont") || task_lower.contains("nanopore") { return Some("map-ont".to_string()); }
        if task_lower.contains("sr") || task_lower.contains("short-read") { return Some("sr".to_string()); }
        return entry.default.clone();
    }

    if desc_lower.contains("evalue") || desc_lower.contains("e-value") || flag_lower.contains("evalue") {
        return task_values.numbers.iter()
            .filter(|n| { let v: f64 = n.parse().unwrap_or(0.0); v > 0.0 && v < 1.0 })
            .cloned()
            .next()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("min-length") || desc_lower.contains("min_len") || flag_lower.contains("minlen") {
        return task_values.numbers.iter()
            .filter(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 50.0 })
            .cloned()
            .next()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("max-length") || desc_lower.contains("max_len") || flag_lower.contains("maxlen") {
        return task_values.numbers.iter()
            .filter(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 50.0 })
            .cloned()
            .next()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("min-quality") || desc_lower.contains("min_qual") || flag_lower.contains("minqual") {
        return task_values.numbers.iter()
            .filter(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 1.0 && v <= 60.0 })
            .cloned()
            .next()
            .or_else(|| entry.default.clone());
    }

    if desc_lower.contains("coverage") || desc_lower.contains("depth") || flag_lower.contains("cov") {
        if flag_lower.contains("min") || desc_lower.contains("minimum") {
            return task_values.numbers.iter()
                .filter(|n| { let v: f64 = n.parse().unwrap_or(0.0); v >= 1.0 && v <= 1000.0 })
                .cloned()
                .next()
                .or_else(|| entry.default.clone());
        }
        return entry.default.clone();
    }

    if !entry.enum_values.is_empty() {
        if let Some(best) = entry.enum_values.iter().find(|v| {
            let v_lower = v.to_ascii_lowercase();
            let v_parts: Vec<&str> = v_lower.split(|c: char| c == '_' || c == '-' || c == ' ')
                .filter(|p| p.len() >= 3)
                .collect();
            task_lower.contains(&v_lower)
                || task_lower.contains(&v_lower.replace("_", " ").replace("-", " "))
                || v_parts.iter().any(|p| task_lower.contains(p))
        }) {
            return Some(best.clone());
        }
        if let Some(best) = entry.enum_values.iter().find(|v| {
            let v_lower = v.to_ascii_lowercase();
            let v_parts: Vec<&str> = v_lower.split(|c: char| c == '_' || c == '-' || c == ' ')
                .filter(|p| p.len() >= 3)
                .collect();
            task_lower.split_whitespace()
                .filter(|w| w.len() >= 3)
                .any(|w| v_lower.contains(w) || v_parts.iter().any(|p| *p == w))
        }) {
            return Some(best.clone());
        }
        return Some(entry.enum_values[0].clone());
    }

    if let Some(ref vt) = entry.value_type {
        let vt_lower = vt.to_ascii_lowercase();
        if vt_lower.contains("file") || vt_lower.contains("path") || vt_lower.contains("dir") {
            if desc_lower.contains("output") || flag_lower.contains("out") {
                return task_values.output_files.iter()
                    .find(|f| !used_files.contains(&f.to_ascii_lowercase()))
                    .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                    .or_else(|| infer_output_from_input_for_rule(desc_lower, flag_lower, &task_values.input_files, used_files))
                    .or_else(|| entry.default.clone());
            }
            return find_any_unused_file(&task_values.input_files, used_files)
                .map(|f| { used_files.insert(f.to_ascii_lowercase()); f.clone() })
                .or_else(|| entry.default.clone());
        }
        if vt_lower.contains("int") || vt_lower.contains("float") || vt_lower.contains("num") {
            return task_values.numbers.first().cloned()
                .or_else(|| entry.default.clone());
        }
        if vt_lower.contains("bool") {
            return Some("true".to_string());
        }
    }

    entry.default.clone()
}

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
        .filter(|w| w.len() > 2 && !STOP_WORDS.contains(w))
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
        let mut score = 0i32;

        if entry.required {
            score += 100;
        }

        score += compute_word_overlap_score(&task_words, &desc_lower);

        for word in &task_words {
            if flag_lower.contains(word) { score += 3; }
        }

        let flag_name_parts: Vec<&str> = flag_lower
            .trim_start_matches('-')
            .split(|c: char| c == '-' || c == '_')
            .filter(|p| p.len() >= 3)
            .collect();
        for part in &flag_name_parts {
            if task_lower.contains(part) {
                score += 6;
            }
            for word in &task_words {
                if word.contains(part) || part.contains(word) {
                    score += 3;
                }
            }
        }

        score += compute_semantic_bonus(&desc_lower, &flag_lower, &task_lower, task_values);
        score += compute_negative_score(&desc_lower, &flag_lower, &task_lower);

        if let Some(ref sub) = effective_subcommand {
            let sub_lower = sub.to_lowercase();
            if desc_lower.contains(&sub_lower) { score += 5; }
        }

        score
    };

    let mut all_flags: Vec<&FlagEntry> = sdoc.flag_catalog.iter().collect();
    all_flags.sort_by(|a, b| score_flag(b).cmp(&score_flag(a)));

    let mut included_flags: std::collections::HashSet<String> = std::collections::HashSet::new();

    let task_has_output = !task_values.output_files.is_empty();
    let task_has_input = !task_values.input_files.is_empty();

    for entry in &all_flags {
        let flag_key = entry.flag.clone();
        if included_flags.contains(&flag_key) { continue; }
        if let Some(ref alt) = entry.alt_form {
            if included_flags.contains(alt) { continue; }
        }

        let score = score_flag(entry);
        let desc_lower = entry.description.to_ascii_lowercase();
        let flag_lower = entry.flag.to_ascii_lowercase();

        let is_output_flag = (desc_lower.contains("output") || flag_lower.contains("out"))
            && !desc_lower.contains("stdout") && !desc_lower.contains("format");
        let is_input_flag = desc_lower.contains("input") || flag_lower.contains("in");
        let is_thread_flag = desc_lower.contains("thread") || desc_lower.contains("cpu")
            || flag_lower == "-@" || flag_lower.contains("thread");
        let is_ref_flag = desc_lower.contains("reference") || flag_lower.contains("ref")
            || desc_lower.contains("genome") || flag_lower == "-x";
        let is_db_flag = desc_lower.contains("database") || flag_lower.contains("db");
        let is_annotation_flag = desc_lower.contains("annotation") || desc_lower.contains("gtf") || desc_lower.contains("gff");
        let is_quality_flag = desc_lower.contains("quality") || desc_lower.contains("qual")
            || flag_lower.contains("qual") || flag_lower.contains("minq")
            || desc_lower.contains("min-quality") || desc_lower.contains("mapping quality");
        let is_region_flag = desc_lower.contains("region") || desc_lower.contains("chrom")
            || desc_lower.contains("interval") || flag_lower.contains("region")
            || flag_lower == "-r" && desc_lower.contains("region");
        let is_sample_flag = desc_lower.contains("sample") || flag_lower.contains("sample")
            || desc_lower.contains("rg-id") || desc_lower.contains("read-group");
        let is_format_flag = (desc_lower.contains("format") || flag_lower.contains("format"))
            && !desc_lower.contains("stdout");
        let is_index_flag = desc_lower.contains("index") || flag_lower.contains("index")
            || desc_lower.contains("prefix") && desc_lower.contains("index");

        let should_auto_include = (is_output_flag && (task_has_output || task_values.input_files.iter().any(|f| {
            let fl = f.to_ascii_lowercase();
            fl.ends_with(".bam") || fl.ends_with(".sam") || fl.ends_with(".fastq") || fl.ends_with(".fq")
                || fl.ends_with(".vcf") || fl.ends_with(".fa") || fl.ends_with(".fasta")
        })))
            || (is_input_flag && task_has_input)
            || (is_ref_flag && (!task_values.reference_files.is_empty() || !task_values.genome_dirs.is_empty()))
            || (is_db_flag && !task_values.database_files.is_empty())
            || (is_annotation_flag && !task_values.annotation_files.is_empty())
            || (is_quality_flag && task_lower.contains("quality"))
            || (is_region_flag && (task_lower.contains("region") || task_lower.contains("chromosome")))
            || (is_sample_flag && (task_lower.contains("sample") || task_lower.contains("read group")))
            || (is_format_flag && (task_lower.contains("format") || task_lower.contains("bam") || task_lower.contains("vcf")))
            || (is_index_flag && (task_lower.contains("index") || task_lower.contains("build")));

        let should_include = entry.required || should_auto_include || score >= 5;

        if !should_include { continue; }

        if !entry.required && !is_output_flag && !is_input_flag && !is_ref_flag
            && !is_db_flag && !is_annotation_flag && !is_quality_flag
            && !is_region_flag && !is_sample_flag && !is_format_flag
            && !is_index_flag && included_flags.len() >= 12 {
            continue;
        }

        let value = resolve_flag_value(entry, &desc_lower, &flag_lower, &task_lower, task_values, &mut used_files);

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

    if !positional_remaining.is_empty() {
        for f in &positional_remaining {
            parts.push((*f).clone());
        }
    } else if !sdoc.usage_pattern.positional_args.is_empty() && task_values.input_files.is_empty() {
        // no input files extracted from task
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
