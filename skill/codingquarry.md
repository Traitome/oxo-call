---
name: codingquarry
category: Gene Prediction
description: A fungal gene predictor that uses RNA-Seq data to train parameters and produce accurate gene models from genomic DNA sequences. CodingQuarry integrates RNA-Seq evidence to identify coding regions, splice sites, and transcript structures.
tags: [gene-prediction, fungi, rna-seq, genomics, ab-initio, evidence-driven, gff3]
author: AI-generated
source_url: https://github.com/Adamtar/codequarry
---

## Concepts

- **RNA-Seq Evidence Integration**: CodingQuarry uses RNA-Seq reads mapped to the genome to derive splice site positions, exon boundaries, and coding region signatures. High-quality RNA-Seq alignments dramatically improve prediction accuracy compared to ab initio methods alone.
- **GFF3 Output Format**: Predicted gene models are exported in GFF3 format, containing feature lines for gene, mRNA, exon, CDS, and UTR regions. This standard format is compatible with genome browsers, manual annotation tools, and downstream analysis pipelines.
- **Training from Evidence**: The tool trains its hidden Markov model (HMM) parameters directly on the input data by extracting statistical patterns from mapped RNA-Seq reads, making it adaptable to species with atypical codon usage or intron characteristics.
- **Input File Requirements**: The genomic sequence must be a clean FASTA file with unique sequence identifiers (.headers without spaces). RNA-Seq reads can be provided as FASTQ/FASTA files that will be internally mapped.

## Pitfalls

- **Insufficient RNA-Seq Coverage**: Using RNA-Seq data with low read depth or poor quality mapping produces fragmented gene models with many partial predictions and missed genes, reducing the biological utility of the output.
- **Incorrect Sequence Headers**: FASTA headers containing spaces or special characters cause parsing errors and silent failures in gene assignment. Always use simple headers like `>chr1` or `>scaffold_01`.
- **Memory Exhaustion with Large Genomes**: Attempting to process chromosome-scale fungal genomes (>50 Mb) without adequate RAM results in crashes or excessive runtime. Use the `-t` flag to limit thread usage on memory-constrained systems.
- **Mixed Species RNA-Seq**: Providing RNA-Seq data from a different or contaminated species introduces false splice sites and incorrectly predicts xenologous genes, corrupting the final annotation.

## Examples

### Running gene prediction with paired-end RNA-Seq data

**Args:** `-g genome.fasta -r rnaseq_1.fq -r rnaseq_2.fq -o output_dir -t 8`
**Explanation:** This runs CodingQuarry on the fungal genome using paired-end RNA-Seq reads for evidence, processing with 8 threads. The RNA-Seq read pairs provide splice junction evidence to identify introns and refine exon boundaries.

### Generating predictions with single-end RNA-Seq reads

**Args:** `-g genome.fasta -r rnaseq_single.fq -o output_dir -s Aspergillus_nidulans`
**Explanation:** Using single-end RNA-Seq data with a specified species name enables CodingQuarry to apply species-specific codon usage bias during parameter training, improving accuracy for the target fungus.

### Running with protein homology hints

**Args:** `-g genome.fasta -r rnaseq.fq -p protein_hints.fasta -o output_dir`
**Explanation:** Including protein sequences from related fungi adds homology-based hints to guide predictions in regions lacking RNA-Seq evidence, helping identify conserved genes and improving completeness.

### Limiting memory usage on systems with constrained resources

**Args:** `-g genome.fasta -r rnaseq.fq -o output_dir -t 2 -m 4GB`
**Explanation:** Restricting thread count to 2 and setting a 4 GB memory limit allows the tool to run on shared compute nodes without dominating available resources or causing out-of-memory failures.

### Producing only evidence-based predictions without training

**Args:** `-g genome.fasta -r rnaseq.fq -o output_dir --static-only`
**Explanation:** Running in static mode uses pre-trained parameters without adapting to the input data, which is useful for quick tests or when RNA-Seq quality is uncertain.