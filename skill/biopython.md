---
name: biopython
category: bioinformatics/library
description: A Python library for computational molecular biology providing tools for sequence analysis, structure analysis, bioinformatics algorithms, and parsing of common bioinformatics file formats including FASTA, GenBank, and PDB.
tags: [python, sequence-analysis, bioinformatics, genomics, proteomics]
author: AI-generated
source_url: https://biopython.org
---

## Concepts

- Biopython provides core classes Seq, SeqRecord, and SeqIO as the foundation for biological sequence handling. Seq represents immutable biological sequences with biological methods (transcribe, translate, complement, reverse_complement), while SeqRecord wraps a Seq with metadata including id, name, description, and annotations.

- Bio.SeqIO is the primary interface for reading and writing sequence files in formats including FASTA, FASTQ, GenBank, and PDB. The parse() function returns an iterator yielding SeqRecord objects for sequential processing, while read() expects exactly one record from single-record file formats.

- Sequence alphabets (DNAAlphabet, RNAAlphabet, ProteinAlphabet, GenericAlphabet) determine which biological operations are valid. Attempting operations like complement on a protein sequence raises an error, and mixed alphabets disable certain biological methods.

- The Bio module namespace organizes functionality logically: Bio.Seq for core sequence operations, Bio.SeqIO for file I/O, Bio.Align for alignment analysis, Bio.Blast for parsing BLAST results, Bio.PDB for structural data, and Bio.Entrez for NCBI database queries.

## Pitfalls

- Attempting to modify a Seq object in-place fails because Seq instances are immutable. Operations like .transcribe() and .translate() return new Seq objects rather than mutating the original.

- Using SeqIO.parse() when the file contains exactly one record leads to an EmptyStopIteration error when calling next(). Use SeqIO.read() for single-record formats like GenBank or PDB.

- String operations like .lower() or .upper() on Seq objects convert them to plain Python strings, losing alphabet metadata and breaking subsequent biological methods that depend on alphabet information.

- Alphabet mixing occurs silently when combining sequences with different alphabets, disabling biological methods on the resulting Seq object. Always verify alphabet compatibility before performing sequence operations.

- Forgetting to set Entrez.email before making NCBI queries results in Bio.MedlineLINEError. NCBI requires identification of API users, and the email parameter must be configured before any Entrez query.

## Examples

### Parse a multi-sequence FASTA file and build an indexed dictionary
**Args:** `from Bio import SeqIO; records = {rec.id: rec for rec in SeqIO.parse("sequences.fasta", "fasta")}`
**Explanation:** Dictionary comprehension efficiently indexes all SeqRecord objects by their identifier for O(1) lookup performance.

### Read a single GenBank record and access organism annotation
**Args:** `record = SeqIO.read("sequence.gb", "genbank"); print(record.annotations["organism"]); print(len(record.features))`
**Explanation:** SeqIO.read() correctly handles single-record GenBank files, with organism metadata accessible via the annotations dictionary.

### Perform in-silico transcription of a DNA sequence
**Args:** `from Bio.Seq import Seq; mrna = Seq("ATGCCATGA").transcribe(); print(mrna)`
**Explanation:** The transcribe() method converts DNA to RNA by replacing thymine with uracil across the sequence.

### Translate a DNA sequence with terminal stop codon preservation
**Args:** `protein = Seq("ATGCCATGA").translate(to_stop=False); print(protein)`
**Explanation:** Setting to_stop=False preserves the * stop codon in the translated protein, while True would terminate translation early.

### Generate reverse complement accounting for strand orientation
**Args:** `from Bio.Seq import Seq; from Bio.Alphabet import DNAAlphabet; dna = Seq("ATCGAT", DNAAlphabet()); print(dna.reverse_complement())`
**Explanation:** reverse_complement() correctly handles DNA strand orientation by reversing sequence order and applying complement.

### Parse BLAST XML output and extract alignment information
**Args:** `from Bio.Blast import NCBIXML; records = NCBIXML.parse(open("blast_results.xml")); result = next(records); print(result.alignments[0].title)`
**Explanation:** NCBIXML.parse() streams BLAST XML records sequentially, allowing extraction of hit titles from alignment objects.

### Query NCBI nucleotide database using Entrez
**Args:** `from Bio import Entrez; Entrez.email = "user@example.com"; handle = Entrez.esearch(db="nucleotide", term="BRCA1[Gene] AND human[Organism]"); record = Entrez.read(handle); print(record["IdList"])`
**Explanation:** Entrez.esearch() queries NCBI databases and returns a dictionary containing matching record IDs.

### Calculate pairwise sequence identity from alignment
**Args:** `from Bio.Align import PairwiseAligner; aligner = PairwiseAligner(); aligner.mode = "local"; alignments = list(aligner.align(seq1, seq2)); print(alignments[0].score, alignments[0].aligned)`
**Explanation:** PairwiseAligner with local alignment mode finds optimal local alignments and returns alignment coordinates.

### Write multiple sequences to FASTQ format with quality scores
**Args:** `from Bio import SeqIO; SeqIO.write(records, open("output.fastq", "w"), "fastq")`
**Explanation:** SeqIO.write() exports SeqRecord objects to FASTQ format, preserving quality scores for downstream sequencing analysis.

### Access specific GenBank feature by type
**Args:** `record = SeqIO.read("sequence.gb", "genbank"); cds_feature = [f for f in record.features if f.type == "CDS"][0]; print(cds_feature.location, cds_feature.qualifiers["gene"])`
**Explanation:** List comprehension filters GenBank features by type, enabling extraction of coding sequence locations and gene qualifiers.