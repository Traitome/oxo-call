---
name: angsd
category: population-genomics
description: Population genomics analyses from genotype likelihoods, avoiding hard genotype calling
tags: [population-genomics, genotype-likelihoods, bam, fst, thetas, sfs, snp, d-statistics, abba-baba, association]
author: oxo-call built-in
source_url: "http://www.popgen.dk/angsd/index.php/ANGSD"
---

## Concepts

- ANGSD works directly from BAM files using genotype likelihoods (-GL) rather than called genotypes, which is better for low-coverage data where hard genotype calls are unreliable.
- -GL sets the genotype likelihood model: 1=SAMtools (default, fastest), 2=GATK, 3=SOAPsnp, 4=SYK, 5=phys. -GL 1 is sufficient for most analyses.
- -doMajorMinor determines how major/minor alleles are inferred: 1=from GL, 2=from allele counts, 4=use reference as major (-ref), 5=use ancestral as major (-anc). Most analyses require this to be set.
- The SFS workflow is multi-step: (1) `angsd -doSaf` computes per-site allele frequency likelihoods, (2) `realSFS` estimates the SFS via EM, (3) `realSFS fst` computes Fst, (4) `thetaStat` computes diversity statistics.
- -doSaf modes: 1=integrate over possible minor alleles (standard), 2=incorporate inbreeding, 4=from genotype posteriors (beagle input), 5=condition on minor from -doMajorMinor.
- -doGeno calls genotypes: 1=write major/minor, 2=encode as 0/1/2, 4=write genotypes (AA/AC etc.), 8=posterior probabilities. Values can be combined by summing (e.g., -doGeno 3 = 1+2).
- -doMaf estimates allele frequencies: 1=fixed major and minor, 2=fixed major unknown minor, 4=from genotype probabilities, 8=allele counts based.
- -doAbbababa performs ABBA-BABA (D-statistics) tests for introgression/gene flow between populations. Requires -anc (outgroup/ancestral FASTA) or -useLast to use last individuals as outgroup.
- -doAsso performs association studies: 1=Frequency Test, 2=Score Test, 4=Latent genotype model, 5=Hybrid score+latent, 6=Dosage regression.
- -doFasta generates consensus FASTA from BAM: 1=random base, 2=most common base, 4=IUPAC codes. Requires -doCounts 1.
- Quality filters should always be applied: -minQ 20 (base quality), -minMapQ 30 (mapping quality), -remove_bads 1 to exclude reads with flag & 512.
- Companion tools: `realSFS` (SFS estimation, Fst), `thetaStat` (theta/Tajima sliding windows), `NGSadmix` (admixture from GL), `ibs` (identity-by-state).
- -nThreads sets ANGSD threads; -P (in realSFS/thetaStat) sets threads for companion tools; use both for full parallelization.

## Pitfalls

- ANGSD has no subcommands. ARGS starts directly with flags like `-bam`, `-GL`, `-doMaf`. The first flag is always a dash-prefixed option — never a bare word.
- Not filtering by mapping quality (-minMapQ) allows reads mapped to repetitive regions to bias allele frequency estimates. Always use -minMapQ 30 or higher.
- Using -doSaf without setting -anc (ancestral FASTA) computes a folded SFS by default; specify -anc for unfolded polarized analyses.
- ANGSD does not check that BAM files are sorted and indexed — unsorted/unindexed BAMs produce incorrect or empty output silently.
- -SNP_pval threshold matters: too lenient includes invariant sites in SFS; too strict removes low-frequency variants. Use 1e-6 for strict SNP calling, 1e-2 for more inclusive analyses.
- The -nThreads flag sets ANGSD threads; -P sets samtools threads in companion tools. Forgetting both makes analysis very slow.
- -bam takes a text file listing BAM paths (one per line), not a single BAM file. For a single BAM, use -i instead.
- -doGlf 2 outputs BEAGLE format (.beagle.gz); -doGlf 1 outputs binary GLF format. BEAGLE is needed for PCAngsd and NGSadmix.
- When running -doSaf for multiple populations, each population must be run separately with its own BAM list, then combined with realSFS.
- -remove_bads 1 is enabled by default but worth confirming; without it, reads marked as bad (flag & 512) are included.
- ANGSD outputs compressed files (.mafs.gz, .beagle.gz, etc.) — use zcat or gunzip -c to inspect.

## Examples

### compute genotype likelihoods and allele frequencies for a set of BAMs
**Args:** `-bam bam_list.txt -GL 1 -doGlf 2 -doMaf 1 -doMajorMinor 1 -SNP_pval 1e-6 -minMapQ 30 -minQ 20 -nThreads 16 -out output`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doGlf 2 outputs BEAGLE format for downstream PCAngsd; -doMaf 1 with -doMajorMinor 1 infers alleles from GL; -SNP_pval 1e-6 filters to likely variable sites; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -nThreads 16 parallel processing; -out output prefix

### compute per-site allele frequency spectrum for a single population
**Args:** `-bam pop1_bams.txt -GL 1 -doSaf 1 -anc ancestral.fasta -minMapQ 30 -minQ 20 -nThreads 16 -out pop1`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doSaf 1 computes per-site allele frequency likelihoods; -anc provides ancestral allele state for unfolded SFS; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -nThreads 16 parallel processing; -out pop1 output prefix; outputs pop1.saf.idx

### estimate 1D site frequency spectrum from doSaf output
**Args:** `realSFS pop1.saf.idx -P 16 > pop1.sfs`
**Explanation:** realSFS uses EM algorithm to estimate the SFS from .saf.idx files; -P 16 sets parallel threads; output is a space-separated SFS vector

### estimate Watterson's theta and Tajima's D in sliding windows
**Args:** `-bam pop1_bams.txt -GL 1 -doSaf 1 -doThetas 1 -pest pop1.sfs -anc ancestral.fasta -minMapQ 30 -minQ 20 -out pop1_thetas`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doSaf 1 allele frequency likelihoods; -doThetas 1 computes diversity statistics; -pest (prior SFS from realSFS) required for theta estimation; -anc ancestral FASTA; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -out pop1_thetas output prefix; outputs .thetas.idx; then use `thetaStat do_stat pop1_thetas.thetas.idx -win 5000 -step 1000` for sliding windows

### compute Fst between two populations using 2D SFS
**Args:** `realSFS pop1.saf.idx pop2.saf.idx -P 16 > pop1_pop2.2dsfs && realSFS fst index pop1.saf.idx pop2.saf.idx -sfs pop1_pop2.2dsfs -fstout pop1_pop2`
**Explanation:** two-step: compute 2D SFS with realSFS (-P 16 threads), then index Fst (-sfs uses 2D SFS, -fstout output prefix); use `realSFS fst stats pop1_pop2.fst.idx` for genome-wide Fst

### call genotypes with posterior probabilities
**Args:** `-bam bam_list.txt -GL 1 -doGeno 4 -doMaf 1 -doMajorMinor 1 -doPost 1 -minMapQ 30 -minQ 20 -nThreads 16 -out genotypes`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doGeno 4 writes genotypes as AA/AC/AG etc.; -doMaf 1 allele frequency estimation; -doMajorMinor 1 infers alleles from GL; -doPost 1 uses frequency as prior for posterior; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -nThreads 16 parallel processing; -out genotypes output prefix; outputs .geno.gz

### perform ABBA-BABA D-statistics test for introgression
**Args:** `-bam bam_list.txt -GL 1 -doAbbababa 1 -anc outgroup.fasta -rmTrans 1 -blockSize 5000000 -nThreads 16 -out dstat`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doAbbababa 1 runs D-statistics; -anc provides outgroup for allele polarization; -rmTrans 1 removes transitions (common for CpG bias); -blockSize 5000000 block size for bootstrapping; -nThreads 16 parallel processing; -out dstat output prefix; outputs .abbababa file

### run association study using Score Test
**Args:** `-bam bam_list.txt -GL 1 -doAsso 2 -doMaf 1 -doMajorMinor 1 -y phenotypes.txt -minMapQ 30 -minQ 20 -nThreads 16 -out association`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doAsso 2 uses Score Test; -doMaf 1 allele frequency estimation; -doMajorMinor 1 infers alleles from GL; -y supplies phenotype file (one value per individual); -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -nThreads 16 parallel processing; -out association output prefix; outputs association statistics

### generate consensus FASTA from BAM alignment
**Args:** `-i sample.bam -GL 1 -doFasta 2 -doCounts 1 -minMapQ 30 -minQ 20 -out consensus`
**Explanation:** -i sample.bam single BAM input; -GL 1 SAMtools likelihood model; -doFasta 2 uses the most common base at each position; -doCounts 1 required for base counting; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -out consensus output prefix; outputs .fa

### compute genotype likelihoods for NGSadmix admixture analysis
**Args:** `-bam bam_list.txt -GL 1 -doGlf 2 -doMajorMinor 1 -doMaf 1 -SNP_pval 1e-6 -minMapQ 30 -minQ 20 -minInd 40 -nThreads 16 -out admix_input`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doGlf 2 outputs BEAGLE format needed by NGSadmix; -doMajorMinor 1 infers alleles from GL; -doMaf 1 allele frequency estimation; -SNP_pval 1e-6 filters to likely variable sites; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -minInd 40 requires data in at least 40 individuals per site; -nThreads 16 parallel processing; -out admix_input output prefix; then run `NGSadmix -beagle admix_input.beagle.gz -K 3 -o admix_result`

### analyze a specific genomic region
**Args:** `-bam bam_list.txt -GL 1 -doMaf 1 -doMajorMinor 1 -r chr1:1000000-2000000 -minMapQ 30 -minQ 20 -out region_output`
**Explanation:** -bam BAM list file; -GL 1 SAMtools likelihood model; -doMaf 1 allele frequency estimation; -doMajorMinor 1 infers alleles from GL; -r chr1:1000000-2000000 restricts analysis to a specific chromosome region; -minMapQ 30 minimum mapping quality; -minQ 20 minimum base quality; -out region_output output prefix; much faster than whole-genome for testing parameters
