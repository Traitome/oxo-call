---
name: snpeff
category: variant-annotation
description: Genetic variant annotation and functional effect prediction tool
tags: [variant-annotation, vcf, effect-prediction, snp, indel, functional, gene-annotation]
author: oxo-call built-in
source_url: "https://pcingola.github.io/SnpEff/"
---

## Concepts

- SnpEff annotates variants with predicted effects (synonymous, missense, stop-gained, splice-site, etc.) using gene models.
- SnpEff requires a genome database; list available databases with 'snpeff databases | grep hg38'.
- Download databases with 'snpeff download hg38' or 'snpeff download GRCh38.105'.
- snpeff annotates VCF in-place, adding ANN INFO field with pipe-delimited effect annotations.
- SnpSift (bundled with SnpEff) filters and extracts fields from annotated VCF files.
- Use -v for verbose mode; -noLog to disable usage reporting; -stats to generate HTML statistics report.
- ANN field format: Allele|Effect|Impact|GeneName|GeneID|FeatureType|FeatureID|TranscriptBiotype|Rank|HGVS.c|HGVS.p|...
- Impact levels: HIGH (stop, frameshift), MODERATE (missense), LOW (synonymous), MODIFIER (intergenic, intronic).
- SnpEff can use canonical transcripts only with -canon flag for more focused annotations.
- Transcript Support Level (TSL) filtering with -maxTSL removes low-confidence transcripts.
- Custom interval annotations (-interval) allow adding BED/VCF-based regulatory or custom regions.
- SnpEff supports both 'ann' and 'eff' subcommands (they are synonyms for annotation).

## Pitfalls

- CRITICAL: snpeff ARGS must start with a subcommand (ann, eff, build, download, databases, dump, cds, closest, count, genes2bed, len, protein, seq, show) — never with flags like -v, -stats, -noLog. The subcommand ALWAYS comes first.
- Genome database name must exactly match: use 'snpeff databases' to find the correct identifier.
- Without -v, SnpEff may silently fail on certain inputs — always check stderr for warnings.
- SnpEff adds ANN field to INFO column — do not confuse with older EFF field from SnpEff v4.0.
- SnpEff requires Java — set heap size with -Xmx: java -Xmx8g -jar snpeff.jar or use JAVA_OPTS.
- For custom genomes, build the database first with 'snpeff build -gff3 -v MyGenome'.
- Multi-allelic VCF sites get multiple ANN entries separated by commas — normalize with bcftools before annotation.
- -download is enabled by default; use -nodownload to prevent automatic database downloads.
- -noStats disables statistics generation, which can speed up processing by ~30% for large files.
- Splice site size defaults to 2 bases; use -ss to adjust for non-canonical splice sites.
- -strict flag only uses validated transcripts, which may exclude many annotated transcripts.

## Examples

### annotate variants in a VCF file using the GRCh38 human genome database
**Args:** `ann -v GRCh38.105 variants.vcf > annotated.vcf`
**Explanation:** GRCh38.105 is the Ensembl genome database name; -v verbose; output to annotated.vcf

### annotate variants and generate an HTML statistics report
**Args:** `ann -v -stats snpeff_summary.html GRCh38.105 variants.vcf > annotated.vcf`
**Explanation:** -stats generates HTML report with variant effect distribution; useful for QC

### annotate variants from hg19/GRCh37 genome
**Args:** `ann -v hg19 variants.vcf > annotated_hg19.vcf`
**Explanation:** hg19 is the UCSC genome database identifier; use GRCh37.75 for Ensembl equivalent

### annotate variants and filter by quality for clinical reporting
**Args:** `ann -v -no-downstream -no-upstream -no-intron -no-intergenic GRCh38.105 variants.vcf > coding_annotated.vcf`
**Explanation:** -no-downstream/-no-upstream/-no-intron/-no-intergenic removes non-coding annotations; focuses on coding region effects for clinical interpretation

### build a custom SnpEff genome database from GFF3 annotation
**Args:** `build -gff3 -v MyGenome`
**Explanation:** builds a custom genome database; requires genome directory in snpEff data/ folder with sequences.fa and genes.gff3

### annotate using only canonical transcripts
**Args:** `ann -v -canon GRCh38.105 variants.vcf > annotated_canon.vcf`
**Explanation:** -canon restricts annotations to canonical transcripts only; reduces noise from alternative isoforms

### filter by transcript support level (TSL)
**Args:** `ann -v -maxTSL 3 GRCh38.105 variants.vcf > annotated_tsl.vcf`
**Explanation:** -maxTSL 3 only uses transcripts with TSL 1-3 (high confidence); filters out low-confidence predictions

### annotate with custom regulatory regions
**Args:** `ann -v -interval regulatory.bed -reg MyReg GRCh38.105 variants.vcf > annotated_reg.vcf`
**Explanation:** -interval adds custom BED regions; -reg names the regulation track; useful for annotating non-coding regulatory variants

### download a genome database
**Args:** `download -v GRCh38.105`
**Explanation:** downloads the specified genome database to the local snpEff data directory; required before first use

### list available databases
**Args:** `databases | grep -i human`
**Explanation:** lists all available pre-built databases; grep filters for human genomes; shows exact database names needed for annotation
