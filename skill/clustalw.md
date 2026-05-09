---
name: clustalw
category: sequence-alignment
description: CLUSTALW is a progressive multiple sequence alignment tool for DNA and protein sequences. It performs global alignments using the Needleman-Wunsch algorithm with iterative refinement options and produces alignments in multiple output formats including ALN, PHYLIP, and FASTA.
tags:
- multiple-sequence-alignment
- dna-alignment
- protein-alignment
- progressive-alignment
- bioinformatics
author: AI-generated
source_url: https://www.clustal.org/clustalw/
---

## Concepts

- **Input formats**: CLUSTALW accepts FASTA, GCG, PIR, NBRF/EMBL, and SwissProt formats for both DNA and protein sequences. Sequences must be in valid biological sequence format; mixed nucleotide/amino acid input within a single file causes alignment failure.
- **Output files**: The tool generates two output files per alignment run: an `.aln` file containing the aligned sequences with conservation markers (`*`, `:`, `.`) and an `.dnd` file containing the New Hampshire-formatted guide tree used to construct the alignment.
- **Progressive alignment algorithm**: CLUSTALW builds alignments iteratively using a UPGMA-based guide tree. Sequences are aligned in order of decreasing similarity to the alignment, meaning gaps introduced early in the process (for distant sequences) become permanent anchors that cannot be shifted by later refinement.
- **Gap penalty system**: Three gap parameters control insertion and deletion placement: `-gapopen` (penalty for opening a new gap), `-gapext` (penalty per residue in an extended gap), and `-gapdist` (penalty for establishing a gap adjacent to an existing gap). High `gapopen` relative to `gapext` favors fewer long gaps; low `gapopen` favors many short gaps.
- **Iteration modes**: The `-iteration` flag enables refinement cycles that reassess the alignment after initial construction. `ALIGNMENT` iteration recomputes the full alignment with modified gap penalties; `TREE` iteration rebuilds only the guide tree. Iteration reduces alignment score in some cases by correcting early placement errors.

## Pitfalls

- **Omitting the `-type` flag for protein sequences**: CLUSTALW defaults to DNA mode when `-type` is not specified. In DNA mode, protein-specific substitution matrices (BLOSUM, Gonnet) are unavailable, and the tool may misalign conserved protein domains by treating codons as independent nucleotides. Always specify `-type=PROTEIN` for amino acid sequences.
- **Setting `-gapopen` too low for divergent sequences**: Low gap opening penalties (below 10 for proteins) cause excessive gap insertion in conserved regions, destroying structural or functional motif alignment. The default gapopen values (15 for proteins, 10 for DNA) are calibrated for sequences with 30-70% identity.
- **Using incorrect substitution matrices for DNA**: The default DNA substitution matrix uses simple match/mismatch scoring, which is appropriate only for very closely related sequences. For distantly related DNA (e.g., promoter regions across species), specify `-dnamatrix=mBLOSUM62` or `-dnamatrix=FAM20` to enable transversion-aware scoring; otherwise, transitions between purines or pyrimidines are scored identically to transversions, producing biased alignments.
- **Specifying `-maxseqlen` smaller than the actual sequence length**: If a sequence exceeds the specified maximum length, CLUSTALW silently truncates sequences at the boundary, producing a partial alignment missing critical residues. Verify that `-maxseqlen` equals or exceeds the longest input sequence.
- **Expecting `-iteration=ALIGNMENT` to always improve scores**: Iterative refinement may increase the alignment score (make it numerically larger) while decreasing biological accuracy by shifting gaps out of frame or disrupting conserved domains. Always visually inspect output alignments for conserved motifs, especially when iterating.

## Examples

### Perform a basic multiple sequence alignment of DNA sequences in FASTA format
**Args:** `-infile=sequences.fasta -outfile=aligned.aln`
**Explanation:** This aligns all sequences in `sequences.fasta` using default DNA parameters (gap opening penalty 10, gap extension 6.66, simple DNA substitution matrix) and writes the alignment to `aligned.aln` in CLUSTALW format.

### Align protein sequences with Gonnet substitution matrix and reduced gap penalties
**Args:** `-infile=proteins.fasta -type=PROTEIN -matrix=GONNET -gapopen=8 -gapext=0.2`
**Explanation:** The Gonnet matrix provides position-specific substitution scores optimized for twilight-zone sequences (below 25% identity), and the reduced gap penalties allow more flexible insertion in variable loop regions common in distantly related proteins.

### Generate both alignment and guide tree files
**Args:** `-infile=sequences.fasta -outfile=alignment.aln -newtree=guide.dnd`
**Explanation:** CLUSTALW builds the alignment using a guide tree; storing the tree enables reproduction of the alignment order and facilitates comparison when adding sequences to an existing alignment with `-tree` mode.

### Produce output in PHYLIP format for phylogenetic analysis software
**Args:** `-infile=sequences.fasta -outfile=phylip_output.phy -output=PHYLIP`
**Explanation:** Phylogenetic tools such as RAxML, PHYML, and PAUP* require PHYLIP interleaved format, which CLUSTALW produces via the output flag; this eliminates the need for external format conversion tools.

### Perform iterative refinement to correct alignment of highly divergent sequences
**Args:** `-infile=divergent.fasta -type=PROTEIN -iteration=ALIGNMENT -numiters=5`
**Explanation:** The iterative alignment mode re-aligns unaligned or weakly aligned fragments five times, reassigning gaps based on updated position-specific penalties; this often corrects gross misalignments in sequence pairs with less than 20% identity.

### Align sequences and reorder output to match input file order
**Args:** `-infile=mixed_order.fasta -outfile=reordered.aln -outorder=INPUT`
**Explanation:** The default output order follows the guide tree, which reorders sequences by similarity; specifying INPUT order preserves the original file order, which is required when the alignment must correspond to a phenotype or sample metadata table.

### Set maximum sequence length to prevent silent truncation of long sequences
**Args:** `-infile=long_sequences.fasta -maxseqlen=15000`
**Explanation:** Default maximum sequence length is 10,000 residues; sequences longer than this are truncated without warning, and the resulting alignment will be missing terminal domains or UTRs that may be biologically relevant.

### Control output case formatting for downstream processing
**Args:** `-infile=sequences.fasta -outfile=uppercase.aln -case=UPPER`
**Explanation:** Some sequence analysis tools require uppercase input; converting to uppercase within CLUSTALW avoids information loss that can occur with external text conversion tools that strip whitespace or modify special characters.

---
name: clustalw
category: sequence-alignment
description: CLUSTALW is a progressive multiple sequence alignment tool for DNA and protein sequences. It performs global alignments using a guide tree approach and outputs aligned sequences in CLUSTALW ALN format along with conservation annotations.
tags:
- multiple-sequence-alignment
- progressive-alignment
- dna-alignment
- protein-alignment
- bioinformatics
author: AI-generated
source_url: https://www.clustal.org/ omega/
---

## Concepts

- **Input sequence formats**: CLUSTALW accepts sequences in FASTA, GCG, PIR, NBRF/EMBL, and SwissProt formats. Each input format has specific requirements: FASTA files use a header line starting with `>` followed by the sequence on subsequent lines, while GCG and PIR formats include length information in the file header that CLUSTALW validates against the actual sequence length.
- **Progressive alignment with guide tree**: The algorithm first calculates a distance matrix using pairwise alignments, then builds a UPGMA guide tree. Sequences are aligned progressively in the order determined by the tree, meaning more closely related sequences are aligned first and their alignment constrains where distant sequences are inserted. This hierarchical approach makes gap placement dependent on alignment order.
- **Gap penalty parameterization**: Gap opening (`-gapopen`) and gap extension (`-gapext`) penalties control insertion and deletion modeling. The default protein values (15.0 and 6.66) reflect the approximate energetic cost of creating a gap versus extending one; changing these values fundamentally alters the alignment topology, especially in low-similarity regions where the guide tree is most uncertain.
- **Substitution matrix selection**: The choice of substitution matrix (BLOSUM, Gonnet, PAM) determines how mismatched residues are scored. BLOSUM matrices are preferred for sequences with less than 40% identity (twilight zone), while PAM matrices suit closer homologs. Using an inappropriate matrix for the similarity level produces alignments biased toward or against substitutions.
- **Output format controls**: The `-output` parameter supports CLUSTALW, GCG, PHYLIP, FASTA, and PIR formats. PHYLIP format uses sequential or interleaved notation; CLUSTALW generates interleaved output by default, which some phylogenetics tools cannot parse without conversion.

## Pitfalls

- **Forgetting the `-type` parameter for protein sequences**: Without `-type=PROTEIN`, CLUSTALW defaults to DNA mode and uses nucleotide substitution scoring, which is inappropriate for amino acid sequences. Protein-specific matrices like BLOSUM30 or BLOSUM62 are only available in protein mode, and the gap penalties are not calibrated for amino acid insertion costs.
- **Setting gap penalties incorrectly for the sequence similarity regime**: Gap opening penalties below 10 or above 25 produce degenerate alignments for most sequences: too low causes excessive gaps in conserved regions, while too high prevents legitimate insertions in variable loops. The default values (15.0/6.66 for proteins, 10.0/0.2 for DNA) are reliable starting points.
- **Specifying a substitution matrix that is unavailable for the sequence type**: CLUSTALW silently falls back to the default matrix when an unsupported matrix is requested. For example, specifying a BLOSUM matrix in DNA mode produces no error message but uses simple match/mismatch scoring instead, yielding an alignment with incorrect transversion penalties.
- **Running alignment with sequences shorter than the minimum informative length**: CLUSTALW cannot produce statistically meaningful alignments when input sequences are shorter than 20-30 residues, as pairwise alignment scores do not differ significantly from random. Short sequences produce arbitrary alignments dominated by end-gap effects.
- **Not specifying output format when compatibility with other tools is required**: The default CLUSTALW ALN format includes conservation annotations (`*`, `:`) that some parsers cannot handle. External tools expecting strict FASTA or PHYLIP format will fail to parse ALN output without conversion.

## Examples

### Align DNA sequences using default parameters
**Args:** `-infile=sequences.fasta -outfile=aligned.aln`
**Explanation:** This runs CLUSTALW with default DNA gap penalties (10.0 opening, 0.2 extension) and the IUB DNA substitution matrix, producing a standard alignment and guide tree in the current directory.

### Align protein sequences with BLOSUM30 matrix for distantly related sequences
**Args:** `-infile=proteins.fasta -type=PROTEIN -matrix=BLOSUM30 -outfile=distal.aln`
**Explanation:** BLOSUM30 is appropriate for sequences with less than 30% identity, as it assigns lower scores to mismatches than higher BLOSUM matrices, producing more conservative alignments that avoid false homologies in the twilight zone.

### Generate PHYLIP-format output for phylogenetic analysis software
**Args:** `-infile=sequences.fasta -type=PROTEIN -output=PHYLIP -outfile=analysis.phy`
**Explanation:** PHYLIP sequential format is required by many phylogenetics packages including RAxML and PHYML; the output flag ensures compatibility without manual format conversion.

### Adjust gap separation penalty to prevent clustering of gaps
**Args:** `-infile=proteins.fasta -type=PROTEIN -gapopen=12 -gapext=0.5 -gapdist=8`
**Explanation:** Increasing the gap separation penalty discourages gaps from clustering in the same region, which is beneficial for alignments where insertions occur in distinct structural elements rather than as runs.

### Align sequences with uppercase output and input order preserved
**Args:** `-infile=sequences.fasta -outfile=uppercase.aln -case=UPPER -outorder=INPUT`
**Explanation:** Uppercase output simplifies downstream text processing, and preserving input order maintains correspondence between alignment positions and sample metadata or experimental labels.

### Use tree-based iteration to improve alignment of moderately divergent sequences
**Args:** `-infile=moderate.fasta -type=PROTEIN -iteration=TREE`
**Explanation:** Tree-based iteration rebuilds the guide tree from the current alignment and realigns only the sequences that have changed most, which is more computationally efficient than full realignment while correcting cases where early gap placement propagated errors through the tree.

### Perform full iterative refinement for low-quality alignments
**Args:** `-infile=noisy.fasta -type=DNA -iteration=ALIGNMENT -numiters=3`
**Explanation:** Full iterative refinement performs up to three cycles of alignment reconstruction with gap penalty adjustment, which helps recover alignment quality when initial parameters are poorly suited for the input set.

### Generate a guide tree file without re-running pairwise alignment
**Args:** `-infile=sequences.fasta -newtree=tree.dnd`
**Explanation:** CLUSTALW calculates the distance matrix during every run; saving the tree allows you to add new sequences to an existing alignment using the `-tree` parameter without recalculating distances for the original set.

### Align sequences with reduced maximum length to reduce memory usage
**Args:** `-infile=long_genomic.fasta -maxseqlen=5000`
**Explanation:** Limiting maximum sequence length reduces memory allocation, which is useful when aligning fragments of genomic sequences or when working on systems with limited RAM.

### Set output filename for both alignment and guide tree in a single command
**Args:** `-infile=sequences.fasta -outfile=results.aln -newtree=results.dnd`
**Explanation:** Specifying both output parameters in a single run is more efficient than running CLUSTALW twice and avoids redundant pairwise distance calculations.

---
name: clustalw
category: sequence-alignment
description: CLUSTALW is a progressive multiple sequence alignment tool for nucleic acid and protein sequences, using iterative refinement and a guide tree approach to produce globally optimized alignments with conservation annotations.
tags:
- multiple-sequence-alignment
- dna
- protein
- progressive-alignment
- bioinformatics
author: AI-generated
source_url: https://www.clustal.org/
---

## Concepts

- **Progressive alignment architecture**: CLUSTALW builds multiple alignments by first computing a pairwise distance matrix through all-against-all global alignment, then constructing a neighbor-joining guide tree. Sequences are added to the alignment in order of increasing distance from each other, meaning that the alignment of the first pair of sequences constrains where all subsequent sequences are inserted.
- **Substitution matrix defaults by molecule type**: In protein mode, the default is Gonnet matrix; in DNA mode, the default is a simple 1/-1 match/mismatch matrix. The matrix choice is the primary determinant of how substitutions are weighted and must be appropriate for the evolutionary distance of the input set.
- **Gap penalty architecture**: Separate opening and extension penalties model the cost of creating versus lengthening gaps. A high gapopen-to-gapext ratio produces sparse, long gaps; a low ratio produces dense, short gaps. These parameters interact with the substitution matrix to determine which alignment has the lowest total cost.
- **Output format specifications**: The `-output` parameter controls alignment format. PHYLIP format uses sequential notation without conservation markers, which is required by most phylogenetics packages but loses annotation information available in CLUSTALW format.
- **Iteration modes**: Three iteration modes refine alignments: TREE iteration rebuilds the guide tree and re-aligns variable regions, ALIGNMENT iteration performs full iterative realignment, and NONE disables refinement entirely.

## Pitfalls

- **Attempting to align sequences with no detectable homology**: CLUSTALW will produce an output file even when input sequences share no significant similarity, but the alignment represents arbitrary residue pairing without biological meaning. Always compute a pairwise alignment score or E-value before attempting multiple alignment.
- **Conflicting sequence type and substitution matrix specifications**: Requesting BLOSUM or Gonnet matrices while in DNA mode causes CLUSTALW to silently use the default DNA matrix instead of reporting an error. The resulting alignment uses inappropriate scoring that does not reflect biological substitution patterns.
- **Underestimating memory requirements for large alignments**: Alignments with more than 100 sequences of moderate length consume significant RAM for the distance matrix. Running on memory-constrained systems may cause segmentation faults or silent truncation of the alignment.
- **Output format incompatibility with downstream tools**: The default CLUSTALW ALN format includes line-wrapped sequences and conservation annotations that many parsers cannot handle correctly. Always verify