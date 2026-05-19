pub struct TaskValues {
    pub input_files: Vec<String>,
    pub output_files: Vec<String>,
    pub numbers: Vec<String>,
    pub keywords: Vec<String>,
    pub genome_dirs: Vec<String>,
    pub reference_files: Vec<String>,
    pub read_files: Vec<String>,
    pub annotation_files: Vec<String>,
    pub database_files: Vec<String>,
}

fn is_output_file_indicator(task_lower: &str, filename_lower: &str) -> bool {
    if filename_lower.contains("output") || filename_lower.contains("out.") || filename_lower.contains("result") {
        return true;
    }
    if task_lower.contains(&format!("to {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("save {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("write {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("generate {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("produce {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("create {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("export {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("output to {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("store {}", filename_lower)) {
        return true;
    }
    if task_lower.contains(&format!("convert to {}", filename_lower)) {
        return true;
    }
    let output_patterns = ["output", "save", "write", "generate", "produce", "create", "export", "store", "convert to", "result"];
    for pattern in &output_patterns {
        if let Some(pos) = task_lower.find(*pattern) {
            let after = &task_lower[pos + pattern.len()..];
            let after_trimmed = after.trim_start();
            if after_trimmed.starts_with(filename_lower) || after_trimmed.starts_with(&filename_lower.replace('_', "-")) {
                return true;
            }
        }
    }
    false
}

fn classify_file(filename: &str) -> FileClass {
    let fl = filename.to_ascii_lowercase();
    if fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
        || fl.ends_with(".fa.gz") || fl.ends_with(".fasta.gz") || fl.ends_with(".fna.gz")
        || fl.ends_with(".mfa") || fl.ends_with(".ffa") || fl.ends_with(".fa.bz2") {
        return FileClass::Reference;
    }
    if fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".fq.gz") || fl.ends_with(".fastq.gz")
        || fl.ends_with(".fq.bz2") || fl.ends_with(".fastq.bz2") {
        return FileClass::Reads;
    }
    if fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".gff3")
        || fl.ends_with(".gtf.gz") || fl.ends_with(".gff.gz") || fl.ends_with(".gff3.gz")
        || fl.ends_with(".gff3.bz2") || fl.ends_with(".bed") || fl.ends_with(".bed.gz") {
        return FileClass::Annotation;
    }
    if fl.ends_with(".dmnd") || fl.ends_with(".ndb") || fl.ends_with(".msh")
        || fl.ends_with(".k2d") || fl.ends_with(".hdr") || fl.ends_with(".bwt")
        || fl.ends_with(".pac") || fl.ends_with(".sa") || fl.ends_with(".amb")
        || fl.ends_with(".ann") || fl.ends_with(".fai") || fl.ends_with(".hmm")
        || fl.ends_with(".sto") || fl.ends_with(".pin") || fl.ends_with(".psq")
        || fl.ends_with(".phr") || fl.ends_with(".pog") || fl.ends_with(".pos")
        || fl.ends_with(".psi") || fl.ends_with(".nsq") || fl.ends_with(".nin")
        || fl.ends_with(".nhr") || fl.ends_with(".nog") || fl.ends_with(".nos")
        || fl.ends_with(".nsi") || fl.ends_with(".00") || fl.ends_with(".idx") {
        return FileClass::Database;
    }
    if fl.contains("_index") || fl.contains("_dir") || fl.contains("genome_dir")
        || fl.contains("genomedir") || fl.contains("star_index") || fl.contains("bowtie2_index")
        || fl.contains("hisat2_index") || fl.contains("bismark_genome")
        || fl.contains("index_dir") || fl.contains("db_dir") || fl.contains("ref_dir") {
        if !fl.contains('.') || fl.ends_with('/') {
            return FileClass::GenomeDir;
        }
    }
    if fl.contains("genome") && (fl.contains("dir") || fl.ends_with("/")) && !fl.contains('.') {
        return FileClass::GenomeDir;
    }
    FileClass::Other
}

enum FileClass {
    Reference,
    Reads,
    Annotation,
    Database,
    GenomeDir,
    Other,
}

fn push_unique(list: &mut Vec<String>, item: &str) {
    let item_lower = item.to_ascii_lowercase();
    if !list.iter().any(|f| f.to_ascii_lowercase() == item_lower) {
        list.push(item.to_string());
    }
}

pub fn extract_task_values(task: &str) -> TaskValues {
    let mut input_files = Vec::new();
    let mut output_files = Vec::new();
    let mut numbers = Vec::new();
    let mut keywords = Vec::new();
    let mut genome_dirs = Vec::new();
    let mut reference_files = Vec::new();
    let mut read_files = Vec::new();
    let mut annotation_files = Vec::new();
    let mut database_files = Vec::new();

    let task_lower = task.to_ascii_lowercase();
    let bio_extensions = [
        ".bam", ".sam", ".fq", ".fastq", ".fa", ".fasta", ".fna",
        ".vcf", ".bcf", ".bed", ".gtf", ".gff", ".gff3",
        ".gz", ".bz2", ".tar", ".zip",
        ".txt", ".csv", ".tsv", ".log", ".out",
        ".saf", ".sfs", ".idx", ".bam.bai",
        ".pdb", ".pdbx", ".cif",
        ".h5", ".hdf5", ".loom",
        ".mtx", ".tsv.gz",
        ".dmnd", ".ndb", ".msh", ".k2d",
        ".fa.gz", ".fasta.gz", ".fna.gz",
        ".fq.gz", ".fastq.gz",
        ".gtf.gz", ".gff.gz", ".gff3.gz",
        ".bai", ".csi", ".tbi",
        ".cram", ".sam.gz",
        ".bed.gz", ".vcf.gz", ".bcf.gz",
        ".narrowpeak", ".broadpeak", ".bedgraph", ".bw", ".bigwig", ".wig",
        ".sif", ".ped", ".map", ".bim", ".fam",
        ".profile", ".motif", ".counts", ".tab",
        ".hmm", ".sto", ".a2m", ".afa",
        ".pheno", ".cov", ".cnt",
        ".report", ".matrix", ".out",
    ];

    let paren_pattern = regex::Regex::new(r"\(([^)]+)\)").ok();
    if let Some(re) = paren_pattern {
        for cap in re.captures_iter(task) {
            let content = cap[1].to_string();
            for part in content.split(',') {
                let part = part.trim();
                if part.contains('.') || part.contains('/') || part.contains("_index") || part.contains("_dir") {
                    let pl = part.to_ascii_lowercase();
                    let is_output = is_output_file_indicator(&task_lower, &pl);
                    if is_output {
                        push_unique(&mut output_files, part);
                    } else {
                        push_unique(&mut input_files, part);
                    }
                    match classify_file(part) {
                        FileClass::Reference => push_unique(&mut reference_files, part),
                        FileClass::Reads => push_unique(&mut read_files, part),
                        FileClass::Annotation => push_unique(&mut annotation_files, part),
                        FileClass::Database => push_unique(&mut database_files, part),
                        FileClass::GenomeDir => push_unique(&mut genome_dirs, part),
                        FileClass::Other => {}
                    }
                }
            }
        }
    }

    for word in task.split_whitespace() {
        let word_clean = word.trim_matches(|c: char| c == ',' || c == ';' || c == ':' || c == '(' || c == ')' || c == '"' || c == '\'');
        let wl = word_clean.to_ascii_lowercase();
        if bio_extensions.iter().any(|ext| wl.ends_with(ext)) {
            let already_in = input_files.iter().any(|f| f.to_ascii_lowercase() == wl)
                || output_files.iter().any(|f| f.to_ascii_lowercase() == wl);
            if !already_in {
                let is_output = is_output_file_indicator(&task_lower, &wl);
                if is_output {
                    push_unique(&mut output_files, word_clean);
                } else {
                    push_unique(&mut input_files, word_clean);
                }
                match classify_file(word_clean) {
                    FileClass::Reference => push_unique(&mut reference_files, word_clean),
                    FileClass::Reads => push_unique(&mut read_files, word_clean),
                    FileClass::Annotation => push_unique(&mut annotation_files, word_clean),
                    FileClass::Database => push_unique(&mut database_files, word_clean),
                    FileClass::GenomeDir => push_unique(&mut genome_dirs, word_clean),
                    FileClass::Other => {}
                }
            }
        } else if wl.contains("_index") || wl.contains("_dir") || wl.contains("genome_dir")
            || wl.contains("genomedir") || wl.contains("star_index") {
            if !wl.starts_with('-') {
                push_unique(&mut genome_dirs, word_clean);
                push_unique(&mut input_files, word_clean);
            }
        }
    }

    for word in task.split_whitespace() {
        let word_clean = word.trim_matches(|c: char| c == ',' || c == ';' || c == ':' || c == '(' || c == ')');
        if word_clean.starts_with('-') || word_clean.contains('.') || word_clean.contains('/') {
            continue;
        }
        if word_clean.chars().all(|c| c.is_ascii_digit()) && word_clean.len() <= 10 {
            if !numbers.contains(&word_clean.to_string()) {
                numbers.push(word_clean.to_string());
            }
        } else if let Some(val) = word_clean.strip_suffix(|c: char| c == 'k' || c == 'K' || c == 'm' || c == 'M' || c == 'g' || c == 'G') {
            if val.chars().all(|c| c.is_ascii_digit() || c == '.') {
                if !numbers.contains(&word_clean.to_string()) {
                    numbers.push(word_clean.to_string());
                }
            }
        }
    }

    if let Some(re) = regex::Regex::new(r"K\s*=\s*(\d+)").ok() {
        for cap in re.captures_iter(task) {
            let k_val = cap[1].to_string();
            if !numbers.contains(&k_val) {
                numbers.push(k_val);
            }
        }
    }

    if let Some(re) = regex::Regex::new(r"(\d+)\s*-?\s*fold").ok() {
        for cap in re.captures_iter(task) {
            let fold_val = cap[1].to_string();
            if !numbers.contains(&fold_val) {
                numbers.push(fold_val);
            }
        }
    }

    if let Some(re) = regex::Regex::new(r"threads?\s*[=:]\s*(\d+)").ok() {
        for cap in re.captures_iter(task) {
            let thread_val = cap[1].to_string();
            if !numbers.contains(&thread_val) {
                numbers.push(thread_val);
            }
        }
    }

    let stop_words = [
        "the", "and", "for", "with", "from", "into", "using", "input", "output",
        "file", "files", "run", "based", "also", "then", "that", "this",
        "which", "where", "when", "how", "can", "will", "has", "been", "was",
    ];
    for word in task.split_whitespace() {
        let word_clean = word.trim_matches(|c: char| c == ',' || c == ';' || c == ':' || c == '(' || c == ')');
        let wl = word_clean.to_ascii_lowercase();
        if wl.len() >= 4 && !stop_words.contains(&wl.as_str())
            && !word_clean.starts_with('-')
            && !word_clean.contains('.')
            && !word_clean.chars().all(|c| c.is_ascii_digit())
        {
            if !keywords.contains(&word_clean.to_string()) {
                keywords.push(word_clean.to_string());
            }
        }
    }

    TaskValues {
        input_files,
        output_files,
        numbers,
        keywords,
        genome_dirs,
        reference_files,
        read_files,
        annotation_files,
        database_files,
    }
}

pub fn rule_based_subcommand_match(
    task: &str,
    subcommands: &[String],
    subcommand_descriptions: &[(String, String)],
) -> Option<String> {
    let task_lower = task.to_ascii_lowercase();

    for sub in subcommands {
        let sub_lower = sub.to_ascii_lowercase();
        if task_lower.split_whitespace().any(|w| w == sub_lower) {
            return Some(sub.clone());
        }
    }

    for sub in subcommands {
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

    let synonym_map: &[(&[&str], &[&str])] = &[
        (&["sort", "sorted", "sort by"], &["sort"]),
        (&["index", "indexing", "create index"], &["index", "faidx"]),
        (&["view", "convert", "extract", "display"], &["view"]),
        (&["merge", "combine", "join"], &["merge", "concat"]),
        (&["call", "genotype", "variant call"], &["call", "mpileup2cns", "mpileup2snp"]),
        (&["align", "mapping", "map"], &["mem", "align", "map"]),
        (&["quantify", "count", "expression"], &["count", "quant"]),
        (&["peak", "callpeak", "chip-seq"], &["callpeak", "findPeaks"]),
        (&["annotate", "annotation"], &["annotate", "ann"]),
        (&["coverage", "depth"], &["depth", "coverage", "bamCoverage"]),
        (&["duplicate", "dedup", "markdup"], &["MarkDuplicates", "markdup", "deduplicate_bismark", "rmdup"]),
        (&["fastq", "bam2fq"], &["bam2fq"]),
        (&["stats", "statistics", "summary"], &["stats", "flagstat", "idxstats"]),
        (&["motif"], &["findMotifsGenome"]),
        (&["methylation"], &["bismark_methylation_extractor"]),
        (&["genome", "prepare", "index"], &["bismark_genome_preparation", "genomeGenerate"]),
        (&["download", "prefetch", "fetch"], &["prefetch", "fasterq-dump", "fastq-dump"]),
        (&["build", "database"], &["build", "bakta_db", "hmmpress", "makeblastdb"]),
        (&["classify", "taxonomic"], &["classify_wf", "classify"]),
        (&["plot", "heatmap", "visualize"], &["plotHeatmap", "plotProfile"]),
        (&["matrix", "compute"], &["computeMatrix"]),
        (&["filter", "select", "subset"], &["filter", "SelectVariants", "VariantFiltration"]),
        (&["somatic", "tumor"], &["Mutect2", "somatic"]),
        (&["haplotype", "germline"], &["HaplotypeCaller"]),
        (&["bamqc"], &["bamqc"]),
        (&["rnaseq", "rna-seq"], &["rnaseq"]),
        (&["bamcoverage"], &["bamCoverage"]),
        (&["computematrix"], &["computeMatrix"]),
        (&["plotheatmap"], &["plotHeatmap"]),
        (&["plotprofile"], &["plotProfile"]),
        (&["prepare", "reference", "rsem"], &["rsem-prepare-reference"]),
        (&["calculate", "expression", "rsem"], &["rsem-calculate-expression"]),
        (&["paired", "pe"], &["PE"]),
        (&["single", "se"], &["SE"]),
        (&["realSFS", "sfs"], &["realSFS"]),
        (&["fst"], &["fst"]),
        (&["hmmsearch"], &["hmmsearch"]),
        (&["hmmscan"], &["hmmscan"]),
        (&["hmmbuild"], &["hmmbuild"]),
        (&["hmmpress"], &["hmmpress"]),
        (&["blastn", "nucleotide blast"], &["blastn"]),
        (&["blastp", "protein blast"], &["blastp"]),
        (&["blastx"], &["blastx"]),
        (&["makeblastdb"], &["makeblastdb"]),
        (&["nucmer"], &["nucmer"]),
        (&["promer"], &["promer"]),
        (&["gff2gtf", "gff to gtf"], &["agat_convert_sp_gff2gtf"]),
        (&["gff2bed", "gff to bed"], &["agat_convert_sp_gff2bed"]),
        (&["batch"], &["batch"]),
        (&["segment"], &["segment"]),
        (&["fix", "standardize"], &["fix"]),
        (&["pileup"], &["pileup", "mpileup"]),
        (&["phase", "phasing"], &["phase"]),
        (&["haplotag"], &["haplotag"]),
        (&["discover"], &["discover"]),
        (&["consensus", "polish", "correct"], &["consensus"]),
        (&["assemble", "assembly"], &["assemble"]),
        (&["predict", "gene predict"], &["predict"]),
        (&["extract", "sequence"], &["extract", "agat_sp_extract_sequences"]),
        (&["statistics", "stats", "gff stats"], &["agat_sp_statistics", "statistics", "stats"]),
        (&["convert", "format"], &["agat_convert_sp_gff2gtf", "agat_convert_sp_gff2bed", "convert"]),
        (&["makedb", "database build"], &["makedb"]),
        (&["easy-search", "search"], &["easy-search"]),
        (&["easy-cluster", "cluster"], &["easy-cluster"]),
        (&["createdb"], &["createdb"]),
        (&["fimo", "motif scan"], &["fimo"]),
        (&["meme", "motif discover"], &["meme"]),
        (&["dreme"], &["dreme"]),
        (&["ame"], &["ame"]),
        (&["bamcompare"], &["bamCompare"]),
        (&["plotfingerprint"], &["plotFingerprint"]),
        (&["multibamsummary"], &["multiBamSummary"]),
        (&["parse"], &["parse"]),
        (&["dedup"], &["dedup"]),
        (&["select"], &["select"]),
        (&["split"], &["split"]),
        (&["call-mods"], &["call-mods"]),
        (&["summary"], &["summary"]),
        (&["extract"], &["extract"]),
        (&["pileup"], &["pileup"]),
        (&["motif-bed"], &["motif-bed"]),
        (&["sample-probs"], &["sample-probs"]),
        (&["update-tags"], &["update-tags"]),
        (&["delta-filter", "filter"], &["delta-filter"]),
        (&["show-coords"], &["show-coords"]),
        (&["show-snps"], &["show-snps"]),
        (&["dnadiff"], &["dnadiff"]),
        (&["reheader"], &["reheader"]),
        (&["collate"], &["collate"]),
        (&["cat"], &["cat"]),
        (&["calmd"], &["calmd"]),
        (&["fixmate"], &["fixmate"]),
        (&["bam2fq"], &["bam2fq"]),
        (&["dict"], &["dict"]),
        (&["target"], &["target"]),
        (&["call"], &["call"]),
        (&["bamqc"], &["bamqc"]),
        (&["rnaseq"], &["rnaseq"]),
        (&["toTDF", "tdf", "count"], &["toTDF"]),
        (&["prefetch"], &["prefetch"]),
        (&["fasterq-dump", "fastq-dump", "fastq", "dump"], &["fasterq-dump", "fastq-dump"]),
        (&["convert"], &["convert"]),
        (&["findpeaks"], &["findPeaks"]),
        (&["findMotifsGenome"], &["findMotifsGenome"]),
        (&["annotatePeaks"], &["annotatePeaks"]),
        (&["predict"], &["predict"]),
        (&["bin", "binning"], &["bin"]),
        (&["ref", "reference", "index"], &["ref"]),
        (&["count", "quantify"], &["count"]),
    ];

    for (task_synonyms, sub_synonyms) in synonym_map {
        let any_match = task_synonyms.iter().any(|syn| task_lower.contains(syn));
        if any_match {
            for sub in subcommands {
                let sub_lower = sub.to_ascii_lowercase();
                if sub_synonyms.iter().any(|syn| sub_lower == *syn || sub_lower.contains(syn)) {
                    return Some(sub.clone());
                }
            }
        }
    }

    let task_words: Vec<&str> = task_lower.split_whitespace().collect();
    let mut best_match: Option<String> = None;
    let mut best_score = 0;

    for (sub, desc) in subcommand_descriptions {
        let desc_lower = desc.to_ascii_lowercase();
        let sub_lower = sub.to_ascii_lowercase();
        let mut score = 0;

        for word in &task_words {
            if word.len() < 3 { continue; }
            if desc_lower.contains(word) {
                score += 2;
            }
            if sub_lower.contains(word) {
                score += 3;
            }
        }

        if score > best_score {
            best_score = score;
            best_match = Some(sub.clone());
        }
    }

    if best_score >= 2 {
        return best_match;
    }

    None
}

pub fn detect_subcommand_for_tool(tool: &str, task: &str, args: &[String]) -> Option<String> {
    let tool_lower = tool.to_ascii_lowercase();
    let task_lower = task.to_ascii_lowercase();

    match tool_lower.as_str() {
        "star" => {
            Some("_NO_SUB_".to_string())
        }
        "porechop" => {
            Some("_NO_SUB_".to_string())
        }
        "chopper" => {
            Some("_NO_SUB_".to_string())
        }
        "flye" => {
            if args.iter().any(|a| a.starts_with("--nano-") || a.starts_with("--pacbio-")) {
                let mode_flag = args.iter().find(|a| a.starts_with("--nano-") || a.starts_with("--pacbio-"));
                mode_flag.map(|f| f.clone())
            } else if task_lower.contains("pacbio") && (task_lower.contains("hifi") || task_lower.contains("hi-fi") || task_lower.contains("ccs")) {
                Some("--pacbio-hifi".to_string())
            } else if task_lower.contains("pacbio") {
                Some("--pacbio-raw".to_string())
            } else if task_lower.contains("nano-hq") || task_lower.contains("high-quality") {
                Some("--nano-hq".to_string())
            } else {
                Some("--nano-raw".to_string())
            }
        }
        "chromap" => {
            if args.iter().any(|a| a == "--preset") {
                args.iter().position(|a| a == "--preset")
                    .and_then(|i| args.get(i + 1).map(|v| format!("--preset {}", v)))
            } else if task_lower.contains("hic") || task_lower.contains("hi-c") {
                Some("--preset hic".to_string())
            } else if task_lower.contains("chip") || task_lower.contains("chip-seq") || task_lower.contains("chipseq") {
                Some("--preset chip".to_string())
            } else {
                Some("--preset atac".to_string())
            }
        }
        "trim_galore" => {
            Some("_NO_SUB_".to_string())
        }
        "fastp" => {
            Some("_NO_SUB_".to_string())
        }
        "fastqc" => {
            Some("_NO_SUB_".to_string())
        }
        "multiqc" => {
            Some("_NO_SUB_".to_string())
        }
        "mosdepth" => {
            Some("_NO_SUB_".to_string())
        }
        "minimap2" => {
            if args.iter().any(|a| a.starts_with("-x")) {
                args.iter().position(|a| a == "-x")
                    .and_then(|i| args.get(i + 1).map(|v| format!("-x {}", v)))
            } else if task_lower.contains("ont") || task_lower.contains("nanopore") {
                Some("-x map-ont".to_string())
            } else if task_lower.contains("pacbio") || task_lower.contains("pb") {
                Some("-x map-pb".to_string())
            } else if task_lower.contains("splice") || task_lower.contains("rna") {
                Some("-x splice".to_string())
            } else if task_lower.contains("short") || task_lower.contains("sr") {
                Some("-x sr".to_string())
            } else {
                Some("-x map-ont".to_string())
            }
        }
        "spades" => {
            if task_lower.contains("rna") || task_lower.contains("transcriptome") {
                Some("--rna".to_string())
            } else if task_lower.contains("meta") || task_lower.contains("metagenome") {
                Some("--meta".to_string())
            } else if task_lower.contains("isolate") {
                Some("--isolate".to_string())
            } else {
                None
            }
        }
        "trinity" => {
            if task_lower.contains("genome_guided") || task_lower.contains("genome-guided") {
                Some("--genome_guided_bam".to_string())
            } else {
                Some("--seqType".to_string())
            }
        }
        "megahit" => {
            Some("_NO_SUB_".to_string())
        }
        "canu" => {
            Some("_NO_SUB_".to_string())
        }
        "hifiasm" => {
            Some("_NO_SUB_".to_string())
        }
        "bcftools" => {
            if task_lower.contains("mpileup") || task_lower.contains("pileup") {
                Some("mpileup".to_string())
            } else if task_lower.contains("call") && !task_lower.contains("call-mods") {
                Some("call".to_string())
            } else if task_lower.contains("filter") && !task_lower.contains("delta-filter") {
                Some("filter".to_string())
            } else if task_lower.contains("merge") || task_lower.contains("combine") {
                Some("merge".to_string())
            } else if task_lower.contains("concat") {
                Some("concat".to_string())
            } else if task_lower.contains("norm") || task_lower.contains("normalize") || task_lower.contains("split multi-allelic") {
                Some("norm".to_string())
            } else if task_lower.contains("annotate") || task_lower.contains("annotation") {
                Some("annotate".to_string())
            } else if task_lower.contains("stats") || task_lower.contains("statistics") {
                Some("stats".to_string())
            } else if task_lower.contains("query") || task_lower.contains("extract field") || task_lower.contains("custom field") {
                Some("query".to_string())
            } else if task_lower.contains("isec") || task_lower.contains("intersection") || task_lower.contains("shared") {
                Some("isec".to_string())
            } else if task_lower.contains("consensus") {
                Some("consensus".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else if task_lower.contains("sort") {
                Some("sort".to_string())
            } else if task_lower.contains("view") || task_lower.contains("convert") || task_lower.contains("extract") || task_lower.contains("select") || task_lower.contains("snp") {
                Some("view".to_string())
            } else {
                None
            }
        }
        "bismark" => {
            if task_lower.contains("genome_preparation") || task_lower.contains("prepare") || task_lower.contains("index") || task_lower.contains("bisulfite genome") {
                Some("bismark_genome_preparation".to_string())
            } else if task_lower.contains("methylation_extractor") || task_lower.contains("methylation") || task_lower.contains("extract methylation") {
                Some("bismark_methylation_extractor".to_string())
            } else if task_lower.contains("deduplicate") || task_lower.contains("dedup") {
                Some("deduplicate_bismark".to_string())
            } else if task_lower.contains("bismark2report") || task_lower.contains("html report") || task_lower.contains("alignment report") {
                Some("bismark2report".to_string())
            } else if task_lower.contains("coverage2cytosine") || task_lower.contains("cytosine") {
                Some("coverage2cytosine".to_string())
            } else {
                Some("bismark".to_string())
            }
        }
        "bracken" => {
            if task_lower.contains("bracken-build") || task_lower.contains("build") || task_lower.contains("database build") {
                Some("bracken-build".to_string())
            } else if task_lower.contains("combine") || task_lower.contains("merge") {
                Some("combine_bracken_outputs".to_string())
            } else {
                None
            }
        }
        "centrifuge" => {
            if task_lower.contains("centrifuge-build") || task_lower.contains("build") || task_lower.contains("index") || task_lower.contains("custom") && task_lower.contains("database") {
                Some("centrifuge-build".to_string())
            } else if task_lower.contains("kreport") || task_lower.contains("kraken") || task_lower.contains("convert") && task_lower.contains("report") {
                Some("centrifuge-kreport".to_string())
            } else {
                None
            }
        }
        "kraken2" => {
            if task_lower.contains("kraken2-build") || task_lower.contains("build") || task_lower.contains("download") && task_lower.contains("database") || task_lower.contains("standard") && task_lower.contains("database") {
                Some("kraken2-build".to_string())
            } else {
                Some("_NO_SUB_".to_string())
            }
        }
        "diamond" => {
            if task_lower.contains("makedb") || task_lower.contains("make database") || task_lower.contains("build database") {
                Some("makedb".to_string())
            } else if task_lower.contains("blastp") || (task_lower.contains("protein") && task_lower.contains("search")) {
                Some("blastp".to_string())
            } else if task_lower.contains("blastx") || (task_lower.contains("dna") && task_lower.contains("protein")) {
                Some("blastx".to_string())
            } else if task_lower.contains("cluster") && !task_lower.contains("linclust") {
                Some("cluster".to_string())
            } else if task_lower.contains("linclust") || task_lower.contains("linear") {
                Some("linclust".to_string())
            } else {
                None
            }
        }
        "medaka" => {
            if task_lower.contains("consensus") || task_lower.contains("polish") || task_lower.contains("all-in-one") {
                Some("medaka_consensus".to_string())
            } else if task_lower.contains("haploid") && task_lower.contains("variant") {
                Some("medaka_haploid_variant".to_string())
            } else if task_lower.contains("variant") && !task_lower.contains("haploid") {
                Some("medaka_variant".to_string())
            } else if task_lower.contains("list") && task_lower.contains("model") {
                Some("tools".to_string())
            } else if task_lower.contains("inference") {
                Some("medaka inference".to_string())
            } else if task_lower.contains("sequence") || task_lower.contains("stitch") {
                Some("medaka sequence".to_string())
            } else if task_lower.contains("vcf") {
                Some("medaka vcf".to_string())
            } else {
                Some("medaka_consensus".to_string())
            }
        }
        "quast" => {
            if task_lower.contains("metaquast") || task_lower.contains("metagenome") {
                Some("metaquast.py".to_string())
            } else {
                None
            }
        }
        "prokka" => {
            None
        }
        "bakta" => {
            if task_lower.contains("download") && task_lower.contains("database") {
                Some("bakta_db download".to_string())
            } else if task_lower.contains("protein") && (task_lower.contains("directly") || task_lower.contains("fasta")) && !task_lower.contains("trusted") {
                Some("bakta_proteins".to_string())
            } else {
                Some("_NO_SUB_".to_string())
            }
        }
        "eggnog-mapper" => {
            Some("_NO_SUB_".to_string())
        }
        "samtools" => {
            if task_lower.contains("sort") || task_lower.contains("sorted") {
                Some("sort".to_string())
            } else if task_lower.contains("index") || task_lower.contains("bai") {
                Some("index".to_string())
            } else if task_lower.contains("flagstat") || task_lower.contains("flag statistics") {
                Some("flagstat".to_string())
            } else if task_lower.contains("idxstats") {
                Some("idxstats".to_string())
            } else if task_lower.contains("depth") || task_lower.contains("coverage") && !task_lower.contains("bamcoverage") {
                Some("depth".to_string())
            } else if task_lower.contains("merge") || task_lower.contains("combine") {
                Some("merge".to_string())
            } else if task_lower.contains("faidx") || task_lower.contains("fasta index") {
                Some("faidx".to_string())
            } else if task_lower.contains("stats") || task_lower.contains("statistics") {
                Some("stats".to_string())
            } else if task_lower.contains("fastq") || task_lower.contains("bam2fq") {
                Some("fastq".to_string())
            } else if task_lower.contains("markdup") || task_lower.contains("duplicate") {
                Some("markdup".to_string())
            } else if task_lower.contains("fixmate") {
                Some("fixmate".to_string())
            } else if task_lower.contains("calmd") {
                Some("calmd".to_string())
            } else if task_lower.contains("collate") {
                Some("collate".to_string())
            } else if task_lower.contains("mpileup") || task_lower.contains("pileup") {
                Some("mpileup".to_string())
            } else if task_lower.contains("dict") || task_lower.contains("dictionary") {
                Some("dict".to_string())
            } else if task_lower.contains("cat") {
                Some("cat".to_string())
            } else if task_lower.contains("reheader") {
                Some("reheader".to_string())
            } else if task_lower.contains("view") || task_lower.contains("convert") || task_lower.contains("extract") || task_lower.contains("filter") || task_lower.contains("select") {
                Some("view".to_string())
            } else {
                None
            }
        }
        "bowtie2" => {
            if task_lower.contains("bowtie2-build") || task_lower.contains("build") && (task_lower.contains("index") || task_lower.contains("genome")) {
                Some("bowtie2-build".to_string())
            } else if task_lower.contains("inspect") {
                Some("bowtie2-inspect".to_string())
            } else {
                Some("_NO_SUB_".to_string())
            }
        }
        "hisat2" => {
            if task_lower.contains("hisat2-build") || task_lower.contains("build") && (task_lower.contains("index") || task_lower.contains("genome")) {
                Some("hisat2-build".to_string())
            } else {
                Some("_NO_SUB_".to_string())
            }
        }
        "pilon" => {
            Some("_NO_SUB_".to_string())
        }
        "sra-tools" => {
            if task_lower.contains("prefetch") || task_lower.contains("download") {
                Some("prefetch".to_string())
            } else if task_lower.contains("fasterq-dump") || task_lower.contains("fastq") && task_lower.contains("dump") {
                Some("fasterq-dump".to_string())
            } else if task_lower.contains("fastq-dump") {
                Some("fastq-dump".to_string())
            } else {
                None
            }
        }
        "cnvkit" => {
            if task_lower.contains("batch") {
                Some("batch".to_string())
            } else if task_lower.contains("target") {
                Some("target".to_string())
            } else if task_lower.contains("antitarget") {
                Some("antitarget".to_string())
            } else if task_lower.contains("coverage") {
                Some("coverage".to_string())
            } else if task_lower.contains("reference") {
                Some("reference".to_string())
            } else if task_lower.contains("fix") {
                Some("fix".to_string())
            } else if task_lower.contains("segment") {
                Some("segment".to_string())
            } else if task_lower.contains("call") {
                Some("call".to_string())
            } else if task_lower.contains("diagram") {
                Some("diagram".to_string())
            } else if task_lower.contains("scatter") {
                Some("scatter".to_string())
            } else {
                None
            }
        }
        "delly" => {
            if task_lower.contains("call") {
                Some("call".to_string())
            } else if task_lower.contains("filter") {
                Some("filter".to_string())
            } else if task_lower.contains("merge") {
                Some("merge".to_string())
            } else {
                None
            }
        }
        "whatshap" => {
            if task_lower.contains("phase") || task_lower.contains("phasing") {
                Some("phase".to_string())
            } else if task_lower.contains("haplotag") {
                Some("haplotag".to_string())
            } else if task_lower.contains("stats") {
                Some("stats".to_string())
            } else if task_lower.contains("compare") {
                Some("compare".to_string())
            } else {
                None
            }
        }
        "snpeff" => {
            if task_lower.contains("ann") || task_lower.contains("annotate") || task_lower.contains("effect") {
                Some("ann".to_string())
            } else if task_lower.contains("download") {
                Some("download".to_string())
            } else if task_lower.contains("build") {
                Some("build".to_string())
            } else {
                None
            }
        }
        "strelka2" => {
            if task_lower.contains("somatic") || task_lower.contains("tumor") {
                Some("configureStrelkaSomaticWorkflow.py".to_string())
            } else {
                Some("configureStrelkaGermlineWorkflow.py".to_string())
            }
        }
        "varscan2" => {
            if task_lower.contains("snp") && !task_lower.contains("indel") {
                Some("mpileup2snp".to_string())
            } else if task_lower.contains("indel") {
                Some("mpileup2indel".to_string())
            } else if task_lower.contains("somatic") {
                Some("somatic".to_string())
            } else if task_lower.contains("copynumber") || task_lower.contains("cnv") {
                Some("copynumber".to_string())
            } else if task_lower.contains("consensus") || task_lower.contains("cns") {
                Some("mpileup2cns".to_string())
            } else {
                None
            }
        }
        "mummer" => {
            if task_lower.contains("nucmer") || task_lower.contains("nucleotide") {
                Some("nucmer".to_string())
            } else if task_lower.contains("promer") || task_lower.contains("protein") {
                Some("promer".to_string())
            } else if task_lower.contains("delta-filter") || task_lower.contains("filter") {
                Some("delta-filter".to_string())
            } else if task_lower.contains("show-coords") || task_lower.contains("coords") {
                Some("show-coords".to_string())
            } else if task_lower.contains("show-snps") || task_lower.contains("snp") {
                Some("show-snps".to_string())
            } else if task_lower.contains("dnadiff") {
                Some("dnadiff".to_string())
            } else {
                None
            }
        }
        "homer" => {
            if task_lower.contains("findpeak") || task_lower.contains("peak") && !task_lower.contains("annotate") {
                Some("findPeaks".to_string())
            } else if task_lower.contains("findmotif") || task_lower.contains("motif") {
                Some("findMotifsGenome.pl".to_string())
            } else if task_lower.contains("annotate") && task_lower.contains("peak") {
                Some("annotatePeaks.pl".to_string())
            } else if task_lower.contains("makeTagDirectory") || task_lower.contains("tag directory") {
                Some("makeTagDirectory".to_string())
            } else {
                None
            }
        }
        "deeptools" => {
            if task_lower.contains("bamcoverage") || task_lower.contains("coverage") && task_lower.contains("bigwig") {
                Some("bamCoverage".to_string())
            } else if task_lower.contains("computematrix") || task_lower.contains("matrix") {
                Some("computeMatrix".to_string())
            } else if task_lower.contains("plotheatmap") || task_lower.contains("heatmap") {
                Some("plotHeatmap".to_string())
            } else if task_lower.contains("plotprofile") || task_lower.contains("profile") {
                Some("plotProfile".to_string())
            } else if task_lower.contains("plotfingerprint") || task_lower.contains("fingerprint") {
                Some("plotFingerprint".to_string())
            } else if task_lower.contains("multibamsummary") || task_lower.contains("correlation") {
                Some("multiBamSummary".to_string())
            } else if task_lower.contains("bamcompare") || task_lower.contains("compare") {
                Some("bamCompare".to_string())
            } else {
                None
            }
        }
        "macs2" => {
            if task_lower.contains("callpeak") || task_lower.contains("peak") {
                Some("callpeak".to_string())
            } else {
                None
            }
        }
        "seqkit" => {
            if task_lower.contains("stats") || task_lower.contains("statistics") {
                Some("stats".to_string())
            } else if task_lower.contains("seq") || task_lower.contains("convert") {
                Some("seq".to_string())
            } else if task_lower.contains("grep") || task_lower.contains("search") || task_lower.contains("filter") {
                Some("grep".to_string())
            } else if task_lower.contains("rmdup") || task_lower.contains("deduplicate") {
                Some("rmdup".to_string())
            } else if task_lower.contains("sample") || task_lower.contains("subsample") {
                Some("sample".to_string())
            } else if task_lower.contains("fx2tab") || task_lower.contains("table") {
                Some("fx2tab".to_string())
            } else if task_lower.contains("replace") || task_lower.contains("rename") {
                Some("replace".to_string())
            } else if task_lower.contains("sort") {
                Some("sort".to_string())
            } else if task_lower.contains("concat") {
                Some("concat".to_string())
            } else if task_lower.contains("split") {
                Some("split2".to_string())
            } else if task_lower.contains("common") {
                Some("common".to_string())
            } else {
                None
            }
        }
        "seqtk" => {
            if task_lower.contains("sample") || task_lower.contains("subsample") {
                Some("sample".to_string())
            } else if task_lower.contains("seq") || task_lower.contains("convert") {
                Some("seq".to_string())
            } else if task_lower.contains("subseq") {
                Some("subseq".to_string())
            } else if task_lower.contains("trimfq") || task_lower.contains("trim") {
                Some("trimfq".to_string())
            } else {
                None
            }
        }
        "pairtools" => {
            if task_lower.contains("parse") {
                Some("parse".to_string())
            } else if task_lower.contains("sort") {
                Some("sort".to_string())
            } else if task_lower.contains("merge") {
                Some("merge".to_string())
            } else if task_lower.contains("dedup") {
                Some("dedup".to_string())
            } else if task_lower.contains("select") {
                Some("select".to_string())
            } else if task_lower.contains("split") {
                Some("split".to_string())
            } else if task_lower.contains("stats") {
                Some("stats".to_string())
            } else {
                None
            }
        }
        "modkit" => {
            if task_lower.contains("pileup") {
                Some("pileup".to_string())
            } else if task_lower.contains("summary") {
                Some("summary".to_string())
            } else if task_lower.contains("extract") {
                Some("extract".to_string())
            } else if task_lower.contains("call-mods") {
                Some("call-mods".to_string())
            } else {
                None
            }
        }
        "mmseqs2" => {
            if task_lower.contains("easy-search") || task_lower.contains("search") {
                Some("easy-search".to_string())
            } else if task_lower.contains("easy-cluster") || task_lower.contains("cluster") {
                Some("easy-cluster".to_string())
            } else if task_lower.contains("createdb") || task_lower.contains("database") {
                Some("createdb".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else {
                None
            }
        }
        "bamtools" => {
            if task_lower.contains("convert") {
                Some("convert".to_string())
            } else if task_lower.contains("sort") {
                Some("sort".to_string())
            } else if task_lower.contains("merge") {
                Some("merge".to_string())
            } else if task_lower.contains("stats") || task_lower.contains("statistics") {
                Some("stats".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else if task_lower.contains("split") {
                Some("split".to_string())
            } else if task_lower.contains("coverage") {
                Some("coverage".to_string())
            } else if task_lower.contains("filter") {
                Some("filter".to_string())
            } else {
                None
            }
        }
        "meme" => {
            if task_lower.contains("fimo") || (task_lower.contains("scan") && task_lower.contains("motif")) || (task_lower.contains("known") && task_lower.contains("tf")) {
                Some("fimo".to_string())
            } else if task_lower.contains("tomtom") || (task_lower.contains("compare") && task_lower.contains("motif")) {
                Some("tomtom".to_string())
            } else if task_lower.contains("ame") || (task_lower.contains("enrichment") && task_lower.contains("motif")) {
                Some("ame".to_string())
            } else if task_lower.contains("streme") || (task_lower.contains("short") && task_lower.contains("motif")) {
                Some("streme".to_string())
            } else if task_lower.contains("de novo") || task_lower.contains("denovo") || task_lower.contains("discover") && task_lower.contains("motif") || task_lower.contains("chip-seq") {
                Some("meme".to_string())
            } else if task_lower.contains("revcomp") || task_lower.contains("reverse complement") {
                Some("meme".to_string())
            } else {
                None
            }
        }
        "blast" => {
            if task_lower.contains("makeblastdb") || (task_lower.contains("build") && task_lower.contains("database")) || (task_lower.contains("create") && task_lower.contains("database")) {
                Some("makeblastdb".to_string())
            } else if task_lower.contains("blastdbcmd") || (task_lower.contains("retrieve") && task_lower.contains("sequence")) {
                Some("blastdbcmd".to_string())
            } else if task_lower.contains("blastp") || (task_lower.contains("protein") && task_lower.contains("protein") && task_lower.contains("search")) {
                Some("blastp".to_string())
            } else if task_lower.contains("blastx") || (task_lower.contains("nucleotide") && task_lower.contains("protein")) || (task_lower.contains("translate") && task_lower.contains("search")) {
                Some("blastx".to_string())
            } else if task_lower.contains("tblastn") || (task_lower.contains("protein") && task_lower.contains("nucleotide") && task_lower.contains("search")) {
                Some("tblastn".to_string())
            } else if task_lower.contains("blastn-short") || (task_lower.contains("short") && task_lower.contains("sequence")) {
                Some("blastn-short".to_string())
            } else if task_lower.contains("blastn") || task_lower.contains("nucleotide") && task_lower.contains("search") || task_lower.contains("similar") && task_lower.contains("sequence") {
                Some("blastn".to_string())
            } else if task_lower.contains("remote") && task_lower.contains("blast") {
                Some("blastn".to_string())
            } else if task_lower.contains("taxonomy") || task_lower.contains("taxid") {
                Some("blastn".to_string())
            } else {
                None
            }
        }
        "angsd" => {
            None
        }
        "plink2" => {
            None
        }
        "stringtie" => {
            Some("_NO_SUB_".to_string())
        }
        "rsem" => {
            if task_lower.contains("calculate-expression") || task_lower.contains("quantify") || task_lower.contains("expression") {
                Some("rsem-calculate-expression".to_string())
            } else if task_lower.contains("prepare-reference") || task_lower.contains("index") {
                Some("rsem-prepare-reference".to_string())
            } else {
                None
            }
        }
        "igvtools" => {
            if task_lower.contains("totdf") || task_lower.contains("tdf") || task_lower.contains("count") {
                Some("toTDF".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else {
                None
            }
        }
        "qualimap" => {
            if task_lower.contains("bamqc") || task_lower.contains("bam qc") {
                Some("bamqc".to_string())
            } else if task_lower.contains("rnaseq") || task_lower.contains("rna-seq") {
                Some("rnaseq".to_string())
            } else {
                None
            }
        }
        "gtdbtk" => {
            if task_lower.contains("classify") {
                Some("classify_wf".to_string())
            } else if task_lower.contains("infer") {
                Some("infer".to_string())
            } else if task_lower.contains("de_novo") || task_lower.contains("denovo") {
                Some("de_novo_wf".to_string())
            } else {
                None
            }
        }
        "checkm2" => {
            if task_lower.contains("predict") {
                Some("predict".to_string())
            } else if task_lower.contains("plot") {
                Some("plot".to_string())
            } else {
                None
            }
        }
        "arriba" => {
            if task_lower.contains("draw") || task_lower.contains("visualize") {
                Some("draw_fusions.R".to_string())
            } else if task_lower.contains("convert") && task_lower.contains("vcf") {
                Some("convert_fusions_to_vcf".to_string())
            } else if task_lower.contains("wrapper") || task_lower.contains("prealigned") {
                Some("run_arriba_on_prealigned_bam".to_string())
            } else if task_lower.contains("pipeline") || task_lower.contains("full") {
                Some("run_arriba".to_string())
            } else {
                None
            }
        }
        "pbsv" => {
            if task_lower.contains("discover") {
                Some("discover".to_string())
            } else if task_lower.contains("call") {
                Some("call".to_string())
            } else {
                None
            }
        }
        "survivor" => {
            if task_lower.contains("merge") {
                Some("merge".to_string())
            } else if task_lower.contains("simsv") || task_lower.contains("simulate") {
                Some("simSV".to_string())
            } else if task_lower.contains("stats") {
                Some("stats".to_string())
            } else {
                None
            }
        }
        "sourmash" => {
            if task_lower.contains("sketch") || task_lower.contains("compute") || task_lower.contains("signature") {
                if task_lower.contains("dna") || task_lower.contains("genome") || task_lower.contains("nucleotide") {
                    Some("sketch dna".to_string())
                } else if task_lower.contains("protein") || task_lower.contains("translate") {
                    Some("sketch protein".to_string())
                } else {
                    Some("sketch dna".to_string())
                }
            } else if task_lower.contains("compare") {
                Some("compare".to_string())
            } else if task_lower.contains("gather") {
                Some("gather".to_string())
            } else if task_lower.contains("search") && !task_lower.contains("gather") {
                Some("search".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else if task_lower.contains("tax") || task_lower.contains("taxonomy") || task_lower.contains("classify") || task_lower.contains("annotate") {
                Some("taxonomy annotate".to_string())
            } else {
                None
            }
        }
        "bedops" => {
            if task_lower.contains("convert") {
                Some("convert2bed".to_string())
            } else if task_lower.contains("intersect") {
                Some("--intersect".to_string())
            } else if task_lower.contains("difference") || task_lower.contains("complement") {
                Some("--difference".to_string())
            } else if task_lower.contains("merge") && !task_lower.contains("map") {
                Some("--merge".to_string())
            } else if task_lower.contains("element-of") || task_lower.contains("subset") {
                Some("--element-of".to_string())
            } else if task_lower.contains("chop") || task_lower.contains("partition") {
                Some("--chop".to_string())
            } else if task_lower.contains("starch") {
                Some("starch".to_string())
            } else if task_lower.contains("sort") {
                Some("sort-bed".to_string())
            } else if task_lower.contains("map") {
                Some("bedmap".to_string())
            } else if task_lower.contains("extract") {
                Some("bedextract".to_string())
            } else {
                None
            }
        }
        "sambamba" => {
            if task_lower.contains("sort") {
                Some("sort".to_string())
            } else if task_lower.contains("view") {
                Some("view".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else if task_lower.contains("merge") {
                Some("merge".to_string())
            } else if task_lower.contains("markdup") || task_lower.contains("duplicate") {
                Some("markdup".to_string())
            } else if task_lower.contains("slice") {
                Some("slice".to_string())
            } else if task_lower.contains("flagstat") {
                Some("flagstat".to_string())
            } else if task_lower.contains("depth") {
                Some("depth".to_string())
            } else {
                None
            }
        }
        "vcftools" => {
            None
        }
        "pbmm2" => {
            if task_lower.contains("align") || task_lower.contains("mapping") || task_lower.contains("map") {
                Some("align".to_string())
            } else if task_lower.contains("index") {
                Some("index".to_string())
            } else if task_lower.contains("sort") {
                Some("sort".to_string())
            } else {
                None
            }
        }
        "truvari" => {
            if task_lower.contains("bench") || task_lower.contains("compare") {
                Some("bench".to_string())
            } else if task_lower.contains("collapse") {
                Some("collapse".to_string())
            } else if task_lower.contains("normalize") {
                Some("normalize".to_string())
            } else if task_lower.contains("anno") {
                Some("anno".to_string())
            } else {
                None
            }
        }
        "kb" => {
            if task_lower.contains("ref") || task_lower.contains("reference") || task_lower.contains("index") {
                Some("ref".to_string())
            } else if task_lower.contains("count") || task_lower.contains("quantify") {
                Some("count".to_string())
            } else {
                None
            }
        }
        "methyldackel" => {
            if task_lower.contains("extract") || task_lower.contains("methylation") {
                Some("extract".to_string())
            } else if task_lower.contains("mbias") {
                Some("mbias".to_string())
            } else {
                None
            }
        }
        "bwa" => {
            if task_lower.contains("index") || task_lower.contains("build") {
                Some("index".to_string())
            } else if task_lower.contains("mem") || task_lower.contains("align") || task_lower.contains("mapping") || task_lower.contains("map") {
                Some("mem".to_string())
            } else {
                Some("mem".to_string())
            }
        }
        "bwa-mem2" => {
            if task_lower.contains("index") || task_lower.contains("build") {
                Some("index".to_string())
            } else {
                Some("mem".to_string())
            }
        }
        "salmon" => {
            if task_lower.contains("index") || task_lower.contains("build") {
                Some("index".to_string())
            } else if task_lower.contains("quant") || task_lower.contains("quantify") || task_lower.contains("expression") {
                Some("quant".to_string())
            } else {
                Some("quant".to_string())
            }
        }
        "kallisto" => {
            if task_lower.contains("index") || task_lower.contains("build") {
                Some("index".to_string())
            } else if task_lower.contains("quant") || task_lower.contains("quantify") || task_lower.contains("expression") {
                Some("quant".to_string())
            } else if task_lower.contains("bus") || task_lower.contains("pseudoalignment") {
                Some("bus".to_string())
            } else {
                Some("quant".to_string())
            }
        }
        "mash" => {
            if task_lower.contains("sketch") || task_lower.contains("compute") {
                Some("sketch".to_string())
            } else if task_lower.contains("dist") || task_lower.contains("distance") {
                Some("dist".to_string())
            } else if task_lower.contains("screen") || task_lower.contains("containment") {
                Some("screen".to_string())
            } else if task_lower.contains("info") {
                Some("info".to_string())
            } else {
                Some("sketch".to_string())
            }
        }
        "trimmomatic" => {
            if task_lower.contains("paired") || task_lower.contains("pe") {
                Some("PE".to_string())
            } else if task_lower.contains("single") || task_lower.contains("se") {
                Some("SE".to_string())
            } else {
                Some("PE".to_string())
            }
        }
        "nanocomp" => {
            Some("NanoComp".to_string())
        }
        "nanoplot" => {
            Some("_NO_SUB_".to_string())
        }
        "nanostat" => {
            Some("_NO_SUB_".to_string())
        }
        "snakemake" => {
            Some("_NO_SUB_".to_string())
        }
        "hmmer" => {
            if task_lower.contains("hmmscan") || task_lower.contains("scan") && task_lower.contains("profile") {
                Some("hmmscan".to_string())
            } else if task_lower.contains("hmmsearch") || task_lower.contains("search") && task_lower.contains("sequence") {
                Some("hmmsearch".to_string())
            } else if task_lower.contains("hmmbuild") || task_lower.contains("build") && task_lower.contains("profile") {
                Some("hmmbuild".to_string())
            } else if task_lower.contains("hmmalign") || task_lower.contains("align") {
                Some("hmmalign".to_string())
            } else if task_lower.contains("phmmer") || task_lower.contains("protein") && task_lower.contains("search") {
                Some("phmmer".to_string())
            } else if task_lower.contains("jackhmmer") || task_lower.contains("iterative") {
                Some("jackhmmer".to_string())
            } else if task_lower.contains("nhmmer") || task_lower.contains("dna") && task_lower.contains("search") {
                Some("nhmmer".to_string())
            } else if task_lower.contains("nhmmscan") || task_lower.contains("dna") && task_lower.contains("scan") {
                Some("nhmmscan".to_string())
            } else {
                None
            }
        }
        "r" => {
            Some("Rscript".to_string())
        }
        "perl" => {
            if task_lower.contains("version") || task_lower.contains("-v") {
                Some("-V".to_string())
            } else if task_lower.contains("one-liner") || task_lower.contains("-e") || task_lower.contains("-ne") || task_lower.contains("-pe") {
                None
            } else if task_lower.contains("module") || task_lower.contains("install") || task_lower.contains("cpan") {
                None
            } else {
                None
            }
        }
        "python" => {
            if task_lower.contains("version") {
                Some("--version".to_string())
            } else if task_lower.contains("module") || task_lower.contains("-m") {
                None
            } else if task_lower.contains("venv") || task_lower.contains("virtual") {
                None
            } else if task_lower.contains("one-liner") || task_lower.contains("-c") {
                None
            } else {
                None
            }
        }
        "bash" => {
            if task_lower.contains("version") {
                Some("--version".to_string())
            } else if task_lower.contains("strict") || task_lower.contains("pipefail") || task_lower.contains("-c") {
                None
            } else if task_lower.contains("debug") || task_lower.contains("trace") || task_lower.contains("-x") {
                None
            } else {
                None
            }
        }
        "java" => {
            if task_lower.contains("version") {
                Some("-version".to_string())
            } else if task_lower.contains("jar") || task_lower.contains("-jar") {
                None
            } else if task_lower.contains("gatk") {
                None
            } else {
                None
            }
        }
        "julia" => {
            if task_lower.contains("version") {
                None
            } else if task_lower.contains("project") || task_lower.contains("environment") {
                None
            } else if task_lower.contains("threads") {
                None
            } else if task_lower.contains("expression") || task_lower.contains("-e") {
                None
            } else {
                None
            }
        }
        "awk" | "sed" | "grep" => {
            None
        }
        "bowtie2" => {
            if task_lower.contains("build") || task_lower.contains("index") {
                Some("build".to_string())
            } else {
                None
            }
        }
        "hisat2" => {
            if task_lower.contains("build") || task_lower.contains("index") {
                Some("build".to_string())
            } else {
                None
            }
        }
        "ssh" | "wget" | "curl" | "rsync" | "find" | "rm" | "tar" => {
            None
        }
        "bedops" => {
            if task_lower.contains("bedmap") || task_lower.contains("map") {
                Some("bedmap".to_string())
            } else if task_lower.contains("bedextract") || task_lower.contains("extract") {
                Some("bedextract".to_string())
            } else if task_lower.contains("sort-bed") || task_lower.contains("sort") {
                Some("sort-bed".to_string())
            } else if task_lower.contains("starch") || task_lower.contains("compress") {
                Some("starch".to_string())
            } else if task_lower.contains("convert2bed") || task_lower.contains("convert") {
                Some("convert2bed".to_string())
            } else {
                None
            }
        }
        _ => None
    }
}

pub fn is_no_subcommand_tool(tool: &str) -> bool {
    let tool_lower = tool.to_ascii_lowercase();
    matches!(tool_lower.as_str(),
        "porechop" | "chopper" | "trim_galore" | "fastp" | "fastqc" | "multiqc" |
        "mosdepth" | "megahit" | "hifiasm" | "racon" | "freebayes" | "longshot" |
        "sniffles" | "liftoff" | "prodigal" | "tabix" | "vcfanno" |
        "fastani" | "orthofinder" | "iqtree2" | "mafft" | "fasttree" |
        "admixture" | "shapeit4" | "pbccs" | "nanoplot" | "nanostat" |
        "metabat2" | "metaphlan" | "repeatmasker" |
        "pilon" | "busco" | "cutadapt" |
        "mash" | "eggnog-mapper" |
        "miniasm" | "wtdbg2" | "verkko" |
        "bwa" | "bwa-mem2" |
        "minimap2" | "salmon" | "kallisto" |
        "featurecounts" | "spades" | "flye" | "canu" |
        "trinity" | "prokka" |
        "fastq-screen" | "bbtools" |
        "cellsnp-lite" | "pbfusion" |
        "trimmomatic" |
        "star" | "stringtie" | "snakemake" |
        "hisat2" | "bowtie2" | "kraken2" |
        "bakta" |
        "awk" | "sed" | "grep" | "perl" | "python" | "bash" | "java" | "julia" |
        "ssh" | "wget" | "curl" | "rsync" | "find" | "rm" | "tar" |
        "r"
    )
}

pub fn get_known_subcommands_for_tool(tool: &str) -> Vec<String> {
    let tool_lower = tool.to_ascii_lowercase();
    match tool_lower.as_str() {
        "samtools" => vec!["view", "sort", "index", "flagstat", "idxstats", "depth", "merge", "faidx", "stats", "fastq", "calmd", "fixmate", "reheader", "rmdup", "collate", "bam2fq", "markdup", "cat", "dict", "mpileup"].iter().map(|s| s.to_string()).collect(),
        "bcftools" => vec!["view", "filter", "merge", "call", "annotate", "concat", "norm", "sort", "index", "stats", "query", "isec", "mpileup", "consensus", "convert", "plugin", "gtcheck", "roh"].iter().map(|s| s.to_string()).collect(),
        "bedtools" => vec!["intersect", "merge", "sort", "genomecov", "coverage", "getfasta", "slop", "flank", "closest", "subtract", "window", "cluster", "complement", "shift", "map", "groupby", "split", "bamtobed", "bedtobam", "unionbedg", "multiinter", "random", "sample", "jaccard", "reldist", "makewindows", "bamtofastq"].iter().map(|s| s.to_string()).collect(),
        "gatk" => vec!["HaplotypeCaller", "Mutect2", "BaseRecalibrator", "ApplyBQSR", "MarkDuplicates", "SplitNCigarReads", "VariantFiltration", "SelectVariants", "CombineVariants", "GenotypeGVCFs", "GenomicsDBImport", "GatherVcfs", "GatherBqsrReports", "IndexFeatureFile", "PrintReads", "ValidateVariants", "ValidateSamFile", "CalculateGenotypePosteriors", "PhaseByTransmission", "ASEReadCounter", "CollectAlignmentSummaryMetrics", "CollectInsertSizeMetrics", "CollectQualityYieldMetrics", "SortSam", "AddOrReplaceReadGroups", "CreateSequenceDictionary", "DepthOfCoverage", "BuildBamIndex", "MergeSamFiles", "ExtractSequences", "FilterMutectCalls", "CombineGVCFs"].iter().map(|s| s.to_string()).collect(),
        "picard" => vec!["MarkDuplicates", "SortSam", "AddOrReplaceReadGroups", "CreateSequenceDictionary", "CollectAlignmentSummaryMetrics", "CollectInsertSizeMetrics", "CollectGcBiasMetrics", "CollectQualityYieldMetrics", "CollectRnaSeqMetrics", "MergeSamFiles", "ValidateSamFile", "SamFormatConverter", "FastqToSam", "SamToFastq", "BuildBamIndex", "CreateSequenceDictionary", "ExtractSequences", "GatherVcfs"].iter().map(|s| s.to_string()).collect(),
        "deeptools" => vec!["bamCoverage", "computeMatrix", "plotHeatmap", "plotProfile", "plotFingerprint", "bamCompare", "multiBamSummary", "plotCoverage", "computeGCBias", "correctGCBias", "alignmentSieve", "plotCorrelation", "plotPCA", "estimateReadFiltering", "estimateScaleFactor", "bamPEFragmentSize", "computeMatrixOperations", "plotEnrichment"].iter().map(|s| s.to_string()).collect(),
        "snakemake" => vec!["run", "dryrun", "dag", "report", "archive", "cleanup", "cleanup_metadata", "conda"].iter().map(|s| s.to_string()).collect(),
        "fastqc" => vec![],
        "multiqc" => vec![],
        "bwa" => vec!["mem", "index", "aln", "sampe", "samse", "bwasw"].iter().map(|s| s.to_string()).collect(),
        "bwa-mem2" => vec!["mem", "index"].iter().map(|s| s.to_string()).collect(),
        "bowtie2" => vec!["build", "inspect", "align", "sam"].iter().map(|s| s.to_string()).collect(),
        "hisat2" => vec!["build", "inspect", "align", "extract-splice-sites", "extract-exons"].iter().map(|s| s.to_string()).collect(),
        "minimap2" => vec![],
        "star" => vec![],
        "salmon" => vec!["index", "quant", "alevin", "swim"].iter().map(|s| s.to_string()).collect(),
        "kallisto" => vec!["index", "quant", "bus", "merge", "h5dump", "inspect", "version"].iter().map(|s| s.to_string()).collect(),
        "featurecounts" => vec![],
        "htseq" => vec!["count", "qa"].iter().map(|s| s.to_string()).collect(),
        "vcftools" => vec![],
        "plink" => vec!["--vcf", "--bfile", "--make-bed", "--assoc", "--linear", "--logistic", "--freq", "--hardy", "--hwe", "--ld", "--recode", "--mind", "--geno", "--maf"].iter().map(|s| s.to_string()).collect(),
        "plink2" => vec!["--pfile", "--bfile", "--vcf", "--make-pgen", "--make-bed", "--assoc", "--linear", "--logistic", "--freq", "--hardy", "--hwe", "--ld", "--recode", "--mind", "--geno", "--maf", "--pca"].iter().map(|s| s.to_string()).collect(),
        "agat" => vec!["sp_sanity_check", "sp_statistics", "sp_filter_feature_from_fasta", "sp_fix_features_positions", "sp_merge_annotations", "sp_add_start_and_stop", "sp_extract_sequences", "sp_to_tab", "sp_gxf_to_gff3", "sp_gff2tsv", "sp_list_attributes", "sp_compare_two_BUSCOs", "sp_fix_fasta", "convert_sp_gff2gtf", "convert_sp_gff2bed", "convert_sp_gxf2gxf", "sp_filter_gene_by_length", "sp_keep_longest_isoform", "sp_manage_IDs"].iter().map(|s| s.to_string()).collect(),
        "diamond" => vec!["blastp", "blastx", "makedb", "view", "getseq", "cluster", "realign", "merge-daa", "seed-index"].iter().map(|s| s.to_string()).collect(),
        "hmmer" => vec!["hmmsearch", "hmmscan", "hmmbuild", "hmmalign", "hmmpress", "hmmemit", "hmmsim"].iter().map(|s| s.to_string()).collect(),
        "blast" => vec!["blastn", "blastp", "blastx", "tblastn", "tblastx", "makeblastdb"].iter().map(|s| s.to_string()).collect(),
        "kraken2" => vec!["classify", "build", "inspect", "translate", "report", "kmer2taxo"].iter().map(|s| s.to_string()).collect(),
        "mmseqs2" => vec!["easy-search", "easy-cluster", "search", "cluster", "createdb", "index", "convert2fasta", "createtsv", "result2repseq", "result2profile", "mergeclusters"].iter().map(|s| s.to_string()).collect(),
        "seqkit" => vec!["stats", "seq", "grep", "rmdup", "sample", "fx2tab", "replace", "sort", "concat", "split2", "common", "subseq", "translate", "head", "range", "bam", "fq2fa"].iter().map(|s| s.to_string()).collect(),
        "pairtools" => vec!["parse", "sort", "merge", "dedup", "select", "split", "stats", "flip", "restrict", "scale"].iter().map(|s| s.to_string()).collect(),
        "modkit" => vec!["pileup", "summary", "extract", "call-mods", "motif-bed", "sample-probs", "update-tags"].iter().map(|s| s.to_string()).collect(),
        "bismark" => vec!["bismark", "bismark_genome_preparation", "bismark_methylation_extractor", "deduplicate_bismark", "bismark2report", "bismark2summary", "coverage2cytosine"].iter().map(|s| s.to_string()).collect(),
        "cnvkit" => vec!["batch", "target", "antitarget", "coverage", "reference", "fix", "segment", "call", "diagram", "scatter", "heatmap", "breaks", "gainloss", "sex", "metrics", "segmetrics"].iter().map(|s| s.to_string()).collect(),
        "mummer" => vec!["nucmer", "promer", "delta-filter", "show-coords", "show-snps", "show-tiling", "dnadiff", "run-mummer1", "run-mummer3"].iter().map(|s| s.to_string()).collect(),
        "homer" => vec!["findPeaks", "findMotifsGenome.pl", "annotatePeaks.pl", "makeTagDirectory", "makeUCSCfile", "pos2bed.pl", "removeDupReads.pl", "makeMultiWigHub.pl"].iter().map(|s| s.to_string()).collect(),
        "arriba" => vec!["run_arriba", "run_arriba_on_prealigned_bam", "draw_fusions.R", "convert_fusions_to_vcf"].iter().map(|s| s.to_string()).collect(),
        "sra-tools" => vec!["prefetch", "fasterq-dump", "fastq-dump", "sam-dump", "sra-pileup", "vdb-config", "vdb-decrypt"].iter().map(|s| s.to_string()).collect(),
        "meme" => vec!["meme", "fimo", "dreme", "ame", "mast", "mcast", "glam2", "glam2scan", "tomtom", "spamo"].iter().map(|s| s.to_string()).collect(),
        "angsd" => vec!["-doSaf", "-doMaf", "-doGeno", "-doThetas", "-doAbbababa", "-doAsso", "-doFasta", "-doCounts"].iter().map(|s| s.to_string()).collect(),
        "varscan2" => vec!["mpileup2snp", "mpileup2indel", "somatic", "copynumber", "readcounts", "mpileup2cns"].iter().map(|s| s.to_string()).collect(),
        "delly" => vec!["call", "filter", "merge", "lr", "genotype"].iter().map(|s| s.to_string()).collect(),
        "whatshap" => vec!["phase", "haplotag", "stats", "compare", "polyphase"].iter().map(|s| s.to_string()).collect(),
        "snpeff" => vec!["ann", "download", "build", "databases", "dump", "count"].iter().map(|s| s.to_string()).collect(),
        "vep" => vec![],
        "stringtie" => vec!["merge", "assemble", "estimate"].iter().map(|s| s.to_string()).collect(),
        "rsem" => vec!["rsem-calculate-expression", "rsem-prepare-reference", "rsem-plot-model", "rsem-simulate-reads"].iter().map(|s| s.to_string()).collect(),
        "trinity" => vec![],
        "spades" => vec![],
        "megahit" => vec![],
        "flye" => vec![],
        "canu" => vec![],
        "hifiasm" => vec![],
        "prokka" => vec![],
        "bakta" => vec!["download", "annotate"].iter().map(|s| s.to_string()).collect(),
        "checkm2" => vec!["predict", "plot"].iter().map(|s| s.to_string()).collect(),
        "gtdbtk" => vec!["classify_wf", "infer", "de_novo_wf", "align", "trim_msa", "assign_taxonomy"].iter().map(|s| s.to_string()).collect(),
        "sourmash" => vec!["compute", "compare", "gather", "search", "index", "categorize", "lca", "tax", "migrate"].iter().map(|s| s.to_string()).collect(),
        "mash" => vec!["sketch", "dist", "screen"].iter().map(|s| s.to_string()).collect(),
        "fastani" => vec![],
        "orthofinder" => vec![],
        "iqtree2" => vec![],
        "mafft" => vec![],
        "muscle" => vec!["-align", "-cluster", "-refine"].iter().map(|s| s.to_string()).collect(),
        "fasttree" => vec![],
        "admixture" => vec![],
        "shapeit4" => vec![],
        "pbmm2" => vec!["align", "index", "sort"].iter().map(|s| s.to_string()).collect(),
        "pbccs" => vec![],
        "porechop" => vec![],
        "chopper" => vec![],
        "nanocomp" => vec!["NanoComp"].iter().map(|s| s.to_string()).collect(),
        "nanoplot" => vec![],
        "nanostat" => vec![],
        "trimmomatic" => vec!["PE", "SE"].iter().map(|s| s.to_string()).collect(),
        "trim_galore" => vec![],
        "cutadapt" => vec![],
        "fastp" => vec![],
        "mosdepth" => vec![],
        "qualimap" => vec!["bamqc", "rnaseq", "counts"].iter().map(|s| s.to_string()).collect(),
        "seqtk" => vec!["sample", "seq", "subseq", "trimfq", "fqchk", "comp", "mergefa", "randbase", "cutN", "listhet"].iter().map(|s| s.to_string()).collect(),
        "bamtools" => vec!["convert", "sort", "merge", "stats", "index", "split", "coverage", "filter", "random", "header", "count", "resolve"].iter().map(|s| s.to_string()).collect(),
        "tabix" => vec![],
        "vcfanno" => vec![],
        "bedops" => vec!["convert2bed", "bedintersect", "starch", "unstarch", "sort-bed", "bedmap"].iter().map(|s| s.to_string()).collect(),
        "sambamba" => vec!["sort", "view", "index", "merge", "markdup", "slice", "flagstat", "depth", "subsamp", "validate"].iter().map(|s| s.to_string()).collect(),
        "igvtools" => vec!["toTDF", "index", "count", "tile"].iter().map(|s| s.to_string()).collect(),
        "metabat2" => vec![],
        "centrifuge" => vec![],
        "metaphlan" => vec![],
        "liftoff" => vec![],
        "repeatmasker" => vec![],
        "augustus" => vec![],
        "prodigal" => vec![],
        "eggnog-mapper" => vec!["annotate", "download", "diamonddb", "mapper", "join"].iter().map(|s| s.to_string()).collect(),
        "medaka" => vec!["medaka_consensus", "medaka_variant", "medaka_haploid_variant", "medaka_variant_phase"].iter().map(|s| s.to_string()).collect(),
        "pilon" => vec![],
        "quast" => vec![],
        "busco" => vec![],
        "freebayes" => vec![],
        "longshot" => vec![],
        "strelka2" => vec!["configureStrelkaGermlineWorkflow.py", "configureStrelkaSomaticWorkflow.py"].iter().map(|s| s.to_string()).collect(),
        "sniffles" => vec![],
        "pbsv" => vec!["discover", "call"].iter().map(|s| s.to_string()).collect(),
        "survivor" => vec!["merge", "simSV", "stats"].iter().map(|s| s.to_string()).collect(),
        "cellsnp-lite" => vec![],
        "kb" => vec!["ref", "count", "matrix", "info", "test"].iter().map(|s| s.to_string()).collect(),
        "fastq-screen" => vec![],
        "bbtools" => vec!["reformat.sh", "bbmap.sh", "bbduk.sh", "bbmerge.sh", "bbsplit.sh", "tadpole.sh", "clumpify.sh", "dedupe.sh", "sendsketch.sh", "comparesketch.sh"].iter().map(|s| s.to_string()).collect(),
        "truvari" => vec!["bench", "compare", "collapse", "normalize", "anno"].iter().map(|s| s.to_string()).collect(),
        _ => vec![],
    }
}
