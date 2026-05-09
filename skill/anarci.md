---
name: anarci
category: Protein Sequence Analysis / Antibodies
description: A tool for numbering antibody and T-cell receptor sequences according to standard schemes (Chothia, Kabat, IMGT, Martin, EU) and identifying germline gene assignments. ANARCI analyzes Ig and TCR sequences to assign framework regions, CDRs, and germline V/D/J genes.
tags: [antibody, immunoglobulin, TCR, CDR, germline, sequence numbering, VDJ, immunology, Fv]
author: AI-generated
source_url: https://github.com/facebookresearch/ANARCI
---

## Concepts

- **Input formats**: ANARCI accepts antibody or TCR sequences in FASTA format, or as aligned sequences in Stockholm/Clustal format. Input sequences should contain only the variable domain (Fv) without constant regions for optimal numbering.
- **Numbering schemes**: ANARCI supports multiple antibody numbering schemes including Chothia (IGPT), Kabat, IMGT, Martin (Abacus), and EU (World Health Organization) numbering. The scheme determines how residues are aligned and numbered relative to conserved framework positions.
- **Output components**: ANARCI produces three output streams — (1) numbered sequences in FASTA format, (2) a CSV file containing residue assignments (framework regions, CDRs at specific positions), and (3) germline gene assignments for V, D (heavy chain only), and J genes.
- **Domain types**: ANARCI processes either immunoglobulin (IG) sequences or T-cell receptor (TCR) sequences. The domain type must be specified with `--domain` to apply the correct biological conventions and numbering rules.
- **Species support**: ANARCI includes germline databases for multiple species (human, mouse, rabbit, rhesus macaque, pig, alpaca, dog, cat). The species flag affects germline gene identification accuracy as different species have different germline repertoires.

## Pitfalls

- **Using constant regions in input**: Including constant domain sequences causes misnumbering because ANARCI aligns input to the variable domain template. Sequences with CH1 or constant regions will be assigned incorrect positions or rejected entirely — always trim to Fv (framework + CDR regions only).
- **Mismatching domain type**: Specifying `--domain IG` for TCR sequences or `--domain TCR` for antibody sequences produces unusable results, as the numbering rules and CDR definitions differ between immunoglobulins and T-cell receptors — always verify the domain flag matches your sequence type.
- **Omitting species for germline assignment**: Running ANARCI without the `--species` flag reduces germline assignment accuracy since the tool defaults to a generic database. Without species specification, V/D/J gene identification may be incomplete or misassigned, producing unreliable germline calls.
- **Inconsistent sequence quality**: Sequences with ambiguous bases (N), excessive X characters, or frameshift errors cause ANARCI to fail silently or produce incomplete output files. Ensure input sequences are high-quality and curated before analysis.
- **Wrong numbering scheme for downstream tools**: Different downstream tools expect different numbering schemes. Using Kabat when the analysis tool expects Chothia (or vice versa) causes CDR boundaries to be misaligned by several residues — verify scheme compatibility with downstream applications.

## Examples

### Number antibody sequences using the Chothia scheme
**Args:** `--sequence GATCGACTAGCTAGCTAAGTGCTACAATCAGTCGATCGAT --domain IG --numbering chothia --species human`
**Explanation:** This assigns numbers to the input antibody sequence according to the Chothia numbering system, identifying framework and CDR positions for human IG sequences.

### Number TCR sequences using the IMGT scheme
**Args:** `--sequence ATCGATCGATCGTAGCTAGCATGCTGATCGTAGCTAGCT --domain TCR --numbering imgt --species human`
**Explanation:** This applies IMGT numbering to a T-cell receptor sequence, aligning it to the TCR variable domain template for human TCRs.

### Output numbered sequences to a specific FASTA file
**Args:** `--sequence GATCGTAGT --domain IG --numbering kabat --species human --outfile numbered.fasta --sequence_output fasta`
**Explanation:** This writes the numbered sequence output to a file in FASTA format for downstream analysis or visualization.

### Run ANARCI with CSV output for scripting
**Args:** `--sequence GATCGTAGT --domain IG --numbering chothia --species human --csv all`
**Explanation:** This generates CSV output containing all numbered sequence data, enabling automated parsing in pipeline scripts.

### Analyze multiple sequences from a FASTA file
**Args:** `--sequencefile antibodies.fasta --domain IG --numbering martin --species mouse --outfile results.csv`
**Explanation:** This processes multiple antibody sequences from an input file, numbering them according to the Martin scheme and assigning mouse germline genes.

### Generate only germline assignments without detailed numbering
**Args:** `--sequence ATGCGCTA --domain IG --numbering chothia --species rabbit --germline`
**Explanation:** This focuses germline gene identification for rabbit antibody sequences, outputting V/D/J gene assignments without full sequence numbering.

### Use EU numbering for therapeutic antibody analysis
**Args:** `--sequence ATCGATCGATCGTAGCT --domain IG --numbering eu --species human`
**Explanation:** This applies the EU (World Health Organization) numbering scheme commonly used for therapeutic antibody characterization and comparability.

### Hide intermediate output to reduce file clutter
**Args:** `--sequence ATGCTAGCT --domain IG --numbering kabat --species human --hide csv`
**Explanation:** This runs ANARCI while suppressing CSV intermediate output, producing only the numbered sequences in FASTA format.