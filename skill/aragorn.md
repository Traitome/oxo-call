---
name: aragorn
category: Genome Annotation
description: A tool for detecting tRNA and tmRNA genes in genomic DNA sequences using sequence covariance models and BLAST-like scoring.
tags: [tRNA, tmRNA, RNA gene prediction, genome annotation, non-coding RNA, taxonomic annotation]
author: AI-generated
source_url: https://lowelodev.com/aragorn/
---

## Concepts

- **Input Format**: Aragorn accepts raw genomic DNA sequences in FASTA format. The input must contain only A, T, G, C, and U nucleotides (RNA sequences will be converted to DNA internally). Sequences should represent the full genomic strand including both forward and reverse complement regions where tRNA genes may reside.
  
- **Output Interpretation**: The tool reports predicted tRNA genes with their anticodons, amino acid specificities, intron positions (for eukaryotic tRNAs), and genomic coordinates. Each prediction includes a covariance score indicating the model's confidence—scores above 20 generally indicate reliable predictions, while scores below 10 often correspond to pseudogenes or trivial matches.

- **Strand Specification**: By default, aragorn searches only the forward strand unless the `-t` flag is used to enable two-strand scanning. Many users miss this and fail to detect reverse-complement tRNA genes located on the negative strand, resulting in incomplete annotation of the genome.

- **Taxonomic Selection**: The `-s` flag specifies the genetic code (standard, mitochondrial, etc.) which determines how anticodons map to amino acids. Using an incorrect genetic code leads to incorrect amino acid assignments for the predicted tRNAs—for example, using the standard code for mitochondrial genomes misassigns AGA and AGG as arginine rather than stop codons.

## Pitfalls

- **Forgetting to scan the reverse strand**: Without `-t`, only the forward DNA strand is scanned. tRNA genes encoded on the opposite strand will be completely missed, leading to incomplete tRNAome annotation. This is especially problematic in bacterial genomes where tRNAs are often distributed across both strands.

- **Confusing sequence types**: Inputting protein (amino acid) sequences instead of nucleic acid sequences will produce meaningless output or errors. Ensure your FASTA files contain nucleotide sequences—the tool does not automatically translate input to nucleotides.

- **Using an incorrect genetic code**: The `-s` flag selects the translation table. Mitochondrial genomes require `-s mt` or similar options; otherwise, tRNA anticodons will be mapped to the wrong amino acids, corrupting downstream analyses that depend on accurate tRNA charging predictions.

- **Ignoring low-scoring predictions**: Predictions with covariance scores below 10 often represent pseudogenes or fragmented remnants. Treating all predictions equally can inflate the count of functional tRNA genes and mislead evolutionary analyses.

- **Missing intron annotation**: For eukaryotic genomes, failing to check the intron status in the output means pseudo genes with introns may be misidentified as functional. Verify intron positions indicated in the output columns.

## Examples

### Scan a bacterial genome for all tRNA genes on both strands
**Args:** `-t -o output.txt input.fasta`
**Explanation:** The `-t` flag enables scanning both forward and reverse DNA strands to ensure all encoded tRNAs are detected, while `-o` directs the detailed results to a file for downstream analysis.

### Find tmRNA genes in a bacterial genome
**Args:** `-m input.fasta`
**Explanation:** The `-m` flag tells aragorn to specifically search for tmRNA genes (transfer-messenger RNA used in trans-translation), which are distinct from standard tRNA predictions and omitted by default.

### Generate predictions in GFF format for genome browsers
**Args:** `-gff -o genes.gff input.fasta`
**Explanation:** The `-gff` flag outputs coordinates and annotations in GFF3 format, enabling direct import into genome browsers like JBrowse or IGV for visualization alongside other genomic features.

### Use the bacterial genetic code for a prokaryotic genome
**Args:** `-s bact -i input.fasta`
**Explanation:** The `-s bact` option applies the bacterial genetic code, ensuring correct anticodon-to-amino-acid mapping—for instance, correctly identifying ATA as isoleucine rather than methionine used in the standard code.

### Build a custom scoring database from known tRNA sequences
**Args:** `-build -i reference_tRNAs.fasta`
**Explanation:** The `-build` companion binary is invoked to construct a custom covariance model from the provided reference tRNA sequences, allowing species-specific or family-specific detection optimizations.

### Filter predictions to keep only high-confidence tRNAs
**Args:** `-t -f -o filtered.txt input.fasta`
**Explanation:** The `-f` flag applies more stringent filtering to the predictions before output, discarding low-covariance matches that likely represent pseudogenes or spurious hits in the genome.

### Scan an entire chromosome and report summary statistics
**Args:** `-w -o summary.txt chr1.fasta`
**Explanation:** The `-w` flag outputs a summary including totals of detected tRNA types, amino acid distribution, and genomic locations in a convenient report format rather than line-by-line gene details.