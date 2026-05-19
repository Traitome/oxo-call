use crate::doc_processor::StructuredDoc;
use super::task_values::TaskValues;

pub struct ToolTemplate {
    pub tool: &'static str,
    pub keywords: &'static [&'static str],
    pub template: &'static str,
}

pub const TOOL_TEMPLATES: &[ToolTemplate] = &[
    ToolTemplate { tool: "samtools", keywords: &["sort", "sorted", "sort by"], template: "sort -@ 4 -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["index", "indexing"], template: "index {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "convert", "extract", "filter", "subset"], template: "view -b -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["flagstat", "flag statistics"], template: "flagstat {input}" },
    ToolTemplate { tool: "samtools", keywords: &["merge", "combine"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "samtools", keywords: &["depth", "coverage"], template: "depth {input}" },
    ToolTemplate { tool: "samtools", keywords: &["idxstats"], template: "idxstats {input}" },
    ToolTemplate { tool: "samtools", keywords: &["stats", "statistics"], template: "stats {input}" },
    ToolTemplate { tool: "samtools", keywords: &["mpileup", "pileup"], template: "mpileup -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["faidx", "fasta index"], template: "faidx {input}" },
    ToolTemplate { tool: "samtools", keywords: &["dict"], template: "dict {input}" },
    ToolTemplate { tool: "samtools", keywords: &["bam2fq", "fastq"], template: "bam2fq {input}" },
    ToolTemplate { tool: "samtools", keywords: &["markdup", "duplicate"], template: "markdup {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["fixmate"], template: "fixmate {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["calmd"], template: "calmd {input} {reference}" },
    ToolTemplate { tool: "samtools", keywords: &["collate"], template: "collate -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["cat"], template: "cat -o {output} {inputs}" },
    ToolTemplate { tool: "samtools", keywords: &["reheader"], template: "reheader {input}" },
    ToolTemplate { tool: "bwa", keywords: &["mem", "align", "mapping", "map"], template: "mem -t 4 {reference} {input}" },
    ToolTemplate { tool: "bwa", keywords: &["index"], template: "index {reference}" },
    ToolTemplate { tool: "bwa", keywords: &["aln"], template: "aln -t 4 {reference} {input}" },
    ToolTemplate { tool: "bwa", keywords: &["sampe"], template: "sampe {reference} {sai1} {sai2} {read1} {read2}" },
    ToolTemplate { tool: "bwa", keywords: &["samse"], template: "samse {reference} {sai} {reads}" },
    ToolTemplate { tool: "bwa-mem2", keywords: &["mem", "align", "mapping", "map"], template: "mem -t 4 {reference} {input}" },
    ToolTemplate { tool: "bwa-mem2", keywords: &["index"], template: "index {reference}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "convert", "extract", "filter"], template: "view -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["call"], template: "call -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["filter"], template: "filter -i 'QUAL>30' -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["mpileup"], template: "mpileup -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["sort"], template: "sort -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["index"], template: "index {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["merge"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["concat"], template: "concat -o {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["annotate"], template: "annotate -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["norm"], template: "norm -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["query"], template: "query -f '%CHROM\\t%POS\\n' {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["stats"], template: "stats {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["isec"], template: "isec -p {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["consensus"], template: "consensus -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["intersect", "overlap"], template: "intersect -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["sort"], template: "sort -i {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["merge"], template: "merge -i {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["subtract"], template: "subtract -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["slop"], template: "slop -i {input} -g {genome} -b 1000" },
    ToolTemplate { tool: "bedtools", keywords: &["closest"], template: "closest -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["coverage", "depth"], template: "coverage -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["getfasta", "sequence"], template: "getfasta -fi {reference} -bed {input} -fo {output}" },
    ToolTemplate { tool: "bedtools", keywords: &["complement"], template: "complement -i {input} -g {genome}" },
    ToolTemplate { tool: "bedtools", keywords: &["window"], template: "window -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["flank"], template: "flank -i {input} -g {genome} -b 500" },
    ToolTemplate { tool: "bedtools", keywords: &["makewindows", "genome windows"], template: "makewindows -g {genome} -w 1000000" },
    ToolTemplate { tool: "bedtools", keywords: &["bamtofastq"], template: "bamtofastq -i {input} -fq {output}" },
    ToolTemplate { tool: "gatk", keywords: &["haplotype", "variant call", "snp"], template: "HaplotypeCaller -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["mutect2", "somatic"], template: "Mutect2 -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["markdup", "duplicate"], template: "MarkDuplicates -I {input} -O {output} -M {metrics}" },
    ToolTemplate { tool: "gatk", keywords: &["baserecalibrator", "bqsr", "recalibrat"], template: "BaseRecalibrator -R {reference} -I {input} --known-sites {known_sites} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["applybqsr"], template: "ApplyBQSR -R {reference} -I {input} --bqsr-recal-file {recal} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["selectvariants", "select variant"], template: "SelectVariants -R {reference} -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["variantfiltration", "filter variant"], template: "VariantFiltration -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["filtermutectcalls"], template: "FilterMutectCalls -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["createsequencedictionary", "sequence dictionary"], template: "CreateSequenceDictionary -R {reference}" },
    ToolTemplate { tool: "gatk", keywords: &["gathervcfs", "combine vcf"], template: "GatherVcfs -I {inputs} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["genomicsdbimport"], template: "GenomicsDBImport -V {input} --genomicsdb-workspace-path {output}" },
    ToolTemplate { tool: "gatk", keywords: &["genotypegvcfs"], template: "GenotypeGVCFs -R {reference} -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["combinegvcfs"], template: "CombineGVCFs -R {reference} -V {inputs} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["splitncigarreads"], template: "SplitNCigarReads -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["addreplacereadgroups", "read group"], template: "AddOrReplaceReadGroups -I {input} -O {output} -RGID 1 -RGLB lib1 -RGPL illumina -RGPU unit1 -RGSM sample1" },
    ToolTemplate { tool: "gatk", keywords: &["collectalignment", "alignment metrics"], template: "CollectAlignmentSummaryMetrics -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["collectinsertsize", "insert size"], template: "CollectInsertSizeMetrics -I {input} -O {output} -H {histogram}" },
    ToolTemplate { tool: "gatk", keywords: &["validatesamfile", "validate"], template: "ValidateSamFile -I {input}" },
    ToolTemplate { tool: "gatk", keywords: &["sortsam"], template: "SortSam -I {input} -O {output} -SO coordinate" },
    ToolTemplate { tool: "gatk", keywords: &["depthofcoverage", "coverage"], template: "DepthOfCoverage -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "picard", keywords: &["markdup", "duplicate"], template: "MarkDuplicates -I {input} -O {output} -M {metrics}" },
    ToolTemplate { tool: "picard", keywords: &["sortsam", "sort"], template: "SortSam -I {input} -O {output} -SO coordinate" },
    ToolTemplate { tool: "picard", keywords: &["addreplacereadgroups", "read group"], template: "AddOrReplaceReadGroups -I {input} -O {output} -RGID 1 -RGLB lib1 -RGPL illumina -RGPU unit1 -RGSM sample1" },
    ToolTemplate { tool: "picard", keywords: &["createsequencedictionary", "dictionary"], template: "CreateSequenceDictionary -R {reference}" },
    ToolTemplate { tool: "picard", keywords: &["collectalignment", "alignment metrics"], template: "CollectAlignmentSummaryMetrics -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "picard", keywords: &["collectinsertsize", "insert size"], template: "CollectInsertSizeMetrics -I {input} -O {output} -H {histogram}" },
    ToolTemplate { tool: "picard", keywords: &["validatesamfile", "validate"], template: "ValidateSamFile -I {input}" },
    ToolTemplate { tool: "picard", keywords: &["mergesamfiles", "merge bam"], template: "MergeSamFiles -I {inputs} -O {output}" },
    ToolTemplate { tool: "picard", keywords: &["buildbamindex", "index"], template: "BuildBamIndex -I {input}" },
    ToolTemplate { tool: "picard", keywords: &["gathervcfs"], template: "GatherVcfs -I {inputs} -O {output}" },
    ToolTemplate { tool: "picard", keywords: &["extractsequences"], template: "ExtractSequences -R {reference} -O {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["align", "mapping", "map"], template: "-x {index} -1 {read1} -2 {read2} -S {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["build", "index"], template: "hisat2-build {reference} {index}" },
    ToolTemplate { tool: "fastp", keywords: &["trim", "quality", "filter", "qc"], template: "-i {input} -o {output} -w 4 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["paired", "paired-end"], template: "-i {read1} -I {read2} -o {out1} -O {out2} -w 8 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastqc", keywords: &["quality", "qc", "quality control"], template: "{input} -o {output_dir}" },
    ToolTemplate { tool: "cutadapt", keywords: &["trim", "adapter"], template: "-a ADAPTER -o {output} {input}" },
    ToolTemplate { tool: "trimmomatic", keywords: &["trim", "paired"], template: "PE -threads 4 -phred33 {read1} {read2} {out1} {unpaired1} {out2} {unpaired2} ILLUMINACLIP:adapters.fa:2:30:10" },
    ToolTemplate { tool: "trimmomatic", keywords: &["trim", "single"], template: "SE -threads 4 -phred33 {input} {output} ILLUMINACLIP:adapters.fa:2:30:10" },
    ToolTemplate { tool: "trim_galore", keywords: &["trim", "adapter"], template: "--paired --quality 20 --length 20 --output_dir {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "minimap2", keywords: &["align", "map", "mapping", "ont", "nanopore"], template: "-t 4 -x map-ont {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["pacbio", "pb"], template: "-t 4 -x map-pb {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["sr", "short read"], template: "-t 4 -x sr {reference} {read1} {read2} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["splice", "rna", "transcript"], template: "-t 4 -x splice {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["hifi", "pacbio hifi", "map-hifi"], template: "-t 4 -x map-hifi {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["asm5", "assembly", "asm"], template: "-t 4 -x asm5 {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["ava-ont", "all-vs-all", "overlap"], template: "-t 4 -x ava-ont {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["splice", "junc-bed", "junction"], template: "-t 4 -x splice --junc-bed {input2} {reference} {input} -o {output}" },
    ToolTemplate { tool: "minimap2", keywords: &["index", "build", "-d"], template: "-d {output} {reference}" },
    ToolTemplate { tool: "minimap2", keywords: &["sam", "bam", "output format", "-a"], template: "-ax map-ont {reference} {input} -o {output}" },
    ToolTemplate { tool: "salmon", keywords: &["quant", "quantify", "expression"], template: "quant -i {index} -l A -1 {read1} -2 {read2} -p 4 -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["index"], template: "index -t {reference} -i {index}" },
    ToolTemplate { tool: "salmon", keywords: &["quant", "single-end", "single", "-r"], template: "quant -i {index} -l A -r {input} -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["gcbias", "gc bias", "gc bias correction"], template: "quant -i {index} -l A -1 {read1} -2 {read2} --gcBias -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["validatemappings", "validate mappings", "sequence bias"], template: "quant -i {index} -l A -1 {read1} -2 {read2} --gcBias --validateMappings -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["seqbias", "sequence bias"], template: "quant -i {index} -l A -1 {read1} -2 {read2} --seqBias --gcBias -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["decoys", "decoy", "d"], template: "quant -i {index} -l A -1 {read1} -2 {read2} -d {input2} -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["strandedness", "library type", "ISR", "SF"], template: "quant -i {index} -l A -1 {read1} -2 {read2} -o {output_dir}" },
    ToolTemplate { tool: "kallisto", keywords: &["quant", "quantify", "expression"], template: "quant -i {index} -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "kallisto", keywords: &["index"], template: "index -i {index} {reference}" },
    ToolTemplate { tool: "featurecounts", keywords: &["count", "quantify", "expression"], template: "-a {annotation} -o {output} -T 4 {input}" },
    ToolTemplate { tool: "stringtie", keywords: &["assemble", "transcript", "assembly"], template: "-p 4 -G {annotation} -o {output} {input}" },
    ToolTemplate { tool: "stringtie", keywords: &["merge"], template: "merge -G {annotation} -o {output} {inputs}" },
    ToolTemplate { tool: "stringtie", keywords: &["estimate", "abundance", "ballgown"], template: "-e -B -G {annotation} -o {output} {input}" },
    ToolTemplate { tool: "trinity", keywords: &["assemble", "de novo", "transcriptome"], template: "--seqType fq --left {read1} --right {read2} --CPU 4 --max_memory 16G --output {output_dir}" },
    ToolTemplate { tool: "rsem", keywords: &["calculate-expression", "quantify", "expression"], template: "rsem-calculate-expression -p 4 --paired-end {read1} {read2} {reference} {output_prefix}" },
    ToolTemplate { tool: "rsem", keywords: &["prepare-reference", "index"], template: "rsem-prepare-reference {reference} {index}" },
    ToolTemplate { tool: "rsem", keywords: &["prepare-reference", "gtf", "annotation"], template: "rsem-prepare-reference --gtf {annotation} {reference} {index}" },
    ToolTemplate { tool: "rsem", keywords: &["calculate-expression", "paired-end", "strandedness"], template: "rsem-calculate-expression -p 4 --paired-end --strandedness reverse {read1} {read2} {reference} {output_prefix}" },
    ToolTemplate { tool: "rsem", keywords: &["calculate-expression", "star"], template: "rsem-calculate-expression -p 4 --paired-end --star {read1} {read2} {reference} {output_prefix}" },
    ToolTemplate { tool: "rsem", keywords: &["generate-data-matrix", "matrix", "count matrix"], template: "rsem-generate-data-matrix {inputs} > {output}" },
    ToolTemplate { tool: "spades", keywords: &["assemble", "assembly"], template: "-t 4 -1 {read1} -2 {read2} -o {output_dir}" },
    ToolTemplate { tool: "spades", keywords: &["rna", "transcriptome"], template: "--rna -t 4 -1 {read1} -2 {read2} -o {output_dir}" },
    ToolTemplate { tool: "megahit", keywords: &["assemble", "assembly", "paired"], template: "-1 {read1} -2 {read2} -o {output_dir} --num-cpu-threads 16 --min-contig-len 500" },
    ToolTemplate { tool: "megahit", keywords: &["meta-large", "large", "complex"], template: "-1 {read1} -2 {read2} -o {output_dir} --num-cpu-threads 32 --presets meta-large --min-contig-len 500" },
    ToolTemplate { tool: "megahit", keywords: &["meta-sensitive", "sensitive", "low-abundance"], template: "-1 {read1} -2 {read2} -o {output_dir} --num-cpu-threads 16 --presets meta-sensitive --min-contig-len 500" },
    ToolTemplate { tool: "megahit", keywords: &["k-min", "k-max", "k-step", "custom k"], template: "-1 {read1} -2 {read2} -o {output_dir} --num-cpu-threads 16 --k-min 27 --k-max 127 --k-step 10" },
    ToolTemplate { tool: "megahit", keywords: &["no-mercy", "memory", "low memory"], template: "-1 {read1} -2 {read2} -o {output_dir} --num-cpu-threads 16 --no-mercy --memory 0.5 --min-contig-len 500" },
    ToolTemplate { tool: "megahit", keywords: &["continue", "resume"], template: "-o {output_dir} --continue" },
    ToolTemplate { tool: "megahit", keywords: &["single-end", "single", "-r"], template: "-r {input} -o {output_dir} --num-cpu-threads 16 --min-contig-len 500" },
    ToolTemplate { tool: "megahit", keywords: &["interleaved", "--12"], template: "--12 {input} -o {output_dir} --num-cpu-threads 16 --min-contig-len 500" },
    ToolTemplate { tool: "flye", keywords: &["assemble", "assembly", "nanopore"], template: "--nano-raw {input} --genome-size 5m --out-dir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["pacbio", "hifi"], template: "--pacbio-hifi {input} --genome-size 5m --out-dir {output_dir}" },
    ToolTemplate { tool: "canu", keywords: &["assemble", "assembly", "nanopore"], template: "-p {prefix} -d {output_dir} genomeSize=5m -nanopore-raw {input} maxThreads=8" },
    ToolTemplate { tool: "canu", keywords: &["pacbio"], template: "-p {prefix} -d {output_dir} genomeSize=5m -pacbio-raw {input} maxThreads=8" },
    ToolTemplate { tool: "hifiasm", keywords: &["assemble", "assembly"], template: "-o {output} -t 8 {input}" },
    ToolTemplate { tool: "wtdbg2", keywords: &["assemble", "assembly"], template: "-x ont -t 8 -i {input} -o {prefix}" },
    ToolTemplate { tool: "verkko", keywords: &["assemble", "assembly"], template: "--assembly --hifi {input} -o {output_dir}" },
    ToolTemplate { tool: "racon", keywords: &["polish", "consensus", "correct"], template: "{input} {overlaps} {reference}" },
    ToolTemplate { tool: "pilon", keywords: &["polish", "correct"], template: "-Xmx64g -jar pilon.jar --genome {reference} --frags {input} --output {output_prefix}" },
    ToolTemplate { tool: "busco", keywords: &["assess", "quality", "completeness"], template: "-i {input} -l bacteria -o {output_dir} -m genome" },
    ToolTemplate { tool: "freebayes", keywords: &["call", "variant", "snp"], template: "-f {reference} -o {output} {input}" },
    ToolTemplate { tool: "varscan2", keywords: &["snp", "variant call"], template: "mpileup2snp {input} --min-coverage 8 --min-var-freq 0.01 --p-value 0.05 --output-vcf 1" },
    ToolTemplate { tool: "varscan2", keywords: &["indel"], template: "mpileup2indel {input} --min-coverage 8 --min-var-freq 0.01 --p-value 0.05 --output-vcf 1" },
    ToolTemplate { tool: "varscan2", keywords: &["somatic"], template: "somatic {input} --output-vcf 1 --output {output}" },
    ToolTemplate { tool: "varscan2", keywords: &["copynumber", "cnv"], template: "copynumber {input} --output {output}" },
    ToolTemplate { tool: "strelka2", keywords: &["germline", "variant"], template: "configureStrelkaGermlineWorkflow.py --bam {input} --referenceFasta {reference} --runDir strelka_germline" },
    ToolTemplate { tool: "strelka2", keywords: &["somatic", "tumor", "tumour"], template: "configureStrelkaSomaticWorkflow.py --normalBam {normal} --tumourBam {tumor} --referenceFasta {reference} --runDir strelka_somatic" },
    ToolTemplate { tool: "strelka2", keywords: &["exome", "wes", "target region"], template: "configureStrelkaGermlineWorkflow.py --bam {input} --referenceFasta {reference} --exome --callRegions {input2} --runDir strelka_wes" },
    ToolTemplate { tool: "strelka2", keywords: &["manta", "indel candidate"], template: "configureStrelkaSomaticWorkflow.py --normalBam {normal} --tumourBam {tumor} --referenceFasta {reference} --indelCandidates {config} --runDir strelka_with_manta" },
    ToolTemplate { tool: "delly", keywords: &["call", "sv", "structural"], template: "call -g {reference} -o {output} {input}" },
    ToolTemplate { tool: "delly", keywords: &["filter"], template: "filter -o {output} {input}" },
    ToolTemplate { tool: "delly", keywords: &["merge"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "sniffles", keywords: &["call", "sv", "structural"], template: "--input {input} --vcf {output}" },
    ToolTemplate { tool: "pbsv", keywords: &["discover"], template: "discover {input} --output {output_prefix}" },
    ToolTemplate { tool: "pbsv", keywords: &["call"], template: "call {reference} {input} --output {output}" },
    ToolTemplate { tool: "pbsv", keywords: &["discover", "hifi"], template: "discover --hifi {input} --output {output_prefix}" },
    ToolTemplate { tool: "pbsv", keywords: &["call", "hifi"], template: "call --hifi {reference} {input} {output}" },
    ToolTemplate { tool: "pbsv", keywords: &["call", "hifi", "tandem-repeats"], template: "call --hifi --tandem-repeats {input2} {reference} {inputs} {output}" },
    ToolTemplate { tool: "longshot", keywords: &["call", "variant", "snp"], template: "-b {input} -f {reference} -o {output}" },
    ToolTemplate { tool: "whatshap", keywords: &["phase", "phasing"], template: "phase --reference {reference} -o {output} {input}" },
    ToolTemplate { tool: "whatshap", keywords: &["haplotag"], template: "haplotag --reference {reference} --output {output} {input}" },
    ToolTemplate { tool: "whatshap", keywords: &["stats"], template: "stats {input}" },
    ToolTemplate { tool: "whatshap", keywords: &["compare"], template: "compare {input1} {input2}" },
    ToolTemplate { tool: "snpeff", keywords: &["ann", "annotate", "effect"], template: "ann {database} {input}" },
    ToolTemplate { tool: "snpeff", keywords: &["download", "database"], template: "download {database}" },
    ToolTemplate { tool: "snpeff", keywords: &["build"], template: "build -genbank -v {database}" },
    ToolTemplate { tool: "vep", keywords: &["annotate", "effect", "variant"], template: "--input_file {input} --output_file {output} --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --fork 8 --offline" },
    ToolTemplate { tool: "vcfanno", keywords: &["annotate"], template: "{config} {input}" },
    ToolTemplate { tool: "vcftools", keywords: &["filter"], template: "--vcf {input} --recode --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["freq", "frequency"], template: "--vcf {input} --freq --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["hardy", "hwe"], template: "--vcf {input} --hardy --out {output}" },
    ToolTemplate { tool: "blast", keywords: &["blastn", "nucleotide"], template: "blastn -query {input} -db {database} -out {output}" },
    ToolTemplate { tool: "blast", keywords: &["blastp", "protein"], template: "blastp -query {input} -db {database} -out {output}" },
    ToolTemplate { tool: "blast", keywords: &["blastx"], template: "blastx -query {input} -db {database} -out {output}" },
    ToolTemplate { tool: "blast", keywords: &["makeblastdb", "database", "build"], template: "makeblastdb -in {input} -dbtype nucl" },
    ToolTemplate { tool: "diamond", keywords: &["blastp", "protein"], template: "blastp -d {database} -q {input} -o {output} --threads 4" },
    ToolTemplate { tool: "diamond", keywords: &["blastx"], template: "blastx -d {database} -q {input} -o {output} --threads 4" },
    ToolTemplate { tool: "diamond", keywords: &["makedb", "database", "build"], template: "makedb --in {input} -d {database}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmsearch", "search profile"], template: "hmmsearch --cpu 4 --tblout {output} {hmm} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmscan", "scan sequence"], template: "hmmscan --cpu 4 --tblout {output} {hmm} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmbuild", "build profile"], template: "hmmbuild {output} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmalign", "align"], template: "hmmalign -o {output} {hmm} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmpress", "press", "index"], template: "hmmpress {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "taxonomic"], template: "--input_type fastq -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "fastq", "database", "db_dir"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "mapout", "bowtie2out"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 --bowtie2out {output2} -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "ignore", "eukaryotes", "archaea"], template: "--input_type fastq --db_dir {database} --nproc 8 --ignore_eukaryotes --ignore_archaea -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "long reads", "long-reads"], template: "--input_type fastq --db_dir {database} --nproc 8 --long_reads -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "tax_lev", "taxonomic level"], template: "--input_type fastq --db_dir {database} --nproc 8 --tax_lev s -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "biom", "biom_format_output"], template: "--input_type fastq --db_dir {database} --nproc 8 --biom_format_output -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "mapout", "input_type"], template: "--input_type mapout --db_dir {database} --nproc 8 -o {output} {input}" },
    ToolTemplate { tool: "metaphlan", keywords: &["merge", "merge_metaphlan_tables", "combine"], template: "merge_metaphlan_tables.py {inputs} > {output}" },
    ToolTemplate { tool: "prokka", keywords: &["annotate", "annotation"], template: "--outdir {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["annotate", "annotation", "genome"], template: "--db {database} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["compliant", "ncbi", "submission"], template: "--db {database} --compliant --locus-tag MYORG --genus Escherichia --species coli --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["plasmid"], template: "--db {database} --plasmid pMYPLASMID --complete --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["download", "database download", "bakta_db"], template: "bakta_db download --output {output_dir}" },
    ToolTemplate { tool: "bakta", keywords: &["proteins", "hmms", "custom"], template: "--db {database} --proteins {input2} --hmms {config} --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["meta", "mag", "metagenome"], template: "--db {database} --meta --translation-table 11 --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["regions", "pre-annotated"], template: "--db {database} --regions {input2} --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["gram", "signal peptide"], template: "--db {database} --gram + --genus Bacillus --species subtilis --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["skip", "minimal", "crispr"], template: "--db {database} --skip-crispr --skip-sorf --output {output_dir} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["protein", "fasta", "bakta_proteins"], template: "bakta_proteins --db {database} --output {output_dir} {input}" },
    ToolTemplate { tool: "augustus", keywords: &["predict", "gene", "annotation"], template: "--species=human {input} --gff3=on --outfile={output}" },
    ToolTemplate { tool: "augustus", keywords: &["hints", "hint", "rna-seq hints", "extrinsic"], template: "--species=human --hintsfile={input2} --extrinsicCfgFile={config} {input} --gff3=on" },
    ToolTemplate { tool: "augustus", keywords: &["bam2hints", "bam to hints", "convert bam"], template: "bam2hints --in={input} --out={output}" },
    ToolTemplate { tool: "augustus", keywords: &["protein", "codingseq", "protein sequence"], template: "--species=fly --gff3=on --protein=on --codingseq=on {input}" },
    ToolTemplate { tool: "augustus", keywords: &["complete", "genemodel", "forward strand"], template: "--species=zebrafish --genemodel=complete --strand=forward --gff3=on {input}" },
    ToolTemplate { tool: "augustus", keywords: &["utr", "softmasking", "repeat-masked"], template: "--species=human {input} --gff3=on --softmasking=1 --UTR=on" },
    ToolTemplate { tool: "augustus", keywords: &["alternative", "splicing", "evidence"], template: "--species=human --hintsfile={input2} --alternatives-from-evidence=true --maxtracks=4 --gff3=on {input}" },
    ToolTemplate { tool: "augustus", keywords: &["region", "predictionstart", "predictionend", "specific region"], template: "--species=human --predictionStart=100000 --predictionEnd=500000 {input} --gff3=on" },
    ToolTemplate { tool: "augustus", keywords: &["proteinprofile", "profile", "kinase"], template: "--species=human --proteinprofile={input2} --gff3=on {input}" },
    ToolTemplate { tool: "repeatmasker", keywords: &["repeat", "mask"], template: "-species human -pa 8 -dir {output_dir} {input}" },
    ToolTemplate { tool: "prodigal", keywords: &["predict", "gene", "orf"], template: "-i {input} -o {output} -a {proteins}" },
    ToolTemplate { tool: "liftoff", keywords: &["lift", "transfer", "annotation"], template: "{target} {reference} -g {annotation} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["convert", "gff2gtf", "gff to gtf"], template: "agat_convert_sp_gff2gtf --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["statistics", "stats"], template: "agat_sp_statistics --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["filter", "length"], template: "agat_sp_filter_gene_by_length --gff {input} --size 300 -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["merge"], template: "agat_sp_merge_annotations --gff {input1} --gff {input2} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["extract", "sequence"], template: "agat_sp_extract_sequences --gff {input} -f {reference} -t cds -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["longest", "isoform"], template: "agat_sp_keep_longest_isoform --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["manage", "id"], template: "agat_sp_manage_IDs --gff {input} --prefix gene -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["bed", "gff2bed"], template: "agat_convert_sp_gff2bed --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["config", "configuration"], template: "agat config --expose" },
    ToolTemplate { tool: "agat", keywords: &["fix", "standardize", "gxf"], template: "agat_convert_sp_gxf2gxf --gff {input} -o {output}" },
    ToolTemplate { tool: "macs2", keywords: &["callpeak", "peak", "chip-seq"], template: "callpeak -t {input} -c {control} -n {prefix} -g hs" },
    ToolTemplate { tool: "deeptools", keywords: &["bamcoverage", "coverage", "bigwig"], template: "bamCoverage -b {input} -o {output} --numberOfProcessors 4" },
    ToolTemplate { tool: "deeptools", keywords: &["computematrix", "matrix"], template: "computeMatrix reference-point -S {input} -R {regions} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotheatmap", "heatmap"], template: "plotHeatmap -m {input} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotprofile", "profile"], template: "plotProfile -m {input} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotfingerprint", "fingerprint"], template: "plotFingerprint -b {inputs} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["multibamsummary", "correlation"], template: "multiBamSummary bins -b {inputs} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["bamcompare", "compare"], template: "bamCompare -b1 {input1} -b2 {input2} -o {output}" },
    ToolTemplate { tool: "methyldackel", keywords: &["extract", "methylation"], template: "extract {reference} {input} -o {output}" },
    ToolTemplate { tool: "methyldackel", keywords: &["mbias"], template: "mbias {reference} {input} {output_prefix}" },
    ToolTemplate { tool: "chromap", keywords: &["index", "build index"], template: "-i -r {reference} -o {output}" },
    ToolTemplate { tool: "chromap", keywords: &["atac", "atac-seq"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["chip", "chip-seq"], template: "--preset chip -x {index} -r {reference} -1 {read1} -2 {read2} -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["hic", "hi-c"], template: "--preset hic -x {index} -r {reference} -1 {read1} -2 {read2} --pairs -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["barcode", "single-cell", "scatac"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} -b {input2} --barcode-whitelist {config} -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["tn5", "tn5-shift"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} --Tn5-shift -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["sam", "sam format"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} --SAM -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["trim", "adapter", "dedup"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} --trim-adapters --remove-pcr-duplicates -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["low-mem", "memory"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} --low-mem -o {output} -t 4" },
    ToolTemplate { tool: "chromap", keywords: &["align", "mapping", "map", "chromatin"], template: "--preset atac -x {index} -r {reference} -1 {read1} -2 {read2} -o {output} -t 4" },
    ToolTemplate { tool: "pairtools", keywords: &["parse"], template: "parse -c {chromsizes} -o {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["sort"], template: "sort -o {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["merge"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "pairtools", keywords: &["dedup"], template: "dedup -o {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["select"], template: "select -o {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["split"], template: "split -o {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["stats"], template: "stats {input}" },
    ToolTemplate { tool: "modkit", keywords: &["pileup"], template: "pileup --ref {reference} --mod-code m --cpg {input} {output}" },
    ToolTemplate { tool: "modkit", keywords: &["summary"], template: "summary {input}" },
    ToolTemplate { tool: "modkit", keywords: &["extract"], template: "extract {input} -o {output}" },
    ToolTemplate { tool: "modkit", keywords: &["call-mods"], template: "call-mods {input} -o {output}" },
    ToolTemplate { tool: "mosdepth", keywords: &["coverage", "depth", "window", "by"], template: "--by 500 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["target", "region", "bed", "wes"], template: "--by {input2} --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["mapq", "filter", "quality filter"], template: "-Q 20 -F 1796 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["summary", "summary only", "no per-base"], template: "-n --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["threshold", "quantize"], template: "--by {input2} -T 1,10,20,30,50 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["fast", "quick"], template: "-x --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["fragment", "chip"], template: "-a --by {input2} --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["median"], template: "-m --by {input2} --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["chromosome", "chr", "specific"], template: "-c chr20 --prefix {prefix} {input}" },
    ToolTemplate { tool: "qualimap", keywords: &["bamqc"], template: "bamqc -bam {input} -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["rnaseq"], template: "rnaseq -bam {input} -gtf {annotation} -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["bamqc", "java-mem-size", "memory"], template: "bamqc -bam {input} --java-mem-size 8G -nt 8 -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["rnaseq", "strandedness", "strand-specific"], template: "rnaseq -bam {input} -gtf {annotation} -p strand-specific-reverse --java-mem-size 8G -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["multi-bamqc", "multi", "multiple samples"], template: "multi-bamqc -d {input} --java-mem-size 4G -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["counts", "count matrix"], template: "counts -c {input} -outdir {output_dir}" },
    ToolTemplate { tool: "seqkit", keywords: &["stats"], template: "stats -j 4 -a {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["seq", "convert"], template: "seq -a {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["seq", "reverse", "complement", "-r", "-p"], template: "seq -r -p {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["seq", "merge", "concatenate", "-m"], template: "seq -m 100 -o {output} {inputs}" },
    ToolTemplate { tool: "seqkit", keywords: &["grep", "search", "filter", "name"], template: "grep -p {pattern} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["grep", "pattern file", "-f"], template: "grep -f {input2} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["sample", "subsample"], template: "sample -s 100 {input} 10000" },
    ToolTemplate { tool: "seqkit", keywords: &["fq2fa", "format convert", "fastq to fasta"], template: "fq2fa {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["split2", "split", "partition"], template: "split2 -p 4 {input} -O {output_dir}" },
    ToolTemplate { tool: "seqkit", keywords: &["rmdup", "deduplicate"], template: "rmdup -s {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["replace", "rename"], template: "replace -p {pattern} -r {replacement} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["sort", "sort by length"], template: "sort -l {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["concat"], template: "concat {inputs} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["common"], template: "common {input1} {input2} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["fx2tab", "table"], template: "fx2tab {input}" },
    ToolTemplate { tool: "seqtk", keywords: &["sample"], template: "sample -s 100 {input} 10000" },
    ToolTemplate { tool: "seqtk", keywords: &["seq", "convert"], template: "seq -a {input}" },
    ToolTemplate { tool: "seqtk", keywords: &["subseq"], template: "subseq {input} {bed}" },
    ToolTemplate { tool: "seqtk", keywords: &["trimfq"], template: "trimfq {input}" },
    ToolTemplate { tool: "sourmash", keywords: &["compute", "sketch"], template: "compute -k 31 -o {output} {input}" },
    ToolTemplate { tool: "sourmash", keywords: &["compare"], template: "compare -o {output} {inputs}" },
    ToolTemplate { tool: "sourmash", keywords: &["gather", "search"], template: "gather {input} {database}" },
    ToolTemplate { tool: "sourmash", keywords: &["index"], template: "index -o {output} {inputs}" },
    ToolTemplate { tool: "sourmash", keywords: &["sketch", "dna", "nucleotide"], template: "sketch dna -p k=31,scaled=1000 {input} -o {output}" },
    ToolTemplate { tool: "sourmash", keywords: &["sketch", "protein", "translate"], template: "sketch protein -p k=21,scaled=1000 {input} -o {output}" },
    ToolTemplate { tool: "sourmash", keywords: &["sketch", "output-dir", "directory"], template: "sketch dna -p k=31,scaled=1000 {input} --output-dir {output_dir}" },
    ToolTemplate { tool: "sourmash", keywords: &["compare", "csv"], template: "compare {inputs} --csv {output} -k 31" },
    ToolTemplate { tool: "sourmash", keywords: &["taxonomy", "annotate", "classify"], template: "taxonomy annotate -g {input} -t {database}" },
    ToolTemplate { tool: "sourmash", keywords: &["search", "database", "threshold"], template: "search {input} {database} --threshold 0.1" },
    ToolTemplate { tool: "mash", keywords: &["sketch"], template: "sketch -p 4 -o {output} {input}" },
    ToolTemplate { tool: "mash", keywords: &["dist", "distance"], template: "dist -p 4 {input1} {input2}" },
    ToolTemplate { tool: "fastani", keywords: &["ani", "average nucleotide", "query", "ref"], template: "--query {input} --ref {reference} --output {output}" },
    ToolTemplate { tool: "fastani", keywords: &["querylist", "reflist", "list"], template: "--queryList {input} --refList {reference} --output {output}" },
    ToolTemplate { tool: "fastani", keywords: &["minfraction", "min fraction"], template: "--queryList {input} --refList {reference} --output {output} --minFraction {args}" },
    ToolTemplate { tool: "fastani", keywords: &["fraglen", "fragment length"], template: "--query {input} --ref {reference} --output {output} --fragLen {args}" },
    ToolTemplate { tool: "fastani", keywords: &["matrix", "distance matrix"], template: "--queryList {input} --refList {reference} --output {output} --matrix" },
    ToolTemplate { tool: "fastani", keywords: &["visualize", "visual"], template: "--query {input} --ref {reference} --output {output} --visualize" },
    ToolTemplate { tool: "fastani", keywords: &["version"], template: "--version" },
    ToolTemplate { tool: "orthofinder", keywords: &["ortholog", "orthogroup"], template: "-f {input_dir} -a 8" },
    ToolTemplate { tool: "mmseqs2", keywords: &["search", "easy-search"], template: "easy-search {input} {database} {output} tmp --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["cluster", "easy-cluster"], template: "easy-cluster {input} {output} tmp --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["createdb", "database"], template: "createdb {input} {output}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["index"], template: "index {input} {output}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["easy-search", "format", "format-mode"], template: "easy-search {input} {database} {output} tmp --format-mode 0 --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["easy-search", "sensitivity", "-s"], template: "easy-search {input} {database} {output} tmp -s 7.5 --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["easy-cluster", "min-seq-id", "sequence identity"], template: "easy-cluster {input} {output} tmp --min-seq-id 0.9 -c 0.8 --cov-mode 0 --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["linclust", "easy-linclust"], template: "easy-linclust {input} {output} tmp --min-seq-id 0.5 -c 0.8 --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["search", "convertalis", "advanced"], template: "search {input} {database} {output} tmp --threads 4" },
    ToolTemplate { tool: "mmseqs2", keywords: &["result2repseq", "representative", "convert2fasta"], template: "result2repseq {input} {output}" },
    ToolTemplate { tool: "iqtree2", keywords: &["tree", "phylogeny", "phylogenetic"], template: "-s {input} -m MFP --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["bootstrap", "ultrafast", "bnni", "-b"], template: "-s {input} -m MFP -B 1000 --bnni --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["alrt", "sh-like"], template: "-s {input} -m MFP --alrt 1000 --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["protein", "amino acid", "aa", "st aa"], template: "-s {input} -st AA -m TEST -B 1000 --bnni --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["partition", "merge model"], template: "-s {input} -p {input2} -m MF+MERGE --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["outgroup", "root", "rooted"], template: "-s {input} -m MFP -b 100 -o {args} --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["timetree", "date", "clock"], template: "-s {input} -m MFP --date {input2} --prefix {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["gcf", "scf", "concordance"], template: "-s {input} -m MFP -B 1000 --prefix {prefix} --gcf {input2} --scfl 100" },
    ToolTemplate { tool: "iqtree2", keywords: &["redo", "resume"], template: "-s {input} -m MFP -B 1000 --bnni --prefix {prefix} --redo" },
    ToolTemplate { tool: "fasttree", keywords: &["nucleotide", "nt", "dna"], template: "-nt -gtr {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["protein", "wag"], template: "-wag {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["lg", "le-gascuel"], template: "-lg {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["boot", "support", "bootstrap"], template: "-nt -gtr -boot 1000 -seed 42 {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["fastest", "fast"], template: "-nt -gtr -fastest {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["gamma", "rate variation"], template: "-nt -gtr -gamma {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["phylip", "matrix.phy"], template: "-nt -gtr -n 1 {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["slownni", "thorough"], template: "-nt -gtr -slownni {input}" },
    ToolTemplate { tool: "fasttree", keywords: &["tree", "phylogeny"], template: "{input}" },
    ToolTemplate { tool: "mafft", keywords: &["align", "alignment", "multiple", "auto"], template: "--auto {input}" },
    ToolTemplate { tool: "mafft", keywords: &["localpair", "accurate", "linsi"], template: "--localpair --maxiterate 1000 {input}" },
    ToolTemplate { tool: "mafft", keywords: &["adjustdirection", "strand", "orientation"], template: "--auto --adjustdirectionaccurately {input}" },
    ToolTemplate { tool: "mafft", keywords: &["phylip", "phylipout"], template: "--auto --phylipout {input}" },
    ToolTemplate { tool: "mafft", keywords: &["add", "add sequence"], template: "--add {input2} {input}" },
    ToolTemplate { tool: "mafft", keywords: &["addfragments", "fragment"], template: "--addfragments {input2} --reorder {input}" },
    ToolTemplate { tool: "mafft", keywords: &["merge", "merge alignment"], template: "--merge {input2} {input}" },
    ToolTemplate { tool: "mafft", keywords: &["seed", "anchor"], template: "--seed {input2} --auto {input}" },
    ToolTemplate { tool: "mafft", keywords: &["gap", "op", "ep", "penalty"], template: "--auto --op 2.0 --ep 0.5 {input}" },
    ToolTemplate { tool: "mafft", keywords: &["retree", "fast", "large"], template: "--retree 2 --maxiterate 0 --thread -1 {input}" },
    ToolTemplate { tool: "muscle", keywords: &["align", "alignment"], template: "-align {input} -output {output}" },
    ToolTemplate { tool: "muscle", keywords: &["super5", "large dataset", "large alignment"], template: "-super5 {input} -output {output} -threads 8" },
    ToolTemplate { tool: "muscle", keywords: &["in", "out", "v3", "legacy"], template: "-in {input} -out {output}" },
    ToolTemplate { tool: "muscle", keywords: &["threads", "parallel", "multi-threaded"], template: "-align {input} -output {output} -threads {threads}" },
    ToolTemplate { tool: "muscle", keywords: &["replicates", "stratified", "ensemble"], template: "-align {input} -output {output} -replicates 10" },
    ToolTemplate { tool: "muscle", keywords: &["diversified", "disperse", "maxcc", "letterconf"], template: "-align {input} -output {output}" },
    ToolTemplate { tool: "admixture", keywords: &["admixture", "ancestry", "population", "cross-validation"], template: "{input} 5 --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["seed", "reproducible"], template: "{input} 3 --seed=42 --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["supervised"], template: "{input} 3 --supervised" },
    ToolTemplate { tool: "admixture", keywords: &["bootstrap", "standard error"], template: "{input} 5 -B100" },
    ToolTemplate { tool: "admixture", keywords: &["projection", "p-matrix"], template: "{input} 5 -P" },
    ToolTemplate { tool: "admixture", keywords: &["em", "method"], template: "{input} 5 --method=em --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["acceleration", "quasi-newton"], template: "{input} 5 --acceleration=qn5 --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["convergence", "convergence criterion"], template: "{input} 5 -C=0.00001 --cv=10" },
    ToolTemplate { tool: "plink2", keywords: &["vcf", "convert", "import"], template: "--vcf {input} --make-pgen --out {output}" },
    ToolTemplate { tool: "plink2", keywords: &["qc", "quality control", "filter"], template: "--pfile {input} --maf 0.01 --geno 0.05 --mind 0.1 --hwe 1e-6 --make-pgen --out {output}" },
    ToolTemplate { tool: "plink2", keywords: &["pca", "principal component", "ld prune"], template: "--pfile {input} --indep-pairwise 50 10 0.1 --out ld_prune" },
    ToolTemplate { tool: "plink2", keywords: &["gwas", "association", "glm", "phenotype"], template: "--pfile {input} --pheno {input2} --pheno-name case_control --covar {config} --glm hide-covar --out {output}" },
    ToolTemplate { tool: "plink2", keywords: &["kinship", "relatedness", "king"], template: "--pfile {input} --extract {input2} --make-king-table --out {output}" },
    ToolTemplate { tool: "shapeit4", keywords: &["phase", "phasing"], template: "--input {input} --map {map_file} --region chr1 --output {output}" },
    ToolTemplate { tool: "angsd", keywords: &["genotype likelihood", "allele frequency", "saf"], template: "-bam {input} -GL 1 -doSaf 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["maf", "allele frequency", "minor allele"], template: "-bam {input} -GL 1 -doMaf 1 -doMajorMinor 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["geno", "genotype"], template: "-bam {input} -GL 1 -doGeno 4 -doMaf 1 -doMajorMinor 1 -doPost 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["theta", "tajima", "neutrality"], template: "-bam {input} -GL 1 -doSaf 1 -doThetas 1 -anc {reference} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["fst", "population differentiation"], template: "-bam {input} -GL 1 -doSaf 1 -anc {reference} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["abbababa", "d-statistic", "introgression"], template: "-bam {input} -GL 1 -doAbbababa 1 -anc {reference} -rmTrans 1 -blockSize 5000000 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["asso", "association"], template: "-bam {input} -GL 1 -doAsso 2 -doMaf 1 -doMajorMinor 1 -y {phenotype} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["fasta", "consensus"], template: "-i {input} -GL 1 -doFasta 2 -doCounts 1 -out {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["batch", "cnv", "copy number"], template: "batch {input} --reference {reference} --output-dir {output_dir}" },
    ToolTemplate { tool: "cnvkit", keywords: &["target"], template: "target {annotation} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["segment"], template: "segment {input} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["call"], template: "call {input} -o {output}" },
    ToolTemplate { tool: "sra-tools", keywords: &["download", "fetch", "prefetch"], template: "prefetch {accession} -O {output_dir}" },
    ToolTemplate { tool: "sra-tools", keywords: &["fastq", "dump", "convert"], template: "fasterq-dump {accession} -O {output_dir} -e 8" },
    ToolTemplate { tool: "tabix", keywords: &["index"], template: "-p vcf {input}" },
    ToolTemplate { tool: "bamtools", keywords: &["convert"], template: "convert -in {input} -out {output}" },
    ToolTemplate { tool: "bamtools", keywords: &["sort"], template: "sort -in {input} -out {output}" },
    ToolTemplate { tool: "bamtools", keywords: &["merge"], template: "merge -out {output} -in {inputs}" },
    ToolTemplate { tool: "bamtools", keywords: &["stats"], template: "stats -in {input}" },
    ToolTemplate { tool: "checkm2", keywords: &["predict", "quality", "completeness"], template: "predict --input {input_dir} --output_dir {output_dir} --threads 4" },
    ToolTemplate { tool: "metabat2", keywords: &["bin", "binning"], template: "-i {input} -o {output_dir}" },
    ToolTemplate { tool: "mummer", keywords: &["nucmer", "nucleotide", "align"], template: "nucmer --maxmatch -p {prefix} {reference} {query}" },
    ToolTemplate { tool: "mummer", keywords: &["promer", "protein", "align"], template: "promer -p {prefix} {reference} {query}" },
    ToolTemplate { tool: "mummer", keywords: &["delta-filter", "filter"], template: "delta-filter -i 95 -1 {input} > {output}" },
    ToolTemplate { tool: "mummer", keywords: &["show-coords", "coords"], template: "show-coords -rcl {input} > {output}" },
    ToolTemplate { tool: "mummer", keywords: &["show-snps", "snp"], template: "show-snps -Clr {input} > {output}" },
    ToolTemplate { tool: "mummer", keywords: &["dnadiff"], template: "dnadiff -p {prefix} {reference} {query}" },
    ToolTemplate { tool: "nextflow", keywords: &["run", "execute"], template: "run {pipeline} -profile docker" },
    ToolTemplate { tool: "nextflow", keywords: &["pull", "download"], template: "pull {pipeline}" },
    ToolTemplate { tool: "snakemake", keywords: &["run", "execute"], template: "--cores 4 --use-conda" },
    ToolTemplate { tool: "arriba", keywords: &["fusion", "detect"], template: "-x {input} -o {output} -O {output2} -g {reference} -a {annotation} -b {blacklist}" },
    ToolTemplate { tool: "arriba", keywords: &["draw", "visualize"], template: "draw_fusions.R --fusions={input} --alignments={bam} --genome={reference} --annotation={annotation} --output={output}" },
    ToolTemplate { tool: "arriba", keywords: &["convert", "vcf"], template: "convert_fusions_to_vcf {input}" },
    ToolTemplate { tool: "arriba", keywords: &["wrapper", "prealigned"], template: "run_arriba_on_prealigned_bam {genome_dir} {annotation} {reference} {output1} {output2} {gff3} {threads} {input}" },
    ToolTemplate { tool: "arriba", keywords: &["pipeline", "full"], template: "run_arriba {genome_dir} {annotation} {reference} {output1} {output2} {gff3} {threads} {read1} {read2}" },
    ToolTemplate { tool: "pbfusion", keywords: &["fusion", "detect"], template: "--bam {input} --gtf {annotation} --output-dir {output_dir}" },
    ToolTemplate { tool: "porechop", keywords: &["trim", "adapter"], template: "-i {input} -o {output} --threads 8" },
    ToolTemplate { tool: "porechop", keywords: &["discard_middle", "middle", "chimeric"], template: "-i {input} -o {output} --discard_middle --threads 8" },
    ToolTemplate { tool: "porechop", keywords: &["demultiplex", "barcode", "split", "-b"], template: "-i {input} -b {output_dir} --threads 8" },
    ToolTemplate { tool: "porechop", keywords: &["check", "adapter detection", "verbosity"], template: "-i {input} --check-reads 1000 --threads 8" },
    ToolTemplate { tool: "porechop", keywords: &["min_split_read_size", "split size"], template: "-i {input} -o {output} --min_split_read_size 200 --threads 8" },
    ToolTemplate { tool: "chopper", keywords: &["filter", "quality", "length"], template: "-q 10 -l 1000" },
    ToolTemplate { tool: "chopper", keywords: &["trim", "trim-by-quality", "cutoff"], template: "--trim-approach trim-by-quality --cutoff 10 -q 10 -l 1000" },
    ToolTemplate { tool: "chopper", keywords: &["best-read-segment", "segment"], template: "--trim-approach best-read-segment --cutoff 12 -q 10 -l 1000" },
    ToolTemplate { tool: "chopper", keywords: &["split", "split-by-low-quality"], template: "--trim-approach split-by-low-quality --cutoff 8 -q 10 -l 500" },
    ToolTemplate { tool: "chopper", keywords: &["headcrop", "tailcrop", "crop", "end"], template: "-q 10 -l 1000 --headcrop 30 --tailcrop 30" },
    ToolTemplate { tool: "chopper", keywords: &["maxlength", "maximum length", "max length"], template: "-q 8 -l 200 --maxlength 50000" },
    ToolTemplate { tool: "chopper", keywords: &["gc", "gc content", "mingc", "maxgc"], template: "-q 10 -l 1000 --mingc 30 --maxgc 60" },
    ToolTemplate { tool: "chopper", keywords: &["inverse", "fail", "exclude"], template: "-q 10 -l 1000 --inverse" },
    ToolTemplate { tool: "chopper", keywords: &["contaminant", "contamination", "reference fasta"], template: "-q 10 -l 1000 -c {input}" },
    ToolTemplate { tool: "nanocomp", keywords: &["qc", "compare"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["bam", "bam comparison", "compare bam"], template: "NanoComp --bam {inputs} --names Run1 Run2 --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["summary", "summary file", "compare summary"], template: "NanoComp --summary {inputs} --names Run1 Run2 --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["fastq", "fastq comparison", "compare fastq", "multiple fastq"], template: "NanoComp --fastq {inputs} --names Run1 Run2 Run3 --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["ridge", "plot type", "ridge plot"], template: "NanoComp --fastq {inputs} --plot ridge --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["downsample", "subsample"], template: "NanoComp --fastq {inputs} --downsample 10000 --outdir {output_dir}" },
    ToolTemplate { tool: "nanoplot", keywords: &["qc", "plot"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanostat", keywords: &["stats", "qc"], template: "--fastq {input}" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "consensus"], template: "{input} {output} --minPasses 3" },
    ToolTemplate { tool: "pbmm2", keywords: &["align", "mapping"], template: "align {reference} {input} {output} --sort" },
    ToolTemplate { tool: "miniasm", keywords: &["assemble", "assembly"], template: "-f {reads} {overlaps}" },
    ToolTemplate { tool: "bbtools", keywords: &["reformat", "convert"], template: "reformat.sh in={input} out={output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbmap", "align", "map"], template: "bbmap.sh ref={reference} in={input} out={output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbduk", "filter", "trim"], template: "bbduk.sh in={input} out={output} qtrim=rl trimq=20" },
    ToolTemplate { tool: "bedops", keywords: &["convert"], template: "convert2bed < {input} > {output}" },
    ToolTemplate { tool: "bedops", keywords: &["intersect", "overlap"], template: "bedintersect {input1} {input2}" },
    ToolTemplate { tool: "truvari", keywords: &["bench", "compare"], template: "bench -b {baseline} -c {call} -o {output_dir}" },
    ToolTemplate { tool: "survivor", keywords: &["merge"], template: "merge {file_list} 500 2 1 1 0 50 {input} {output}" },
    ToolTemplate { tool: "survivor", keywords: &["simsv", "simulate"], template: "simSV {config}" },
    ToolTemplate { tool: "survivor", keywords: &["stats"], template: "stats {input}" },
    ToolTemplate { tool: "cellsnp-lite", keywords: &["snp", "pileup"], template: "-s {input} -R {reference} -o {output_dir} -p 4" },
    ToolTemplate { tool: "kb", keywords: &["ref", "reference", "index"], template: "ref -i {index} -g {annotation} -f {reference}" },
    ToolTemplate { tool: "kb", keywords: &["count", "quantify"], template: "count -i {index} -g {t2g} -x 10xv3 -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "meme", keywords: &["fimo", "motif scan"], template: "fimo --oc {output_dir} {motif} {input}" },
    ToolTemplate { tool: "meme", keywords: &["meme", "motif discover"], template: "meme -oc {output_dir} -mod zoops -nmotifs 5 {input}" },
    ToolTemplate { tool: "meme", keywords: &["dreme"], template: "dreme -oc {output_dir} -p {input}" },
    ToolTemplate { tool: "meme", keywords: &["ame"], template: "ame --oc {output_dir} --control {control} {input} {motif_db}" },
    ToolTemplate { tool: "java", keywords: &["jar", "run java"], template: "-Xmx64g -jar {jar_file} {args}" },
    ToolTemplate { tool: "python", keywords: &["script", "run", "execute"], template: "{input}" },
    ToolTemplate { tool: "python", keywords: &["-c", "one-liner", "expression"], template: "-c \"{args}\"" },
    ToolTemplate { tool: "python", keywords: &["-m", "module", "http.server"], template: "-m {args}" },
    ToolTemplate { tool: "python", keywords: &["venv", "virtual environment"], template: "-m venv {output_dir}" },
    ToolTemplate { tool: "python", keywords: &["pytest", "test"], template: "-m pytest {input} -v" },
    ToolTemplate { tool: "python", keywords: &["profile", "cprofile"], template: "-m cProfile -s cumtime {input}" },
    ToolTemplate { tool: "python", keywords: &["version"], template: "--version" },
    ToolTemplate { tool: "python", keywords: &["-u", "unbuffered"], template: "-u {input}" },
    ToolTemplate { tool: "python", keywords: &["-W", "warning"], template: "-W all {input}" },
    ToolTemplate { tool: "perl", keywords: &["script", "run"], template: "-e 'print'" },
    ToolTemplate { tool: "r", keywords: &["script", "run", "Rscript"], template: "Rscript {input}" },
    ToolTemplate { tool: "r", keywords: &["-e", "expression", "one-liner"], template: "Rscript -e \"{args}\"" },
    ToolTemplate { tool: "r", keywords: &["install", "package", "install.packages"], template: "Rscript -e \"install.packages('ggplot2')\"" },
    ToolTemplate { tool: "r", keywords: &["biocmanager", "bioconductor"], template: "Rscript -e \"BiocManager::install(c('DESeq2','edgeR'))\"" },
    ToolTemplate { tool: "r", keywords: &["version", "package version"], template: "Rscript -e \"packageVersion('DESeq2')\"" },
    ToolTemplate { tool: "r", keywords: &["vanilla", "quiet", "suppress"], template: "Rscript --vanilla --quiet {input}" },
    ToolTemplate { tool: "r", keywords: &["rmarkdown", "render"], template: "Rscript -e \"rmarkdown::render('{input}', output_format='html_document')\"" },
    ToolTemplate { tool: "r", keywords: &["libpaths", "library path"], template: "Rscript -e \".libPaths()\"" },
    ToolTemplate { tool: "bash", keywords: &["script", "run"], template: "-c \"echo hello\"" },
    ToolTemplate { tool: "git", keywords: &["clone", "shallow", "depth"], template: "clone --depth 1 --branch main {url}" },
    ToolTemplate { tool: "git", keywords: &["clone"], template: "clone {url} {output_dir}" },
    ToolTemplate { tool: "git", keywords: &["commit", "stage", "all changes"], template: "commit -a -m \"message\"" },
    ToolTemplate { tool: "git", keywords: &["push", "upstream", "set upstream"], template: "push -u origin main" },
    ToolTemplate { tool: "git", keywords: &["checkout", "new branch", "create branch"], template: "checkout -b {branch}" },
    ToolTemplate { tool: "git", keywords: &["log", "oneline", "graph", "history"], template: "log --oneline --graph --decorate --all" },
    ToolTemplate { tool: "git", keywords: &["diff", "changes", "unstaged"], template: "diff HEAD" },
    ToolTemplate { tool: "git", keywords: &["stash", "save", "temporary"], template: "stash push -m \"WIP\"" },
    ToolTemplate { tool: "git", keywords: &["rebase", "onto"], template: "rebase origin/main" },
    ToolTemplate { tool: "git", keywords: &["rm", "cached", "untrack", "stop tracking"], template: "rm --cached {input}" },
    ToolTemplate { tool: "git", keywords: &["pull", "rebase"], template: "pull --rebase origin main" },
    ToolTemplate { tool: "curl", keywords: &["download", "fetch"], template: "-o {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["download", "fetch"], template: "-O {output} {url}" },
    ToolTemplate { tool: "ssh", keywords: &["connect", "remote"], template: "user@host 'command'" },
    ToolTemplate { tool: "rsync", keywords: &["sync", "transfer", "copy", "remote"], template: "-avz {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["dry-run", "preview", "simulate"], template: "-avzn {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["delete", "mirror", "exact"], template: "-avz --delete {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["resume", "partial", "interrupted"], template: "-avzP {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["exclude", "ignore"], template: "-avz --exclude='{pattern}' {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["ssh", "port", "non-standard"], template: "-avz -e 'ssh -p 2222' {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["progress", "info"], template: "-avz --info=progress2 {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["hardlink", "preserve"], template: "-avzH {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["update", "newer"], template: "-avz --update {source} {destination}" },
    ToolTemplate { tool: "find", keywords: &["find", "search file"], template: "{directory} -name '{pattern}'" },
    ToolTemplate { tool: "rm", keywords: &["remove", "delete"], template: "-rf {path}" },
    ToolTemplate { tool: "tar", keywords: &["compress", "archive"], template: "-czf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["extract", "decompress"], template: "-xzf {input}" },
    ToolTemplate { tool: "grep", keywords: &["search", "pattern", "case-insensitive"], template: "-in \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["recursive", "include", "find in files"], template: "-rn \"{pattern}\" --include='{input}' {directory}" },
    ToolTemplate { tool: "grep", keywords: &["context", "surrounding", "around"], template: "-C 3 \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["count", "number of"], template: "-c \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["extended", "regex", "multiple pattern"], template: "-E \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["filenames", "list files", "-l"], template: "-rl \"{pattern}\" {directory}" },
    ToolTemplate { tool: "grep", keywords: &["invert", "exclude", "not match"], template: "-v \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["extract", "only matching", "-o"], template: "-oE \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["filename", "header", "-H"], template: "-Hn \"{pattern}\" {input}" },
    ToolTemplate { tool: "grep", keywords: &["fixed", "literal", "-F"], template: "-F \"{pattern}\" {input}" },
    ToolTemplate { tool: "sed", keywords: &["replace", "substitute"], template: "-i 's/old/new/g' {input}" },
    ToolTemplate { tool: "awk", keywords: &["process", "column", "field"], template: "-F ',' '{print $1,$3}' {input}" },
    ToolTemplate { tool: "julia", keywords: &["script", "run", "execute"], template: "{input}" },
    ToolTemplate { tool: "julia", keywords: &["project", "environment"], template: "--project=. {input}" },
    ToolTemplate { tool: "julia", keywords: &["threads", "multi-thread"], template: "--threads auto {input}" },
    ToolTemplate { tool: "julia", keywords: &["-e", "expression", "pkg"], template: "-e '{args}'" },
    ToolTemplate { tool: "julia", keywords: &["startup", "no startup", "ci"], template: "--startup-file=no --project=. {input}" },
    ToolTemplate { tool: "julia", keywords: &["compile", "ahead-of-time"], template: "--compile=all -O2 {input}" },
    ToolTemplate { tool: "julia", keywords: &["pluto", "notebook"], template: "-e 'import Pluto; Pluto.run(port=1234)'" },
    ToolTemplate { tool: "fastq-screen", keywords: &["contamination", "screen", "screening"], template: "--conf {config} --outdir {output_dir} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["subset", "all reads", "no subsampling"], template: "--conf {config} --subset 0 --outdir {output_dir} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["bisulfite", "bismark", "paired"], template: "--conf {config} --aligner bismark --paired --outdir {output_dir} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["no_html", "no plot", "table only"], template: "--conf {config} --no_html --outdir {output_dir} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["nohits", "unmapped"], template: "--conf {config} --nohits --outdir {output_dir} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["filter", "filtering"], template: "--conf {config} --filter 1000 --outdir {output_dir} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["tag", "tagging"], template: "--conf {config} --tag --subset 0 --outdir {output_dir} {input}" },
    ToolTemplate { tool: "multiqc", keywords: &["report", "summary", "aggregate", "qc", "quality"], template: ". -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["report", "summary", "aggregate"], template: "{input_dir} -o {output_dir}" },
    ToolTemplate { tool: "multiqc", keywords: &["specific directory", "path", "results directory", "results/"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["custom name", "report name", "-n"], template: "{input_dir} -o {output_dir} -n {prefix} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["ignore", "exclude directory", "old"], template: "{input_dir} --ignore {input2} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["module", "specific module", "-m"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["exclude module", "-e"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["sample names", "rename", "sample-names", "replace-names"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["data format", "export data", "json", "tsv", "csv"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["flat", "non-interactive", "pdf"], template: "{input_dir} -o {output_dir} --flat -f" },
    ToolTemplate { tool: "multiqc", keywords: &["no report", "data only"], template: "{input_dir} -o {output_dir} --no-report -f" },
    ToolTemplate { tool: "prokka", keywords: &["annotate", "annotation"], template: "--outdir {output_dir} --prefix {prefix} --kingdom Bacteria {input}" },
    ToolTemplate { tool: "liftoff", keywords: &["lift", "transfer", "annotation"], template: "{input} {reference} -g {annotation} -o {output} -p {threads}" },
    ToolTemplate { tool: "mafft", keywords: &["align", "alignment", "multiple"], template: "--auto --thread {threads} {input}" },
    ToolTemplate { tool: "muscle", keywords: &["align", "alignment"], template: "-align {input} -output {output}" },
    ToolTemplate { tool: "fasttree", keywords: &["tree", "phylogeny"], template: "-nt {input}" },
    ToolTemplate { tool: "nanocomp", keywords: &["qc", "compare", "nanopore"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanoplot", keywords: &["qc", "plot", "nanopore"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanostat", keywords: &["stats", "qc", "nanopore"], template: "--fastq {input}" },
    ToolTemplate { tool: "porechop", keywords: &["trim", "adapter", "nanopore"], template: "-i {input} -o {output} --threads {threads}" },
    ToolTemplate { tool: "chopper", keywords: &["filter", "quality", "length", "nanopore"], template: "-i {input} -o {output} --quality 10 --length 1000 --threads {threads}" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "consensus", "pacbio"], template: "{input} {output} --minPasses 3" },
    ToolTemplate { tool: "ssh", keywords: &["connect", "remote"], template: "user@host" },
    ToolTemplate { tool: "rsync", keywords: &["sync", "transfer", "copy"], template: "-avz {input} {output}" },
    ToolTemplate { tool: "curl", keywords: &["download", "fetch"], template: "-o {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["download", "fetch"], template: "-O {output} {url}" },
    ToolTemplate { tool: "find", keywords: &["find", "search file"], template: ". -name '{input}'" },
    ToolTemplate { tool: "rm", keywords: &["remove", "delete"], template: "-rf {input}" },
    ToolTemplate { tool: "tar", keywords: &["compress", "archive"], template: "-czf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["extract", "decompress"], template: "-xzf {input}" },
    ToolTemplate { tool: "grep", keywords: &["search", "pattern"], template: "-E '{pattern}' {input}" },
    ToolTemplate { tool: "sed", keywords: &["replace", "substitute"], template: "-i 's/old/new/g' {input}" },
    ToolTemplate { tool: "awk", keywords: &["process", "column", "field"], template: "-F ',' '{{print $1,$3}}' {input}" },
    ToolTemplate { tool: "r", keywords: &["script", "run"], template: "Rscript -e \"library(ggplot2)\"" },
    ToolTemplate { tool: "python", keywords: &["script", "run"], template: "-c \"import sys; sys.exit()\"" },
    ToolTemplate { tool: "perl", keywords: &["script", "run"], template: "-e 'print'" },
    ToolTemplate { tool: "bash", keywords: &["script", "run"], template: "-c \"echo hello\"" },
    ToolTemplate { tool: "java", keywords: &["jar", "run java"], template: "-Xmx64g -jar {jar_file} {args}" },
    ToolTemplate { tool: "git", keywords: &["clone"], template: "clone {url} {output_dir}" },
    ToolTemplate { tool: "git", keywords: &["pull"], template: "pull" },
    ToolTemplate { tool: "git", keywords: &["commit"], template: "commit -m \"message\"" },
    ToolTemplate { tool: "git", keywords: &["push"], template: "push" },
    ToolTemplate { tool: "git", keywords: &["checkout"], template: "checkout -b {branch}" },
    ToolTemplate { tool: "trim_galore", keywords: &["trim", "adapter"], template: "--paired --quality 20 --length 20 --output_dir {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "trim_galore", keywords: &["trim", "single"], template: "--quality 20 --length 20 --output_dir {output_dir} {input}" },
    ToolTemplate { tool: "orthofinder", keywords: &["ortholog", "orthogroup"], template: "-f {input_dir} -a {threads}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["search", "easy-search"], template: "easy-search {input} {database} {output} tmp --threads {threads}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["cluster", "easy-cluster"], template: "easy-cluster {input} {output} tmp --threads {threads}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["createdb", "database"], template: "createdb {input} {output}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["index"], template: "index {input} {output}" },
    ToolTemplate { tool: "admixture", keywords: &["admixture", "ancestry", "population"], template: "{input} 5 --cv=10" },
    ToolTemplate { tool: "plink2", keywords: &["qc", "quality control"], template: "--pfile {input} --maf 0.01 --geno 0.05 --mind 0.1 --hwe 1e-6 --make-pgen --out {output}" },
    ToolTemplate { tool: "plink2", keywords: &["assoc", "association"], template: "--pfile {input} --assoc --out {output}" },
    ToolTemplate { tool: "plink2", keywords: &["pca", "principal component"], template: "--pfile {input} --pca 10 --out {output}" },
    ToolTemplate { tool: "shapeit4", keywords: &["phase", "phasing"], template: "--input {input} --map {map_file} --region chr1 --output {output}" },
    ToolTemplate { tool: "angsd", keywords: &["genotype likelihood", "allele frequency", "saf"], template: "-bam {input} -GL 1 -doSaf 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["maf", "allele frequency", "minor allele"], template: "-bam {input} -GL 1 -doMaf 1 -doMajorMinor 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["geno", "genotype"], template: "-bam {input} -GL 1 -doGeno 4 -doMaf 1 -doMajorMinor 1 -doPost 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["theta", "tajima", "neutrality"], template: "-bam {input} -GL 1 -doSaf 1 -doThetas 1 -anc {reference} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["fst", "population differentiation"], template: "-bam {input} -GL 1 -doSaf 1 -anc {reference} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["abbababa", "d-statistic", "introgression"], template: "-bam {input} -GL 1 -doAbbababa 1 -anc {reference} -rmTrans 1 -blockSize 5000000 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["asso", "association"], template: "-bam {input} -GL 1 -doAsso 2 -doMaf 1 -doMajorMinor 1 -y {phenotype} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["fasta", "consensus"], template: "-i {input} -GL 1 -doFasta 2 -doCounts 1 -out {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["batch", "cnv", "copy number"], template: "batch {input} --reference {reference} --output-dir {output_dir}" },
    ToolTemplate { tool: "cnvkit", keywords: &["target"], template: "target {annotation} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["segment"], template: "segment {input} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["call"], template: "call {input} -o {output}" },
    ToolTemplate { tool: "sra-tools", keywords: &["download", "fetch", "prefetch"], template: "prefetch {accession} -O {output_dir}" },
    ToolTemplate { tool: "sra-tools", keywords: &["fastq", "dump", "convert"], template: "fasterq-dump {accession} -O {output_dir} -e {threads}" },
    ToolTemplate { tool: "tabix", keywords: &["index"], template: "-p vcf {input}" },
    ToolTemplate { tool: "bamtools", keywords: &["convert"], template: "convert -in {input} -out {output}" },
    ToolTemplate { tool: "bamtools", keywords: &["sort"], template: "sort -in {input} -out {output}" },
    ToolTemplate { tool: "bamtools", keywords: &["merge"], template: "merge -out {output} -in {inputs}" },
    ToolTemplate { tool: "bamtools", keywords: &["stats"], template: "stats -in {input}" },
    ToolTemplate { tool: "checkm2", keywords: &["predict", "quality", "completeness"], template: "predict --input {input_dir} --output_dir {output_dir} --threads {threads}" },
    ToolTemplate { tool: "metabat2", keywords: &["bin", "binning"], template: "-i {input} -o {output_dir}" },
    ToolTemplate { tool: "mummer", keywords: &["nucmer", "nucleotide", "align"], template: "nucmer --maxmatch -p {prefix} {reference} {query}" },
    ToolTemplate { tool: "mummer", keywords: &["promer", "protein", "align"], template: "promer -p {prefix} {reference} {query}" },
    ToolTemplate { tool: "mummer", keywords: &["delta-filter", "filter"], template: "delta-filter -i 95 -1 {input} > {output}" },
    ToolTemplate { tool: "mummer", keywords: &["show-coords", "coords"], template: "show-coords -rcl {input} > {output}" },
    ToolTemplate { tool: "mummer", keywords: &["show-snps", "snp"], template: "show-snps -Clr {input} > {output}" },
    ToolTemplate { tool: "mummer", keywords: &["dnadiff"], template: "dnadiff -p {prefix} {reference} {query}" },
    ToolTemplate { tool: "nextflow", keywords: &["run", "execute"], template: "run {pipeline} -profile docker" },
    ToolTemplate { tool: "nextflow", keywords: &["pull", "download"], template: "pull {pipeline}" },
    ToolTemplate { tool: "snakemake", keywords: &["run", "execute"], template: "--cores {threads} --use-conda" },
    ToolTemplate { tool: "arriba", keywords: &["fusion", "detect"], template: "-x {input} -o {output} -O {output2} -g {reference} -a {annotation} -b {blacklist}" },
    ToolTemplate { tool: "arriba", keywords: &["draw", "visualize"], template: "draw_fusions.R --fusions={input} --alignments={bam} --genome={reference} --annotation={annotation} --output={output}" },
    ToolTemplate { tool: "arriba", keywords: &["convert", "vcf"], template: "convert_fusions_to_vcf {input}" },
    ToolTemplate { tool: "arriba", keywords: &["wrapper", "prealigned"], template: "run_arriba_on_prealigned_bam {genome_dir} {annotation} {reference} {output1} {output2} {gff3} {threads} {input}" },
    ToolTemplate { tool: "arriba", keywords: &["pipeline", "full"], template: "run_arriba {genome_dir} {annotation} {reference} {output1} {output2} {gff3} {threads} {read1} {read2}" },
    ToolTemplate { tool: "pbfusion", keywords: &["fusion", "detect"], template: "--bam {input} --gtf {annotation} --output-dir {output_dir}" },
    ToolTemplate { tool: "porechop", keywords: &["trim", "adapter"], template: "-i {input} -o {output} --threads {threads}" },
    ToolTemplate { tool: "chopper", keywords: &["filter", "quality", "length"], template: "-i {input} -o {output} --quality 10 --length 1000 --threads {threads}" },
    ToolTemplate { tool: "nanocomp", keywords: &["qc", "compare"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanoplot", keywords: &["qc", "plot"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanostat", keywords: &["stats", "qc"], template: "--fastq {input}" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "consensus"], template: "{input} {output} --minPasses 3" },
    ToolTemplate { tool: "pbmm2", keywords: &["align", "mapping"], template: "align {reference} {input} {output} --sort" },
    ToolTemplate { tool: "miniasm", keywords: &["assemble", "assembly"], template: "-f {reads} {overlaps}" },
    ToolTemplate { tool: "bbtools", keywords: &["reformat", "convert"], template: "reformat.sh in={input} out={output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbmap", "align", "map"], template: "bbmap.sh ref={reference} in={input} out={output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbduk", "filter", "trim"], template: "bbduk.sh in={input} out={output} qtrim=rl trimq=20" },
    ToolTemplate { tool: "bedops", keywords: &["convert"], template: "convert2bed < {input} > {output}" },
    ToolTemplate { tool: "bedops", keywords: &["intersect", "overlap"], template: "bedintersect {input1} {input2}" },
    ToolTemplate { tool: "truvari", keywords: &["bench", "compare"], template: "bench -b {baseline} -c {call} -o {output_dir}" },
    ToolTemplate { tool: "survivor", keywords: &["merge"], template: "merge {file_list} 500 2 1 1 0 50 {input} {output}" },
    ToolTemplate { tool: "survivor", keywords: &["simsv", "simulate"], template: "simSV {config}" },
    ToolTemplate { tool: "survivor", keywords: &["stats"], template: "stats {input}" },
    ToolTemplate { tool: "cellsnp-lite", keywords: &["snp", "pileup"], template: "-s {input} -R {reference} -o {output_dir} -p {threads}" },
    ToolTemplate { tool: "kb", keywords: &["ref", "reference", "index"], template: "ref -i {index} -g {annotation} -f {reference}" },
    ToolTemplate { tool: "kb", keywords: &["count", "quantify"], template: "count -i {index} -g {t2g} -x 10xv3 -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "meme", keywords: &["fimo", "motif scan"], template: "fimo --oc {output_dir} {motif} {input}" },
    ToolTemplate { tool: "meme", keywords: &["meme", "motif discover"], template: "meme -oc {output_dir} -mod zoops -nmotifs 5 {input}" },
    ToolTemplate { tool: "meme", keywords: &["dreme"], template: "dreme -oc {output_dir} -p {input}" },
    ToolTemplate { tool: "meme", keywords: &["ame"], template: "ame --oc {output_dir} --control {control} {input} {motif_db}" },
    ToolTemplate { tool: "java", keywords: &["jar", "run java"], template: "-Xmx64g -jar {jar_file} {args}" },
    ToolTemplate { tool: "python", keywords: &["script", "run"], template: "-c \"import sys; sys.exit()\"" },
    ToolTemplate { tool: "perl", keywords: &["script", "run"], template: "-e 'print'" },
    ToolTemplate { tool: "r", keywords: &["script", "run"], template: "Rscript -e \"library(ggplot2)\"" },
    ToolTemplate { tool: "bash", keywords: &["script", "run"], template: "-c \"echo hello\"" },
    ToolTemplate { tool: "git", keywords: &["clone"], template: "clone {url} {output_dir}" },
    ToolTemplate { tool: "git", keywords: &["pull"], template: "pull" },
    ToolTemplate { tool: "git", keywords: &["commit"], template: "commit -m \"message\"" },
    ToolTemplate { tool: "git", keywords: &["push"], template: "push" },
    ToolTemplate { tool: "git", keywords: &["checkout"], template: "checkout -b {branch}" },
    ToolTemplate { tool: "git", keywords: &["log"], template: "log --oneline -10" },
    ToolTemplate { tool: "curl", keywords: &["download", "fetch"], template: "-o {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["download", "fetch"], template: "-O {output} {url}" },
    ToolTemplate { tool: "ssh", keywords: &["connect", "remote"], template: "user@host 'command'" },
    ToolTemplate { tool: "rsync", keywords: &["sync", "transfer", "copy"], template: "-avz {source} {destination}" },
    ToolTemplate { tool: "find", keywords: &["find", "search file"], template: "{directory} -name '{pattern}'" },
    ToolTemplate { tool: "rm", keywords: &["remove", "delete"], template: "-rf {path}" },
    ToolTemplate { tool: "tar", keywords: &["compress", "archive"], template: "-czf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["extract", "decompress"], template: "-xzf {input}" },
    ToolTemplate { tool: "grep", keywords: &["search", "pattern"], template: "-E '{pattern}' {input}" },
    ToolTemplate { tool: "sed", keywords: &["replace", "substitute"], template: "-i 's/old/new/g' {input}" },
    ToolTemplate { tool: "awk", keywords: &["process", "column", "field"], template: "-F ',' '{{print $1,$3}}' {input}" },
    ToolTemplate { tool: "julia", keywords: &["script", "run"], template: "-e \"println()\"" },
    ToolTemplate { tool: "fastq-screen", keywords: &["contamination", "screen"], template: "--outdir {output_dir} {input}" },
    ToolTemplate { tool: "mafft", keywords: &["automatic", "auto algorithm"], template: "--auto {input}" },
    ToolTemplate { tool: "mafft", keywords: &["highly accurate", "fewer than 200", "localpair", "linsi"], template: "--localpair --maxiterate 1000 {input}" },
    ToolTemplate { tool: "mafft", keywords: &["rna", "strand orientation", "adjustdirection"], template: "--auto --adjustdirectionaccurately {input}" },
    ToolTemplate { tool: "mafft", keywords: &["phylip", "phylogenetic format"], template: "--auto --phylipout {input}" },
    ToolTemplate { tool: "mafft", keywords: &["add new sequences", "add sequence to existing"], template: "--add {input2} {input}" },
    ToolTemplate { tool: "mafft", keywords: &["addfragments", "fragment to existing"], template: "--addfragments {input2} --reorder {input}" },
    ToolTemplate { tool: "mafft", keywords: &["merge alignment", "merge two existing", "without re-aligning"], template: "--merge {input2} {input}" },
    ToolTemplate { tool: "mafft", keywords: &["seed alignment", "anchor"], template: "--seed {input2} --auto {input}" },
    ToolTemplate { tool: "mafft", keywords: &["gap penalt", "custom gap", "fine-tuning"], template: "--auto --op 2.0 --ep 0.5 {input}" },
    ToolTemplate { tool: "mafft", keywords: &["very large dataset", "fast alignment", "retree"], template: "--retree 2 --maxiterate 0 --thread -1 {input}" },
    ToolTemplate { tool: "orthofinder", keywords: &["ortholog", "orthogroup", "proteome"], template: "-f {input_dir} -a 8" },
    ToolTemplate { tool: "orthofinder", keywords: &["msa", "mafft", "iq-tree", "diamond"], template: "-f {input_dir} -M msa -S diamond -A mafft -T iqtree -a 8" },
    ToolTemplate { tool: "orthofinder", keywords: &["orthogroup only", "without gene tree", "fast proteome"], template: "-f {input_dir} -og" },
    ToolTemplate { tool: "orthofinder", keywords: &["restart", "existing", "add a new species"], template: "-b {database} -f {input_dir} -a 8" },
    ToolTemplate { tool: "orthofinder", keywords: &["mmseqs2", "faster all-vs-all"], template: "-f {input_dir} -S mmseqs2 -a 8" },
    ToolTemplate { tool: "orthofinder", keywords: &["output directory", "fixed output"], template: "-f {input_dir} -o {output_dir} -a 8" },
    ToolTemplate { tool: "trim_galore", keywords: &["paired-end", "illumina", "quality-filter paired"], template: "--paired --quality 20 --length 36 --gzip -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "trim_galore", keywords: &["rrbs", "bisulfite"], template: "--paired --rrbs --quality 20 --length 20 --gzip -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "trim_galore", keywords: &["single-end", "automatic adapter"], template: "--quality 20 --length 36 --gzip -o {output_dir} {input}" },
    ToolTemplate { tool: "trim_galore", keywords: &["specific adapter", "non-standard", "custom adapter"], template: "--paired --adapter AGATCGGAAGAGCACACGTCT --adapter2 AGATCGGAAGAGCGTCGTGTA --quality 20 --gzip -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "r", keywords: &["rscript", "script non-interactively", "run an r script"], template: "Rscript {input}" },
    ToolTemplate { tool: "r", keywords: &["command-line argument", "rscript.*--input"], template: "Rscript {input} --input {input2} --output {output}" },
    ToolTemplate { tool: "r", keywords: &["one-liner", "expression", "rscript -e"], template: "Rscript -e \"{args}\"" },
    ToolTemplate { tool: "r", keywords: &["install", "package", "install.packages", "cran"], template: "Rscript -e \"install.packages('{args}')\"" },
    ToolTemplate { tool: "r", keywords: &["bioconductor", "biocmanager"], template: "Rscript -e \"BiocManager::install(c('{args}'))\"" },
    ToolTemplate { tool: "r", keywords: &["version", "packageversion", "check installed version"], template: "Rscript -e \"packageVersion('{args}')\"" },
    ToolTemplate { tool: "r", keywords: &["vanilla", "quiet", "suppress startup"], template: "Rscript --vanilla --quiet {input}" },
    ToolTemplate { tool: "r", keywords: &["libpaths", "library path"], template: "Rscript -e \".libPaths()\"" },
    ToolTemplate { tool: "r", keywords: &["rmarkdown", "render"], template: "Rscript -e \"rmarkdown::render('{input}', output_format='html_document')\"" },
    ToolTemplate { tool: "r", keywords: &["list.*package", "installed.package"], template: "Rscript -e \"ip <- installed.packages(lib.loc=.libPaths()[1]); cat(paste(ip[,'Package'],ip[,'Version'],sep='='), sep='\\n')\"" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "consensus", "pacbio"], template: "{input} {output}" },
    ToolTemplate { tool: "pbccs", keywords: &["min-passes", "minimum passes"], template: "{input} {output} --min-passes 3" },
    ToolTemplate { tool: "pbccs", keywords: &["hifi-kinetics", "kinetics"], template: "{input} {output} --hifi-kinetics" },
    ToolTemplate { tool: "pbccs", keywords: &["chunk"], template: "{input} {output} --chunk 1/4" },
    ToolTemplate { tool: "fastqc", keywords: &["quality control", "qc report"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["thread", "multi-thread", "-t"], template: "-t 4 -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["noextract", "no extraction"], template: "--noextract -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["bam", "bam file qc"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["casava"], template: "--casava -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["svg"], template: "--svg -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["memory"], template: "--memory 1024 -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["contaminant", "-c", "-a"], template: "-a {input2} -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["kmer", "-k"], template: "-k 5 -o {output_dir} {input}" },
    ToolTemplate { tool: "bowtie2", keywords: &["build", "index", "bowtie2-build"], template: "bowtie2-build {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["build.*thread"], template: "bowtie2-build --threads 8 {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["paired-end", "paired", "-1", "-2"], template: "-x {index} -1 {read1} -2 {read2} -p 8" },
    ToolTemplate { tool: "bowtie2", keywords: &["single-end", "single", "-u"], template: "-x {index} -U {input} --very-sensitive" },
    ToolTemplate { tool: "bowtie2", keywords: &["no-unal", "no unaligned"], template: "-x {index} -1 {read1} -2 {read2} --no-unal -S {output}" },
    ToolTemplate { tool: "bowtie2", keywords: &["rg-id", "read group", "rg sm"], template: "-x {index} -1 {read1} -2 {read2} --rg-id sample1 --rg SM:sample1 --rg LB:lib1 --rg PL:ILLUMINA" },
    ToolTemplate { tool: "bowtie2", keywords: &["local", "very-sensitive-local"], template: "-x {index} -1 {read1} -2 {read2} --local --very-sensitive-local" },
    ToolTemplate { tool: "bowtie2", keywords: &["fast", "fast alignment"], template: "-x {index} -U {input} --fast -S {output}" },
    ToolTemplate { tool: "bowtie2", keywords: &["un-conc", "un-concordant"], template: "-x {index} -1 {read1} -2 {read2} --un-conc {output}" },
    ToolTemplate { tool: "star", keywords: &["genomegenerate", "generate genome", "genome index", "create index"], template: "--runMode genomeGenerate --runThreadN 4 --genomeDir {genome_dir} --genomeFastaFiles {reference} --sjdbGTFfile {annotation}" },
    ToolTemplate { tool: "star", keywords: &["alignreads", "align", "mapping", "map read"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {input} --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["zcat", "gz", "readfilecommand"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["twopass", "two-pass", "2-pass"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {input} --readFilesCommand zcat --twopassMode Basic --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["unmapped", "outreadsunmapped"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outReadsUnmapped Fastx" },
    ToolTemplate { tool: "star", keywords: &["genecounts", "quantmode", "quantification"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --quantMode GeneCounts" },
    ToolTemplate { tool: "star", keywords: &["solo", "single-cell", "cell barcode"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM Unsorted --outFileNamePrefix {output_prefix} --soloType CB_UMI_Simple --soloCBwhitelist {input2} --soloUMIlen 10 --soloFeatures Gene" },
    ToolTemplate { tool: "star", keywords: &["chimeric", "chimouttype", "fusion"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --chimOutType WithinBAM --chimSegmentMin 20" },
    ToolTemplate { tool: "star", keywords: &["genomeload", "shared memory"], template: "--runMode alignReads --genomeDir {genome_dir} --genomeLoad LoadAndKeep" },
    ToolTemplate { tool: "star", keywords: &["multimap", "outfiltermultimapnmax", "unique mapping"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outFilterMultimapNmax 1 --outSAMattributes NH HI AS NM" },
    ToolTemplate { tool: "star", keywords: &["stringent", "mismatch", "outfiltermismatch"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outFilterMismatchNoverLmax 0.05 --outFilterScoreMinOverLread 0.9" },
    ToolTemplate { tool: "nanocomp", keywords: &["fastq", "nanocomp.*fastq"], template: "NanoComp --fastq {inputs} --names {args} --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["bam", "nanocomp.*bam"], template: "NanoComp --bam {inputs} --names {args} --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["summary", "nanocomp.*summary"], template: "NanoComp --summary {inputs} --names {args} --plot ridge --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["downsample"], template: "NanoComp --bam {inputs} --names {args} --downsample 50000 --outdir {output_dir}" },
    ToolTemplate { tool: "nanocomp", keywords: &["prefix"], template: "NanoComp --fastq {inputs} --names {args} --outdir {output_dir} --prefix {prefix}" },
    ToolTemplate { tool: "bakta", keywords: &["annotate", "annotation", "bakta annotate"], template: "--db {database} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["compliant", "ncbi", "locus-tag"], template: "--db {database} --compliant --locus-tag {args} --genus {args} --species {args} --strain {args} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["plasmid", "complete"], template: "--db {database} --plasmid {args} --complete --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["bakta_db", "download database"], template: "bakta_db download --output {database}" },
    ToolTemplate { tool: "bakta", keywords: &["proteins", "hmms", "custom"], template: "--db {database} --proteins {input2} --hmms {input3} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["meta", "metagenome"], template: "--db {database} --meta --translation-table 11 --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["regions"], template: "--db {database} --regions {input2} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["gram"], template: "--db {database} --gram + --genus {args} --species {args} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["skip-crispr", "skip-sorf", "minimal"], template: "--db {database} --skip-crispr --skip-sorf --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["bakta_proteins", "protein"], template: "bakta_proteins --db {database} --output {output_dir} {input}" },
    ToolTemplate { tool: "angsd", keywords: &["genotype likelihood", "glf", "snp_pval"], template: "-bam {input} -GL 1 -doGlf 2 -doMaf 1 -doMajorMinor 1 -SNP_pval 1e-6 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["saf", "allele frequency spectrum", "per-site"], template: "-bam {input} -GL 1 -doSaf 1 -anc {reference} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["realsfs", "1d site frequency"], template: "realSFS {input}" },
    ToolTemplate { tool: "angsd", keywords: &["theta", "tajima", "sliding window"], template: "-bam {input} -GL 1 -doSaf 1 -doThetas 1 -pest {input2} -anc {reference} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["fst", "2d sfs", "population differentiation"], template: "realSFS {input} {input2} && realSFS fst index {input} {input2} -sfs {input3} -fstout {output}" },
    ToolTemplate { tool: "angsd", keywords: &["geno", "genotype", "posterior"], template: "-bam {input} -GL 1 -doGeno 4 -doMaf 1 -doMajorMinor 1 -doPost 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["abbababa", "d-statistic", "introgression", "dstat"], template: "-bam {input} -GL 1 -doAbbababa 1 -anc {reference} -rmTrans 1 -blockSize 5000000 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["asso", "association", "score test"], template: "-bam {input} -GL 1 -doAsso 2 -doMaf 1 -doMajorMinor 1 -y {input2} -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["consensus", "fasta", "doFasta"], template: "-i {input} -GL 1 -doFasta 2 -doCounts 1 -out {output}" },
    ToolTemplate { tool: "angsd", keywords: &["ngsadmix", "admix_input", "genotype likelihood for admix"], template: "-bam {input} -GL 1 -doGlf 2 -doMajorMinor 1 -doMaf 1 -SNP_pval 1e-6 -out {output}" },
    ToolTemplate { tool: "admixture", keywords: &["cross-validation", "cv", "k=5"], template: "{input} 5 --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["seed", "reproducible", "convergence testing"], template: "{input} 3 --seed=42 --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["multiple k", "optimal k", "across multiple"], template: "{input} K --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["supervised", "reference population"], template: "{input} 3 --supervised" },
    ToolTemplate { tool: "admixture", keywords: &["bootstrap", "standard error", "replicate"], template: "{input} 5 -B100" },
    ToolTemplate { tool: "admixture", keywords: &["projection", "p-matrix"], template: "{input} 5 -P" },
    ToolTemplate { tool: "admixture", keywords: &["em", "em algorithm", "difficult convergence"], template: "{input} 5 --method=em --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["quasi-newton", "acceleration", "qn5"], template: "{input} 5 --acceleration=qn5 --cv=10" },
    ToolTemplate { tool: "admixture", keywords: &["stricter convergence", "convergence criterion"], template: "{input} 5 -C=0.00001 --cv=10" },
    ToolTemplate { tool: "fastqc", keywords: &["qc", "quality"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["hifi", "pacbio hifi"], template: "-o {output} -t 4 {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["nanopore", "ont"], template: "--nano {input} -o {output} -t 4" },
    ToolTemplate { tool: "hifiasm", keywords: &["hic", "hi-c"], template: "-o {output} -t 4 --h1 {read1} --h2 {read2} {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["trio", "trio-binning"], template: "-o {output} -t 4 -1 {read1} -2 {read2} {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["purge", "purge_dups"], template: "-o {output} -t 4 -l 3 {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "kraken2 classify"], template: "--db {database} {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["paired", "paired-end"], template: "--db {database} --paired {read1} {read2} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["confidence", "score threshold"], template: "--db {database} --confidence 0.1 {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["report-minimizer"], template: "--db {database} --report-minimizer-data {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["use-names", "scientific name"], template: "--db {database} --use-names {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["gzip", "compressed", "gz"], template: "--db {database} --gzip-compressed {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["build", "kraken2-build"], template: "kraken2-build --db {database} --add-to-library {input}" },
    ToolTemplate { tool: "vcftools", keywords: &["filter", "quality"], template: "--vcf {input} --filter 'QUAL>30' --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["maf", "minor allele frequency"], template: "--vcf {input} --maf 0.01 --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["hwe", "hardy-weinberg"], template: "--vcf {input} --hwe 1e-6 --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["missing", "geno", "mind"], template: "--vcf {input} --max-missing 0.9 --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["thin", "ld", "prune"], template: "--vcf {input} --thin 1000 --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["freq", "frequency"], template: "--vcf {input} --freq --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["hardy"], template: "--vcf {input} --hardy --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["site-depth", "depth per site"], template: "--vcf {input} --site-depth --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["window-pi", "pi", "nucleotide diversity"], template: "--vcf {input} --window-pi 10000 --out {output}" },
    ToolTemplate { tool: "vcftools", keywords: &["windowed-weir-fst-pop", "fst", "population"], template: "--vcf {input} --weir-fst-pop {input2} --weir-fst-pop {input3} --out {output}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmsearch", "search profile"], template: "hmmsearch --cpu 4 --tblout {output} {input} {database}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmscan", "scan sequence"], template: "hmmscan --cpu 4 --tblout {output} {database} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmbuild", "build profile"], template: "hmmbuild {output} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmalign", "align"], template: "hmmalign -o {output} {input} {input2}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmemit", "emit"], template: "hmmemit -o {output} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["jackhmmer", "iterative"], template: "jackhmmer --cpu 4 --tblout {output} {input} {database}" },
    ToolTemplate { tool: "hmmer", keywords: &["phmmer", "protein search"], template: "phmmer --cpu 4 --tblout {output} {input} {database}" },
    ToolTemplate { tool: "hmmer", keywords: &["cut_ga", "gathering threshold"], template: "hmmsearch --cpu 4 --cut_ga --tblout {output} {input} {database}" },
    ToolTemplate { tool: "hmmer", keywords: &["domtblout", "domain"], template: "hmmsearch --cpu 4 --domtblout {output} {input} {database}" },
    ToolTemplate { tool: "star", keywords: &["chimsegmentmin", "chimsegment", "arriba"], template: "--runMode alignReads --runThreadN 4 --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --chimSegmentMin 10 --chimOutType WithinBAM --chimJunctionOverhangMin 10 --chimScoreDropMax 30 --peOverlapNbasesMin 12" },
    ToolTemplate { tool: "multiqc", keywords: &["multiqc", "report", "aggregate"], template: ". -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["specific directory"], template: "{input_dir} -o {output_dir}" },
    ToolTemplate { tool: "multiqc", keywords: &["module", "specific module"], template: "-m fastqc -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["sample name", "rename", "sample-names"], template: "--sample-names {input} -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["exclude"], template: "--exclude fastqc -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["interactive", "export"], template: "--export -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["flat", "no interactive"], template: "--flat -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["pdf", "template"], template: "--template default -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["zip", "data export"], template: "--data-dir -o {output_dir} ." },
    ToolTemplate { tool: "multiqc", keywords: &["cl_config", "config"], template: "--cl_config '{args}' -o {output_dir} ." },
    ToolTemplate { tool: "seqkit", keywords: &["stats", "statistics"], template: "stats {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["grep", "search", "filter by name"], template: "grep -p -r '{pattern}' {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["seq", "extract sequence"], template: "seq {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["fx2tab", "convert"], template: "fx2tab {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["rmdup", "remove duplicate"], template: "rmdup -s {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["sample", "subsample"], template: "sample -p 0.1 {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["replace", "rename"], template: "replace -p '(.+)' -r 'new_\\1' {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["sort", "sort by length"], template: "sort -l {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["split", "split file"], template: "split -p 10 {input} -O {output_dir}" },
    ToolTemplate { tool: "seqkit", keywords: &["concat", "merge"], template: "concat {inputs} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["translate", "protein"], template: "translate {input} -o {output}" },
    ToolTemplate { tool: "sra-tools", keywords: &["prefetch", "download sra"], template: "prefetch {accession} -O {output_dir}" },
    ToolTemplate { tool: "sra-tools", keywords: &["fasterq-dump", "convert sra"], template: "fasterq-dump {accession} -O {output_dir} -e 8" },
    ToolTemplate { tool: "sra-tools", keywords: &["fastq-dump", "split"], template: "fastq-dump --split-files {accession} -O {output_dir}" },
    ToolTemplate { tool: "sra-tools", keywords: &["sam-dump", "bam"], template: "sam-dump {accession} | samtools view -bS - > {output}" },
    ToolTemplate { tool: "sra-tools", keywords: &["abidump", "abi"], template: "abidump {accession}" },
    ToolTemplate { tool: "igvtools", keywords: &["count", "coverage", "wig"], template: "count -z 5 -w 25 {input} {output} hg38" },
    ToolTemplate { tool: "igvtools", keywords: &["tdf", "totdf"], template: "toTDF -z 5 {input} {output} hg38" },
    ToolTemplate { tool: "igvtools", keywords: &["index"], template: "index {input}" },
    ToolTemplate { tool: "igvtools", keywords: &["sort"], template: "sort {input} {output}" },
    ToolTemplate { tool: "igvtools", keywords: &["tile"], template: "tile {input} {output}" },
    ToolTemplate { tool: "igvtools", keywords: &["version"], template: "version" },
    ToolTemplate { tool: "igvtools", keywords: &["wib", "wibToTdf"], template: "wibToTdf {input} {output}" },
    ToolTemplate { tool: "igvtools", keywords: &["formatexp"], template: "formatexp -c {input} {output}" },
    ToolTemplate { tool: "wget", keywords: &["download", "fetch"], template: "-O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["recursive", "mirror"], template: "-r -l 1 -nd -P {output_dir} {url}" },
    ToolTemplate { tool: "wget", keywords: &["auth", "user", "password"], template: "--user={args} --password={args} -O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["continue", "resume"], template: "-c -O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["quiet"], template: "-q -O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["no-check-certificate"], template: "--no-check-certificate -O {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["download", "fetch"], template: "-o {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["upload", "post"], template: "-T {input} {url}" },
    ToolTemplate { tool: "curl", keywords: &["header", "api"], template: "-H '{args}' -o {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["silent", "quiet"], template: "-s -o {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["follow redirect"], template: "-L -o {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["form", "multipart"], template: "-F 'file=@{input}' {url}" },
    ToolTemplate { tool: "ssh", keywords: &["connect", "remote"], template: "{args}" },
    ToolTemplate { tool: "ssh", keywords: &["tunnel", "port forward", "-l"], template: "-L {args} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["key", "identity"], template: "-i {input} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["command", "execute"], template: "{args} '{args}'" },
    ToolTemplate { tool: "trinity", keywords: &["trinity", "de novo", "assemble transcriptome"], template: "--seqType fq --max_memory 50G --CPU 4 --left {read1} --right {read2} --output {output_dir}" },
    ToolTemplate { tool: "trinity", keywords: &["genome-guided", "genome guided"], template: "--genome_guided_bam {input} --genome_guided_max_intron 10000 --max_memory 50G --CPU 4 --output {output_dir}" },
    ToolTemplate { tool: "trinity", keywords: &["single-end", "single"], template: "--seqType fq --max_memory 50G --CPU 4 --single {input} --output {output_dir}" },
    ToolTemplate { tool: "trinity", keywords: &["normalize", "insilico"], template: "normalize_by_kmer_coverage --seqType fq --max_cov 50 --single {input} --output {output_dir}" },
    ToolTemplate { tool: "trinity", keywords: &["super-read", "bowtie"], template: "run_bowtie2_for_trinity.pl --target {input} --output {output_dir}" },
    ToolTemplate { tool: "pilon", keywords: &["pilon", "polish", "fix"], template: "--genome {input} --bam {input2} --output {output} --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["fix", "fix bases only"], template: "--genome {input} --bam {input2} --output {output} --fix bases --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["changes", "vcf output"], template: "--genome {input} --bam {input2} --output {output} --changes --vcf --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["diploid"], template: "--genome {input} --bam {input2} --output {output} --diploid --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["frags"], template: "--genome {input} --frags {input2} --output {output} --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["unpaired"], template: "--genome {input} --unpaired {input2} --output {output} --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["chunk"], template: "--genome {input} --bam {input2} --output {output} --chunksize 5000000 --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["gap"], template: "--genome {input} --bam {input2} --output {output} --fix gaps --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["local"], template: "--genome {input} --bam {input2} --output {output} --fix all --threads 4" },
    ToolTemplate { tool: "pilon", keywords: &["mindepth"], template: "--genome {input} --bam {input2} --output {output} --mindepth 5 --threads 4" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-calculate-expression", "calculate expression"], template: "calculate-expression --paired --num-threads 4 {read1} {read2} {index} {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-calculate-expression.*single", "single-end expression"], template: "calculate-expression --num-threads 4 {input} {index} {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-prepare-reference", "prepare reference"], template: "prepare-reference --gtf {annotation} {reference} {index}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-generate-data-matrix", "data matrix"], template: "generate-data-matrix {inputs} > {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-generate-ngvector", "ngvector"], template: "generate-ngvector {inputs} > {output}" },
    ToolTemplate { tool: "rsem", keywords: &["ebseq", "ebseq-test"], template: "ebseq-test {input} {input2} > {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-plot-model", "plot model"], template: "plot-model {input} {output}" },
    ToolTemplate { tool: "rsem", keywords: &["bam2wig", "wig"], template: "bam2wig {input}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-simulate-reads", "simulate"], template: "simulate-reads {index} {args} {output}" },
    ToolTemplate { tool: "rsem", keywords: &["sort-learned-params", "sort params"], template: "sort-learned-params {input} {output}" },
    ToolTemplate { tool: "homer", keywords: &["maketagdirectory", "tag directory", "create tag"], template: "makeTagDirectory {output_dir} {input} -genome {genome} -checkGC" },
    ToolTemplate { tool: "homer", keywords: &["findpeaks", "peak calling", "factor"], template: "findPeaks {input} -style factor -o {output}" },
    ToolTemplate { tool: "homer", keywords: &["histone", "broad peak"], template: "findPeaks {input} -style histone -o {output}" },
    ToolTemplate { tool: "homer", keywords: &["findmotif", "motif discovery", "findmotifsgenome"], template: "findMotifsGenome.pl {input} {genome} {output_dir} -size 200 -mask" },
    ToolTemplate { tool: "homer", keywords: &["findmotifsgenome.*bg", "background"], template: "findMotifsGenome.pl {input} {genome} {output_dir} -size 200 -bg {input2} -mask" },
    ToolTemplate { tool: "homer", keywords: &["annotatepeaks", "annotation"], template: "annotatePeaks.pl {input} {genome}" },
    ToolTemplate { tool: "homer", keywords: &["annotatepeaks.*gtf"], template: "annotatePeaks.pl {input} {genome} -gtf {input2}" },
    ToolTemplate { tool: "homer", keywords: &["makeucscfile", "ucsc"], template: "makeUCSCfile {input} -o {output}" },
    ToolTemplate { tool: "homer", keywords: &["pos2bed", "bed"], template: "pos2bed.pl {input}" },
    ToolTemplate { tool: "homer", keywords: &["getdifferentialpeaksreplicates", "differential replicates"], template: "getDifferentialPeaksReplicates.pl -t {input} -c {input2} -genome {genome} -o {output}" },
    ToolTemplate { tool: "homer", keywords: &["getdifferentialpeaks", "differential"], template: "getDifferentialPeaks {input} {input2}" },
    ToolTemplate { tool: "homer", keywords: &["getdifferentialgenes", "rna"], template: "getDifferentialGenes.pl {input} {input2}" },
    ToolTemplate { tool: "homer", keywords: &["mergepeaks", "merge"], template: "mergePeaks {inputs} -d 100 -prefix {prefix} -venn {output}" },
    ToolTemplate { tool: "homer", keywords: &["annotatepeaks.*pl.*go", "go"], template: "annotatePeaks.pl {input} {genome} -go" },
    ToolTemplate { tool: "homer", keywords: &["makegenomedirectory", "genome directory"], template: "makeGenomeDirectory.pl {genome}" },
    ToolTemplate { tool: "pairtools", keywords: &["parse", "parse sam"], template: "parse {input} -c {annotation} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["sort"], template: "sort {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["dedup", "deduplicate"], template: "dedup {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["select", "filter"], template: "select '{args}' {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["merge"], template: "merge {inputs} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["flip"], template: "flip {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["restrict", "restriction"], template: "restrict {input} -f {annotation} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["phase", "phasing"], template: "phase {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["markasdup"], template: "markasdup {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["split"], template: "split {input} -o {output}" },
    ToolTemplate { tool: "meme", keywords: &["fimo", "motif scan"], template: "fimo --oc {output_dir} {input} {input2}" },
    ToolTemplate { tool: "meme", keywords: &["meme", "motif discover"], template: "meme -oc {output_dir} -mod zoops -nmotifs 5 {input}" },
    ToolTemplate { tool: "meme", keywords: &["dreme"], template: "dreme -oc {output_dir} -p {input}" },
    ToolTemplate { tool: "meme", keywords: &["ame"], template: "ame --oc {output_dir} --control {input2} {input} {database}" },
    ToolTemplate { tool: "meme", keywords: &["mast", "motif search"], template: "mast {input} {database} -o {output_dir}" },
    ToolTemplate { tool: "meme", keywords: &["tomtom", "motif compare"], template: "tomtom -o {output_dir} {input} {database}" },
    ToolTemplate { tool: "meme", keywords: &["glam2", "gapless alignment"], template: "glam2 -o {output_dir} {input}" },
    ToolTemplate { tool: "meme", keywords: &["glam2scan"], template: "glam2scan -o {output_dir} {input} {database}" },
    ToolTemplate { tool: "meme", keywords: &["meme-chip"], template: "meme-chip -oc {output_dir} -db {database} {input}" },
    ToolTemplate { tool: "meme", keywords: &["centrimo", "centrality"], template: "centrimo --oc {output_dir} {input} {database}" },
    ToolTemplate { tool: "meme", keywords: &["spamo", "spacing"], template: "SpaMo -oc {output_dir} {input} {database}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["classify", "classify_wf"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus 4" },
    ToolTemplate { tool: "gtdbtk", keywords: &["infer", "infer_wf"], template: "infer_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus 4" },
    ToolTemplate { tool: "gtdbtk", keywords: &["ani_screen"], template: "ani_screen --genome_dir {input_dir} --out_dir {output_dir} --cpus 4" },
    ToolTemplate { tool: "gtdbtk", keywords: &["decorate"], template: "decorate --input_tree {input} --output_tree {output}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["root"], template: "root --input_tree {input} --output_tree {output}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["convert_to_itol"], template: "convert_to_itol --input_tree {input} --output_tree {output}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["export_msa"], template: "export_msa --output {output}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["trim_msa"], template: "trim_msa --input {input} --output {output}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["assign_taxonomy"], template: "assign_taxonomy --genome_dir {input_dir} --out_dir {output_dir}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["infer_tree"], template: "infer_tree --msa {input} --out_dir {output_dir}" },
    ToolTemplate { tool: "metaphlan", keywords: &["profile", "metaphlan profile"], template: "--input_type fastq {input} --output {output} --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["bowtie2", "database"], template: "--input_type fastq {input} --output {output} --bowtie2db {database} --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["bam", "input_type bam"], template: "--input_type bam {input} --output {output} --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["merge", "merge_metaphlan"], template: "merge_metaphlan_tables.py {inputs} > {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["strainphlan"], template: "strainphlan --ifn_samples {input} --output {output_dir} --nprocs 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["stat", "stat_q"], template: "stat_q {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["sample_id"], template: "--input_type fastq {input} --output {output} --sample_id {args} --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["unclassified"], template: "--input_type fastq {input} --output {output} --unclassified_estimation --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["add_viruses"], template: "--input_type fastq {input} --output {output} --add_viruses --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["ignore_viruses"], template: "--input_type fastq {input} --output {output} --ignore_viruses --nproc 4" },
    ToolTemplate { tool: "metaphlan", keywords: &["ignore_eukaryotes"], template: "--input_type fastq {input} --output {output} --ignore_eukaryotes --nproc 4" },
    ToolTemplate { tool: "java", keywords: &["jar", "run java"], template: "-Xmx64g -jar {jar_file} {args}" },
    ToolTemplate { tool: "java", keywords: &["classpath", "-cp"], template: "-cp {jar_file} {args}" },
    ToolTemplate { tool: "java", keywords: &["xmx", "memory"], template: "-Xmx{args} -jar {jar_file} {args}" },
    ToolTemplate { tool: "python", keywords: &["script", "run"], template: "{input}" },
    ToolTemplate { tool: "python", keywords: &["-c", "expression"], template: "-c \"{args}\"" },
    ToolTemplate { tool: "python", keywords: &["-m", "module"], template: "-m {args}" },
    ToolTemplate { tool: "perl", keywords: &["script", "run"], template: "{input}" },
    ToolTemplate { tool: "perl", keywords: &["-e", "expression"], template: "-e '{args}'" },
    ToolTemplate { tool: "bash", keywords: &["script", "run"], template: "{input}" },
    ToolTemplate { tool: "bash", keywords: &["-c", "command"], template: "-c \"{args}\"" },
    ToolTemplate { tool: "julia", keywords: &["script", "run"], template: "{input}" },
    ToolTemplate { tool: "julia", keywords: &["-e", "expression"], template: "-e \"{args}\"" },
    ToolTemplate { tool: "git", keywords: &["clone"], template: "clone {url} {output_dir}" },
    ToolTemplate { tool: "git", keywords: &["pull"], template: "pull" },
    ToolTemplate { tool: "git", keywords: &["commit"], template: "commit -m \"{args}\"" },
    ToolTemplate { tool: "git", keywords: &["push"], template: "push" },
    ToolTemplate { tool: "git", keywords: &["checkout"], template: "checkout -b {args}" },
    ToolTemplate { tool: "git", keywords: &["log"], template: "log --oneline -10" },
    ToolTemplate { tool: "git", keywords: &["branch"], template: "branch -a" },
    ToolTemplate { tool: "git", keywords: &["diff"], template: "diff" },
    ToolTemplate { tool: "git", keywords: &["status"], template: "status" },
    ToolTemplate { tool: "git", keywords: &["merge"], template: "merge {args}" },
    ToolTemplate { tool: "git", keywords: &["stash"], template: "stash" },
    ToolTemplate { tool: "git", keywords: &["add"], template: "add {input}" },
    ToolTemplate { tool: "git", keywords: &["reset"], template: "reset --hard HEAD" },
    ToolTemplate { tool: "find", keywords: &["find", "search file"], template: "{directory} -name '{pattern}'" },
    ToolTemplate { tool: "find", keywords: &["type"], template: "{directory} -name '{pattern}' -type f" },
    ToolTemplate { tool: "find", keywords: &["delete"], template: "{directory} -name '{pattern}' -type f -delete" },
    ToolTemplate { tool: "find", keywords: &["user"], template: "{directory} -user {args} -type f" },
    ToolTemplate { tool: "find", keywords: &["size"], template: "{directory} -name '{pattern}' -size +{args}" },
    ToolTemplate { tool: "find", keywords: &["mtime", "modified"], template: "{directory} -name '{pattern}' -mtime -{args}" },
    ToolTemplate { tool: "find", keywords: &["exec"], template: "{directory} -name '{pattern}' -exec {args} {{}} \\;" },
    ToolTemplate { tool: "find", keywords: &["perm", "permission"], template: "{directory} -perm {args}" },
    ToolTemplate { tool: "find", keywords: &["empty"], template: "{directory} -empty" },
    ToolTemplate { tool: "find", keywords: &["maxdepth"], template: "{directory} -maxdepth {args} -name '{pattern}'" },
    ToolTemplate { tool: "rm", keywords: &["remove", "delete"], template: "-rf {path}" },
    ToolTemplate { tool: "rm", keywords: &["force"], template: "-f {path}" },
    ToolTemplate { tool: "rm", keywords: &["interactive"], template: "-i {path}" },
    ToolTemplate { tool: "rm", keywords: &["verbose"], template: "-rvf {path}" },
    ToolTemplate { tool: "rm", keywords: &["symlink"], template: "-rf {path}" },
    ToolTemplate { tool: "tar", keywords: &["compress", "archive", "create"], template: "-czf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["extract", "decompress"], template: "-xzf {input}" },
    ToolTemplate { tool: "tar", keywords: &["list", "contents"], template: "-tzf {input}" },
    ToolTemplate { tool: "tar", keywords: &["append"], template: "-rf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["verbose"], template: "-czvf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["bzip2"], template: "-cjf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["xz"], template: "-cJf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["directory"], template: "-czf {output} -C {directory} {input}" },
    ToolTemplate { tool: "tar", keywords: &["exclude"], template: "-czf {output} --exclude='{pattern}' {input}" },
    ToolTemplate { tool: "tar", keywords: &["wildcard"], template: "-czf {output} --wildcards '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["search", "pattern"], template: "-E '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["recursive", "directory"], template: "-r '{pattern}' {directory}" },
    ToolTemplate { tool: "grep", keywords: &["invert", "exclude"], template: "-v '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["count"], template: "-c '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["context", "surrounding"], template: "-C 3 '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["file", "from file"], template: "-f {input2} {input}" },
    ToolTemplate { tool: "grep", keywords: &["line number"], template: "-n '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["word", "whole word"], template: "-w '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["case insensitive"], template: "-i '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["fixed string", "literal"], template: "-F '{pattern}' {input}" },
    ToolTemplate { tool: "sed", keywords: &["replace", "substitute"], template: "-i 's/{args}/{args}/g' {input}" },
    ToolTemplate { tool: "sed", keywords: &["delete line", "delete"], template: "'/pattern/d' {input}" },
    ToolTemplate { tool: "sed", keywords: &["print", "range"], template: "-n '1,10p' {input}" },
    ToolTemplate { tool: "sed", keywords: &["append"], template: "'/pattern/a\\\\text' {input}" },
    ToolTemplate { tool: "sed", keywords: &["insert"], template: "'/pattern/i\\\\text' {input}" },
    ToolTemplate { tool: "sed", keywords: &["in-place"], template: "-i 's/{args}/{args}/g' {input}" },
    ToolTemplate { tool: "sed", keywords: &["line number"], template: "'10s/{args}/{args}/' {input}" },
    ToolTemplate { tool: "sed", keywords: &["empty line", "blank"], template: "'/^$/d' {input}" },
    ToolTemplate { tool: "sed", keywords: &["multiple"], template: "-e 's/{args}/{args}/g' -e 's/{args}/{args}/g' {input}" },
    ToolTemplate { tool: "sed", keywords: &["quiet", "silent"], template: "-n '{args}p' {input}" },
    ToolTemplate { tool: "awk", keywords: &["process", "column", "field"], template: "'{print $1,$3}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["separator", "delimiter", "-f:"], template: "-F: '{print $1}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["condition", "filter"], template: "'$3 > 100 {print $0}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["sum", "total"], template: "'{sum += $1} END {print sum}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["count", "lines"], template: "'END {print NR}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["begin", "header"], template: "'BEGIN {print \\\"header\\\"} {print $0}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["printf", "format"], template: "'{printf \\\"%s\\t%d\\n\\\", $1, $2}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["regex", "match"], template: "'/pattern/ {print $0}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["field separator", "comma"], template: "-F, '{print $1,$2}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["tab separated"], template: "-F'\\t' '{print $1,$2}' {input}" },
    ToolTemplate { tool: "fastani", keywords: &["find", "search genome"], template: "find /genomes -name '{input}'" },
    ToolTemplate { tool: "java", keywords: &["version"], template: "-version" },
    ToolTemplate { tool: "java", keywords: &["xshowsettings", "show settings"], template: "-XshowSettings:all -version" },
    ToolTemplate { tool: "java", keywords: &["printflags", "flags"], template: "-XX:+PrintFlagsFinal -version" },
    ToolTemplate { tool: "java", keywords: &["sortsam", "sort sam"], template: "-Xmx16g -jar {jar_file} SortSam I={input} O={output} SORT_ORDER=coordinate" },
    ToolTemplate { tool: "java", keywords: &["haplotypecaller", "variant call"], template: "-Xmx8g -jar {jar_file} HaplotypeCaller -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "java", keywords: &["trimmomatic", "pe", "illumina clip"], template: "-Xmx4g -jar {jar_file} PE -threads 8 {read1} {read2} {out1} {unpaired1} {out2} {unpaired2} ILLUMINACLIP:{input2}:2:30:10" },
    ToolTemplate { tool: "java", keywords: &["classpath", "-cp"], template: "-cp {jar_file} {args}" },
    ToolTemplate { tool: "java", keywords: &["zgc", "usezgc"], template: "-Xmx32g -XX:+UseZGC -jar {jar_file} {args}" },
    ToolTemplate { tool: "julia", keywords: &["project", "environment"], template: "--project=. {input}" },
    ToolTemplate { tool: "julia", keywords: &["threads", "multi-thread"], template: "--threads auto {input}" },
    ToolTemplate { tool: "julia", keywords: &["startup", "no startup", "ci"], template: "--startup-file=no --project=. {input}" },
    ToolTemplate { tool: "julia", keywords: &["compile", "ahead-of-time"], template: "--compile=all -O2 {input}" },
    ToolTemplate { tool: "julia", keywords: &["pluto", "notebook"], template: "-e 'import Pluto; Pluto.run(port=1234)'" },
    ToolTemplate { tool: "kraken2", keywords: &["memory-mapping", "memory mapping"], template: "--db {database} --memory-mapping --paired {read1} {read2} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["quick", "quick operation"], template: "--db {database} --quick --paired {read1} {read2} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["minimum-hit-groups"], template: "--db {database} --paired --minimum-hit-groups 3 --confidence 0.1 {read1} {read2} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["use-mpa-style", "mpa"], template: "--db {database} --paired --report {output2} --use-mpa-style {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["build", "standard", "kraken2-build standard"], template: "kraken2-build --standard --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["download-taxonomy", "download library", "build custom"], template: "kraken2-build --download-taxonomy --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["classified-out", "unclassified-out"], template: "--db {database} --paired --output {output} --report {output2} --unclassified-out {args} {read1} {read2}" },
    ToolTemplate { tool: "metaphlan", keywords: &["db_dir", "database dir"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["bowtie2db", "bowtie2 database"], template: "--input_type fastq --bowtie2db {database} --index latest --nproc 8 {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["tax_lev", "taxonomic level"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 --tax_lev g {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["ignore_eukaryotes", "ignore archaea"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 --ignore_eukaryotes --ignore_archaea {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["mapout", "bowtie2out"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 --bowtie2out {output2} {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["input_type mapout"], template: "--input_type mapout --db_dir {database} --index latest {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["biom_format_output", "biom"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 --biom_format_output {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["long_reads", "long read"], template: "--input_type fastq --db_dir {database} --index latest --nproc 8 --long_reads {input} -o {output}" },
    ToolTemplate { tool: "metaphlan", keywords: &["merge", "merge_metaphlan"], template: "merge_metaphlan_tables.py {inputs}" },
    ToolTemplate { tool: "multiqc", keywords: &["ignore", "exclude directory"], template: "{input_dir} --ignore {input2} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["module", "specific module", "-m"], template: "{input_dir} -m fastqc -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["exclude module", "-e"], template: "{input_dir} -e cutadapt -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["sample-names", "rename sample"], template: "{input_dir} --sample-names {input2} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["replace-names"], template: "{input_dir} --replace-names {input2} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["data-format", "json", "no-report"], template: "{input_dir} --data-format json --no-report -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["flat", "no interactive"], template: "{input_dir} --flat -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["export"], template: "{input_dir} --export -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["multiple dir"], template: "{inputs} -o {output_dir} -f" },
    ToolTemplate { tool: "flye", keywords: &["meta", "metagenome"], template: "--meta --nano-raw {input} --out-dir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["nano-hq", "high quality"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["resume"], template: "--nano-raw {input} --genome-size 5m --out-dir {output_dir} --resume" },
    ToolTemplate { tool: "flye", keywords: &["asm-coverage"], template: "--pacbio-hifi {input} --genome-size 3g --out-dir {output_dir} --asm-coverage 40" },
    ToolTemplate { tool: "flye", keywords: &["scaffold"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir} --scaffold" },
    ToolTemplate { tool: "flye", keywords: &["keep-haplotypes", "diploid"], template: "--pacbio-hifi {input} --genome-size 600m --out-dir {output_dir} --keep-haplotypes" },
    ToolTemplate { tool: "flye", keywords: &["read-error"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir} --read-error 0.05" },
    ToolTemplate { tool: "flye", keywords: &["stop-after", "contigger"], template: "--nano-raw {input} --genome-size 5m --out-dir {output_dir} --stop-after contigger" },
    ToolTemplate { tool: "flye", keywords: &["iterations"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir} --iterations 2" },
    ToolTemplate { tool: "kallisto", keywords: &["quant", "quantify", "paired"], template: "quant -i {index} -o {output_dir} -b 100 {read1} {read2}" },
    ToolTemplate { tool: "kallisto", keywords: &["single", "single-end"], template: "quant -i {index} -o {output_dir} --single -l 200 -s 20 -b 100 {input}" },
    ToolTemplate { tool: "kallisto", keywords: &["rf-stranded", "stranded"], template: "quant -i {index} -o {output_dir} --rf-stranded -b 100 {read1} {read2}" },
    ToolTemplate { tool: "kallisto", keywords: &["index", "k 21"], template: "index -k 21 -i {index} {input}" },
    ToolTemplate { tool: "kallisto", keywords: &["d-list"], template: "index -i {index} --d-list {input2} {input}" },
    ToolTemplate { tool: "kallisto", keywords: &["pseudobam"], template: "quant -i {index} -o {output_dir} --pseudobam -b 100 {read1} {read2}" },
    ToolTemplate { tool: "kallisto", keywords: &["genomebam"], template: "quant -i {index} -o {output_dir} --genomebam -g {annotation} -c {input2} -b 100 {read1} {read2}" },
    ToolTemplate { tool: "kallisto", keywords: &["bus", "10x"], template: "bus -i {index} -o {output_dir} -x 10xv3 {read1} {read2}" },
    ToolTemplate { tool: "fastp", keywords: &["paired", "paired-end", "-i -I"], template: "-i {read1} -I {read2} -o {out1} -O {out2} -w 8 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["single", "single-end", "trim"], template: "-i {input} -o {output} -l 50 -w 8 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["quality", "-q", "qual filter"], template: "-i {read1} -I {read2} -o {out1} -O {out2} -q 20 -l 36 -w 8 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["trim_poly_a", "polya"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --trim_poly_a -w 8 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["disable_adapter", "disable quality"], template: "-i {read1} -I {read2} -o /dev/null -O /dev/null --disable_adapter_trimming --disable_quality_filtering -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["merge", "--merge"], template: "-i {read1} -I {read2} --merge --merged_out {output} -o {out1} -O {out2} -w 8" },
    ToolTemplate { tool: "fastp", keywords: &["correction"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --correction -w 8" },
    ToolTemplate { tool: "fastp", keywords: &["trim_poly_g"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --trim_poly_g --poly_g_min_len 10" },
    ToolTemplate { tool: "fastp", keywords: &["cut_front", "cut_tail"], template: "-i {input} -o {output} --cut_front --cut_tail -q 20 -w 8" },
    ToolTemplate { tool: "fastp", keywords: &["dedup"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --dedup --dup_calc_accuracy 4 -w 8" },
    ToolTemplate { tool: "fastqc", keywords: &["thread", "multi-thread", "-t"], template: "-t 4 -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["noextract"], template: "--noextract -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["bam"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["contaminant", "-a", "-c"], template: "-a {input2} -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["kmer", "-k"], template: "-k 5 -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["casava"], template: "--casava -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["svg"], template: "--svg -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["memory"], template: "--memory 1024 -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["format", "-f"], template: "-f fastq -o {output_dir} {input}" },
    ToolTemplate { tool: "kb", keywords: &["ref", "reference", "index", "kb ref"], template: "ref -i {index} -g {annotation} -f1 {reference}" },
    ToolTemplate { tool: "kb", keywords: &["ref.*database", "ref.*mouse"], template: "ref -d mouse -i {index} -g {annotation}" },
    ToolTemplate { tool: "kb", keywords: &["ref.*nac", "nac workflow"], template: "ref --workflow nac -i {index} -g {annotation} -f1 {reference} -f2 {input2} -c1 {input3} -c2 {input4}" },
    ToolTemplate { tool: "kb", keywords: &["ref.*kite"], template: "ref --workflow kite -i {index} -g {annotation} -f1 {reference}" },
    ToolTemplate { tool: "kb", keywords: &["count", "quantify", "kb count"], template: "count -i {index} -g {annotation} -x 10xv3 -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "kb", keywords: &["count.*lamanno", "velocity"], template: "count -i {index} -g {annotation} -x 10xv3 --workflow lamanno -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "kb", keywords: &["count.*h5ad"], template: "count -i {index} -g {annotation} -x 10xv3 --h5ad -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "kb", keywords: &["count.*nac"], template: "count -i {index} -g {annotation} -c1 {input2} -c2 {input3} -x 10xv3 --workflow nac --h5ad -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "kb", keywords: &["count.*kite"], template: "count -i {index} -g {annotation} -x 10xv3 --workflow kite --h5ad -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "kb", keywords: &["count.*cellranger"], template: "count -i {index} -g {annotation} -x 10xv3 --cellranger -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "mummer", keywords: &["nucmer.*prefix", "nucmer.*-p"], template: "nucmer --prefix={prefix} {reference} {input}" },
    ToolTemplate { tool: "mummer", keywords: &["nucmer.*mum"], template: "nucmer --mum -p {prefix} {reference} {input}" },
    ToolTemplate { tool: "mummer", keywords: &["nucmer.*nosimplify"], template: "nucmer --maxmatch --nosimplify --prefix={prefix} {reference} {input}" },
    ToolTemplate { tool: "mummer", keywords: &["show-coords", "coords"], template: "show-coords -r -c -l {input}" },
    ToolTemplate { tool: "mummer", keywords: &["show-snps", "snp"], template: "show-snps -Clr {input}" },
    ToolTemplate { tool: "mummer", keywords: &["show-tiling"], template: "show-tiling -i 95 -l 1000 {input}" },
    ToolTemplate { tool: "mummer", keywords: &["mummerplot", "plot"], template: "mummerplot --png --prefix={prefix} {input}" },
    ToolTemplate { tool: "miniasm", keywords: &["ava-ont", "ont assembly"], template: "-x ava-ont {input} {input2}" },
    ToolTemplate { tool: "miniasm", keywords: &["assemble", "assembly", "-f"], template: "-f {input} {input2}" },
    ToolTemplate { tool: "miniasm", keywords: &["mapping", "-R"], template: "-R -f {input} {input2}" },
    ToolTemplate { tool: "miniasm", keywords: &["sg", "string graph"], template: "-p sg -f {input} {input2}" },
    ToolTemplate { tool: "arriba", keywords: &["fusion", "detect", "-x"], template: "-x {input} -o {output} -O {output2} -g {reference} -a {annotation} -b {blacklist}" },
    ToolTemplate { tool: "arriba", keywords: &["draw", "visualize", "draw_fusions"], template: "draw_fusions.R --fusions={input} --alignments={bam} --genome={reference} --annotation={annotation} --output={output}" },
    ToolTemplate { tool: "arriba", keywords: &["convert", "vcf"], template: "convert_fusions_to_vcf {input}" },
    ToolTemplate { tool: "arriba", keywords: &["run_arriba_on_prealigned_bam", "prealigned"], template: "run_arriba_on_prealigned_bam {genome_dir} {annotation} {reference} {output} {output2} {gff3} {threads} {input}" },
    ToolTemplate { tool: "arriba", keywords: &["run_arriba", "full pipeline"], template: "run_arriba {genome_dir} {annotation} {reference} {output} {output2} {gff3} {threads} {read1} {read2}" },
    ToolTemplate { tool: "r", keywords: &["rscript", "script non-interactively", "run an r script", ".r"], template: "Rscript {input}" },
    ToolTemplate { tool: "r", keywords: &["command-line argument", "rscript.*--input"], template: "Rscript {input} --input {input2} --output {output}" },
    ToolTemplate { tool: "r", keywords: &["one-liner", "expression", "rscript -e"], template: "Rscript -e \"{args}\"" },
    ToolTemplate { tool: "r", keywords: &["install", "package", "install.packages", "cran"], template: "Rscript -e \"install.packages('{args}', repos='https://cloud.r-project.org')\"" },
    ToolTemplate { tool: "r", keywords: &["bioconductor", "biocmanager"], template: "Rscript -e \"BiocManager::install(c('{args}'))\"" },
    ToolTemplate { tool: "r", keywords: &["version", "packageversion", "check installed version"], template: "Rscript -e \"packageVersion('{args}')\"" },
    ToolTemplate { tool: "r", keywords: &["vanilla", "quiet", "suppress startup"], template: "Rscript --vanilla --quiet {input}" },
    ToolTemplate { tool: "r", keywords: &["libpaths", "library path"], template: "Rscript -e \".libPaths()\"" },
    ToolTemplate { tool: "r", keywords: &["rmarkdown", "render"], template: "Rscript -e \"rmarkdown::render('{input}', output_format='html_document')\"" },
    ToolTemplate { tool: "r", keywords: &["list.*package", "installed.package"], template: "Rscript -e \"ip <- installed.packages(lib.loc=.libPaths()[1]); cat(paste(ip[,'Package'],ip[,'Version'],sep='='), sep='\\n')\"" },
    ToolTemplate { tool: "r", keywords: &["cat", "paste", "collapse"], template: "Rscript -e \"cat(paste({args}, collapse=','), '\\n')\"" },
    ToolTemplate { tool: "find", keywords: &["size", "large file"], template: ". -type f -size +{args}" },
    ToolTemplate { tool: "find", keywords: &["iname", "case insensitive name"], template: ". -iname '{pattern}'" },
    ToolTemplate { tool: "find", keywords: &["newer", "modified after"], template: ". -type f -newer {input}" },
    ToolTemplate { tool: "find", keywords: &["perm", "permission"], template: ". -type f -perm {args}" },
    ToolTemplate { tool: "bowtie2", keywords: &["bowtie2-build", "build index", "build a bowtie2 index"], template: "bowtie2-build {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["bowtie2-build", "threads", "large genome"], template: "bowtie2-build --threads {threads} {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["paired-end", "paired", "-1", "-2", "align paired"], template: "-x {index} -1 {read1} -2 {read2} -p {threads}" },
    ToolTemplate { tool: "bowtie2", keywords: &["single-end", "single", "-U", "align single"], template: "-x {index} -U {input} --very-sensitive" },
    ToolTemplate { tool: "bowtie2", keywords: &["no-unal", "discard unaligned", "unaligned"], template: "-x {index} -1 {read1} -2 {read2} --no-unal" },
    ToolTemplate { tool: "bowtie2", keywords: &["rg-id", "read group", "rg SM", "gatk"], template: "-x {index} -1 {read1} -2 {read2} --rg-id sample1 --rg SM:sample1 --rg LB:lib1 --rg PL:ILLUMINA" },
    ToolTemplate { tool: "bowtie2", keywords: &["local", "soft-clip", "very-sensitive-local"], template: "-x {index} -1 {read1} -2 {read2} --local --very-sensitive-local" },
    ToolTemplate { tool: "bowtie2", keywords: &["rna-seq", "rna", "discarding unaligned"], template: "-x {index} -1 {read1} -2 {read2} --no-unal" },
    ToolTemplate { tool: "bowtie2", keywords: &["fast", "quick", "quality check"], template: "-x {index} -U {input} --fast" },
    ToolTemplate { tool: "bowtie2", keywords: &["un-conc", "unmapped", "unmapped reads"], template: "-x {index} -1 {read1} -2 {read2} --un-conc {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["hisat2-build", "build index", "build a hisat2"], template: "hisat2-build {reference} {index}" },
    ToolTemplate { tool: "hisat2", keywords: &["hisat2-build", "splice", "ss", "exon"], template: "hisat2-build {reference} {index} --ss {input2} --exon {config}" },
    ToolTemplate { tool: "hisat2", keywords: &["dta", "transcriptome", "align paired rna"], template: "-x {index} -1 {read1} -2 {read2} --dta" },
    ToolTemplate { tool: "hisat2", keywords: &["strand", "rf", "strand-specific", "fr-first"], template: "-x {index} -1 {read1} -2 {read2} --rna-strandness RF --dta" },
    ToolTemplate { tool: "hisat2", keywords: &["single-end", "single", "-U", "align single rna"], template: "-x {index} -U {input} --dta" },
    ToolTemplate { tool: "hisat2", keywords: &["no-spliced", "genomic", "dna-seq", "non-spliced"], template: "-x {index} -U {input} --no-spliced-alignment" },
    ToolTemplate { tool: "hisat2", keywords: &["no-unal", "discard unmapped", "unmapped"], template: "-x {index} -1 {read1} -2 {read2} --dta --no-unal" },
    ToolTemplate { tool: "star", keywords: &["genomegenerate", "generate genome", "genome index", "build index", "build genome index"], template: "--runMode genomeGenerate --genomeDir {genome_dir} --genomeFastaFiles {reference} --sjdbGTFfile {annotation}" },
    ToolTemplate { tool: "star", keywords: &["alignreads", "align", "paired-end", "gzipped", "zcat"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["twopass", "two-pass", "junction", "single-end"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {input} --readFilesCommand zcat --twopassMode Basic --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["unmapped", "fastx", "output unmapped"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outReadsUnmapped Fastx" },
    ToolTemplate { tool: "star", keywords: &["genecounts", "quantmode", "gene counts", "quantification"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --quantMode GeneCounts" },
    ToolTemplate { tool: "star", keywords: &["solo", "starsolo", "10x", "single-cell", "cb_umi"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM Unsorted --outFileNamePrefix {output_prefix} --soloType CB_UMI_Simple --soloCBwhitelist {input2} --soloUMIlen 10 --soloFeatures Gene" },
    ToolTemplate { tool: "star", keywords: &["chim", "chimeric", "fusion", "arriba"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --chimOutType WithinBAM --chimSegmentMin 20" },
    ToolTemplate { tool: "star", keywords: &["genomeload", "shared memory", "load genome"], template: "--runMode alignReads --genomeDir {genome_dir} --genomeLoad LoadAndKeep" },
    ToolTemplate { tool: "star", keywords: &["unique", "multimap", "uniquely mapped", "outfiltermultimap"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outFilterMultimapNmax 1" },
    ToolTemplate { tool: "star", keywords: &["strict", "mismatch", "high specificity", "outfiltermismatch"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outFilterMismatchNoverLmax 0.05 --outFilterScoreMinOverLread 0.9" },
    ToolTemplate { tool: "bcftools", keywords: &["mpileup", "pileup", "call variant from bam"], template: "mpileup -f {reference} -Ou {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "filter", "qual", "snp", "high-quality"], template: "view -i 'QUAL>30 && INFO/DP>10 && TYPE=\"snp\"' -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["merge", "combine", "multiple vcf"], template: "merge -O z -o {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "sample", "extract sample", "specific sample"], template: "view -s SAMPLE_NAME -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["norm", "normalize", "split multi-allelic", "multi-allelic"], template: "norm -m -any -f {reference} -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["stats", "statistics", "variant statistics"], template: "stats {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "snps only", "select snps"], template: "view -v snps -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["annotate", "add id", "dbsnp"], template: "annotate -a {input2} -c ID -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["isec", "intersection", "shared", "shared between"], template: "isec -p {output_dir} -n=2 {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["query", "extract field", "custom field", "tsv"], template: "query -f '%CHROM\\t%POS\\t%REF\\t%ALT\\t%INFO/DP\\t[%GT\\t]\\n' {input}" },
    ToolTemplate { tool: "samtools", keywords: &["sort", "sorted", "sort by coordinate"], template: "sort -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["index", "bai", "create index"], template: "index {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "properly paired", "primary", "filter bam"], template: "view -b -f 2 -F 256 -F 2048 -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["flagstat", "alignment statistics", "mapped unmapped"], template: "flagstat {input}" },
    ToolTemplate { tool: "samtools", keywords: &["fastq", "bam2fq", "convert bam to fastq", "paired-end fastq"], template: "fastq -1 {output} -2 {input2} -0 /dev/null -s /dev/null -n {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "region", "chromosome", "extract region"], template: "view -b -o {output} {input} {args}" },
    ToolTemplate { tool: "samtools", keywords: &["markdup", "duplicate", "pcr duplicate"], template: "markdup -f {input2} {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["merge", "combine bam", "multiple bam"], template: "merge -f {output} {inputs}" },
    ToolTemplate { tool: "samtools", keywords: &["depth", "per-base depth", "coverage depth"], template: "depth -a -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "header", "bam header"], template: "view -H {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "paired-end", "standard database"], template: "--db {database} --paired --output {output} --report {input2} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["confidence", "unclassified", "save unclassified"], template: "--db {database} --paired --confidence 0.1 --output {output} --report {input2} --unclassified-out {config} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["single-end", "classify single", "report"], template: "--db {database} --output {output} --report {input2} {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["classified-out", "extract classified"], template: "--db {database} --paired --output {output} --report {input2} --classified-out {config} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["kraken2-build", "standard", "build database", "download"], template: "kraken2-build --standard --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["kraken2-build", "custom", "bacteria", "viral"], template: "kraken2-build --download-taxonomy --db {database} && kraken2-build --download-library bacteria --db {database} && kraken2-build --download-library viral --db {database} && kraken2-build --build --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["memory-mapping", "low-ram", "memory"], template: "--db {database} --memory-mapping --paired --output {output} --report {input2} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["quick", "preliminary"], template: "--db {database} --quick --paired --output {output} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["minimum-hit-groups", "stringency", "hit groups"], template: "--db {database} --paired --minimum-hit-groups 3 --confidence 0.1 --output {output} --report {input2} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["mpa-style", "metaphlan", "report format"], template: "--db {database} --paired --report {output} --use-mpa-style {read1} {read2}" },
    ToolTemplate { tool: "centrifuge", keywords: &["classify", "paired-end", "bacterial", "database"], template: "-x {database} -1 {read1} -2 {read2} -S {output} --report-file {input2}" },
    ToolTemplate { tool: "centrifuge", keywords: &["classify", "single-end", "nt"], template: "-x {database} -U {input} -S {output} --report-file {input2}" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge-build", "build", "custom", "index"], template: "centrifuge-build --taxonomy-tree {input} --name-table {input2} --conversion-table {config} {reference} custom_db" },
    ToolTemplate { tool: "centrifuge", keywords: &["viral", "sensitivity", "min-hitlen"], template: "-x {database} -U {input} -S {output} --report-file {input2} --min-hitlen 16" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge-kreport", "kraken", "pavian", "krona"], template: "centrifuge-kreport -x {database} {input}" },
    ToolTemplate { tool: "centrifuge", keywords: &["human", "remove human", "hg38", "un-conc"], template: "-x {database} -1 {read1} -2 {read2} -S {output} --un-conc {config}" },
    ToolTemplate { tool: "centrifuge", keywords: &["unclassified", "save unclassified", "downstream assembly"], template: "-x {database} -1 {read1} -2 {read2} -S {output} --report-file {input2} --un-conc {config}" },
    ToolTemplate { tool: "bismark", keywords: &["genome_preparation", "prepare", "bisulfite genome", "index"], template: "bismark_genome_preparation {genome_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["align", "paired-end", "wgbs", "bisulfite"], template: "--genome {genome_dir} -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["deduplicate", "dedup", "paired-end bam"], template: "deduplicate_bismark --paired --bam {input}" },
    ToolTemplate { tool: "bismark", keywords: &["methylation_extractor", "methylation", "extract methylation"], template: "bismark_methylation_extractor --paired-end --comprehensive --CX_context --genome_folder {genome_dir} --output_dir {output_dir} {input}" },
    ToolTemplate { tool: "bismark", keywords: &["rrbs", "mspi"], template: "--genome {genome_dir} --rrbs -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["hisat2", "single-end", "bisulfite hisat2"], template: "--genome {genome_dir} --hisat2 {input} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["non-directional", "pbat", "scbs-seq"], template: "--genome {genome_dir} --non_directional -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["minimap2", "long-read", "nanopore", "pacbio bisulfite"], template: "--genome {genome_dir} --minimap2 {input} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["slam", "slam-seq", "time-resolved"], template: "--genome {genome_dir} --slam -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["bismark2report", "html report", "alignment report"], template: "bismark2report --output_dir {output_dir}" },
    ToolTemplate { tool: "bracken", keywords: &["bracken-build", "build", "database build"], template: "bracken-build -d {database} -k 35 -l 150 -y kraken2" },
    ToolTemplate { tool: "bracken", keywords: &["bracken-build", "krakenuniq"], template: "bracken-build -d {database} -k 31 -l 100 -y krakenuniq" },
    ToolTemplate { tool: "bracken", keywords: &["species", "abundance", "species-level"], template: "-d {database} -i {input} -o {output} -w {input2} -l S -r 150" },
    ToolTemplate { tool: "bracken", keywords: &["genus", "genus-level"], template: "-d {database} -i {input} -o {output} -l G -r 150" },
    ToolTemplate { tool: "bracken", keywords: &["combine", "merge", "combine_bracken"], template: "combine_bracken_outputs --files {inputs} --names s1,s2,s3 --output {output}" },
    ToolTemplate { tool: "bracken", keywords: &["short", "75", "short reads"], template: "-d {database} -i {input} -o {output} -l S -r 75" },
    ToolTemplate { tool: "bracken", keywords: &["family", "family-level"], template: "-d {database} -i {input} -o {output} -l F -r 150" },
    ToolTemplate { tool: "diamond", keywords: &["makedb", "build database", "make database"], template: "makedb --in {input} -d {database}" },
    ToolTemplate { tool: "diamond", keywords: &["blastp", "protein search", "protein sequence"], template: "blastp -q {input} -d {database} -o {output} --outfmt 6 --evalue 1e-5" },
    ToolTemplate { tool: "diamond", keywords: &["blastx", "dna", "protein database"], template: "blastx -q {input} -d {database} -o {output} --outfmt 6 --evalue 1e-5 --max-target-seqs 1" },
    ToolTemplate { tool: "diamond", keywords: &["more-sensitive", "sensitive", "custom output"], template: "blastp -q {input} -d {database} -o {output} --outfmt '6 qseqid sseqid pident length evalue bitscore stitle' --more-sensitive" },
    ToolTemplate { tool: "diamond", keywords: &["taxonomy", "taxonmap", "taxonnodes"], template: "blastx -q {input} -d {database} --taxonmap {input2} --taxonnodes {config} -o {output} --outfmt '6 qseqid sseqid pident evalue bitscore staxids sscinames'" },
    ToolTemplate { tool: "diamond", keywords: &["ultra-sensitive", "distant homolog"], template: "blastp -q {input} -d {database} -o {output} --outfmt 6 --ultra-sensitive" },
    ToolTemplate { tool: "diamond", keywords: &["memory", "block-size", "large database"], template: "blastp -q {input} -d {database} -o {output} --outfmt 6 --block-size 1 --index-chunks 8" },
    ToolTemplate { tool: "diamond", keywords: &["cluster", "cd-hit"], template: "cluster -d {input} -o {output} --approx-id 50" },
    ToolTemplate { tool: "diamond", keywords: &["linclust", "linear", "very large"], template: "linclust -d {input} -o {output} --approx-id 50" },
    ToolTemplate { tool: "diamond", keywords: &["sam", "sam format", "output sam"], template: "blastx -q {input} -d {database} -o {output} --outfmt 101" },
    ToolTemplate { tool: "pilon", keywords: &["polish", "paired-end", "illumina", "frags"], template: "-Xmx64g -jar {input} --genome {reference} --frags {input2} --output polished --changes" },
    ToolTemplate { tool: "pilon", keywords: &["mate-pair", "jumps", "combined"], template: "-Xmx128g -jar {input} --genome {reference} --frags {input2} --jumps {config} --output polished_v2" },
    ToolTemplate { tool: "pilon", keywords: &["fix bases", "snp", "small indel"], template: "-Xmx64g -jar {input} --genome {reference} --frags {input2} --output polished --fix bases" },
    ToolTemplate { tool: "pilon", keywords: &["variant", "vcf", "generate vcf"], template: "-Xmx64g -jar {input} --genome {reference} --frags {input2} --output variants --variant" },
    ToolTemplate { tool: "pilon", keywords: &["targets", "specific", "unplaced contigs"], template: "-Xmx32g -jar {input} --genome {reference} --frags {input2} --output polished_contigs --targets {config}" },
    ToolTemplate { tool: "multiqc", keywords: &["current directory", "aggregate all", "all qc"], template: ". -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["specific", "directory", "path"], template: "{input_dir} -o {output_dir} -n project_qc_report -f" },
    ToolTemplate { tool: "multiqc", keywords: &["ignore", "exclude", "subdirectory"], template: "{input_dir} --ignore {config} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["flat", "non-interactive", "pdf"], template: ". --flat -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["fastqc", "trimmomatic", "specific tool"], template: "{input_dir} {input2} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["module", "only", "specific module"], template: "{input_dir} -m fastqc -m star -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["exclude", "module", "exclude module"], template: "{input_dir} -e cutadapt -e fastqc -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["sample-names", "rename", "tsv"], template: "{input_dir} --sample-names {input} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["replace-names", "new names"], template: "{input_dir} --replace-names {input} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["json", "data-format", "no-report"], template: "{input_dir} --data-format json --no-report -o {output_dir} -f" },
    ToolTemplate { tool: "fastqc", keywords: &["single", "quality control", "single fastq"], template: "{input} -o {output_dir}" },
    ToolTemplate { tool: "fastqc", keywords: &["paired-end", "threads", "multiple fastq"], template: "-t {threads} -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "fastqc", keywords: &["noextract", "multiple", "keep zip"], template: "--noextract -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["bam", "bam file"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["adapter", "custom adapter", "format"], template: "-f fastq -a {input2} -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["svg", "publication", "graphics"], template: "--svg -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["memory", "long reads", "increased memory"], template: "--memory 1024 -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["contaminant", "custom contaminant"], template: "-c {input2} -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["kmer", "kmer length", "kmer content"], template: "-k 5 -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["casava", "casava output"], template: "--casava -o {output_dir} {inputs}" },
    ToolTemplate { tool: "cutadapt", keywords: &["illumina", "truseq", "paired-end", "adapter"], template: "-a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT -o {output} -p {input2} {read1} {read2}" },
    ToolTemplate { tool: "cutadapt", keywords: &["quality", "minimum-length", "discard short"], template: "-a AGATCGGAAGAGC -A AGATCGGAAGAGC -q 20 --minimum-length 36 -o {output} -p {input2} {read1} {read2}" },
    ToolTemplate { tool: "cutadapt", keywords: &["polya", "polyA", "single-end rna"], template: "-a A{20} -q 20 --minimum-length 30 -o {output} {input}" },
    ToolTemplate { tool: "cutadapt", keywords: &["nextera", "atac-seq", "transposase"], template: "-a CTGTCTCTTATA -A CTGTCTCTTATA -q 20 --minimum-length 20 -o {output} -p {input2} {read1} {read2}" },
    ToolTemplate { tool: "cutadapt", keywords: &["5' primer", "amplicon", "discard-untrimmed"], template: "-g ACACTGACGACATGGTTCTACA --discard-untrimmed -o {output} {input}" },
    ToolTemplate { tool: "cutadapt", keywords: &["nextseq", "nextseq-trim"], template: "-a AGATCGGAAGAGC --nextseq-trim 20 -o {output} {input}" },
    ToolTemplate { tool: "cutadapt", keywords: &["mask", "mask adapter", "instead of trimming"], template: "-a AGATCGGAAGAGC --action mask -o {output} {input}" },
    ToolTemplate { tool: "cutadapt", keywords: &["rc", "reverse complement"], template: "-a AGATCGGAAGAGC --rc -o {output} {input}" },
    ToolTemplate { tool: "fastp", keywords: &["paired-end", "quality trim", "filter"], template: "-i {read1} -I {read2} -o {output} -O {input2} -h {report_html} -j {report_json} -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["single-end", "trim adapter", "filter short"], template: "-i {input} -o {output} -l 50 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["quality", "minimum quality", "q 20"], template: "-i {read1} -I {read2} -o {output} -O {input2} -q 20 -l 36 -w {threads} -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["polya", "polyA", "rna-seq", "trim_poly_a"], template: "-i {read1} -I {read2} -o {output} -O {input2} --trim_poly_a -w {threads} -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["qc only", "no trimming", "disable adapter", "quality control only"], template: "-i {read1} -I {read2} -o /dev/null -O /dev/null --disable_adapter_trimming --disable_quality_filtering -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["merge", "overlapping", "overlapping reads"], template: "-i {read1} -I {read2} --merge --merged_out {output} -o {input2} -O {config} -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["correction", "base correction", "overlap"], template: "-i {read1} -I {read2} -o {output} -O {input2} --correction -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["polyg", "novaseq", "nextseq polyg"], template: "-i {read1} -I {read2} -o {output} -O {input2} --trim_poly_g --poly_g_min_len 10" },
    ToolTemplate { tool: "fastp", keywords: &["sliding window", "cut_front", "cut_tail"], template: "-i {input} -o {output} --cut_front --cut_tail -q 20 -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["dedup", "deduplication", "dup_calc_accuracy"], template: "-i {read1} -I {read2} -o {output} -O {input2} --dedup --dup_calc_accuracy 4 -w {threads}" },
    ToolTemplate { tool: "mosdepth", keywords: &["window", "500", "genome-wide", "by 500"], template: "--by 500 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["target", "bed", "wes", "region"], template: "--by {input2} --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["mapq", "quality filter", "filter"], template: "-Q 20 -F 1796 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["summary", "summary only", "no per-base"], template: "-n --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["threshold", "quantize", "threshold analysis"], template: "--by {input2} -T 1,10,20,30,50 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["fast", "quick", "fast mode"], template: "-x --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["fragment", "chip-seq", "fragment coverage"], template: "-a --by {input2} --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["median", "median coverage"], template: "-m --by {input2} --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["chromosome", "chr", "specific chromosome"], template: "-c chr20 --prefix {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["quantize", "bins", "quantize coverage"], template: "-q 0:1:10:50:100: --prefix {prefix} {input}" },
    ToolTemplate { tool: "prokka", keywords: &["bacteria", "genus", "species", "strain"], template: "--kingdom Bacteria --genus Escherichia --species coli --strain K12 --outdir {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "prokka", keywords: &["metagenome", "mag", "metagenome-assembled"], template: "--metagenome --outdir {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "prokka", keywords: &["archaea", "archaea genome"], template: "--kingdom Archaea --outdir {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "prokka", keywords: &["custom protein", "proteins", "custom database"], template: "--kingdom Bacteria --proteins {input2} --outdir {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "prokka", keywords: &["locustag", "locus tag", "prefix"], template: "--kingdom Bacteria --locustag MYORG --outdir {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "spades", keywords: &["bacterial", "careful", "paired-end"], template: "-1 {read1} -2 {read2} -o {output_dir} --memory 32 --careful" },
    ToolTemplate { tool: "spades", keywords: &["meta", "metagenome", "metaspades"], template: "--meta -1 {read1} -2 {read2} -o {output_dir} --memory 128" },
    ToolTemplate { tool: "spades", keywords: &["plasmid", "plasmidspades"], template: "--plasmid -1 {read1} -2 {read2} -o {output_dir} --memory 16" },
    ToolTemplate { tool: "spades", keywords: &["sc", "single-cell", "mda"], template: "--sc -1 {read1} -2 {read2} -o {output_dir} --memory 32" },
    ToolTemplate { tool: "spades", keywords: &["continue", "resume", "interrupted"], template: "-o {output_dir} --continue" },
    ToolTemplate { tool: "spades", keywords: &["hybrid", "nanopore", "long reads"], template: "-1 {read1} -2 {read2} --nanopore {input2} -o {output_dir} --memory 64" },
    ToolTemplate { tool: "spades", keywords: &["isolate", "isolate mode"], template: "--isolate -1 {read1} -2 {read2} -o {output_dir} --memory 32" },
    ToolTemplate { tool: "spades", keywords: &["rnaviral", "viral rna"], template: "--rnaviral -1 {read1} -2 {read2} -o {output_dir} --memory 16" },
    ToolTemplate { tool: "spades", keywords: &["corona", "coronavirus"], template: "--corona -1 {read1} -2 {read2} -o {output_dir} --memory 16" },
    ToolTemplate { tool: "spades", keywords: &["bio", "biosynthetic", "gene cluster"], template: "--bio -1 {read1} -2 {read2} -o {output_dir} --memory 64" },
    ToolTemplate { tool: "canu", keywords: &["nanopore", "ont", "bacterial genome"], template: "-p {prefix} -d {output_dir} genomeSize=5m -nanopore-raw {input} maxMemory=16g maxThreads=8" },
    ToolTemplate { tool: "canu", keywords: &["pacbio", "hifi", "pacbio-hifi"], template: "-p {prefix} -d {output_dir} genomeSize=3g -pacbio-hifi {input} maxMemory=64g maxThreads=32" },
    ToolTemplate { tool: "canu", keywords: &["metagenome", "ont metagenome"], template: "-p {prefix} -d {output_dir} genomeSize=100m -nanopore-raw {input} maxMemory=128g maxThreads=32 useGrid=false" },
    ToolTemplate { tool: "canu", keywords: &["assemble only", "skip correction", "skip trimming"], template: "-p {prefix} -d {output_dir} -assemble genomeSize=5m -nanopore-corrected {input} maxMemory=16g maxThreads=8" },
    ToolTemplate { tool: "canu", keywords: &["trio", "diploid", "haplotype", "parental"], template: "-p {prefix} -d {output_dir} genomeSize=3g -haplotypeMAT {input2} -haplotypePAT {config} -nanopore-raw {input} maxMemory=256g maxThreads=64" },
    ToolTemplate { tool: "canu", keywords: &["clr", "pacbio-raw", "error rate"], template: "-p {prefix} -d {output_dir} genomeSize=500m -pacbio-raw {input} rawErrorRate=0.350 correctedErrorRate=0.05 maxMemory=64g maxThreads=32" },
    ToolTemplate { tool: "canu", keywords: &["correct only", "correction stage"], template: "-p {prefix} -d {output_dir} -correct genomeSize=5m -nanopore-raw {input} maxMemory=16g maxThreads=8" },
    ToolTemplate { tool: "canu", keywords: &["coverage", "high-depth", "limit coverage"], template: "-p {prefix} -d {output_dir} genomeSize=5m -nanopore-raw {input} maxInputCoverage=100 corOutCoverage=100 maxMemory=16g maxThreads=8" },
    ToolTemplate { tool: "flye", keywords: &["nano-raw", "ont", "bacterial genome"], template: "--nano-raw {input} --genome-size 5m --out-dir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["pacbio-hifi", "hifi", "pacbio"], template: "--pacbio-hifi {input} --genome-size 3g --out-dir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["meta", "metagenomic", "ont metagenome"], template: "--meta --nano-raw {input} --out-dir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["nano-hq", "high-quality", "r10", "q20"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir} --iterations 2" },
    ToolTemplate { tool: "flye", keywords: &["resume", "interrupted"], template: "--nano-raw {input} --genome-size 5m --out-dir {output_dir} --resume" },
    ToolTemplate { tool: "flye", keywords: &["asm-coverage", "large genome", "reduced memory"], template: "--pacbio-hifi {input} --genome-size 3g --out-dir {output_dir} --asm-coverage 40" },
    ToolTemplate { tool: "flye", keywords: &["scaffold", "scaffolding"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir} --scaffold --iterations 2" },
    ToolTemplate { tool: "flye", keywords: &["diploid", "keep-haplotypes"], template: "--pacbio-hifi {input} --genome-size 600m --out-dir {output_dir} --keep-haplotypes --iterations 2" },
    ToolTemplate { tool: "flye", keywords: &["read-error", "custom error rate", "guppy5"], template: "--nano-hq {input} --genome-size 5m --out-dir {output_dir} --read-error 0.05" },
    ToolTemplate { tool: "flye", keywords: &["stop-after", "contigger", "specific stage"], template: "--nano-raw {input} --genome-size 5m --out-dir {output_dir} --stop-after contigger" },
    ToolTemplate { tool: "porechop", keywords: &["trim", "adapter", "nanopore"], template: "-i {input} -o {output}" },
    ToolTemplate { tool: "porechop", keywords: &["discard_middle", "chimeric", "remove chimeric"], template: "-i {input} -o {output} --discard_middle" },
    ToolTemplate { tool: "porechop", keywords: &["demultiplex", "barcode", "separate"], template: "-i {input} -b {output_dir}" },
    ToolTemplate { tool: "porechop", keywords: &["min_split_read_size", "minimum length"], template: "-i {input} -o {output} --min_split_read_size 1000" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["diamond", "annotate protein", "protein fasta"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --override" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["bacterial", "bacteria", "tax_scope"], template: "-m diamond -i {input} --itype proteins --tax_scope 2 --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["nucleotide", "cds", "coding sequence", "fna"], template: "-m diamond -i {input} --itype CDS --translate --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["mmseqs2", "mmseqs", "fast annotation", "metagenomic protein"], template: "-m mmseqs2 -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["resume", "interrupted", "continue"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --resume" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["genome", "gene prediction", "predict genes"], template: "-m diamond -i {input} --itype genome --data_dir {database} -o {prefix} --cpu {threads} --genepred" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["pfam", "domain", "realign", "pfam_realign"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --pfam_realign" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["experimental", "go evidence", "go_evidence"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --go_evidence non-electronic" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["ortholog", "one-to-one", "target_orthologs"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --target_orthologs one2one" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["decorate", "gff", "decorated gff"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --decorate_gff {input2}" },
    ToolTemplate { tool: "medaka", keywords: &["polish", "consensus", "ont assembly", "all-in-one", "pipeline"], template: "medaka_consensus -i {input} -d {reference} -o {output_dir} -m r941_min_hac_g507 -t {threads}" },
    ToolTemplate { tool: "medaka", keywords: &["haploid", "variant", "call variant"], template: "medaka_haploid_variant -i {input} -r {reference} -o {output_dir} -m r941_min_hac_g507" },
    ToolTemplate { tool: "medaka", keywords: &["list", "models", "available model"], template: "tools list_models" },
    ToolTemplate { tool: "medaka", keywords: &["gpu", "gpu acceleration", "cuda"], template: "medaka_consensus -i {input} -d {reference} -o {output_dir} -m r1041_e82_400bps_hac_v4.2.0 --gpu" },
    ToolTemplate { tool: "medaka", keywords: &["targeted", "region", "bed"], template: "medaka_variant -i {input} -r {reference} -o {output_dir} -m r1041_e82_400bps_hac_v4.2.0 --regions {input2}" },
    ToolTemplate { tool: "medaka", keywords: &["reduce memory", "low memory", "large genome", "chunk"], template: "medaka_consensus -i {input} -d {reference} -o {output_dir} -m r1041_e82_400bps_hac_v4.2.0 --chunk_len 5000 --chunk_ovlp 500" },
    ToolTemplate { tool: "medaka", keywords: &["feature", "intermediate", "save feature"], template: "medaka inference --save_features --model r1041_e82_400bps_hac_v4.2.0 {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["inference", "chromosome", "specific chrom"], template: "medaka inference --regions {input2} --model r1041_e82_400bps_hac_v4.2.0 {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["stitch", "consensus", "hdf5", "sequence"], template: "medaka sequence {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["vcf", "diploid", "create vcf"], template: "medaka vcf {input} {output} {reference}" },
    ToolTemplate { tool: "quast", keywords: &["assess", "quality", "reference genome", "evaluate assembly"], template: "-r {reference} -g {input2} {input} -o {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["compare", "multiple assembly", "without reference"], template: "{inputs} -o {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["metaquast", "metagenome", "meta"], template: "metaquast.py -r {reference} {input} -o {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["min-contig", "minimum contig", "contig length filter"], template: "-r {reference} {input} -o {output_dir} --min-contig 1000" },
    ToolTemplate { tool: "quast", keywords: &["gff", "annotation", "gene annotation"], template: "-t {threads} -o {output_dir} -g {input2} {input}" },
    ToolTemplate { tool: "quast", keywords: &["rna", "transcriptome", "rna-seq assembly"], template: "-t {threads} -o {output_dir} --rna {inputs}" },
    ToolTemplate { tool: "quast", keywords: &["eukaryote", "eukaryotic", "large genome"], template: "-t {threads} -o {output_dir} --eukaryote {inputs}" },
    ToolTemplate { tool: "quast", keywords: &["conserved", "busco", "conserved genes"], template: "-t {threads} -o {output_dir} --conserved-genes-finding {inputs}" },
    ToolTemplate { tool: "quast", keywords: &["plots", "icarus", "visualize"], template: "-t {threads} -o {output_dir} --plots {inputs}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["classify", "classify_wf", "taxonomy"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["identify", "identify_wf"], template: "identify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["align", "align_wf"], template: "align_wf --identify_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["de_novo", "de novo_wf", "denovo"], template: "de_novo_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["genes", "gene", "amino acid"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads} --genes" },
    ToolTemplate { tool: "gtdbtk", keywords: &["batchfile", "batch", "custom genome list"], template: "classify_wf --batchfile {input} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["min_perc_aa", "minimum percent", "quality filter"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads} --min_perc_aa 10" },
    ToolTemplate { tool: "gtdbtk", keywords: &["scratch_dir", "temporary", "scratch"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads} --scratch_dir /tmp/gtdbtk" },
    ToolTemplate { tool: "gtdbtk", keywords: &["pplacer_cpus", "placement", "phylogenetic placement"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads} --pplacer_cpus {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["run", "execute", "workflow"], template: "--cores {threads} --use-conda" },
    ToolTemplate { tool: "snakemake", keywords: &["dry-run", "dry run", "plan", "preview"], template: "--cores {threads} --dry-run" },
    ToolTemplate { tool: "snakemake", keywords: &["slurm", "cluster", "executor slurm"], template: "--cores {threads} --executor slurm" },
    ToolTemplate { tool: "snakemake", keywords: &["profile", "configuration profile"], template: "--cores {threads} --profile {args}" },
    ToolTemplate { tool: "snakemake", keywords: &["configfile", "config file", "configuration"], template: "--cores {threads} --configfile {input}" },
    ToolTemplate { tool: "snakemake", keywords: &["forcerun", "force run", "rerun"], template: "--cores {threads} --forcerun {args}" },
    ToolTemplate { tool: "snakemake", keywords: &["unlock", "locked", "unlock directory"], template: "--cores {threads} --unlock" },
    ToolTemplate { tool: "snakemake", keywords: &["dag", "graph", "visualize dag"], template: "--cores {threads} --dag" },
    ToolTemplate { tool: "snakemake", keywords: &["singularity", "container", "docker"], template: "--cores {threads} --use-singularity" },
    ToolTemplate { tool: "snakemake", keywords: &["rerun-incomplete", "incomplete", "restart incomplete"], template: "--cores {threads} --rerun-incomplete" },
    ToolTemplate { tool: "multiqc", keywords: &["aggregate", "current directory", "all qc"], template: ". -o {output_dir}" },
    ToolTemplate { tool: "multiqc", keywords: &["specific", "results directory", "directory"], template: "{input_dir} -o {output_dir}" },
    ToolTemplate { tool: "multiqc", keywords: &["ignore", "exclude directory", "subdirectory"], template: "{input_dir} -o {output_dir} --ignore {args}" },
    ToolTemplate { tool: "multiqc", keywords: &["flat", "non-interactive", "pdf"], template: "{input_dir} -o {output_dir} --flat" },
    ToolTemplate { tool: "multiqc", keywords: &["module", "fastqc", "trimmomatic", "only"], template: "{input_dir} -o {output_dir} -m fastqc trimmomatic" },
    ToolTemplate { tool: "multiqc", keywords: &["star", "specific module"], template: "{input_dir} -o {output_dir} -m fastqc star" },
    ToolTemplate { tool: "multiqc", keywords: &["exclude module", "exclude specific"], template: "{input_dir} -o {output_dir} -e {args}" },
    ToolTemplate { tool: "multiqc", keywords: &["sample-names", "rename sample", "tsv"], template: "{input_dir} -o {output_dir} --sample-names {input}" },
    ToolTemplate { tool: "multiqc", keywords: &["replace-names", "replace name"], template: "{input_dir} -o {output_dir} --replace-names {input}" },
    ToolTemplate { tool: "multiqc", keywords: &["json", "data-format", "export data"], template: "{input_dir} -o {output_dir} --data-format json" },
    ToolTemplate { tool: "meme", keywords: &["fimo", "scan motif", "motif occurrence", "known tf", "tf binding motif"], template: "fimo --thresh 1e-4 --oc {output_dir} {input2} {input}" },
    ToolTemplate { tool: "meme", keywords: &["meme", "discover motif", "de novo motif", "find motif", "chip-seq peak"], template: "-dna -mod zoops -nmotifs 10 -minw 6 -maxw 20 -oc {output_dir} {input}" },
    ToolTemplate { tool: "meme", keywords: &["dreme", "short motif", "discriminative"], template: "dreme -oc {output_dir} -p {input}" },
    ToolTemplate { tool: "meme", keywords: &["ame", "enrichment", "motif enrichment", "foreground", "background"], template: "ame --oc {output_dir} --control {input2} {input}" },
    ToolTemplate { tool: "meme", keywords: &["streme", "streme motif", "fast short motif"], template: "streme --oc {output_dir} --dna --p {input} --n {input2}" },
    ToolTemplate { tool: "meme", keywords: &["tomtom", "compare motif", "motif similarity", "known database"], template: "tomtom -oc {output_dir} {input} {input2}" },
    ToolTemplate { tool: "meme", keywords: &["meme-chip", "chip-seq motif", "chip"], template: "meme-chip -oc {output_dir} -db {input2} {input}" },
    ToolTemplate { tool: "meme", keywords: &["revcomp", "reverse complement"], template: "meme -oc {output_dir} -revcomp -mod zoops -nmotifs 5 {input}" },
    ToolTemplate { tool: "bowtie2", keywords: &["build", "index", "bowtie2-build"], template: "bowtie2-build {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["build", "threads", "large genome"], template: "bowtie2-build --threads {threads} {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["paired-end", "paired", "align paired", "-1", "-2"], template: "-x {index} -1 {read1} -2 {read2} -p {threads}" },
    ToolTemplate { tool: "bowtie2", keywords: &["single-end", "sensitive", "very-sensitive", "-U"], template: "-x {index} -U {read1} --very-sensitive" },
    ToolTemplate { tool: "bowtie2", keywords: &["no-unal", "unaligned", "discard unaligned", "save aligned"], template: "-x {index} -1 {read1} -2 {read2} --no-unal -S {output}" },
    ToolTemplate { tool: "bowtie2", keywords: &["rg", "read group", "gatk", "rg-id"], template: "-x {index} -1 {read1} -2 {read2} --rg-id sample1 --rg SM:sample1 --rg LB:lib1 --rg PL:ILLUMINA" },
    ToolTemplate { tool: "bowtie2", keywords: &["local", "soft-clip", "very-sensitive-local"], template: "-x {index} -1 {read1} -2 {read2} --local --very-sensitive-local" },
    ToolTemplate { tool: "bowtie2", keywords: &["fast", "quick", "quality check"], template: "-x {index} -U {read1} --fast -S {output}" },
    ToolTemplate { tool: "bowtie2", keywords: &["un-conc", "unmapped", "separate file"], template: "-x {index} -1 {read1} -2 {read2} --un-conc {output}" },
    ToolTemplate { tool: "star", keywords: &["genomegenerate", "generate genome", "genome index", "build index"], template: "--runMode genomeGenerate --runThreadN {threads} --genomeDir {genome_dir} --genomeFastaFiles {reference} --sjdbGTFfile {annotation}" },
    ToolTemplate { tool: "star", keywords: &["align", "paired-end", "gzipped", "zcat"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["two-pass", "twopass", "junction"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {input} --twopassMode Basic --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["unmapped", "fastq", "output unmapped"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outReadsUnmapped Fastx" },
    ToolTemplate { tool: "star", keywords: &["quant", "gene count", "quantification", "differential expression"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --quantMode GeneCounts" },
    ToolTemplate { tool: "star", keywords: &["solo", "single-cell", "10x", "starsolo", "cellranger"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --soloType CB_UMI_Simple --soloCBstart 1 --soloCBlen 16 --soloUMIstart 17 --soloUMIlen 10 --soloCBwhitelist {input2} --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["chimeric", "fusion", "arriba"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --readFilesCommand zcat --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --chimSegmentMin 10 --chimOutType WithinBAM --chimJunctionOverhangMin 10 --chimScoreDropMax 30 --peOverlapNbasesMin 12" },
    ToolTemplate { tool: "star", keywords: &["shared memory", "load genome", "genomeLoad"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --genomeLoad LoadAndKeep --readFilesIn {input} --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "star", keywords: &["unique", "no multi", "unique-only", "outMultimapperOrder"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outMultimapperOrder Random --outFilterMultimapNmax 1" },
    ToolTemplate { tool: "star", keywords: &["mismatch", "strict", "high specificity", "filter"], template: "--runMode alignReads --runThreadN {threads} --genomeDir {genome_dir} --readFilesIn {read1} {read2} --outSAMtype BAM SortedByCoordinate --outFileNamePrefix {output_prefix} --outFilterMismatchNmax 3 --outFilterMismatchNoverLmax 0.04" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "paired-end", "standard database", "paired"], template: "--db {database} --paired --output {output} --report {input2} --threads {threads} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["confidence", "unclassified", "save unclassified"], template: "--db {database} --paired --output {output} --report {input2} --threads {threads} --confidence 0.1 --unclassified-out {args} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["single-end", "single", "classify single"], template: "--db {database} --output {output} --report {input2} --threads {threads} {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["classified", "extract classified", "output classified"], template: "--db {database} --paired --output {output} --report {input2} --threads {threads} --classified-out {args} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["kraken2-build", "standard", "build database", "download"], template: "kraken2-build --standard --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["custom", "build custom", "library"], template: "kraken2-build --add-to-library {input} --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["memory-mapping", "low-ram", "memory map"], template: "--db {database} --paired --output {output} --report {input2} --threads {threads} --memory-mapping {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["quick", "preliminary", "quick mode"], template: "--db {database} --output {output} --report {input2} --threads {threads} --quick {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["minimum hit", "hit groups", "stringency"], template: "--db {database} --paired --output {output} --report {input2} --threads {threads} --minimum-hit-groups 3 {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["metaphlan", "report format", "mpa"], template: "--db {database} --output {output} --report {input2} --threads {threads} --report-minimizer-data {input}" },
    ToolTemplate { tool: "centrifuge", keywords: &["classify", "paired-end", "bacterial", "viral"], template: "-x {database} -1 {read1} -2 {read2} -S {output} --report-file {input2}" },
    ToolTemplate { tool: "centrifuge", keywords: &["single-end", "nt", "classify single"], template: "-x {database} -U {input} -S {output} --report-file {input2}" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge-build", "build", "custom index", "taxonomy"], template: "centrifuge-build --taxonomy-tree {input} --name-table {input2} --conversion-table {args} {reference} {database}" },
    ToolTemplate { tool: "centrifuge", keywords: &["sensitivity", "viral", "min-hitlen"], template: "-x {database} -U {input} -S {output} --report-file {input2} --min-hitlen 16" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge-kreport", "kraken", "kreport", "pavian"], template: "centrifuge-kreport -x {database} {input}" },
    ToolTemplate { tool: "centrifuge", keywords: &["human", "host", "remove host", "un-conc"], template: "-x {database} -1 {read1} -2 {read2} -S {output} --un-conc {args}" },
    ToolTemplate { tool: "centrifuge", keywords: &["unclassified", "save unclassified", "host-depleted"], template: "-x {database} -1 {read1} -2 {read2} -S {output} --report-file {input2} --un-conc {args}" },
    ToolTemplate { tool: "centrifuge", keywords: &["precision", "high specificity", "metagenomic"], template: "-x {database} -U {input} -S {output} --report-file {input2} --min-hitlen 22" },
    ToolTemplate { tool: "blast", keywords: &["makeblastdb", "build database", "create database", "blast database"], template: "makeblastdb -in {input} -dbtype nucl -out {database} -parse_seqids" },
    ToolTemplate { tool: "blast", keywords: &["blastn", "nucleotide search", "similar sequence", "find similar"], template: "blastn -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "blast", keywords: &["blastp", "protein search", "protein database"], template: "blastp -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "blast", keywords: &["blastx", "nucleotide protein", "translate search", "annotate nucleotide"], template: "blastx -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "blast", keywords: &["remote", "ncbi", "online"], template: "blastn -query {input} -db nr -out {output} -outfmt 6 -remote" },
    ToolTemplate { tool: "blast", keywords: &["subject", "without database", "fasta file"], template: "blastn -query {input} -subject {input2} -out {output} -outfmt 6 -evalue 1e-5" },
    ToolTemplate { tool: "blast", keywords: &["distant homolog", "traditional blastn", "task blastn"], template: "blastn -task blastn -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-10 -word_size 11" },
    ToolTemplate { tool: "blast", keywords: &["short sequence", "blastn-short", "short query"], template: "blastn -task blastn-short -query {input} -db {database} -out {output} -outfmt 6 -evalue 1000 -word_size 7" },
    ToolTemplate { tool: "blast", keywords: &["blastdbcmd", "retrieve sequence", "accession"], template: "blastdbcmd -db {database} -entry {args} -out {output}" },
    ToolTemplate { tool: "blast", keywords: &["taxonomy", "taxid", "filter taxonomy"], template: "blastn -query {input} -db nt -out {output} -outfmt 6 -taxids {args} -evalue 1e-5 -remote" },
    ToolTemplate { tool: "bismark", keywords: &["genome_preparation", "prepare genome", "bisulfite genome", "genome index"], template: "bismark_genome_preparation {genome_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["paired-end", "wgbs", "align paired", "bisulfite align"], template: "--genome {genome_dir} -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["deduplicate", "remove duplicate", "dedup"], template: "deduplicate_bismark --paired --bam {input}" },
    ToolTemplate { tool: "bismark", keywords: &["methylation_extractor", "extract methylation", "methylation"], template: "bismark_methylation_extractor --paired-end --comprehensive --CX_context --genome_folder {genome_dir} --output_dir {output_dir} {input}" },
    ToolTemplate { tool: "bismark", keywords: &["rrbs", "mspi"], template: "--genome {genome_dir} --rrbs -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["hisat2", "bisulfite hisat2"], template: "--genome {genome_dir} --hisat2 {input} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["non_directional", "pbat", "scbs"], template: "--genome {genome_dir} --non_directional -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["minimap2", "nanopore", "long-read", "pacbio"], template: "--genome {genome_dir} --minimap2 {input} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["slam", "slam-seq", "time-resolved"], template: "--genome {genome_dir} --slam -1 {read1} -2 {read2} --output_dir {output_dir}" },
    ToolTemplate { tool: "bismark", keywords: &["bismark2report", "html report", "alignment report"], template: "bismark2report --output_dir {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["reference", "assess quality", "reference genome", "gff"], template: "-r {reference} -g {annotation} {input} -o {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["multiple", "compare assemblies", "without reference"], template: "{inputs} -o {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["metaquast", "metagenome", "metaquast.py"], template: "metaquast.py -r {reference} {input} -o {output_dir}" },
    ToolTemplate { tool: "quast", keywords: &["min-contig", "minimum contig", "contig length"], template: "-r {reference} {input} -o {output_dir} --min-contig 1000" },
    ToolTemplate { tool: "bakta", keywords: &["annotate", "bacterial genome", "annotation"], template: "--db {database} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["compliant", "ncbi", "locus-tag", "submission"], template: "--db {database} --compliant --locus-tag MYORG --genus Escherichia --species coli --strain K12 --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["plasmid", "complete"], template: "--db {database} --plasmid pMYPLASMID --complete --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["download", "bakta_db", "database download"], template: "bakta_db download --output {database}" },
    ToolTemplate { tool: "bakta", keywords: &["trusted", "proteins", "hmms", "custom"], template: "--db {database} --proteins {input2} --hmms {hmm} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["mag", "metagenome-assembled", "meta"], template: "--db {database} --meta --translation-table 11 --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["regions", "pre-annotated", "gff3"], template: "--db {database} --regions {annotation} --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["gram", "gram-positive", "signal peptide"], template: "--db {database} --gram + --genus Bacillus --species subtilis --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["skip", "crispr", "sorf", "minimal"], template: "--db {database} --skip-crispr --skip-sorf --output {output_dir} --prefix {prefix} {input}" },
    ToolTemplate { tool: "bakta", keywords: &["bakta_proteins", "proteins directly", "protein fasta"], template: "bakta_proteins --db {database} --output {output_dir} {input}" },
    ToolTemplate { tool: "medaka", keywords: &["consensus", "polish", "all-in-one"], template: "medaka_consensus -i {input} -d {reference} -o {output_dir} -m r941_min_hac_g507" },
    ToolTemplate { tool: "medaka", keywords: &["haploid", "variant", "haploid variant"], template: "medaka_haploid_variant -i {input} -r {reference} -o {output_dir} -m r941_min_hac_g507" },
    ToolTemplate { tool: "medaka", keywords: &["list", "models", "available model"], template: "tools list_models" },
    ToolTemplate { tool: "medaka", keywords: &["gpu", "acceleration"], template: "medaka_consensus -i {input} -d {reference} -o {output_dir} -m r1041_e82_400bps_hac_v4.2.0 --gpu" },
    ToolTemplate { tool: "medaka", keywords: &["targeted", "region", "bed"], template: "medaka_variant -i {input} -r {reference} -o {output_dir} -m r1041_e82_400bps_hac_v4.2.0 --regions {annotation}" },
    ToolTemplate { tool: "medaka", keywords: &["low memory", "chunk", "reduce memory"], template: "medaka_consensus -i {input} -d {reference} -o {output_dir} -m r1041_e82_400bps_hac_v4.2.0 --chunk_len 5000 --chunk_ovlp 500" },
    ToolTemplate { tool: "medaka", keywords: &["inference", "save feature", "features"], template: "medaka inference --save_features --model r1041_e82_400bps_hac_v4.2.0 {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["inference", "chromosome", "regions chr"], template: "medaka inference --regions chr1 chr2 chr3 --model r1041_e82_400bps_hac_v4.2.0 {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["stitch", "sequence", "consensus from"], template: "medaka sequence {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["vcf", "diploid"], template: "medaka vcf {input} {output} {reference}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["protein", "diamond", "annotate protein"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["bacterial", "tax_scope", "restrict"], template: "-m diamond -i {input} --itype proteins --tax_scope 2 --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["cds", "nucleotide", "coding sequence", "fna", "translate"], template: "-m diamond -i {input} --itype CDS --translate --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["mmseqs", "mmseqs2", "fast annotation"], template: "-m mmseqs -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["resume", "interrupted"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --resume" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["genome", "gene prediction", "prodigal"], template: "-m diamond -i {input} --itype genome --data_dir {database} -o {prefix} --cpu {threads} --genepred prodigal" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["pfam", "realign", "domain"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --pfam_realign realign" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["go", "experimental", "go_evidence"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --go_evidence experimental" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["one-to-one", "ortholog", "target_orthologs"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --target_orthologs one2one" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["decorate", "gff", "decorated gff"], template: "-m diamond -i {input} --itype proteins --data_dir {database} -o {prefix} --cpu {threads} --decorate_gff yes" },
    ToolTemplate { tool: "multiqc", keywords: &["aggregate", "current directory", "all qc"], template: ". -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["specific directory", "results directory"], template: "{input_dir} -o {output_dir} -n {prefix} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["ignore", "subdirectory", "exclude dir"], template: "{input_dir} --ignore {input2} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["flat", "non-interactive", "pdf"], template: ". --flat -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["fastqc", "trimmomatic", "specific tool"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["module", "-m", "specific module"], template: "results/ -m fastqc -m star -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["exclude", "-e", "exclude module"], template: "results/ -e cutadapt -e fastqc -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["sample-names", "rename", "tsv"], template: "results/ --sample-names {input} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["replace-names", "replace name"], template: "results/ --replace-names {input} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["json", "data-format", "export data", "no-report"], template: "results/ --data-format json --no-report -o {output_dir} -f" },
    ToolTemplate { tool: "snakemake", keywords: &["cores", "all available", "use-conda"], template: "--cores all --use-conda" },
    ToolTemplate { tool: "snakemake", keywords: &["dry-run", "dry run", "printshellcmds"], template: "--dry-run --printshellcmds" },
    ToolTemplate { tool: "snakemake", keywords: &["slurm", "cluster", "executor"], template: "--executor slurm --jobs 50 --default-resources mem_mb=4096 runtime=60 --use-conda" },
    ToolTemplate { tool: "snakemake", keywords: &["configfile", "configuration file", "yaml"], template: "--configfile {input}" },
    ToolTemplate { tool: "snakemake", keywords: &["profile", "named profile"], template: "--profile slurm" },
    ToolTemplate { tool: "snakemake", keywords: &["forcerun", "force re-run"], template: "--forcerun trimming alignment" },
    ToolTemplate { tool: "snakemake", keywords: &["unlock", "crash"], template: "--unlock" },
    ToolTemplate { tool: "snakemake", keywords: &["dag", "dependency graph"], template: "--dag" },
    ToolTemplate { tool: "snakemake", keywords: &["rerun-incomplete", "incomplete"], template: "--rerun-incomplete --cores all" },
    ToolTemplate { tool: "snakemake", keywords: &["singularity", "container"], template: "--use-singularity --singularity-args '--bind /scratch'" },
    ToolTemplate { tool: "samtools", keywords: &["sort", "sorted", "sort by coordinate"], template: "sort -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["index", "bai", "indexing"], template: "index {input}" },
    ToolTemplate { tool: "samtools", keywords: &["flagstat", "flag statistics", "alignment statistics"], template: "flagstat {input}" },
    ToolTemplate { tool: "samtools", keywords: &["fastq", "bam2fq", "convert bam"], template: "fastq -1 {read1} -2 {read2} -0 /dev/null -s /dev/null -n {input}" },
    ToolTemplate { tool: "samtools", keywords: &["extract", "region", "chromosome", "chr1"], template: "view -b -o {output} {input} {args}" },
    ToolTemplate { tool: "samtools", keywords: &["markdup", "duplicate", "pcr duplicate"], template: "markdup -f {metrics} {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["merge", "combine bam", "multiple bam"], template: "merge -f {outputs} {inputs}" },
    ToolTemplate { tool: "samtools", keywords: &["depth", "coverage", "per-base"], template: "depth -a -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["header", "view header", "bam header"], template: "view -H {input}" },
    ToolTemplate { tool: "samtools", keywords: &["filter", "properly paired", "primary alignment"], template: "view -b -f 2 -F 256 -F 2048 -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["mpileup", "pileup", "call variant", "genotype likelihood"], template: "mpileup -f {reference} -Ou {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["filter", "qual", "snp", "high-quality"], template: "view -i 'QUAL>30 && INFO/DP>10 && TYPE=\"snp\"' -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["merge", "combine vcf", "multiple vcf"], template: "merge -O z -o {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["sample", "extract sample", "specific sample"], template: "view -s {args} -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["norm", "normalize", "split multi-allelic", "left-align"], template: "norm -m -any -f {reference} -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["stats", "statistics", "variant statistics"], template: "stats {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["select snp", "snp only", "-v snps"], template: "view -v snps -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["annotate", "add id", "dbsnp", "annotation"], template: "annotate -a {input2} -c ID -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["isec", "intersection", "shared variant"], template: "isec -p {output_dir} -n=2 {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["query", "extract field", "custom field", "tsv"], template: "query -f '%CHROM\\t%POS\\t%REF\\t%ALT\\t%INFO/DP\\t[%GT\\t]\\n' {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["intersect", "overlap", "find overlap"], template: "intersect -a {input} -b {input2} -wa" },
    ToolTemplate { tool: "bedtools", keywords: &["subtract", "remove region"], template: "subtract -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["merge", "merge interval", "overlapping interval"], template: "merge -i {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["genomecov", "coverage bam", "bedgraph", "per-base coverage"], template: "genomecov -ibam {input} -bg" },
    ToolTemplate { tool: "bedtools", keywords: &["closest", "nearest feature", "non-overlapping"], template: "closest -a {input} -b {input2} -d" },
    ToolTemplate { tool: "bedtools", keywords: &["count overlap", "-c", "overlap count"], template: "intersect -a {input} -b {input2} -c" },
    ToolTemplate { tool: "bedtools", keywords: &["getfasta", "sequence", "extract sequence"], template: "getfasta -fi {reference} -bed {input} -fo {output} -s" },
    ToolTemplate { tool: "bedtools", keywords: &["bga", "zero coverage", "genomecov bga"], template: "genomecov -ibam {input} -bga" },
    ToolTemplate { tool: "bedtools", keywords: &["-wb", "original interval", "report both"], template: "intersect -a {input} -b {input2} -wb" },
    ToolTemplate { tool: "bedtools", keywords: &["makewindows", "window", "fixed size"], template: "makewindows -g {input} -w 1000" },
    ToolTemplate { tool: "gatk", keywords: &["haplotypecaller", "gvcf", "germline variant", "erc gvcf"], template: "HaplotypeCaller -R {reference} -I {input} -O {output} -ERC GVCF" },
    ToolTemplate { tool: "gatk", keywords: &["haplotypecaller", "genotype", "single sample", "not gvcf"], template: "HaplotypeCaller -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["markduplicate", "markdup", "pcr duplicate"], template: "MarkDuplicates -I {input} -O {output} -M {metrics}" },
    ToolTemplate { tool: "gatk", keywords: &["mutect2", "somatic", "matched normal"], template: "Mutect2 -R {reference} -I {input} -I {input2} -normal {args} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["filtermutectcalls", "filter mutect", "filter somatic"], template: "FilterMutectCalls -R {reference} -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["createsequencedictionary", "sequence dictionary", "dict"], template: "CreateSequenceDictionary -R {reference}" },
    ToolTemplate { tool: "gatk", keywords: &["addreplacereadgroups", "read group", "rgid"], template: "AddOrReplaceReadGroups -I {input} -O {output} -RGID sample1 -RGLB lib1 -RGPL ILLUMINA -RGPU unit1 -RGSM sample1" },
    ToolTemplate { tool: "gatk", keywords: &["baserecalibrator", "bqsr", "recalibrat", "known-sites"], template: "BaseRecalibrator -R {reference} -I {input} --known-sites {input2} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["applybqsr", "apply bqsr", "recalibrated"], template: "ApplyBQSR -R {reference} -I {input} --bqsr-recal-file {input2} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["selectvariants", "select variant", "snp only"], template: "SelectVariants -V {input} -O {output} --select-type-to-include SNP" },
    ToolTemplate { tool: "hisat2", keywords: &["hisat2-build", "build index", "genome index"], template: "hisat2-build {reference} {index}" },
    ToolTemplate { tool: "hisat2", keywords: &["hisat2-build", "splice-site", "ss", "exon"], template: "hisat2-build {reference} {index} --ss {annotation} --exon {input2}" },
    ToolTemplate { tool: "hisat2", keywords: &["paired-end", "dta", "rna-seq", "align paired"], template: "-p {threads} -x {index} -1 {read1} -2 {read2} --dta -S {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["strand-specific", "rna-strandness", "rf"], template: "-x {index} -1 {read1} -2 {read2} --rna-strandness RF --dta -S {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["single-end", "single", "align single"], template: "-x {index} -U {read1} --dta -S {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["no-spliced", "genomic", "dna-seq"], template: "-x {index} -U {read1} --no-spliced-alignment -S {output}" },
    ToolTemplate { tool: "hisat2", keywords: &["no-unal", "discard unmapped"], template: "-x {index} -1 {read1} -2 {read2} --dta --no-unal -S {output}" },
    ToolTemplate { tool: "fastp", keywords: &["paired-end", "quality trim", "filter paired"], template: "-i {read1} -I {read2} -o {out1} -O {out2} -h {report_html} -j {report_json} -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["single-end", "adapter", "short read"], template: "-i {input} -o {output} -l 50 -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["quality", "minimum quality", "-q"], template: "-i {read1} -I {read2} -o {out1} -O {out2} -q 20 -l 36 -w {threads} -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["polya", "poly-a", "rna-seq trim"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --trim_poly_a -w {threads} -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["qc only", "disable trimming", "quality control only"], template: "-i {read1} -I {read2} -o /dev/null -O /dev/null --disable_adapter_trimming --disable_quality_filtering -h {report_html} -j {report_json}" },
    ToolTemplate { tool: "fastp", keywords: &["merge", "overlapping", "merged_out"], template: "-i {read1} -I {read2} --merge --merged_out {output} -o {out1} -O {out2} -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["correction", "base correction"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --correction -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["polyg", "poly-g", "novaseq"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --trim_poly_g --poly_g_min_len 10" },
    ToolTemplate { tool: "fastp", keywords: &["sliding window", "cut_front", "cut_tail"], template: "-i {input} -o {output} --cut_front --cut_tail -q 20 -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["dedup", "deduplication", "dup_calc_accuracy"], template: "-i {read1} -I {read2} -o {out1} -O {out2} --dedup --dup_calc_accuracy 4 -w {threads}" },
    ToolTemplate { tool: "fastqc", keywords: &["single", "quality control", "qc"], template: "{input} -o {output_dir}" },
    ToolTemplate { tool: "fastqc", keywords: &["paired-end", "multiple", "threads"], template: "-t {threads} -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "fastqc", keywords: &["noextract", "keep zip", "multiple samples"], template: "--noextract -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["bam", "bam file"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["adapter", "custom adapter", "contaminant"], template: "-f fastq -a {input2} -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["svg", "publication"], template: "--svg -o {output_dir} {inputs}" },
    ToolTemplate { tool: "fastqc", keywords: &["memory", "long read"], template: "--memory 1024 -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["contaminant", "custom contaminant", "-c"], template: "-c {input2} -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["kmer", "kmer length", "-k"], template: "-k 5 -o {output_dir} {input}" },
    ToolTemplate { tool: "fastqc", keywords: &["casava"], template: "--casava -o {output_dir} {inputs}" },
    ToolTemplate { tool: "macs2", keywords: &["callpeak", "broad", "broadpeak", "broad peak"], template: "callpeak -t {input} -c {control} -n {prefix} --broad -g hs" },
    ToolTemplate { tool: "macs2", keywords: &["callpeak", "narrow", "narrowpeak", "factor", "tf"], template: "callpeak -t {input} -c {control} -n {prefix} -g hs -q 0.05" },
    ToolTemplate { tool: "macs2", keywords: &["callpeak", "nomodel", "shift", "no model"], template: "callpeak -t {input} -c {control} -n {prefix} --nomodel --shift -100 --extsize 200 -g hs" },
    ToolTemplate { tool: "macs2", keywords: &["bdgcmp", "fold change", "log2ratio", "bedgraph"], template: "bdgcmp -t {input} -c {control} -m FE -o {output}" },
    ToolTemplate { tool: "macs2", keywords: &["bdgopt", "normalize", "scale"], template: "bdgopt -i {input} -m multiply -p 1.0 -o {output}" },
    ToolTemplate { tool: "macs2", keywords: &["filterdup", "remove duplicate"], template: "filterdup -i {input} -o {output}" },
    ToolTemplate { tool: "macs2", keywords: &["predictd", "fragment size", "cross-correlation"], template: "predictd -i {input} -g hs" },
    ToolTemplate { tool: "macs2", keywords: &["pileup", "bedgraph pileup"], template: "pileup -i {input} -o {output}" },
    ToolTemplate { tool: "macs2", keywords: &["randsample", "subsample"], template: "randsample -i {input} -p 50 -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotcoverage", "coverage plot"], template: "plotCoverage -b {inputs} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["computematrix", "scale-regions", "scale region"], template: "computeMatrix scale-regions -S {input} -R {regions} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotcorrelation", "correlation plot", "scatter"], template: "plotCorrelation -in {input} --corMethod spearman --skipZeros --plotScatter -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotpca", "pca plot"], template: "plotPCA -in {input} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["bamcoverage", "rpkm", "cpm"], template: "bamCoverage -b {input} -o {output} --normalizeUsing RPKM --numberOfProcessors 4" },
    ToolTemplate { tool: "deeptools", keywords: &["bamcoverage", "bins", "bin size"], template: "bamCoverage -b {input} -o {output} --binSize 10 --numberOfProcessors 4" },
    ToolTemplate { tool: "deeptools", keywords: &["plottss", "tss", "transcription start"], template: "computeMatrix reference-point -S {input} -R {regions} --referencePoint TSS -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["alignmentsieve", "filter bam", "bam filter"], template: "alignmentSieve -b {input} -o {output} --minMappingQuality 30" },
    ToolTemplate { tool: "deeptools", keywords: &["estimateinsertsize", "fragment size", "insert size"], template: "estimateInsertSize -b {inputs} -o {output}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "single-end", "single"], template: "--db {database} {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "paired-end", "paired", "-1", "-2"], template: "--db {database} -1 {read1} -2 {read2} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "confidence"], template: "--db {database} --confidence 0.1 {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "use-names", "scientific name"], template: "--db {database} --use-names {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "gzip", "compressed"], template: "--db {database} --gzip-compressed {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "report-minimizer"], template: "--db {database} --report-minimizer-data {input} --output {output} --report {output2}" },
    ToolTemplate { tool: "kraken2", keywords: &["bracken", "abundance"], template: "bracken -d {database} -i {input} -o {output} -l S" },
    ToolTemplate { tool: "kraken2", keywords: &["build", "kraken2-build", "add-to-library"], template: "kraken2-build --db {database} --add-to-library {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["build", "download"], template: "kraken2-build --db {database} --download-library bacteria" },
    ToolTemplate { tool: "ssh", keywords: &["connect", "remote", "user@host"], template: "{args}" },
    ToolTemplate { tool: "ssh", keywords: &["key", "identity", "-i"], template: "-i {input} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["port", "-p"], template: "-p {args} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["forward", "local port", "-L"], template: "-L {args} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["command", "run command", "remote command"], template: "{args} '{args}'" },
    ToolTemplate { tool: "ssh", keywords: &["x11", "forwarding", "-X"], template: "-X {args}" },
    ToolTemplate { tool: "ssh", keywords: &["reverse", "reverse port", "-R"], template: "-R {args} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["socks", "proxy", "-D"], template: "-D {args} -N {args}" },
    ToolTemplate { tool: "ssh", keywords: &["keep-alive", "alive", "ServerAlive"], template: "-o ServerAliveInterval=60 -o ServerAliveCountMax=3 {args}" },
    ToolTemplate { tool: "ssh", keywords: &["jump", "bastion", "-J"], template: "-J {args} {args}" },
    ToolTemplate { tool: "curl", keywords: &["download", "save", "-O", "-o"], template: "-L -o {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["post", "json", "-X POST"], template: "-X POST -H 'Content-Type: application/json' -d '{args}' {url}" },
    ToolTemplate { tool: "curl", keywords: &["auth", "bearer", "token"], template: "-H 'Authorization: Bearer TOKEN' {url}" },
    ToolTemplate { tool: "curl", keywords: &["resume", "continue", "-C"], template: "-L -C - -O {url}" },
    ToolTemplate { tool: "curl", keywords: &["header", "headers", "-I"], template: "-I {url}" },
    ToolTemplate { tool: "curl", keywords: &["upload", "multipart", "-F"], template: "-X POST -F 'file=@{input}' {url}" },
    ToolTemplate { tool: "curl", keywords: &["progress", "bar"], template: "-L --progress-bar -o {output} {url}" },
    ToolTemplate { tool: "curl", keywords: &["timeout", "retry"], template: "--connect-timeout 10 --retry 3 --retry-delay 5 -L -O {url}" },
    ToolTemplate { tool: "curl", keywords: &["basic auth", "user", "-u"], template: "-u {args} {url}" },
    ToolTemplate { tool: "wget", keywords: &["download", "save", "-O"], template: "-O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["resume", "continue", "-c"], template: "-c {url}" },
    ToolTemplate { tool: "wget", keywords: &["background", "daemon", "-b"], template: "-b -q {url}" },
    ToolTemplate { tool: "wget", keywords: &["retry", "timeout", "tries"], template: "--tries=5 --timeout=30 --wait=2 {url}" },
    ToolTemplate { tool: "wget", keywords: &["mirror", "recursive", "website"], template: "-r -l 2 -np -P {output_dir} {url}" },
    ToolTemplate { tool: "wget", keywords: &["list", "input file", "-i"], template: "-i {input} -P {output_dir}" },
    ToolTemplate { tool: "wget", keywords: &["post", "post-data"], template: "--post-data='{args}' -O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["user-agent", "agent"], template: "--user-agent='Mozilla/5.0' -O {output} {url}" },
    ToolTemplate { tool: "find", keywords: &["size", "large", "big"], template: ". -type f -size +100M" },
    ToolTemplate { tool: "find", keywords: &["name", "pattern", "filename"], template: ". -name '{pattern}'" },
    ToolTemplate { tool: "find", keywords: &["delete", "remove", "clean"], template: ". -name '{pattern}' -type f -delete" },
    ToolTemplate { tool: "find", keywords: &["case-insensitive", "iname"], template: ". -iname '{pattern}'" },
    ToolTemplate { tool: "find", keywords: &["directory", "dir", "type d"], template: ". -maxdepth 1 -type d" },
    ToolTemplate { tool: "find", keywords: &["empty"], template: ". -empty" },
    ToolTemplate { tool: "find", keywords: &["exec", "execute", "run command"], template: ". -name '{pattern}' -exec {args} {} \\;" },
    ToolTemplate { tool: "find", keywords: &["user", "owner"], template: "/home -user {args} -type f" },
    ToolTemplate { tool: "find", keywords: &["newer", "modified", "recent"], template: ". -type f -newer {input}" },
    ToolTemplate { tool: "find", keywords: &["permission", "writable", "perm"], template: ". -type f -perm /o+w" },
    ToolTemplate { tool: "rm", keywords: &["single file", "remove file"], template: "{input}" },
    ToolTemplate { tool: "rm", keywords: &["pattern", "glob", "wildcard"], template: "-v {args}" },
    ToolTemplate { tool: "rm", keywords: &["directory", "recursive", "dir"], template: "-r {input}" },
    ToolTemplate { tool: "rm", keywords: &["interactive", "confirm", "-i"], template: "-i {input}" },
    ToolTemplate { tool: "rm", keywords: &["force", "force-remove", "without prompt"], template: "-rf {input}" },
    ToolTemplate { tool: "rm", keywords: &["empty directory", "-d"], template: "-d {input}" },
    ToolTemplate { tool: "rm", keywords: &["symlink", "symbolic link"], template: "{input}" },
    ToolTemplate { tool: "rm", keywords: &["verbose", "-v"], template: "-v {args}" },
    ToolTemplate { tool: "tar", keywords: &["create", "compress", "gzip", "archive"], template: "-czf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["extract", "decompress", "unzip"], template: "-xzf {input}" },
    ToolTemplate { tool: "tar", keywords: &["extract", "directory", "-C"], template: "-xf {input} -C {output_dir}" },
    ToolTemplate { tool: "tar", keywords: &["list", "contents", "-t"], template: "-tf {input}" },
    ToolTemplate { tool: "tar", keywords: &["bzip2", "bz2", "verbose"], template: "-cjvf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["strip", "strip-components"], template: "-xzf {input} --strip-components=1 -C {output_dir}" },
    ToolTemplate { tool: "tar", keywords: &["exclude"], template: "-czf {output} {input} --exclude='{pattern}'" },
    ToolTemplate { tool: "tar", keywords: &["append", "add", "-r"], template: "-rf {input} {args}" },
    ToolTemplate { tool: "tar", keywords: &["xz", "high compression"], template: "-cJf {output} {input}" },
    ToolTemplate { tool: "tar", keywords: &["single file", "specific file"], template: "-xzf {input} {args}" },
    ToolTemplate { tool: "rsync", keywords: &["sync", "transfer", "copy"], template: "-avz {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["dry-run", "preview", "simulate"], template: "-avzn {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["delete", "mirror", "exact"], template: "-avz --delete {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["remote", "download from"], template: "-avz {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["resume", "partial", "interrupted"], template: "-avzP {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["exclude", "ignore"], template: "-avz --exclude='{pattern}' {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["port", "ssh port"], template: "-avz -e 'ssh -p 2222' {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["progress", "info"], template: "-avz --info=progress2 {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["hardlink", "preserve"], template: "-avzH {source} {destination}" },
    ToolTemplate { tool: "rsync", keywords: &["update", "newer"], template: "-avz --update {source} {destination}" },
    ToolTemplate { tool: "sed", keywords: &["replace", "substitute", "in-place"], template: "-i 's/{args}/{args}/g' {input}" },
    ToolTemplate { tool: "sed", keywords: &["backup", ".bak"], template: "-i.bak 's/{args}/{args}/g' {input}" },
    ToolTemplate { tool: "sed", keywords: &["delete line", "blank", "empty"], template: "'/^$/d' {input}" },
    ToolTemplate { tool: "sed", keywords: &["delete pattern", "remove line"], template: "-i '/{args}/d' {input}" },
    ToolTemplate { tool: "sed", keywords: &["print", "grep-like", "pattern match"], template: "-n '/{args}/p' {input}" },
    ToolTemplate { tool: "sed", keywords: &["regex", "capture", "reformat"], template: "-E 's/{args}/{args}/' {input}" },
    ToolTemplate { tool: "sed", keywords: &["prefix", "prepend", "add beginning"], template: "'s/^/{args}: /' {input}" },
    ToolTemplate { tool: "sed", keywords: &["trailing", "whitespace", "rstrip"], template: "-E 's/[[:space:]]+$//' {input}" },
    ToolTemplate { tool: "sed", keywords: &["specific line", "line number"], template: "'{args}s/{args}/{args}/' {input}" },
    ToolTemplate { tool: "sed", keywords: &["insert", "after pattern", "append"], template: "'/^\\[section\\]/a {args}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["csv", "comma", "column", "-F"], template: "-F ',' '{{print $1,$3}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["sum", "total", "add"], template: "'{{sum+=$2}} END{{print \"Total:\", sum}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["filter", "threshold", "greater than"], template: "'$3 > 100 {{print $0}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["count", "unique", "occurrences"], template: "'{{count[$1]++}} END{{for(k in count) print k, count[k]}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["range", "between", "pattern range"], template: "'/START/,/END/{{print}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["deduplicate", "consecutive", "remove duplicate"], template: "'prev!=$0{{print; prev=$0}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["line number", "nr", "number"], template: "'{{print NR, $0}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["tab", "tsv", "convert separator"], template: "-F '\t' 'BEGIN{{OFS=\",\"}} {{$1=$1; print}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["average", "mean", "calculate"], template: "'{{sum+=$1; n++}} END{{if(n>0) print \"Average:\", sum/n}}' {input}" },
    ToolTemplate { tool: "awk", keywords: &["last field", "last column", "$NF"], template: "'{{print $NF}}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["case-insensitive", "ignore case", "-i"], template: "-in '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["recursive", "find in files", "-r"], template: "-rn '{pattern}' --include='{args}' {input_dir}" },
    ToolTemplate { tool: "grep", keywords: &["context", "surrounding", "-C"], template: "-C 3 '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["count", "number", "-c"], template: "-c '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["extended", "regex", "multiple", "-E"], template: "-E '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["filenames", "list files", "-l"], template: "-rl '{pattern}' {input_dir}" },
    ToolTemplate { tool: "grep", keywords: &["invert", "exclude", "not match", "-v"], template: "-v '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["extract", "only matching", "-o"], template: "-oE '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["filename", "header", "-H"], template: "-Hn '{pattern}' {input}" },
    ToolTemplate { tool: "grep", keywords: &["fixed", "literal", "-F"], template: "-F '{pattern}' {input}" },
    ToolTemplate { tool: "perl", keywords: &["script", "run", ".pl"], template: "{input}" },
    ToolTemplate { tool: "perl", keywords: &["version", "-V"], template: "-V" },
    ToolTemplate { tool: "perl", keywords: &["one-liner", "-ne", "pattern"], template: "-ne 'print if /^>/' {input}" },
    ToolTemplate { tool: "perl", keywords: &["column", "extract", "-lane"], template: "-lane 'print join(\"\\t\", @F[0,2,4])' {input}" },
    ToolTemplate { tool: "perl", keywords: &["in-place", "substitute", "-i"], template: "-i.bak -pe 's/{args}/{args}/g' {input}" },
    ToolTemplate { tool: "perl", keywords: &["count", "fasta", "sequence"], template: "-ne '$c++ if /^>/; END {{print \"$c sequences\\n\"}}' {input}" },
    ToolTemplate { tool: "perl", keywords: &["cpan", "install", "module"], template: "-MCPAN -e 'CPAN::Shell->install(\"{args}\")'" },
    ToolTemplate { tool: "perl", keywords: &["local::lib", "local lib"], template: "-Mlocal::lib" },
    ToolTemplate { tool: "perl", keywords: &["check module", "installed"], template: "-M{args} -e 1" },
    ToolTemplate { tool: "perl", keywords: &["library path", "-I", "custom path"], template: "-I {input_dir} {input}" },
    ToolTemplate { tool: "python", keywords: &["script", "run", ".py"], template: "{input}" },
    ToolTemplate { tool: "python", keywords: &["one-liner", "-c", "expression"], template: "-c \"{args}\"" },
    ToolTemplate { tool: "python", keywords: &["module", "-m"], template: "-m {args}" },
    ToolTemplate { tool: "python", keywords: &["venv", "virtual environment"], template: "-m venv {output_dir}" },
    ToolTemplate { tool: "python", keywords: &["test", "pytest"], template: "-m pytest {input} -v" },
    ToolTemplate { tool: "python", keywords: &["json", "process", "stdin"], template: "-c \"import json,sys; data=json.load(sys.stdin); [print(r['name']) for r in data]\"" },
    ToolTemplate { tool: "python", keywords: &["unbuffered", "-u", "pipeline"], template: "-u {input}" },
    ToolTemplate { tool: "python", keywords: &["profile", "cprofile"], template: "-m cProfile -s cumtime {input}" },
    ToolTemplate { tool: "python", keywords: &["version", "--version"], template: "--version" },
    ToolTemplate { tool: "python", keywords: &["warning", "-W"], template: "-W all {input}" },
    ToolTemplate { tool: "julia", keywords: &["script", "run", ".jl"], template: "{input}" },
    ToolTemplate { tool: "julia", keywords: &["project", "environment"], template: "--project=. {input}" },
    ToolTemplate { tool: "julia", keywords: &["threads", "multi-thread"], template: "--threads auto {input}" },
    ToolTemplate { tool: "julia", keywords: &["expression", "-e", "pkg"], template: "-e '{args}'" },
    ToolTemplate { tool: "julia", keywords: &["version", "depot"], template: "-e 'println(VERSION); println(DEPOT_PATH)'" },
    ToolTemplate { tool: "julia", keywords: &["startup", "no startup", "ci"], template: "--startup-file=no --project=. {input}" },
    ToolTemplate { tool: "julia", keywords: &["compile", "ahead-of-time"], template: "--compile=all -O2 {input}" },
    ToolTemplate { tool: "julia", keywords: &["pluto", "notebook"], template: "-e 'import Pluto; Pluto.run(port=1234)'" },
    ToolTemplate { tool: "java", keywords: &["version", "-version"], template: "-version" },
    ToolTemplate { tool: "java", keywords: &["jar", "heap", "memory", "-Xmx"], template: "-Xmx16g -jar {input} {args}" },
    ToolTemplate { tool: "java", keywords: &["gatk", "HaplotypeCaller", "variant"], template: "-Xmx8g -jar {input} HaplotypeCaller -R {reference} -I {args} -O {output}" },
    ToolTemplate { tool: "java", keywords: &["fastqc", "qc"], template: "-Xmx2g -jar {input} {args}" },
    ToolTemplate { tool: "java", keywords: &["settings", "properties"], template: "-XshowSettings:all -version" },
    ToolTemplate { tool: "java", keywords: &["gc", "garbage collector", "flags"], template: "-XX:+PrintFlagsFinal -version" },
    ToolTemplate { tool: "java", keywords: &["trimmomatic", "PE", "illumina"], template: "-Xmx4g -jar {input} PE -threads 8 {read1} {read2} {args}" },
    ToolTemplate { tool: "java", keywords: &["classpath", "-cp"], template: "-cp {args} {args}" },
    ToolTemplate { tool: "java", keywords: &["zgc", "low latency"], template: "-Xmx32g -XX:+UseZGC -jar {input} {args}" },
    ToolTemplate { tool: "bash", keywords: &["script", "run", ".sh"], template: "{input}" },
    ToolTemplate { tool: "bash", keywords: &["strict", "pipefail", "error"], template: "-euo pipefail -c '{args}'" },
    ToolTemplate { tool: "bash", keywords: &["source", "config", "bashrc"], template: "-c 'source ~/.bashrc && printenv'" },
    ToolTemplate { tool: "bash", keywords: &["version", "--version"], template: "--version" },
    ToolTemplate { tool: "bash", keywords: &["debug", "-x", "trace"], template: "-x {input}" },
    ToolTemplate { tool: "bash", keywords: &["functions", "alias"], template: "-c 'declare -f; alias'" },
    ToolTemplate { tool: "bash", keywords: &["subshell", "export"], template: "-c 'export MY_VAR=test && echo $MY_VAR'" },
    ToolTemplate { tool: "bash", keywords: &["process substitution", "diff"], template: "-c 'diff <(sort {input}) <(sort {input2})'" },
    ToolTemplate { tool: "bash", keywords: &["background", "pid", "wait"], template: "-c '{args} &; PID=$!; wait $PID; echo \"exit: $?\"'" },
    ToolTemplate { tool: "bash", keywords: &["loop", "for", "iterate"], template: "-c 'for f in {input}; do {args} \"$f\"; done'" },
    ToolTemplate { tool: "blast", keywords: &["makeblastdb", "build database", "create blast database", "blast database from fasta"], template: "makeblastdb -in {input} -dbtype nucl -out {prefix} -parse_seqids" },
    ToolTemplate { tool: "blast", keywords: &["makeblastdb", "protein database", "protein db"], template: "makeblastdb -in {input} -dbtype prot -out {prefix} -parse_seqids" },
    ToolTemplate { tool: "blast", keywords: &["blastn", "nucleotide blast", "blast nucleotide"], template: "blastn -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "blast", keywords: &["blastp", "protein blast", "blast protein"], template: "blastp -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "blast", keywords: &["blastx", "translated query", "nucleotide against protein"], template: "blastx -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "blast", keywords: &["tblastn", "protein against nucleotide"], template: "tblastn -query {input} -db {database} -out {output} -outfmt 6 -evalue 1e-5 -num_threads {threads}" },
    ToolTemplate { tool: "multiqc", keywords: &["multiqc", "quality report", "aggregate report", "qc report"], template: "{input_dir} -o {output_dir} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["multiqc", "specific module", "module"], template: "{input_dir} -o {output_dir} -m {args} -f" },
    ToolTemplate { tool: "multiqc", keywords: &["multiqc", "ignore", "exclude"], template: "{input_dir} --ignore {args} -o {output_dir} -f" },
    ToolTemplate { tool: "gtdbtk", keywords: &["classify_wf", "classify workflow", "taxonomic classification"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["identify", "identify marker", "gtdb identify"], template: "identify --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["de_novo_wf", "de novo workflow", "de novo tree"], template: "de_novo_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["align", "align marker", "gtdb align"], template: "align --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "gtdbtk", keywords: &["classify", "gtdb classify", "classify genome"], template: "classify_wf --genome_dir {input_dir} --out_dir {output_dir} --cpus {threads}" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "circular consensus", "hifi", "generate ccs"], template: "{input} {output}" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "min passes", "minimum pass"], template: "{input} {output} --min-passes 3" },
    ToolTemplate { tool: "pbccs", keywords: &["ccs", "hifi-kinetics", "kinetics"], template: "{input} {output} --hifi-kinetics" },
    ToolTemplate { tool: "varscan2", keywords: &["mpileup2snp", "snp from pileup", "call snp"], template: "mpileup2snp {input} --min-coverage 8 --min-var-freq 0.01 --p-value 0.05 --output-vcf 1" },
    ToolTemplate { tool: "varscan2", keywords: &["mpileup2indel", "indel from pileup", "call indel"], template: "mpileup2indel {input} --min-coverage 8 --min-var-freq 0.01 --p-value 0.05 --output-vcf 1" },
    ToolTemplate { tool: "varscan2", keywords: &["somatic", "somatic variant", "tumor-normal"], template: "somatic {input} {input2} --output-vcf 1 --output {output}" },
    ToolTemplate { tool: "varscan2", keywords: &["processsomatic", "process somatic", "filter somatic"], template: "processSomatic {input} --min-tumor-freq 0.1 --max-normal-freq 0.05 --p-value 0.05" },
    ToolTemplate { tool: "sra-tools", keywords: &["fasterq-dump", "fasterq", "download fastq", "convert sra to fastq"], template: "fasterq-dump {accession} -O {output_dir} -e {threads}" },
    ToolTemplate { tool: "sra-tools", keywords: &["prefetch", "prefetch sra", "download sra"], template: "prefetch {accession} -O {output_dir}" },
    ToolTemplate { tool: "sra-tools", keywords: &["vdb-validate", "validate sra", "validate file"], template: "vdb-validate {input}" },
    ToolTemplate { tool: "sra-tools", keywords: &["fastq-dump", "fastq dump", "sra to fastq"], template: "fastq-dump {accession} -O {output_dir}" },
    ToolTemplate { tool: "wget", keywords: &["download", "save", "output file", "save as"], template: "-O {output} {url}" },
    ToolTemplate { tool: "wget", keywords: &["continue", "resume", "partial"], template: "-c {url}" },
    ToolTemplate { tool: "wget", keywords: &["background", "quiet", "silent"], template: "-b -q {url}" },
    ToolTemplate { tool: "wget", keywords: &["retry", "tries", "timeout"], template: "--tries=5 --timeout=30 --wait=2 {url}" },
    ToolTemplate { tool: "ssh", keywords: &["connect", "remote", "login"], template: "{args}" },
    ToolTemplate { tool: "ssh", keywords: &["identity", "key", "private key"], template: "-i {input} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["port", "specific port"], template: "-p {args} {args}" },
    ToolTemplate { tool: "ssh", keywords: &["forward", "tunnel", "local forward"], template: "-L {args} {args}" },
    ToolTemplate { tool: "vcftools", keywords: &["freq", "allele frequency"], template: "--vcf {input} --freq --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["hardy", "hardy weinberg", "hwe"], template: "--vcf {input} --hardy --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["het", "heterozygosity"], template: "--vcf {input} --het --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["pi", "nucleotide diversity", "site-pi"], template: "--vcf {input} --site-pi --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["tajima", "tajima d"], template: "--vcf {input} --TajimaD 10000 --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["window-pi", "pi window"], template: "--vcf {input} --window-pi 10000 --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["remove indels", "keep snp", "snp only"], template: "--vcf {input} --remove-indels --maf 0.05 --max-missing 0.9 --recode --recode-INFO-all --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["filter", "maf", "missing"], template: "--vcf {input} --maf 0.05 --max-missing 0.9 --recode --out {prefix}" },
    ToolTemplate { tool: "vcftools", keywords: &["plink", "convert plink"], template: "--vcf {input} --plink --out {prefix}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmsearch", "search profile", "search hmm against sequence"], template: "hmmsearch --cpu {threads} --tblout {output} -E 1e-5 {input} {input2}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmscan", "scan sequence", "scan against profile"], template: "hmmscan --cpu {threads} --tblout {output} -E 1e-5 {input} {input2}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmbuild", "build profile", "build hmm"], template: "hmmbuild --cpu {threads} {output} {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmpress", "press hmm", "format hmm database"], template: "hmmpress {input}" },
    ToolTemplate { tool: "hmmer", keywords: &["phmmer", "search protein sequence"], template: "phmmer --cpu {threads} --tblout {output} -E 1e-5 {input} {input2}" },
    ToolTemplate { tool: "hmmer", keywords: &["hmmalign", "align sequence to profile"], template: "hmmalign -o {output} {input} {input2}" },
    ToolTemplate { tool: "picard", keywords: &["markduplicates", "mark duplicate", "pcr duplicate"], template: "MarkDuplicates -I {input} -O {output} -M {metrics} --CREATE_INDEX true" },
    ToolTemplate { tool: "picard", keywords: &["sortsam", "sort sam", "sort bam"], template: "SortSam -I {input} -O {output} --SORT_ORDER coordinate --CREATE_INDEX true" },
    ToolTemplate { tool: "picard", keywords: &["addorreplacereadgroups", "read group", "add rg"], template: "AddOrReplaceReadGroups -I {input} -O {output} --RGLB lib1 --RGPL ILLUMINA --RGPU unit1 --RGSM sample1 --CREATE_INDEX true" },
    ToolTemplate { tool: "picard", keywords: &["collectalignmentsummarymetrics", "alignment metric", "alignment summary"], template: "CollectAlignmentSummaryMetrics -I {input} -O {output} -R {reference}" },
    ToolTemplate { tool: "picard", keywords: &["collectinsertsizemetrics", "insert size", "insert metric"], template: "CollectInsertSizeMetrics -I {input} -O {output} -H {histogram} -R {reference}" },
    ToolTemplate { tool: "picard", keywords: &["validatesamfile", "validate sam"], template: "ValidateSamFile -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["haplotypecaller", "germline variant", "call variant"], template: "HaplotypeCaller -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["markduplicates", "mark duplicate"], template: "MarkDuplicates -I {input} -O {output} -M {metrics} --CREATE_INDEX true" },
    ToolTemplate { tool: "gatk", keywords: &["mutect2", "somatic variant", "somatic mutat"], template: "Mutect2 -R {reference} -I {input} -normal {args} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["filtermutectcalls", "filter mutect"], template: "FilterMutectCalls -R {reference} -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["createsequencedictionary", "sequence dictionary"], template: "CreateSequenceDictionary -R {reference}" },
    ToolTemplate { tool: "gatk", keywords: &["addorreplacereadgroups", "read group"], template: "AddOrReplaceReadGroups -I {input} -O {output} --RGLB lib1 --RGPL ILLUMINA --RGPU unit1 --RGSM sample1" },
    ToolTemplate { tool: "gatk", keywords: &["baserecalibrator", "base recalibrat", "bqsr"], template: "BaseRecalibrator -R {reference} -I {input} --known-sites {input2} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["applybqsr", "apply bqsr"], template: "ApplyBQSR -R {reference} -I {input} --bqsr-recal-file {input2} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["selectvariants", "select variant"], template: "SelectVariants -R {reference} -V {input} -O {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-prepare-reference", "prepare reference", "rsem reference"], template: "rsem-prepare-reference --gtf {annotation} {input} {prefix}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-calculate-expression", "calculate expression", "rsem expression", "rsem quant"], template: "rsem-calculate-expression --paired-end {read1} {read2} {prefix} {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-calculate-expression", "star", "rsem star"], template: "rsem-calculate-expression --paired-end --star {read1} {read2} {prefix} {output}" },
    ToolTemplate { tool: "rsem", keywords: &["rsem-generate-data-matrix", "data matrix"], template: "rsem-generate-data-matrix {inputs}" },
    ToolTemplate { tool: "delly", keywords: &["call", "call structural", "call sv", "detect structural"], template: "call -g {reference} -o {output} {input}" },
    ToolTemplate { tool: "delly", keywords: &["lr", "long-read", "long read sv", "pacbio sv"], template: "lr -y pb -g {reference} -o {output} {input}" },
    ToolTemplate { tool: "delly", keywords: &["filter", "filter sv"], template: "filter -o {output} {input}" },
    ToolTemplate { tool: "delly", keywords: &["merge", "merge sv", "merge bcf"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "delly", keywords: &["cnv", "copy number"], template: "cnv -g {reference} -o {output} {input}" },
    ToolTemplate { tool: "mmseqs2", keywords: &["easy-search", "search sequence"], template: "easy-search {input} {input2} {output} tmp --format-mode 0 -s 7.5" },
    ToolTemplate { tool: "mmseqs2", keywords: &["easy-cluster", "cluster sequence"], template: "easy-cluster {input} {output} tmp --min-seq-id 0.9 -c 0.8" },
    ToolTemplate { tool: "mmseqs2", keywords: &["easy-linclust", "linclust", "linear cluster"], template: "easy-linclust {input} {output} tmp --min-seq-id 0.5 -c 0.8" },
    ToolTemplate { tool: "mmseqs2", keywords: &["createdb", "create database"], template: "createdb {input} {output}" },
    ToolTemplate { tool: "sourmash", keywords: &["sketch", "create sketch", "compute signature"], template: "sketch {input} -p k=31,scaled=1000 -o {output}" },
    ToolTemplate { tool: "sourmash", keywords: &["compare", "compare signature", "compare sketch", "distance matrix"], template: "compare {inputs} --csv {output} -k 31" },
    ToolTemplate { tool: "sourmash", keywords: &["gather", "metagenomic gather", "find genome"], template: "gather {input} {input2} -k 31 --threshold-bp 50000 -o {output}" },
    ToolTemplate { tool: "sourmash", keywords: &["taxonomy", "classify taxonom", "taxonomic"], template: "taxonomy annotate -g {input} -t {input2} -o {output}" },
    ToolTemplate { tool: "sourmash", keywords: &["search", "find similar", "search signature"], template: "search {input} {input2} -k 31 -o {output}" },
    ToolTemplate { tool: "igvtools", keywords: &["totdf", "to tdf", "tdf"], template: "toTDF -z 5 -f mean {input} {output} hg38" },
    ToolTemplate { tool: "igvtools", keywords: &["count", "coverage count"], template: "count -z 5 -w 25 {input} {output} hg38" },
    ToolTemplate { tool: "igvtools", keywords: &["sort", "igv sort"], template: "sort {input} {output}" },
    ToolTemplate { tool: "igvtools", keywords: &["index", "igv index"], template: "index {input}" },
    ToolTemplate { tool: "igvtools", keywords: &["formatexp", "format exp"], template: "formatexp -c {input} {output}" },
    ToolTemplate { tool: "mummer", keywords: &["nucmer", "nucleotide align", "nucleotide mummer"], template: "nucmer -p {prefix} {input} {input2}" },
    ToolTemplate { tool: "mummer", keywords: &["dnadiff", "dna diff", "compare genome"], template: "dnadiff {input} {input2}" },
    ToolTemplate { tool: "mummer", keywords: &["delta-filter", "filter delta"], template: "delta-filter {input}" },
    ToolTemplate { tool: "mummer", keywords: &["show-coords", "show coordinate"], template: "show-coords -Clr {input}" },
    ToolTemplate { tool: "mummer", keywords: &["mummerplot", "plot mummer", "dot plot"], template: "mummerplot --png --prefix={prefix} {input}" },
    ToolTemplate { tool: "mummer", keywords: &["show-snps", "show snp"], template: "show-snps -Clr {input}" },
    ToolTemplate { tool: "survivor", keywords: &["stats", "sv statistic", "survivor stat"], template: "stats -i {input} -o {output}" },
    ToolTemplate { tool: "survivor", keywords: &["filter", "survivor filter", "filter sv"], template: "filter -i {input} -o {output} -s 50 -e 100000 -f 0" },
    ToolTemplate { tool: "survivor", keywords: &["merge", "survivor merge", "merge sv"], template: "merge {inputs} 500 2 1 1 0 50 {output}" },
    ToolTemplate { tool: "survivor", keywords: &["simsv", "simulate sv"], template: "simSV {input} {output} 0 0 simulated" },
    ToolTemplate { tool: "truvari", keywords: &["bench", "benchmark variant", "truvari bench"], template: "bench -b {input} -c {input2} -f {reference} -o {output_dir}" },
    ToolTemplate { tool: "truvari", keywords: &["collapse", "collapse variant"], template: "collapse -i {input} -o {output} --chain --keep common" },
    ToolTemplate { tool: "truvari", keywords: &["refine", "refine region"], template: "refine --reference {reference} --regions {input2} {input}" },
    ToolTemplate { tool: "vcfanno", keywords: &["annotate vcf", "vcf annotate", "annotate"], template: "{input} {input2}" },
    ToolTemplate { tool: "cellsnp-lite", keywords: &["cellsnp", "snp calling", "cell snp"], template: "-s {input} -O {output_dir} -R {input2} --minMAF 0.1 --minCOUNT 20 -p {threads}" },
    ToolTemplate { tool: "kb", keywords: &["ref", "kb ref", "build reference", "build index"], template: "ref -i {input} -g {annotation} -f1 {input2}" },
    ToolTemplate { tool: "kb", keywords: &["count", "kb count", "quantify", "count cell"], template: "count -i {input} -g {annotation} -x 10xv3 -o {output_dir} {read1} {read2}" },
    ToolTemplate { tool: "mash", keywords: &["sketch", "mash sketch", "create sketch"], template: "sketch {input} -o {output}" },
    ToolTemplate { tool: "mash", keywords: &["dist", "mash dist", "distance", "compare genome"], template: "dist {input} {input2}" },
    ToolTemplate { tool: "mash", keywords: &["triangle", "all-vs-all", "pairwise"], template: "triangle {input}" },
    ToolTemplate { tool: "mash", keywords: &["screen", "screen contain"], template: "screen {input} {input2}" },
    ToolTemplate { tool: "mash", keywords: &["paste", "merge sketch"], template: "paste {output} {inputs}" },
    ToolTemplate { tool: "mash", keywords: &["info", "sketch info"], template: "info {input}" },
    ToolTemplate { tool: "diamond", keywords: &["makedb", "make database", "build database"], template: "makedb --in {input} -d {prefix}" },
    ToolTemplate { tool: "diamond", keywords: &["blastp", "protein search", "diamond blastp"], template: "blastp -q {input} -d {database} -o {output} --outfmt 6 -e 1e-5 -p {threads}" },
    ToolTemplate { tool: "diamond", keywords: &["blastx", "translated search", "diamond blastx"], template: "blastx -q {input} -d {database} -o {output} --outfmt 101 -e 1e-5 -p {threads}" },
    ToolTemplate { tool: "diamond", keywords: &["cluster", "cluster protein"], template: "cluster -d {input} -o {output} --approx-id 50" },
    ToolTemplate { tool: "diamond", keywords: &["linclust", "linear cluster", "fast cluster"], template: "linclust -d {input} -o {output} --approx-id 50" },
    ToolTemplate { tool: "agat", keywords: &["gff to gtf", "convert gff gtf"], template: "agat_convert_sp_gff2gtf --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["gff statistic", "annotation statistic"], template: "agat_sp_statistics --gff {input}" },
    ToolTemplate { tool: "agat", keywords: &["filter gene by length", "filter by length"], template: "agat_sp_filter_gene_by_length --gff {input} --size 300 --test \">=\" -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["fix gff", "standardize gff"], template: "agat_convert_sp_gxf2gxf --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["extract sequence from gff"], template: "agat_sp_extract_sequences --gff {input} -f {reference} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["longest isoform"], template: "agat_sp_keep_longest_isoform --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["merge annotation", "merge gff"], template: "agat_sp_merge_annotations --gff {inputs} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["manage id", "fix id"], template: "agat_sp_manage_IDs --gff {input} -o {output}" },
    ToolTemplate { tool: "agat", keywords: &["gff to bed"], template: "agat_convert_sp_gff2bed --gff {input} -o {output}" },
    ToolTemplate { tool: "git", keywords: &["clone", "clone repo"], template: "clone {url}" },
    ToolTemplate { tool: "git", keywords: &["checkout", "switch branch", "create branch"], template: "checkout -b {args}" },
    ToolTemplate { tool: "git", keywords: &["commit", "create commit"], template: "commit -m \"{args}\"" },
    ToolTemplate { tool: "git", keywords: &["push", "push to remote"], template: "push" },
    ToolTemplate { tool: "git", keywords: &["pull", "pull from remote"], template: "pull" },
    ToolTemplate { tool: "git", keywords: &["log", "commit log", "commit history"], template: "log --oneline --graph --decorate --all" },
    ToolTemplate { tool: "git", keywords: &["branch", "list branch"], template: "branch -a" },
    ToolTemplate { tool: "git", keywords: &["merge", "merge branch"], template: "merge {args}" },
    ToolTemplate { tool: "git", keywords: &["status", "working tree"], template: "status" },
    ToolTemplate { tool: "git", keywords: &["diff", "show diff"], template: "diff {args}" },
    ToolTemplate { tool: "git", keywords: &["add", "stage file"], template: "add {input}" },
    ToolTemplate { tool: "git", keywords: &["stash", "stash change"], template: "stash" },
    ToolTemplate { tool: "git", keywords: &["tag", "create tag"], template: "tag {args}" },
    ToolTemplate { tool: "git", keywords: &["reset", "undo commit"], template: "reset {args}" },
    ToolTemplate { tool: "git", keywords: &["rebase", "rebase branch"], template: "rebase {args}" },
    ToolTemplate { tool: "git", keywords: &["fetch", "fetch remote"], template: "fetch" },
    ToolTemplate { tool: "hifiasm", keywords: &["hifi", "pacbio hifi", "hifi assembly"], template: "-o {output} --hifi {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["trio", "paternal maternal", "hap1 hap2"], template: "-o {output} --h1 {read1} --h2 {read2} {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["polyploid", "n-hap"], template: "-o {output} --n-hap 4 {input}" },
    ToolTemplate { tool: "hifiasm", keywords: &["purge duplicate", "l0"], template: "-o {output} -l0 {input}" },
    ToolTemplate { tool: "verkko", keywords: &["hifi", "hifi assembly"], template: "--hifi {input} -d {output_dir}" },
    ToolTemplate { tool: "verkko", keywords: &["ont", "nanopore"], template: "--ont {input} -d {output_dir}" },
    ToolTemplate { tool: "verkko", keywords: &["trio"], template: "--hifi {input} --hap-kmers {input2} {args} -d {output_dir}" },
    ToolTemplate { tool: "bowtie2", keywords: &["bowtie2-build", "build index", "create index"], template: "bowtie2-build {reference} {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["bowtie2-inspect", "inspect index"], template: "bowtie2-inspect {index}" },
    ToolTemplate { tool: "bowtie2", keywords: &["align", "map", "mem"], template: "-x {index} -1 {read1} -2 {read2} -p {threads}" },
    ToolTemplate { tool: "bowtie2", keywords: &["align", "single-end", "unpaired"], template: "-U {input} -x {index} -p {threads}" },
    ToolTemplate { tool: "minimap2", keywords: &["map-ont", "ont", "nanopore"], template: "-ax map-ont -t {threads} {reference} {input}" },
    ToolTemplate { tool: "minimap2", keywords: &["map-pb", "pacbio", "clr"], template: "-ax map-pb -t {threads} {reference} {input}" },
    ToolTemplate { tool: "minimap2", keywords: &["map-hifi", "hifi", "ccs"], template: "-ax map-hifi -t {threads} {reference} {input}" },
    ToolTemplate { tool: "minimap2", keywords: &["splice", "rna", "long read rna"], template: "-ax splice -t {threads} {reference} {input}" },
    ToolTemplate { tool: "minimap2", keywords: &["index", "build index"], template: "-d {output} {reference}" },
    ToolTemplate { tool: "star", keywords: &["genomegenerate", "genome index", "build index"], template: "--runMode genomeGenerate --genomeDir {genome_dir} --genomeFastaFiles {reference} --sjdbGTFfile {annotation} --runThreadN {threads}" },
    ToolTemplate { tool: "star", keywords: &["align", "map read"], template: "--runMode alignReads --genomeDir {genome_dir} --readFilesIn {input} --runThreadN {threads} --outFileNamePrefix {output_prefix}" },
    ToolTemplate { tool: "bwa", keywords: &["mem", "align", "map"], template: "mem -t {threads} {reference} {input}" },
    ToolTemplate { tool: "bwa", keywords: &["index", "build index"], template: "index {reference}" },
    ToolTemplate { tool: "bwa", keywords: &["mem", "paired-end", "paired"], template: "mem -t {threads} {reference} {read1} {read2}" },
    ToolTemplate { tool: "bwa-mem2", keywords: &["mem", "align", "map"], template: "mem -t {threads} {reference} {input}" },
    ToolTemplate { tool: "bwa-mem2", keywords: &["index", "build index"], template: "index {reference}" },
    ToolTemplate { tool: "bwa-mem2", keywords: &["mem", "paired-end", "paired", "read group"], template: "mem -t {threads} -R '@RG\\tID:s1\\tSM:s1\\tLB:lib1\\tPL:ILLUMINA' {reference} {read1} {read2}" },
    ToolTemplate { tool: "bedtools", keywords: &["genomecov", "genome coverage", "bedgraph", "bga"], template: "genomecov -ibam {input} -bga" },
    ToolTemplate { tool: "bedtools", keywords: &["genomecov", "genome coverage", "bg"], template: "genomecov -ibam {input} -bg" },
    ToolTemplate { tool: "bedtools", keywords: &["getfasta", "extract sequence", "fasta from bed"], template: "getfasta -fi {reference} -bed {input} -fo {output}" },
    ToolTemplate { tool: "bedtools", keywords: &["intersect", "overlap"], template: "intersect -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["subtract", "remove overlap"], template: "subtract -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["merge", "merge overlapping"], template: "merge -i {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["closest", "nearest feature"], template: "closest -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["coverage", "coverage per feature"], template: "coverage -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["makewindows", "create window", "tile"], template: "makewindows -g {input} -w 10000" },
    ToolTemplate { tool: "bedtools", keywords: &["slop", "extend bed"], template: "slop -i {input} -g {input2} -b 1000" },
    ToolTemplate { tool: "bedtools", keywords: &["sort", "sort bed"], template: "sort -i {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["mpileup", "pileup", "call variant from bam"], template: "mpileup -f {reference} -Ou {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "filter vcf", "extract", "convert"], template: "view -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "filter", "snp only", "quality"], template: "view -i 'QUAL>30' -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["call", "call variant"], template: "call -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["filter", "filter vcf"], template: "filter -i 'QUAL>30' -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["sort"], template: "sort -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["index"], template: "index {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["merge", "merge vcf"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["concat"], template: "concat -o {output} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["norm", "normalize", "split multi-allelic"], template: "norm -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["stats", "statistic"], template: "stats {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["annotate"], template: "annotate -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["isec", "intersection"], template: "isec -p {output_dir} {inputs}" },
    ToolTemplate { tool: "bcftools", keywords: &["query", "extract field"], template: "query -f '%CHROM\\t%POS\\n' {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "convert", "filter bam", "extract"], template: "view -b -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["sort", "sort bam", "sort by coordinate"], template: "sort -@ {threads} -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["index", "bai index"], template: "index {input}" },
    ToolTemplate { tool: "samtools", keywords: &["flagstat", "flag statistic"], template: "flagstat {input}" },
    ToolTemplate { tool: "samtools", keywords: &["merge", "merge bam"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "samtools", keywords: &["depth", "coverage depth"], template: "depth {input}" },
    ToolTemplate { tool: "samtools", keywords: &["stats", "statistic"], template: "stats {input}" },
    ToolTemplate { tool: "samtools", keywords: &["mpileup", "pileup"], template: "mpileup -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["faidx", "fasta index"], template: "faidx {input}" },
    ToolTemplate { tool: "samtools", keywords: &["markdup", "mark duplicate"], template: "markdup {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["bam2fq", "fastq", "bam to fastq"], template: "bam2fq {input}" },
    ToolTemplate { tool: "samtools", keywords: &["idxstats"], template: "idxstats {input}" },
    ToolTemplate { tool: "samtools", keywords: &["dict"], template: "dict {input}" },
    ToolTemplate { tool: "samtools", keywords: &["fixmate"], template: "fixmate {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["calmd"], template: "calmd {input} {reference}" },
    ToolTemplate { tool: "whatshap", keywords: &["phase", "phasing", "haplotype"], template: "phase --reference {reference} -o {output} {input}" },
    ToolTemplate { tool: "whatshap", keywords: &["haplotag", "assign haplotype", "tag read"], template: "haplotag --output {output} --reference {reference} {input} {input2}" },
    ToolTemplate { tool: "whatshap", keywords: &["stats", "phasing statistic"], template: "stats {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["parse", "parse sam", "parse alignment"], template: "parse -c {input} {input2}" },
    ToolTemplate { tool: "pairtools", keywords: &["sort", "sort pair"], template: "sort -o {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["dedup", "deduplicate pair"], template: "dedup --nproc {threads} --output-stats {output} {input}" },
    ToolTemplate { tool: "pairtools", keywords: &["cload", "load cooler"], template: "cload pairs {input} {input2} {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["select", "select pair"], template: "select {input} -o {output}" },
    ToolTemplate { tool: "pairtools", keywords: &["merge", "merge pair"], template: "merge -o {output} {inputs}" },
    ToolTemplate { tool: "modkit", keywords: &["pileup", "methylation pileup", "call modification"], template: "pileup --ref {reference} --cpg --combine-strands {input} {output}" },
    ToolTemplate { tool: "modkit", keywords: &["extract", "extract modification"], template: "extract {input} -o {output}" },
    ToolTemplate { tool: "modkit", keywords: &["summary", "modification summary"], template: "summary {input} -o {output}" },
    ToolTemplate { tool: "modkit", keywords: &["motif-bed", "motif"], template: "motif-bed {input} CG 0" },
    ToolTemplate { tool: "modkit", keywords: &["sample-probs"], template: "sample-probs {input} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["bamcoverage", "bam coverage", "coverage bigwig"], template: "bamCoverage -b {input} -o {output} --normalizeUsing RPKM --binSize 10" },
    ToolTemplate { tool: "deeptools", keywords: &["bamcompare", "bam compare", "differential coverage"], template: "bamCompare -b1 {input} -b2 {input2} -o {output} --normalizeUsing RPKM" },
    ToolTemplate { tool: "deeptools", keywords: &["computematrix", "compute matrix", "heatmap"], template: "computeMatrix scale-regions -S {input} -R {input2} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotheatmap", "plot heatmap"], template: "plotHeatmap -m {input} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["multibamsummary", "correlation"], template: "multiBamSummary bins -b {inputs} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotfingerprint", "fingerprint", "chip quality"], template: "plotFingerprint -b {inputs} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotcoverage", "coverage plot"], template: "plotCoverage -b {inputs} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["plotprofile", "profile plot"], template: "plotProfile -m {input} -o {output}" },
    ToolTemplate { tool: "deeptools", keywords: &["estimatereadfiltering"], template: "estimateReadFiltering -b {inputs} -o {output_dir}" },
    ToolTemplate { tool: "deeptools", keywords: &["alignmentsieve"], template: "alignmentSieve -b {input} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["batch", "cnvkit batch"], template: "batch {input} --reference {input2} --output-dir {output_dir}" },
    ToolTemplate { tool: "cnvkit", keywords: &["genemetrics", "gene metric"], template: "genemetrics {input} -t -m 0.3 -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["scatter", "scatter plot"], template: "scatter {input} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["segment"], template: "segment {input} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["heatmap"], template: "heatmap {inputs} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["call"], template: "call {input} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["access"], template: "access {reference} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["coverage"], template: "coverage {input} -l {input2} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["reference"], template: "reference {inputs} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["fix"], template: "fix {input} {input2} -o {output}" },
    ToolTemplate { tool: "cnvkit", keywords: &["diagram"], template: "diagram {input} -o {output}" },
    ToolTemplate { tool: "qualimap", keywords: &["bamqc", "bam qc"], template: "bamqc -bam {input} -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["rnaseq", "rna-seq qc"], template: "rnaseq -bam {input} -gtf {annotation} -outdir {output_dir}" },
    ToolTemplate { tool: "qualimap", keywords: &["multi-bamqc"], template: "multi-bamqc -d {input} -outdir {output_dir}" },
    ToolTemplate { tool: "flye", keywords: &["nano-raw", "ont raw", "nanopore raw"], template: "--nano-raw {input} -o {output_dir} -g 5m -t {threads}" },
    ToolTemplate { tool: "flye", keywords: &["nano-corr", "ont corrected"], template: "--nano-corr {input} -o {output_dir} -g 5m -t {threads}" },
    ToolTemplate { tool: "flye", keywords: &["pacbio-raw", "pacbio clr"], template: "--pacbio-raw {input} -o {output_dir} -g 5m -t {threads}" },
    ToolTemplate { tool: "flye", keywords: &["pacbio-corr", "pacbio corrected"], template: "--pacbio-corr {input} -o {output_dir} -g 5m -t {threads}" },
    ToolTemplate { tool: "flye", keywords: &["pacbio-hifi", "hifi", "ccs"], template: "--pacbio-hifi {input} -o {output_dir} -g 5m -t {threads}" },
    ToolTemplate { tool: "bracken", keywords: &["bracken", "estimate abundance", "abundance"], template: "-d {database} -i {input} -o {output} -l S -r 150" },
    ToolTemplate { tool: "bracken", keywords: &["bracken", "species level", "species abundance"], template: "-d {database} -i {input} -o {output} -l S -r 150" },
    ToolTemplate { tool: "bracken", keywords: &["bracken", "genus level", "genus abundance"], template: "-d {database} -i {input} -o {output} -l G -r 150" },
    ToolTemplate { tool: "bracken", keywords: &["bracken-build", "build bracken"], template: "bracken-build -d {database} -k 35 -l 150" },
    ToolTemplate { tool: "bracken", keywords: &["combine_bracken_outputs", "combine bracken"], template: "combine_bracken_outputs -i {inputs} -o {output}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "taxonomic", "kraken2 classify"], template: "--db {database} --paired --output {output} --report {report} {read1} {read2}" },
    ToolTemplate { tool: "kraken2", keywords: &["classify", "single-end", "single read"], template: "--db {database} --output {output} --report {report} {input}" },
    ToolTemplate { tool: "kraken2", keywords: &["kraken2-build", "build database"], template: "kraken2-build --standard --db {database}" },
    ToolTemplate { tool: "kraken2", keywords: &["confidence"], template: "--db {database} --confidence 0.1 --paired --output {output} --report {report} {read1} {read2}" },
    ToolTemplate { tool: "checkm2", keywords: &["predict", "completeness", "contamination"], template: "predict --input {input_dir} --output-directory {output_dir} --threads {threads}" },
    ToolTemplate { tool: "checkm2", keywords: &["database", "download database"], template: "database --download" },
    ToolTemplate { tool: "checkm2", keywords: &["testrun", "test"], template: "testrun" },
    ToolTemplate { tool: "macs2", keywords: &["callpeak", "peak calling"], template: "callpeak -t {input} -c {input2} -f BAM -g hs -n {prefix} --bdg --outdir {output_dir}" },
    ToolTemplate { tool: "macs2", keywords: &["predictd", "fragment size"], template: "predictd -i {input} -g hs" },
    ToolTemplate { tool: "macs2", keywords: &["bdgcmp", "compare bedgraph"], template: "bdgcmp -t {input} -c {input2} -m FE -o {output}" },
    ToolTemplate { tool: "macs2", keywords: &["bdgdiff", "differential peak"], template: "bdgdiff --t1 {input} --c1 {input2} --t2 {args} --c2 {args} -o {output}" },
    ToolTemplate { tool: "liftoff", keywords: &["liftoff", "lift over", "map annotation"], template: "{input} {input2} -g {annotation} -o {output} -u {args}" },
    ToolTemplate { tool: "liftoff", keywords: &["liftoff", "copies"], template: "{input} {input2} -g {annotation} -o {output} -copies -u {args}" },
    ToolTemplate { tool: "liftoff", keywords: &["liftoff", "partial"], template: "{input} {input2} -g {annotation} -o {output} -sc 0.95 -s 0.9 -u {args}" },
    ToolTemplate { tool: "snpeff", keywords: &["ann", "annotate vcf", "annotate variant"], template: "ann {database} {input}" },
    ToolTemplate { tool: "snpeff", keywords: &["build", "build database"], template: "build -gff3 -v {args}" },
    ToolTemplate { tool: "snpeff", keywords: &["download", "download database"], template: "download {args}" },
    ToolTemplate { tool: "repeatmasker", keywords: &["repeatmasker", "mask repeat", "species"], template: "-species {args} -pa {threads} {input}" },
    ToolTemplate { tool: "repeatmasker", keywords: &["repeatmasker", "custom library", "library"], template: "-lib {input} -pa {threads} {input2}" },
    ToolTemplate { tool: "repeatmasker", keywords: &["repeatmasker", "no int"], template: "-noint -pa {threads} {input}" },
    ToolTemplate { tool: "salmon", keywords: &["index", "build index"], template: "index -t {input} -i {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["quant", "quantify", "expression"], template: "quant -i {index} -l A -1 {read1} -2 {read2} -p {threads} -o {output_dir}" },
    ToolTemplate { tool: "salmon", keywords: &["quant", "single-end"], template: "quant -i {index} -l A -r {input} -p {threads} -o {output_dir}" },
    ToolTemplate { tool: "kallisto", keywords: &["index", "build index"], template: "index -i {output} {input}" },
    ToolTemplate { tool: "kallisto", keywords: &["quant", "quantify", "expression"], template: "quant -i {index} -o {output_dir} -b 100 {read1} {read2}" },
    ToolTemplate { tool: "kallisto", keywords: &["bus", "bus format"], template: "bus -i {index} -o {output_dir} -x 10xv3 {read1} {read2}" },
    ToolTemplate { tool: "stringtie", keywords: &["assemble", "transcript", "expression"], template: "-G {annotation} -o {output} -p {threads} {input}" },
    ToolTemplate { tool: "stringtie", keywords: &["merge", "merge transcript"], template: "--merge -G {annotation} -o {output} {inputs}" },
    ToolTemplate { tool: "stringtie", keywords: &["estimate abundance", "ballgown"], template: "-e -G {annotation} -o {output} -p {threads} {input}" },
    ToolTemplate { tool: "featurecounts", keywords: &["count", "featurecounts", "read count"], template: "-a {annotation} -o {output} -T {threads} {input}" },
    ToolTemplate { tool: "featurecounts", keywords: &["paired-end", "paired"], template: "-a {annotation} -o {output} -T {threads} -p {input}" },
    ToolTemplate { tool: "featurecounts", keywords: &["strand-specific", "stranded"], template: "-a {annotation} -o {output} -T {threads} -s 2 {input}" },
    ToolTemplate { tool: "trinity", keywords: &["de novo", "assemble transcript", "trinity"], template: "--seqType fq --left {read1} --right {read2} --max_memory 50G --CPU {threads} --output {output_dir}" },
    ToolTemplate { tool: "trinity", keywords: &["genome-guided", "genome guided"], template: "--genome_guided_bam {input} --genome_guided_max_intron 10000 --max_memory 50G --CPU {threads} --output {output_dir}" },
    ToolTemplate { tool: "spades", keywords: &["assemble", "spades"], template: "-1 {read1} -2 {read2} -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "spades", keywords: &["meta", "metagenomic"], template: "--meta -1 {read1} -2 {read2} -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "spades", keywords: &["plasmid"], template: "--plasmid -1 {read1} -2 {read2} -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "spades", keywords: &["isolate"], template: "--isolate -1 {read1} -2 {read2} -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "spades", keywords: &["rna viral", "viral"], template: "--rnaviral -s {input} -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "megahit", keywords: &["assemble", "megahit"], template: "-1 {read1} -2 {read2} -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "megahit", keywords: &["meta", "metagenomic"], template: "-1 {read1} -2 {read2} --presets meta-sensitive -o {output_dir} -t {threads}" },
    ToolTemplate { tool: "canu", keywords: &["assemble", "canu"], template: "-p {prefix} -d {output_dir} genome={args} -pacbio-raw {input}" },
    ToolTemplate { tool: "canu", keywords: &["ont", "nanopore"], template: "-p {prefix} -d {output_dir} genome={args} -nanopore-raw {input}" },
    ToolTemplate { tool: "canu", keywords: &["pacbio", "clr"], template: "-p {prefix} -d {output_dir} genome={args} -pacbio-raw {input}" },
    ToolTemplate { tool: "canu", keywords: &["pacbio", "hifi", "corrected"], template: "-p {prefix} -d {output_dir} genome={args} -pacbio-hifi {input}" },
    ToolTemplate { tool: "prokka", keywords: &["annotate", "prokka"], template: "--outdir {output_dir} --prefix {prefix} --kingdom Bacteria {input}" },
    ToolTemplate { tool: "prodigal", keywords: &["predict gene", "gene finding", "prodigal"], template: "-i {input} -a {output}" },
    ToolTemplate { tool: "prodigal", keywords: &["prodigal", "nucleotide"], template: "-i {input} -d {output}" },
    ToolTemplate { tool: "prodigal", keywords: &["prodigal", "gff"], template: "-i {input} -f gff -o {output}" },
    ToolTemplate { tool: "augustus", keywords: &["predict gene", "gene finding", "augustus"], template: "--species={args} {input}" },
    ToolTemplate { tool: "augustus", keywords: &["augustus", "gff3"], template: "--species={args} --gff3 {input}" },
    ToolTemplate { tool: "bakta", keywords: &["annotate", "bakta"], template: "--db {database} --output {output_dir} --prefix {prefix} --threads {threads} {input}" },
    ToolTemplate { tool: "eggnog-mapper", keywords: &["annotate protein", "eggnog", "emapper"], template: "emapper.py -i {input} -o {prefix} --cpu {threads}" },
    ToolTemplate { tool: "metaphlan", keywords: &["metaphlan", "profile", "taxonomic profile"], template: "--input-type fastq {input} -o {output} --nproc {threads}" },
    ToolTemplate { tool: "metaphlan", keywords: &["merge", "combine metaphlan"], template: "merge_metaphlan_tables.py -i {inputs} -o {output}" },
    ToolTemplate { tool: "orthofinder", keywords: &["find ortholog", "orthofinder find"], template: "-f {input_dir} -t {threads}" },
    ToolTemplate { tool: "orthofinder", keywords: &["from blast", "blast result"], template: "-b {input_dir} -t {threads}" },
    ToolTemplate { tool: "fastani", keywords: &["fastani", "compare genome", "ani"], template: "--query {input} --ref {input2} --output {output}" },
    ToolTemplate { tool: "fastani", keywords: &["fastani", "query list", "reference list"], template: "--ql {input} --rl {input2} --output {output}" },
    ToolTemplate { tool: "plink2", keywords: &["pca", "principal component"], template: "--bfile {prefix} --pca --out {prefix}" },
    ToolTemplate { tool: "plink2", keywords: &["association", "assoc"], template: "--bfile {prefix} --assoc --out {prefix}" },
    ToolTemplate { tool: "plink2", keywords: &["make-bed", "binary"], template: "--vcf {input} --make-bed --out {prefix}" },
    ToolTemplate { tool: "plink2", keywords: &["freq", "allele frequency"], template: "--bfile {prefix} --freq --out {prefix}" },
    ToolTemplate { tool: "plink2", keywords: &["hardy", "hwe"], template: "--bfile {prefix} --hardy --out {prefix}" },
    ToolTemplate { tool: "plink2", keywords: &["filter", "maf", "mind", "geno"], template: "--bfile {prefix} --maf 0.05 --mind 0.1 --geno 0.1 --hwe 1e-6 --make-bed --out {prefix}" },
    ToolTemplate { tool: "shapeit4", keywords: &["phase", "phasing", "shapeit"], template: "--input {input} --output {output} --region {args} --thread {threads}" },
    ToolTemplate { tool: "admixture", keywords: &["admixture", "population structure", "ancestry"], template: "{input} K --cv=10" },
    ToolTemplate { tool: "angsd", keywords: &["saf", "site allele frequency"], template: "-bam {input} -out {prefix} -doSaf 1 -anc {reference} -GL 1" },
    ToolTemplate { tool: "angsd", keywords: &["do genotype", "genotype likelihood"], template: "-bam {input} -out {prefix} -doGeno 2 -doMajorMinor 1 -doMaf 1 -GL 1" },
    ToolTemplate { tool: "angsd", keywords: &["sfs", "frequency spectrum"], template: "-bam {input} -out {prefix} -doSaf 1 -anc {reference} -GL 1" },
    ToolTemplate { tool: "angsd", keywords: &["fst", "fixation index", "population diff"], template: "-bam {input} -out {prefix} -doSaf 1 -anc {reference} -GL 1" },
    ToolTemplate { tool: "arriba", keywords: &["arriba", "fusion", "fusion detection"], template: "-x {input} -o {output} -g {reference} -a {annotation} -b {input2}" },
    ToolTemplate { tool: "pbfusion", keywords: &["pbfusion", "pacbio fusion"], template: "-i {input} -o {output} -g {reference} -r {args}" },
    ToolTemplate { tool: "freebayes", keywords: &["freebayes", "variant call", "call variant"], template: "-f {reference} -p 1 {input} -v {output}" },
    ToolTemplate { tool: "longshot", keywords: &["longshot", "variant call", "long read variant"], template: "-F bam -f {reference} {input} -o {output}" },
    ToolTemplate { tool: "sniffles", keywords: &["sniffles", "sv call", "structural variant"], template: "--input {input} --output {output} --min_support 10" },
    ToolTemplate { tool: "sniffles", keywords: &["sniffles", "genotype"], template: "--input {input} --output {output} --genotype --min_support 10" },
    ToolTemplate { tool: "strelka2", keywords: &["germline", "germline variant"], template: "configureStrelkaGermlineWorkflow.py --bam {input} --referenceFasta {reference} --runDir {output_dir}" },
    ToolTemplate { tool: "strelka2", keywords: &["somatic", "somatic variant"], template: "configureStrelkaSomaticWorkflow.py --normal {input} --tumor {input2} --referenceFasta {reference} --runDir {output_dir}" },
    ToolTemplate { tool: "medaka", keywords: &["consensus", "medaka consensus"], template: "medaka_consensus -i {input} -d {input2} -o {output_dir} -m {args}" },
    ToolTemplate { tool: "medaka", keywords: &["variant", "medaka variant"], template: "medaka_variant -i {input} -o {output_dir}" },
    ToolTemplate { tool: "medaka", keywords: &["sequence"], template: "medaka sequence {input} {output}" },
    ToolTemplate { tool: "medaka", keywords: &["inference"], template: "medaka inference --save_features --model {args} {input} {output}" },
    ToolTemplate { tool: "nanocomp", keywords: &["nanocomp", "compare quality"], template: "--outdir {output_dir} {input} {input2}" },
    ToolTemplate { tool: "nanocomp", keywords: &["nanocomp", "plot"], template: "--outdir {output_dir} --plot {input}" },
    ToolTemplate { tool: "chopper", keywords: &["chopper", "filter quality", "quality filter"], template: "-q 10 --min_length 50 -i {input} -o {output}" },
    ToolTemplate { tool: "cutadapt", keywords: &["cutadapt", "adapter", "trim adapter", "3 prime adapter"], template: "-a {args} -o {output} {input}" },
    ToolTemplate { tool: "cutadapt", keywords: &["cutadapt", "paired-end", "paired adapter"], template: "-a {args} -A {args} -o {output} -p {output2} {read1} {read2}" },
    ToolTemplate { tool: "trim_galore", keywords: &["trim_galore", "trim", "adapter trim"], template: "{input} -o {output_dir}" },
    ToolTemplate { tool: "trim_galore", keywords: &["trim_galore", "paired-end", "paired"], template: "--paired {read1} {read2} -o {output_dir}" },
    ToolTemplate { tool: "trim_galore", keywords: &["trim_galore", "quality", "quality trim"], template: "--quality 20 --length 20 {input} -o {output_dir}" },
    ToolTemplate { tool: "fastp", keywords: &["fastp", "quality control", "qc"], template: "-i {input} -o {output} -w {threads}" },
    ToolTemplate { tool: "fastp", keywords: &["fastp", "paired-end", "paired"], template: "-i {read1} -I {read2} -o {out1} -O {out2} -w {threads}" },
    ToolTemplate { tool: "fastqc", keywords: &["fastqc", "quality control", "qc"], template: "-o {output_dir} -t {threads} {input}" },
    ToolTemplate { tool: "fastq-screen", keywords: &["fastq-screen", "contamination", "screen"], template: "--conf {input} --outdir {output_dir} {input2}" },
    ToolTemplate { tool: "nanoplot", keywords: &["nanoplot", "fastq", "quality plot"], template: "--fastq {input} -o {output_dir}" },
    ToolTemplate { tool: "nanoplot", keywords: &["nanoplot", "bam", "bam quality"], template: "--bam {input} -o {output_dir}" },
    ToolTemplate { tool: "nanoplot", keywords: &["nanoplot", "summary"], template: "--summary {input} -o {output_dir}" },
    ToolTemplate { tool: "nanostat", keywords: &["nanostat", "fastq", "statistic"], template: "--fastq {input} -o {output}" },
    ToolTemplate { tool: "nanostat", keywords: &["nanostat", "bam"], template: "--bam {input} -o {output}" },
    ToolTemplate { tool: "nanostat", keywords: &["nanostat", "summary"], template: "--summary {input} -o {output}" },
    ToolTemplate { tool: "porechop", keywords: &["porechop", "adapter trim", "trim nanopore"], template: "-i {input} -o {output}" },
    ToolTemplate { tool: "porechop", keywords: &["porechop", "paired-end", "middle adapter"], template: "-i {input} -o {output} --middle-trim" },
    ToolTemplate { tool: "miniasm", keywords: &["miniasm", "assemble", "overlap"], template: "-f {reference} {input}" },
    ToolTemplate { tool: "racon", keywords: &["racon", "polish", "consensus"], template: "-t {threads} {input} {input2} {args}" },
    ToolTemplate { tool: "pilon", keywords: &["pilon", "polish", "fix"], template: "--genome {reference} --frags {input} --output {prefix} --threads {threads}" },
    ToolTemplate { tool: "pilon", keywords: &["pilon", "bam", "fix"], template: "--genome {reference} --bam {input} --output {prefix} --threads {threads}" },
    ToolTemplate { tool: "quast", keywords: &["quast", "assembly quality"], template: "-o {output_dir} {input}" },
    ToolTemplate { tool: "quast", keywords: &["metaquast", "meta quast", "metagenome"], template: "metaquast.py -o {output_dir} {input}" },
    ToolTemplate { tool: "busco", keywords: &["busco", "completeness", "assessment"], template: "-i {input} -l bacteria_odb10 -o {prefix} -m genome -c {threads}" },
    ToolTemplate { tool: "busco", keywords: &["busco", "eukaryote"], template: "-i {input} -l eukaryota_odb12 -o {prefix} -m genome -c {threads}" },
    ToolTemplate { tool: "busco", keywords: &["busco", "vertebrate"], template: "-i {input} -l vertebrata_odb12 -o {prefix} -m genome -c {threads}" },
    ToolTemplate { tool: "busco", keywords: &["busco", "augustus"], template: "-i {input} -l vertebrata_odb12 -o {prefix} -m genome --augustus --long -c {threads}" },
    ToolTemplate { tool: "busco", keywords: &["busco", "miniprot"], template: "-i {input} -l bacteria_odb12 -o {prefix} -m genome --miniprot -c {threads}" },
    ToolTemplate { tool: "mosdepth", keywords: &["mosdepth", "coverage", "depth"], template: "-t {threads} {prefix} {input}" },
    ToolTemplate { tool: "mosdepth", keywords: &["mosdepth", "bed"], template: "-t {threads} --by {input2} {prefix} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["stats", "statistic", "seqkit stat"], template: "stats {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["seq", "transform", "convert"], template: "seq -o {output} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["grep", "search sequence", "seqkit grep"], template: "grep -f {input2} {input} -o {output}" },
    ToolTemplate { tool: "seqkit", keywords: &["sample", "random sample"], template: "sample -n 1000 -o {output} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["fq2fa", "fastq to fasta"], template: "fq2fa -o {output} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["split2", "split sequence"], template: "split2 -s 1000 -O {output_dir} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["rmdup", "remove duplicate"], template: "rmdup -s -o {output} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["sort", "sort sequence"], template: "sort -l -o {output} {input}" },
    ToolTemplate { tool: "seqkit", keywords: &["replace", "replace name"], template: "replace -p '(.+)' -r '{args}' -o {output} {input}" },
    ToolTemplate { tool: "seqtk", keywords: &["sample", "random sample"], template: "sample -s 100 {input} 0.1" },
    ToolTemplate { tool: "seqtk", keywords: &["seq", "convert", "transform"], template: "seq -a {input}" },
    ToolTemplate { tool: "seqtk", keywords: &["subseq", "extract subsequence"], template: "subseq {input} {input2}" },
    ToolTemplate { tool: "seqtk", keywords: &["trimfq", "trim fastq"], template: "trimfq -q 0.05 {input}" },
    ToolTemplate { tool: "seqtk", keywords: &["comp", "composition"], template: "comp {input}" },
    ToolTemplate { tool: "seqtk", keywords: &["mergepe", "merge paired"], template: "mergepe {read1} {read2}" },
    ToolTemplate { tool: "bamtools", keywords: &["stats", "statistic"], template: "stats {input}" },
    ToolTemplate { tool: "bamtools", keywords: &["count", "count read"], template: "count -in {input}" },
    ToolTemplate { tool: "bamtools", keywords: &["filter", "filter bam"], template: "filter -in {input} -out {output} -tag \"NM:<5\"" },
    ToolTemplate { tool: "bamtools", keywords: &["merge", "merge bam"], template: "merge -out {output} -in {inputs}" },
    ToolTemplate { tool: "bamtools", keywords: &["split", "split bam"], template: "split -in {input} -reference" },
    ToolTemplate { tool: "bamtools", keywords: &["convert", "convert bam"], template: "convert -in {input} -format json" },
    ToolTemplate { tool: "bamtools", keywords: &["index"], template: "index -in {input}" },
    ToolTemplate { tool: "bamtools", keywords: &["coverage"], template: "coverage -in {input}" },
    ToolTemplate { tool: "bamtools", keywords: &["header"], template: "header -in {input}" },
    ToolTemplate { tool: "igvtools", keywords: &["totdf", "to tdf", "tdf"], template: "toTDF {input} {output} hg38" },
    ToolTemplate { tool: "homer", keywords: &["maketagdirectory", "tag directory"], template: "makeTagDirectory {output_dir} {input}" },
    ToolTemplate { tool: "homer", keywords: &["findpeaks", "peak calling"], template: "findPeaks {input} -style factor -o {output}" },
    ToolTemplate { tool: "homer", keywords: &["annotatepeaks", "annotate peak"], template: "annotatePeaks.pl {input} hg38 > {output}" },
    ToolTemplate { tool: "homer", keywords: &["findmotifsgenome", "find motif"], template: "findMotifsGenome.pl {input} hg38 {output_dir}" },
    ToolTemplate { tool: "homer", keywords: &["makeucscfile"], template: "makeUCSCfile {input} -o {output}" },
    ToolTemplate { tool: "homer", keywords: &["pos2bed"], template: "pos2bed.pl {input} > {output}" },
    ToolTemplate { tool: "homer", keywords: &["mergepeaks"], template: "mergePeaks {inputs} > {output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbduk", "quality filter", "adapter"], template: "bbduk.sh -in {input} -out {output} -qtrim r -trimq 20" },
    ToolTemplate { tool: "bbtools", keywords: &["bbmap", "align", "map"], template: "bbmap.sh -in1 {read1} -in2 {read2} -ref {reference} -out {output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbmerge", "merge read", "extend"], template: "bbmerge.sh -in1 {read1} -in2 {read2} -out {output}" },
    ToolTemplate { tool: "bbtools", keywords: &["reformat", "convert format"], template: "reformat.sh -in {input} -out {output}" },
    ToolTemplate { tool: "bbtools", keywords: &["dedupe", "remove duplicate"], template: "dedupe.sh -in {input} -out {output}" },
    ToolTemplate { tool: "bbtools", keywords: &["bbsplit", "separate organism"], template: "bbsplit.sh -in1 {read1} -in2 {read2} -ref {reference} -basename {prefix}" },
    ToolTemplate { tool: "tabix", keywords: &["tabix", "index", "tbi"], template: "-p vcf {input}" },
    ToolTemplate { tool: "tabix", keywords: &["tabix", "query", "region"], template: "{input} {args}" },
    ToolTemplate { tool: "vep", keywords: &["vep", "annotate variant", "variant effect"], template: "-i {input} -o {output} --cache --assembly GRCh38" },
    ToolTemplate { tool: "vep", keywords: &["vep", "offline", "cache"], template: "-i {input} -o {output} --offline --cache --dir_cache {args}" },
    ToolTemplate { tool: "vcfanno", keywords: &["vcfanno", "annotate vcf"], template: "{input} {input2}" },
    ToolTemplate { tool: "nextflow", keywords: &["run", "execute pipeline"], template: "run {args} -profile docker" },
    ToolTemplate { tool: "nextflow", keywords: &["clean", "clean cache"], template: "clean -but last" },
    ToolTemplate { tool: "nextflow", keywords: &["pull", "download pipeline"], template: "pull {args}" },
    ToolTemplate { tool: "nextflow", keywords: &["list", "list pipeline"], template: "list" },
    ToolTemplate { tool: "nextflow", keywords: &["version"], template: "-version" },
    ToolTemplate { tool: "snakemake", keywords: &["run", "execute workflow"], template: "--cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["dry-run", "dryrun"], template: "--dry-run --cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["configfile"], template: "--configfile {input} --cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["profile"], template: "--profile {args} --cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["unlock"], template: "--unlock" },
    ToolTemplate { tool: "snakemake", keywords: &["dag", "workflow graph"], template: "--dag --cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["singularity", "container"], template: "--use-singularity --cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["forcerun"], template: "--forcerun {args} --cores {threads}" },
    ToolTemplate { tool: "snakemake", keywords: &["rerun-incomplete"], template: "--rerun-incomplete --cores {threads}" },
    ToolTemplate { tool: "meme", keywords: &["meme", "discover motif", "find motif"], template: "-dna -mod zoops -nmotifs 5 -minw 6 -maxw 20 -oc {output_dir} {input}" },
    ToolTemplate { tool: "meme", keywords: &["fimo", "scan motif", "motif occurrence"], template: "fimo --oc {output_dir} {input} {input2}" },
    ToolTemplate { tool: "meme", keywords: &["streme", "de novo motif"], template: "streme -dna -oc {output_dir} {input}" },
    ToolTemplate { tool: "meme", keywords: &["tomtom", "compare motif"], template: "tomtom -oc {output_dir} {input} {input2}" },
    ToolTemplate { tool: "meme", keywords: &["ame", "motif enrichment"], template: "ame --oc {output_dir} {input} {input2}" },
    ToolTemplate { tool: "meme", keywords: &["revcomp", "reverse complement"], template: "-dna -revcomp -mod zoops -nmotifs 5 -oc {output_dir} {input}" },
    ToolTemplate { tool: "methyldackel", keywords: &["extract", "methylation extract"], template: "extract {reference} {input} -o {output}" },
    ToolTemplate { tool: "methyldackel", keywords: &["mbias", "bias plot"], template: "mbias {reference} {input} {output}" },
    ToolTemplate { tool: "chromap", keywords: &["chromap", "index", "build index"], template: "index -r {reference} -o {output}" },
    ToolTemplate { tool: "chromap", keywords: &["chromap", "map", "align"], template: "map -r {reference} -x {index} -1 {read1} -2 {read2} -o {output}" },
    ToolTemplate { tool: "chromap", keywords: &["chromap", "single-end"], template: "map -r {reference} -x {index} -1 {input} -o {output}" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge", "classify", "taxonomic"], template: "-x {database} -U {input} -S {output} --report-file {report}" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge-build", "build database"], template: "centrifuge-build -p {threads} {input} {prefix}" },
    ToolTemplate { tool: "centrifuge", keywords: &["centrifuge-kreport", "kraken report"], template: "centrifuge-kreport -x {database} {input} > {output}" },
    ToolTemplate { tool: "metabat2", keywords: &["metabat2", "bin", "binning"], template: "-i {input} -o {output_dir}" },
    ToolTemplate { tool: "metabat2", keywords: &["jgi_summarize_bam_contig_depths", "depth file"], template: "jgi_summarize_bam_contig_depths -o {output} {inputs}" },
    ToolTemplate { tool: "iqtree2", keywords: &["iqtree2", "phylogenetic tree", "tree"], template: "-s {input} -m MFP -T {threads} -pre {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["iqtree2", "model finder", "model selection"], template: "-s {input} -m MFP -T {threads}" },
    ToolTemplate { tool: "iqtree2", keywords: &["iqtree2", "bootstrap", "ultrafast"], template: "-s {input} -m MFP -B 1000 -T {threads} -pre {prefix}" },
    ToolTemplate { tool: "iqtree2", keywords: &["iqtree2", "partition", "partition model"], template: "-s {input} -p {input2} -m MFP+MERGE -T {threads} -pre {prefix}" },
    ToolTemplate { tool: "muscle", keywords: &["muscle", "align", "multiple alignment"], template: "-align {input} -output {output}" },
    ToolTemplate { tool: "muscle", keywords: &["muscle", "super5", "large alignment"], template: "-super5 {input} -output {output}" },
    ToolTemplate { tool: "mafft", keywords: &["mafft", "align", "multiple alignment"], template: "--auto {input} > {output}" },
    ToolTemplate { tool: "mafft", keywords: &["mafft", "linsi", "accurate"], template: "--linsi {input} > {output}" },
    ToolTemplate { tool: "mafft", keywords: &["mafft", "fftns", "fast"], template: "--fftns {input} > {output}" },
    ToolTemplate { tool: "fasttree", keywords: &["fasttree", "nucleotide tree", "dna"], template: "-nt {input} > {output}" },
    ToolTemplate { tool: "fasttree", keywords: &["fasttree", "protein tree", "wag"], template: "-wag {input} > {output}" },
    ToolTemplate { tool: "fasttree", keywords: &["fasttree", "lg model"], template: "-lg {input} > {output}" },
    ToolTemplate { tool: "gatk", keywords: &["splitncigarreads", "split ncigar"], template: "SplitNCigarReads -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["variantfiltration", "variant filter"], template: "VariantFiltration -R {reference} -V {input} -O {output} --filter-expression 'QD < 2.0' --filter-name 'LowQD'" },
    ToolTemplate { tool: "gatk", keywords: &["combinevariants", "combine variant"], template: "CombineVariants -R {reference} -V {inputs} -o {output}" },
    ToolTemplate { tool: "gatk", keywords: &["genotypegvcfs", "genotype gvcf"], template: "GenotypeGVCFs -R {reference} -V {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["splitcigarreads", "split cigar"], template: "SplitNCigarReads -R {reference} -I {input} -O {output}" },
    ToolTemplate { tool: "gatk", keywords: &["collecthsmetrics", "hs metric", "hybrid selection"], template: "CollectHsMetrics -I {input} -O {output} -R {reference} -TI {input2} -BI {args}" },
    ToolTemplate { tool: "picard", keywords: &["collectrnaseqmetrics", "rna-seq metric"], template: "CollectRnaSeqMetrics -I {input} -O {output} -REF_FLAT {input2} -STRAND_SPECIFICITY NONE" },
    ToolTemplate { tool: "picard", keywords: &["collectgcbiasmetrics", "gc bias"], template: "CollectGcBiasMetrics -I {input} -O {output} -R {reference} -CHART {args}" },
    ToolTemplate { tool: "picard", keywords: &["collectinsertsizemetrics", "insert size"], template: "CollectInsertSizeMetrics -I {input} -O {output} -H {histogram} -R {reference}" },
    ToolTemplate { tool: "picard", keywords: &["collectalignmentsummarymetrics", "alignment metric"], template: "CollectAlignmentSummaryMetrics -I {input} -O {output} -R {reference}" },
    ToolTemplate { tool: "picard", keywords: &["mergebamalignment", "merge alignment"], template: "MergeBamAlignment -R {reference} -UNMAPPED {input} -ALIGNED {input2} -O {output}" },
    ToolTemplate { tool: "picard", keywords: &["revertsam", "revert sam"], template: "RevertSam -I {input} -O {output}" },
    ToolTemplate { tool: "picard", keywords: &["createsequencedictionary", "sequence dictionary"], template: "CreateSequenceDictionary -R {reference}" },
    ToolTemplate { tool: "picard", keywords: &["collectmultiplemetrics", "multiple metric"], template: "CollectMultipleMetrics -I {input} -O {prefix} -R {reference}" },
    ToolTemplate { tool: "bcftools", keywords: &["consensus"], template: "consensus -f {reference} -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["roh", "run of homozygosity"], template: "roh {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["call", "call variant", "variant calling"], template: "call -vm -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "snp only", "type snp"], template: "view -v snps -O z -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["view", "sample", "extract sample"], template: "view -s {args} -o {output} {input}" },
    ToolTemplate { tool: "bcftools", keywords: &["annotate", "add id", "annotate id"], template: "annotate -I +'%CHROM\\_%POS\\_%REF\\_%FIRST_ALT' -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["collate"], template: "collate -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["cat"], template: "cat -o {output} {inputs}" },
    ToolTemplate { tool: "samtools", keywords: &["reheader"], template: "reheader {input}" },
    ToolTemplate { tool: "samtools", keywords: &["addreplacerg", "replace read group"], template: "addreplacerg -r '@RG\\tID:s1\\tSM:s1\\tLB:lib1\\tPL:ILLUMINA' -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "bam to sam", "convert to sam"], template: "view -h -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "sam to bam", "convert to bam"], template: "view -bS -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "count read", "count"], template: "view -c {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "mapped only", "filter unmapped"], template: "view -b -F 4 -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "unmapped only"], template: "view -b -f 4 -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["view", "region", "chromosome"], template: "view -b {input} {args} -o {output}" },
    ToolTemplate { tool: "samtools", keywords: &["sort", "queryname", "sort by name"], template: "sort -n -@ {threads} -o {output} {input}" },
    ToolTemplate { tool: "samtools", keywords: &["markdup", "remove duplicate", "remove_dup"], template: "markdup -r -f {metrics} {input} {output}" },
    ToolTemplate { tool: "samtools", keywords: &["markdup", "metrics"], template: "markdup -f {metrics} {input} {output}" },
    ToolTemplate { tool: "bedtools", keywords: &["complement"], template: "complement -i {input} -g {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["window"], template: "window -a {input} -b {input2} -w 1000" },
    ToolTemplate { tool: "bedtools", keywords: &["groupby"], template: "groupby -i {input} -g 1 -c 4 -o sum" },
    ToolTemplate { tool: "bedtools", keywords: &["bamtofastq"], template: "bamtofastq -i {input} -fq {out1} -fq2 {out2}" },
    ToolTemplate { tool: "bedtools", keywords: &["bedtobam"], template: "bedtobam -i {input} -g {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["bamtobed"], template: "bamtobed -i {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["shift"], template: "shift -i {input} -g {input2} -m 100 -s 0" },
    ToolTemplate { tool: "bedtools", keywords: &["flank"], template: "flank -i {input} -g {input2} -l 1000 -r 1000" },
    ToolTemplate { tool: "bedtools", keywords: &["jaccard"], template: "jaccard -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["reldist"], template: "reldist -a {input} -b {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["random"], template: "random -l 100 -n 1000 -g {input}" },
    ToolTemplate { tool: "bedtools", keywords: &["shuffle"], template: "shuffle -i {input} -g {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["annotate"], template: "annotate -i {input} -files {input2}" },
    ToolTemplate { tool: "bedtools", keywords: &["multiinter"], template: "multiinter -i {inputs}" },
    ToolTemplate { tool: "bedtools", keywords: &["cluster"], template: "cluster -i {input} -d 1000" },
    ToolTemplate { tool: "bedtools", keywords: &["map"], template: "map -a {input} -b {input2} -c 4 -o sum" },
    ToolTemplate { tool: "bedtools", keywords: &["expand"], template: "expand -i {input} -c 4" },
    ToolTemplate { tool: "bedtools", keywords: &["split"], template: "split -i {input} -p {prefix}" },
    ToolTemplate { tool: "bedtools", keywords: &["pairtobed"], template: "pairtobed -abam {input} -b {input2} -type either" },
    ToolTemplate { tool: "bedtools", keywords: &["unionbedg"], template: "unionbedg -i {inputs}" },
];

pub fn find_best_template(tool: &str, task: &str) -> Option<&'static str> {
    find_best_template_with_score(tool, task).map(|(template, _)| template)
}

pub fn find_best_template_with_score(tool: &str, task: &str) -> Option<(&'static str, i32)> {
    let task_lower = task.to_ascii_lowercase();
    let tool_lower = tool.to_lowercase();

    let generic_keywords: &[&str] = &[
        "run", "script", "execute", "process", "start", "launch", "perform",
        "trim", "filter", "convert", "sort", "merge", "split", "index",
        "align", "map", "assemble", "annotate", "call", "count", "quantify",
        "build", "create", "generate", "download", "upload", "search", "find",
        "remove", "delete", "extract", "compress", "decompress", "archive",
        "connect", "remote", "sync", "transfer", "copy", "move",
        "quality", "qc", "report", "summary", "statistics", "stats",
        "profile", "classify", "detect", "identify", "compare", "combine",
        "install", "update", "version", "check", "list", "show", "display",
        "format", "output", "input", "data", "file", "result",
    ];

    let mut best_match: Option<(&'static str, i32)> = None;

    for tmpl in TOOL_TEMPLATES {
        if tmpl.tool.to_lowercase() != tool_lower {
            continue;
        }

        let mut score: i32 = 0;
        for keyword in tmpl.keywords {
            let kw_lower = keyword.to_ascii_lowercase();
            let is_generic = generic_keywords.iter().any(|gk| kw_lower == *gk);

            if task_lower.contains(&kw_lower) {
                if is_generic && kw_lower.len() <= 5 {
                    score += 3;
                } else if is_generic {
                    score += 5;
                } else if kw_lower.contains('-') || kw_lower.contains('_') {
                    score += 25;
                } else if kw_lower.len() >= 8 {
                    score += 22;
                } else {
                    score += 15;
                }
            }
            let kw_words: Vec<&str> = kw_lower.split_whitespace().collect();
            if kw_words.len() > 1 {
                let all_present = kw_words.iter().all(|w| task_lower.contains(w));
                if all_present {
                    let specific_words = kw_words.iter().filter(|w| w.len() >= 4 && !generic_keywords.contains(&**w)).count();
                    if specific_words >= 2 {
                        score += 20;
                    } else if specific_words == 1 {
                        score += 10;
                    } else {
                        score += 3;
                    }
                }
                let any_present = kw_words.iter().any(|w| task_lower.contains(w) && w.len() >= 3);
                if any_present && !all_present {
                    let specific_any = kw_words.iter().any(|w| w.len() >= 4 && task_lower.contains(w) && !generic_keywords.contains(&*w));
                    if specific_any {
                        score += 8;
                    } else {
                        score += 2;
                    }
                }
            }
            for kw_part in kw_lower.split(|c: char| c == '-' || c == '_') {
                if kw_part.len() >= 4 && task_lower.contains(kw_part) && !generic_keywords.contains(&kw_part) {
                    score += 8;
                } else if kw_part.len() >= 3 && task_lower.contains(kw_part) {
                    score += 2;
                }
            }
        }

        if score > 0 {
            match best_match {
                Some((_, best_score)) if score <= best_score => {}
                _ => best_match = Some((tmpl.template, score)),
            }
        }
    }

    best_match
}

pub fn fill_template(
    template: &str,
    task: &str,
    task_values: &TaskValues,
) -> Vec<String> {
    let mut result = template.to_string();
    let mut used_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    let input_file = task_values.input_files.first().cloned().unwrap_or_else(|| "input.bam".to_string());
    used_files.insert(input_file.to_ascii_lowercase());

    let output_file = task_values.output_files.first().cloned().unwrap_or_else(|| "output.bam".to_string());

    let reference = task_values.reference_files.first().cloned()
        .or_else(|| task_values.input_files.iter()
            .find(|f| {
                let fl = f.to_ascii_lowercase();
                fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
                    || fl.ends_with(".fa.gz") || fl.ends_with(".fasta.gz") || fl.ends_with(".fna.gz")
                    || fl.ends_with(".mfa") || fl.ends_with(".ffa")
                    || fl.contains("genome") || fl.contains("reference") || fl.contains("ref.")
            })
            .cloned())
        .unwrap_or_else(|| "reference.fa".to_string());
    used_files.insert(reference.to_ascii_lowercase());

    let read1 = task_values.read_files.first().cloned()
        .or_else(|| task_values.input_files.iter()
            .find(|f| {
                let fl = f.to_ascii_lowercase();
                (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".fq.gz") || fl.ends_with(".fastq.gz") || fl.ends_with(".gz"))
                    && !used_files.contains(&fl)
            })
            .cloned())
        .unwrap_or_else(|| "reads_1.fq".to_string());
    used_files.insert(read1.to_ascii_lowercase());

    let read2 = task_values.read_files.iter()
        .nth(1)
        .cloned()
        .or_else(|| task_values.input_files.iter()
            .filter(|f| {
                let fl = f.to_ascii_lowercase();
                (fl.ends_with(".fq") || fl.ends_with(".fastq") || fl.ends_with(".fq.gz") || fl.ends_with(".fastq.gz") || fl.ends_with(".gz"))
                    && !used_files.contains(&fl)
            })
            .nth(0)
            .cloned())
        .unwrap_or_else(|| "reads_2.fq".to_string());
    used_files.insert(read2.to_ascii_lowercase());

    let genome_dir = task_values.genome_dirs.first().cloned()
        .or_else(|| {
            let task_lower = task.to_ascii_lowercase();
            if task_lower.contains("genomegenerate") || task_lower.contains("generate genome") || task_lower.contains("genome index") {
                task_values.output_files.first().cloned()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "/path/to/star_index".to_string());

    let output_dir = task_values.output_files.first()
        .map(|f| {
            let path = std::path::Path::new(f);
            let parent = path.parent();
            match parent {
                Some(p) if !p.as_os_str().is_empty() => p.to_string_lossy().to_string(),
                _ => {
                    let stem = path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "output".to_string());
                    let stem_lower = stem.to_ascii_lowercase();
                    if stem_lower.ends_with(".bam") || stem_lower.ends_with(".vcf")
                        || stem_lower.ends_with(".fa") || stem_lower.ends_with(".fq")
                        || stem_lower.ends_with(".txt") || stem_lower.ends_with(".tsv") {
                        let base = stem.trim_end_matches(".bam").trim_end_matches(".vcf")
                            .trim_end_matches(".fa").trim_end_matches(".fq")
                            .trim_end_matches(".txt").trim_end_matches(".tsv");
                        if base.is_empty() { "output_dir/".to_string() } else { format!("{}/", base) }
                    } else {
                        format!("{}/", stem)
                    }
                }
            }
        })
        .unwrap_or_else(|| "output_dir/".to_string());
    let output_prefix = task_values.output_files.first()
        .map(|f| {
            let path = std::path::Path::new(f);
            path.parent()
                .map(|p| format!("{}/", p.to_string_lossy()))
                .unwrap_or_else(|| "".to_string())
        })
        .unwrap_or_else(|| "".to_string());

    let thread_num = task_values.numbers.iter()
        .find(|n| {
            let v: f64 = n.parse().unwrap_or(0.0);
            v >= 1.0 && v <= 128.0
        })
        .cloned()
        .unwrap_or_else(|| "4".to_string());

    let index_name = task_values.reference_files.first()
        .or_else(|| task_values.input_files.iter()
            .find(|f| {
                let fl = f.to_ascii_lowercase();
                fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna")
            }))
        .map(|f| {
            let stem = std::path::Path::new(f)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "genome_index".to_string());
            let stem = stem.trim_end_matches(".fa").trim_end_matches(".fasta").trim_end_matches(".fna");
            if stem.is_empty() { "genome_index".to_string() } else { stem.to_string() }
        })
        .unwrap_or_else(|| "genome_index".to_string());

    let input2 = task_values.input_files.iter()
        .filter(|f| {
            let fl = f.to_ascii_lowercase();
            !used_files.contains(&fl)
        })
        .nth(0)
        .cloned()
        .unwrap_or_else(|| "input2.bed".to_string());
    used_files.insert(input2.to_ascii_lowercase());

    let annotation = task_values.annotation_files.first().cloned()
        .or_else(|| task_values.input_files.iter()
            .find(|f| {
                let fl = f.to_ascii_lowercase();
                (fl.ends_with(".gtf") || fl.ends_with(".gff") || fl.ends_with(".gff3")
                    || fl.ends_with(".gtf.gz") || fl.ends_with(".gff.gz") || fl.ends_with(".gff3.gz"))
                    && !used_files.contains(&fl)
            })
            .cloned())
        .unwrap_or_else(|| "annotation.gtf".to_string());
    used_files.insert(annotation.to_ascii_lowercase());

    let database = task_values.database_files.first().cloned()
        .or_else(|| task_values.input_files.iter()
            .find(|f| {
                let fl = f.to_ascii_lowercase();
                (fl.ends_with(".dmnd") || fl.ends_with(".ndb") || fl.ends_with(".msh")
                    || fl.ends_with(".k2d") || fl.contains("db") || fl.contains("index"))
                    && !used_files.contains(&fl)
            })
            .cloned())
        .unwrap_or_else(|| "database".to_string());
    used_files.insert(database.to_ascii_lowercase());

    let metrics = task_values.output_files.iter()
        .find(|f| {
            let fl = f.to_ascii_lowercase();
            fl.ends_with(".txt") || fl.contains("metric")
        })
        .cloned()
        .or_else(|| {
            let prefix = task_values.output_files.first()
                .map(|f| std::path::Path::new(f).file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "output".to_string()))
                .unwrap_or_else(|| "output".to_string());
            Some(format!("{}_metrics.txt", prefix))
        })
        .unwrap_or_else(|| "metrics.txt".to_string());

    let prefix = task_values.output_files.first()
        .map(|f| {
            std::path::Path::new(f)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "output".to_string())
        })
        .unwrap_or_else(|| "output".to_string());

    let all_inputs = task_values.input_files.join(" ");
    let all_outputs = task_values.output_files.join(" ");

    let input_dir = task_values.input_files.first()
        .map(|f| {
            let path = std::path::Path::new(f);
            let parent = path.parent();
            match parent {
                Some(p) if !p.as_os_str().is_empty() => p.to_string_lossy().to_string(),
                _ => ".".to_string(),
            }
        })
        .unwrap_or_else(|| ".".to_string());

    let report_html = format!("{}.html", prefix);
    let report_json = format!("{}.json", prefix);
    let out1 = format!("{}_1.fq", prefix);
    let out2 = format!("{}_2.fq", prefix);
    let unpaired1 = format!("{}_unpaired_1.fq", prefix);
    let unpaired2 = format!("{}_unpaired_2.fq", prefix);
    let sai1 = format!("{}.sai", read1);
    let sai2 = format!("{}.sai", read2);
    let sai = format!("{}.sai", read1);

    let k_value = task_lower_extract_k(task);

    let replacements: Vec<(&str, String)> = vec![
        ("{input}", input_file.clone()),
        ("{inputs}", all_inputs),
        ("{input1}", input_file.clone()),
        ("{input2}", input2.clone()),
        ("{input_dir}", input_dir),
        ("{output}", output_file.clone()),
        ("{outputs}", all_outputs),
        ("{reference}", reference.clone()),
        ("{read1}", read1.clone()),
        ("{read2}", read2.clone()),
        ("{genome_dir}", genome_dir.clone()),
        ("{output_dir}", output_dir.clone()),
        ("{output_prefix}", output_prefix.clone()),
        ("{index}", index_name.clone()),
        ("{annotation}", annotation.clone()),
        ("{database}", database.clone()),
        ("{metrics}", metrics.clone()),
        ("{prefix}", prefix.clone()),
        ("{threads}", thread_num.clone()),
        ("{control}", input2.clone()),
        ("{normal}", input_file.clone()),
        ("{tumor}", input2.clone()),
        ("{report}", metrics.clone()),
        ("{histogram}", metrics.clone()),
        ("{recal}", output_file.clone()),
        ("{known_sites}", input2.clone()),
        ("{config}", input2.clone()),
        ("{map_file}", input2.clone()),
        ("{chromsizes}", input2.clone()),
        ("{overlaps}", input2.clone()),
        ("{hmm}", input2.clone()),
        ("{proteins}", output_file.clone()),
        ("{report_html}", report_html),
        ("{report_json}", report_json),
        ("{out1}", out1),
        ("{out2}", out2),
        ("{unpaired1}", unpaired1),
        ("{unpaired2}", unpaired2),
        ("{sai1}", sai1),
        ("{sai2}", sai2),
        ("{sai}", sai),
        ("{jar_file}", "tool.jar".to_string()),
        ("{args}", String::new()),
        ("{url}", "https://example.com".to_string()),
        ("{source}", input_file.clone()),
        ("{destination}", output_file.clone()),
        ("{directory}", ".".to_string()),
        ("{pattern}", "*.bam".to_string()),
        ("{path}", input_file.clone()),
        ("{accession}", "SRR123456".to_string()),
        ("{branch}", "main".to_string()),
        ("{genome}", reference.clone()),
        ("{file_list}", input2.clone()),
        ("{phenotype}", input2.clone()),
        ("{regions}", input2.clone()),
        ("{t2g}", input2.clone()),
        ("{motif}", input2.clone()),
        ("{motif_db}", input2.clone()),
        ("{blacklist}", input2.clone()),
        ("{bam}", input_file.clone()),
        ("{output1}", output_file.clone()),
        ("{output2}", input2.clone()),
        ("{gff3}", input2.clone()),
        ("{bed}", input2.clone()),
        ("{query}", input_file.clone()),
    ];

    for (placeholder, value) in &replacements {
        result = result.replace(placeholder, value);
    }

    if result.contains("K=") || result.contains("K ") {
        if let Some(ref k_val) = k_value {
            result = result.replace("K", k_val);
        }
    }

    crate::llm::response::parse_shell_args(&result)
}

fn task_lower_extract_k(task: &str) -> Option<String> {
    let task_lower = task.to_ascii_lowercase();
    if let Some(re) = regex::Regex::new(r"K\s*=\s*(\d+)").ok() {
        for cap in re.captures_iter(&task_lower) {
            return Some(cap[1].to_string());
        }
    }
    for word in task_lower.split_whitespace() {
        if word.starts_with("k=") || word.starts_with("K=") {
            if let Some(val) = word.split('=').nth(1) {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn generate_from_template(
    tool: &str,
    task: &str,
    sdoc: &StructuredDoc,
) -> Option<Vec<String>> {
    let (template, score) = find_best_template_with_score(tool, task)?;

    if score < 8 {
        return None;
    }

    let task_values = super::task_values::extract_task_values(task);
    let args = fill_template(template, task, &task_values);

    if args.is_empty() {
        return None;
    }

    let mut result = args;

    if !sdoc.has_subcommands && !result.is_empty() {
        let first = result[0].to_ascii_lowercase();
        let looks_like_flag = first.starts_with('-');
        let looks_like_file = first.contains('.') || first.contains('/');
        let known_subcommand = sdoc.subcommands.iter().any(|s| s.to_ascii_lowercase() == first);
        let is_companion = sdoc.companion_binaries.iter().any(|s| s.to_ascii_lowercase() == first);
        let looks_like_executable = !first.contains('.')
            && !first.contains('/')
            && !first.starts_with('-')
            && first.len() >= 3
            && (first.contains('_')
                || first.contains('-')
                || first.starts_with("rscript")
                || first == "realsfs"
                || first.starts_with("bracken")
                || first.starts_with("deduplicate")
                || first.starts_with("bismark")
                || first.starts_with("medaka")
                || first.starts_with("nano")
                || first == "bowtie2-build"
                || first == "hisat2-build"
                || first == "bwa-mem2"
                || first == "convert2bed"
                || first == "combine_bracken_outputs"
                || first == "bakta_db"
                || first == "bakta_proteins"
                || first == "merge_metaphlan_tables.py"
                || first == "strainphlan"
                || first == "findmotifsgenome.pl"
                || first == "annotatepeaks.pl"
                || first == "getdifferentialpeaks"
                || first == "getdifferentialgenes.pl"
                || first == "makegenomedirectory.pl"
                || first == "pos2bed.pl"
                || first == "makeucscfile"
                || first == "run_bowtie2_for_trinity.pl"
                || first == "draw_fusions.r"
                || first == "convert_fusions_to_vcf"
                || first == "run_arriba"
                || first == "run_arriba_on_prealigned_bam"
                || first == "bismark2report"
                || first == "bismark_methylation_extractor"
                || first == "metaquast.py"
                || first == "normalize_by_kmer_coverage"
                || first == "kraken2-build"
                || first == "centrifuge-build"
                || first == "centrifuge-kreport"
                || first == "fastq-dump"
                || first == "fasterq-dump"
                || first == "sam-dump"
                || first == "prefetch"
                || first == "abidump"
                || first == "wibtotdf"
                || first == "emapper.py"
                || first == "jgi_summarize_bam_contig_depths"
                || first == "agat_convert_sp_gff2gtf"
                || first == "agat_sp_statistics"
                || first == "agat_sp_filter_gene_by_length"
                || first == "agat_convert_sp_gxf2gxf"
                || first == "agat_sp_extract_sequences"
                || first == "agat_sp_keep_longest_isoform"
                || first == "agat_sp_merge_annotations"
                || first == "agat_sp_manage_ids"
                || first == "agat_convert_sp_gff2bed"
                || first == "makeblastdb"
                || first == "blastn"
                || first == "blastp"
                || first == "blastx"
                || first == "tblastn"
                || first == "blastdbcmd"
                || first == "medaka_consensus"
                || first == "medaka_haploid_variant"
                || first == "medaka_variant"
                || first == "medaka_inference"
                || first == "medaka_sequence"
            );
        if !looks_like_flag && !looks_like_file && !known_subcommand && !looks_like_executable && !is_companion {
            result.remove(0);
        }
    }

    if result.is_empty() {
        return None;
    }

    Some(result)
}

pub fn merge_template_with_llm(
    template_args: &[String],
    llm_args: &[String],
    sdoc: &StructuredDoc,
) -> Vec<String> {
    if llm_args.is_empty() {
        return template_args.to_vec();
    }
    if template_args.is_empty() {
        return llm_args.to_vec();
    }

    let mut result: Vec<String> = Vec::new();

    let tmpl_has_sub = sdoc.has_subcommands && !template_args.is_empty()
        && !template_args[0].starts_with('-')
        && sdoc.subcommands.iter().any(|s| s.to_ascii_lowercase() == template_args[0].to_ascii_lowercase());
    let llm_has_sub = sdoc.has_subcommands && !llm_args.is_empty()
        && !llm_args[0].starts_with('-')
        && sdoc.subcommands.iter().any(|s| s.to_ascii_lowercase() == llm_args[0].to_ascii_lowercase());

    if tmpl_has_sub {
        result.push(template_args[0].clone());
    } else if llm_has_sub {
        result.push(llm_args[0].clone());
    }

    let tmpl_flags = extract_flag_value_pairs(&template_args[if tmpl_has_sub { 1 } else { 0 }..]);
    let llm_flags = extract_flag_value_pairs(&llm_args[if llm_has_sub { 1 } else { 0 }..]);

    let mut merged: Vec<(String, Option<String>)> = Vec::new();
    let mut seen_flags: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (flag, value) in &tmpl_flags {
        let flag_key = flag.split('=').next().unwrap_or(flag).to_ascii_lowercase();
        if !seen_flags.contains(&flag_key) {
            if let Some((_, llm_value)) = llm_flags.iter().find(|(f, _)| {
                f.split('=').next().unwrap_or(f).to_ascii_lowercase() == flag_key
            }) {
                let is_placeholder = value.as_ref().map_or(false, |v| {
                    let vl = v.to_ascii_lowercase();
                    vl.starts_with("/path/to/")
                        || vl == "input.bam" || vl == "output.bam"
                        || vl == "reads_1.fq" || vl == "reads_2.fq"
                        || vl == "reference.fa" || vl == "input2.bed"
                        || vl == "annotation.gtf" || vl == "database"
                        || vl == "metrics.txt" || vl == "tool.jar"
                        || vl == "https://example.com" || vl == "*.bam"
                        || vl == "SRR123456" || vl == "main"
                        || vl == "genome_index" || vl == "."
                        || vl == "output" || vl == "input"
                        || vl.ends_with("_metrics.txt")
                        || vl.ends_with(".sai")
                        || (vl.ends_with(".html") && vl.contains("output"))
                        || (vl.ends_with(".json") && vl.contains("output"))
                        || vl.ends_with("_unpaired_1.fq")
                        || vl.ends_with("_unpaired_2.fq")
                        || vl.ends_with("_1.fq") && vl.contains("output")
                        || vl.ends_with("_2.fq") && vl.contains("output")
                });
                if is_placeholder && llm_value.is_some() {
                    merged.push((flag.clone(), llm_value.clone()));
                } else {
                    merged.push((flag.clone(), value.clone()));
                }
            } else {
                merged.push((flag.clone(), value.clone()));
            }
            seen_flags.insert(flag_key);
        }
    }

    let known_flags: std::collections::HashSet<String> = sdoc.flag_catalog.iter()
        .flat_map(|e| {
            let mut flags = vec![e.flag.to_ascii_lowercase()];
            if let Some(ref alt) = e.alt_form {
                flags.push(alt.to_ascii_lowercase());
            }
            flags
        })
        .collect();

    for (flag, value) in &llm_flags {
        let flag_key = flag.split('=').next().unwrap_or(flag).to_ascii_lowercase();
        if seen_flags.contains(&flag_key) {
            continue;
        }
        let flag_base = flag_key.trim_start_matches('-');
        let is_known = known_flags.contains(&flag_key)
            || known_flags.iter().any(|kf| kf.trim_start_matches('-') == flag_base);
        if is_known || value.is_some() {
            merged.push((flag.clone(), value.clone()));
        }
        seen_flags.insert(flag_key);
    }

    for (flag, value) in &merged {
        if flag.contains('=') {
            result.push(flag.clone());
        } else {
            result.push(flag.clone());
            if let Some(v) = value {
                result.push(v.clone());
            }
        }
    }

    let tmpl_positional = extract_positional_args(template_args, sdoc);
    let llm_positional = extract_positional_args(llm_args, sdoc);
    let result_str = result.join(" ").to_ascii_lowercase();

    for pos in &tmpl_positional {
        if !result_str.contains(&pos.to_ascii_lowercase()) {
            result.push(pos.clone());
        }
    }
    for pos in &llm_positional {
        if !result_str.contains(&pos.to_ascii_lowercase()) {
            result.push(pos.clone());
        }
    }

    if !result.is_empty() && !result[0].starts_with('-') && !tmpl_has_sub {
        let first_lower = result[0].to_ascii_lowercase();
        let is_known_sub = sdoc.subcommands.iter().any(|s| s.to_ascii_lowercase() == first_lower);
        if !is_known_sub {
            result.remove(0);
        }
    }

    result
}

fn extract_flag_value_pairs(args: &[String]) -> Vec<(String, Option<String>)> {
    let mut pairs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            if arg.contains('=') {
                pairs.push((arg.clone(), None));
            } else if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                pairs.push((arg.clone(), Some(args[i + 1].clone())));
                i += 1;
            } else {
                pairs.push((arg.clone(), None));
            }
        }
        i += 1;
    }
    pairs
}

fn extract_positional_args(args: &[String], sdoc: &StructuredDoc) -> Vec<String> {
    let mut positionals = Vec::new();
    let mut i = 0;
    let skip_first = sdoc.has_subcommands && !args.is_empty() && !args[0].starts_with('-');
    if skip_first {
        i = 1;
    }
    while i < args.len() {
        if args[i].starts_with('-') {
            if !args[i].contains('=') && i + 1 < args.len() && !args[i + 1].starts_with('-') {
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        positionals.push(args[i].clone());
        i += 1;
    }
    positionals
}

pub fn remove_duplicate_flags_vec(args: &[String]) -> Vec<String> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            let flag_key = arg.split('=').next().unwrap_or(arg).to_ascii_lowercase();
            if seen.contains(&flag_key) {
                if !arg.contains('=') && i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            seen.insert(flag_key);
            result.push(arg.clone());
            if !arg.contains('=') && i + 1 < args.len() && !args[i + 1].starts_with('-') {
                result.push(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else {
            result.push(arg.clone());
        }
        i += 1;
    }
    result
}
