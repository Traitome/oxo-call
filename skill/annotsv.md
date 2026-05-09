---
name: annotsv
category: variant-annotation
description: AnnotSV annotates and ranks structural variants (SVs) and copy number variants (CNVs) from VCF or BED files, providing gene-based, regulatory, and whole-genome annotations with a clinical relevance ranking score. It supports multiple genome builds (GRCh37/hg19, GRCh38/hg38) and outputs results in TSV, VCF, or HTML formats.
tags: [structural-variants, copy-number-variants, vcf, bed, annotation, ranking, clinical, genomic-features, gene-annotation, regulatory-elements]
author: AI-generated
source_url: https://github.com/lgmgeo/AnnotSV
---

## Concepts

- **Input formats and SV types**: AnnotSV accepts VCF (version 4.0+) or BED files as input, where each line defines a structural variant with chromosome, start position, end position, and type (DEL, INS, DUP, INV, CNV, or INS/ALT). The input file must match the specified genome build; mixing hg19 and hg38 references causes incorrect mappings.
- **Annotation hierarchy and ranking score**: AnnotSV maps each SV to genomic features using interval overlap and generates a ranking score (RANK_SCORE) ranging from 0 to 100 based on clinical relevance. Exonic deletions truncating genes receive higher scores than intronic or intergenic variants, and the score considers gene essentiality, known disease associations, and HiC interacting domains.
- **Output formats and columns**: Default TSV output includes columns for variant coordinates, overlap percentages with genes/regions, RANK_SCORE, gnomAD-SV frequency, DGV matches, and gene names (HPO, OMIM, ORpha). The `-outputAll` flag adds all annotation columns, while `-outputVCF` produces a VCF with INFO fields for annotated SVs.
- **Annotation sources and configuration**: AnnotSV uses pre-built annotation databases (stored in `/AnnotationsSV_*` directory) containing genes (RefSeq), regulatory elements (Ensembl regulatory build), conserved regions (phyloP), and population frequencies (gnomAD-SV). Custom annotations can be defined in an `annotations.toml` file passed via `-annotationfile`.
- **Filtering and candidate selection**: The `-type`, `-minSize`, and `-maxSize` flags filter input variants by type and length before annotation, reducing runtime. For candidate selection in clinical reports, `-candidateFiles` specifies multiple VCFs for combined rare-variant prioritization based on cohort frequency.

## Pitfalls

- **Incorrect genome build mismatch**: Specifying the wrong genome build (e.g., `-genomeBuild GRCh37` for hg38 input) causes all coordinates to map incorrectly, producing meaningless annotations or missing overlaps entirely. Always verify the input VCF and genome build match before running.
- **Missing or malformed VCF headers**: AnnotSV requires a valid VCF header including the chromosome naming convention (chr1 vs 1). A missing or incomplete header results in silent failures where variants are skipped, leaving output files empty or truncated without clear error messages.
- **Unconfigured annotations directory**: Running AnnotSV without the annotations directory (set via `-annotationsDir`) or with a missing/corrupted annotation file produces only raw coordinates in output with no functional annotations, RANK_SCORE, or gene names, defeating the tool's purpose.
- **Memory exhaustion with large VCF files**: Annotating large VCFs with thousands of SVs without setting `-bedParsing` appropriately causes excessive memory usage. Splitting input by chromosome or using the `--splitbychr` option mitigates memory issues for whole-genome cohorts.
- **Overlapping annotations not filtered by uniqueness**: When an SV overlaps multiple genes, AnnotSV reports all overlapping genes without automatic prioritization. Users must manually interpret the output to determine the most relevant gene, especially for large deletions affecting multiple nearby genes.

## Examples

### Annotate structural variants from a VCF file with default settings
**Args:** `-VCFvariants input.vcf -genomeBuild GRCh38`
**Explanation:** Reads all SVs from the VCF, annotates them using the GRCh38 reference, and outputs a TSV file with gene-based annotations and ranking scores sorted by clinical relevance.

### Filter annotations by structural variant type
**Args:** `-VCFvariants input.vcf -genomeBuild GRCh38 -type DEL`
**Explanation:** Annotates only deletions from the input VCF, ignoring insertions, duplications, and inversions, which reduces processing time when only a specific SV type is of interest.

### Specify a custom genome assembly directory
**Args:** `-VCFvariants input.vcf -genomeBuild GRCh37 -genomeAssembly-directory /path/to/hg19/AnnotationsSV_3.4.0`
**Explanation:** Overrides the default annotations directory to use a custom GRCh37/hg19 annotation database when analyzing legacy datasets or cohorts aligned to the older genome build.

### Export annotated variants in VCF format with INFO fields
**Args:** `-VCFvariants input.vcf -genomeBuild GRCh38 -outputVCF annotated_output.vcf`
**Explanation:** Outputs a VCF file where each annotated SV includes INFO fields (ANNOTSVRANK, ANNOTSV_GENE_NAME, ANNOTSV_FREQ) rather than a separate TSV, enabling downstream pipeline compatibility with VCF-consuming tools.

### Generate a full HTML report with all annotation columns
**Args:** `-VCFvariants input.vcf -genomeBuild GRCh38 -outputAll -outputFile annotsv_report.html`
**Explanation:** Produces an HTML report containing every annotation column (gnomAD frequency, DGV links, OMIM phenotypes, conservation scores) in an interactive sortable table for clinical review and sharing with collaborators.

### Annotate from BED file and merge overlapping annotations
**Args:** `-BEDvariants cnv_calls.bed -genomeBuild GRCh38 -mergeAnnotations -outputFile cnv_annotated.tsv`
**Explanation:** Reads copy number variants from a BED file instead of VCF, merges overlapping gene annotations, and outputs a simplified TSV where each region is mapped to the most relevant overlapping gene with a single RANK_SCORE.