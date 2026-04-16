---
name: arriba
category: rna-seq
description: Fast and accurate gene fusion detection from RNA-seq data using STAR chimeric alignments
tags: [rna-seq, fusion, gene-fusion, cancer, star, transcript, structural-variant, viral-integration, itd]
author: oxo-call built-in
source_url: "https://github.com/suhrig/arriba"
---

## Concepts

- Arriba detects gene fusions from RNA-seq data aligned with STAR; it reads chimeric alignments embedded in the BAM file.
- STAR must be run with chimeric alignment flags: `--chimSegmentMin 10 --chimOutType WithinBAM --chimJunctionOverhangMin 10`. Without these, Arriba finds no fusions.
- Required inputs: STAR-aligned BAM (`-x`), genome FASTA (`-a`), gene annotation GTF (`-g`).
- Optional but recommended: blacklist file (`-b`) to filter recurrent artifacts; known fusions file (`-k`) to boost sensitivity for known cancer fusions.
- Output: `-o` fusions that passed all filters; `-O` discarded fusions for review. Both are TSV format.
- In addition to gene fusions, Arriba detects: viral integration sites, internal tandem duplications (ITDs), whole-exon duplications, intragenic inversions, enhancer hijacking events, and gene truncations.
- Strandedness (`-s`): auto (default), yes, no, reverse. Strand-specific data helps resolve ambiguous fusions.
- Use `-k` (known fusions) to boost sensitivity for cancer-type-specific recurrent fusions; the file has two tab-separated columns of gene names.
- Use `-p` (protein domains GFF3) to report retained protein domains in fusion transcripts.
- Use `-d` (WGS structural variants) to integrate genomic evidence and increase sensitivity for weakly expressed fusions.
- Wrapper scripts: `run_arriba` (full pipeline: STAR + Arriba), `run_arriba_on_prealigned_bam` (realigns only unmapped/clipped reads from existing BAM).
- Threading: `-@` sets threads for BAM reading (1 is usually optimal; >2 rarely helps).

## Pitfalls

- Arriba has no subcommands. ARGS starts directly with flags like `-x`, `-g`, `-a`. The first argument must be a dash-prefixed option.
- STAR must be run with chimeric alignment flags — regular STAR BAM without `--chimSegmentMin` and `--chimOutType WithinBAM` produces no chimeric reads for Arriba.
- Without the blacklist file (`-b`), many false positives from paralogs, read-throughs, and repetitive regions appear. Always use the latest blacklist from the Arriba release.
- The GTF annotation must match the genome FASTA used for STAR indexing. Mismatched builds produce incorrect or missing fusions.
- When STAR was run with `--chimOutType WithinBAM`, use only `-x` (omit `-c`). When STAR was run with `--chimOutType SeparateSAMold`, use both `-x` and `-c Chimeric.out.sam`.
- Arriba works on one BAM per run — one sample per invocation. For cohort analysis, run Arriba per sample then aggregate results.
- The `run_arriba_on_prealigned_bam` script realigns only clipped/unmapped reads, which is faster but may miss some fusions compared to full realignment with `run_arriba`.
- For small genomes, STAR index needs `--genomeSAindexNbases` adjusted (e.g., 14 for bacterial genomes).
- Filter tuning: `-f` disables specific filters (e.g., `-f read_through` to keep read-through fusions). Be cautious — disabling filters increases false positives.

## Examples

### run STAR with chimeric output for Arriba fusion detection
**Args:** `--runMode alignReads --genomeDir /star_index/ --readFilesIn R1.fastq.gz R2.fastq.gz --readFilesCommand zcat --runThreadN 8 --outSAMtype BAM SortedByCoordinate --outFileNamePrefix sample/ --chimSegmentMin 10 --chimOutType WithinBAM --chimJunctionOverhangMin 10 --chimScoreDropMax 30 --peOverlapNbasesMin 12`
**Explanation:** required STAR chimeric parameters; --chimSegmentMin 10 enables chimeric detection; --chimOutType WithinBAM embeds chimeric reads in the main BAM

### detect gene fusions with Arriba using blacklist and known fusions
**Args:** `-x sample/Aligned.sortedByCoord.out.bam -o fusions.tsv -O discarded.tsv -g genome.fa -a genes.gtf -b blacklist_hg38_GRCh38_v2.5.1.tsv.gz -k known_fusions.tsv`
**Explanation:** -x STAR BAM with chimeric reads; -b blacklist filters artifacts; -k known_fusions boosts sensitivity for recurrent cancer fusions

### detect fusions with protein domain annotation and WGS structural variant support
**Args:** `-x sample/Aligned.sortedByCoord.out.bam -o fusions.tsv -O discarded.tsv -g genome.fa -a genes.gtf -b blacklist.tsv -p protein_domains.gff3 -d wgs_structural_variants.tsv`
**Explanation:** -p reports retained protein domains in fusions; -d integrates WGS breakpoints to increase sensitivity for weakly expressed fusions

### run Arriba with strand-specific library protocol
**Args:** `-x sample/Aligned.sortedByCoord.out.bam -o fusions.tsv -O discarded.tsv -g genome.fa -a genes.gtf -b blacklist.tsv -s reverse`
**Explanation:** -s reverse for reverse-strand library protocol; helps resolve ambiguous fusion strand orientation; default is auto-detection

### run Arriba with relaxed filters for higher sensitivity
**Args:** `-x sample/Aligned.sortedByCoord.out.bam -o fusions.tsv -O discarded.tsv -g genome.fa -a genes.gtf -b blacklist.tsv -E 1.0 -S 1`
**Explanation:** -E 1.0 relaxes e-value threshold to allow more candidates; -S 1 requires only 1 supporting read; use for low-input or cell-free RNA

### detect fusions from pre-aligned BAM using the wrapper script
**Args:** `run_arriba_on_prealigned_bam /star_index genes.gtf genome.fa blacklist.tsv known_fusions.tsv protein_domains.gff3 8 aligned.bam`
**Explanation:** realigns only unmapped/clipped reads from existing BAM; faster than full realignment; positional args: genomeDir, GTF, FASTA, blacklist, known_fusions, protein_domains, threads, BAM

### run the full Arriba pipeline (STAR alignment + fusion detection)
**Args:** `run_arriba /star_index genes.gtf genome.fa blacklist.tsv known_fusions.tsv protein_domains.gff3 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** wrapper script that runs STAR with chimeric flags then Arriba; positional args: genomeDir, GTF, FASTA, blacklist, known_fusions, protein_domains, threads, read1, [read2]

### visualize detected fusions with Arriba draw_fusions
**Args:** `draw_fusions.R --fusions=fusions.tsv --alignments=sample/Aligned.sortedByCoord.out.bam --genome=genome.fa --annotation=genes.gtf --output=fusion_plots.pdf`
**Explanation:** R script for fusion visualization; outputs PDF with structural diagrams for each fusion; requires R with ggplot2

### convert Arriba fusions output to VCF format
**Args:** `convert_fusions_to_vcf fusions.tsv > fusions.vcf`
**Explanation:** converts Arriba TSV output to VCF format for compatibility with variant calling pipelines
