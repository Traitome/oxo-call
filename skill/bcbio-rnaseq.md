---
name: bcbio-rnaseq
category: RNA-seq Expression Analysis
description: A verified RNA-seq pipeline for quantification and differential expression analysis using bcbio-nextgen. Performs alignment-free quantification with Salmon, quality control, and statistical differential expression testing with edgeR or DESeq2.
tags:
- rna-seq
- differential-expression
- gene-expression
- quantification
- transcriptomics
- salmon
- edger
- deseq2
- alignment-free
author: AI-generated
source_url: https://bcbio-nextgen.readthedocs.io/en/latest/contents/rnaseq.html
---

## Concepts

- **Sample Configuration File**: bcbio-rnaseq requires a YAML or CSV sample configuration file defining sample names, FASTQ paths, sample groups, and the comparison matrix for differential expression. Without a properly formatted config, the pipeline cannot determine which samples belong to which condition for statistical testing.

- **Alignment-Free Quantification**: The tool uses Salmon for transcript-level quantification, which is significantly faster than traditional alignment-based methods. It requires a pre-built Salmon index for the target transcriptome (created with bcbio-rnaseq build or external Salmon indexing tools).

- **Standardized Output Structure**: Results are organized into a timestamped analysis directory containing tximport/ for summarized counts, edgeR/ and DESeq2/ for differential expression results, and a concatenated qc/ directory for QC metrics. Output files include normalized counts, log2 fold changes, p-values, and significant gene lists.

- **Integrated Quality Control**: The pipeline runs FastQC on raw reads,454 QC, and alignment-based metrics automatically. QC failures (e.g., low read quality, adapter contamination) are reported in the summary but do not stop the pipeline unless explicitly configured.

- **Genome Annotation and Index**: Accurate quantification requires the correct transcriptome annotation (GTF/GFF3) matching the Salmon index. Mismatched annotations between the index and the_gtf file used will produce meaningless count data because the tool cannot correctly map transcripts to genes.

---

## Pitfalls

- **Misconfigured Sample Comparisons**: Specifying incorrect comparison groups in the configuration file results in meaningless differential expression results. The "samples" section must accurately define which samples belong to which condition, and the "differential" section must correctly specify the comparison pairs.

- **Mismatched Transcriptome Index and GTF**: Using a Salmon index built from one annotation version while providing a GTF from a different version leads to gene-level counts that cannot be properly aggregated. Always ensure the index, GTF, and any reference files are from the same genome build.

- **Ignoring Library Strandedness**: RNA-seq libraries are often stranded; specifying the wrong strandedness (--stranded for stranded, --unstranded for unstranded) causes quantification errors where reads mapping to overlapping genes on opposite strands are counted incorrectly, inflating or deflating expression values.

- **Insufficient Memory for Salmon**: Salmon quantification loads the entire index into memory plus memory for read alignment matrices. Running with default memory on large transcriptomes (e.g., human) causes crashes; allocate at least 8GB for typical experiments and more for plant transcriptomes with many isoforms.

- **Skipping Validation of Input Files**: Running on corrupted or misnamed FASTQ files without checking that they exist and are readable produces cryptic error messages. Always verify file paths and formats with `ls -la` and `file` before initiating the pipeline.

---

## Examples

### Run RNA-seq quantification on a single sample
**Args:** `run -c samples.csv --genome mm10 --outdir results/ --focus rnaseq`
**Explanation:** This runs the full RNA-seq pipeline on samples defined in a CSV file, using the mm10 genome build and outputting results to a timestamped directory, with --focus rnaseq ensuring RNA-seq specific processing.

### Perform differential expression analysis with edgeR
**Args:** `run -c samples.csv --genome mm10 --outdir results/ --tool edger --focus rnaseq`
**Explanation:** Using the edgeR method for differential expression instead of the default DESeq2 enables specific statistical modeling appropriate for experiments with complex designs or low replication.

### Specify stranded library protocol
**Args:** `run -c samples.csv --genome hg38 --outdir results/ --stranded reverse --focus rnaseq`
**Explanation:** Specifying --stranded reverse indicates the library was prepared with a reverse stranded protocol (most common), ensuring proper assignment of reads to transcripts when genes overlap on opposite strands.

### Skip quality control filtering
**Args:** `run -c samples.csv --genome mm10 --outdir results/ --skip-qc --focus rnaseq`
**Explanation:** Using --skip-qc bypasses quality control checks and filtering, which can be useful when input data has already been pre-processed or for quick test runs, though it risks missing issues in the data.

### Customize filtering parameters for low-expressed genes
**Args:** `run -c samples.csv --genome mm10 --outdir results/ --min-count 3 --min replicates 2 --focus rnaseq`
**Explanation:** This sets a minimum count threshold of 3 and requires at least 2 replicates to retain a gene, filtering out low-expressed features that would otherwise produce false-positive differential expression calls.

### Use transcript-level quantification instead of gene-level
**Args:** `run -c samples.csv --genome mm10 --outdir results/ --txlevel --focus rnaseq`
**Explanation:** Using --txlevel performs quantification at the transcript isoform level rather than aggregating to genes, enabling detection of differential isoform usage between conditions.