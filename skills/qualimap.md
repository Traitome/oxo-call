---
name: qualimap
category: qc
description: Qualimap — Java-based BAM/RNA-seq quality control tool; bamqc, rnaseq, multi-bamqc, and counts modules for comprehensive sequencing QC
tags: [qualimap, bamqc, rnaseq, qc, bam, sequencing, coverage, gc-bias, rna-seq, ngs]
author: oxo-call built-in
source_url: "http://qualimap.conesalab.org/"
---

## Concepts
- Qualimap is a Java-based QC tool; the `qualimap` command wraps `java -jar qualimap.jar`; always set `-Djava.io.tmpdir` and `--java-mem-size`.
- **Modules**: `bamqc` (alignment QC), `rnaseq` (RNA-seq QC with gene annotations), `multi-bamqc` (aggregate report across samples), `counts` (count data QC).
- `bamqc` output directory: by default `<input_bam>_qualimap_results/` in CWD; specify with `-outdir`.
- `-outformat PDF:HTML` generates both PDF and HTML reports (default is HTML).
- `--java-mem-size` controls heap memory (e.g. `--java-mem-size 4G`); critical for large BAM files; overrides default 1200M.
- `-nt` (number of threads) parallelises coverage computation; set to match available CPUs.
- `rnaseq` module requires a gene annotation in GTF/GFF format (`-gtf`) and the BAM file (`-bam`).
- `rnaseq` `-p` flag: `non-strand-specific` (default), `strand-specific-forward`, or `strand-specific-reverse`; must match library preparation protocol.
- `multi-bamqc` aggregates results from multiple `bamqc` runs; input is a TSV file listing BAM names and their `bamqc` output directories.
- Output HTML reports include: coverage uniformity, GC bias, insert size, mapping quality, duplication rate, and per-chromosome depth.
- Qualimap can run without a reference genome; the `--genome-gc-distr` flag enables GC bias calculation against pre-computed GC profiles.
- `--skip-duplicated` excludes duplicated reads from analysis.
- `--skip-dup-mode` controls which duplicates to skip: 0 (none), 1 (only optical), 2 (all).
- `-gff` provides feature file for targeted region analysis.
- `--paint-chromosome-limits` adds chromosome boundary lines in coverage plots.
- `-os` outputs statistics only (no graphics generation).

## Pitfalls
- CRITICAL: qualimap ARGS must start with a subcommand (bamqc, rnaseq, multi-bamqc, counts, clustering, comp-counts) — never with flags like -bam, -outdir, -gtf. The subcommand ALWAYS comes first.
- running `qualimap bamqc` on a very large (>100 GB) BAM without `--java-mem-size` causes OutOfMemoryError; always set at least `--java-mem-size 8G` for WGS data.
- Qualimap requires the BAM to be sorted and indexed; run `samtools sort` and `samtools index` first.
- `rnaseq` module strand direction (`-p`) must match the library; wrong setting leads to incorrect read assignment rates and misleading reports.
- Not specifying `-outdir` causes all output to go into the current directory with a name derived from the BAM; collisions can occur in batch runs.
- Qualimap GUI launch (`qualimap` without subcommand) requires a display; use `qualimap bamqc` (command-line mode) on HPC or headless servers.
- The `-gd HUMAN` or `-gd MOUSE` option computes GC bias against a built-in GC distribution; if your organism is not included, use `--genome-gc-distr` with a custom BED file.
- Java temporary directory `/tmp` may fill up for large genomes; always set `-Djava.io.tmpdir=/scratch/tmp` on HPC.
- `--skip-duplicated` changes coverage calculations; use consistently across samples.
- `--skip-dup-mode` 2 removes all duplicates; may significantly reduce apparent coverage.
- `-os` mode is faster but provides no visualizations; use for quick statistics.

## Examples

### run BAM QC on a sorted, indexed BAM
**Args:** `bamqc -bam sorted.bam --java-mem-size 8G -nt 8 -outdir qualimap_results/`
**Explanation:** --java-mem-size 8G prevents OutOfMemoryError on large BAMs; -nt 8 parallelises coverage; output HTML/PDF in qualimap_results/

### run RNA-seq QC with strand information
**Args:** `rnaseq -bam sorted.bam -gtf annotation.gtf -p strand-specific-reverse --java-mem-size 8G -outdir qualimap_rnaseq/`
**Explanation:** -p strand-specific-reverse matches dUTP/RF library prep; -gtf provides gene coordinates for proper read assignment rates

### run multi-sample QC and aggregate report
**Args:** `multi-bamqc -d samples.txt --java-mem-size 4G -outdir multiqc_qualimap/`
**Explanation:** samples.txt lists: sample_name<TAB>bamqc_outdir per line; generates a comparative HTML report across all samples

### run BAM QC with GC bias correction (human)
**Args:** `bamqc -bam sorted.bam -gd HUMAN --java-mem-size 8G -nt 8 -outdir qualimap_gc/`
**Explanation:** -gd HUMAN computes GC bias using the human reference GC distribution; useful for identifying PCR amplification bias

### generate PDF and HTML reports
**Args:** `bamqc -bam sorted.bam -outformat PDF:HTML --java-mem-size 4G -outdir qualimap_output/`
**Explanation:** -outformat PDF:HTML generates both formats; PDF is useful for archiving; HTML is better for interactive exploration

### run BAM QC on whole-genome sequencing data
**Args:** `bamqc -bam wgs.bam -gd HUMAN --java-mem-size 16G -nt 16 --paint-chromosome-limits -outdir wgs_qualimap/`
**Explanation:** --paint-chromosome-limits adds chromosome boundary lines in coverage plot; -nt 16 for WGS parallelism; 16G heap for whole-genome BAMs

### count QC for differential expression count matrices
**Args:** `counts -d counts.txt -c 2 -s C -outdir counts_qc/`
**Explanation:** counts module assesses quality of count data; -c specifies the column index of count values; -s C for condition column; outputs distribution plots

### run BAM QC excluding duplicates
**Args:** `bamqc -bam sorted.bam --skip-duplicated --java-mem-size 8G -nt 8 -outdir qualimap_nodup/`
**Explanation:** --skip-duplicated excludes PCR/optical duplicates from coverage analysis

### run BAM QC with feature file
**Args:** `bamqc -bam sorted.bam -gff targets.gff --java-mem-size 8G -nt 8 -outdir qualimap_targets/`
**Explanation:** -gff provides feature file for targeted region analysis; useful for capture/exome data

### run BAM QC statistics only (no graphics)
**Args:** `bamqc -bam sorted.bam -os --java-mem-size 8G -nt 8 -outdir qualimap_stats/`
**Explanation:** -os outputs statistics only; faster execution without generating plots

### run BAM QC with chromosome limits
**Args:** `bamqc -bam sorted.bam --paint-chromosome-limits --java-mem-size 8G -nt 8 -outdir qualimap_limits/`
**Explanation:** --paint-chromosome-limits adds chromosome boundary lines in coverage plots

### run BAM QC skipping only optical duplicates
**Args:** `bamqc -bam sorted.bam --skip-dup-mode 1 --java-mem-size 8G -nt 8 -outdir qualimap_nooptical/`
**Explanation:** --skip-dup-mode 1 skips only optical duplicates; keeps PCR duplicates for coverage

### run RNA-seq QC with forward strand specificity
**Args:** `rnaseq -bam sorted.bam -gtf annotation.gtf -p strand-specific-forward --java-mem-size 8G -outdir qualimap_fwd/`
**Explanation:** -p strand-specific-forward for forward-stranded libraries (e.g., Ligation method)
