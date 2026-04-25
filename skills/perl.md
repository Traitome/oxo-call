---
name: perl
category: programming
description: Perl interpreter; scripting language widely used in legacy bioinformatics pipelines, text processing, BioPerl, and system automation
tags: [perl, bioperl, cpan, scripting, text-processing, one-liner, regex, bioinformatics]
author: oxo-call built-in
source_url: "https://perldoc.perl.org/"
---

## Concepts
- Perl is invoked with `perl`; scripts have a `.pl` extension and typically start with `#!/usr/bin/env perl` or `#!/usr/bin/perl`.
- Module search path `@INC` lists directories where Perl looks for modules; inspect with `perl -V` or `perl -e 'print join "\n", @INC'`.
- CPAN (Comprehensive Perl Archive Network) provides thousands of modules; install with `cpanm` (cpanminus) or `cpan`.
- User-local module installation (no root needed) via `local::lib`: modules go to `~/perl5/`; add `eval $(perl -I ~/perl5/lib/perl5 -Mlocal::lib)` to shell profile.
- `~/.cpan/` — default CPAN build and source directory; configure with `~/.cpan/CPAN/MyConfig.pm`.
- BioPerl lives in the `Bio::` namespace; install with `cpanm Bio::Perl` or via conda (`conda install -c bioconda perl-bioperl`).
- One-liners: `perl -e 'code'` executes an expression; `-n` wraps code in a `while (<>)` loop; `-p` is like `-n` but prints `$_` each iteration.
- `-i[extension]` edits files in-place (like `sed -i`); `-i.bak` makes backup copies before editing.
- `-w` enables warnings; `use strict; use warnings;` is the best-practice preamble for scripts.
- Regular expressions: Perl's regex engine is highly expressive; use `=~` for matching, `s///` for substitution, `split /pattern/, $str` for splitting.
- `$ENV{VAR}` accesses environment variables inside Perl scripts.
- Perl is included in most Unix/Linux distributions at `/usr/bin/perl`; check with `perl --version`.
- `-T` enables taint mode for security; recommended for CGI scripts and web applications.
- `-c` checks script syntax without executing; useful for pre-deployment validation.
- `BEGIN` and `END` blocks execute code before/after the main script logic.

## Pitfalls
- Not using `use strict; use warnings;` leads to hard-to-debug variable typos and undefined-value warnings.
- On macOS, the system perl at `/usr/bin/perl` is very old (5.18); use a Homebrew or perlbrew-managed version for modern bioinformatics.
- `perl -i` (without an extension) edits files in-place with no backup; always test with `-n` first.
- Module paths: if a bioinformatics script fails with "Can't locate Module.pm", add its parent dir to PERL5LIB: `export PERL5LIB=/path/to/lib:$PERL5LIB`.
- `cpan` versus `cpanm`: prefer `cpanm` (App::cpanminus) for easier, faster installs; `cpan` asks many interactive questions during first run.
- Encoding issues: legacy bioinformatics Perl scripts may break with wide characters; pass `use open ':std', ':utf8';` or `binmode STDOUT, ':utf8';`.
- `while (<STDIN>)` holds the entire line including `\n`; always `chomp` before string comparisons.
- `-T` taint mode can cause unexpected failures with external data; test thoroughly before enabling in production.
- `-c` only checks syntax; it does not catch runtime errors or logical bugs.

## Examples

### run a Perl script
**Args:** `script.pl input.txt`
**Explanation:** perl command; script.pl Perl script; input.txt script argument; executes script.pl with input.txt as argument; script must be readable

### print the Perl version and module search paths
**Args:** `-V`
**Explanation:** perl command; -V prints compile-time configuration; shows @INC paths and Perl version; useful for debugging module-not-found errors

### one-liner: print lines matching a pattern
**Args:** `-ne 'print if /^>/' sequences.fasta`
**Explanation:** perl command; -n loops over each line; -e 'print if /^>/' expression to print; sequences.fasta input FASTA; prints lines starting with '>' (FASTA headers); equivalent to grep but with full Perl regex

### one-liner: extract specific columns from a TSV
**Args:** `-lane 'print join("\t", @F[0,2,4])' data.tsv`
**Explanation:** perl command; -l adds newline handling; -a splits on whitespace into @F; -n loops over lines; -e 'print join("\t", @F[0,2,4])' expression; data.tsv input TSV; prints columns 1, 3, and 5 (0-indexed)

### in-place substitution (edit file directly)
**Args:** `-i.bak -pe 's/chr/Chr/g' genome.fa`
**Explanation:** perl command; -i.bak saves backup as genome.fa.bak; -p prints each modified line; -e 's/chr/Chr/g' substitution expression; genome.fa input FASTA; s/// replaces all occurrences

### count FASTA sequences in a file
**Args:** `-ne '$c++ if /^>/; END { print "$c sequences\n" }' input.fa`
**Explanation:** perl command; -n loops over lines; -e '$c++ if /^>/; END { print "$c sequences\n" }' expression; input.fa input FASTA; increments counter for each header line; END block executes after all input is processed

### install a module via CPAN one-liner
**Args:** `-MCPAN -e 'CPAN::Shell->install("Bio::SeqIO")'`
**Explanation:** perl command; -MCPAN loads CPAN module; -e 'CPAN::Shell->install("Bio::SeqIO")' expression; installs Bio::SeqIO via the CPAN shell; prefer `cpanm Bio::SeqIO` (cpanminus) for a simpler, non-interactive install

### set up a local user-space Perl module directory
**Args:** `-Mlocal::lib`
**Explanation:** perl command; -Mlocal::lib loads local::lib module; prints shell commands to configure ~/perl5/ as a local lib dir; eval the output: eval $(perl -Mlocal::lib)

### check if a required module is installed
**Args:** `-MBio::SeqIO -e 1`
**Explanation:** perl command; -MBio::SeqIO loads module; -e 1 minimal expression; exits 0 if Bio::SeqIO loads successfully, non-zero otherwise; useful in CI/pipeline pre-checks

### run a bioinformatics script with a custom library path
**Args:** `-I /path/to/bioperl-lib script.pl input.fasta`
**Explanation:** perl command; -I /path/to/bioperl-lib adds path to @INC; script.pl Perl script; input.fasta script argument; useful when BioPerl is not system-installed but located in a local directory

### check script syntax without executing
**Args:** `-c script.pl`
**Explanation:** perl command; -c syntax check flag; script.pl Perl script; validates syntax and exits; prints any syntax errors; useful for CI/CD pipelines and pre-deployment checks

### run with taint mode enabled
**Args:** `-T script.pl`
**Explanation:** perl command; -T taint mode flag; script.pl Perl script; enables taint mode for security; marks external data as "tainted"; must be sanitized before system calls

### execute code in BEGIN block
**Args:** `-e 'BEGIN { print "Initializing...\n" } print "Main code\n"'`
**Explanation:** perl command; -e 'BEGIN { print "Initializing...\n" } print "Main code\n"' expression; BEGIN block runs before main script; useful for module loading, initialization, or setting up environment

### execute code in END block
**Args:** `-e 'END { print "Cleaning up...\n" } print "Main code\n"'`
**Explanation:** perl command; -e 'END { print "Cleaning up...\n" } print "Main code\n"' expression; END block runs after main script completes; useful for cleanup, logging, or resource release

### process multiple files with one-liner
**Args:** `-ne 'print if /pattern/' file1.txt file2.txt file3.txt`
**Explanation:** perl command; -n loops over lines; -e 'print if /pattern/' expression; file1.txt file2.txt file3.txt input files; processes multiple files sequentially; @ARGV contains filenames; <> reads from each file in turn
