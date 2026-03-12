# Supplementary Materials

## Supplementary Table S1. Built-in workflow templates

| Template | Assay type | Description | Pipeline steps | Expanded tasks (3 samples) | Parse time (μs) | Formats |
|----------|-----------|-------------|----------------|---------------------------|----------------|---------|
| rnaseq | Transcriptomics | Bulk RNA-seq analysis | fastp → STAR → featureCounts → MultiQC | 13 | 761 | TOML, SMK, NF |
| wgs | Genomics | Whole-genome sequencing variant calling | fastp → BWA-MEM2 → GATK BQSR → HaplotypeCaller | 13 | 932 | TOML, SMK, NF |
| atacseq | Epigenomics | Chromatin accessibility | fastp → Bowtie2 → Picard dedup → blacklist → MACS3 | 11 | 840 | TOML, SMK, NF |
| chipseq | Epigenomics | Histone/TF ChIP-seq | fastp → Bowtie2 → MarkDup → blacklist → MACS3 → bigWig | 19 | 1017 | TOML, SMK, NF |
| metagenomics | Metagenomics | Shotgun taxonomic classification | fastp → host removal → Kraken2 → Bracken | 9 | 743 | TOML, SMK, NF |
| methylseq | Epigenomics | Whole-genome bisulfite sequencing | Trim Galore → Bismark → dedup → methylation extraction | – | – | TOML, SMK, NF |
| scrnaseq | Single-cell | 10x Chromium 3' scRNA-seq | fastp → STARsolo (10x v3) → EmptyDrops → QC | 9 | 827 | TOML, SMK, NF |
| amplicon16s | Metagenomics | 16S amplicon sequencing | cutadapt → fastp → DADA2 ASV → SILVA taxonomy | – | – | TOML, SMK, NF |
| longreads | Genomics | Long-read de novo assembly | NanoQ QC → Flye → Medaka polishing → QUAST | 11 | 830 | TOML, SMK, NF |

TOML = native .oxo.toml format; SMK = Snakemake; NF = Nextflow DSL2. Parse times from oxo-bench (200 runs averaged). Expanded tasks counted for 3-sample wildcard expansion.

---

## Supplementary Table S2. Complete evaluation task suite (70 tasks)

The full list of evaluation tasks used in the oxo-bench benchmark suite, exported from `crates/oxo-bench/src/bench/llm.rs`. Each task specifies a tool, natural-language description, required patterns for accuracy scoring, and analytical category.

| Category | Tool | Task description | Required patterns |
|----------|------|-----------------|-------------------|
| alignment | bwa | align reads.fastq to ref.fa with 8 threads | mem; -t 8 |
| alignment | bwa | build BWA index for genome.fa | index |
| alignment | bwa-mem2 | align paired reads with read group | mem; -R |
| alignment | bowtie2 | align paired reads with 4 threads | -x; -1; -2 |
| alignment | bowtie2 | build bowtie2 index | bowtie2-build |
| alignment | minimap2 | align long reads with 16 threads | -t |
| alignment | minimap2 | splice-aware ONT cDNA mapping | -ax splice |
| alignment | hisat2 | align paired reads with 8 threads | -x; -1; -2 |
| alignment | STAR | align paired reads, coord-sorted BAM | --genomeDir; --readFilesIn; SortedByCoordinate |
| alignment | STAR | generate genome index with 8 threads | --runMode genomeGenerate; --genomeFastaFiles; --sjdbGTFfile |
| qc | fastp | trim paired reads with adapter auto-detection | --in1; --in2; --detect_adapter_for_pe |
| qc | fastp | trim single-end, filter <50bp | --in1; --length_required |
| qc | fastqc | QC with 4 threads | -t; -o |
| qc | trimmomatic | paired-end trim with sliding window | PE; LEADING; TRAILING; SLIDINGWINDOW |
| qc | cutadapt | trim TruSeq adapters, discard <20bp | -a; -o; -m |
| qc | multiqc | aggregate QC reports | -o |
| sam-bam | samtools | sort by coordinate | sort; -o |
| sam-bam | samtools | index sorted BAM | index |
| sam-bam | samtools | view mapped reads as BAM | view; -F 4; -b |
| sam-bam | samtools | flagstat statistics | flagstat |
| sam-bam | samtools | merge BAMs | merge |
| sam-bam | samtools | depth information | depth |
| sam-bam | samtools | extract region chr1 | view; chr1 |
| sam-bam | picard | mark duplicates | MarkDuplicates; INPUT |
| interval-ops | bedtools | intersect peaks with genes | intersect; -a; -b |
| interval-ops | bedtools | genome coverage bedGraph | genomecov; -ibam; -bg |
| interval-ops | bedtools | sort BED | sort; -i |
| interval-ops | bedtools | merge overlapping intervals | merge; -i |
| variant-calling | gatk | gVCF mode HaplotypeCaller | HaplotypeCaller; -ERC GVCF |
| variant-calling | gatk | BQSR application | ApplyBQSR; -R |
| variant-calling | gatk | combine gVCFs | CombineGVCFs |
| variant-calling | bcftools | call SNVs/indels | call; -m |
| variant-calling | bcftools | filter by QUAL/DP | filter; QUAL |
| variant-calling | bcftools | VCF statistics | stats |
| variant-calling | freebayes | call with min mapping quality | -f; --min-mapping-quality |
| variant-calling | deepvariant | call WGS variants | --model_type; --ref; --reads |
| structural-variants | delly | call SVs | call; -g |
| structural-variants | manta | configure and run SV calling | --bam; --referenceFasta |
| quantification | featureCounts | count paired-end | -a; -p |
| quantification | featureCounts | strand-specific counting | -a; -s 2 |
| quantification | salmon | quant paired reads | quant; -1; -2 |
| quantification | salmon | build index | index; -t |
| quantification | kallisto | quant with bootstraps | quant; -i; -b |
| quantification | htseq-count | count reverse strand | -s reverse |
| quantification | stringtie | assemble with annotation | -G; -p |
| metagenomics | kraken2 | classify paired reads | --db; --paired; --report |
| metagenomics | bracken | estimate abundance | -d; -i; -r 150 |
| metagenomics | metaphlan | profile metagenome | --input_type; --nproc; -o |
| metagenomics | megahit | assemble metagenome | -1; -2; -t; -o |
| metagenomics | spades | meta mode assembly | --meta |
| epigenomics | macs3 | narrow peak calling | callpeak; -t; -c; -g hs |
| epigenomics | macs3 | broad peak calling | callpeak; --broad |
| epigenomics | deeptools | bamCoverage RPKM | bamCoverage; --normalizeUsing RPKM |
| epigenomics | deeptools | plotHeatmap from matrix | plotHeatmap; -m |
| epigenomics | bismark | bisulfite-seq alignment | --genome; -1; -2 |
| single-cell | cellranger | count 3' gene expression | count; --fastqs; --transcriptome |
| single-cell | STARsolo | 10x v3 alignment | --soloType; --genomeDir |
| single-cell | velocyto | spliced/unspliced counting | run |
| assembly | flye | nanopore assembly | --nano-raw; --genome-size |
| assembly | hifiasm | HiFi assembly with 16 threads | -t |
| assembly | quast | evaluate with reference | -r |
| annotation | prokka | annotate bacterial genome | --genus; --outdir |
| annotation | snpEff | annotate variants hg38 | hg38 |
| annotation | blast | blastp search | blastp; -evalue; -outfmt 6 |
| annotation | diamond | sensitive search | blastp; --sensitive; -p |
| sequence-ops | seqkit | extract sequences >1000bp | seq; --min-len |
| sequence-ops | seqkit | compute statistics | stats |
| sequence-ops | seqkit | subsample 10000 reads | sample; -n |
| phylogenetics | mafft | MSA auto strategy | --auto |
| phylogenetics | iqtree2 | ML tree with bootstraps | -s; -B 1000; -m |
| format-conversion | samtools | SAM to BAM | view; -b |
| format-conversion | bcftools | BCF to VCF | view |
| format-conversion | bedtools | BAM to BED | bamtobed; -i |

---

## Supplementary Figure Legends

**Supplementary Figure S1.** Built-in skill coverage across analytical domains. Treemap representation of the 119 built-in tool skills organised by 14 analytical categories. Each category box lists the tools covered and indicates the quality threshold met (minimum examples, concepts, or pitfalls). The metagenomics category has the largest coverage (11 tools), reflecting the diversity of taxonomic classification and assembly tools in this domain.

**Supplementary Figure S2.** Command provenance and reproducibility architecture. (Top) Timeline of a single command execution showing how provenance metadata is collected at each stage: documentation hash, skill name, LLM model, and tool version. The resulting JSONL history entry contains all fields needed to trace the command back to its exact generation inputs. (Bottom) Drift detection mechanism: when a tool's --help output changes between versions, the SHA-256 documentation hash will differ, signalling that previously generated commands may need re-evaluation.

**Supplementary Figure S3.** Benchmark simulation scenarios. Nine canonical omics scenarios implemented in the oxo-bench simulation suite, spanning RNA-seq, WGS, ATAC-seq, metagenomics, ChIP-seq, methylation-seq, scRNA-seq, and 16S amplicon sequencing. Each scenario generates synthetic paired-end FASTQ data with configurable read length, error rate, and sample count. All scenarios use deterministic random seeds for full reproducibility.

**Supplementary Figure S4.** Prompt construction and LLM interaction protocol. (Top) Detailed view of how the enriched user prompt is assembled from five layers: tool identifier, skill knowledge section, raw documentation, task description, and strict output format instructions. The system prompt contains 11 rules governing response format and flag validity. (Middle) Response parsing and validation flow, including the corrective retry mechanism. (Bottom) Multi-provider abstraction showing the unified chat-completion API that supports GitHub Copilot, OpenAI, Anthropic, and Ollama.
